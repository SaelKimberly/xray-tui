# rstest Test Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the e2e variant matrix and the repetitive unit tables in `xray-tui-tls` + `xray-tui-native` to rstest 0.26.1 fixtures and case tables (2 e2e fns → 78 generated tests, 25 unit fns → 3 rstest tables + 4 one-offs), with no non-test behavior changes.

**Architecture:** `#[case::name(...)]` rows describe scenarios (CaseSpec builders stay; trait-object variants stay); `#[values(...)]` adds the core axis; `#[once]`/per-test `#[fixture]`s supply cores/certs/echo/tls_echo from a new `tests/common/mod.rs`; the e2e runner gains the shared-resource params. Unit tables use `#[case]` payloads + `#[values]` splits.

**Tech Stack:** rstest 0.26.1 (crates.io, latest stable), existing tokio/rcgen/tiny_http e2e harness, existing CaseSpec/TlsVariant engine.

## Global Constraints

- rstest `= "0.26.1"` — add ONCE to root `[workspace.dependencies]`, then `rstest.workspace = true` in `crates/xray-tui-tls/Cargo.toml` + `crates/xray-tui-native/Cargo.toml` `[dev-dependencies]` (2+ crates rule).
- E2E gating unchanged: `#![cfg(feature = "native-e2e")]` on integration tests; env `XRAY_TUI_CORE_BIN_DIR` hard-fails via `CoreUnderTest::resolve` (no default).
- NO `#[timeout]` on e2e tests (rstest abort skips `CoreGuard` Drop → orphan cores). NO `#[trace]` (CaseSpec not Debug).
- Clippy pedantic+nursery: ZERO warnings workspace-wide (`cargo clippy --workspace --all-targets`). rustfmt clean.
- NO non-test source changes except: `crates/xray-tui-native/src/e2e/mod.rs` (runner signature), test modules inside `src/transport/grpc.rs` + `src/spec/mod.rs`, and new `tests/common/mod.rs`. `src/e2e/core.rs`, `config.rs`, `variant.rs`, `case.rs`, `harness.rs` untouched.
- No `pkill`/blanket kills of core binaries; core processes are only ever reaped by `CoreGuard` Drop (tests) or targeted `kill <pid>`.
- Existing e2e expectations (`E2eExpect`), `ATTEMPTS=3`, 7-step lifecycle, and the fallback assertions (connect errors with `NativeError::Reality` + Spider-X preface reaches dest) are behavior-preserved.

---

### Task 1: Add rstest to workspace + both crates

**Files:**
- Modify: `Cargo.toml` (workspace `[workspace.dependencies]`)
- Modify: `crates/xray-tui-tls/Cargo.toml` (`[dev-dependencies]`)
- Modify: `crates/xray-tui-native/Cargo.toml` (`[dev-dependencies]`)

**Interfaces:**
- Produces: `rstest` crate available to test code in both crates.

- [ ] **Step 1: Add workspace dependency**

In root `Cargo.toml`, under `[workspace.dependencies]`, add:
```toml
rstest = "0.26.1"
```
(Alphabetical placement; do not touch other entries.)

- [ ] **Step 2: Enable in xray-tui-tls dev-dependencies**

In `crates/xray-tui-tls/Cargo.toml` `[dev-dependencies]`, add:
```toml
rstest.workspace = true
```

- [ ] **Step 3: Enable in xray-tui-native dev-dependencies**

In `crates/xray-tui-native/Cargo.toml` `[dev-dependencies]`, add:
```toml
rstest.workspace = true
```

- [ ] **Step 4: Verify resolution**

