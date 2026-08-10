# Native VMESS + Unified E2E Testing Pipeline — Design

Date: 2026-08-07
Status: approved in conversation; written up for review
Scope: `crates/xray-tui-native`

## Goal

1. Implement a second native protocol: **VMess over TCP, modern AEAD header, AES-128-GCM payload security** — the smallest VMess slice both cores accept, interoperable with xray-core and sing-box.
2. Replace the ad-hoc per-protocol e2e tests with a **unified, declarative testing pipeline**: one driver executing the same 7 steps for any protocol/core pair, with each protocol providing a testing-enablement trait impl behind `feature = "native-e2e"`.
3. VMess is the proof-of-concept: it and the existing VLESS implementation both implement the trait, and both e2e suites run through the pipeline.

## Current State

- `crates/xray-tui-native` has `ConnectShape` taxonomy, chain fold (`connect_chain` → transport → security → protocol), working VLESS (header codec + peel-on-first-read response header stream), and stubs for the other 23 TCP-stream kinds including VMess (`protocol/vmess/mod.rs` placeholder with milestone-M4 note).
- E2E helpers live in `tests/common/mod.rs`: `EchoServer`, `Certs` (rcgen), `free_port`, `core_bin`, `spawn_core` + readiness, bounded `probe` (5s write + 5s read, `(0,"")` on failure).
- Tests: `tests/vless_singbox.rs`, `tests/vless_xray.rs` — each repeatedly spawns core + connects with retry-on-failure.
- Example `examples/native_connect_check.rs` — harness-free manual interop check (proved the sandbox harness flake). **Deleted** — the pipeline becomes the single testing entry.
- Deps: `md-5` present (VLESS paranoid prefix hash); NO `aes`/`aes-gcm`/`hmac`/`cfb` yet.

## Architecture

### Unified pipeline — `src/e2e/` (crate module, `#[cfg(feature = "native-e2e")]`)

```
src/e2e/
  mod.rs      — E2eCase trait, run() driver, E2eExpect, ServerEnv
  core.rs     — CoreKind, CoreUnderTest resolution + version probe
  harness.rs  — EchoServer, Certs, free_port, spawn_core, probe (migrated from tests/common)
```

**The trait** (`mod.rs`):

```rust
pub struct E2eExpect { pub status: u16, pub body: String }

pub struct ServerEnv<'a> {
    pub port: u16,
    pub certs: &'a Certs,
    pub tmp: &'a std::path::Path,
}

pub trait E2eCase {
    /// protocol identity for logs (e.g. "vless/tcp/tls")
    fn label(&self) -> &'static str;
    /// Full server config JSON for the given binary core.
    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String;
    /// Native client connect params (sni=localhost, target=echo).
    fn client_params(&self, port: u16, target: std::net::SocketAddr) -> NativeConnectParams;
    /// Expected probe outcome.
    fn expected(&self) -> E2eExpect;
    /// Install client trust for this case (default: no-op).
    fn client_trust(&self, certs: &Certs) { let _ = certs; }
}
```

**The driver** (`run`):

```
pub async fn run<C: E2eCase>(case: &C, core: CoreUnderTest) -> Result<(), String>
```

Executes, in order, with bounded waits everywhere (a hang is a bug):
1. `harness::spawn_echo()` — tiny_http thread, body from `case.expected().body`.
2. `tempfile::tempdir()` + `harness::generate_certs()`.
3. `case.server_config(core.kind, &ServerEnv{...})` → write `config.json` in tmp.
4. `harness::spawn_core(bin, json, port)` + TCP-readiness wait (bounded).
5. `case.client_trust(&certs)`; `connect(case.client_params(port, echo_addr)).await`.
6. `harness::probe(&mut tunnel)` — 5s write / 5s read, fails fast.
7. Assert `status == expected.status && body == expected.body`.

**Flake policy**: `run` retries the flaky segment, 3 attempts. Attempt `n` = **fresh core spawn + fresh connect + fresh probe** (the observed sandbox flake lives in the core TLS path and in connect/waker stalls, so both are re-created per attempt). The echo server, tempdir, and certs are created once and reused — our own code, stable, never the flaky party. Failures logged via `eprintln!("[e2e] {label}: attempt {n} ...")`. Exhausted attempts → `Err(String)` with the last `(status, body)`/error detail; the test `.expect()`s it.

**Core resolution** (`core.rs`):

```rust
pub enum CoreKind { Xray, SingBox }

pub struct CoreUnderTest { pub kind: CoreKind, pub bin: PathBuf, pub version: String }

impl CoreUnderTest {
    pub fn resolve(kind: CoreKind, expected_version: &str) -> Result<Self, String>;
}
```

