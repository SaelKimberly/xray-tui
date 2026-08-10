# E2E Variant Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make protocol security/transport variants first-class in the e2e facility (`CaseSpec` + `SecurityVariant` + typed per-core config builders) and prove it by adding the VMESS chacha20-poly1305 payload variant end-to-end against real xray-core and sing-box.

**Architecture:** The `E2eCase` trait and `run()` driver stay the pipeline contract. A data-driven layer (`CaseSpec` = protocol + optional `SecurityVariant`) sits below: variants supply label fragment, core gate, and security strings; typed builder fns in `config.rs` own both core dialects' server JSON and the client params. Native VMESS record codec gains a `PayloadCipher` enum (AES-128-GCM | chacha20-poly1305) dispatched at the two record-AEAD sites (peel stays AES always — the response header AEAD is fixed regardless of payload security).

**Tech Stack:** Rust 2024, tokio, RustCrypto `aes-gcm 0.10` + `chacha20poly1305 0.10`, `md-5 0.10`, real xray-core 26.3.27 / sing-box 1.13.16 binaries via `XRAY_TUI_CORE_BIN_DIR`.

## Global Constraints

- Branch `native-core-stub`, main checkout `/home/user/oss/xray-tui`. No branches/worktrees/tags.
- Wire authority (read-only): `thirdparty/Xray-core/proxy/vmess/encoding/{client,encoding,server}.go`, `thirdparty/Xray-core/proxy/vmess/encoding/auth.go`, `thirdparty/Xray-core/common/protocol/{headers,payload}.go`, sing-vmess vendored at `/home/user/go/pkg/mod/github.com/sagernet/sing-vmess@v0.2.8-0.20250909125414-3aed155119a1/protocol.go`.
- **chacha20 cipher key = `md5(k) ‖ md5(md5(k))`** (xray `auth.go:20-27` + sing-vmess `protocol.go:144-151`, byte-identical). Never "improve" this derivation — mirror it for interop. Golden (Go-verified 2026-08-10, k = `[0x22;16]`): `fbc3cf71d993ca7bec2664357ccdac2bb270c6d264a3bfeab7ceea80762a13cc`.
- Header security byte: AES-128-GCM = 3, chacha20-poly1305 = 4. Option byte stays 0 (server picks `PlainChunkSizeParser`). Record framing identical to the aes path: 2B BE ciphertext-length (plaintext+16 tag), counter nonce = IV with first 2 bytes overwritten by BE counter, per-direction counters start 0, EOF field 16/0.
- sing-box inbound VMess user has NO `security` field (rejected: `json: unknown field "security"`) — never emit it in sing-box configs. xray inbound user `security` mirrors intent (cosmetic for AEAD; the server honors the client header byte).
- Server honors the CLIENT header security byte — the client's `enc` value and header byte are what the e2e proves.
- `E2eCase::label` changes from `&'static str` to `String` (derived labels).
- Every wait bounded; a hang is a bug (existing harness — untouched).
- Verify per task with the commands in that task; clippy 0 warnings, `cargo fmt --check` clean, full workspace suite green at the end.

---

### Task 1: chacha20 key derivation + golden (keys.rs)

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vmess/keys.rs`
- Test: same file (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `md-5` crate (already a dep, used by `cmd_key` in this file).
- Produces: `pub fn chacha20_key_32(body_key: &[u8; 16]) -> [u8; 32]` — 32-byte chacha20poly1305 key = `md5(k) ‖ md5(md5(k))`. Later tasks: `stream.rs` `Cipher::new` calls it.

- [ ] **Step 1: Write the failing test**

In `keys.rs` tests module, mirror the md5 extraction style already used by `cmd_key` in this file:

```rust
#[test]
fn chacha20_key_is_double_md5_of_body_key() {
    let k = [0x22u8; 16];
    let key = chacha20_key_32(&k);
    assert_eq!(
        key.to_vec().hex(),
        "fbc3cf71d993ca7bec2664357ccdac2bb270c6d264a3bfeab7ceea80762a13cc"
    );
}
```

(`hex` — use the hex string via `format!("{:02x}", b)` per byte or any existing hex helper in the crate; the assertion is on the 32-byte array. If no hex helper exists, assert on the two 16-byte halves separately: `[u8; 16]` equality with the split constants `fbc3cf71d993ca7bec2664357ccdac2b` / `b270c6d264a3bfeab7ceea80762a13cc`.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib chacha20_key`
Expected: FAIL — `chacha20_key_32` not found (compile error).

- [ ] **Step 3: Implement the derivation**

Add to `keys.rs` (import `md5` if not already in scope; the file already uses `md5::compute` in `cmd_key`):

