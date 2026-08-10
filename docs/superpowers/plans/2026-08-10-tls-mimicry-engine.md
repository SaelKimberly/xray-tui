# TLS Fingerprint Mimicry Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `xray-tui-tls` — a ring-based TLS 1.3 client library with browser-fingerprint ClientHello mimicry and a REALITY client — and wire it into `xray-tui-native` as the custom `TlsConnector`, proven by tier-1 offline JA3/JA4 tests, a tier-2 tls.peet.ws grader, and tier-3 e2e against real xray-core / sing-box.

**Architecture:** New workspace crate `crates/xray-tui-tls`. Declarative `ClientHelloSpec` data + `define_profiles!` macro (uTLS-style spec, wreq-util-style dispatch). One TLS 1.3 handshake engine with two personalities (plain TLS with webpki verification; REALITY with auth-key session seal). Generic over `AsyncRead + AsyncWrite`; returns a record-framed `TlsStream<S>`. `xray-tui-native` consumes it via the existing `TlsProvider::Custom(Arc<dyn TlsConnector>)` seam.

**Tech Stack:** ring 0.17 (crypto), tokio (io-util), rustls-webpki 0.103 + webpki-roots (cert verify), rustls-pki-types, md-5 (JA3), thiserror. Adopted sources: `thirdparty/tls-fingerprint/` (hello/record/handshake/http2), `thirdparty/shoes/src/reality/` (REALITY client), reference `thirdparty/utls/` (spec model, `u_parrots.go` Chrome-133 data), `thirdparty/wreq-util/` (macro dispatch).

## Global Constraints

- **ring only** in `xray-tui-tls`: no aws-lc-rs, no `rand` crate, no `unsafe`. RNG via `ring::rand::SecureRandom` trait (`SystemRandom` prod; fixed-seed impl in tests).
- Workspace conventions: edition 2024, `[lints] workspace = true` (clippy pedantic+nursery warn), rustfmt, thiserror for errors.
- **No implicit network in usual tests.** Tier-2 grader = example binary + `#[ignore]`d tests. Tier-3 e2e = feature `native-e2e` AND env `XRAY_TUI_CORE_BIN_DIR` (existing harness gate).
- Every network step in the native crate stays wrapped in `timeouts::SECURITY` (10 s).
- Port profile extension lists from `tls-fingerprint/src/profiles/*.rs` **verbatim** (captured browser data). Wire contract for REALITY = the 9-step contract documented in `crates/xray-tui-native/src/security/reality.rs` module doc.
- `xray-tui-proto` is NOT modified. `xray-tui-native` must compile after every task (`cargo check -p xray-tui-native`).
- Baseline fingerprints (captured 2026-08-10 against tls.peet.ws): Chrome/130 JA3 `2b916ec56aedf4a5ecbeb5804f60c242` (GREASE-randomized per run), JA4 `t13d1516h2_8daaf6152771_f37e75b10bcc`; Firefox/128 ESR JA3 `361e0ca6ef1ca4dbe3a1d987722a1980`, JA4 `t13d1314h2_07be0c029dc8_46701d79520f`.

---

### Task 1: Scaffold `xray-tui-tls` crate

**Files:**
- Create: `crates/xray-tui-tls/Cargo.toml`
- Create: `crates/xray-tui-tls/src/lib.rs`
- Create: `crates/xray-tui-tls/src/error.rs`
- Create: `crates/xray-tui-tls/src/spec/mod.rs`, `src/hello/mod.rs`, `src/crypto/mod.rs`, `src/record/mod.rs`, `src/handshake/mod.rs`, `src/verify/mod.rs`, `src/reality/mod.rs`, `src/profiles/mod.rs`, `src/http2/mod.rs` (empty module stubs with doc comments)
- Modify: root `Cargo.toml` (add `"crates/xray-tui-tls"` to `members`; add `xray-tui-tls = { path = "crates/xray-tui-tls" }` to `[workspace.dependencies]`)

**Interfaces:**
- Produces: crate skeleton compiling; `TlsError`/`Result` public; module tree matches the design spec.

- [ ] **Step 1: Write the manifest**

```toml
[package]
name = "xray-tui-tls"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "ring-based TLS 1.3 client with browser fingerprint mimicry + REALITY"

[dependencies]
# crypto (workspace standard: ring)
ring = "0.17"
# async I/O seam
tokio = { workspace = true, features = ["io-util"] }
# cert verification
rustls-webpki = { version = "0.103", default-features = false, features = ["ring", "std"] }
rustls-pki-types = "1"
webpki-roots = "1"
# JA3 hash
md-5 = "0.10"
# errors
thiserror.workspace = true

[lints]
workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread", "net", "time"] }
rcgen = "0.13"

[[example]]
name = "grader"
path = "examples/grader.rs"
```

Add the member + workspace dep in the root `Cargo.toml` (alphabetical placement next to `xray-tui-proto`).

- [ ] **Step 2: Write `error.rs`**

```rust
//! Error types for the TLS engine.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("spec error: {0}")]
    Spec(String),
    #[error("handshake error: {0}")]
    Handshake(String),
    #[error("certificate verification failed: {0}")]
    Verify(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("server sent HelloRetryRequest; retry is not supported")]
    HelloRetryRequest,
}

pub type Result<T> = std::result::Result<T, TlsError>;
```

- [ ] **Step 3: Write `lib.rs`** — module tree with doc comments mirroring the design spec (spec/profiles/hello/crypto/record/handshake/verify/reality/http2/error), plus:

```rust
/// Byte-stream seam: everything the engine touches is generic over this.
pub trait Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> Stream for T {}
```

- [ ] **Step 4: Stub modules** — each `mod.rs` gets a module doc comment only (no code). Root lib compiles.

- [ ] **Step 5: Run tests**

Run: `cargo check -p xray-tui-tls && cargo check -p xray-tui-native`
Expected: both PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-tls Cargo.toml Cargo.lock
git commit -m "feat(tls): scaffold xray-tui-tls crate"
```

---

### Task 2: Spec model + extension wire encoding

**Files:**
- Create: `crates/xray-tui-tls/src/spec/mod.rs`
- Create: `crates/xray-tui-tls/src/spec/grease.rs`

**Interfaces:**
- Consumes: `TlsError` (Task 1).
- Produces (later tasks consume):

```rust
pub fn is_grease(v: u16) -> bool;                 // v >> 8 == v & 0xFF
pub fn random_grease(rng: &dyn ring::rand::SecureRandom) -> Result<u16, TlsError>; // uniform over the 16 valid values
pub const GREASE_PLACEHOLDER: u16;                // 0xCACA — what specs write for a grease slot

pub enum SessionIdSpec { Random32, AuthPayload { len: usize } } // len = full wire length (plaintext + 16 tag)
pub enum ExtensionSpec {
    ServerName, SupportedGroups(Vec<u16>), KeyShare(Vec<KeyShareGroup>),
    SupportedVersions(Vec<u16>), SignatureAlgorithms(Vec<u16>), Alpn(Vec<String>),
    EcPointFormats, SessionTicket, PskKeyExchangeModes, StatusRequest,
    SignedCertificateTimestamp, RenegotiationInfo, CompressCertificate(Vec<u16>),
    ApplicationSettings(Vec<String>), RecordSizeLimit(u16), Padding, Grease,
    Raw { ty: u16, data: Vec<u8> },
}
pub enum KeyShareGroup { Grease, X25519 }
pub struct ClientHelloSpec {
    pub legacy_version: u16,          // 0x0303
    pub cipher_suites: Vec<u16>,      // GREASE_PLACEHOLDER allowed
    pub compression_methods: Vec<u8>, // [0]
    pub session_id: SessionIdSpec,
    pub extensions: Vec<ExtensionSpec>,
}
```

- [ ] **Step 1: Write the failing test** — extension wire encoding. Each `ExtensionSpec` encodes to the exact extension bytes (type u16 + length u16 + body), given runtime fill values passed in:

```rust
// spec/mod.rs tests
#[test]
fn grease_detection() {
    assert!(is_grease(0x0A0A) && is_grease(0xCACA) && is_grease(0xFAFA));
    assert!(!is_grease(0x1301) && !is_grease(0x1516) && !is_grease(0x0000));
}