Run: `cargo check -p xray-tui-tls -p xray-tui-native --tests`
Expected: compiles; Cargo.lock gains `rstest 0.26.1` + `rstest_macros`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/xray-tui-tls/Cargo.toml crates/xray-tui-native/Cargo.toml Cargo.lock
git commit -m "build: add rstest 0.26.1 dev-dep (tls + native)"
```

---

### Task 2: grpc codec unit tests → rstest table

**Files:**
- Modify: `crates/xray-tui-native/src/transport/grpc.rs` (tests module only)

**Interfaces:**
- Consumes: `encode_frame(&[u8]) -> Vec<u8>`, `decode_frame(&mut BytesMut) -> Option<Vec<u8>>`, `varint_len(u64) -> usize`, `grpc_service_name(&GrpcConfig) -> String`, `grpc_path(&GrpcConfig, &str) -> String` (all existing, unchanged).
- Produces: `frame_roundtrip` (3 cases), `frame_parse_splits_across_chunks` (3 cases × 4 splits = 12 tests). `frame_parse_handles_multiple_messages`, `service_name_is_verbatim_no_default`, `path_builds_slash_service_tun` stay as-is.

- [ ] **Step 1: Replace the two framing tests**

In the `#[cfg(test)] mod tests` of `crates/xray-tui-native/src/transport/grpc.rs`, delete `frame_roundtrip` and `frame_hunk_wraps_long_payloads_with_varint`. Add `use rstest::rstest;` at the top of the tests module. Add:

```rust
    #[rstest]
    #[case::short(b"hello vless".to_vec())]
    #[case::varint_two_byte(vec![0x42; 200])]
    #[case::empty(Vec::new())]
    fn frame_roundtrip(#[case] payload: Vec<u8>) {
        let framed = encode_frame(&payload);
        // gRPC prefix (flag 0 + BE hunk byte-len) + Hunk protobuf:
        // tag 0x0A + varint(data_len) + data.
        // `varint_len(n)` returns the varint BYTES, so its len() is the
        // encoded width.
        let hunk_len = 1 + varint_len(payload.len()).len() + payload.len();
        assert_eq!(
            &framed[..5],
            &[0, 0, 0, (hunk_len >> 8) as u8, hunk_len as u8]
        );
        assert_eq!(framed[5], 0x0A);
        let mut buf = BytesMut::from(&framed[..]);
        assert_eq!(decode_frame(&mut buf), Some(payload));
        assert!(buf.is_empty());
    }

    #[rstest]
    #[case::short(b"abcdef".to_vec())]
    #[case::varint_two_byte(vec![0x42; 200])]
    #[case::empty(Vec::new())]
    fn frame_parse_splits_across_chunks(
        #[case] payload: Vec<u8>,
        #[values(0, 2, 5, 6)] split: usize,
    ) {
        let framed = encode_frame(&payload);
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&framed[..split]);
        assert_eq!(decode_frame(&mut buf), None); // partial frame
        buf.extend_from_slice(&framed[split..]);
        assert_eq!(decode_frame(&mut buf), Some(payload));
    }
```

Note: `varint_len` is already in scope via `use super::*;`. For the `empty`
case `framed.len()` is 7 (5-byte prefix + `[0x0A, 0x00]`); split points
0/2/5/6 all stay inside it and each partial prefix yields `None`. `BytesMut`
is already imported in the tests module.

- [ ] **Step 2: Run the unit tests**

Run: `cargo test -p xray-tui-native --lib transport::grpc`
Expected: 18 grpc tests pass (3 roundtrip + 12 split + 3 one-offs: multi-message, service_name, path).

- [ ] **Step 3: Clippy check**

Run: `cargo clippy -p xray-tui-native --all-targets`
Expected: zero warnings. If pedantic fires on rstest-generated code (e.g. `needless_pass_by_value` on `#[case] payload: Vec<u8>`), add the minimal targeted `#[allow]` on that fn with a one-line comment; do not loosen workspace lints.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/transport/grpc.rs
git commit -m "test(native): rstest tables for grpc frame codec"
```

---

### Task 3: spec extension wire-encoding → one rstest table

**Files:**
- Modify: `crates/xray-tui-tls/src/spec/mod.rs` (tests module only)

**Interfaces:**
- Consumes: `ExtensionSpec::encode_body(&RuntimeValues) -> Result<Vec<u8>, TlsError>`, `RuntimeValues` (fields `server_name: String`, `grease_a: u16`, `grease_b: u16`, `x25519_pub: [u8; 32]`, `padding_len: usize`, `Default`), `ExtensionSpec` variants, `KeyShareGroup::{Grease, X25519}` (all existing).
- Produces: one rstest fn `extension_wire_encoding` with 18 named `#[case]` rows. `grease_detection` stays as a one-off.

