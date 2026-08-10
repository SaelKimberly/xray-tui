# Design: E2E Variant Hardening — VMESS chacha20-poly1305 PoC

Date: 2026-08-10
Status: Approved (brainstorming, user approval 2026-08-10)
Plan: `docs/superpowers/plans/2026-08-10-native-e2e-variant-hardening.md` (after spec review)

## Problem

The e2e facility (`crates/xray-tui-native/src/e2e/`, feature `native-e2e`) models one
struct per full scenario: `E2eCase` bakes protocol + transport + payload security +
both core dialects into a unit struct (`VlessCase`, `VmessCase`). A protocol with
multiple supported security/transport variants would explode into near-identical
struct copies: the chacha20-poly1305 VMESS case differs from the aes-128-gcm case by
exactly two config strings and a label, yet would duplicate the server JSON (two core
dialects), the client config JSON, trust wiring, and expectations.

Secondary pain points to fix in the same stroke:

1. Server config: full per-core JSON inside one `server_config` fn per case — variant
   delta is 2 lines, duplication is ~80.
2. Client params: whole `ProtocolConfig` JSON rebuilt per case.
3. Label: flat string, no structured identity.
4. Tests: hand-written per (case × core); version constants duplicated per test file.
5. No per-variant core gate (a variant may be supported by only one core — e.g.
   sing-box supports ciphers xray-core deprecates).

## Goal

Refine the e2e facility so a protocol security/transport variant is a first-class,
small unit: add a variant → new impl + one test, no struct copy. Prove it by adding
the **chacha20-poly1305 payload variant** for VMESS (native protocol support + e2e
tests against real xray-core and sing-box), while migrating the existing
aes-128-gcm VMESS case and the VLESS case onto the new shape as regression proof.

## Wire Contract: VMESS chacha20-poly1305 (verified against vendored Go)

Sources of truth (read-only): `thirdparty/Xray-core/proxy/vmess/encoding/{client,encoding,server}.go`,
`thirdparty/Xray-core/proxy/vmess/aead/`, `thirdparty/Xray-core/proxy/vmess/encoding/auth.go`,
and sing-box's vendored `sing-vmess@v0.2.8-0.20250909125414-3aed155119a1`
(`/home/user/go/pkg/mod/github.com/sagernet/sing-vmess@…/protocol.go`).

| Thing | Value |
|---|---|
| Header security byte | `4` = `SecurityTypeChacha20Poly1305` (xray `common/protocol/headers.go`, sing-vmess `protocol.go:38`) |
| Record cipher | chacha20poly1305, 12-byte nonce (RustCrypto `chacha20poly1305` crate) |
| Cipher key | `md5(k) ‖ md5(md5(k))` — 32 bytes, k = the 16-byte KDF16 session body key. xray `encoding/auth.go:20-27` and sing-vmess `protocol.go:144-151` are byte-identical: `key[0:16]=md5(k)`, `key[16:32]=md5(md5(k))`. **Not** md5‖zeros. |
| Record framing | Identical to the existing aes-128-gcm path: 2B BE ciphertext-length (plaintext+16 GCM tag), counter nonce = IV with first 2 bytes overwritten by BE record counter, per-direction counters start 0, EOF marker = length field 16 (or 0 defensively) |
| Option byte | stays 0 — server picks `PlainChunkSizeParser` from the option byte (proven by the aes-128-gcm e2e) |
| Server behavior | honors the CLIENT header security byte (`encoding/server.go:264+`). xray inbound user `security` field is cosmetic for AEAD; set `"chacha20-poly1305"` to mirror intent. sing-box inbound user has NO security field (established in the VMESS task: `inbounds[].users[].security` is rejected) — sing-box case is a client-only change |
| Client config | `VmessConfig.security.enc = "chacha20-poly1305"` — existing config path; no `NativeConnectParams` change |

## Native Protocol Changes (`crates/xray-tui-native/src/protocol/vmess/`)