#[test]
fn server_name_encodes_host() {
    let ext = ExtensionSpec::ServerName;
    let body = ext.encode_body(&RuntimeValues { server_name: "example.com", ..Default::default() }).unwrap();
    // body = 00 00 00 0b 00 09 00 00 00 09 65 78 61 6d 70 6c 65 2e 63 6f 6d
    assert_eq!(body, vec![0x00,0x00,0x00,0x0b,0x00,0x09,0x00,0x00,0x00,0x09,
                          b'e',b'x',b'a',b'm',b'p',b'l',b'e',b'.',b'c',b'o',b'm']);
}
```

(The `RuntimeValues` struct is defined in Task 3 — for Task 2 keep the encode signature as `encode_body(&self, rt: &RuntimeValues) -> Result<Vec<u8>, TlsError>` and declare the struct in `spec/mod.rs` with fields `server_name: String, alpn: Vec<String>, x25519_pub: [u8; 32], grease_a: u16, grease_b: u16`; Task 3 fills it.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib spec`
Expected: FAIL — `ExtensionSpec` / `encode_body` / `RuntimeValues` don't exist yet.

- [ ] **Step 3: Implement** — `grease.rs` (detection + `random_grease` picking uniformly from `[0x0A0A, 0x1A1A, ..., 0xFAFA]` via `rng.fill`), `ExtensionSpec::encode_body` per arm:
  - `ServerName`: `00 00 <len16> <type 00> <host>` (RFC 8446 §4.2.1 — empty extension type 0x00).
  - `Alpn`: `00 10 <len16> <list>` where list = `00 02 h2 00 08 http/1.1` style per entry.
  - `SupportedGroups`: `00 0a <len16> <count16> <groups…>`.
  - `KeyShare`: `00 33 <len16> <list>` — `Grease` entry = `grease_a, 00 01, 00`; `X25519` entry = `00 1d, 00 20, <x25519_pub>`.
  - `SupportedVersions`: `00 2b <len16> <count8> <versions…>` (client shape).
  - `SignatureAlgorithms`: `00 0d <len16> <count16> <schemes…>`.
  - `EcPointFormats`: `00 0b <len16> 01 00`.
  - `SessionTicket` `00 23 00 00`; `PskKeyExchangeModes` `00 2d <len16> 01 01`; `StatusRequest` `00 05 <len16> 01 00 00 00 00`; `SignedCertificateTimestamp` `00 12 00 00`; `RenegotiationInfo` `ff 01 <len16> 00`.
  - `CompressCertificate`: `00 1b <len16> <count16> <algos…>`.
  - `ApplicationSettings`: `00 44 69 <len16> <list>` (same list shape as ALPN).
  - `RecordSizeLimit`: `00 1c <len16> <limit16>`.
  - `Padding`: `00 15 <len16> <zeroes…>` — length from `rt.padding_len` field on `RuntimeValues` (add `padding_len: usize`).
  - `Grease`: `grease_b, <len16=1>, 00`.
  - `Raw`: `ty, <len16>, data`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib spec`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): ClientHello spec model + extension wire encoding"
```

---

### Task 3: ClientHello builder (GREASE pairing, session id, padding)

**Files:**
- Create: `crates/xray-tui-tls/src/hello/mod.rs`
- Modify: `crates/xray-tui-tls/src/spec/mod.rs` (fill `RuntimeValues` if not already complete)

**Interfaces:**
- Consumes: `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec`/grease fns (Task 2).
- Produces:

```rust
pub struct BuildParams<'a> {
    pub server_name: &'a str,
    pub alpn: Option<&'a [&'a str]>,   // None → use spec's Alpn
    pub x25519_pub: &'a [u8; 32],
    pub rng: &'a dyn ring::rand::SecureRandom,
}
pub struct BuiltHello {
    pub handshake_bytes: Vec<u8>,        // type 0x01 + 3-byte len + body
    pub record_bytes: Vec<u8>,           // handshake record (0x16 0x0301 len)
    pub session_id_range: Option<std::ops::Range<usize>>, // Some for AuthPayload
}
pub fn build_hello(spec: &ClientHelloSpec, params: &BuildParams) -> Result<BuiltHello, TlsError>;
pub fn to_record(handshake_bytes: &[u8]) -> Vec<u8>;
```

GREASE pairing rule (Chrome family): the FIRST `GREASE_PLACEHOLDER` occurrence in each of {cipher_suites, supported_groups, supported_versions, key_share groups} is replaced with the SAME value `grease_a`; a standalone `ExtensionSpec::Grease` uses `grease_b`. Any additional placeholder gets a fresh value. Firefox-family specs contain no placeholders.

- [ ] **Step 1: Write the failing test** — golden bytes captured from the reference implementation (`/tmp/tf`, the `tls-fingerprint` crate, same profile + fixed seed):

```rust
// hello/mod.rs tests
struct FixedRandom { bytes: Vec<u8>, pos: usize }
impl ring::rand::SecureRandom for FixedRandom {
    fn fill(&mut self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified> {
        for b in dest.iter_mut() {
            *b = *self.bytes.get(self.pos).ok_or(ring::error::Unspecified)?;
            self.pos += 1;
        }
        Ok(())
    }
}

#[test]
fn chrome130_golden_hello_with_fixed_seed() {
    // Reference capture (run the CURRENT tls-fingerprint example with the
    // same fixed seed and record the ClientHello bytes; lock them here):
    //   spec = profiles::chrome::v130::spec() (Task 10 adds the real data —
    //   until then use a placeholder spec assembled from the chrome.rs
    //   constants in thirdparty/tls-fingerprint/src/profiles/chrome.rs)
    let spec = test_spec(); // CIPHER_SUITES etc. copied from tls-fingerprint chrome.rs
    let rng = FixedRandom { bytes: vec![0x42; 64], pos: 0 };
    let hello = build_hello(&spec, &BuildParams {
        server_name: "tls.peet.ws",
        alpn: Some(&["h2", "http/1.1"]),
        x25519_pub: &[0xAB; 32],
        rng: &rng,
    }).unwrap();
    assert_eq!(hello.handshake_bytes, EXPECTED_HELLO); // captured from reference
}
```

Capture procedure for `EXPECTED_HELLO`: in `/tmp/tf` (the standalone tls-fingerprint copy), add a temporary example that builds the Chrome130 hello with a fixed-seed RNG and prints `hex::encode(handshake_bytes)`; copy the output into this test. The ported builder must produce byte-identical output.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib hello`
Expected: FAIL — no `build_hello`.

- [ ] **Step 3: Implement** — `hello/mod.rs`:
  - Draw `grease_a` + `grease_b` from `params.rng` (skip if spec has no placeholders).
  - Assemble body: `legacy_version` (2) + random (32, from rng) + session_id (per `SessionIdSpec`: `Random32` → 32 random bytes; `AuthPayload { len }` → `len` zero bytes, record the range) + cipher_suites (placeholders → grease values, count u16) + compression_methods + extensions (each `encode_body` with the filled `RuntimeValues`; compute `Padding` length from the accumulated size — Chrome pads to ~517 total; keep padding zero for the golden test).
  - Wrap: `to_record` = `0x16 0x03 0x01 <len16> <handshake>`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib hello`
Expected: PASS (byte-identical to reference).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): ClientHello builder with GREASE pairing"
```

---

### Task 4: ClientHello parser + JA3/JA4 encoders

**Files:**
- Create: `crates/xray-tui-tls/src/hello/parse.rs`
- Create: `crates/xray-tui-tls/src/crypto/fingerprint/mod.rs`, `ja3.rs`, `ja4.rs`

**Interfaces:**
- Consumes: `build_hello` (Task 3).
- Produces:

```rust
pub struct ParsedClientHello {
    pub legacy_version: u16,
    pub random: [u8; 32],
    pub session_id: Vec<u8>,
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<(u16, Vec<u8>)>,
}
pub fn parse_hello(handshake_bytes: &[u8]) -> Result<ParsedClientHello, TlsError>;