- [ ] **Step 1: Replace the 18 encoding tests with one table fn**

In `crates/xray-tui-tls/src/spec/mod.rs` tests module, delete the 18 `#[test] fn *encodes*` fns (`server_name_encodes_host` … `raw_encodes_type_length_data`, inclusive — every fn whose body is `X.encode_body(&rt).unwrap()` + `assert_eq!`). Add `use rstest::rstest;` and this fn (expected vectors copied verbatim from the deleted tests):

```rust
    #[rstest]
    #[case::server_name(
        ExtensionSpec::ServerName,
        RuntimeValues { server_name: "example.com".into(), ..RuntimeValues::default() },
        vec![0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x00, 0x0b, b'e', b'x', b'a', b'm',
             b'p', b'l', b'e', b'.', b'c', b'o', b'm']
    )]
    #[case::supported_groups(
        ExtensionSpec::SupportedGroups(vec![0x1301, 0x1302, 0x1303]),
        RuntimeValues::default(),
        vec![0x00, 0x0a, 0x00, 0x08, 0x00, 0x06, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03]
    )]
    #[case::key_share(
        ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
        RuntimeValues { grease_a: 0x1A1A, x25519_pub: [0xAB; 32], ..RuntimeValues::default() },
        {
            let mut v = vec![0x00, 0x33, 0x00, 0x2b, 0x00, 0x29, 0x1a, 0x1a, 0x00, 0x01, 0x00,
                             0x00, 0x1d, 0x00, 0x20];
            v.extend_from_slice(&[0xAB; 32]);
            v
        }
    )]
    #[case::supported_versions(
        ExtensionSpec::SupportedVersions(vec![0x0A0A, 0x0304, 0x0303]),
        RuntimeValues::default(),
        vec![0x00, 0x2b, 0x00, 0x07, 0x06, 0x0a, 0x0a, 0x03, 0x04, 0x03, 0x03]
    )]
    #[case::signature_algorithms(
        ExtensionSpec::SignatureAlgorithms(vec![0x0403, 0x0804]),
        RuntimeValues::default(),
        vec![0x00, 0x0d, 0x00, 0x06, 0x00, 0x04, 0x04, 0x03, 0x08, 0x04]
    )]
    #[case::alpn(
        ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
        RuntimeValues::default(),
        vec![0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, b'h', b'2', 0x08, b'h', b't', b't', b'p',
             b'/', b'1', b'.', b'1']
    )]
    #[case::ec_point_formats(
        ExtensionSpec::EcPointFormats,
        RuntimeValues::default(),
        vec![0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]
    )]
    #[case::session_ticket(
        ExtensionSpec::SessionTicket,
        RuntimeValues::default(),
        vec![0x00, 0x23, 0x00, 0x00]
    )]
    #[case::psk_key_exchange_modes(
        ExtensionSpec::PskKeyExchangeModes,
        RuntimeValues::default(),
        vec![0x00, 0x2d, 0x00, 0x02, 0x01, 0x01]
    )]
    #[case::status_request(
        ExtensionSpec::StatusRequest,
        RuntimeValues::default(),
        vec![0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00]
    )]
    #[case::signed_certificate_timestamp(
        ExtensionSpec::SignedCertificateTimestamp,
        RuntimeValues::default(),
        vec![0x00, 0x12, 0x00, 0x00]
    )]
    #[case::renegotiation_info(
        ExtensionSpec::RenegotiationInfo,
        RuntimeValues::default(),
        vec![0xff, 0x01, 0x00, 0x01, 0x00]
    )]
    #[case::compress_certificate(
        ExtensionSpec::CompressCertificate(vec![0x0002, 0x0001]),
        RuntimeValues::default(),
        vec![0x00, 0x1b, 0x00, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01]
    )]
    #[case::application_settings(
        ExtensionSpec::ApplicationSettings(vec!["h2".into()]),
        RuntimeValues::default(),
        vec![0x44, 0x69, 0x00, 0x06, 0x00, 0x04, 0x00, 0x02, b'h', b'2']
    )]
    #[case::record_size_limit(
        ExtensionSpec::RecordSizeLimit(0x00FF),
        RuntimeValues::default(),
        vec![0x00, 0x1c, 0x00, 0x02, 0x00, 0xff]
    )]
    #[case::padding(
        ExtensionSpec::Padding,
        RuntimeValues { padding_len: 4, ..RuntimeValues::default() },
        vec![0x00, 0x15, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00]
    )]
    #[case::grease(
        ExtensionSpec::Grease,
        RuntimeValues { grease_b: 0x1A1A, ..RuntimeValues::default() },
        vec![0x1a, 0x1a, 0x00, 0x01, 0x00]
    )]
    #[case::raw(
        ExtensionSpec::Raw { ty: 0x1234, data: vec![0xde, 0xad] },
        RuntimeValues::default(),
        vec![0x12, 0x34, 0x00, 0x02, 0xde, 0xad]
    )]
    fn extension_wire_encoding(
        #[case] ext: ExtensionSpec,
        #[case] rt: RuntimeValues,
        #[case] expected: Vec<u8>,
    ) {
        assert_eq!(ext.encode_body(&rt).unwrap(), expected);
    }
```