1. Dependency: `chacha20poly1305 = "0.10"` (RustCrypto; same family/era as `aes-gcm 0.10`).
2. `stream.rs` — parameterize the record cipher:
   - `enum PayloadCipher { Aes128Gcm, Chacha20Poly1305 }`, held as an enum of the two
     concrete cipher instances (both implement `aead::Aead` with 12B nonce; no `dyn`).
   - Key derivation per cipher: aes-gcm uses the 16-byte session key directly;
     chacha20 uses `md5(k) ‖ md5(md5(k))` (new helper, mirrored from xray `auth.go`).
   - Read/write record paths dispatch on the enum; framing, nonce/counter logic,
     EOF semantics unchanged.
3. `header.rs`:
   - `pub const SECURITY_CHACHA20_POLY1305: u8 = 4;`
   - `encode_request` takes the security byte (or `Session` carries it); the body
     security byte is written from the parameter instead of the hardcoded `3`.
   - `Session` carries the chosen cipher through to `VmessClientStream`.
4. `mod.rs`:
   - `check_security` accepts `"chacha20-poly1305"` (alongside `None`/`""`/`"auto"`/
     `"aes-128-gcm"`); still rejects `none`/`zero`/legacy names with `NativeError::Config`
     (xray-core 26.x refuses those body streams server-side).
   - `connect` maps `cfg.security.enc` → `PayloadCipher` → `encode_request` +
     `VmessClientStream::new`.
5. Golden vectors (unit tests, computed against real Go — the `/tmp/gokdf` pattern
   from the VMESS task, never python-guessed for protocol-critical bytes):
   - chacha20 key derivation: known 16B session key → expected 32B `md5‖md5(md5)` key.
   - One full record seal: plaintext → expected ciphertext+tag with a fixed counter,
     cross-checked against Go's `AuthenticationWriter` with `GenerateChacha20Poly1305Key`.

## Facility Architecture (`crates/xray-tui-native/src/e2e/`)

The `E2eCase` trait and the `run()` driver are the pipeline contract and stay as-is.
A new data-driven layer sits below them.

```
src/e2e/
  mod.rs     — E2eCase trait, run(), exports. Add pub consts SINGBOX_VERSION / XRAY_VERSION
              (single source; delete per-test-file copies).
  variant.rs — SecurityVariant trait:
                 fn name(&self) -> &'static str                      // label fragment
                 fn cores(&self) -> &'static [CoreKind]              // gate
                 fn server_security(&self, core) -> Option<&'static str>
                 fn client_security(&self) -> &'static str           // enc value
  case.rs    — CaseSpec:
                 protocol: ProtocolKind { Vless, Vmess }
                 transport: Tcp (unit for now — axis deferred)
                 security: Option<Box<dyn SecurityVariant>>          // None = no payload security
               E2eCase impl: derived label "{proto}/tcp/tls[/{security.name()}]",
               client_trust (shared TLS-CA hook), expected (200 + body).
  config.rs  — typed per-core builder fns (both core dialects in one place per protocol):
                 vmess_inbound(core, port, uuid, security, tls_conf, tmp) -> String
                 vless_inbound(core, port, uuid, tls_conf, tmp) -> String
                 client_params_vmess(enc, port, target) -> NativeConnectParams
                 client_params_vless(port, target) -> NativeConnectParams
```

- A case = one `CaseSpec` value + variant impls. Adding chacha20 = one new
  `SecurityVariant` impl + one test registration. No struct copy.
- `cores()` gate: cores outside the list are skipped with `eprintln` (same skip
  pattern as absent `XRAY_TUI_CORE_BIN_DIR`), not failed.
- Labels: `vmess/tcp/tls/aes-128-gcm`, `vmess/tcp/tls/chacha20-poly1305`, `vless/tcp/tls`
  — structured and filterable, derived from the case data.
- TLS/transport stay fixed fields (tcp+tls) — the transport axis is explicitly
  deferred (Section: Out of Scope). The `SecurityVariant` trait shape leaves room for
  a parallel `TransportVariant` dimension later.
- Server TLS certificate/key PEM paths come from `ServerEnv` (unchanged).

## Test Shape