pub struct Ja3Fields {
    pub version: u16, pub ciphers: Vec<u16>, pub extensions: Vec<u16>,
    pub curves: Vec<u16>, pub point_formats: Vec<u8>,
    pub signature_algorithms: Vec<u16>, pub alpn: Vec<String>,
}
impl From<&ParsedClientHello> for Ja3Fields;   // curves/pt_fmts/sigalgs/alpn extracted from extensions
pub fn ja3_string(f: &Ja3Fields) -> String;    // "version,ciphers,exts,curves,pt_fmts" decimal/hex per JA3
pub fn ja3_hash(f: &Ja3Fields) -> String;      // md5 hex of ja3_string
pub fn ja4_a(f: &Ja3Fields) -> String;         // full JA4-A per FoxIO spec
```

JA4-A format (implement per the published spec): `t13d` + first cipher (4 hex, GREASE skipped) + `d` + extension count (2 digits) + sigalg count (2 digits) + alpn + `_` + sorted extension ids joined `,` (GREASE ids → `g`) + `_` + sorted sigalgs joined `,`.

- [ ] **Step 1: Write the failing test** — round-trip build → parse → JA3/JA4; JA3 md5 matches the known md5 of a hand-written canonical string; JA4 normalizes GREASE:

```rust
#[test]
fn parse_roundtrip_of_built_hello() {
    let spec = test_spec(); // same as Task 3
    let hello = build_hello(&spec, &params_fixed()).unwrap();
    let parsed = parse_hello(&hello.handshake_bytes).unwrap();
    assert_eq!(parsed.legacy_version, 0x0303);
    assert_eq!(parsed.cipher_suites.len(), spec.cipher_suites.len());
    assert!(parsed.extensions.iter().any(|(t, _)| *t == 0x0000)); // SNI present
}

#[test]
fn ja3_hash_known_value() {
    let f = Ja3Fields { version: 771, ciphers: vec![0x1301, 0x1302, 0x1303],
        extensions: vec![0x0000, 0x001a, 0x0a0a], curves: vec![0x001d, 0x0017],
        point_formats: vec![0], ..Default::default() };
    // md5("771,1301,1302,1303,0,1a,a0a,1d,17,0") — compute and hardcode
    assert_eq!(ja3_hash(&f), "<md5 hex>");
}

