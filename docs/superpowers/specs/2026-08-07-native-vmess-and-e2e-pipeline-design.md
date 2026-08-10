# Native VMESS + Unified E2E Testing Pipeline — Design

Date: 2026-08-07
Status: approved in conversation; written up for review
Scope: `crates/xray-tui-native`

## Goal

1. Implement a second native protocol: **VMess over TCP, modern AEAD header, `none` payload security** — the simplest correct VMess slice, interoperable with xray-core and sing-box.
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
  TLS + TCP transport, security payload `none` (VMess).
- `client_trust` installs the harness CA: `security::tls::set_test_config(test_client_config(&certs.ca_der))` (existing pattern).
- `server_config(xray | sing-box)` hand-rolls flat inbound JSON: xray `inbounds/outbounds` shape, sing-box `inbounds/outbounds` shape. **Deliberately not** the proto `inject_to` builders — those emit outbound blocks for profile configs, wrong shape for inbound test servers.

### VMess protocol — `src/protocol/vmess/` (mirrors vless layout)

```
src/protocol/vmess/
  mod.rs    — connect(): build params → codec request → wrap stream
  header.rs — AEAD request/response header codec
  stream.rs — VMessStream: peel response header on first read
```

Wire contract implemented **verbatim from `thirdparty/Xray-core/proxy/vmess/encoding/`** (read the Go source at implementation time, not memory):

- cmdKey = HMAC-MD5 KDF over md5(uuid·bytes), salted with the standard VMess constant `c48619fe-8f02-49e0-b9e9-edf763e17e21` (the milestone-M4 note already names it).
- Request: `version(1)=1` | `requestBodyIV(16)` | `requestBodyKey(16)` | AES-128-GCM encrypted header body — command 0x01 (TCP), port (BE), addr type, addr, random(4), timestamp(4).
- Payload: security `none` → raw bytes after the header (AEAD header mandatory on modern cores regardless).
- Response: the server's response header (`version(1)=0` + length + V(4)/IV(16)/Key(16), exact framing from Go) is **peeled on first read** — same pattern as VLESS, which proved robust against both cores' send-timing variants.
- `protocol/mod.rs`: replace `ProtocolConfig::Vmess(_) => not_impl("vmess")` with `vmess::connect(...)`. `ConnectShape` already routes VMess to `TcpStream` (exhaustive match — confirm at implementation time).

### Dependencies

- **Runtime (unconditional, production paths):** `aes-gcm`, `hmac` (VMess header is mandatory for any VMess client → not test-gated). `md-5` already present.
- **Feature-gated (`native-e2e`):** `serde_json`, `tempfile`, `rcgen`, `tiny_http` move from dev-deps to `[dependencies]` with `optional = true`, enabled by the `native-e2e` feature (the e2e module lives in the crate, so dev-deps are unavailable to it).
- `aes`/`cfb-mode` deferred until a CFB security mode is added; `chacha20-poly1305` likewise.
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

- VMess payload securities beyond `none` (aes-128-gcm / chacha20-poly1305 / auto / zero) and legacy alterId/AEAD-less clients (modern cores reject them).
- Non-TCP transports for either protocol; VMess over mKCP/WS/QUIC.
- Version-conditioned server configs.
- Making the sandbox cargo-test-harness stall go away (environmental; bounded retries contain it).

## Risks

- **Wire details from Go source are authoritative but dense** (option flags, KDF chain). Mitigation: implement header-by-header with the encoding.go file open; e2e against real cores is the final arbiter.
- **Response-header peel timing** differs between xray/sing-box VMess servers — the VLESS pattern already handles this; the parse must not consume relayed bytes (peel = exact header length only).
- **sing-box VMess `none` security acceptance** — verify server-side config field during implementation; if sing-box rejects explicit `none`, fall back to omitting security where the server default matches (document the choice in the spec's risks section if it happens).
