# Native VMESS + Unified E2E Pipeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement VMess over TCP (AEAD header + AES-128-GCM payload, modern xtls dialect) as the second native protocol, and replace the ad-hoc-e2e tests with a declarative per-protocol testing pipeline that runs the same 7 steps for any (core, protocol) pair.

**Architecture:** A feature-gated `src/e2e/` module owns the whole e2e lifecycle — echo server, certs, core spawn/readiness, version sanity, bounded probe, 3-attempt full-restart retry — driven by a per-protocol `E2eCase` trait impl in each protocol module. VMess is implemented as `protocol/vmess/{keys,header,stream,mod}.rs` mirroring the VLESS module layout, with the wire contract taken verbatim from `thirdparty/Xray-core/proxy/vmess/encoding/*.go`.

**Tech Stack:** Rust 2024, tokio, `aes`/`aes-gcm`/`hmac`/`sha2`/`crc32fast`/`md-5` for VMess crypto; `serde_json`/`tempfile`/`rcgen`/`tiny_http` as optional deps behind `native-e2e`.

## Global Constraints

- Wire authority: `thirdparty/Xray-core/proxy/vmess/encoding/client.go`, `encoding.go`, `server.go`, `aead/{consts,kdf,authid,encrypt}.go`, `common/protocol/{id,headers,payload}.go` (that's `NewID` for cmdKey, `SecurityType`=3, `AddressType` 1/2/3, `IDBytesLen`=16). Cross-checks: `thirdparty/shoes/src/vmess/*` and `~/go/pkg/mod/github.com/sagernet/sing-vmess@v0.2.8-*/` (do NOT edit either).
- Security payload for e2e MUST be `aes-128-gcm` (security nibble 3). xray-core 26.x rejects `none`/`zero`/`auto` in the body stream (`encoding/server.go` `default: invalid option: Security`); the native client must Config-error on `security.enc` values other than `aes-128-gcm` or `auto`.
- `KDF` = nested HMAC-SHA256: `msg = HMAC-SHA256("VMess AEAD KDF", key)`, then for each path string `v`: `msg = HMAC-SHA256(v, msg)`; `KDF16` = first 16 bytes.
- `cmdKey = md5(uuid_bytes ‖ "c48619fe-8f02-49e0-b9e9-edf763e17e21")` where `uuid_bytes` is the raw 16 bytes.
- Golden vectors (universally fixed, computed from the Go semantics): see Task 2 and Task 6 — they are the unit-test authority; e2e interop is the final authority.
- A hang is a bug: every network await bounded (`timeouts::{DIAL,TLS,PROTOCOL}`, probe 5s/5s, readiness 10s).
- Do NOT run the e2e suite unless `XRAY_TUI_CORE_BIN_DIR` points at real `xray` + `sing-box` binaries. Cores used during development: sing-box 1.13.16, xray 26.3.27 in `/tmp/native-cores`.
- Skip formatters/linters/project-wide suites during task steps unless the step says otherwise; clippy+fmt+full suite only in Task 10.

---

### Task 1: Dependencies and feature wiring

**Files:**
- Modify: `crates/xray-tui-native/Cargo.toml`
- Delete: `crates/xray-tui-native/examples/native_connect_check.rs` (the pipeline replaces it)

**Interfaces:**
- Produces: runtime deps `aes`, `aes-gcm`, `hmac`, `sha2`, `crc32fast` (all unconditional); optional deps `serde_json`, `tempfile`, `rcgen`, `tiny_http` enabled by `native-e2e`; `examples/` gone.

- [ ] **Step 1: Edit Cargo.toml**

```toml
[dependencies]
xray-tui-proto.workspace = true

tokio = { workspace = true, features = ["net", "io-util", "time"] }

rustls = { workspace = true, features = ["ring"] }
tokio-rustls = "0.26"
webpki-roots = "1"

# VMess wire crypto (mandatory for any VMess client — not feature-gated)
aes = "0.8"
aes-gcm = "0.10"
hmac = "0.12"
sha2 = "0.10"
crc32fast = "1"
md-5 = "0.10"

thiserror.workspace = true
uuid.workspace = true

# e2e pipeline (crate-side module -> real deps, optional behind the feature)
serde_json = { workspace = true, optional = true }
tempfile = { version = "3", optional = true }
rcgen = { version = "0.13", optional = true }
tiny_http = { version = "0.12", optional = true }

[lints]
workspace = true

[features]
native-e2e = ["dep:serde_json", "dep:tempfile", "dep:rcgen", "dep:tiny_http"]

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

Remove the old `[dev-dependencies]` entries for serde_json/tempfile/rcgen/tiny_http (they move up, optional). Remove `examples/native_connect_check.rs`.

- [ ] **Step 2: Verify it builds**

Run: `cargo check -p xray-tui-native && cargo check -p xray-tui-native --features native-e2e`
Expected: both pass.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui-native/Cargo.toml crates/xray-tui-native/examples/ && git rm -q crates/xray-tui-native/examples/native_connect_check.rs
git commit -m "chore(native): VMess crypto deps + feature-gated e2e deps, drop manual-check example"
```

---

### Task 2: E2E pipeline module — types, harness, core resolution

**Files:**
- Create: `crates/xray-tui-native/src/e2e/mod.rs`, `crates/xray-tui-native/src/e2e/core.rs`, `crates/xray-tui-native/src/e2e/harness.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` (register the module)

**Interfaces:**
- Produces (later tasks consume):
```rust
// src/e2e/core.rs
pub enum CoreKind { Xray, SingBox }
pub struct CoreUnderTest { pub kind: CoreKind, pub bin: PathBuf, pub version: String }
impl CoreUnderTest {
    pub fn resolve(kind: CoreKind, expected_version: &str) -> Result<Self, String>
    pub fn spawn_args(&self, config_path: &Path) -> Vec<String>  // ["-c", p] resp. ["run", "-c", p]
}
pub fn check_version(actual: &str, want: &str) -> bool  // actual.contains(want)

// src/e2e/harness.rs
pub struct Certs { pub cert_pem: String, pub key_pem: String, pub ca_der: Vec<u8> }
pub struct EchoServer { pub addr: SocketAddr }
pub fn spawn_echo() -> EchoServer
pub fn free_port() -> u16
pub fn generate_certs() -> Certs
pub struct CoreGuard;  // kills child on Drop
pub fn spawn_core(bin: &Path, kind: CoreKind, config_path: &Path, port: u16) -> CoreGuard
pub async fn probe(tunnel: &mut xray_tui_native::NativeTunnel) -> (u16, String)  // bounded 5s/5s, (0,"") on failure

// src/e2e/mod.rs
pub use core::{CoreKind, CoreUnderTest};
pub use harness::{Certs, EchoServer, probe, spawn_core, spawn_echo, generate_certs, free_port};
pub struct ServerEnv<'a> { pub port: u16, pub certs: &'a Certs, pub tmp: &'a Path }
pub struct E2eExpect { pub status: u16, pub body: String }
pub trait E2eCase {
    fn label(&self) -> &'static str;
    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String;
    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams;
    fn expected(&self) -> E2eExpect;
    fn client_trust(&self, certs: &Certs) { drop(certs); }
}
```
- Consumes: nothing (standalone; migration target for `tests/common/mod.rs` contents).

- [ ] **Step 1: Register the module in lib.rs**

```rust
pub mod addr;
pub mod chain;
#[cfg(feature = "native-e2e")]
pub mod e2e;
pub mod context;
```

- [ ] **Step 2: Write `src/e2e/core.rs`** (real code)

```rust
//! Binary-core resolution + version sanity for the e2e pipeline.
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreKind { Xray, SingBox }

impl CoreKind {
    /// Binary file name inside `XRAY_TUI_CORE_BIN_DIR}.
    pub fn bin_name(self) -> &'static str {
        match self { Self::Xray => "xray", Self::SingBox => "sing-box" }
    }
}