#[test]
fn ja4_normalizes_grease_and_sorts() {
    let f = Ja3Fields { version: 771, ciphers: vec![0x0A0A, 0x1301, 0x1302],
        extensions: vec![0x001b, 0x0a0a, 0x002b, 0x0017], curves: vec![],
        point_formats: vec![0], signature_algorithms: vec![0x0403, 0x0804],
        alpn: vec!["h2".into()] };
    // t13d + first non-grease cipher 1301 + d + ext_count 04 + sig_count 02 + h2
    assert_eq!(ja4_a(&f), "t13d1301d0402h2_17,1b,2b,g_403,804");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib hello::parse --lib crypto::fingerprint`
Expected: FAIL.

- [ ] **Step 3: Implement** — `parse.rs` (walk handshake body: skip type+len, read legacy_version, random, session_id len+bytes, cipher len+list, compression, extension list; tolerate trailing data); move `Ja3Fields`/`ja3_string`/`ja3_hash` from `crates/xray-tui-native/src/crypto/fingerprint/ja3.rs` (delete the native module in Task 11); implement `ja4_a` per spec. GREASE detection reuses `spec::is_grease`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): ClientHello parser + JA3/JA4 encoders"
```

---

### Task 5: crypto — ring port (key schedule, AEAD, X25519)

**Files:**
- Create: `crates/xray-tui-tls/src/crypto/mod.rs`

**Interfaces:**
- Consumes: `TlsError`.
- Produces (port of `tls-fingerprint/src/crypto.rs`, ring API):

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CipherSuiteId { Aes128GcmSha256, Aes256GcmSha384, Chacha20Poly1305Sha256 }
impl CipherSuiteId {
    pub fn from_u16(v: u16) -> Option<Self>;  // 0x1301/0x1302/0x1303
    pub fn to_u16(self) -> u16;
    pub fn digest(self) -> &'static ring::digest::Algorithm;   // SHA256/SHA384/SHA256
    pub fn aead(self) -> &'static ring::aead::Algorithm;       // AES_128_GCM/AES_256_GCM/CHACHA20_POLY1305
    pub fn key_len(self) -> usize;
    pub fn hash_len(self) -> usize;
}
pub struct X25519KeyPair { /* ring::agreement::EphemeralPrivateKey + [u8; 32] pub */ }
impl X25519KeyPair {
    pub fn generate(rng: &dyn ring::rand::SecureRandom) -> Result<Self, TlsError>;
    pub fn public_key(&self) -> [u8; 32];
    pub fn agree(&self, peer: &[u8; 32]) -> Result<[u8; 32], TlsError>;
}
pub struct KeySchedule { suite: CipherSuiteId, transcript: Vec<u8> }
impl KeySchedule {
    pub fn new(suite: CipherSuiteId) -> Self;
    pub fn add_transcript(&mut self, hs_msg: &[u8]);
    pub fn transcript_hash(&self) -> Vec<u8>;
    pub fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>, TlsError>;
    pub fn hkdf_expand_label(&self, prk: &[u8], label: &str, ctx: &[u8], len: usize) -> Result<Vec<u8>, TlsError>;
    pub fn derive_secret(&self, prk: &[u8], label: &str) -> Result<Vec<u8>, TlsError>; // HKDF-Expand-Label(_, label, Hash(transcript), hash_len)
}
pub struct AeadKey { key: ring::aead::LessSafeKey }
impl AeadKey {
    pub fn new(suite: CipherSuiteId, secret: &[u8]) -> Result<Self, TlsError>; // expand-label "key" from traffic secret
    pub fn seal(&self, seq: u64, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, TlsError>;
    pub fn open(&self, seq: u64, aad: &[u8], ciphertext: &mut [u8]) -> Result<Vec<u8>, TlsError>;
}
pub fn empty_hash(suite: CipherSuiteId) -> Vec<u8>;
```

- [ ] **Step 1: Write the failing test** — RFC 8448 Appendix vectors (TLS 1.3 handshake): use the published client/server traffic secrets for AES-128-GCM-SHA256 from the RFC 8448 test vectors; HKDF-Expand-Label known answer; AEAD seal/open round-trip with a fixed key:

```rust
#[test]
fn hkdf_expand_label_matches_rfc8448() {
    let ks = KeySchedule::new(CipherSuiteId::Aes128GcmSha256);
    // RFC 8448 §2: client_handshake_traffic_secret derivation. prk = early_secret etc.
    // Use the exact byte strings from the RFC and assert the derived secret.
    // (Copy the vector bytes from the RFC text during implementation.)
}

#[test]
fn aead_roundtrip() {
    let key = AeadKey::new(CipherSuiteId::Aes128GcmSha256, &[0x11; 16]).unwrap();
    let ct = key.seal(0, &[0x17, 0x03, 0x03, 0x00, 0x10], b"hello world").unwrap();
    let mut buf = ct.clone();
    let pt = key.open(0, &[0x17, 0x03, 0x03, 0x00, 0x10], &mut buf).unwrap();
    assert_eq!(pt, b"hello world");
}

#[test]
fn x25519_agrees() {
    let a = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
    let b = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
    let sa = a.agree(&b.public_key()).unwrap();
    let sb = b.agree(&a.public_key()).unwrap();
    assert_eq!(sa, sb);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib crypto`
Expected: FAIL.

- [ ] **Step 3: Implement** — port `tls-fingerprint/src/crypto.rs` to ring:
  - HKDF: `ring::hkdf::Salt::new(alg, salt).extract(ikm)` → `Prk`; expand with a `ring::hkdf::KeyType` adapter (copy the `ExpandLen` pattern from tls-fingerprint, replacing aws-lc-rs imports with ring's `ring::hkdf`).
  - `hkdf_expand_label`: encode `"tls13 " + label` per RFC 8446 §7.1, context = transcript hash (for `derive_secret`) or raw ctx bytes.
  - AEAD: `ring::aead::LessSafeKey::new(UnboundKey::new(alg, key))`; AAD = `[0x17, 0x03, 0x03, len_hi, len_lo]` built by the caller (`record::aead_aad`, Task 6).
  - Transcript: `ring::digest::digest(suite.digest(), data)`.
  - Match the tls-fingerprint key-schedule structure (EarlySecret → HandshakeSecret → MasterSecret, `client_handshake_traffic_secret` etc.) so Task 8 drops in unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib crypto`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): ring crypto port (key schedule, AEAD, X25519)"
```

---

### Task 6: record layer primitives

**Files:**
- Create: `crates/xray-tui-tls/src/record/mod.rs`

**Interfaces:**
- Consumes: `TlsError`; `crypto::AeadKey` (for the AAD helper only).
- Produces:

```rust
pub const CONTENT_CHANGE_CIPHER_SPEC: u8 = 0x14;
pub const CONTENT_ALERT: u8 = 0x15;
pub const CONTENT_HANDSHAKE: u8 = 0x16;
pub const CONTENT_APPLICATION_DATA: u8 = 0x17;
pub const HS_CLIENT_HELLO: u8 = 0x01; pub const HS_SERVER_HELLO: u8 = 0x02;
pub const HS_ENCRYPTED_EXTENSIONS: u8 = 0x08; pub const HS_CERTIFICATE: u8 = 0x0B;
pub const HS_CERTIFICATE_VERIFY: u8 = 0x0F; pub const HS_FINISHED: u8 = 0x14;

pub struct TlsRecord { pub content_type: u8, pub payload: Vec<u8> }
pub async fn read_record<S>(stream: &mut S) -> Result<TlsRecord, TlsError>
    where S: tokio::io::AsyncRead + Unpin;      // 5-byte header, then payload; alert → error with level/desc
pub async fn skip_ccs<S>(stream: &mut S) -> Result<TlsRecord, TlsError> where S: AsyncRead + Unpin;
pub fn parse_handshake_messages(payload: &[u8]) -> Result<Vec<(u8, Vec<u8>)>, TlsError>;
pub fn make_handshake_record(msg_type: u8, body: &[u8]) -> Vec<u8>;
pub fn make_app_data_record(ciphertext: &[u8]) -> Vec<u8>;
pub fn aead_aad(ciphertext_len: usize) -> [u8; 5];
```

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn read_record_parses_header_and_payload() {
    let mut stream = tokio::io::duplex(1024).0; // write side into a buf via duplex
    // Build: 16 03 03 00 05 01 02 03 04 05
    let mut w = tokio::io::duplex(1024).1;
    w.write_all(&[0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05]).await.unwrap();
    // (read from the other end)
}

#[test]
fn parse_multi_handshake_messages() {
    let payload = [0x02, 0x00, 0x00, 0x02, 0xaa, 0xbb, 0x08, 0x00, 0x00, 0x00]; // SH + EE
    let msgs = parse_handshake_messages(&payload).unwrap();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0], (0x02, vec![0xaa, 0xbb]));
}

#[test]
fn aad_matches_tls13_rule() {
    assert_eq!(aead_aad(0x0010), [0x17, 0x03, 0x03, 0x00, 0x10]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib record`
Expected: FAIL.

- [ ] **Step 3: Implement** — port `tls-fingerprint/src/record.rs`, changing `read_record`/`skip_ccs` from `&mut TcpStream` to `S: tokio::io::AsyncRead + Unpin` (use `AsyncReadExt::read_exact`). Alert handling: parse `[level, description]`, return `TlsError::Handshake("alert: <level> <desc>")`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib record`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): record layer primitives"
```

---

### Task 7: `TlsStream<S>` — record-framed AsyncRead/AsyncWrite

**Files:**
- Create: `crates/xray-tui-tls/src/record/stream.rs`

**Interfaces:**
- Consumes: `record` primitives (Task 6), `crypto::AeadKey`/`CipherSuiteId` (Task 5).
- Produces:

```rust
pub struct AppKeys { pub read_key: AeadKey, pub write_key: AeadKey, pub read_seq: u64, pub write_seq: u64 }
pub struct TlsStream<S> { inner: S, keys: AppKeys, read_buf: Vec<u8>, read_pos: usize, closed: bool }
impl<S: AsyncRead + AsyncWrite + Unpin + Send> TlsStream<S> {
    pub fn new(inner: S, keys: AppKeys) -> Self;
}
impl<S: AsyncRead + AsyncWrite + Unpin + Send> tokio::io::AsyncRead for TlsStream<S> { /* decrypt records on demand */ }
impl<S: AsyncRead + AsyncWrite + Unpin + Send> tokio::io::AsyncWrite for TlsStream<S> { /* encrypt + frame, 16KB split */ }
```

- [ ] **Step 1: Write the failing test** — duplex round-trip through the record layer with fixed keys; fragmented records; close_notify → EOF:

```rust
#[tokio::test]
async fn encrypted_duplex_roundtrip() {
    let (a, b) = tokio::io::duplex(4096);
    let key = AeadKey::new(CipherSuiteId::Aes128GcmSha256, &[0x11; 16]).unwrap();
    let keys = AppKeys { read_key: key.clone_key(), write_key: key.clone_key(), read_seq: 0, write_seq: 0 };
    let mut client = TlsStream::new(a, keys.clone());
    let mut server = TlsStream::new(b, keys);
    client.write_all(b"ping").await.unwrap();
    let mut buf = [0u8; 4];
    server.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"ping");
}
```

(`AeadKey` gets a `Clone`-friendly constructor — derive `Clone` on it; `clone_key()` = same key material. Adjust the crypto API in Task 5 if needed: `AeadKey::new` from raw secret, plus `AeadKey::from_key_bytes` for tests.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib record::stream`
Expected: FAIL.

- [ ] **Step 3: Implement** — `TlsStream<S>`:
  - Read: on `poll_read`, if buffered bytes remain serve them; else read one record (header + payload), decrypt in place with `read_key` (seq increments per record), extract inner content type + padding (port `strip_padding` from `tls-fingerprint/src/handshake.rs`); `CONTENT_APPLICATION_DATA` → buffer plaintext; `CONTENT_ALERT` close_notify(0) → EOF, other alerts → error; `CONTENT_HANDSHAKE` → error (no renegotiation).
  - Write: split plaintext at 16 KiB, encrypt with `write_key`, frame as `0x17 0x0303 <len>`, write all. Non-blocking write semantics: buffer partial writes (keep a small pending buffer) — implement via `poll_write` writing one record per poll.
  - Drop: no close_notify on drop (matches rustls default in this codebase; connection teardown = TCP close).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib record::stream`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): record-framed TlsStream (AsyncRead/AsyncWrite)"
```

---

### Task 8: TLS 1.3 client handshake (plain TLS, verifier seam)

**Files:**
- Create: `crates/xray-tui-tls/src/handshake/mod.rs`

**Interfaces:**
- Consumes: `build_hello` (Task 3), `parse_hello` (Task 4), `crypto` (Task 5), `record` (Tasks 6-7).
- Produces:

```rust
pub trait ServerVerifier: Send + Sync {
    fn verify(&self, ctx: &VerifyContext<'_>) -> Result<(), TlsError>;
}
pub struct VerifyContext<'a> {
    pub chain: &'a [Vec<u8>],              // DER certs, leaf first
    pub sni: &'a str,
    pub signature_scheme: u16,             // from CertificateVerify
    pub cert_verify_body: &'a [u8],        // raw CertificateVerify body
    pub transcript: &'a [u8],              // transcript up to (excl) CertificateVerify
    pub suite: CipherSuiteId,
}
pub struct AcceptAll;                      // test helper: Ok(())
impl ServerVerifier for AcceptAll { fn verify(&self, _: &VerifyContext<'_>) -> Result<(), TlsError> { Ok(()) } }

pub struct HandshakeParams<'a> {
    pub spec: &'a ClientHelloSpec,
    pub server_name: &'a str,
    pub alpn: Option<&'a [&'a str]>,
    pub verifier: &'a dyn ServerVerifier,
    pub rng: &'a dyn ring::rand::SecureRandom,
}
pub async fn connect<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S, params: HandshakeParams<'_>,
) -> Result<TlsStream<S>, TlsError>;
```

Handshake flow (port `tls-fingerprint/src/handshake.rs`):
1. `X25519KeyPair::generate`, `build_hello`, write `record_bytes`.
2. `read_server_hello` (skip CCS): parse ServerHello — record the raw handshake bytes into the transcript; parse `legacy_version`, `random`, `session_id`, cipher suite, `key_share` extension (peer X25519 pub); if the random is `CF 21 AD 74 E5 9A 61 11 BE 1D 8C 02 1E 65 B8 91 C2 A2 11 16 7A BB 8C 5E 07 9E 09 E2 C8 A8 33 9C` → return `TlsError::HelloRetryRequest`.
3. `shared = agree(peer_key)`; derive handshake secret; `handshake_traffic_secret`; read encrypted handshake records (port `read_server_hs_messages`): EncryptedExtensions (parse ALPN + supported_versions), Certificate (parse chain), CertificateVerify, Finished. Decrypt each with `server_hs_key` (seq from 0).
4. Verify: accumulate transcript up to (excl) CertificateVerify; call `params.verifier.verify(&VerifyContext { chain, sni, signature_scheme, cert_verify_body, transcript, suite })` — Task 9 implements the real verifier; the transcript after CertificateVerify is needed for the client Finished, so Task 8 computes the transcript internally for its own Finished and passes the pre-CV transcript to the verifier.
5. Client Finished: `client_hs_key` seal of `derive_secret(handshake_secret, "client finished", transcript)`; write as handshake record (encrypted with `client_hs_key`).
6. Derive `client_application_traffic_secret` / `server_application_traffic_secret`; build `AppKeys`; return `TlsStream::new(stream, keys)`.

- [ ] **Step 1: Write the failing test** — handshake against a rustls server (interop proof). Uses `rcgen` for a server cert (CN=localhost, SAN localhost + 127.0.0.1), `rustls::ServerConfig` (dev-dependency `rustls = { version = "0.23", features = ["ring"] }` in Cargo.toml), and `tokio-rustls`? NO — to prove OUR client speaks TLS 1.3, the server can be plain `rustls` over a duplex (no tokio-rustls needed — rustls `ServerConnection` is sync over `Read/Write`; use a thread with `tokio::task::spawn_blocking` and a `std::io::duplex`-style pipe, or simpler: `tokio::io::duplex` with a `std::sync::mpsc` pump). Simplest reliable pattern: run rustls server on a `TcpListener` in `spawn_blocking` (rustls `ServerConnection::read_tls`/`process_new_packets`/`writer`), client connects via `TcpStream`.

```rust
// handshake/mod.rs tests (requires dev-deps: rustls 0.23 ring, rcgen)
fn server_config(cert: rcgen::Certificate, key: &rcgen::KeyPair) -> rustls::ServerConfig {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert.der().to_vec())],
            PrivateKeyDer::try_from(key.serialize_der()).unwrap(),
        ).unwrap()
}