Keep `grease_detection` untouched.

- [ ] **Step 2: Run the tests**

Run: `cargo test -p xray-tui-tls --lib spec`
Expected: 19 pass (18 table rows + grease_detection).

- [ ] **Step 3: Clippy check**

Run: `cargo clippy -p xray-tui-tls --all-targets`
Expected: zero warnings. If pedantic fires on a `#[case]` arg (e.g. `vec!` → `Vec::with_capacity` hints, or value-passed params), add the minimal targeted `#[allow]` on the fn with a one-line comment.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-tls/src/spec/mod.rs
git commit -m "test(tls): rstest table for extension wire encoding"
```

---

### Task 4: e2e runner gains shared resources + shared fixtures module

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (runner `run`/`run_against_cores` → `run_against`)
- Create: `crates/xray-tui-native/tests/common/mod.rs`

**Interfaces:**
- Consumes: `E2eCase` (unchanged trait), `CaseSpec` (unchanged), `CoreUnderTest::resolve`, `Certs { cert_pem, key_pem }`, `EchoServer`, `TlsEchoServer::reset_recording`, `spawn_echo()`, `generate_certs()`, `spawn_tls_echo(&Certs)`, `CoreKind` (Copy).
- Produces:
  - `pub async fn run_against(case: &CaseSpec, core: &CoreUnderTest, certs: &Certs, echo: &EchoServer, tls_echo: &TlsEchoServer) -> Result<(), String>` — the same 7-step lifecycle + 3 attempts, using the passed resources (no internal spawn of echo/certs/tls_echo).
  - `tests/common/mod.rs`: `#[fixture] #[once] pub fn cores() -> (CoreUnderTest, CoreUnderTest)`; `#[fixture] #[once] pub fn certs() -> Certs`; `#[fixture] pub fn echo() -> EchoServer`; `#[fixture] pub fn tls_echo(certs: &Certs) -> TlsEchoServer`; `pub fn pick<'a>(cores: &'a (CoreUnderTest, CoreUnderTest), kind: CoreKind) -> &'a CoreUnderTest`.

- [ ] **Step 1: Rework the runner**

In `crates/xray-tui-native/src/e2e/mod.rs`, replace `run<C: E2eCase + Sync>(case: &C, core: &CoreUnderTest)` and `run_against_cores` with:

```rust
/// Run the fixed 7-step e2e lifecycle for `case` against `core`.
///
/// The flaky segment (core spawn + connect + probe) is retried with fresh
/// resources; echo/certs/tls_echo/tempdir are created once by the caller
/// (rstest fixtures) and shared across the retries.
pub async fn run_against(
    case: &CaseSpec,
    core: &CoreUnderTest,
    certs: &Certs,
    echo: &EchoServer,
    tls_echo: &TlsEchoServer,
) -> Result<(), String> {
    let expect = case.expected();
    let dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).map_err(|e| format!("cert write: {e}"))?;
    std::fs::write(&key_path, &certs.key_pem).map_err(|e| format!("key write: {e}"))?;
    let config_path = dir.path().join("config.json");

    for attempt in 1..=ATTEMPTS {
        // Fresh recording per attempt: the tls_echo is created once before
        // the loop, so without a reset its buffer accumulates across
        // attempts — a preface that arrived just after attempt N's poll
        // expiry could satisfy attempt N+1 spuriously.
        tls_echo.reset_recording();
        let port = free_port();
        let env = ServerEnv {
            port,
            certs,
            tmp: dir.path(),
            echo: echo.addr,
            tls_echo: tls_echo.addr,
        };
        let config_json = case.server_config(core.kind, &env);
        if std::fs::write(&config_path, &config_json).is_err() {
            return Err(format!("attempt {attempt}: config write failed"));
        }
        let _core = spawn_core(&core.bin, core.kind, &config_path, port);

        case.client_trust(certs);
        let params = case.client_params(port, case.probe_target(&env));
        // ...rest of the attempt body unchanged from the current run()...
    }
    // ...final Ok(()) unchanged...
}
```

Mechanically: take the CURRENT body of `run()`; delete its first three lines (`let echo = spawn_echo(); let certs = generate_certs(); let tls_echo = spawn_tls_echo(&certs);`), change the signature as above (case type becomes `&CaseSpec`), change `certs: &certs` → `certs` in the `ServerEnv` construction (it is already `&Certs`), and delete `run_against_cores` entirely. Everything after `let _core = spawn_core(...)` stays byte-identical. The `E2eCase` generic bound is dropped (the fn now takes `&CaseSpec` only).

- [ ] **Step 2: Create the fixtures module**

Create `crates/xray-tui-native/tests/common/mod.rs`:

```rust
//! Shared rstest fixtures for the e2e integration tests (feature `native-e2e`).
#![cfg(feature = "native-e2e")]

use rstest::fixture;
use xray_tui_native::e2e::{
    CoreKind, CoreUnderTest, Certs, EchoServer, TlsEchoServer, SINGBOX_VERSION, XRAY_VERSION,
    generate_certs, spawn_echo, spawn_tls_echo,
};

/// Resolved core binaries, once per test binary (hard-fails when
/// `XRAY_TUI_CORE_BIN_DIR` is missing/unversioned).
#[fixture]
#[once]
pub fn cores() -> (CoreUnderTest, CoreUnderTest) {
    let xray = CoreUnderTest::resolve(CoreKind::Xray, XRAY_VERSION).expect("resolve xray-core");
    let singbox =
        CoreUnderTest::resolve(CoreKind::SingBox, SINGBOX_VERSION).expect("resolve sing-box");
    (xray, singbox)
}

/// Harness CA (cert/key), generated once per test binary.
#[fixture]
#[once]
pub fn certs() -> Certs {
    generate_certs()
}

/// Plain HTTP echo; per-test (tiny_http owns a thread; Drop unblocks it).
#[fixture]
pub fn echo() -> EchoServer {
    spawn_echo()
}

/// TLS HTTP echo with the harness CA; per-test — `reset_recording`/
/// `saw_h2_preface` is mutable fallback-case state, so sharing would race.
#[fixture]
pub fn tls_echo(certs: &Certs) -> TlsEchoServer {
    spawn_tls_echo(certs)
}

/// Select the resolved core for a `CoreKind` case value.
#[must_use]
pub fn pick<'a>(cores: &'a (CoreUnderTest, CoreUnderTest), kind: CoreKind) -> &'a CoreUnderTest {
    match kind {
        CoreKind::Xray => &cores.0,
        CoreKind::SingBox => &cores.1,
    }
}
```

- [ ] **Step 3: Verify compile (fixtures unused so far)**