- Bin path from `XRAY_TUI_CORE_BIN_DIR/<xray|sing-box>` (existing convention); absent → `Err` (tests never skip silently when binaries are required — the feature gate is the skip).
- Version probe: `-version` for xray, `version` for sing-box (fallback: try both flags); capture stdout; store.
- **Sanity**: `version.contains(expected_version)` else `Err` ("want 1.13.16, got ..."). Metadata-only — no config branching on version.

### Protocol cases (feature-gated impls)

```
src/protocol/vless/e2e.rs   #[cfg(feature = "native-e2e")] impl E2eCase for VlessCase
src/protocol/vmess/e2e.rs   #[cfg(feature = "native-e2e")] impl E2eCase for VmessCase
```

- `VlessCase` / `VmessCase` — `Default` unit structs; hardcoded shared UUID
  `00000000-0000-0000-0000-000000000000`, sni `localhost`, alpn `http/1.1`,
  TLS + TCP transport, payload security `aes-128-gcm` (VMess — see the
  security-scope decision: xray-core 26.x validates only AES-128-GCM=3 /
  CHACHA20=4 in the body stream; `none`/`zero`/`auto` are refused server-side,
  verified in `thirdparty/Xray-core/proxy/vmess/encoding/server.go`).
- `client_trust` installs the harness CA: `security::tls::set_test_config(test_client_config(&certs.ca_der))` (existing pattern).
- `server_config(xray | sing-box)` hand-rolls flat inbound JSON: xray `inbounds/outbounds` shape, sing-box `inbounds/outbounds` shape. **Deliberately not** the proto `inject_to` builders — those emit outbound blocks for profile configs, wrong shape for inbound test servers.

### VMess protocol — `src/protocol/vmess/` (mirrors vless layout)

```
src/protocol/vmess/
  mod.rs    — connect(): build params → codec request → wrap stream
  header.rs — AEAD request/response header codec
  stream.rs — VMessStream: peel response header on first read
```

Wire contract implemented **verbatim from `thirdparty/Xray-core/proxy/vmess/encoding/`** — Go source is the authority; `thirdparty/shoes/src/vmess/` (complete Rust VMess, aws-lc-rs AEAD) and sing-box's vendored `sing-vmess` (`~/go/pkg/mod/github.com/sagernet/sing-vmess@v0.2.8-*/`) are cross-checks.

- `cmdKey = md5(uuid_bytes ‖ "c48619fe-8f02-49e0-b9e9-edf763e17e21")` — confirmed in `common/protocol/id.go` `NewID` (`uuid.Bytes()` = the 16 raw UUID bytes).
- Session (client, per connection): random `requestBodyIV(16)` + `requestBodyKey(16)` + random `responseHeader(1)` byte. `responseBodyKey = sha256(requestBodyKey)[:16]`, `responseBodyIV = sha256(requestBodyIV)[:16]`.
- AuthID (per request): AES-128-ECB(key = `KDF16(cmdKey, "AES Auth ID Encryption")`, block = `[unix_ts: 8B BE][random: 4B][crc32-ieee of the 12 bytes: 4B BE]`) — 16 bytes out.
- `KDF(key, path...)` = nested HMAC-SHA256: `msg = HMAC-SHA256("VMess AEAD KDF", key)`, then for each path string `v`: `msg = HMAC-SHA256(v, msg)`; `KDF16` = first 16 bytes. (Verified against Go `aead/kdf.go`.)
- **Request header body** (pre-seal, command TCP): `version(1)=1 | requestBodyIV(16) | requestBodyKey(16) | responseHeader(1) | option(1)=0 | [padding_len(4 bits)|security(4 bits)=0x03](1) | 0(1) | command(1)=0x01 | addr(type|addr) | port(2 BE) | [padding] | FNV-1a32(4 BE over all preceding bytes)`. Addr types: 1=IPv4, 2=domain, 3=IPv6 (`payload.go`).
- **Seal** (`aead/encrypt.go`): `nonce(8B random)`; `lenAEAD = AES-128-GCM(KDF16(cmdKey,"VMess Header AEAD Key_Length",authID,nonce), KDF(cmdKey,"VMess Header AEAD Nonce_Length",authID,nonce)[:12], 2B BE header length, aad=authID)` (18B); `payloadAEAD = AES-128-GCM(KDF16(cmdKey,"VMess Header AEAD Key",authID,nonce), KDF(...,[:12]), body, aad=authID)`. Wire: `authID(16) | lenAEAD(18) | nonce(8) | payloadAEAD`.
- **Payload records** (both directions, `aes-128-gcm`): each record = `2B BE length | AES-128-GCM(key=sessionKey, nonce)` where request-side session = `requestBodyKey`/`requestBodyIV`, response-side = `responseBody*`; nonce = session IV with the FIRST TWO bytes overwritten by a BE chunk counter starting at 0 (`GenerateChunkNonce`), used 12 bytes. No authenticated length, no chunk masking (option=0).
- **Response header**: after the request header, the server replies `lenAEAD(18) | payloadAEAD` where `lenAEAD = AES-128-GCM(KDF16(respKey,"AEAD Resp Header Len Key"), KDF(respIV,"AEAD Resp Header Len IV")[:12], 2B len, aad=nil)` and the payload (4B: `responseHeader | option | 0 | 0`) is `AES-128-GCM(KDF16(respKey,"AEAD Resp Header Key"), KDF(respIV,"AEAD Resp Header IV")[:12], ..., aad=nil)`. Client verifies byte 0 == its random `responseHeader` (echo check), then body is the response-side record stream (handled by the tunnel).
- `vmess::connect` (in `protocol/mod.rs`, replacing `not_impl`): write request body+seal, **read + verify the response header eagerly** (bounded by `timeouts::PROTOCOL`), return `VmessClientStream` (decrypts response records on read, encrypts request records on write). `ConnectShape` already routes VMess to `TcpStream`.