#[tokio::test]
async fn handshake_completes_against_rustls_server() {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()]).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cfg = server_config(cert, &cert.key_pair);
    let server = tokio::task::spawn_blocking(move || {
        let (sock, _) = listener.accept().unwrap();
        let mut conn = rustls::ServerConnection::new(cfg).unwrap();
        let mut tls_in = std::io::BufReader::new(sock.try_clone().unwrap());
        let mut tls_out = sock;
        // read_tls + process_new_packets until handshake complete, then echo
        // (standard rustls server loop; bounded by read timeouts)
    });
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let mut tls = connect(stream, HandshakeParams {
        spec: &test_spec(),
        server_name: "localhost",
        alpn: Some(&["http/1.1"]),
        verifier: &AcceptAll,
        rng: &ring::rand::SystemRandom::new(),
    }).await.unwrap();
    tls.write_all(b"ping").await.unwrap();
    let mut buf = [0u8; 4];
    tls.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"ping");
    server.await.unwrap();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib handshake`
Expected: FAIL (no `connect`).

- [ ] **Step 3: Implement** per the flow above, porting from `tls-fingerprint/src/handshake.rs` (adapt: generic stream, transcript handling, verifier seam, HRR detection, seq counters for decrypt).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib handshake`
Expected: PASS (rustls server completes the TLS 1.3 handshake with our client).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls Cargo.toml Cargo.lock
git commit -m "feat(tls): TLS 1.3 client handshake (interop-tested vs rustls)"
```

---

### Task 9: Certificate verification (rustls-webpki + ring)

**Files:**
- Create: `crates/xray-tui-tls/src/verify/mod.rs`

**Interfaces:**
- Consumes: `handshake::ServerVerifier`/`VerifyContext` (Task 8), `crypto::CipherSuiteId`.
- Produces:

```rust
pub struct WebPkiVerifier { roots: Vec<rustls_webpki::anchor::TrustAnchor<'static>>, insecure: bool, pin_sha256: Option<[u8; 32]> }
impl WebPkiVerifier {
    pub fn from_roots(roots: Vec<rustls_webpki::anchor::TrustAnchor<'static>>) -> Self;
    pub fn webpki_roots() -> Self;                          // webpki-roots::TLS_SERVER_ROOTS
    pub fn from_ca_der(ca_der: &[u8]) -> Result<Self, TlsError>; // single test CA (rustls_webpki::anchor_from_trusted_cert)
    pub fn with_insecure(mut self, insecure: bool) -> Self;
    pub fn with_pin(mut self, pin: [u8; 32]) -> Self;
}
impl ServerVerifier for WebPkiVerifier { /* chain + SAN + CertificateVerify signature */ }
```

Verification rules: `pin_sha256` present → only check SHA-256 of leaf SPKI (DER `SubjectPublicKeyInfo` from the leaf cert) equals the pin (chain skipped — v2rayN semantics). `insecure` → `Ok(())`. Else: `EndEntityCert::verify_for_usage(SUPPORTED_SIG_ALGS, &roots, &[], Time::try_from(now)?, KeyUsage::server_auth(), None, None)` + `verify_is_valid_for_subject_name(ServerName::try_from(sni)?)`; then verify the CertificateVerify signature over the transcript with the leaf SPKI via `ring::signature::UnparsedPublicKey` (scheme map: 0x0403 → ECDSA_P256_SHA256_ASN1, 0x0503 → ECDSA_P384_SHA384_ASN1, 0x0804 → RSA_PSS_2048_8192_SHA256, 0x0805 → RSA_PSS_2048_8192_SHA384, 0x0806 → RSA_PSS_2048_8192_SHA512, 0x0807 → ED25519; unknown scheme → `TlsError::Verify`).

- [ ] **Step 1: Write the failing test** — three cases against a rustls server with a known CA: valid chain passes; wrong SNI fails; pin matches/mismatches:

```rust
#[tokio::test]
async fn verifies_valid_chain_and_sni() {
    // rcgen CA + leaf (SAN localhost), leaf signed by CA; rustls server
    // presents leaf; WebPkiVerifier::from_ca_der(ca_der) passes.
    // (Reuse the handshake test server from Task 8, parameterized by verifier.)
}

#[tokio::test]
async fn rejects_wrong_sni() {
    // Same server, connect with server_name "wrong.example"
    // Expected: Err(TlsError::Verify(_))
}

#[tokio::test]
async fn pin_sha256_matches_leaf_spki() {
    // Compute sha256(leaf.spki_der()) → with_pin(pin) passes;
    // with_pin([0; 32]) fails.
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib verify`
Expected: FAIL.

- [ ] **Step 3: Implement** per the rules above. SPKI extraction: `EndEntityCert::as_ca()`? No — `rustls_pki_types::CertificateDer` → parse SPKI: `webpki::EndEntityCert::try_from(&der)?.subject_public_key_info()` returns `&[u8]` (SPKI DER) — hash with `ring::digest::digest(&ring::digest::SHA256, spki)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib verify`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): webpki certificate verification + pin support"
```

---

### Task 10: Profiles + `define_profiles!` macro

**Files:**
- Create: `crates/xray-tui-tls/src/profiles/mod.rs`
- Create: `crates/xray-tui-tls/src/profiles/chrome.rs`, `chrome119.rs`, `chrome_android130.rs`, `edge.rs`, `brave167.rs`, `opera114.rs`, `firefox.rs`, `firefox128esr.rs`, `safari.rs`, `safari_ios17.rs`, `chrome133.rs`
- Modify: `crates/xray-tui-tls/src/lib.rs` (export `profiles::BrowserProfile`)

**Interfaces:**
- Consumes: `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec` (Task 2).
- Produces:

```rust
macro_rules! define_profiles { ... }   // in profiles/mod.rs
define_profiles! {
    Chrome, Chrome119, Chrome130, ChromeAndroid130, Edge130, Brave167, Opera114,
    Firefox, Firefox128Esr, Safari17, SafariIos17, Chrome133;
    Chrome119       => ("chrome_119",        chrome119::spec),
    Chrome130       => ("chrome_130",        chrome::spec),
    Chrome133       => ("chrome_133",        chrome133::spec),
    ChromeAndroid130=> ("chrome_android_130", chrome_android130::spec),
    Edge130         => ("edge_130",          edge::spec),
    Brave167        => ("brave_167",         brave167::spec),
    Opera114        => ("opera_114",         opera114::spec),
    Firefox         => ("firefox",           firefox::spec),
    Firefox128Esr   => ("firefox_128_esr",   firefox128esr::spec),
    Safari17        => ("safari_17",         safari::spec),
    SafariIos17     => ("safari_ios_17",     safari_ios17::spec),
}
// generates:
pub enum BrowserProfile { Chrome, Chrome119, /* ... */ }
impl BrowserProfile {
    pub fn name(self) -> &'static str;      // "chrome_130", ...
    pub fn spec(self) -> ClientHelloSpec;   // dispatch via match
    pub fn all() -> &'static [BrowserProfile];
}
```

The macro (adapted from `wreq-util/src/emulate.rs` `define_enum!`): variant list, then `Variant => ("name", path::spec)` pairs; generates the enum, `name()`, `spec()`, `all()`.

- [ ] **Step 1: Write the failing test** — every profile builds + parses; JA3/JA4 stable under fixed seed; macro dispatch names:

```rust
#[test]
fn all_profiles_build_and_parse() {
    for profile in BrowserProfile::all() {
        let spec = profile.spec();
        let rng = FixedRandom { bytes: vec![0x5A; 256], pos: 0 };
        let hello = build_hello(&spec, &BuildParams {
            server_name: "tls.peet.ws",
            alpn: None, // use spec's Alpn
            x25519_pub: &[0xAB; 32],
            rng: &rng,
        }).unwrap();
        let parsed = parse_hello(&hello.handshake_bytes).unwrap();
        let fields = Ja3Fields::from(&parsed);
        // Firefox family has no GREASE: JA3 hash is stable across seeds.
        if !matches!(profile, BrowserProfile::Firefox | BrowserProfile::Firefox128Esr) {
            assert!(parsed.cipher_suites.iter().any(|c| is_grease(*c)));
        }
        assert!(!ja3_hash(&fields).is_empty());
        assert!(ja4_a(&fields).starts_with("t13d"));
    }
}

#[test]
fn firefox128esr_ja3_matches_reference() {
    let spec = BrowserProfile::Firefox128Esr.spec();
    let hello = build_hello(&spec, &BuildParams { /* fixed seed */ }).unwrap();
    let fields = Ja3Fields::from(&parse_hello(&hello.handshake_bytes).unwrap());
    assert_eq!(ja3_hash(&fields), "361e0ca6ef1ca4dbe3a1d987722a1980");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib profiles`
Expected: FAIL.

- [ ] **Step 3: Implement** — port each `tls-fingerprint/src/profiles/*.rs` `build()` fn into a `spec()` fn returning `ClientHelloSpec`. The ported profile data is the constants + extension list from each file, re-expressed via `ExtensionSpec` (ServerName → placeholder filled at build; `key_share_x25519_with_grease` → `KeyShare(vec![Grease, X25519])`; `grease_ext` → `Grease`; ALPN literal from the profile). `chrome133.rs`: transcribe the Chrome-133 ClientHello from `thirdparty/utls/u_parrots.go` (find the `HelloChrome_133` spec — cipher suites, extension order, GREASE slots; uTLS presets are the tlsfingerprint.io captures). Firefox specs: no GREASE placeholders, ALPN h2,http/1.1, session id Random32, `SessionIdSpec::Random32`. Verify the Firefox128Esr JA3 test passes; if it doesn't, the port drifted — diff against the reference `/tmp/tf` output.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib`
Expected: PASS (including all earlier tests — hello golden test now uses the REAL `chrome::spec`).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): browser profiles as spec data + define_profiles! macro"
```

---

### Task 11: Native integration — custom TlsConnector + wrap() dispatch

**Files:**
- Create: `crates/xray-tui-native/src/security/fingerprint.rs`
- Modify: `crates/xray-tui-native/src/security/mod.rs` (wrap dispatch)
- Modify: `crates/xray-tui-native/src/security/tls_provider.rs` (move `HelloProvisioner` out; re-export from xray-tui-tls; doc updates)
- Modify: `crates/xray-tui-native/src/security/reality.rs` (keep trait re-exports; body lands in Task 13)
- Modify: `crates/xray-tui-native/src/context.rs` (add `tls_provider: TlsProvider` to `NativeConnectParams` + `LinkContext::tls_provider()` accessor + `Default`)
- Modify: `crates/xray-tui-native/src/security/tls.rs` (doc: Standard path; shared `TEST_CFG` pattern documented as the model for `fingerprint.rs`)
- Delete: `crates/xray-tui-native/src/crypto/fingerprint/` (moved to xray-tui-tls)
- Modify: `crates/xray-tui-native/Cargo.toml` (add `xray-tui-tls.workspace = true`)
- Modify: root `Cargo.toml` (workspace dep already added in Task 1)

**Interfaces:**
- Consumes: `xray-tui-tls::{build_hello, handshake::connect, profiles::BrowserProfile, verify::WebPkiVerifier}`.
- Produces:

```rust
// native security/fingerprint.rs
pub struct FingerprintConnector; // default custom engine
fn parse_fingerprint_id(s: &str) -> Result<tls_provider::FingerprintId, NativeError>; // exact ids only
fn profile_for(id: tls_provider::FingerprintId) -> Result<BrowserProfile, NativeError>;
    // Chrome/ChromeRandomized → Chrome130, Firefox → Firefox128Esr, Safari → Safari17, Random → Chrome130
impl tls_provider::TlsConnector for FingerprintConnector {
    // builds HandshakeParams { spec: profile_for(fp).spec(), server_name: params.sni,
    //   alpn: params.alpn as &[&str], verifier: &WebPkiVerifier (from thread-local test CA
    //   or webpki-roots; insecure from TlsOpts; pin from TlsOpts.pin_sha256), rng: SystemRandom }
    // → handshake::connect(stream, params), boxed
}
#[doc(hidden)] #[cfg(any(test, feature = "native-e2e"))] thread_local TEST_CA: RefCell<Option<WebPkiVerifier>>;
#[doc(hidden)] pub fn set_test_ca(ca_der: &[u8]); // mirrors tls.rs set_test_config
```

`wrap()` dispatch (security/mod.rs):

```rust
match &sec.tls {
    Some(TlsConfig::Tls(opts)) => {
        let custom = matches!(ctx.params.tls_provider, TlsProvider::Custom(_))
            || opts.fp.as_ref().is_some_and(|f| !f.is_empty());
        if custom {
            let connector: Arc<dyn TlsConnector> = match &ctx.params.tls_provider {
                TlsProvider::Custom(c) => c.clone(),
                _ => Arc::new(FingerprintConnector),
            };
            let params = TlsParams { sni: ctx.sni(), alpn: ctx.alpn_vec(),
                fingerprint: opts.fp.as_deref().map(parse_fingerprint_id).transpose()? };
            connector.connect(stream, params).await
        } else {
            tls::connect(ctx, stream).await
        }
    }
    Some(TlsConfig::Reality(_)) => reality::connect(ctx, stream).await, // Task 13 real
    None => Ok(stream),
}
```

`FingerprintConnector` reads `insecure`/`pin_sha256` from the `TlsOpts` (pass them through `TlsParams` — extend `TlsParams` with `insecure: bool, pin_sha256: Option<[u8; 32]>`).

- [ ] **Step 1: Write the failing test** — mirror `security/mod.rs`'s existing `tls_wraps_and_passes_bytes` but through the fingerprint path with a harness CA:

```rust
// security/mod.rs tests — new
#[tokio::test]
async fn fingerprint_tls_wraps_and_passes_bytes() {
    // rcgen CA + server cert (SAN localhost); rustls server (thread)
    // set_test_ca(&ca_der);
    // ctx with vless_with_tls("localhost") + fp: Some("chrome") + tls_provider: Custom(FingerprintConnector)
    // wrap() → write "ping" → read echo
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib security`
Expected: FAIL.

- [ ] **Step 3: Implement** per above. Also: `HelloProvisioner`/`HelloProvisionParams`/`ProvisionedHello`/`FixedChrome133` move from `native security/reality.rs` to `xray-tui-tls/src/reality/mod.rs` (Task 13 fills `FixedChrome133`; native `reality.rs` re-exports them for source compat until Task 14). Delete `crypto/fingerprint/`; fix any imports. Update `tls.rs` module doc (Standard path note). Extend `TlsParams` with `insecure` + `pin_sha256` and thread them through `FingerprintConnector`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-native --lib && cargo test -p xray-tui-tls --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native crates/xray-tui-tls Cargo.toml Cargo.lock
git commit -m "feat(native): wire fingerprint TlsConnector into security wrap"
```

---

### Task 12: Grader example + tier-2 tls.peet.ws verification

**Files:**
- Create: `crates/xray-tui-tls/src/http2/mod.rs` (port `tls-fingerprint/src/http2.rs`: client SETTINGS/HEADERS/DATA framing + response assembly)
- Create: `crates/xray-tui-tls/examples/grader.rs`
- Create: `crates/xray-tui-tls/tests/tls_peet_ws.rs` (`#[ignore]`d)

**Interfaces:**
- Consumes: `handshake::connect`, `profiles::BrowserProfile`, `verify::WebPkiVerifier::webpki_roots()`, `record::stream::TlsStream`, http2 framing.

- [ ] **Step 1: Write the grader** (`examples/grader.rs`) — port `tls-fingerprint/examples/fingerprint_test.rs`: for each of `[Chrome130, Firefox128Esr]` (all profiles optional via CLI arg): connect to `tls.peet.ws:443`, ALPN `["h2", "http/1.1"]`, verifier = `WebPkiVerifier::webpki_roots()`, HTTP/2 GET `/api/all` through the http2 framing, parse JSON, print + assert:

```rust
const EXPECTED: &[(BrowserProfile, &str, &str)] = &[
    (BrowserProfile::Chrome130,     "<locked ja4>", "<locked ja3>"),
    (BrowserProfile::Firefox128Esr, "t13d1314h2_07be0c029dc8_46701d79520f", "361e0ca6ef1ca4dbe3a1d987722a1980"),
];
```

JA3 comparison note: Chrome's GREASE is per-connection random, so its JA3 hash varies run to run. Assert the JA4 (GREASE-normalized) strictly; for JA3 assert only Firefox (stable) or compare with GREASE masked. Lock the Chrome130 JA4 constant from the first live run after confirming it matches real Chrome 130 (cross-check `tlsfingerprint.io` / the uTLS preset capture for Chrome 130).

- [ ] **Step 2: Verify it fails/behaves**

Run: `cargo run -p xray-tui-tls --example grader -- --profile chrome_130`
Expected: prints tls.peet.ws JSON fields (tls_version_negotiated 772, ja3/ja4). Network required — this is the tier-2 gate.

- [ ] **Step 3: Reconcile tls.peet.ws JA4 semantics** — if tls.peet.ws's reported `ja4` differs from our offline `ja4_a()` on the same connection, inspect their raw fields (`tls.ciphers`, `tls.extensions`, `tls.sigalgs`, `tls.alpn`) and replicate their exact computation in the grader's expected-value path (document the divergence in the test). The offline `ja4_a()` stays per the published spec.

- [ ] **Step 4: Write the `#[ignore]`d test** (`tests/tls_peet_ws.rs`): `#[ignore = "network"]` — same assertions as the grader, run via `cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored`.

- [ ] **Step 5: Lock constants + commit**

Capture the final expected JA3/JA4 (Chrome130 + Firefox128ESR) from a live grader run; hardcode them in both the example and the ignored test; commit.

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): tls.peet.ws grader + ignored tier-2 verification"
```

---

### Task 13: REALITY client (ring port from shoes)

**Files:**
- Create: `crates/xray-tui-tls/src/reality/mod.rs` (trait re-exports + `connect_reality`)
- Create: `crates/xray-tui-tls/src/reality/auth.rs` (port `reality_auth.rs`)
- Create: `crates/xray-tui-tls/src/reality/verify.rs` (port `reality_client_verify.rs`)
- Create: `crates/xray-tui-tls/src/reality/messages.rs` (port `reality_tls13_messages.rs` + `reality_util.rs` client-side pieces)
- Modify: `crates/xray-tui-tls/src/hello/mod.rs` — `SessionIdSpec::AuthPayload` already supported (Task 3)

**Interfaces:**
- Consumes: `crypto` (Task 5), `record` (Tasks 6-7), `spec`/`build_hello` (Tasks 2-3).
- Produces (wire contract = the 9-step doc in `crates/xray-tui-native/src/security/reality.rs`):

```rust
pub trait HelloProvisioner: Send + Sync {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello, TlsError>;
}
pub struct HelloProvisionParams<'a> { pub server_name: &'a str, pub alpn: Option<&'a [&'a str]> }
pub struct ProvisionedHello { pub handshake_bytes: Vec<u8>, pub session_id_range: std::ops::Range<usize> }
pub struct FixedChrome133; // Chrome-133 spec with SessionIdSpec::AuthPayload{len:32}
impl HelloProvisioner for FixedChrome133;