Run: `cargo check -p xray-tui-native --tests --features native-e2e`
Expected: compiles (unused `common` module is not referenced by any test yet — the module file alone is inert). If clippy is run on the module and flags the `#[once]` fixture fns, ignore for now; Task 5 wires them.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/e2e/mod.rs crates/xray-tui-native/tests/common/mod.rs
git commit -m "test(e2e): run_against takes shared fixtures; common rstest fixtures"
```

---

### Task 5: vless e2e → rstest matrix (21 cases × 2 cores = 42 tests)

**Files:**
- Modify: `crates/xray-tui-native/tests/vless.rs` (full rewrite)

**Interfaces:**
- Consumes: `CaseSpec` (`::vless()`, `::with_network(&str)`, `::with_tls(Box<dyn TlsVariant>)`), variants `FingerprintTls("chrome")`, `RealityTls::fresh()`, `RealityWrongPbkTls::fresh()`, `RealityWrongSidTls::fresh()`, `RealityServerPlainClientTls::fresh()`, `PlainServerRealityClientTls::fresh()`; `run_against` (Task 4); fixtures `cores`, `certs`, `echo`, `tls_echo` + `pick` (Task 4).
- Produces: `vless_against_cores` — 42 generated tests.

- [ ] **Step 1: Rewrite the file**

Replace the whole content of `crates/xray-tui-native/tests/vless.rs` with:

```rust
//! E2E: native VLESS matrix — network {tcp, ws, grpc} × TLS variant ×
//! core {xray, sing-box}. One generated test per (case, core).
#![cfg(feature = "native-e2e")]

mod common;

use rstest::rstest;
use xray_tui_native::e2e::{
    CaseSpec, CoreKind, CoreUnderTest, Certs, EchoServer, FingerprintTls,
    PlainServerRealityClientTls, RealityServerPlainClientTls, RealityTls, RealityWrongPbkTls,
    RealityWrongSidTls, TlsEchoServer, run_against,
};
use common::{certs, cores, echo, pick, tls_echo};

#[rstest]
#[case::tcp_plain(CaseSpec::vless())]
#[case::tcp_chrome(CaseSpec::vless().with_tls(Box::new(FingerprintTls("chrome"))))]
#[case::tcp_reality(CaseSpec::vless().with_tls(Box::new(RealityTls::fresh())))]
#[case::tcp_reality_wrong_pbk(CaseSpec::vless().with_tls(Box::new(RealityWrongPbkTls::fresh())))]
#[case::tcp_reality_wrong_sid(CaseSpec::vless().with_tls(Box::new(RealityWrongSidTls::fresh())))]
#[case::tcp_plain_into_reality_server(
    CaseSpec::vless().with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::tcp_reality_into_plain_server(
    CaseSpec::vless().with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
#[case::ws_plain(CaseSpec::vless().with_network("ws"))]
#[case::ws_chrome(CaseSpec::vless().with_network("ws").with_tls(Box::new(FingerprintTls("chrome"))))]
#[case::ws_reality(CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityTls::fresh())))]
#[case::ws_reality_wrong_pbk(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityWrongPbkTls::fresh()))
)]
#[case::ws_reality_wrong_sid(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityWrongSidTls::fresh()))
)]
#[case::ws_plain_into_reality_server(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::ws_reality_into_plain_server(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
#[case::grpc_plain(CaseSpec::vless().with_network("grpc"))]
#[case::grpc_chrome(CaseSpec::vless().with_network("grpc").with_tls(Box::new(FingerprintTls("chrome"))))]
#[case::grpc_reality(CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityTls::fresh())))]
#[case::grpc_reality_wrong_pbk(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityWrongPbkTls::fresh()))
)]
#[case::grpc_reality_wrong_sid(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityWrongSidTls::fresh()))
)]
#[case::grpc_plain_into_reality_server(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::grpc_reality_into_plain_server(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
#[tokio::test]
async fn vless_against_cores(
    #[case] case: CaseSpec,
    #[values(CoreKind::Xray, CoreKind::SingBox)] core: CoreKind,
    cores: &(CoreUnderTest, CoreUnderTest),
    certs: &Certs,
    echo: EchoServer,
    tls_echo: TlsEchoServer,
) {
    // `#[once]` fixtures inject as `&T` (cores, certs); per-test fixtures
    // inject as OWNED `T` (echo, tls_echo) — borrow them for the runner.
    run_against(&case, pick(cores, core), certs, &echo, &tls_echo)
        .await
        .expect("vless e2e failed");
}
```

Note: `#[tokio::test]` MUST come after every `#[case::…]` attribute (implicit test attribute rule — it binds to the generated test fns, not a case). The fixtures `cores`/`certs`/`echo`/`tls_echo` resolve by arg NAME to the imported `#[fixture]` fns.