- `tests/vmess.rs` and `tests/vless.rs` rewrite:
  - One `#[tokio::test]` per variant, iterating `variant.cores()` sequentially in-test
    (thread-local `TEST_CFG` is safe: `#[tokio::test]` = current-thread runtime per
    test thread, set+connect on the same thread).
  - Per-core iteration failure names the core in the panic message
    (`.expect("vmess/chacha20-poly1305/xray …")`).
- Test inventory after the change:
  - `vless_tcp_tls_against_cores` (security: None) — xray + sing-box
  - `vmess_tcp_tls_aes128gcm_against_cores` — xray + sing-box
  - `vmess_tcp_tls_chacha20poly1305_against_cores` — xray + sing-box
- Unchanged: bounded probes (5s write/5s read), 3× fresh-core retries, echo/certs/
  tempdir created once, `XRAY_TUI_CORE_BIN_DIR` skip, `E2eExpect { 200, "hello native core" }`.

## Verification

1. Unit (lib, default + `native-e2e` features): chacha key-derivation golden vs Go,
   record seal/peel roundtrip per cipher, fragmented-header regression test (existing,
   stays), security validation (accepts chacha20-poly1305, rejects none/zero),
   existing 56 native-e2e tests keep passing.
2. E2E interop sweep against real binaries
   (`XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores`: sing-box 1.13.16, xray 26.3.27):
   6 runs — vless ×2 cores, vmess/aes ×2, vmess/chacha ×2 — all must pass.
3. Full workspace `cargo test` (884+), `cargo clippy` 0 warnings, `cargo fmt --check` clean.

## Out of Scope (explicit)

- Transport variants (ws/grpc/h2…) — deferred; the dimension pattern is proven with
  security first, transport comes when a real variant exists (it needs stream-layer
  work, not just config deltas).
- Other VMESS securities (`none`, `zero`, legacy aes-128-cfb) — refused by xray-core
  26.x body streams; not added.
- Changes to the `run()` driver, harness, probe, or retry semantics — the flake-hardened
  core is untouched.
- Non-VMESS protocol variants (e.g. VLESS realities) — same pattern applies later.

## File Map

| File | Change |
|---|---|
| `crates/xray-tui-native/Cargo.toml` | + `chacha20poly1305 = "0.10"` |
| `crates/xray-tui-native/src/protocol/vmess/stream.rs` | `PayloadCipher` enum, dispatch, chacha key helper, goldens |
| `crates/xray-tui-native/src/protocol/vmess/header.rs` | security-byte param, `SECURITY_CHACHA20_POLY1305` |
| `crates/xray-tui-native/src/protocol/vmess/mod.rs` | `check_security`, `connect` cipher mapping |
| `crates/xray-tui-native/src/e2e/mod.rs` | version consts; exports |
| `crates/xray-tui-native/src/e2e/variant.rs` (new) | `SecurityVariant` trait + impls (aes-128-gcm, chacha20-poly1305) |
| `crates/xray-tui-native/src/e2e/case.rs` (new) | `CaseSpec` + `E2eCase` impl |
| `crates/xray-tui-native/src/e2e/config.rs` (new) | typed per-core builder fns |
| `crates/xray-tui-native/src/protocol/vmess/e2e.rs` | rewritten onto `CaseSpec` + variants |
| `crates/xray-tui-native/src/protocol/vless/e2e.rs` | rewritten onto `CaseSpec` (security: None) |
| `crates/xray-tui-native/tests/vmess.rs`, `tests/vless.rs` | one test per variant, cores in-test |

## Risks

- **Key-derivation goldens**: the `md5‖md5(md5)` chain is the highest-risk wire fact;
  mitigated by computing goldens against the vendored Go running verbatim (T5-proven
  `/tmp/gokdf` method) before baking them into tests.
- **sing-vmess dialect**: sing-box's VMess may differ in edge details (option byte
  handling); e2e adjudicates — if sing-box rejects the xtls header-body layout, adapt
  only the body generator, never weaken the xray side (same rule as the VMESS task).
- **Double-md5 is not a security property we choose**: it is xray's fixed derivation;
  we mirror it for interop, never "improve" it.