pub struct RealityParams<'a> {
    pub server_name: &'a str,        // steal target SNI
    pub public_key: &'a [u8; 32],    // server pbk (x25519 pub, decoded)
    pub short_id: &'a [u8],          // ≤8 bytes
    pub provisioner: &'a dyn HelloProvisioner,
    pub rng: &'a dyn ring::rand::SecureRandom,
}
pub async fn connect_reality<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S, params: RealityParams<'_>,
) -> Result<TlsStream<S>, TlsError>;
```

Handshake (port of `shoes/src/reality/reality_client_connection.rs` + friends, ring):
1. X25519 keypair; client_random (32 B).
2. `shared = agree(server_pub)`; `auth_key = HKDF-SHA256(shared, salt = random[0..20], info "REALITY")`.
3. SessionId plaintext (16 B) = `[1, 8, 0, 0] + timestamp u32 BE + short_id (8, zero-padded)`.
4. `provisioner.provision` → hello with SessionId zeros (range returned); splice: seal plaintext with `AES-256-GCM(auth_key, nonce = random[20..32], aad = hello with session_id zeroed)` and write the 32-byte result into `session_id_range`.
5. Send hello record.
6. Read ServerHello (skip CCS; HRR → error). ECDH shared → TLS 1.3 handshake secret via the same `KeySchedule` as Task 8.
7. Decrypt EE + Certificate + CertificateVerify + Finished (server handshake key).
8. Server auth (port `reality_client_verify.rs`): certificate's signature field must equal `HMAC-SHA512(auth_key, ed25519_pub_of_cert)`; then verify the Ed25519 `CertificateVerify` over the transcript with the cert's ed25519 SPKI. No chain.
9. Client Finished; app keys; `TlsStream`.

- [ ] **Step 1: Write the failing test** — auth-key derivation known-answer + session seal/unseal round-trip with fixed inputs; full handshake tested in Task 14 (real cores):

```rust
#[test]
fn session_id_seal_roundtrip() {
    // fixed: server pub (from a known X25519 keypair), client keypair,
    // random bytes [0..32]. Derive shared + auth_key; seal plaintext into a
    // hello built with AuthPayload; unseal with the same auth_key; assert
    // plaintext == [1,8,0,0, ts_be, short_id...].
    // RFC 7748 §6.1 X25519 vector as the ECDH known-answer.
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-tls --lib reality`
Expected: FAIL.

- [ ] **Step 3: Implement** per the flow above. Porting notes: shoes `reality_auth.rs` `derive_auth_key`/`perform_ecdh`/`encrypt_session_id` (aws-lc-rs → ring: `agreement` + `hkdf` + `aead::AES_256_GCM`); `reality_tls13_messages.rs` `construct_client_hello` → REPLACED by our `build_hello` + provisioner (shoes' `RealityClientConfig` maps onto `RealityParams`); `reality_records.rs`/`reality_io_state.rs`/`slide_buffer.rs` → REPLACED by our `record` + `TlsStream`; `reality_client_verify.rs` → port directly (ring HMAC + ED25519 verify). Timestamp: seconds since UNIX epoch.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-tls --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls
git commit -m "feat(tls): REALITY client (ring port from shoes)"
```

---

### Task 14: Native Reality wiring + tier-3 e2e cases

**Files:**
- Modify: `crates/xray-tui-native/src/security/reality.rs` — real `connect()` + `RealityConnector` (`impl TlsConnector`), keep `HelloProvisionerChoice` (provisioner = `FixedChrome133` default, `Custom(Arc<dyn HelloProvisioner>)`); `HelloProvisioner` re-exported from xray-tui-tls
- Modify: `crates/xray-tui-native/src/security/mod.rs` — `Reality` arm calls `reality::connect`
- Modify: `crates/xray-tui-native/src/e2e/variant.rs` — add `TlsVariant` trait + `StandardTls`, `FingerprintTls(&'static str)`, `RealityTls { pbk: String, sid: String }`
- Modify: `crates/xray-tui-native/src/e2e/case.rs` — `CaseSpec` gains `tls: Option<Box<dyn TlsVariant>>`; `server_config`/`client_params`/`client_trust` honor it (reality inbound JSON for xray + sing-box; fingerprint client sets `fp` + `TlsProvider::Custom`; reality client sets `security=reality` + pbk/sid)
- Modify: `crates/xray-tui-native/src/e2e/config.rs` — reality inbound builders + `pbk`/`sid` generation (x25519 keypair: private → server `privateKey`, public → client `pbk`, both base64url; sid = 8 hex chars)
- Modify: `crates/xray-tui-native/tests/vless.rs`, `tests/vmess.rs` — add cases: `vless-tls-chrome`, `vmess-tls-firefox`, `vless-reality`, `vmess-reality`
- Modify: `crates/xray-tui-native/src/security/fingerprint.rs` — `set_test_ca` shared by both standard + fingerprint variants (`client_trust` installs CA for all TLS variants; Reality needs no CA)

**Interfaces:**
- Consumes: `xray-tui-tls::reality::{connect_reality, RealityParams, FixedChrome133}` (Task 13), e2e harness (existing).
- Produces: tier-3 proof — native VLESS/VMESS through the fingerprint engine AND REALITY, against real xray + sing-box.

- [ ] **Step 1: Write the failing test** — new e2e cases:

```rust
// tests/vless.rs — add
#[tokio::test]
async fn vless_tls_chrome_against_cores() {
    run_against_cores(&CaseSpec::vless().with_tls(TlsVariant::fingerprint("chrome")))
        .await.expect("vless tls chrome e2e failed");
}
#[tokio::test]
async fn vless_reality_against_cores() {
    run_against_cores(&CaseSpec::vless().with_tls(TlsVariant::reality()))
        .await.expect("vless reality e2e failed");
}
// tests/vmess.rs — vmess_tls_firefox + vmess_reality (with Aes128GcmVariant)
```

- [ ] **Step 2: Run to verify (needs cores)**

Run: `XRAY_TUI_CORE_BIN_DIR=<dir with xray + sing-box> cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: FAIL (Reality arm still NotImplemented).

- [ ] **Step 3: Implement** per above. Reality inbound config (xray): `"realitySettings": { "show": false, "dest": "<echo server>", "serverNames": ["<sni>"], "privateKey": "<b64url>", "shortIds": ["<sid>"] }`; sing-box: `"reality": { "enabled": true, "handshake": { "server": "<echo host>", "server_port": <port> }, "private_key": "<b64url>", "short_id": ["<sid>"] }`. Client params for reality: `SecurityConfig { tls: Some(TlsConfig::Reality(RealityOpts { sni, pbk, sid, fp: Some("chrome"), .. })) }`. dest/handshake point at the local echo server so non-reality traffic degrades gracefully; our client is reality-authenticated and tunnels the probe.
   Note: sing-box reality may require `handshake` server reachable — the echo server satisfies it.

- [ ] **Step 4: Run to verify passes**

Run: `XRAY_TUI_CORE_BIN_DIR=... cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: PASS (all 8 cases: vless/vmess × tls-standard/tls-chrome/tls-firefox/reality × xray/sing-box).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native
git commit -m "feat(native): REALITY wiring + fingerprint/reality e2e cases"
```

---

### Task 15: Cleanup + docs + full verification

**Files:**
- Modify: `AGENTS.md` — add `crates/xray-tui-tls/src/…` to Key Source Files; document the custom TLS engine decision (replaces the "M2" forward references); update the `xray-tui-native` section
- Modify: `crates/xray-tui-native/src/security/tls.rs`, `tls_provider.rs`, `reality.rs` — final doc pass (no stale "M2/M3" markers)
- Verify: `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --check`

- [ ] **Step 1: Doc updates** — AGENTS.md + stale milestone comments.
- [ ] **Step 2: Full test run**

Run: `cargo test --workspace`
Expected: PASS (tier-2 ignored tests not run; tier-3 gated by feature+env).

- [ ] **Step 3: Clippy + fmt**

Run: `cargo clippy --workspace --all-targets && cargo fmt --check`
Expected: PASS (fix any lints; workspace lints are pedantic+nursery warn — `#[allow]` only where the ported code justifies it).

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md crates
git commit -m "docs(native): TLS mimicry engine — AGENTS.md + final cleanup"
```

---

## Self-review

- **Spec coverage:** spec decisions 1-7 map to Tasks 1-15: ring (5), hybrid extensibility (2, 3, 10), cert verification (9), TLS 1.3 only + HRR error (8), REALITY (13, 14), new crate (1), Chrome130/FF128ESR verification (12). Tier-1 (4, 10), tier-2 (12), tier-3 (14). Integration details (11). HTTP/2 grader (12).
- **Type consistency:** `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec` introduced in Task 2, consumed 3-14. `TlsStream<S>` introduced Task 7, consumed 8/13. `ServerVerifier` introduced 8, implemented 9, consumed 11. `HelloProvisioner` moved 11, implemented 13, wired 14. `TlsParams.insecure/pin_sha256` extended 11, consumed by `FingerprintConnector` same task.
- **Known risk (accepted):** tls.peet.ws JA4 semantics reconciled empirically in Task 12; Chrome130 JA3 locked after GREASE masking in Task 12.