/// A resolved, version-checked core binary.
#[derive(Debug, Clone)]
pub struct CoreUnderTest {
    pub kind: CoreKind,
    pub bin: PathBuf,
    pub version: String,
}

impl CoreUnderTest {
    /// Resolve from `XRAY_TUI_CORE_BIN_DIR/<bin_name>`, probe its version,
    /// and sanity-check it against `expected_version` (substring match).
    pub fn resolve(kind: CoreKind, expected_version: &str) -> Result<Self, String> {
        let dir = std::env::var("XRAY_TUI_CORE_BIN_DIR")
            .map_err(|_| "XRAY_TUI_CORE_BIN_DIR is not set".to_string())?;
        let bin = PathBuf::from(dir).join(kind.bin_name());
        if !bin.is_file() {
            return Err(format!("no '{}' binary at {}", kind.bin_name(), bin.display()));
        }
        let actual = probe_version(&bin, kind)?;
        if !check_version(&actual, expected_version) {
            return Err(format!(
                "core version mismatch: want {expected_version:?}, got {actual:?}"
            ));
        }
        Ok(Self { kind, bin, version: actual })
    }

    /// argv for spawning with the given config file.
    pub fn spawn_args(&self, config_path: &std::path::Path) -> Vec<String> {
        let p = config_path.to_string_lossy().into_owned();
        match self.kind {
            CoreKind::Xray => vec!["-c".into(), p],
            CoreKind::SingBox => vec!["run".into(), "-c".into(), p],
        }
    }
}

/// Probe the version string: `-version` first, then `version` (sing-box).
fn probe_version(bin: &std::path::Path, kind: CoreKind) -> Result<String, String> {
    for flag in [Some("-version"), None] {
        let mut cmd = Command::new(bin);
        match flag {
            Some(f) => { cmd.arg(f); }
            None => { cmd.arg("version"); }
        }
        if let Ok(out) = cmd.output() {
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            let text = text.trim();
            if !text.is_empty() {
                return Ok(text.to_string());
            }
        }
        if kind == CoreKind::SingBox {
            break; // `run` would block; only try `-version` then exit
        }
    }
    Err(format!("failed to probe version of {}", bin.display()))
}