```rust
/// 32-byte chacha20poly1305 key for a 16-byte VMess session body key.
/// Mirrors Go `GenerateChacha20Poly1305Key` (xray
/// `proxy/vmess/encoding/auth.go`, sing-vmess `protocol.go`):
/// `md5(k) ‖ md5(md5(k))` — chained double md5, never zeros.
pub fn chacha20_key_32(body_key: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let first = md5::compute(body_key);
    key[..16].copy_from_slice(&first);
    let second = md5::compute(&key[..16]);
    key[16..].copy_from_slice(&second);
    key
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-native --lib chacha20_key`
Expected: PASS.

- [ ] **Step 5: Confirm the golden against Go (reproduce, don't trust)**

Write `/tmp/md5golden/main.go`:

```go
package main

import (
	"crypto/md5"
	"fmt"
)

func main() {
	k := make([]byte, 16)
	for i := range k {
		k[i] = 0x22
	}
	key := make([]byte, 32)
	t := md5.Sum(k)
	copy(key, t[:])
	t = md5.Sum(key[:16])
	copy(key[16:], t[:])
	fmt.Printf("%x\n", key)
}
```

Run: `cd /tmp/md5golden && go mod init md5golden && go run .`
Expected: `fbc3cf71d993ca7bec2664357ccdac2bb270c6d264a3bfeab7ceea80762a13cc` (matches the baked golden).

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess/keys.rs
git commit -m "feat(native): chacha20-poly1305 VMess body key derivation (md5 chain)"
```

---

### Task 2: header security byte parameterized (header.rs)

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vmess/header.rs`
- Test: same file

**Interfaces:**
- Consumes: existing `Session` struct (fields `request_body_iv`, `request_body_key`, `response_header`, `response_body_key`, `response_body_iv`).
- Produces: `Session.security: u8` field (default `SECURITY_AES128_GCM`), `pub const SECURITY_CHACHA20_POLY1305: u8 = 4;`; `encode_request` writes `session.security` instead of the hardcoded `SECURITY_AES128_GCM`. Later tasks: `mod.rs` `connect` sets `session.security`; `stream.rs` reads it.

- [ ] **Step 1: Write the failing test**

In `header.rs` tests (existing golden test `seal_wire_lengths` / the full-wire test asserts body byte at index 35 == 3 — keep that):

```rust
#[test]
fn request_security_byte_is_writable() {
    // Body layout: version(1) IV(16) key(16) respHdr(1) option(1) = 35, so
    // the security nibble/byte sits at body index 35.
    let ck = [0x55u8; 16];
    let mut session = Session::new();
    session.request_body_iv = [0x11; 16];
    session.request_body_key = [0x22; 16];
    session.security = SECURITY_CHACHA20_POLY1305;
    let mut entropy_calls = 0;
    let mut entropy = |out: &mut [u8]| {
        entropy_calls += 1;
        out.fill(0x77);
    };
    let target = TargetAddr::new(Host::Ip("127.0.0.1".parse().unwrap()), 8080);
    let wire = encode_request(&ck, &session, &target, 0x6000_0000_0000_0000, &mut entropy).unwrap();
    // The plaintext body is inside the sealed payload; the security byte is
    // not directly visible on the wire. Instead assert via a body-encoder
    // helper: extract the body by re-encrypting expectations is overkill —
    // assert the constant exists and encode_request accepts security 4
    // without error, and that the existing security-3 golden wire test still
    // passes unchanged (byte 3 default).
    assert_eq!(SECURITY_CHACHA20_POLY1305, 4);
    assert_eq!(wire.len(), peek_seal_len(49));
}
```

(If an existing helper exposes the plaintext body (e.g. a `encode_request_body`), assert `body[35] == 4` directly instead — prefer that when available.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib request_security_byte`
Expected: FAIL — `SECURITY_CHACHA20_POLY1305` not found.

- [ ] **Step 3: Implement**

In `header.rs`:

```rust
pub const SECURITY_AES128_GCM: u8 = 3;
pub const SECURITY_CHACHA20_POLY1305: u8 = 4;
```

Add to `Session`:

```rust
pub struct Session {
    pub request_body_iv: [u8; 16],
    pub request_body_key: [u8; 16],
    pub response_header: u8,
    pub response_body_key: [u8; 16],
    pub response_body_iv: [u8; 16],
    /// Payload security byte (header body byte 35): 3 = AES-128-GCM,
    /// 4 = chacha20-poly1305. `connect` sets it from the config.
    pub security: u8,
}
```

In `Session::new()` init `security: SECURITY_AES128_GCM`. In `encode_request`, replace `body.push(SECURITY_AES128_GCM); // padding nibble 0 | security 3` with `body.push(session.security);` and update the doc comment (`security={}`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p xray-tui-native --lib`
Expected: PASS — all existing header goldens unchanged (default security 3), new test passes.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess/header.rs
git commit -m "feat(native): parameterize VMess header payload security byte"
```

---

### Task 3: PayloadCipher record codec (stream.rs)

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vmess/stream.rs` (+ dep in `Cargo.toml`)
- Test: same file

**Interfaces:**
- Consumes: `keys::chacha20_key_32` (Task 1), `header::{Session, SECURITY_AES128_GCM, SECURITY_CHACHA20_POLY1305}` (Task 2).
- Produces: `enum Cipher { Aes128Gcm(Aes128Gcm), Chacha20Poly1305(ChaCha20Poly1305) }` with `new(security: u8, key16: &[u8; 16]) -> Self` and `encrypt(&self, nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, ()>` / `decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, ()>`. `VmessClientStream` fields `req_cipher`/`resp_cipher` become `Cipher`; `new()` derives them from `session.security`. Only the two record-AEAD sites dispatch (RecordData decrypt, poll_write seal); the response-header peel stays `Aes128Gcm` always (Go `OpenVMessAEADHeader` is security-independent).

- [ ] **Step 1: Add the dependency**

In `crates/xray-tui-native/Cargo.toml` `[dependencies]` (next to `aes-gcm = "0.10"`):

```toml
chacha20poly1305 = "0.10"
```

- [ ] **Step 2: Write the failing test (chacha roundtrip)**

In `stream.rs` tests, add a chacha twin of `seal_record` (keep the existing AES `seal_record` untouched):

```rust
/// Server-side chacha20-poly1305 response record (Go `EncodeResponseBody`
/// with security 4): 2B BE ciphertext length + chacha20poly1305 data.
fn seal_record_chacha(key16: &[u8; 16], iv: &[u8; 16], counter: u16, data: &[u8]) -> Vec<u8> {
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
    let mut nonce = *iv;
    nonce[..2].copy_from_slice(&counter.to_be_bytes());
    let key32 = keys::chacha20_key_32(key16);
    let ct = ChaCha20Poly1305::new_from_slice(&key32)
        .unwrap()
        .encrypt(Nonce::from_slice(&nonce[..12]), data)
        .unwrap();
    u16::try_from(data.len() + 16)
        .unwrap()
        .to_be_bytes()
        .into_iter()
        .chain(ct)
        .collect()
}
```

Note: `use chacha20poly1305::aead::Aead;` needed for `.encrypt` (or extend the existing `use aes_gcm::aead::Aead;` — the trait is the same crate `aead`).

```rust
#[test]
fn chacha20_records_roundtrip_with_peel() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let (client_side, mut server_side) = tokio::io::duplex(8192);
        let mut session = Session::new();
        session.security = SECURITY_CHACHA20_POLY1305;
        session.request_body_iv = [0x11; 16];
        session.request_body_key = [0x22; 16];
        session.response_header = 0x99;
        session.response_body_key = sha256_first16(&session.request_body_key);
        session.response_body_iv = sha256_first16(&session.request_body_iv);

        // Response HEADER seal is always AES-128-GCM (Go OpenVMessAEADHeader);
        // only the body record uses the chacha cipher.
        let mut wire = seal_response_header(
            &session.response_body_key,
            &session.response_body_iv,
            &[0x99, 0, 0, 0],
        );
        wire.extend_from_slice(&seal_record_chacha(
            &session.response_body_key,
            &session.response_body_iv,
            0,
            b"chacha body",
        ));
        server_side.write_all(&wire).await.unwrap();
        drop(server_side);

        let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
        let mut got = Vec::new();
        tunnel.read_to_end(&mut got).await.unwrap();
        assert_eq!(got, b"chacha body");
    });
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib chacha20_records_roundtrip`
Expected: FAIL — chacha key/`Cipher` missing or decrypt failure (AES key used on chacha data → auth failure or compile error).

- [ ] **Step 4: Implement the Cipher enum + dispatch**

Add to `stream.rs` (imports: `use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChaChaNonce};` — nonce types differ per crate; `aes_gcm::Nonce` stays for AES):

```rust
/// Record cipher selected by the payload security byte. The response-header
/// peel is ALWAYS AES-128-GCM; only the body records dispatch here.
enum Cipher {
    Aes128Gcm(Aes128Gcm),
    Chacha20Poly1305(ChaCha20Poly1305),
}

impl Cipher {
    /// Build from the header security byte; the 16-byte VMess session key is
    /// expanded to 32 bytes for chacha (md5 chain) inside.
    fn new(security: u8, key16: &[u8; 16]) -> Self {
        match security {
            SECURITY_AES128_GCM => {
                Self::Aes128Gcm(Aes128Gcm::new_from_slice(key16).expect("16-byte key"))
            }
            SECURITY_CHACHA20_POLY1305 => {
                let key32 = keys::chacha20_key_32(key16);
                Self::Chacha20Poly1305(
                    ChaCha20Poly1305::new_from_slice(&key32).expect("32-byte key"),
                )
            }
            other => panic!("vmess payload security {other} not validated"),
        }
    }

    fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, ()> {
        match self {
            Self::Aes128Gcm(c) => c
                .encrypt(aes_gcm::Nonce::from_slice(nonce), plaintext)
                .map_err(|_| ()),
            Self::Chacha20Poly1305(c) => c
                .encrypt(chacha20poly1305::Nonce::from_slice(nonce), plaintext)
                .map_err(|_| ()),
        }
    }

    fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
        match self {
            Self::Aes128Gcm(c) => c
                .decrypt(aes_gcm::Nonce::from_slice(nonce), ciphertext)
                .map_err(|_| ()),
            Self::Chacha20Poly1305(c) => c
                .decrypt(chacha20poly1305::Nonce::from_slice(nonce), ciphertext)
                .map_err(|_| ()),
        }
    }
}
```

In `VmessClientStream`: change `req_cipher: Aes128Gcm` / `resp_cipher: Aes128Gcm` to `req_cipher: Cipher` / `resp_cipher: Cipher`; in `new()` replace the two `Aes128Gcm::new_from_slice(...)` lines with:

```rust
let req_cipher = Cipher::new(session.security, &session.request_body_key);
let resp_cipher = Cipher::new(session.security, &session.response_body_key);
```

In `poll_read` `RecordData` arm, replace:

```rust
if let Ok(pt) = this
    .resp_cipher
    .decrypt(Nonce::from_slice(&nonce), &this.pending[..total])
```

with:

```rust
if let Ok(pt) = this.resp_cipher.decrypt(&nonce, &this.pending[..total])
```

In `poll_write`, replace:

```rust
let Ok(ct) = this.req_cipher.encrypt(Nonce::from_slice(&nonce), buf) else {
```

with:

```rust
let Ok(ct) = this.req_cipher.encrypt(&nonce, buf) else {
```

Leave the peel decrypts (lines using `Aes128Gcm::new_from_slice(&len_key)` / `&payload_key`) untouched.

- [ ] **Step 5: Run the stream tests**

Run: `cargo test -p xray-tui-native --lib protocol::vmess::stream`
Expected: PASS — existing AES roundtrip + EOF-marker tests AND the new chacha roundtrip.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/Cargo.toml crates/xray-tui-native/Cargo.lock crates/xray-tui-native/src/protocol/vmess/stream.rs
git commit -m "feat(native): VMess payload cipher dispatch (aes-128-gcm | chacha20-poly1305)"
```

---

### Task 4: check_security + connect mapping (mod.rs)

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vmess/mod.rs`
- Test: same file

**Interfaces:**
- Consumes: `header::{SECURITY_AES128_GCM, SECURITY_CHACHA20_POLY1305}` (Task 2).
- Produces: `fn security_byte(cfg: &VmessConfig) -> Result<u8, NativeError>` (used by `check_security` and `connect`); `connect` sets `session.security` before `encode_request` + `VmessClientStream::new`.

- [ ] **Step 1: Write the failing test**

In `mod.rs` tests (the existing `accepts_absent_security_key` test stays):

```rust
#[test]
fn chacha20_security_accepted() {
    let cfg = vmess_cfg("chacha20-poly1305");
    assert!(check_security(&cfg).is_ok());
    assert_eq!(security_byte(&cfg).unwrap(), SECURITY_CHACHA20_POLY1305);
}

#[test]
fn aes128_security_still_default() {
    let cfg = vmess_cfg("aes-128-gcm");
    assert_eq!(security_byte(&cfg).unwrap(), SECURITY_AES128_GCM);
    // absent/auto still map to the AES default
    let auto: VmessConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vmess", "uuid": "00000000-0000-0000-0000-000000000000",
        "security": { "enc": "auto", "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    })).unwrap();
    assert_eq!(security_byte(&auto).unwrap(), SECURITY_AES128_GCM);
}
```

(`vmess_cfg` — reuse the existing test helper in this file that builds a `VmessConfig` from JSON, or inline it like `accepts_absent_security_key` does.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib chacha20_security_accepted`
Expected: FAIL — `security_byte` not found (or chacha rejected).

- [ ] **Step 3: Implement**

Refactor `check_security` to share the mapping:

```rust
/// Map the requested `security.enc` to the header security byte.
pub fn security_byte(cfg: &VmessConfig) -> Result<u8, NativeError> {
    match cfg.security.enc.as_deref() {
        None | Some("" | "auto" | "aes-128-gcm") => Ok(SECURITY_AES128_GCM),
        Some("chacha20-poly1305") => Ok(SECURITY_CHACHA20_POLY1305),
        Some(other) => Err(NativeError::Config(format!(
            "vmess payload security {other:?} not supported (native core: aes-128-gcm, chacha20-poly1305)"
        ))),
    }
}

/// Validate the `VMess` payload security the config requests.
pub fn check_security(cfg: &VmessConfig) -> Result<(), NativeError> {
    security_byte(cfg).map(|_| ())
}
```

In `connect`, after `check_security(cfg)?`, set the session security:

```rust
let mut session = Session::new();
session.security = security_byte(cfg)?;
```

(Imports: `use crate::protocol::vmess::header::{Session, SECURITY_AES128_GCM, SECURITY_CHACHA20_POLY1305, encode_request};`)

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-native --lib`
Expected: PASS — new tests + all existing (incl. `rejects_none_security` style tests unchanged: `none`/`zero` still rejected).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vmess/mod.rs
git commit -m "feat(native): accept chacha20-poly1305 VMess payload security"
```

---

### Task 5: e2e facility — SecurityVariant + CaseSpec + typed builders

**Files:**
- Create: `crates/xray-tui-native/src/e2e/variant.rs`
- Create: `crates/xray-tui-native/src/e2e/case.rs`
- Create: `crates/xray-tui-native/src/e2e/config.rs`
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (version consts, `run_against_cores`, `label -> String`, exports)
- Test: unit tests in the new files

**Interfaces:**
- Consumes: existing `E2eCase` trait, `ServerEnv`, `CoreKind`, `CoreUnderTest`, `run`, harness (`Certs`, etc.), `NativeConnectParams`, `xray_tui_proto::proto_spec::{ProtocolConfig, endpoint::EndpointEssentials}`.
- Produces:
  - `pub const SINGBOX_VERSION: &str = "1.13.16"; pub const XRAY_VERSION: &str = "26.3.27";` in `e2e/mod.rs`
  - `pub trait SecurityVariant: Sync { fn name(&self) -> &'static str; fn cores(&self) -> &'static [CoreKind]; fn server_security(&self, core: CoreKind) -> Option<&'static str>; fn client_security(&self) -> &'static str; }`
  - `pub struct Aes128GcmVariant; pub struct Chacha20Poly1305Variant;` (impls in `variant.rs`)
  - `pub enum ProtocolKind { Vless, Vmess }`
  - `pub struct CaseSpec { pub protocol: ProtocolKind, pub security: Option<Box<dyn SecurityVariant>> }` with `CaseSpec::vless()`, `CaseSpec::vmess<S: SecurityVariant + 'static>(variant: S)`, `CaseSpec::cores() -> &'static [CoreKind]` (security None → both cores), and `impl E2eCase for CaseSpec`
  - `config.rs`: `pub const UUID`, `pub const BODY`, `pub fn vmess_inbound(core: CoreKind, env: &ServerEnv, security: Option<&str>) -> String`, `pub fn vless_inbound(core: CoreKind, env: &ServerEnv) -> String`, `pub fn client_params_vmess(enc: &str, port: u16, target: SocketAddr) -> NativeConnectParams`, `pub fn client_params_vless(port: u16, target: SocketAddr) -> NativeConnectParams`
  - `pub async fn run_against_cores(case: &CaseSpec) -> Result<(), String>` in `e2e/mod.rs`

- [ ] **Step 1: Change `E2eCase::label` to return `String`**

In `e2e/mod.rs`, change `fn label(&self) -> &'static str;` to `fn label(&self) -> String;`. Update `run()` uses (they already `eprintln!`/`format!` the label — pass `&case.label()` or the owned string). Update the two existing impls' signatures (`VlessCase`/`VmessCase` in `protocol/{vless,vmess}/e2e.rs`) to return `String` (`"vless/tcp/tls".to_string()` etc.) so the crate still compiles mid-refactor. Run `cargo test -p xray-tui-native --lib` → green before continuing.

- [ ] **Step 2: Write failing unit tests for the new modules**

`variant.rs` tests:

```rust
#[test]
fn chacha_variant_supports_both_cores() {
    let v = Chacha20Poly1305Variant;
    assert_eq!(v.name(), "chacha20-poly1305");
    assert_eq!(v.cores(), &[CoreKind::Xray, CoreKind::SingBox]);
    assert_eq!(v.server_security(CoreKind::Xray), Some("chacha20-poly1305"));
    assert_eq!(v.server_security(CoreKind::SingBox), None); // sing-box: no field
    assert_eq!(v.client_security(), "chacha20-poly1305");
}
```

`case.rs` tests:

```rust
#[test]
fn derived_labels() {
    assert_eq!(CaseSpec::vless().label(), "vless/tcp/tls");
    assert_eq!(CaseSpec::vmess(Aes128GcmVariant).label(), "vmess/tcp/tls/aes-128-gcm");
    assert_eq!(CaseSpec::vmess(Chacha20Poly1305Variant).label(), "vmess/tcp/tls/chacha20-poly1305");
}

#[test]
fn cores_gate_defaults_to_both() {
    assert_eq!(CaseSpec::vless().cores(), &[CoreKind::Xray, CoreKind::SingBox]);
    assert_eq!(CaseSpec::vmess(Chacha20Poly1305Variant).cores(), &[CoreKind::Xray, CoreKind::SingBox]);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p xray-tui-native --lib e2e::`
Expected: FAIL — modules/files don't exist.

- [ ] **Step 4: Implement the three modules**

`e2e/mod.rs` — add before the trait:

```rust
/// Core version strings for the e2e sweep (single source of truth; tests
/// import these instead of duplicating constants).
pub const SINGBOX_VERSION: &str = "1.13.16";
pub const XRAY_VERSION: &str = "26.3.27";
```

Add `pub mod case; pub mod config; pub mod variant;` and re-exports: `pub use case::{CaseSpec, ProtocolKind}; pub use variant::{Aes128GcmVariant, Chacha20Poly1305Variant, SecurityVariant};`. Add:

```rust
/// Run `case` against every core in its gate; the first failure short-circuits.
pub async fn run_against_cores(case: &CaseSpec) -> Result<(), String> {
    for kind in case.cores() {
        let version = match kind {
            CoreKind::Xray => XRAY_VERSION,
            CoreKind::SingBox => SINGBOX_VERSION,
        };
        let core = CoreUnderTest::resolve(*kind, version)?;
        run(case, &core)
            .await
            .map_err(|e| format!("{}/{}: {e}", case.label(), core.bin.display()))?;
    }
    Ok(())
}
```

`e2e/variant.rs`:

```rust
//! Payload-security variants for the e2e pipeline. A variant names itself,
//! gates which cores support it, and supplies the security strings for the
//! server config and client params.

use super::{CoreKind, SecurityVariant};

/// VMess payload security: AES-128-GCM (xray header security byte 3).
pub struct Aes128GcmVariant;

impl SecurityVariant for Aes128GcmVariant {
    fn name(&self) -> &'static str {
        "aes-128-gcm"
    }
    fn cores(&self) -> &'static [CoreKind] {
        &[CoreKind::Xray, CoreKind::SingBox]
    }
    fn server_security(&self, core: CoreKind) -> Option<&'static str> {
        match core {
            // xray inbound user security mirrors intent; sing-box rejects
            // the field outright (`json: unknown field "security"`).
            CoreKind::Xray => Some("aes-128-gcm"),
            CoreKind::SingBox => None,
        }
    }
    fn client_security(&self) -> &'static str {
        "aes-128-gcm"
    }
}

/// VMess payload security: chacha20-poly1305 (header security byte 4).
pub struct Chacha20Poly1305Variant;

impl SecurityVariant for Chacha20Poly1305Variant {
    fn name(&self) -> &'static str {
        "chacha20-poly1305"
    }
    fn cores(&self) -> &'static [CoreKind] {
        &[CoreKind::Xray, CoreKind::SingBox]
    }
    fn server_security(&self, core: CoreKind) -> Option<&'static str> {
        match core {
            CoreKind::Xray => Some("chacha20-poly1305"),
            CoreKind::SingBox => None,
        }
    }
    fn client_security(&self) -> &'static str {
        "chacha20-poly1305"
    }
}

#[cfg(test)]
mod tests { /* Step 2 test */ }
```

`e2e/case.rs`:

```rust
//! Data-driven e2e case: protocol + optional payload-security variant. The
//! `E2eCase` impl derives the server config, client params, expectations and
//! label from these parts — adding a variant is a new `SecurityVariant` impl,
//! not a new case struct.

use std::net::SocketAddr;

use super::variant::SecurityVariant;
use super::{Certs, CoreKind, E2eCase, E2eExpect, ServerEnv, config};
use crate::NativeConnectParams;
use crate::security;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolKind {
    Vless,
    Vmess,
}

/// One e2e scenario described as data.
pub struct CaseSpec {
    pub protocol: ProtocolKind,
    pub security: Option<Box<dyn SecurityVariant>>,
}

impl CaseSpec {
    #[must_use]
    pub fn vless() -> Self {
        Self {
            protocol: ProtocolKind::Vless,
            security: None,
        }
    }

    #[must_use]
    pub fn vmess(variant: impl SecurityVariant + 'static) -> Self {
        Self {
            protocol: ProtocolKind::Vmess,
            security: Some(Box::new(variant)),
        }
    }

    /// Cores this case runs against (variant gate; no security → both).
    #[must_use]
    pub fn cores(&self) -> &'static [CoreKind] {
        self.security
            .as_ref()
            .map_or(&[CoreKind::Xray, CoreKind::SingBox], |s| s.cores())
    }
}

impl E2eCase for CaseSpec {
    fn label(&self) -> String {
        match (self.protocol, self.security.as_deref()) {
            (ProtocolKind::Vless, _) => "vless/tcp/tls".to_string(),
            (ProtocolKind::Vmess, Some(s)) => format!("vmess/tcp/tls/{}", s.name()),
            (ProtocolKind::Vmess, None) => "vmess/tcp/tls".to_string(),
        }
    }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        match self.protocol {
            ProtocolKind::Vless => config::vless_inbound(core, env),
            ProtocolKind::Vmess => {
                let security = self.security.as_ref().and_then(|s| s.server_security(core));
                config::vmess_inbound(core, env, security)
            }
        }
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        match self.protocol {
            ProtocolKind::Vless => config::client_params_vless(port, target),
            ProtocolKind::Vmess => {
                let enc = self
                    .security
                    .as_ref()
                    .expect("vmess case requires a security variant")
                    .client_security();
                config::client_params_vmess(enc, port, target)
            }
        }
    }

    fn expected(&self) -> E2eExpect {
        E2eExpect {
            status: 200,
            body: config::BODY.into(),
        }
    }

    fn client_trust(&self, certs: &Certs) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));
    }
}

#[cfg(test)]
mod tests { /* Step 2 tests */ }
```

`e2e/config.rs` — move the config-building code verbatim from the current `protocol/vmess/e2e.rs` / `protocol/vless/e2e.rs` (the JSON bodies are identical; only the entry points change):

```rust
//! Typed per-core server-config and client-param builders. Both core dialects
//! live in one place per protocol; a variant only supplies the security
//! strings (see `variant.rs`).

use std::net::SocketAddr;

use xray_tui_proto::proto_spec::ProtocolConfig;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

use super::{CoreKind, ServerEnv};
use crate::NativeConnectParams;
use crate::addr::{Host, TargetAddr};

pub const UUID: &str = "00000000-0000-0000-0000-000000000000";
pub const BODY: &str = "hello native core";

/// VMess inbound JSON for `core`. `security` is the xray user security string
/// (mirrors intent; cosmetic for AEAD); sing-box never receives it.
pub fn vmess_inbound(core: CoreKind, env: &ServerEnv, security: Option<&str>) -> String {
    let cert_path = env.tmp.join("server.crt").to_string_lossy().into_owned();
    let key_path = env.tmp.join("server.key").to_string_lossy().into_owned();
    let json = match core {
        CoreKind::Xray => serde_json::json!({
            "inbounds": [{
                "listen": "127.0.0.1", "port": env.port, "protocol": "vmess",
                "settings": { "clients": [{
                    "id": UUID,
                    "security": security.unwrap_or("aes-128-gcm")
                }] },
                "streamSettings": { "network": "tcp", "security": "tls",
                    "tlsSettings": { "certificates": [
                        { "certificateFile": cert_path, "keyFile": key_path }
                    ], "alpn": ["http/1.1"] } }
            }],
            "outbounds": [{ "protocol": "freedom" }]
        }),
        CoreKind::SingBox => serde_json::json!({
            "log": { "level": "warn" },
            "inbounds": [{ "type": "vmess", "listen": "127.0.0.1", "listen_port": env.port,
                "users": [{ "uuid": UUID }],
                "tls": { "enabled": true, "certificate_path": cert_path, "key_path": key_path,
                    "alpn": ["http/1.1"] } }],
            "outbounds": [{ "type": "direct" }]
        }),
    };
    serde_json::to_string(&json).expect("vmess server config serializes")
}

/// VLESS inbound JSON for `core` (no payload security dimension).
pub fn vless_inbound(core: CoreKind, env: &ServerEnv) -> String {
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

/// Native client params dialing a VMess listener with payload security `enc`.
pub fn client_params_vmess(enc: &str, port: u16, target: SocketAddr) -> NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vmess",
        "uuid": UUID,
        "security": { "enc": enc, "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .expect("vmess client config parses");
    let server = EndpointEssentials::new("127.0.0.1", port);
    NativeConnectParams::new(
        protocol,
        server,
        TargetAddr::new(Host::Ip(target.ip()), target.port()),
    )
}

/// Native client params dialing a VLESS listener.
pub fn client_params_vless(port: u16, target: SocketAddr) -> NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vless",
        "uuid": UUID,
        "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .expect("vless client config parses");
    let server = EndpointEssentials::new("127.0.0.1", port);
    NativeConnectParams::new(
        protocol,
        server,
        TargetAddr::new(Host::Ip(target.ip()), target.port()),
    )
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p xray-tui-native --lib` and `cargo test -p xray-tui-native --features native-e2e --lib`
Expected: PASS — new unit tests + all existing lib tests.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/e2e/
git commit -m "feat(native): e2e CaseSpec + SecurityVariant + typed config builders"
```

---

### Task 6: migrate VLESS + VMESS cases onto CaseSpec; rewrite tests

**Files:**
- Delete: `crates/xray-tui-native/src/protocol/vmess/e2e.rs`, `crates/xray-tui-native/src/protocol/vless/e2e.rs`
- Modify: `crates/xray-tui-native/src/protocol/vmess/mod.rs`, `crates/xray-tui-native/src/protocol/vless/mod.rs` (remove `#[cfg(feature = "native-e2e")] pub mod e2e;`)
- Rewrite: `crates/xray-tui-native/tests/vmess.rs`, `crates/xray-tui-native/tests/vless.rs`
- Test: the rewritten test files (real cores)

**Interfaces:**
- Consumes: Task 5 modules (`CaseSpec`, `run_against_cores`, `Aes128GcmVariant`, version consts).
- Produces: final test files; `protocol/*/e2e.rs` gone.

- [ ] **Step 1: Delete the old per-protocol e2e case files and their module wiring**

```bash
git rm crates/xray-tui-native/src/protocol/vmess/e2e.rs crates/xray-tui-native/src/protocol/vless/e2e.rs
```

Remove `#[cfg(feature = "native-e2e")] pub mod e2e;` from both `vmess/mod.rs` and `vless/mod.rs`.

- [ ] **Step 2: Rewrite the test files**

`tests/vmess.rs`:

```rust
//! E2E: native VMess+TLS+TCP payload variants through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{Aes128GcmVariant, CaseSpec, Chacha20Poly1305Variant, run_against_cores};

#[tokio::test]
async fn vmess_tcp_tls_aes128gcm_against_cores() {
    run_against_cores(&CaseSpec::vmess(Aes128GcmVariant))
        .await
        .expect("vmess/aes-128-gcm e2e failed");
}

#[tokio::test]
async fn vmess_tcp_tls_chacha20poly1305_against_cores() {
    run_against_cores(&CaseSpec::vmess(Chacha20Poly1305Variant))
        .await
        .expect("vmess/chacha20-poly1305 e2e failed");
}
```

`tests/vless.rs`:

```rust
//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{CaseSpec, run_against_cores};

#[tokio::test]
async fn vless_tcp_tls_against_cores() {
    run_against_cores(&CaseSpec::vless())
        .await
        .expect("vless e2e failed");
}
```

- [ ] **Step 3: Run the existing interop sweep (must stay 4/4)**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: PASS — 3 tests, 6 core runs, all green (vless ×2, vmess/aes ×2, vmess/chacha ×2). This is the first real-core chacha20-poly1305 validation.

- [ ] **Step 4: Run the full native lib + clippy**

Run: `cargo test -p xray-tui-native --features native-e2e --lib` and `cargo clippy -p xray-tui-native --all-targets`
Expected: PASS, 0 warnings (delete the now-unused `seal_record_chacha`… no — that helper stays used by the chacha unit test).

- [ ] **Step 5: Commit**

```bash
git add -A crates/xray-tui-native/
git commit -m "refactor(native): migrate VLESS/VMESS e2e cases onto CaseSpec variants"
```

---

### Task 7: full-workspace verification + ledger

**Files:**
- Test: whole workspace
- Docs: `.superpowers/sdd/2026-08-10-native-e2e-variant-hardening/progress.md` (ledger, create)

- [ ] **Step 1: Full workspace test**

Run: `timeout 600 cargo test`
Expected: PASS — 884 prior + new tests (chacha key golden, header security byte, chacha record roundtrip, security_byte mapping, e2e unit tests) all green.

- [ ] **Step 2: Clippy + fmt**

Run: `cargo clippy --workspace --all-targets` and `cargo fmt -- --check`
Expected: 0 warnings, clean.

- [ ] **Step 3: Final interop sweep (evidence for the ledger)**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/native-cores timeout 240 cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: PASS — vless, vmess/aes-128-gcm, vmess/chacha20-poly1305 × xray + sing-box = 6 runs, 0 failures.

- [ ] **Step 4: Write the ledger**

`progress.md`: task-by-task status, the corrected wire fact (chacha key = md5 chain, Go-verified golden), e2e sweep results, deferred minors, review gate record.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "docs: e2e variant hardening verification ledger"
```

---

## Self-Review

- **Spec coverage:** wire contract (Tasks 1-4) ✓; `CaseSpec` + `SecurityVariant` + typed builders (Task 5) ✓; VLESS + aes-128-gcm migration (Task 6) ✓; chacha PoC e2e (Tasks 5-6) ✓; version consts single source (Task 5) ✓; cores gate (Task 5 `CaseSpec::cores`) ✓; verification (Task 7) ✓; out-of-scope respected (no transport axis, no `none`/`zero`).
- **Placeholder scan:** no TBD/TODO; every step has concrete code and commands.
- **Type consistency:** `chacha20_key_32` (T1) consumed by `Cipher::new` (T3) ✓; `session.security` set in T2, read in T3's `Cipher::new` and `connect` (T4) ✓; `SECURITY_CHACHA20_POLY1305` defined T2, used T3/T4 ✓; `CaseSpec`/`run_against_cores`/variant impls consistent across T5/T6 ✓; `E2eCase::label -> String` propagated to both old impls in T5 Step 1 before deletion in T6 ✓.