- [ ] **Step 2: Compile + list**

Run: `cargo test -p xray-tui-native --features native-e2e --test vless -- --list`
Expected: compiles; 42 test names of shape `vless_against_cores::case_1_tcp_plain::core_1_Xray` (21 cases × 2 cores). Fix any attribute-order/import errors before proceeding.

- [ ] **Step 3: Full sweep + triage**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless -- --nocapture`
Expected: 42 pass.

Triage rule for red rows (investigate, do NOT silently delete):
1. Re-run the single test alone (`--exact …::case_N_…::core_2_SingBox`) to rule out flake.
2. If a new combo fails with a real server-side semantic mismatch (e.g. the stealth case `*_plain_into_reality_server` over ws/grpc: the plain client's ws-framed GET hits the dest HTTP server, which answers the raw HTTP probe but not ws framing — the tunnel is proxied BEFORE any VLESS/transport processing, so the transport framing cannot be honored), mark that row `#[ignore = "plain client into reality server: dest sees ws framing, not HTTP (transport unreachable pre-proxy)"]` — keep the row visible, documented, excluded from the pass set.
3. If a failure is OUR bug (config shape, expectation, codec), fix it in the appropriate file — the plan's non-test-source constraint allows fixes in `src/e2e/config.rs` ONLY for genuine config bugs; anything else goes back to the owning module and is out of this plan's scope (note it, don't fix).

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/tests/vless.rs
git commit -m "test(e2e): vless matrix (network x tls x core) via rstest"
```

---

### Task 6: vmess e2e → rstest matrix (18 cases × 2 cores = 36 tests)

**Files:**
- Modify: `crates/xray-tui-native/tests/vmess.rs` (full rewrite)

**Interfaces:**
- Consumes: `CaseSpec::vmess(SecurityVariant)` with `Aes128GcmVariant` / `Chacha20Poly1305Variant`; `FingerprintTls("chrome")`; `RealityTls::fresh()`; `run_against` + fixtures (Tasks 4–5).
- Produces: `vmess_against_cores` — 36 generated tests.

- [ ] **Step 1: Rewrite the file**

Replace the whole content of `crates/xray-tui-native/tests/vmess.rs` with the imports, the two case-builder helpers, the 18 `#[case::…]` rows, and the test fn below:

```rust
//! E2E: native VMess matrix — security {aes-128-gcm, chacha20-poly1305} ×
//! network {tcp, ws, grpc} × TLS {standard, chrome, reality} × core.
//! One generated test per (case, core).
#![cfg(feature = "native-e2e")]

mod common;

use rstest::rstest;
use xray_tui_native::e2e::{
    Aes128GcmVariant, CaseSpec, Chacha20Poly1305Variant, CoreKind, CoreUnderTest, Certs,
    EchoServer, FingerprintTls, RealityTls, SecurityVariant, TlsEchoServer, TlsVariant,
    run_against,
};
use common::{certs, cores, echo, pick, tls_echo};

/// `CaseSpec::vmess` takes the security variant by value; these helpers keep
/// the `#[case]` rows one-liners.
fn vmess(sec: impl SecurityVariant + 'static, net: &'static str) -> CaseSpec {
    CaseSpec::vmess(sec).with_network(net)
}

fn vmess_tls(
    sec: impl SecurityVariant + 'static,
    net: &'static str,
    tls: Box<dyn TlsVariant>,
) -> CaseSpec {
    vmess(sec, net).with_tls(tls)
}

#[rstest]
#[case::tcp_aes128gcm(vmess(Aes128GcmVariant, "tcp"))]
#[case::tcp_chacha20(vmess(Chacha20Poly1305Variant, "tcp"))]
#[case::tcp_aes128gcm_chrome(vmess_tls(Aes128GcmVariant, "tcp", Box::new(FingerprintTls("chrome"))))]
#[case::tcp_chacha20_chrome(vmess_tls(Chacha20Poly1305Variant, "tcp", Box::new(FingerprintTls("chrome"))))]
#[case::tcp_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "tcp", Box::new(RealityTls::fresh())))]
#[case::tcp_chacha20_reality(vmess_tls(Chacha20Poly1305Variant, "tcp", Box::new(RealityTls::fresh())))]
#[case::ws_aes128gcm(vmess(Aes128GcmVariant, "ws"))]
#[case::ws_chacha20(vmess(Chacha20Poly1305Variant, "ws"))]
#[case::ws_aes128gcm_chrome(vmess_tls(Aes128GcmVariant, "ws", Box::new(FingerprintTls("chrome"))))]
#[case::ws_chacha20_chrome(vmess_tls(Chacha20Poly1305Variant, "ws", Box::new(FingerprintTls("chrome"))))]
#[case::ws_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "ws", Box::new(RealityTls::fresh())))]
#[case::ws_chacha20_reality(vmess_tls(Chacha20Poly1305Variant, "ws", Box::new(RealityTls::fresh())))]
#[case::grpc_aes128gcm(vmess(Aes128GcmVariant, "grpc"))]
#[case::grpc_chacha20(vmess(Chacha20Poly1305Variant, "grpc"))]
#[case::grpc_aes128gcm_chrome(vmess_tls(Aes128GcmVariant, "grpc", Box::new(FingerprintTls("chrome"))))]
#[case::grpc_chacha20_chrome(vmess_tls(Chacha20Poly1305Variant, "grpc", Box::new(FingerprintTls("chrome"))))]
#[case::grpc_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "grpc", Box::new(RealityTls::fresh())))]
#[case::grpc_chacha20_reality(vmess_tls(Chacha20Poly1305Variant, "grpc", Box::new(RealityTls::fresh())))]
#[tokio::test]
async fn vmess_against_cores(
    #[case] case: CaseSpec,
    #[values(CoreKind::Xray, CoreKind::SingBox)] core: CoreKind,
    cores: &(CoreUnderTest, CoreUnderTest),
    certs: &Certs,
    echo: EchoServer,
    tls_echo: TlsEchoServer,
) {
    // `#[once]` fixtures inject as `&T` (cores, certs); per-test fixtures
    // inject as OWNED `T` (echo, tls_echo) — borrow them for the runner.
    run_against(&case, pick(cores, core), certs, &echo, &tls_echo)
        .await
        .expect("vmess e2e failed");
}
```

- [ ] **Step 2: Compile + list**

Run: `cargo test -p xray-tui-native --features native-e2e --test vmess -- --list`
Expected: compiles; 36 test names.

- [ ] **Step 3: Full sweep + triage**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vmess -- --nocapture`
Expected: 36 pass. Same triage rule as Task 5 step 3 (rerun alone → server-side semantic mismatch gets `#[ignore]` with a documented reason → our bug gets fixed or noted out-of-scope).

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/tests/vmess.rs
git commit -m "test(e2e): vmess matrix (security x network x tls x core) via rstest"
```

---

### Task 7: Full verification sweep

**Files:**
- None (verification only; docs touch-ups allowed if a verification claim changed)

**Interfaces:**
- Consumes: everything from Tasks 1–6.

- [ ] **Step 1: Workspace unit tests**

Run: `cargo test`
Expected: all crates green. Counts: xray-tui-tls lib ≈ 102 (spec: 18 table rows + grease_detection, unchanged total), xray-tui-native lib = 84 (72 − 6 old grpc fns + 18 new grpc tests) — confirm against the printed `test result: ok`.

- [ ] **Step 2: E2E sweep**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: 78 pass (42 vless + 36 vmess), 0 fail, 0 ignored (unless documented `#[ignore]` rows from triage).

- [ ] **Step 3: Clippy + fmt**

Run: `cargo clippy --workspace --all-targets` → zero warnings. `cargo fmt --check -p xray-tui-tls -p xray-tui-native` → clean (pre-existing app-crate fmt drift in `crates/xray-tui/src` is out of scope, unchanged).

- [ ] **Step 4: Update NATIVE_CORE.md verification row if the e2e counts changed**

In `NATIVE_CORE.md`, the tier-2/e2e verification line mentions test counts if present; otherwise no change. AGENTS.md: only if the test-organization description changed materially (the e2e test FILES are unchanged paths — likely no edit needed; judge by diff).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test: rstest redesign verification sweep"
```