/// Loose sanity: the reported version contains the expected one.
pub fn check_version(actual: &str, want: &str) -> bool {
    actual.contains(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_substring_matches() {
        assert!(check_version("Xray 26.3.27 (Xray-core 26.3.27)", "26.3.27"));
        assert!(check_version("sing-box 1.13.16", "1.13.16"));
        assert!(!check_version("sing-box 1.12.0", "1.13.16"));
    }
}
```

- [ ] **Step 3: Write `src/e2e/harness.rs`** — migrate verbatim from `tests/common/mod.rs`: `spawn_echo`, `free_port`, `generate_certs`, `CoreGuard` (Drop kills child), `spawn_core(bin, kind, config_path, port)` (write nothing — config already on disk; spawn with `kind`'s args; readiness = TCP connect to `port` 100×100ms), and the bounded `probe` (5s write + 5s read, `(0, String::new())` on failure). Keep `CoreType`-free: use `CoreKind` from `core.rs`. The old `core_bin` helper is replaced by `CoreUnderTest::resolve`.

- [ ] **Step 4: Write `src/e2e/mod.rs`** — the trait, `ServerEnv`, `E2eExpect`. No `run()` yet (Task 3).

- [ ] **Step 5: Build + unit test**

Run: `cargo test -p xray-tui-native --features native-e2e --lib`
Expected: `test result: ok` (new `version_substring_matches` passes; count grows).

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/e2e crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): e2e pipeline module — trait, harness, core resolution (no driver yet)"
```

---

### Task 3: E2E driver `run()` + VLESS migration

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (add `run`)
- Create: `crates/xray-tui-native/src/protocol/vless/e2e.rs`
- Delete: `crates/xray-tui-native/tests/common/`, `crates/xray-tui-native/tests/vless_singbox.rs`, `crates/xray-tui-native/tests/vless_xray.rs`
- Create: `crates/xray-tui-native/tests/vless.rs`

**Interfaces:**
- Consumes: `e2e::{CoreUnderTest, CoreKind, Certs, EchoServer, ServerEnv, E2eExpect, spawn_echo, generate_certs, spawn_core, probe, free_port}`, `connect`, `security::tls::{set_test_config, test_client_config}`, `NativeConnectParams`.
- Produces: `pub async fn run<C: E2eCase>(case: &C, core: &CoreUnderTest) -> Result<(), String>` — executes steps 1–7, retries the flaky segment (fresh core + fresh connect + fresh probe) 3 times, eprintln per attempt, returns the last failure.
- Produces: `pub struct VlessCase;` with `impl E2eCase for VlessCase` (label `"vless/tcp/tls"`, body `"hello native core"`, UUID `00000000-0000-0000-0000-000000000000`, sni `localhost`, alpn `http/1.1`).

- [ ] **Step 1: Write the driver in `src/e2e/mod.rs`** (real code)

```rust
use std::net::SocketAddr;
use std::path::Path;

use crate::NativeConnectParams;

pub use core::{CoreKind, CoreUnderTest};
pub use harness::{Certs, EchoServer, free_port, generate_certs, probe, spawn_core, spawn_echo};

pub mod core;
pub mod harness;

pub struct ServerEnv<'a> {
    pub port: u16,
    pub certs: &'a Certs,
    pub tmp: &'a Path,
}

pub struct E2eExpect { pub status: u16, pub body: String }

pub trait E2eCase {
    fn label(&self) -> &'static str;
    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String;
    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams;
    fn expected(&self) -> E2eExpect;
    fn client_trust(&self, _certs: &Certs) {}
}

const ATTEMPTS: u32 = 3;

/// Run the fixed 7-step e2e lifecycle for `case` against `core`.
///
/// The flaky segment (core spawn + connect + probe) is retried with fresh
/// resources; echo/certs/tempdir are created once.
pub async fn run<C: E2eCase>(case: &C, core: &CoreUnderTest) -> Result<(), String> {
    let expect = case.expected();
    let echo = spawn_echo();
    let certs = generate_certs();
    let dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).map_err(|e| format!("cert write: {e}"))?;
    std::fs::write(&key_path, &certs.key_pem).map_err(|e| format!("key write: {e}"))?;
    let config_path = dir.path().join("config.json");

    for attempt in 1..=ATTEMPTS {
        let port = free_port();
        let env = ServerEnv { port, certs: &certs, tmp: dir.path() };
        let config_json = case.server_config(core.kind, &env);
        if std::fs::write(&config_path, &config_json).is_err() {
            return Err(format!("attempt {attempt}: config write failed"));
        }
        let _core = spawn_core(&core.bin, core.kind, &config_path, port);

        case.client_trust(&certs);
        let params = case.client_params(port, echo.addr);
        let mut tunnel = match crate::connect(params).await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[e2e] {}: attempt {attempt}/{ATTEMPTS} connect: {e}", case.label());
                continue;
            }
        };
        let (status, body) = probe(&mut tunnel).await;
        if status == expect.status && body == expect.body {
            return Ok(());
        }
        eprintln!(
            "[e2e] {}: attempt {attempt}/{ATTEMPTS} probe status {status} body {body:?}",
            case.label()
        );
    }
    Err(format!("{}: failed after {ATTEMPTS} attempts", case.label()))
}
```

- [ ] **Step 2: Write `src/protocol/vless/e2e.rs`** (real code, feature-gated)

```rust
//! E2E case for VLESS (TLS + TCP) — declarative input to the e2e pipeline.
#![cfg(feature = "native-e2e")]

use std::net::SocketAddr;

use xray_tui_proto::proto_spec::{ProtocolConfig, VlessConfig};

use crate::e2e::{Certs, CoreKind, E2eCase, E2eExpect, ServerEnv};
use crate::security;
use crate::{NativeConnectParams, addr};

pub struct VlessCase;

const UUID: &str = "00000000-0000-0000-0000-000000000000";
const BODY: &str = "hello native core";

impl E2eCase for VlessCase {
    fn label(&self) -> &'static str { "vless/tcp/tls" }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        let cert = env.certs.cert_pem.as_bytes();
        let key = env.certs.key_pem.as_bytes();
        let _ = (cert, key); // configs below reference PEM FILES on disk
        let cert_path = env.tmp.join("server.crt").to_string_lossy().into_owned();
        let key_path = env.tmp.join("server.key").to_string_lossy().into_owned();
        let json = match core {
            CoreKind::Xray => serde_json::json!({
                "inbounds": [{
                    "listen": "127.0.0.1", "port": env.port, "protocol": "vless",
                    "settings": { "clients": [{ "id": UUID }], "decryption": "none" },
                    "streamSettings": { "network": "tcp", "security": "tls",
                        "tlsSettings": { "certificates": [
                            { "certificateFile": cert_path, "keyFile": key_path }
                        ], "alpn": ["http/1.1"] } }
                }],
                "outbounds": [{ "protocol": "freedom" }]
            }),
            CoreKind::SingBox => serde_json::json!({
                "log": { "level": "warn" },
                "inbounds": [{ "type": "vless", "listen": "127.0.0.1", "listen_port": env.port,
                    "users": [{ "uuid": UUID }],
                    "tls": { "enabled": true, "certificate_path": cert_path, "key_path": key_path,
                        "alpn": ["http/1.1"] } }],
                "outbounds": [{ "type": "direct" }]
            }),
        };
        serde_json::to_string(&json).expect("vless server config serializes")
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": UUID,
            "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
            "transport": { "type": "tcp" }
        })).expect("vless client config parses");
        let server = xray_tui_proto::proto_spec::endpoint::EndpointEssentials::new("127.0.0.1", port);
        NativeConnectParams::new(protocol, server, crate::addr::TargetAddr::new(
            crate::addr::Host::Ip(target.ip()), target.port()))
    }

    fn expected(&self) -> E2eExpect { E2eExpect { status: 200, body: BODY.into() } }

    fn client_trust(&self, certs: &Certs) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));
    }
}

impl Default for VlessCase { fn default() -> Self { Self } }
```

Note: `VlessConfig` import above is used by the JSON schema name check — the JSON `"schema": "Vless"` is what `serde` maps to `ProtocolConfig::Vless`. Remove the unused `Xray`/`VlessConfig` imports if clippy complains; keep the JSON shape authoritative.

- [ ] **Step 3: Register the module** in `src/protocol/vless/mod.rs`: `#[cfg(feature = "native-e2e")] pub mod e2e;`

- [ ] **Step 4: Write `tests/vless.rs`** and delete the old tests + common

```rust
//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{run, CoreKind, CoreUnderTest};
use xray_tui_native::protocol::vless::e2e::VlessCase;

const SINGBOX_VERSION: &str = "1.13.16";
const XRAY_VERSION: &str = "26.3.27";

#[tokio::test]
async fn vless_tcp_tls_against_xray() {
    let core = CoreUnderTest::resolve(CoreKind::Xray, XRAY_VERSION).unwrap();
    run(&VlessCase::default(), &core).await.expect("vless/xray e2e failed");
}

#[tokio::test]
async fn vless_tcp_tls_against_singbox() {
    let core = CoreUnderTest::resolve(CoreKind::SingBox, SINGBOX_VERSION).unwrap();
    run(&VlessCase::default(), &core).await.expect("vless/singbox e2e failed");
}
```

Delete `tests/common/`, `tests/vless_singbox.rs`, `tests/vless_xray.rs` via `git rm -r`.

- [ ] **Step 5: Run the migrated suite**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vless`
Expected: 2 tests pass (`test result: ok`). If a test hangs > 2 min, every step in the driver is bounded (spawn readiness 10s, connect ≤ 30s, probe 5s+5s, 3 attempts) — a true hang is a bug in the driver; then re-check the bounded awaits.

- [ ] **Step 6: Commit**

```bash
git rm -q -r crates/xray-tui-native/tests/common crates/xray-tui-native/tests/vless_singbox.rs crates/xray-tui-native/tests/vless_xray.rs
git add crates/xray-tui-native/src/e2e crates/xray-tui-native/src/protocol/vless/e2e.rs crates/xray-tui-native/src/protocol/vless/mod.rs crates/xray-tui-native/tests/vless.rs
git commit -m "feat(native): e2e driver + migrate VLESS to declarative pipeline tests"
```

---

### Task 4: VMess key derivation module

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vmess/keys.rs`, `crates/xray-tui-native/src/protocol/vmess/mod.rs` (replace placeholder text; will grow in Task 7)
- Modify: `crates/xray-tui-native/src/protocol/mod.rs` — no change yet (dispatch swaps in Task 7)

**Interfaces:**
- Produces:
```rust
pub const VMESS_SALT: &str = "c48619fe-8f02-49e0-b9e9-edf763e17e21";
pub fn cmd_key(uuid_bytes: &[u8; 16]) -> [u8; 16];          // md5(uuid ‖ VMESS_SALT)
pub fn kdf(key: &[u8], path: &[&str]) -> [u8; 32];          // HMAC-SHA256 nesting
pub fn kdf16(key: &[u8], path: &[&str]) -> [u8; 16];
pub fn fnv1a32(data: &[u8]) -> u32;
pub fn crc32_ieee(data: &[u8]) -> u32;
pub fn auth_id(cmd_key: &[u8; 16], ts_secs: i64, rand4: &[u8; 4]) -> [u8; 16];
pub fn sha256_first16(data: &[u8]) -> [u8; 16];
```

- [ ] **Step 1: Write the failing tests** (golden vectors, computed from Go semantics)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_UUID: [u8; 16] = [0; 16];

    #[test]
    fn cmd_key_matches_go_id_new() {
        // Go: NewID(uuid.Zero) cmdKey (computed via md5(uuid ‖ const))
        assert_eq!(
            hex(cmd_key(&ZERO_UUID)),
            "5e20f3239545e3f48e0ff445aa7c4c3b"
        );
    }

    #[test]
    fn kdf_matches_go_nested_hmac() {
        let ck = cmd_key(&ZERO_UUID);
        // Go aead.KDF16(ck, "AES Auth ID Encryption")
        assert_eq!(hex(kdf16(&ck, &["AES Auth ID Encryption"])), "e4e63970ccc1e39dd4a315d51b66abf7");
        // Go aead.KDF16(ck, "AEAD Resp Header Len Key")
        assert_eq!(hex(kdf16(&ck, &["AEAD Resp Header Len Key"])), "2e2b7503835a5934f683a5f84910e211");
    }

    #[test]
    fn fnv1a32_matches_go() {
        // sample request-body prefix used for the golden: version|iv|key|resp|opt|sec|0|cmd|addr
        let mut body = vec![1u8];
        body.extend_from_slice(&[0x11; 16]);
        body.extend_from_slice(&[0x22; 16]);
        body.extend_from_slice(&[0x33, 0x00, 0x00, 0x00, 0x01]);
        body.extend_from_slice(&[0x01, 127, 0, 0, 1]);
        body.extend_from_slice(&0u16.to_be_bytes());
        assert_eq!(fnv1a32(&body), 0x51e818a9);
    }

    #[test]
    fn auth_id_encrypts_ecb_golden() {
        let ck = cmd_key(&ZERO_UUID);
        // ts=0x6000000000000000, rand=0xaabbccdd -> crc32 0x277774dc; key = KDF16(ck, salt)
        // golden ciphertext (openssl aes-128-ecb): 5659b5bba7891c58aad644346470c210
        let id = auth_id(&ck, 0x6000_0000_0000_0000, &[0xaa, 0xbb, 0xcc, 0xdd]);
        assert_eq!(hex(id), "5659b5bba7891c58aad644346470c210");
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
```

- [ ] **Step 2: Run to see them fail**

Run: `cargo test -p xray-tui-native --lib vmess`
Expected: FAIL — `mod vmess` is a stub with no `keys`.

- [ ] **Step 3: Implement `keys.rs`** (real code)

```rust
//! VMess key derivation (v2ray `proxy/vmess/aead` + `common/protocol/id.go`).
use md5::Md5;
use md5::digest::Digest as _;
use sha2::Sha256;

pub const VMESS_SALT: &str = "c48619fe-8f02-49e0-b9e9-edf763e17e21";
const KDF_ROOT: &str = "VMess AEAD KDF";

/// cmdKey = md5(uuid_bytes ‖ VMESS_SALT) — Go `protocol.NewID`.
pub fn cmd_key(uuid_bytes: &[u8; 16]) -> [u8; 16] {
    let mut m = Md5::new();
    m.update(uuid_bytes);
    m.update(VMESS_SALT.as_bytes());
    m.finalize().into()
}

/// Nested HMAC-SHA256: `HMAC(p[n], … HMAC(p[1], HMAC(KDF_ROOT, key)))`.
/// Go `aead.KDF` — message chains root → key at the innermost position.
pub fn kdf(key: &[u8], path: &[&str]) -> [u8; 32] {
    let mut msg: [u8; 32] = hmac_sha256(KDF_ROOT.as_bytes(), key);
    for v in path {
        msg = hmac_sha256(v.as_bytes(), &msg);
    }
    msg
}

pub fn kdf16(key: &[u8], path: &[&str]) -> [u8; 16] {
    let full = kdf(key, path);
    let mut out = [0u8; 16];
    out.copy_from_slice(&full[..16]);
    out
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("hmac key any size");
    mac.update(msg);
    mac.finalize().into_bytes().into()
}

/// FNV-1a 32-bit (Go `hash/fnv`).
pub fn fnv1a32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in data {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// CRC-32 IEEE (Go `hash/crc32`).
pub fn crc32_ieee(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Request AuthID: AES-128-ECB(key = KDF16(cmdKey, "AES Auth ID Encryption"),
/// block = [ts_be][rand4][crc32(ts_be ‖ rand4)]). Go `aead.CreateAuthID`.
pub fn auth_id(cmd_key: &[u8; 16], ts_secs: i64, rand4: &[u8; 4]) -> [u8; 16] {
    use aes::cipher::{BlockEncrypt, KeyInit};
    let key = kdf16(cmd_key, &["AES Auth ID Encryption"]);
    let mut block = [0u8; 16];
    block[..8].copy_from_slice(&ts_secs.to_be_bytes());
    block[8..12].copy_from_slice(rand4);
    let crc = crc32_ieee(&block[..12]);
    block[12..].copy_from_slice(&crc.to_be_bytes());
    let cipher = aes::Aes128::new_from_slice(&key).expect("16-byte key");
    let mut out = [0u8; 16];
    cipher.encrypt_block((&mut out).into(), (&block).into());
    out
}

/// sha256(data)[..16] — responseBodyKey/IV derivation (Go `crypto/sha256`).
pub fn sha256_first16(data: &[u8]) -> [u8; 16] {
    let d = Sha256::digest(data);
    let mut out = [0u8; 16];
    out.copy_from_slice(&d[..16]);
    out
}
```

- [ ] **Step 4: `vmess/mod.rs`** — keep the placeholder doc, add `pub mod keys;`

- [ ] **Step 5: Run to verify**

Run: `cargo test -p xray-tui-native --lib vmess`
Expected: all 4 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess
git commit -m "feat(native): VMess key derivation — cmdKey/KDF/authID/FNV/CRC with Go golden vectors"
```

---

### Task 5: VMess request header codec (body + seal)

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vmess/header.rs`

**Interfaces:**
- Consumes: `keys::{cmd_key, kdf, kdf16, fnv1a32, auth_id, sha256_first16}`, `addr::{TargetAddr, encode_addr}` (check `encode_addr`'s exact signature in `src/addr.rs`; VLESS uses it — reuse), `NativeError`, `timeouts`.
- Produces:
```rust
pub const VERSION: u8 = 1;
pub const SECURITY_AES128_GCM: u8 = 3;
pub const COMMAND_TCP: u8 = 1;

pub struct Session {
    pub request_body_iv: [u8; 16],
    pub request_body_key: [u8; 16],
    pub response_header: u8,
    pub response_body_key: [u8; 16],
    pub response_body_iv: [u8; 16],
}
impl Session { pub fn new() -> Self }          // CSPRNG IV/Key/responseHeader; sha256-derive response keys

pub fn encode_request(cmd_key: &[u8; 16], session: &Session,
                      target: &TargetAddr, auth_ts: i64, entropy: &mut impl FnMut(&mut [u8])) -> Vec<u8>;
pub fn peek_seal_len(body_len: usize) -> usize;  // 16 + 18 + 8 + body_len + 16
```

Wire (Go `encoding/client.go` + `aead/encrypt.go`): wire = `authID(16) | lenAEAD(18) | nonce8 | payloadAEAD`; payloadAEAD plaintext = `version 1 | IV16 | key16 | responseHeader1 | option 0 | (padding 0) | security 3 | 0 | command 1 | addr | port BE2 | fnv1a32 BE`. Addr bytes: type 1/2/3, payload, then port — the PORT-LAST ordering is the default of the VMess `AddressParser` (`PortThenAddress` is only for VLESS; VMess uses the zero-value order = address then port). Confirm against `thirdparty/Xray-core/proxy/vmess/encoding/encoding.go` (it has no `PortThenAddress` option → default order).

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{Host, TargetAddr};
    use crate::protocol::vmess::keys::cmd_key;

    fn fixed_session() -> Session {
        let mut s = Session::new();
        s.request_body_iv = [0x11; 16];
        s.request_body_key = [0x22; 16];
        s.response_header = 0x33;
        s.response_body_key = [0x44; 16];
        s.response_body_iv = [0x55; 16];
        s
    }

    /// Deterministic entropy: the 4-byte auth-rand must be aabbccdd for the
    /// authID golden; the 8-byte connection nonce and anything else = 0xAB.
    fn fixed_entropy(out: &mut [u8]) {
        if out.len() == 4 {
            out.copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
        } else {
            out.fill(0xAB);
        }
    }

    #[test]
    fn request_wire_lengths() {
        let ck = cmd_key(&[0; 16]);
        let tgt = TargetAddr::new(Host::Ip("127.0.0.1".parse().unwrap()), 80);
        let s = fixed_session();
        let wire = encode_request(&ck, &s, &tgt, 0x6000_0000_0000_0000, &mut fixed_entropy);
        // body = 38 fixed + addr(1+4) + 2 + fnv 4 = 49; payloadAEAD = 49+16; lenAEAD 18; authID 16; nonce 8
        assert_eq!(wire.len(), 16 + 18 + 8 + 49 + 16);
        // authID golden (ts=0x6000000000000000, rand=aabbccdd, fixed cmdKey)
        assert_eq!(&wire[..16], &hex_decode("5659b5bba7891c58aad644346470c210"));
    }

    #[test]
    fn request_contains_session_material() {
        // bytes after the 16-byte authID are ciphertext — session IV/key never
        // appear in plaintext anywhere on the wire (wire[42..] is sealed).
        let ck = cmd_key(&[0; 16]);
        let tgt = TargetAddr::new(Host::Ip("127.0.0.1".parse().unwrap()), 80);
        let s = fixed_session();
        let wire = encode_request(&ck, &s, &tgt, 42, &mut fixed_entropy);
        assert!(!wire[..16].iter().any(|b| *b == 0x11 || *b == 0x22)); // authID is ciphertext
        assert_eq!(wire.len(), 16 + 18 + 8 + 49 + 16);
    }

    fn hex_decode(h: &str) -> Vec<u8> {
        (0..h.len()).step_by(2).map(|i| u8::from_str_radix(&h[i..i+2], 16).unwrap()).collect()
    }
}
```

- [ ] **Step 2: Run to see them fail**

Run: `cargo test -p xray-tui-native --lib vmess::header`
Expected: FAIL — module absent.

- [ ] **Step 3: Implement `header.rs`** (real code)

```rust
//! VMess AEAD request header codec (v2ray `proxy/vmess/encoding/client.go`
//! + `proxy/vmess/aead/encrypt.go`).

use aes::cipher::{KeyIvInit, KeyInit};
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes128Gcm, Nonce};

use crate::addr::TargetAddr;
use crate::error::NativeError;
use crate::protocol::vmess::keys;

pub const VERSION: u8 = 1;
pub const SECURITY_AES128_GCM: u8 = 3;
pub const COMMAND_TCP: u8 = 1;

/// Per-connection VMess session material (mirrors Go `ClientSession`).
pub struct Session {
    pub request_body_iv: [u8; 16],
    pub request_body_key: [u8; 16],
    pub response_header: u8,
    pub response_body_key: [u8; 16],
    pub response_body_iv: [u8; 16],
}

impl Session {
    pub fn new() -> Self {
        use rand_bytes as rng; // see below
        let mut iv = [0u8; 16];
        let mut key = [0u8; 16];
        rng(&mut iv);
        rng(&mut key);
        let mut resp_header = [0u8; 1];
        rng(&mut resp_header);
        let response_body_key = keys::sha256_first16(&key);
        let response_body_iv = keys::sha256_first16(&iv);
        Self { request_body_iv: iv, request_body_key: key, response_header: resp_header[0], response_body_key, response_body_iv }
    }
}

/// CSPRNG fill (tiny helper; the VMess session is per-connection randomness).
fn rand_bytes(out: &mut [u8]) {
    // tokio doesn't provide rand; use the same provider the crate trusts.
    // rustls ring is available: use its random.
    use ring::rand::{SecureRandom, SystemRandom};
    SystemRandom::new().fill(out).expect("rng failure");
}

/// Encode the sealed AEAD request header for a TCP command.
///
/// `entropy` supplies the 4-byte auth-rand and 8-byte connection nonce (fixed
/// for tests). The header body (Go `EncodeRequestHeader`):
/// version | IV | key | respHeader | option=0 | security=3 | 0 | cmd=1 | addr | port | fnv1a32.
#[must_use]
pub fn encode_request(
    cmd_key: &[u8; 16],
    session: &Session,
    target: &TargetAddr,
    auth_ts: i64,
    entropy: &mut impl FnMut(&mut [u8]),
) -> Vec<u8> {
    // --- plaintext header body ---
    let mut body = Vec::with_capacity(49);
    body.push(VERSION);
    body.extend_from_slice(&session.request_body_iv);
    body.extend_from_slice(&session.request_body_key);
    body.push(session.response_header);
    body.push(0); // option
    body.push(SECURITY_AES128_GCM); // padding nibble 0 | security 3
    body.push(0); // reserved
    body.push(COMMAND_TCP);
    encode_address_port(&mut body, target);
    // padding: length 0 (we send the minimal body)
    let fnv = keys::fnv1a32(&body);
    body.extend_from_slice(&fnv.to_be_bytes());

    // --- randomness for the seal ---
    let mut auth_rand = [0u8; 4];
    entropy(&mut auth_rand);
    let auth_id = keys::auth_id(cmd_key, auth_ts, &auth_rand);
    let mut conn_nonce = [0u8; 8];
    entropy(&mut conn_nonce);
    let auth_id_s = String::from_utf8_lossy(&auth_id).into_owned();

    // --- length AEAD ---
    let len_key = keys::kdf16(cmd_key, &["VMess Header AEAD Key_Length", &auth_id_s, &String::from_utf8_lossy(&conn_nonce)]);
    // NOTE: kdf path takes &str; encode nonce/authid as lossy-str slices is WRONG —
    // Go passes the raw BYTES as the hmac key, not a string. Use byte paths.
    // See kdf16_bytes variant below.
    unreachable!("use byte-path KDF — see Step 3 note")
}
```

**Step 3 note (read before coding):** Go's `KDF(key, path...)` receives authID/nonce as raw 16/8 BYTES via `string(...)` conversion — the bytes are the HMAC key material, not UTF-8 text. Therefore extend `keys::kdf` to a byte-path form:

```rust
// keys.rs addition:
pub fn kdf_bytes_path(key: &[u8], path: &[&[u8]]) -> [u8; 32] {
    let mut msg: [u8; 32] = hmac_sha256(KDF_ROOT.as_bytes(), key);
    for v in path {
        msg = hmac_sha256(v, &msg);
    }
    msg
}
pub fn kdf16_bytes_path(key: &[u8], path: &[&[u8]]) -> [u8; 16] {
    let full = kdf_bytes_path(key, path);
    let mut out = [0u8; 16];
    out.copy_from_slice(&full[..16]);
    out
}
```

Re-implement `encode_request` using `kdf16_bytes_path(cmd_key, &[b"VMess Header AEAD Key_Length", &auth_id, &conn_nonce])` (and the `..._Nonce_Length` / `...AEAD Key` / `...AEAD Nonce` variants), `aes_gcm::Aes128Gcm` with 12-byte nonces, `Payload { msg: &body, aad: &auth_id }`, lenAEAD sealing `(body.len() as u16).to_be_bytes()` with AAD `auth_id`, output `auth_id ‖ len_cipher ‖ nonce ‖ body_cipher`. The `keys::auth_id` golden test stays valid (that KDF path is pure strings).

Add a unit test asserting the byte-path KDF agrees with string-path KDF when path entries are ASCII text (so `kdf16` string form remains usable for the response-header salts).

- [ ] **Step 4: Run to verify**

Run: `cargo test -p xray-tui-native --lib vmess`
Expected: header tests PASS (lengths + authID golden + no-plaintext-leak).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess
git commit -m "feat(native): VMess AEAD request header codec (seal + session)"
```

---

### Task 6: VMess record tunnel stream (payload AEAD + response-header peel)

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vmess/stream.rs`
- Modify: `crates/xray-tui-native/src/protocol/vmess/mod.rs` (add `pub mod stream;`)

**Interfaces:**
- Consumes: `header::Session`, `keys::{kdf16_bytes_path, sha256_first16}`.
- Produces:
```rust
pub struct VmessClientStream { inner: BoxStream, session: Session,
    // read side
    read_state: ReadState, pending: Vec<u8>, pending_pos: usize,
    // write side
    write_pending: Option<Vec<u8>>, req_nonce: [u8; 12], req_counter: u16, resp_nonce: [u8; 12], resp_counter: u16 }
impl tokio::io::AsyncRead / AsyncWrite for VmessClientStream
```
- Record framing both directions: `2B BE len | AES-128-GCM(key=session.{request,response}_body_key, nonce=IV-with-first-2-bytes-overwritten-by-BE-counter)` — write direction uses request keys + its own counter; read direction response keys + counter. Nonce = first 12 bytes of the modified IV (Go `GenerateChunkNonce`).
- Read side starts in `PeelHeader` state: read lenAEAD(18) → decrypt len (key `kdf16_bytes_path(resp_key, &[b"AEAD Resp Header Len Key"])`, nonce `kdf(resp_iv, [b"AEAD Resp Header Len IV"])[:12]`, AAD none) → read `len+16` → decrypt (key `...Resp Header Key`, nonce `...Resp Header IV`[:12]) → assert payload[0] == session.response_header → then record loop.

- [ ] **Step 1: Write the failing tests** (roundtrip through a duplex pair)

```rust
#[cfg(test)]
mod tests {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::protocol::vmess::header::Session;
    use crate::protocol::vmess::keys::{kdf16_bytes_path, sha256_first16};

    /// Server-side response header seal (mirrors Go `EncodeResponseHeader`):
    /// lenAEAD(18) + payloadAEAD(len+16), both AES-128-GCM, AAD nil, nonce = KDF [:12].
    fn seal_response_header(key: &[u8; 16], iv: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        fn ae(key: &[u8; 16], iv: &[u8; 16], key_salt: &str, iv_salt: &str, msg: &[u8]) -> Vec<u8> {
            let k = kdf16_bytes_path(key, &[key_salt.as_bytes()]);
            let v = kdf16_bytes_path(iv, &[iv_salt.as_bytes()]);
            Aes128Gcm::new_from_slice(&k)
                .unwrap()
                .encrypt(Nonce::from_slice(&v[..12]), msg)
                .unwrap()
        }
        let mut wire = ae(
            key, iv,
            "AEAD Resp Header Len Key", "AEAD Resp Header Len IV",
            &(payload.len() as u16).to_be_bytes(),
        );
        wire.extend_from_slice(&ae(key, iv, "AEAD Resp Header Key", "AEAD Resp Header IV", payload));
        wire
    }

    /// Server-side response record (mirrors Go `EncodeResponseBody` chunk):
    /// 2B BE length + AES-128-GCM data, counter nonce from the response IV.
    fn seal_record(key: &[u8; 16], iv: &[u8; 16], counter: u16, data: &[u8]) -> Vec<u8> {
        let mut nonce = *iv;
        nonce[..2].copy_from_slice(&counter.to_be_bytes());
        let ct = Aes128Gcm::new_from_slice(key)
            .unwrap()
            .encrypt(Nonce::from_slice(&nonce[..12]), data)
            .unwrap();
        (data.len() as u16).to_be_bytes().into_iter().chain(ct).collect()
    }

    #[test]
    fn records_roundtrip_with_peel() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (client_side, mut server_side) = tokio::io::duplex(8192);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);

            // fake server: response header ([resp_header, option 0, cmd 0, len 0])
            // then one data record with counter 0.
            let mut wire = seal_response_header(
                &session.response_body_key,
                &session.response_body_iv,
                &[0x99, 0, 0, 0],
            );
            wire.extend_from_slice(&seal_record(
                &session.response_body_key,
                &session.response_body_iv,
                0,
                b"hello",
            ));
            server_side.write_all(&wire).await.unwrap();
            drop(server_side); // EOF after the data — read_to_end can finish

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            let mut got = Vec::new();
            tunnel.read_to_end(&mut got).await.unwrap();
            assert_eq!(got, b"hello");
        });
    }

    #[test]
    fn peel_rejects_wrong_echo_byte() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (client_side, mut server_side) = tokio::io::duplex(8192);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);

            // wrong echo byte 0x98 -> must error, not stream garbage
            let wire = seal_response_header(
                &session.response_body_key,
                &session.response_body_iv,
                &[0x98, 0, 0, 0],
            );
            server_side.write_all(&wire).await.unwrap();
            drop(server_side);

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            let mut got = Vec::new();
            let res = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tunnel.read_to_end(&mut got),
            )
            .await;
            assert!(res.is_err() || matches!(res.unwrap(), Err(_)), "must fail, got {got:?}");
        });
    }
}
```

The `seal_response_header`/`seal_record` helpers are the authoritative server-side mirror (Go `EncodeResponseHeader`/response-chunk) — independent validation of the client peel and record decoder.

- [ ] **Step 2: Run to see it fail**

Run: `cargo test -p xray-tui-native --lib vmess::stream`
Expected: FAIL — module absent.

- [ ] **Step 3: Implement `stream.rs`**

Copy the state-machine shape of `protocol/vless/stream.rs` (peel enum + pending buffers + `AsyncRead`/`AsyncWrite` impls poll-loops). Differences: the peel is AEAD (read exact 18 → decrypt len → read+decrypt until payload → echo-check byte 0 vs `session.response_header`), and after the peel every read goes through the record decoder, every write through the record encoder:

- **Read record**: await exactly `2` bytes → len (BE); if len == 0 → next record; await `len + 16` → decrypt with current resp nonce → append; increment counter; loop until the caller's buffer is satisfied.
- **Write record**: on `poll_write`, if `write_pending` is None, build `[2B len][ciphertext+tag]` from the caller's buffer (chunk = the whole buffer; empty write → record with len 0 or skip — mirror xray: skip empty), write it with the inner `poll_write` loop, returning `Ok(n)` for the ORIGINAL length once fully flushed; `poll_flush`/`poll_shutdown` forward to inner.
- Nonce bookkeeping: `let mut nonce = session.{req,resp}_body_iv; nonce[0..2] = counter.to_be_bytes();` per record, counter `u16` wrapping.

- [ ] **Step 4: Run to verify**

Run: `cargo test -p xray-tui-native --lib vmess`
Expected: stream roundtrip test PASSES (peel + record decode + echo check).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess
git commit -m "feat(native): VMess record tunnel — payload AEAD + response-header peel"
```

---

### Task 7: `vmess::connect` + config validation + dispatch

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vmess/mod.rs`, `crates/xray-tui-native/src/protocol/mod.rs`

**Interfaces:**
- Consumes: `header::{Session, encode_request, COMMAND_TCP}`, `stream::VmessClientStream`, `LinkContext`, `NativeError`/`timeouts`, proto `VmessConfig` (fields `uuid`, `security.enc`).
- Produces: `pub async fn connect(ctx: &LinkContext, stream: BoxStream, cfg: &VmessConfig) -> Result<BoxStream, NativeError>`.

- [ ] **Step 1: Write the failing validation tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_proto::proto_spec::ProtocolConfig;

    fn vcfg(enc: &str) -> VmessConfig {
        serde_json::from_value(serde_json::json!({
            "schema": "Vmess",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": { "enc": enc },
            "transport": { "type": "tcp" }
        })).expect("vmess config parses")
    }

    #[test]
    fn rejects_unsupported_payload_securities() {
        for enc in ["none", "zero", "chacha20-poly1305"] {
            let cfg = vcfg(enc);
            assert!(matches!(check_security(&cfg), Err(NativeError::Config(_))), "{enc}");
        }
    }

    #[test]
    fn accepts_aes128_gcm_and_auto() {
        assert!(check_security(&vcfg("aes-128-gcm")).is_ok());
        assert!(check_security(&vcfg("auto")).is_ok());
        assert!(check_security(&vcfg("")).is_ok()); // absent -> auto
    }
}
```

- [ ] **Step 2: Run to see them fail**

Run: `cargo test -p xray-tui-native --lib vmess::mod`
Expected: FAIL — `check_security` absent.

- [ ] **Step 3: Implement `vmess/mod.rs`**

```rust
//! `VMess` — native client (modern AEAD, xtls dialect).
//!
//! Wire contract: `thirdparty/Xray-core/proxy/vmess/encoding/{client,encoding,server}.go`
//! + `proxy/vmess/aead/` (MIT). Only TCP + AES-128-GCM payload security;
//! xray-core 26.x refuses `none`/`zero`/`auto` body streams server-side.

use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::VmessConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vmess::header::{Session, encode_request, COMMAND_TCP};
use crate::protocol::vmess::keys::cmd_key;
use crate::protocol::vmess::stream::VmessClientStream;

pub mod keys;
pub mod header;
pub mod stream;

/// Validate the VMess payload security the config requests.
pub fn check_security(cfg: &VmessConfig) -> Result<(), NativeError> {
    let enc = cfg.security.enc.clone();
    match enc.as_deref() {
        None | Some("") | Some("auto") | Some("aes-128-gcm") => Ok(()),
        Some(other) => Err(NativeError::Config(format!(
            "vmess payload security {other:?} not supported (native core: aes-128-gcm only)"
        ))),
    }
}

/// Connect through a VMess outbound over an already-secured stream.
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VmessConfig,
) -> Result<BoxStream, NativeError> {
    check_security(cfg)?;
    let uuid = crate::protocol::vless::header::uuid_bytes(&cfg.uuid)?;
    let ck = cmd_key(&uuid);
    let session = Session::new();

    let mut entropy = |out: &mut [u8]| {
        use ring::rand::{SecureRandom, SystemRandom};
        SystemRandom::new().fill(out).expect("rng failure");
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let request = encode_request(&ck, &session, &ctx.target, ts, &mut entropy);
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout { step: "vmess request write", limit: timeout })??;

    Ok(Box::new(VmessClientStream::new(stream, session)))
}
```

Note: `security.enc` is `Option<TinyText>` in `SecurityConfig`; use `cfg.security.enc.as_deref()` (adjust to the actual field type — read `VmessConfig` in `crates/xray-tui-proto/src/proto_spec/vmess.rs` first). If `SecurityConfig::type_str`/`enc` accessors differ, use the typed accessors.

- [ ] **Step 4: Swap the dispatch arm** in `protocol/mod.rs`

```rust
ProtocolConfig::Vmess(cfg) => vmess::connect(ctx, stream, cfg).await,
```

- [ ] **Step 5: Verify shape + build**

Confirm `connect_shape(ProtocolKind::Vmess)` == `TcpStream` (read the match in `src/shape.rs`; it is exhaustive, so the arm already exists). Run: `cargo test -p xray-tui-native --lib`
Expected: VMess validation tests pass; existing 35+ tests still green.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess crates/xray-tui-native/src/protocol/mod.rs
git commit -m "feat(native): VMess connect + payload-security validation, wire dispatch"
```

---

### Task 8: VMess e2e case + tests

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vmess/e2e.rs`, `crates/xray-tui-native/tests/vmess.rs`
- Modify: `crates/xray-tui-native/src/protocol/vmess/mod.rs` (register `e2e`)

**Interfaces:**
- Consumes: `e2e::{E2eCase, E2eExpect, ServerEnv, CoreKind}`, `VmessConfig` JSON shape (schema `"Vmess"`), `vless::e2e` pattern.
- Produces: `pub struct VmessCase;` with `impl E2eCase` — server configs include an **inbound VMess user with `security: "aes-128-gcm"`**, client params with `"security": { "enc": "aes-128-gcm", "tls": {...} }`.

- [ ] **Step 1: Write `src/protocol/vmess/e2e.rs`** — copy the `VlessCase` skeleton; differences:
  - `label()` → `"vmess/tcp/tls"`.
  - xray server inbound: `"protocol": "vmess"` with `"settings": { "clients": [{ "id": UUID, "security": "aes-128-gcm" }] }` (keep `decryption` OUT — VMess settings take `security` per client).
  - sing-box server inbound: `"type": "vmess"`, `"users": [{ "uuid": UUID, "security": "aes-128-gcm" }]`, TLS block identical to VLESS.
  - client params JSON:
```json
{ "schema": "Vmess", "uuid": "00000000-0000-0000-0000-000000000000",
  "security": { "enc": "aes-128-gcm", "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
  "transport": { "type": "tcp" } }
```
  (Confirm `SecurityConfig` serde shape: `enc` sits beside the flattened `tls` — see `proto_spec/common.rs` `SecurityConfig`.)

- [ ] **Step 2: Write `tests/vmess.rs`** — same shape as `tests/vless.rs`, `VmessCase`, both cores, same version constants.

- [ ] **Step 3: First run — xray only**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vmess vmess_tcp_tls_against_xray -- --nocapture`
Expected: `test result: ok` (proves the whole AEAD header + records against xtls).

- [ ] **Step 4: Full VMess run (both cores)**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vmess`
Expected: 2/2 pass. If the sing-box case fails with an AEAD/auth error: the xtls header-body dialect (responseHeader byte / padding / FNV) is being rejected by sing-vmess — see Spec Risks; first try dropping the FNV+padding correctness assumption (re-verify against `sing-vmess` decode source), then adjust `encode_request` (never weaken the xray side).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess crates/xray-tui-native/tests/vmess.rs
git commit -m "feat(native): VMess e2e case + tests against xray + sing-box"
```

---

### Task 9: Chain integration + extra unit vectors

**Files:**
- Modify: `crates/xray-tui-native/src/chain.rs` (note only — verify `connect_chain` folds VMess through `protocol::connect` unchanged), `crates/xray-tui-native/src/protocol/vmess/keys.rs` (add `kdf16_bytes_path`)
- Create: `crates/xray-tui-native/src/protocol/vmess/keys.rs` tests (if not already added in Task 5)

**Interfaces:**
- Consumes: Task 5's note-added `kdf_bytes_path`/`kdf16_bytes_path`.

- [ ] **Step 1: Add byte-path KDF + consistency test** (if not already done in Task 5)

```rust
#[test]
fn bytes_path_matches_string_path_for_ascii() {
    let key = [0x42; 16];
    let a = kdf16(&key, &["AEAD Resp Header Len Key"]);
    let b = kdf16_bytes_path(&key, &[b"AEAD Resp Header Len Key"]);
    assert_eq!(a, b);
}
```

- [ ] **Step 2: Chain note** — `connect_chain` needs no change: VMess arrives via `protocol::connect` like VLESS. Verify by reading `chain.rs` (no edit expected).

- [ ] **Step 3: Full unit suite**

Run: `cargo test -p xray-tui-native`
Expected: all pass (35 previous + VMess additions).

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess
git commit -m "test(native): VMess byte-path KDF consistency vector"
```

---

### Task 10: Final verification + ledger

**Files:**
- Modify: `.superpowers/sdd/native-vmess-e2e-pipeline/progress.md` (create dir + ledger)

- [ ] **Step 1: Full workspace suite**

Run: `cargo test`
Expected: 864+ tests pass, 0 failed (workspace).

- [ ] **Step 2: Lint + format**

Run: `cargo clippy -p xray-tui-native --all-targets && cargo fmt -p xray-tui-native -- --check`
Expected: 0 warnings; fmt clean.

- [ ] **Step 3: Interop sweep — both protocols, both cores**

Run:
```bash
XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vless --test vmess
```
Expected: 4/4 pass.

- [ ] **Step 4: Ledger**

Append to `.superpowers/sdd/native-vmess-e2e-pipeline/progress.md`: task list with commit hashes, e2e results (4/4), versions (sing-box 1.13.16, xray 26.3.27), and any dialect notes from Task 8 Step 4.

- [ ] **Step 5: Commit**

```bash
git add .superpowers/sdd/native-vmess-e2e-pipeline/progress.md
git commit -m "docs(native): VMess + e2e pipeline — verification ledger"
```

---

## File Map

| File | Responsibility |
|---|---|
| `crates/xray-tui-native/Cargo.toml` | VMess crypto deps; feature-gated e2e deps |
| `crates/xray-tui-native/src/e2e/mod.rs` | trait, `run()` driver, retry policy |
| `crates/xray-tui-native/src/e2e/core.rs` | CoreKind/CoreUnderTest, version sanity |
| `crates/xray-tui-native/src/e2e/harness.rs` | echo, certs, core spawn/readiness, probe (migrated) |
| `crates/xray-tui-native/src/protocol/vless/e2e.rs` | VLESS declarative case |
| `crates/xray-tui-native/src/protocol/vmess/{keys,header,stream,mod,e2e}.rs` | VMess wire + case |
| `crates/xray-tui-native/tests/{vless,vmess}.rs` | declarative e2e tests |
| `crates/xray-tui-native/tests/common/` (deleted) | superseded by `src/e2e` |
| `crates/xray-tui-native/examples/native_connect_check.rs` (deleted) | superseded by pipeline |