### Dependencies

- **Runtime (unconditional, production paths):** `aes` (ECB for AuthID + block ops), `aes-gcm` (header + record AEAD), `hmac` (KDF + md5-HMAC chains), `sha2` (responseBodyKey/IV), `crc32fast` (AuthID checksum), `md-5` (cmdKey — already present). VMess's AEAD header + records are mandatory for any VMess client → not test-gated.
- **Feature-gated (`native-e2e`):** `serde_json`, `tempfile`, `rcgen`, `tiny_http` move from dev-deps to `[dependencies]` with `optional = true`, enabled by the `native-e2e` feature (the e2e module lives in the crate, so dev-deps are unavailable to it).
- `chacha20-poly1305` (body security) and legacy CFB modes deferred.
- `examples/native_connect_check.rs` deleted; `tiny_http`/`rcgen`/`tempfile` disappear from dev-deps.

## Data Flow (one attempt)

```
echo thread (tiny_http, port E)        core subprocess (VLESS|VMess + TLS, port P)
       ▲ probe GET via tunnel                    ▲ config.json from server_config()
       │                                         │
test ── run() ── steps 1-4 ── connect() ── transport tcp → security tls → protocol vmess
       └ 160-byte HTTP response; assert status+body; retry ×3 on any failure
```

## Error Handling

- `run` returns `Result<(), String>`; every step's error is folded into the attempt detail; eprintln per attempt; final Err carries the last failure.
- All network waits bounded (probe 5s/5s; readiness 10s; connect's existing dial/TLS/protocol timeouts).
- Version sanity mismatch refuses to run (fail fast, no flaky half-tests).
- Sandbox harness flake (observed all-futex waker stall under `cargo test`) — the 3× full-restart policy bounds it: worst case ~60s then a loud, specific failure. Not silent.

## Testing

- **Unit (native crate):** VMess header codec — roundtrip decode of our own encode; structural asserts (lengths, version bytes, timestamp freshness, auth-id nonce uniqueness). Golden Go-generated vectors IF a `go` toolchain is available during implementation; otherwise interop e2e is the correctness proof.
- **E2E (`--features native-e2e`, `XRAY_TUI_CORE_BIN_DIR`):**
  - `tests/vless.rs` — VLESS × {xray, sing-box} — migrated, declarative.
  - `tests/vmess.rs` — VMess `none` × {xray, sing-box} — new.
  - Each test: `e2e::run(&Case::default(), CoreUnderTest::resolve(kind, version)?).await.expect("vmess/xray")`.
- Manual: full workspace `cargo test`, `cargo clippy --all-targets`, `cargo fmt -- --check`.

## Out of Scope

- VMess payload securities beyond `aes-128-gcm` (chacha20-poly1305, legacy alterId/AEAD-less clients — modern cores reject or never accept them). Config values `none`/`zero`/`auto` on the native client error loudly (xray-core 26.x cannot serve them).
- Non-TCP transports for either protocol; VMess over mKCP/WS/QUIC.
- Version-conditioned server configs.
- Making the sandbox cargo-test-harness stall go away (environmental; bounded retries contain it).

## Risks

- **Wire details from Go source are authoritative but dense** (option flags, KDF chain). Mitigation: implement header-by-header with the encoding.go file open; every KDF step has a committed golden vector (python-computed from the Go semantics); e2e against real cores is the final arbiter.
- **xtls header-body dialect vs sing-vmess**: the xtls request-body layout (responseHeader/option/padding/FNV-1a) must be accepted by sing-box's server too. sing-vmess is a reimplementation; if it rejects the xtls layout, the body generator adapts to the dialect sing-vmess parses (fields are additive — expected to accept; both cores' e2e prove it).
- **Echo-back mismatch attack surface**: response-header byte 0 must match the client's random byte; verify and fail loudly (catches stream misalignment early).
