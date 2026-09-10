# Native Shadowsocks / Shadowsocks-2022 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Shadowsocks classic-AEAD and 2022-blake3 client (TCP + UDP) inside `xray-tui-native`, behind a capability gate, with unit tests, real-core e2e rows, and benchmarks.

**Architecture:** Two codec families (`protocol/ss/stream.rs`, `stream2022.rs`) over the shared `crypto/aead.rs` + `crypto/kdf.rs` primitives, wired into the existing uniform pipeline (`chain.rs` → `protocol::connect`) and the UDP carrier family (`PacketTunnel::Ss`). The `blake3`/`hkdf`/`sha1` crates own the KDFs; the classic password→key MD5 `EVP_BytesToKey` stays in `protocol/ss/method.rs`.

**Tech Stack:** Rust 2024, tokio, RustCrypto AEAD (`aes-gcm` 0.11, `chacha20poly1305` 0.11) + `sha1`/`hkdf`/`blake3`, `md-5`, criterion, rstest, real xray-core 26.3.27 / sing-box 1.13.16 servers.

**Spec:** `docs/superpowers/specs/2026-09-10-native-shadowsocks-design.md`

## Global Constraints

- **Dependency set is frozen to three new crates:** `sha1 = { version = "0.11", features = ["zeroize"] }`, `hkdf = "0.13"`, `blake3 = { version = "1", features = ["zeroize"] }`. `blake3` moves from `[dev-dependencies]` to `[dependencies]` (remove the dev-dependency line). No other manifest change beyond the new `[[bench]] ss_codec` target.
- **No `xray-tui-proto` change.** `SsConfig { method, password, security, remarks, plugin, plugin_opts }` already carries everything.
- **Wire references (authoritative, read-only):** 2022 edition spec §2.2/§3.1/§3.2/§4.1; `thirdparty/shadowsocks-rust/crates/shadowsocks/src/relay/{tcp,udp}relay/aead{,_2022}.rs`; `thirdparty/mihomo/transport/shadowsocks/{shadowaead,shadowstream}`; `thirdparty/Xray-core/proxy/shadowsocks/config.go` + `infra/conf/shadowsocks.go`.
- **Secret hygiene (decision 19):** master keys, session subkeys, and HKDF output buffers are `Zeroizing`; `sha1`/`blake3` carry the explicit `zeroize` feature; after the manifest edit run `cargo tree -p xray-tui-native -e features | grep zeroize` and keep the output in the task notes.
- **After any `Cargo.lock` change:** `cargo hakari generate`, and append the new crates (`sha1`, `hkdf`, `blake3`) to the `xray-tui-hakari` machete ignore list using the AGENTS.md awk recipe. `just quality-gate` must pass at the end.
- **UDP mode guard unchanged:** `reject_vless_only_mode` stays as-is; SS rows use `PacketMode::Raw` (inert — the wire always carries a per-packet address).
- **Out of scope (never add silently):** SIP003 plugins, legacy stream ciphers, 2022 multi-user EIH, server side, `PacketMode` additions.
- **Skip per-task formatters/linters/suites.** Run `cargo fmt`/`clippy`/full nextest once at the end (Task 11).
- **Commits are the coordinator's.** This plan runs under subagent-driven development: implementers edit and test but never stage/commit/branch; the coordinator stages the task-owned paths and creates one task commit after the spec-compliance and code-quality reviews pass. Every "Step N: Commit" below is the coordinator's step, and the listed message is the suggested one.
- Salts come from `crate::rand::fill_nonsecret` (wire-visible filler).
- Errors: return `NativeError::Config` for bad method/password, `NativeError::Protocol { kind: ProtocolKind::Shadowsocks|Shadowsocks2022, detail }` for wire failures.

---

### Task 0: Dependencies + KDF primitives

**Files:**
- Modify: `crates/xray-tui-native/Cargo.toml`
- Create: `crates/xray-tui-native/src/crypto/kdf.rs` (replace the stub body; keep the module doc, add the SS section)
- Modify: `crates/xray-tui-native/src/crypto/mod.rs:12` (no change needed — `kdf` is already declared)

**Interfaces:**
- Consumes: nothing.
- Produces: `crypto::kdf::hkdf_sha1(psk: &[u8], salt: &[u8], info: &[u8], out: &mut [u8])`, `crypto::kdf::blake3_derive_key(context: &str, material: &[u8]) -> [u8; 32]`, `crypto::kdf::evp_bytes_to_key_md5(password: &[u8], key_len: usize) -> Zeroizing<Vec<u8>>`.

- [ ] **Step 1: Add the dependencies**

```toml
# in [dependencies], next to the other RustCrypto crypto deps
# Classic-AEAD subkey KDF: HKDF-SHA1(psk, salt, b"ss-subkey").
sha1 = { version = "0.11", features = ["zeroize"] }
hkdf = "0.13"
# 2022-blake3 session subkey: blake3::derive_key("shadowsocks 2022 session subkey",
# psk || salt). Promoted from dev-dependency (b3.rs cross-validation uses it too).
blake3 = { version = "1", features = ["zeroize"] }
```
Delete the `blake3 = "1"` line from `[dev-dependencies]`.

- [ ] **Step 2: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// HKDF-SHA1 pinned to vectors computed with an INDEPENDENT
    /// implementation (Python `hmac`/`hashlib`, HMAC-SHA1 extract/expand per
    /// RFC 5869) — not with the crate under test. First case: IKM = 0x0b×22,
    /// salt = 0x00..0x0c, info = 0xf0..0xf9, L = 42.
    #[test]
    fn hkdf_sha1_matches_independent_vectors() {
        let ikm = [0x0b_u8; 22];
        let salt: Vec<u8> = (0..13u8).collect();
        let info: Vec<u8> = (0xf0..0xfa).collect();
        let mut okm = [0u8; 42];
        hkdf_sha1(&ikm, &salt, &info, &mut okm);
        assert_eq!(
            okm,
            [
                0xd6, 0x00, 0x0f, 0xfb, 0x5b, 0x50, 0xbd, 0x39, 0x70, 0xb2, 0x60, 0x01, 0x77, 0x98,
                0xfb, 0x9c, 0x8d, 0xf9, 0xce, 0x2e, 0x2c, 0x16, 0xb6, 0xcd, 0x70, 0x9c, 0xca, 0x07,
                0xdc, 0x3c, 0xf9, 0xcf, 0x26, 0xd6, 0xc6, 0xd7, 0x50, 0xd0, 0xaa, 0xf5, 0xac, 0x94,
            ]
        );
    }

    /// The exact classic-SS call: subkey = HKDF-SHA1(psk, salt, "ss-subkey").
    #[test]
    fn ss_subkey_matches_independent_vector() {
        let mut out = [0u8; 16];
        hkdf_sha1(b"password", &[0x11_u8; 16], b"ss-subkey", &mut out);
        assert_eq!(
            out,
            [
                0x8e, 0x2b, 0x1a, 0x61, 0x11, 0x23, 0x92, 0x29, 0x40, 0x0b, 0x5d, 0xd6, 0x12, 0x77,
                0x19, 0x31,
            ]
        );
    }

    /// The 2022 helper must delegate to the crate with the spec's ASCII
    /// context. (The derive-key primitive's independence is pinned in
    /// `protocol/vless/encryption/b3.rs`; the caller's `key ‖ salt` material
    /// order is pinned in `protocol/ss/method.rs`'s tests, against that same
    /// hand-rolled implementation.)
    #[test]
    fn blake3_helper_uses_the_spec_context() {
        assert_eq!(SS2022_SUBKEY_CONTEXT, "shadowsocks 2022 session subkey");
        let key = [0x22_u8; 32];
        let salt = [0x33_u8; 32];
        let material = [key.as_slice(), salt.as_slice()].concat();
        assert_eq!(
            blake3_derive_key(SS2022_SUBKEY_CONTEXT, &material),
            blake3::derive_key("shadowsocks 2022 session subkey", &material)
        );
        assert_ne!(
            blake3_derive_key(SS2022_SUBKEY_CONTEXT, &material),
            blake3_derive_key(SS2022_SUBKEY_CONTEXT, &{
                let mut swapped = salt.to_vec();
                swapped.extend_from_slice(&key);
                swapped
            }),
            "the underlying KDF is order-sensitive (the caller's `key ‖ salt` \
             concatenation is pinned in protocol/ss/method.rs's tests against \
             the hand-rolled BLAKE3)"
        );
    }

    /// Classic password → key: MD5 `EVP_BytesToKey`, ALWAYS (xray
    /// `passwordToCipherKey` — no raw-password shortcut). Second block =
    /// MD5(digest ‖ password).
    #[test]
    fn evp_bytes_to_key_md5_vectors() {
        assert_eq!(
            &*evp_bytes_to_key_md5(b"password", 16),
            &[
                0x5f, 0x4d, 0xcc, 0x3b, 0x5a, 0xa7, 0x65, 0xd6, 0x1d, 0x83, 0x27, 0xde, 0xb8, 0x82,
                0xcf, 0x99,
            ]
        );
        assert_eq!(
            &*evp_bytes_to_key_md5(b"password", 32),
            &[
                0x5f, 0x4d, 0xcc, 0x3b, 0x5a, 0xa7, 0x65, 0xd6, 0x1d, 0x83, 0x27, 0xde, 0xb8, 0x82,
                0xcf, 0x99, 0x2b, 0x95, 0x99, 0x0a, 0x91, 0x51, 0x37, 0x4a, 0xbd, 0x8f, 0xf8, 0xc5,
                0xa7, 0xa0, 0xfe, 0x08,
            ]
        );
        // 24 bytes is the ONLY non-multiple-of-16 key length in the classic
        // cipher table (aes-192-gcm), so it is the only case that exercises
        // the final `truncate` — independent vector, same provenance.
        assert_eq!(
            &*evp_bytes_to_key_md5(b"password", 24),
            &[
                0x5f, 0x4d, 0xcc, 0x3b, 0x5a, 0xa7, 0x65, 0xd6, 0x1d, 0x83, 0x27, 0xde, 0xb8, 0x82,
                0xcf, 0x99, 0x2b, 0x95, 0x99, 0x0a, 0x91, 0x51, 0x37, 0x4a,
            ]
        );
    }

    /// The in-tree hand-rolled BLAKE3 (already pinned against the `blake3`
    /// crate by `derive_key_matches_reference_crate`) must agree with the
    /// helper on the 2022 context — an implementation the crate call cannot
    /// silently define into correctness. Add this test to
    /// `protocol/vless/encryption/b3.rs`'s test module.
    #[test]
    fn ss2022_subkey_matches_hand_rolled_derive_key() {
        let key = [0x22_u8; 32];
        let salt = [0x33_u8; 32];
        let material = [key.as_slice(), salt.as_slice()].concat();
        assert_eq!(
            crate::crypto::kdf::blake3_derive_key("shadowsocks 2022 session subkey", &material),
            derive_key_bytes(b"shadowsocks 2022 session subkey", &material)
        );
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p xray-tui-native --lib crypto::kdf`
Expected: FAIL — `cannot find function hkdf_sha1` / `blake3_derive_key` / `evp_bytes_to_key_md5`.

- [ ] **Step 4: Implement the module and pin the two generated vectors**

```rust
//! HKDF / SHA helpers shared by protocol crypto.
//! ... (keep the existing vmess/reality doc paragraph)
//!
//! Shadowsocks consumers: classic AEAD derives its per-salt stream subkey with
//! `HKDF-SHA1(psk, salt, b"ss-subkey")` (xray `config.go::hkdfSHA1`, mihomo
//! `shadowaead/cipher.go::hkdfSHA1`), and the classic password→key step is
//! OpenSSL's `EVP_BytesToKey` with MD5 (xray `passwordToCipherKey`).
//! 2022-blake3 derives session subkeys with BLAKE3's key-derivation mode
//! (2022 edition spec §2.2) — an ASCII context, so the `blake3` crate is used
//! directly (unlike VLESS's binary-context derive-key in
//! `protocol/vless/encryption/b3.rs`).
//!
//! Backends: REALITY auth-key derivation uses ring (workspace standard); the
//! Shadowsocks helpers above use RustCrypto `hkdf`/`sha1`, `md-5`, and the
//! `blake3` crate.

use hkdf::Hkdf;
use sha1::Sha1;
use zeroize::Zeroizing;

/// `HKDF-SHA1(psk, salt, info)` → `out` (classic Shadowsocks AEAD subkey).
pub fn hkdf_sha1(psk: &[u8], salt: &[u8], info: &[u8], out: &mut [u8]) {
    Hkdf::<Sha1>::new(Some(salt), psk)
        .expand(info, out)
        .expect("HKDF-SHA1 output length is bounded by callers (< 255*20)");
}

/// The 2022 session-subkey context (2022 edition spec §2.2) — the ONE owner
/// of this string; `protocol/ss/method.rs` imports it.
pub(crate) const SS2022_SUBKEY_CONTEXT: &str = "shadowsocks 2022 session subkey";

/// `blake3::derive_key(context, material)` — the 2022 session-subkey KDF.
#[must_use]
pub fn blake3_derive_key(context: &str, material: &[u8]) -> [u8; 32] {
    blake3::derive_key(context, material)
}

/// OpenSSL `EVP_BytesToKey` (MD5, no salt, count 1): `d_i = MD5(d_{i-1} || password)`,
/// `d_0 = MD5(password)`; the key is the concatenation truncated to `key_len`.
///
/// Shadowsocks applies this to the password **unconditionally** — a 32-char
/// password for aes-256-gcm is still KDF'd (xray `passwordToCipherKey`,
/// mihomo `PickCipher`'s `Kdf(password, …)`, ss-rust `make_derived_key`).
#[must_use]
pub fn evp_bytes_to_key_md5(password: &[u8], key_len: usize) -> Zeroizing<Vec<u8>> {
    use md5::{Digest as _, Md5};
    let mut key = Zeroizing::new(Vec::with_capacity(key_len + 16));
    let mut prev: Option<Zeroizing<[u8; 16]>> = None;
    while key.len() < key_len {
        let mut hasher = Md5::new();
        if let Some(prev) = &prev {
            hasher.update(prev.as_slice());
        }
        hasher.update(password);
        let digest = Zeroizing::new(<[u8; 16]>::from(hasher.finalize()));
        key.extend_from_slice(digest.as_slice());
        prev = Some(digest);
    }
    key.truncate(key_len);
    key
}
```

No vector generation step is needed: the HKDF-SHA1 and `EVP_BytesToKey` values above were computed with an independent implementation (Python `hmac`/`hashlib`) and are pinned verbatim; the BLAKE3 helper is cross-checked against the in-tree hand-rolled implementation in `b3.rs` (the last test in Step 2), and tier-3 interop with both cores is the wire-level authority.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p xray-tui-native --lib crypto::kdf`
Expected: PASS (4 tests).

- [ ] **Step 6: Lockfile + hygiene gates**

```bash
cargo tree -p xray-tui-native -e features | grep -E "zeroize|sha1|hkdf|blake3" | head -20
cargo hakari generate
# append sha1/hkdf/blake3 to the xray-tui-hakari machete ignore list per AGENTS.md
cargo machete --with-metadata --skip-target-dir
```
Expected: `sha1` and `blake3` show `zeroize` in their feature list; hakari regenerates without unexpected churn.

- [ ] **Step 7: Commit**

```bash
git add crates/xray-tui-native/Cargo.toml crates/xray-tui-native/src/crypto/kdf.rs Cargo.lock crates/xray-tui-hakari/Cargo.toml
git commit -m "feat(native): shadowsocks KDF primitives (HKDF-SHA1, blake3, EVP_BytesToKey)"
```

---

### Task 1: AEAD codec (`crypto/aead.rs`)

**Files:**
- Modify: `crates/xray-tui-native/src/crypto/aead.rs` (replace stub body)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub enum SsAead { Aes128Gcm, Aes192Gcm, Aes256Gcm, ChaCha20Poly1305, XChaCha20Poly1305 }` with `const fn key_len(self) -> usize`, `const fn salt_len(self) -> usize`, `const fn nonce_len(self) -> usize`, `const fn tag_len(self) -> usize`,
  - `pub fn seal(&self, key: &[u8], nonce: &[u8], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, NativeError>`,
  - `pub fn open(&self, key: &[u8], nonce: &[u8], aad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, NativeError>`,
  - `pub struct NonceCounter { .. }` with `fn new(len: usize) -> Self`, `fn next(&mut self) -> &[u8]` (little-endian increment over the whole buffer).

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_match_the_wire() {
        assert_eq!(SsAead::Aes128Gcm.key_len(), 16);
        assert_eq!(SsAead::Aes192Gcm.key_len(), 24);
        assert_eq!(SsAead::Aes256Gcm.key_len(), 32);
        assert_eq!(SsAead::ChaCha20Poly1305.key_len(), 32);
        assert_eq!(SsAead::XChaCha20Poly1305.key_len(), 32);
        // salt = max(16, key_len); nonce = 12 (XChaCha 24); tag = 16.
        assert_eq!(SsAead::Aes128Gcm.salt_len(), 16);
        assert_eq!(SsAead::Aes192Gcm.salt_len(), 24);
        assert_eq!(SsAead::XChaCha20Poly1305.salt_len(), 32);
        assert_eq!(SsAead::XChaCha20Poly1305.nonce_len(), 24);
        assert_eq!(SsAead::ChaCha20Poly1305.nonce_len(), 12);
        for aead in [
            SsAead::Aes128Gcm,
            SsAead::Aes192Gcm,
            SsAead::Aes256Gcm,
            SsAead::ChaCha20Poly1305,
            SsAead::XChaCha20Poly1305,
        ] {
            assert_eq!(aead.tag_len(), 16);
        }
    }

    #[test]
    fn seal_open_round_trip_every_cipher() {
        for aead in [
            SsAead::Aes128Gcm,
            SsAead::Aes192Gcm,
            SsAead::Aes256Gcm,
            SsAead::ChaCha20Poly1305,
            SsAead::XChaCha20Poly1305,
        ] {
            let key = vec![0x42u8; aead.key_len()];
            let nonce = vec![0x07u8; aead.nonce_len()];
            let ct = aead.seal(&key, &nonce, b"aad", b"hello").unwrap();
            assert_eq!(ct.len(), 5 + aead.tag_len());
            assert_eq!(aead.open(&key, &nonce, b"aad", &ct).unwrap(), b"hello");
        }
    }

    #[test]
    fn tamper_and_wrong_aad_are_rejected() {
        let aead = SsAead::Aes256Gcm;
        let key = vec![0u8; 32];
        let nonce = vec![0u8; 12];
        // Wrong AAD against an INTACT ciphertext (isolates AAD authentication
        // from tamper detection).
        let ct = aead.seal(&key, &nonce, b"", b"payload").unwrap();
        assert!(aead.open(&key, &nonce, b"other", &ct).is_err());
        // Tampered ciphertext with the right AAD.
        let mut ct = aead.seal(&key, &nonce, b"", b"payload").unwrap();
        ct[0] ^= 1;
        assert!(aead.open(&key, &nonce, b"", &ct).is_err());
    }

    /// The classic/2022 TCP nonce is a little-endian counter over the whole
    /// nonce buffer. Guards the increment width (XChaCha's 24 bytes) and the
    /// carry chain (a 1-byte counter wraps 0xff → 0x00 without panicking).
    #[test]
    fn nonce_counter_is_little_endian_and_wraps() {
        let mut c = NonceCounter::new(12);
        assert_eq!(c.next(), &[0u8; 12]);
        assert_eq!(c.next(), &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // Width: the XChaCha path hands out 24 bytes.
        let mut wide = NonceCounter::new(24);
        assert_eq!(wide.next().len(), 24);
        let mut expected = [0u8; 24];
        expected[0] = 1;
        assert_eq!(wide.next(), &expected[..]);
        // Carry chain + wrap: byte 0 carries into byte 1, and the top byte's
        // carry is dropped rather than panicking.
        let mut c = NonceCounter::new(2);
        for _ in 0..256 {
            let _ = c.next(); // `next` is #[must_use]; the value is discarded on purpose
        }
        assert_eq!(c.next(), &[0x00, 0x01]); // 256 as u16 LE
        let mut c = NonceCounter::new(1);
        for _ in 0..256 {
            let _ = c.next();
        }
        assert_eq!(c.next(), &[0x00]); // 256 truncated to one byte
        let mut c = NonceCounter::new(0);
        assert_eq!(c.next(), &[] as &[u8]);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib crypto::aead`
Expected: FAIL — `cannot find type SsAead`.

- [ ] **Step 3: Implement**

```rust
//! AEAD body ciphers: AES-128/192/256-GCM and ChaCha20/XChaCha20-Poly1305.
//!
//! Consumers: `protocol/ss` (classic AEAD + 2022-blake3, Tasks 3/4/6). The
//! VMess record codec (`protocol/vmess/stream.rs`) drives the same RustCrypto
//! primitives directly for its own record framing.
//! Backend: RustCrypto (`aes-gcm`, `chacha20poly1305`) — not ring: both are
//! already in the tree with the `zeroize` feature, and their explicit-nonce
//! APIs are what the Shadowsocks counters need. References: RFC 8439,
//! NIST SP 800-38D, draft-irtf-cfrg-xchacha.

use aes_gcm::aead::{Aead as _, Payload};
use aes_gcm::{Aes128Gcm, Aes192Gcm, Aes256Gcm, KeyInit as _};
use chacha20poly1305::{ChaCha20Poly1305, XChaCha20Poly1305};
use zeroize::Zeroizing;

use crate::error::NativeError;
use xray_tui_proto::proto_spec::ProtocolKind;

/// The Shadowsocks cipher set native implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsAead {
    Aes128Gcm,
    Aes192Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
    XChaCha20Poly1305,
}

impl SsAead {
    /// AEAD key length (16/24/32) — the method's "key size".
    #[must_use]
    pub const fn key_len(self) -> usize {
        match self {
            Self::Aes128Gcm => 16,
            Self::Aes192Gcm => 24,
            Self::Aes256Gcm | Self::ChaCha20Poly1305 | Self::XChaCha20Poly1305 => 32,
        }
    }

    /// Stream/packet salt length: `max(16, key_len)` (mihomo `SaltSize`).
    #[must_use]
    pub const fn salt_len(self) -> usize {
        if self.key_len() > 16 { self.key_len() } else { 16 }
    }

    #[must_use]
    pub const fn nonce_len(self) -> usize {
        match self {
            Self::XChaCha20Poly1305 => 24,
            _ => 12,
        }
    }

    #[must_use]
    pub const fn tag_len(self) -> usize {
        16
    }

    pub fn seal(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, NativeError> {
        match self {
            Self::Aes128Gcm => seal_gcm::<Aes128Gcm>(key, nonce, aad, plaintext),
            Self::Aes192Gcm => seal_gcm::<Aes192Gcm>(key, nonce, aad, plaintext),
            Self::Aes256Gcm => seal_gcm::<Aes256Gcm>(key, nonce, aad, plaintext),
            Self::ChaCha20Poly1305 => seal_chacha::<ChaCha20Poly1305>(key, nonce, aad, plaintext),
            Self::XChaCha20Poly1305 => seal_chacha::<XChaCha20Poly1305>(key, nonce, aad, plaintext),
        }
    }

    pub fn open(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, NativeError> {
        match self {
            Self::Aes128Gcm => open_gcm::<Aes128Gcm>(key, nonce, aad, ciphertext),
            Self::Aes192Gcm => open_gcm::<Aes192Gcm>(key, nonce, aad, ciphertext),
            Self::Aes256Gcm => open_gcm::<Aes256Gcm>(key, nonce, aad, ciphertext),
            Self::ChaCha20Poly1305 => open_chacha::<ChaCha20Poly1305>(key, nonce, aad, ciphertext),
            Self::XChaCha20Poly1305 => open_chacha::<XChaCha20Poly1305>(key, nonce, aad, ciphertext),
        }
    }
}

fn cipher_error(detail: &str) -> NativeError {
    NativeError::Protocol {
        kind: ProtocolKind::Shadowsocks,
        detail: detail.to_owned(),
    }
}

fn seal_gcm<C>(key: &[u8], nonce: &[u8], aad: &[u8], pt: &[u8]) -> Result<Vec<u8>, NativeError>
where
    C: aes_gcm::aead::Aead + KeyInit,
{
    C::new_from_slice(key)
        .map_err(|_| cipher_error("bad AEAD key length"))?
        .encrypt(nonce.into(), Payload { msg: pt, aad })
        .map_err(|_| cipher_error("AEAD seal failed"))
}

fn open_gcm<C>(key: &[u8], nonce: &[u8], aad: &[u8], ct: &[u8]) -> Result<Vec<u8>, NativeError>
where
    C: aes_gcm::aead::Aead + KeyInit,
{
    C::new_from_slice(key)
        .map_err(|_| cipher_error("bad AEAD key length"))?
        .decrypt(nonce.into(), Payload { msg: ct, aad })
        .map_err(|_| cipher_error("AEAD open failed (tampered or wrong key)"))
}

fn seal_chacha<C>(key: &[u8], nonce: &[u8], aad: &[u8], pt: &[u8]) -> Result<Vec<u8>, NativeError>
where
    C: chacha20poly1305::aead::Aead + chacha20poly1305::KeyInit,
{
    C::new_from_slice(key)
        .map_err(|_| cipher_error("bad AEAD key length"))?
        .encrypt(nonce.into(), Payload { msg: pt, aad })
        .map_err(|_| cipher_error("AEAD seal failed"))
}

fn open_chacha<C>(key: &[u8], nonce: &[u8], aad: &[u8], ct: &[u8]) -> Result<Vec<u8>, NativeError>
where
    C: chacha20poly1305::aead::Aead + chacha20poly1305::KeyInit,
{
    C::new_from_slice(key)
        .map_err(|_| cipher_error("bad AEAD key length"))?
        .decrypt(nonce.into(), Payload { msg: ct, aad })
        .map_err(|_| cipher_error("AEAD open failed (tampered or wrong key)"))
}

/// Little-endian counter over a fixed-width nonce, incremented after every
/// seal/open — the classic-AEAD and 2022 TCP nonce.
pub struct NonceCounter {
    buf: Zeroizing<[u8; 24]>,
    len: usize,
}

impl NonceCounter {
    #[must_use]
    pub fn new(len: usize) -> Self {
        // `assert!`, not `debug_assert!`: an over-wide nonce would otherwise
        // panic on the first hand-out, far from the constructor that accepted
        // it — and only in release builds.
        assert!(len <= 24, "NonceCounter width must be <= 24 bytes, got {len}");
        Self {
            buf: Zeroizing::new([0u8; 24]),
            len,
        }
    }

    /// The nonce for the next operation, incrementing the counter.
    pub fn next(&mut self) -> &[u8] {
        let out = &self.buf[..self.len];
        // Increment AFTER handing out `out` on the following call: copy-free
        // readers take a snapshot, so bump at the start of the next call.
        let snapshot = out.to_vec();
        for byte in self.buf[..self.len].iter_mut() {
            let (v, carry) = byte.overflowing_add(1);
            *byte = v;
            if !carry {
                break;
            }
        }
        // `out` aliases `self.buf`; return the pre-increment snapshot.
        self.buf[..self.len].copy_from_slice(&snapshot);
        &self.buf[..self.len]
    }
}
```

`NonceCounter::next` above is deliberately trivial-but-correct; simplify to a plain `copy_from_slice` into a caller buffer if the aliasing dance reads badly — the *observable* contract the tests pin is: first call all-zero, second call `[1, 0, …]`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib crypto::aead`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/crypto/aead.rs
git commit -m "feat(native): shadowsocks AEAD codec (5 ciphers + nonce counter)"
```

---

### Task 2: Method table + password→key (`protocol/ss/method.rs`)

**Files:**
- Create: `crates/xray-tui-native/src/protocol/ss/method.rs`
- Modify: `crates/xray-tui-native/src/protocol/ss/mod.rs` (`pub mod method;` + module doc)

**Interfaces:**
- Consumes: `crypto::aead::SsAead`, `crypto::kdf::{evp_bytes_to_key_md5, hkdf_sha1, blake3_derive_key}`.
- Produces:
  - `pub enum SsFamily { Classic, Blake3_2022 }`,
  - `pub struct SsMethod { pub aead: SsAead, pub family: SsFamily }` with `pub fn from_method(method: &str) -> Option<SsMethod>`, `pub const fn key_len(self) -> usize` (delegates to `aead`),
  - `pub fn stream_subkey(method: SsMethod, key: &[u8], salt: &[u8]) -> Zeroizing<Vec<u8>>` (classic: HKDF-SHA1; 2022: blake3 `derive_key` over `key ‖ salt`),
  - `pub fn password_key(method: SsMethod, password: &str) -> Result<Zeroizing<Vec<u8>>, NativeError>` — classic: MD5 `EVP_BytesToKey`; 2022: base64-decode, must be exactly `key_len` bytes.

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::aead::SsAead;

    #[test]
    fn method_table_covers_the_native_set_including_aliases() {
        let cases: &[(&str, SsAead, SsFamily)] = &[
            ("aes-128-gcm", SsAead::Aes128Gcm, SsFamily::Classic),
            ("aead_aes_128_gcm", SsAead::Aes128Gcm, SsFamily::Classic),
            ("aes-192-gcm", SsAead::Aes192Gcm, SsFamily::Classic),
            ("aes-256-gcm", SsAead::Aes256Gcm, SsFamily::Classic),
            ("aead_aes_256_gcm", SsAead::Aes256Gcm, SsFamily::Classic),
            ("chacha20-ietf-poly1305", SsAead::ChaCha20Poly1305, SsFamily::Classic),
            ("chacha20-poly1305", SsAead::ChaCha20Poly1305, SsFamily::Classic),
            ("aead_chacha20_poly1305", SsAead::ChaCha20Poly1305, SsFamily::Classic),
            ("xchacha20-ietf-poly1305", SsAead::XChaCha20Poly1305, SsFamily::Classic),
            ("xchacha20-poly1305", SsAead::XChaCha20Poly1305, SsFamily::Classic),
            ("aead_xchacha20_poly1305", SsAead::XChaCha20Poly1305, SsFamily::Classic),
            ("aead_aes_192_gcm", SsAead::Aes192Gcm, SsFamily::Classic),
            ("2022-blake3-aes-128-gcm", SsAead::Aes128Gcm, SsFamily::Blake3_2022),
            ("2022-blake3-aes-256-gcm", SsAead::Aes256Gcm, SsFamily::Blake3_2022),
            ("2022-blake3-chacha20-poly1305", SsAead::ChaCha20Poly1305, SsFamily::Blake3_2022),
        ];
        for (name, aead, family) in cases {
            let m = SsMethod::from_method(name).unwrap_or_else(|| panic!("{name} resolves"));
            assert_eq!(m.aead, *aead, "{name}");
            assert_eq!(m.family, *family, "{name}");
        }
        // Legacy stream ciphers and unknown names stay unresolved (defer to sing-box).
        for name in ["aes-256-cfb", "rc4-md5", "chacha20-ietf", "none", "bogus"] {
            assert!(SsMethod::from_method(name).is_none(), "{name}");
        }
    }

    #[test]
    fn method_names_are_case_insensitive() {
        assert!(SsMethod::from_method("AES-256-GCM").is_some());
    }

    #[test]
    fn classic_password_key_is_md5_evp_bytes_to_key() {
        let m = SsMethod::from_method("aes-128-gcm").unwrap();
        let key = password_key(m, "password").unwrap();
        assert_eq!(key.len(), 16);
        assert_eq!(
            &*key,
            &hex("5f4dcc3b5aa765d61d8327deb882cf99"),
            "MD5(password) is the first EVP_BytesToKey block"
        );
    }

    /// A 32-char password for aes-256-gcm is STILL KDF'd (no raw shortcut).
    #[test]
    fn classic_password_of_key_length_is_still_kdfed() {
        let m = SsMethod::from_method("aes-256-gcm").unwrap();
        let pwd = "0123456789abcdef0123456789abcdef";
        assert_eq!(password_key(m, pwd).unwrap().len(), 32);
        assert_ne!(&*password_key(m, pwd).unwrap(), pwd.as_bytes());
    }

    #[test]
    fn c2022_password_is_base64_psk_of_exact_key_length() {
        let m = SsMethod::from_method("2022-blake3-aes-128-gcm").unwrap();
        // 16 zero bytes, base64.
        let key = password_key(m, "AAAAAAAAAAAAAAAAAAAAAA==").unwrap();
        assert_eq!(&*key, &[0u8; 16]);
        assert!(matches!(
            password_key(m, "c2hvcnQ="),
            Err(NativeError::Config(_))
        ));
        assert!(matches!(
            password_key(m, "not base64!"),
            Err(NativeError::Config(_))
        ));
    }

    /// Classic subkey = HKDF-SHA1; 2022 subkey = blake3 derive_key over key ‖ salt.
    #[test]
    fn stream_subkeys_differ_per_family_and_track_the_salt() {
        let classic = SsMethod::from_method("aes-128-gcm").unwrap();
        let a = stream_subkey(classic, b"password", &[1u8; 16]);
        let b = stream_subkey(classic, b"password", &[2u8; 16]);
        assert_eq!(a.len(), 16);
        assert_ne!(&*a, &*b);

        let s2022 = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
        let key = [9u8; 32];
        let c = stream_subkey(s2022, &key, &[3u8; 32]);
        let d = stream_subkey(s2022, &key, &[4u8; 32]);
        assert_eq!(c.len(), 32);
        assert_ne!(&*c, &*d);
    }

    /// The `key ‖ salt` material order is pinned against the hand-rolled
    /// BLAKE3 (an implementation independent of the `blake3` crate the helper
    /// delegates to). Requires exposing it crate-wide: in
    /// `protocol/vless/encryption/mod.rs` change `mod b3;` to
    /// `pub(crate) mod b3;` and in `b3.rs` change
    /// `pub(super) fn derive_key_bytes` to `pub(crate) fn derive_key_bytes`.
    #[test]
    fn blake3_subkey_material_order_is_key_then_salt() {
        let s2022 = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
        let key = [0x11u8; 32];
        let salt = [0x22u8; 32];
        let material = [key.as_slice(), salt.as_slice()].concat();
        assert_eq!(
            &*stream_subkey(s2022, &key, &salt),
            &crate::protocol::vless::encryption::b3::derive_key_bytes(
                b"shadowsocks 2022 session subkey",
                &material
            )[..]
        );
        // …and the subkey advances with the salt (per-connection freshness).
        assert_ne!(&*stream_subkey(s2022, &key, &salt), &*stream_subkey(s2022, &key, &[0x23u8; 32]));
    }

    /// Classic KAT through the PUBLIC API: pins the `key`/`salt` argument
    /// positions (`hkdf_sha1(psk, salt, b"ss-subkey")`) — the primitive's own
    /// vector cannot catch a transposition at this call site, and the codec
    /// round-trips use `stream_subkey` on both ends, so a transposition would
    /// otherwise only surface at tier-3 interop. Vector = kdf.rs's independent
    /// HKDF-SHA1("password", 0x11×16, "ss-subkey").
    #[test]
    fn classic_subkey_is_hkdf_sha1_of_key_and_salt() {
        let m = SsMethod::from_method("aes-128-gcm").unwrap();
        assert_eq!(
            &*stream_subkey(m, b"password", &[0x11u8; 16]),
            &hex("8e2b1a6111239229400b5dd612771931")
        );
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib protocol::ss::method`
Expected: FAIL — module/type not found.

- [ ] **Step 3: Implement**

```rust
//! Shadowsocks method table + key derivation.
//!
//! Native covers the AEAD set (2017 edition) and the 2022-blake3 family; the
//! legacy stream ciphers (`aes-*-cfb/ctr`, `rc4-md5`, `chacha20-ietf`,
//! `xchacha20`, `none`) stay on sing-box and are deliberately absent here.

use base64::Engine as _;
use xray_tui_proto::proto_spec::ProtocolKind;
use zeroize::Zeroizing;

use crate::crypto::aead::SsAead;
use crate::crypto::kdf::{SS2022_SUBKEY_CONTEXT, blake3_derive_key, evp_bytes_to_key_md5, hkdf_sha1};
use crate::error::NativeError;

/// The KDF family a method belongs to — it selects the subkey derivation and
/// the wire codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsFamily {
    /// 2017 AEAD: HKDF-SHA1 subkey, `[salt][2B len][tag][ct][tag]` chunks.
    Classic,
    /// 2022-blake3: BLAKE3 subkey, standalone header chunks, `[salt][header]…`.
    Blake3_2022,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsMethod {
    pub aead: SsAead,
    pub family: SsFamily,
}

impl SsMethod {
    /// Resolve a method string (case-insensitive), aliases included.
    #[must_use]
    pub fn from_method(method: &str) -> Option<Self> {
        let method = method.trim().to_ascii_lowercase();
        let (aead, family) = match method.as_str() {
            "aes-128-gcm" | "aead_aes_128_gcm" => (SsAead::Aes128Gcm, SsFamily::Classic),
            "aes-192-gcm" | "aead_aes_192_gcm" => (SsAead::Aes192Gcm, SsFamily::Classic),
            "aes-256-gcm" | "aead_aes_256_gcm" => (SsAead::Aes256Gcm, SsFamily::Classic),
            "chacha20-poly1305" | "chacha20-ietf-poly1305" | "aead_chacha20_poly1305" => {
                (SsAead::ChaCha20Poly1305, SsFamily::Classic)
            }
            "xchacha20-poly1305" | "xchacha20-ietf-poly1305" | "aead_xchacha20_poly1305" => {
                (SsAead::XChaCha20Poly1305, SsFamily::Classic)
            }
            "2022-blake3-aes-128-gcm" => (SsAead::Aes128Gcm, SsFamily::Blake3_2022),
            "2022-blake3-aes-256-gcm" => (SsAead::Aes256Gcm, SsFamily::Blake3_2022),
            "2022-blake3-chacha20-poly1305" => (SsAead::ChaCha20Poly1305, SsFamily::Blake3_2022),
            _ => return None,
        };
        Some(Self { aead, family })
    }

    #[must_use]
    pub const fn key_len(self) -> usize {
        self.aead.key_len()
    }

    #[must_use]
    pub const fn kind(self) -> ProtocolKind {
        match self.family {
            SsFamily::Classic => ProtocolKind::Shadowsocks,
            SsFamily::Blake3_2022 => ProtocolKind::Shadowsocks2022,
        }
    }
}

/// Per-stream/-session subkey.
///
/// Classic: `HKDF-SHA1(key, salt, "ss-subkey")`, output = key_len.
/// 2022: `blake3::derive_key("shadowsocks 2022 session subkey", key ‖ salt)`.
#[must_use]
pub fn stream_subkey(method: SsMethod, key: &[u8], salt: &[u8]) -> Zeroizing<Vec<u8>> {
    match method.family {
        SsFamily::Classic => {
            let mut out = Zeroizing::new(vec![0u8; method.key_len()]);
            hkdf_sha1(key, salt, b"ss-subkey", &mut out);
            out
        }
        SsFamily::Blake3_2022 => {
            let mut material = Zeroizing::new(Vec::with_capacity(key.len() + salt.len()));
            material.extend_from_slice(key);
            material.extend_from_slice(salt);
            // BLAKE3 derive-key is an XOF: the session key is only the first
            // `key_len` bytes (16 for 2022-blake3-aes-128-gcm, 32 otherwise) —
            // v2ray-core `kdf_blake3.go` fills a caller-sized outKey, shoes
            // fills `session_key_len` from `finalize_xof`. The full 32 bytes
            // would fail the 16-byte AEAD's key check.
            let root = Zeroizing::new(blake3_derive_key(SS2022_SUBKEY_CONTEXT, &material));
            Zeroizing::new(root[..method.key_len()].to_vec())
        }
    }
}

/// Master key from the profile's password.
///
/// Classic: OpenSSL `EVP_BytesToKey` (MD5) — unconditional.
/// 2022: the password IS a base64 PSK; wrong length is a config error.
pub fn password_key(method: SsMethod, password: &str) -> Result<Zeroizing<Vec<u8>>, NativeError> {
    match method.family {
        SsFamily::Classic => Ok(evp_bytes_to_key_md5(password.as_bytes(), method.key_len())),
        SsFamily::Blake3_2022 => {
            // `Zeroizing` BEFORE the length check: a wrong-length PSK is
            // still real key material and must not be dropped unwiped.
            let decoded = Zeroizing::new(
                base64::engine::general_purpose::STANDARD
                    .decode(password.trim())
                    .map_err(|e| {
                        NativeError::Config(format!(
                            "shadowsocks-2022 password is not base64: {e}"
                        ))
                    })?,
            );
            if decoded.len() != method.key_len() {
                return Err(NativeError::Config(format!(
                    "shadowsocks-2022 key is {} bytes, method needs {}",
                    decoded.len(),
                    method.key_len()
                )));
            }
            Ok(decoded)
        }
    }
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib protocol::ss::method`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/ss/
git commit -m "feat(native): shadowsocks method table + key derivation"
```

---

### Task 3: Classic AEAD TCP codec (`protocol/ss/stream.rs`)

**Files:**
- Create: `crates/xray-tui-native/src/protocol/ss/stream.rs`
- Modify: `crates/xray-tui-native/src/protocol/ss/mod.rs` (`pub mod stream;`)

**Interfaces:**
- Consumes: `SsMethod`, `password_key`, `stream_subkey`, `SsAead`, `NonceCounter`, `crate::addr::encode_addr_port_last`, `crate::rand::fill_nonsecret`.
- Produces: `pub struct SsStream<S>`, `pub async fn connect(ctx: &LinkContext, stream: BoxStream, cfg: &SsConfig, method: SsMethod) -> Result<BoxStream, NativeError>`.

**Address family:** Shadowsocks carries the SOCKS5 address (`ATYP | addr | port BE2`, ATYP `0x01`/`0x03`/`0x04`) — the **port-last** family, i.e. `addr::encode_addr_port_last` / `decode_addr_port_last`, NOT `encode_addr` (VLESS/VMess port-first, `ADDR_TYPE_*` 1/2/3).

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    /// The first request chunk on the wire is
    /// `[salt][2B len + tag][ct(addr || data) + tag]` — two seals, counter +2.
    #[tokio::test]
    async fn first_chunk_is_length_seal_then_payload_seal() {
        let method = SsMethod::from_method("aes-128-gcm").unwrap();
        let key = [0x11u8; 16];
        let (mut client, mut server) = duplex(64 * 1024);
        let target = TargetAddr::new(Host::Ip(IpAddr::from([127, 0, 0, 1])), 8080);
        let task = tokio::spawn(async move {
            connect_chunked(method, &key, target, &mut client).await.unwrap();
        });
        let mut wire = vec![0u8; 16 + 2 + 16 + 7 + 16];
        server.read_exact(&mut wire).await.unwrap();
        // salt is the first 16 bytes and is NOT a constant.
        assert_ne!(&wire[..16], &[0u8; 16]);
        let sub = stream_subkey(method, &key, &wire[..16]);
        let mut counter = NonceCounter::new(12);
        let len_pt = method.aead.open(&sub, counter.next(), b"", &wire[16..34]).unwrap();
        assert_eq!(len_pt, vec![0x00, 0x07]);
        let payload = method.aead.open(&sub, counter.next(), b"", &wire[34..57]).unwrap();
        assert_eq!(payload, encode_addr_port_last(&target).unwrap());
        task.await.unwrap();
    }

    /// A 0x3FFF-byte write is one length seal + one payload seal spanning two
    /// chunks' worth of stream, and a >0x3FFF write splits into two chunks.
    #[tokio::test]
    async fn writes_are_split_at_the_3fff_cap() {
        let method = SsMethod::from_method("chacha20-ietf-poly1305").unwrap();
        let (client, server) = tokio::io::duplex(1 << 20);
        let key = [0x22u8; 32];
        let salt = [0x33u8; 32];
        let mut s = SsStream::new(client, method, Zeroizing::new(key.to_vec()), salt, None);
        let payload = vec![0xABu8; 0x4000 + 5];
        let expected = payload.clone();
        let reader = tokio::spawn(async move { read_all_chunks(server, method, key, salt).await });
        s.write_all(&payload).await.unwrap();
        drop(s); // close: the reader stops on EOF
        assert_eq!(reader.await.unwrap(), expected);
    }
}
```

The two helpers used above (`connect_chunked`, `read_all_chunks`) are test-local: `connect_chunked` = the address-first write path, `read_all_chunks` = a decrypt loop mirroring the codec. Implement them in the same test module from the same primitives (`stream_subkey` + `NonceCounter` + `aead.open`), so they *cannot* share a bug with the reader under test.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib protocol::ss::stream`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

```rust
//! Classic Shadowsocks AEAD (2017) TCP codec — client side.
//!
//! Wire (shadowsocks-rust `tcprelay/aead.rs`, mihomo `shadowaead/stream.go`):
//!
//! ```text
//! request : [salt][ [2B BE len][16B tag] [ct(payload)][16B tag] ]*
//! response: [salt][ [2B BE len][16B tag] [ct(payload)][16B tag] ]*
//! ```
//!
//! Each chunk is TWO AEAD seals with two consecutive counter nonces (length,
//! then payload); `len ≤ 0x3FFF`. Request and response have independent salt,
//! subkey and counter — the client writes its salt first and reads the
//! server's before the first response chunk. The client's first payload is
//! the SOCKS-style target address (`addr::encode_addr`).

/// Classic AEAD's payload cap (`MAX_PACKET_SIZE`).
pub const MAX_CHUNK: usize = 0x3FFF;

/// Read/write state of one direction of a classic AEAD stream.
struct Half {
    subkey: Zeroizing<Vec<u8>>,
    counter: NonceCounter,
}

pub struct SsStream<S> {
    inner: S,
    aead: SsAead,
    read: Option<Half>,
    write: Half,
    /// Encrypted bytes waiting to go out (length+payload seal are batched
    /// into one `poll_write` where possible).
    out: Vec<u8>,
    /// Ciphertext bytes read but not yet consumed by `poll_read`.
    inbound: Vec<u8>,
    inbound_pos: usize,
    /// Remaining plaintext of the chunk currently being handed to the caller.
    pending: Vec<u8>,
    pending_pos: usize,
    salt_len: usize,
    eof: bool,
}
```

Implement:

```rust
impl<S> SsStream<S> {
    fn new(inner: S, method: SsMethod, key: Zeroizing<Vec<u8>>, salt: &[u8], first: Option<Vec<u8>>) -> Self {
        let write = Half { subkey: stream_subkey(method, &key, salt), counter: NonceCounter::new(method.aead.nonce_len()) };
        let mut s = Self { inner, aead: method.aead, read: None, write, out: Vec::new(), inbound: Vec::new(), inbound_pos: 0, pending: Vec::new(), pending_pos: 0, salt_len: method.aead.salt_len(), eof: false };
        s.out.extend_from_slice(salt);           // the salt leads the stream
        if let Some(first) = first { s.push_chunks(&first); }
        s
    }

    /// Seal `plaintext` into `self.out` as `[2B len + tag][ct + tag]` per
    /// ≤ `MAX_CHUNK` slice — TWO seals, TWO counter nonces, per chunk.
    fn push_chunks(&mut self, plaintext: &[u8]) {
        for slice in plaintext.chunks(MAX_CHUNK) {
            let len_nonce = self.write.counter.next().to_vec();
            self.out.extend_from_slice(
                &self.aead.seal(&self.write.subkey, &len_nonce, b"", &(slice.len() as u16).to_be_bytes()).expect("seal"),
            );
            let payload_nonce = self.write.counter.next().to_vec();
            self.out.extend_from_slice(
                &self.aead.seal(&self.write.subkey, &payload_nonce, b"", slice).expect("seal"),
            );
        }
    }
}
```

(The `.expect("seal")` calls are inside the codec's own state machine — an AEAD failure there is a programmer error; the *wire* failures that reach the caller are the `open` errors in `read_chunk`.)

```rust
/// Pull bytes out of `inner` until one decrypted chunk is available.
/// Cancel-safe: the partially-read chunk lives in `inbound`, not in the future.
async fn fill_chunk<S: AsyncRead + Unpin>(&mut self) -> std::io::Result<()> { /* read_exact len+tag, then len+tag */ }

/// First read on the stream: the server's salt, then the subkey.
async fn init_reader<S: AsyncRead + Unpin>(&mut self) -> std::io::Result<()> { /* salt_len bytes → Half */ }

/// `[2B len + tag][ct + tag]` → plaintext, advancing the counter twice.
fn read_chunk(&mut self, wire: &[u8], out: &mut Vec<u8>) -> Result<usize, NativeError> {
    let tag = self.aead.tag_len();
    if wire.len() < 2 + tag {
        return Err(chunk_error("short length chunk"));
    }
    let len = self.aead.open(&self.read.as_ref().unwrap().subkey, self.read.as_mut().unwrap().counter.next(), b"", &wire[..2 + tag])?;
    let n = usize::from(u16::from_be_bytes([len[0], len[1]]));
    if n == 0 {
        return Err(chunk_error("zero-length chunk"));
    }
    let body = &wire[2 + tag..2 + tag + n + tag];
    out.extend_from_slice(&self.aead.open(&self.read.as_ref().unwrap().subkey, self.read.as_mut().unwrap().counter.next(), b"", body)?);
    Ok(2 + tag + n + tag)
}
```

Then `impl AsyncWrite for SsStream<S>` (buffer plaintext, flush through `push_chunks` at ≥ `MAX_CHUNK`, `poll_flush`/`poll_shutdown` drain `out`) and `impl AsyncRead` (`init_reader` once, then `fill_chunk` + `read_chunk` into `pending`, copy out). Keep both `!Send`-free by holding all state in the struct — the futures must not borrow across `await`s in a way that trips `clippy::future_not_send`.

`connect`:

```rust
pub async fn connect(ctx: &LinkContext, stream: BoxStream, cfg: &SsConfig, method: SsMethod) -> Result<BoxStream, NativeError> {
    let key = password_key(method, &cfg.password)?;
    let mut salt = Zeroizing::new(vec![0u8; method.aead.salt_len()]);
    crate::rand::fill_nonsecret(&mut salt);
    let first = encode_addr_port_last(&ctx.params.target)?;
    let mut s = SsStream::new(stream, method, key, &salt, Some(first));
    s.flush_out().await?;                      // salt ‖ len seal ‖ payload seal, one write
    Ok(Box::new(s))
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib protocol::ss::stream`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/ss/
git commit -m "feat(native): classic shadowsocks AEAD TCP codec"
```

---

### Task 4: 2022-blake3 TCP codec (`protocol/ss/stream2022.rs`)

**Files:**
- Create: `crates/xray-tui-native/src/protocol/ss/stream2022.rs`
- Modify: `crates/xray-tui-native/src/protocol/ss/mod.rs` (`pub mod stream2022;`)

**Interfaces:**
- Consumes: same as Task 3 plus `std::time::{SystemTime, UNIX_EPOCH}`, `SsFamily::Blake3_2022`, `crate::rand::u32_below`.
- Produces: `pub struct Ss2022Stream<S>`, `pub async fn connect(ctx: &LinkContext, stream: BoxStream, cfg: &SsConfig, method: SsMethod) -> Result<BoxStream, NativeError>`.

Header constants (2022 edition spec §3.1.3):

```rust
const HEADER_TYPE_CLIENT_STREAM: u8 = 0;
const HEADER_TYPE_SERVER_STREAM: u8 = 1;
const MAX_PAYLOAD: usize = 0xFFFF;
const MAX_PADDING: u32 = 900; // MaxPaddingLength
const TIMESTAMP_TOLERANCE_SECS: u64 = 30;
/// Request fixed header: `type(1) + timestamp(8 BE) + length(2 BE)`.
const REQUEST_FIXED_LEN: usize = 11;
```

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_fixed_header_layout_is_type_timestamp_length() {
        let h = request_fixed_header(0x0102_0304_0506_0708, 0x0011);
        assert_eq!(h.len(), REQUEST_FIXED_LEN);
        assert_eq!(h[0], HEADER_TYPE_CLIENT_STREAM);
        assert_eq!(&h[1..9], &0x0102_0304_0506_0708u64.to_be_bytes());
        assert_eq!(&h[9..11], &0x0011u16.to_be_bytes());
    }

    #[test]
    fn variable_header_carries_addr_padding_length_and_padding() {
        let target = TargetAddr::new(Host::Ip(IpAddr::from([127, 0, 0, 1])), 443);
        let v = request_variable_header(&target, &[0xAA, 0xBB]).unwrap();
        let addr = encode_addr_port_last(&target).unwrap();
        assert_eq!(&v[..addr.len()], &addr[..]);
        let pad_len = u16::from_be_bytes([v[addr.len()], v[addr.len() + 1]]) as usize;
        assert_eq!(v.len(), addr.len() + 2 + pad_len + 2);
        assert_eq!(&v[addr.len() + 2 + pad_len..], &[0xAA, 0xBB]);
    }

    /// A request header MUST carry payload or non-zero padding (spec §3.1.3).
    #[test]
    fn request_variable_header_always_pads_when_no_initial_payload() {
        let target = TargetAddr::new(Host::Ip(IpAddr::from([10, 0, 0, 1])), 80);
        let v = request_variable_header(&target, &[]).unwrap();
        let addr = encode_addr_port_last(&target).unwrap();
        let pad_len = u16::from_be_bytes([v[addr.len()], v[addr.len() + 1]]) as usize;
        assert!(pad_len >= 1 && pad_len <= 900);
    }

    /// The client MUST reject a response whose `request_salt` differs.
    #[test]
    fn response_header_request_salt_mismatch_is_rejected() {
        let method = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
        let key = [0x01u8; 32];
        let req_salt = [0xAAu8; 32];
        let resp = sealed_response_header(method, &key, &[0xBBu8; 32], 0x1234, 0);
        assert!(matches!(
            open_response_header(method, &key, &req_salt, &resp[32..], &resp[..32]),
            Err(NativeError::Protocol { kind: ProtocolKind::Shadowsocks2022, .. })
        ));
    }

    #[tokio::test]
    async fn salt_and_header_go_out_in_one_write() {
        let (client, mut server) = RecordingStream::paired();
        let method = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
        let target = TargetAddr::new(Host::Ip(IpAddr::from([127, 0, 0, 1])), 8080);
        let mut s = Ss2022Stream::new(
            Box::new(client),
            method,
            Zeroizing::new(vec![0x05u8; 32]),
            vec![0x07u8; 32], // fixed salt so the assertion is deterministic
            target,
        );
        s.write_handshake().await.unwrap();
        assert_eq!(server.write_calls(), 1, "salt + both header chunks must be one write");
    }
}
```

`RecordingStream` is a test-local `AsyncRead + AsyncWrite` that counts `poll_write` calls — 20 lines, in the same module.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib protocol::ss::stream2022`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

```rust
//! Shadowsocks 2022 (blake3) TCP codec — client side.
//!
//! Wire (2022 edition spec §3.1; shadowsocks-rust `tcprelay/aead_2022.rs`):
//!
//! ```text
//! request : [salt][seal(11B: type=0 | ts u64be | len u16be)]
//!                 [seal(varlen: ATYP|addr|port | pad_len u16be | padding | initial payload)]
//!                 [seal(2B len)][seal(payload)]*
//! response: [salt][seal(27/43B: type=1 | ts u64be | request_salt | len u16be)]
//!                 [seal(payload)][seal(2B len)][seal(payload)]*
//! ```
//!
//! Subkey = `blake3::derive_key("shadowsocks 2022 session subkey", psk ‖ salt)`;
//! nonce = 12-byte LE counter advanced by EVERY seal/open (spec §3.1.1).
//! Counter sequence — an off-by-one here breaks interop silently: request =
//! fixed header 0, variable header 1, then len 2 / payload 3, len 4 / payload 5…
//! response = fixed header 0 (it doubles as the first length chunk), payload 1,
//! len 2, payload 3… so the reader keeps ONE `Half` across the response header
//! and the chunks that follow. Payload chunks cap at 0xFFFF (`MAX_PAYLOAD`).
//! The client MUST verify the response header's `request_salt`, and MUST write
//! salt + both header chunks in ONE call.

/// Random padding length for a request with no initial payload
/// (`MaxPaddingLength = 900`; spec §3.1.3's "always send random padding").
fn random_padding_len() -> usize {
    (crate::rand::u32_below(MAX_PADDING) + 1) as usize
}

/// Request fixed-length header: `type | timestamp u64be | length u16be`.
fn request_fixed_header(timestamp: u64, len: u16) -> [u8; REQUEST_FIXED_LEN] {
    let mut h = [0u8; REQUEST_FIXED_LEN];
    h[0] = HEADER_TYPE_CLIENT_STREAM;
    h[1..9].copy_from_slice(&timestamp.to_be_bytes());
    h[9..11].copy_from_slice(&len.to_be_bytes());
    h
}

/// Request variable-length header:
/// `ATYP|addr|port | pad_len u16be | padding | initial payload`.
/// Padding is non-zero when there is no initial payload (spec §3.1.3).
fn request_variable_header(
    target: &TargetAddr,
    initial: &[u8],
) -> Result<Vec<u8>, NativeError> {
    let addr = encode_addr_port_last(target)?;
    let pad_len = if initial.is_empty() { random_padding_len() } else { 0 };
    let mut out = Vec::with_capacity(addr.len() + 2 + pad_len + initial.len());
    out.extend_from_slice(&addr);
    out.extend_from_slice(&(pad_len as u16).to_be_bytes());
    let pad_start = out.len();
    out.resize(pad_start + pad_len, 0);
    crate::rand::fill_nonsecret(&mut out[pad_start..]);
    out.extend_from_slice(initial);
    Ok(out)
}
```

Implement `Ss2022Stream<S>` with the same `Half` shape as Task 3, plus:
- `fn sealed_response_header(method, key, request_salt, len, ts) -> Vec<u8>` (response fixed header = `type=1 | ts | request_salt | len`, sealed with counter 0) — test-support only, mirroring what a real server sends.
- `fn open_response_header(method, key, request_salt, fixed_ct, salt) -> Result<u16, NativeError>` — derives the response subkey from the server salt, opens with counter 0, validates `type == 1`, the timestamp within `TIMESTAMP_TOLERANCE_SECS`, and `request_salt` equality; returns the first payload length AND the `Half` (counter now 1) so the caller continues with it.
- `pub struct Ss2022Stream<S>` with `pub fn new(inner, method, key, salt: Vec<u8>, target)` (tests pass a fixed salt; production a random one)
- `pub async fn write_handshake(&mut self)` — salt ‖ seal(fixed header) ‖ seal(variable header) in one `write_all`
- `impl AsyncWrite` — chunked writes (cap `MAX_PAYLOAD`, two seals per chunk, counter continues from 2)
- `impl AsyncRead` — server salt + response header (validated, counter 0→1), then `len`/payload chunk pairs on the same `Half`.
- `pub async fn connect(ctx, stream, cfg, method)` — `password_key` → new stream → `write_handshake` → `Box::new(stream)`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib protocol::ss::stream2022`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/ss/
git commit -m "feat(native): shadowsocks-2022 TCP codec"
```

---

### Task 5: SS TCP dispatch

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/ss/mod.rs` (replace the placeholder doc + add `connect`)
- Modify: `crates/xray-tui-native/src/protocol/mod.rs:50` (the `ProtocolConfig::Ss(_) => not_impl("shadowsocks")` arm)

**Interfaces:**
- Consumes: `stream::connect`, `stream2022::connect`, `method::{SsMethod, password_key}`.
- Produces: `protocol::ss::connect(ctx: &LinkContext, stream: BoxStream, cfg: &SsConfig) -> Result<BoxStream, NativeError>` (named `shadowsocks_connect` in the dispatch to avoid a name clash with `crate::connect`; the inner fns stay `stream::connect` / `stream2022::connect`).

- [ ] **Step 1: Write the failing test** (`protocol/ss/mod.rs` tests)

```rust
#[tokio::test]
async fn dispatch_picks_the_family_codec() {
    let method = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
    assert_eq!(method.family, SsFamily::Blake3_2022);
    let method = SsMethod::from_method("aes-128-gcm").unwrap();
    assert_eq!(method.family, SsFamily::Classic);
}

#[tokio::test]
async fn unknown_method_is_a_config_error() {
    let cfg = SsConfig {
        method: "aes-256-cfb".into(),
        password: "pw".into(),
        security: SecurityConfig::default(),
        remarks: None,
        plugin: None,
        plugin_opts: None,
    };
    let err = resolve_method(&cfg).unwrap_err();
    assert!(matches!(err, NativeError::Config(_)));
}
```

(`SecurityConfig` derives `Default` — `proto_spec::common.rs:294`; `method` is a `TinyText` (`SmartString`), so `"aes-256-cfb".into()` works.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib protocol::ss`
Expected: FAIL — `resolve_method` not found.

- [ ] **Step 3: Implement**

```rust
//! Shadowsocks — native client (classic AEAD + 2022-blake3, TCP and UDP).
//!
//! Cipher scope is the AEAD set + 2022-blake3 (`method.rs`); legacy stream
//! ciphers route to sing-box via [`crate::capability`]. SIP003 plugins are
//! gated off there too. References: shadowsocks-rust `relay/`, mihomo
//! `transport/shadowsocks/`, and the 2022 edition spec.

pub mod method;
pub mod stream;
pub mod stream2022;
pub mod udp;

/// Resolve the row's method, refusing anything native does not implement.
fn resolve_method(cfg: &SsConfig) -> Result<SsMethod, NativeError> {
    SsMethod::from_method(&cfg.method).ok_or_else(|| {
        NativeError::Config(format!(
            "shadowsocks method {:?} has no native implementation",
            cfg.method
        ))
    })
}

/// TCP protocol phase: the family codec writes the handshake and owns the tunnel.
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &SsConfig,
) -> Result<BoxStream, NativeError> {
    let method = resolve_method(cfg)?;
    match method.family {
        SsFamily::Classic => stream::connect(ctx, stream, cfg, method).await,
        SsFamily::Blake3_2022 => stream2022::connect(ctx, stream, cfg, method).await,
    }
}
```

and in `protocol/mod.rs`:

```rust
        ProtocolConfig::Ss(cfg) => ss::connect(ctx, stream, cfg).await,
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib protocol::ss`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol/
git commit -m "feat(native): dispatch shadowsocks TCP through the family codecs"
```

---

### Task 6: Shadowsocks UDP — dial-end carrier (classic + 2022), `PacketTunnel::Ss`

**Files:**
- Create: `crates/xray-tui-native/src/protocol/ss/udp.rs`
- Modify: `crates/xray-tui-native/src/protocol/mod.rs` (`PacketTunnel`/`PacketReader`/`PacketWriter` variants, `is_udp_dial_link`)
- Modify: `crates/xray-tui-native/src/chain.rs` (`connect_chain_udp` arm + `ss_udp_guard`)

**Interfaces:**
- Consumes: `SsMethod`, `password_key`, `stream_subkey`, `addr::{encode_addr_port_last, decode_addr_port_last}`, `rand::fill_nonsecret`, `tokio::net::UdpSocket`.
- Produces:
  - `fn seal_datagram(method, key: &[u8], dest: &TargetAddr, payload: &[u8]) -> Result<Vec<u8>, NativeError>`,
  - `fn open_datagram(method, key: &[u8], packet: &[u8]) -> Result<(TargetAddr, Vec<u8>), NativeError>`,
  - `pub struct SsUdpTunnel { socket: Arc<UdpSocket>, method: SsMethod, key: Arc<Zeroizing<Vec<u8>>>, target: TargetAddr, s2022_writer: Option<Ss2022WriterState>, s2022_reader: Option<Ss2022ReaderState> }` with `send`/`recv`/`split` (the two 2022 states are defined in this task's later steps; classic rows leave both `None`),
  - `pub struct SsUdpReader` / `pub struct SsUdpWriter` (split halves; independent state over the shared `Arc<UdpSocket>`),
  - `pub async fn connect_udp(ctx: &LinkContext, method: SsMethod, cfg: &SsConfig) -> Result<SsUdpTunnel, NativeError>`.

**Transport shape:** SS UDP is a **dial-end** — `bind` a fresh UDP socket and send each datagram to `ctx.params.server`; there is NO TCP tunnel, NO security wrap and NO transport upgrade (the SS server's UDP relay is its own UDP endpoint). One SS packet = one UDP datagram, so there is no carrier length prefix. A chain cannot carry it: the link must be alone/last, guarded like `quic_guard`.

**Address family:** SOCKS5 order (ATYP `0x01`/`0x03`/`0x04`, port LAST) — `encode_addr_port_last` / `decode_addr_port_last`. IPv4 happens to be 7 bytes in both families, so a wrong-family unit test still passes while the wire is wrong; keep the decoder and encoder on the port-last pair in every SS path (classic UDP, 2022 UDP).

- [ ] **Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn classic_udp_packet_is_salt_then_sealed_address_payload() {
    let method = SsMethod::from_method("aes-128-gcm").unwrap();
    let key = Zeroizing::new(vec![0x11u8; 16]);
    let dest = TargetAddr::new(Host::Ip(IpAddr::from([1, 2, 3, 4])), 53);
    let packet = seal_datagram(method, &key, &dest, b"query").unwrap();
    assert_eq!(packet.len(), 16 + 7 + 5 + 16);
    let sub = stream_subkey(method, &key, &packet[..16]);
    let body = method.aead.open(&sub, &[0u8; 12], b"", &packet[16..]).unwrap();
    assert_eq!(&body[..7], &encode_addr_port_last(&dest).unwrap()[..]);
    assert_eq!(&body[7..], b"query");
    // And the decoder reads back exactly that address + payload.
    let (decoded, payload) = open_datagram(method, &key, &packet).unwrap();
    assert_eq!(decoded, dest);
    assert_eq!(payload, b"query");
}

/// Two datagrams to the same destination must use different salts (so a
/// server-side replay cache never rejects the second one).
#[test]
fn classic_udp_salts_are_fresh_per_datagram() {
    let method = SsMethod::from_method("aes-128-gcm").unwrap();
    let key = Zeroizing::new(vec![0x33u8; 16]);
    let dest = TargetAddr::new(Host::Ip(IpAddr::from([10, 0, 0, 7])), 5353);
    let a = seal_datagram(method, &key, &dest, b"ping").unwrap();
    let b = seal_datagram(method, &key, &dest, b"ping").unwrap();
    assert_ne!(&a[..16], &b[..16]);
}

#[tokio::test]
async fn ss_udp_tunnel_round_trips_against_a_loopback_echo() {
    // Bind an echo UdpSocket, then `connect_udp` at a ctx whose `server` is
    // that socket's address; send one datagram through the tunnel and read
    // its reply. The echo side decodes with `open_datagram` and re-seals the
    // reply with `seal_datagram` (the same primitives, not the code under
    // test), so the tunnel's send and recv paths both cross the wire.
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib protocol::ss::udp`
Expected: FAIL.

- [ ] **Step 3: Implement**

```rust
//! Shadowsocks classic-AEAD UDP codec (mihomo `shadowaead/packet.go`).
//!
//! ```text
//! packet = [salt][seal(all-zero nonce, ATYP|addr|port ‖ payload)]
//! ```
//! One packet per UDP datagram, fresh salt and subkey per packet. The address
//! is ALWAYS on the wire (there is no header-destination mode), so `send(None)`
//! means the session target (`ctx.params.target`). Address family is SOCKS5
//! port-last (`encode_addr_port_last`).

/// One datagram: `[salt][seal(zerononce, addr ‖ payload)]`.
fn seal_datagram(
    method: SsMethod,
    key: &[u8],
    dest: &TargetAddr,
    payload: &[u8],
) -> Result<Vec<u8>, NativeError> {
    let mut salt = Zeroizing::new(vec![0u8; method.aead.salt_len()]);
    crate::rand::fill_nonsecret(&mut salt);
    let sub = stream_subkey(method, key, &salt);
    let addr = encode_addr_port_last(dest)?;
    let mut body = Vec::with_capacity(addr.len() + payload.len());
    body.extend_from_slice(&addr);
    body.extend_from_slice(payload);
    let mut out = Vec::with_capacity(salt.len() + body.len() + method.aead.tag_len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&method.aead.seal(
        &sub,
        &vec![0u8; method.aead.nonce_len()],
        b"",
        &body,
    )?);
    Ok(out)
}

/// One received datagram: salt → subkey → open → address/payload split.
fn open_datagram(
    method: SsMethod,
    key: &[u8],
    packet: &[u8],
) -> Result<(TargetAddr, Vec<u8>), NativeError> {
    let salt_len = method.aead.salt_len();
    if packet.len() < salt_len + method.aead.tag_len() {
        return Err(datagram_error("short datagram"));
    }
    let sub = stream_subkey(method, key, &packet[..salt_len]);
    let body = method.aead.open(
        &sub,
        &vec![0u8; method.aead.nonce_len()],
        b"",
        &packet[salt_len..],
    )?;
    let Some((dest, payload)) = decode_addr_port_last(&body) else {
        return Err(datagram_error("undecodable address in datagram"));
    };
    Ok((dest, payload.to_vec()))
}
```

Tunnel:

```rust
/// The SS UDP carrier: owns the UDP socket to the SS server plus the codec key.
///
/// The 2022 session state is split by direction (writer: client session id +
/// packet counter; reader: server-session map + replay window), so
/// [`Self::split`] hands each half its own state with no lock. Classic rows
/// leave both `None` (stateless per datagram).
pub struct SsUdpTunnel {
    socket: Arc<UdpSocket>,
    method: SsMethod,
    /// Shared (one wipe copy) with the split halves.
    key: Arc<Zeroizing<Vec<u8>>>,
    target: TargetAddr,
    s2022_writer: Option<Ss2022WriterState>,
    s2022_reader: Option<Ss2022ReaderState>,
}

impl SsUdpTunnel {
    /// Send one datagram. `dest = None` → `self.target`. Both codecs take a
    /// `TargetAddr` (the address travels INSIDE the sealed body, so a domain
    /// destination is legal on the wire).
    pub async fn send(&mut self, dest: Option<SocketAddr>, payload: &[u8]) -> std::io::Result<()> {
        let packet = match &mut self.s2022_writer {
            None => {
                let dest = dest.map_or_else(|| self.target.clone(), target_from_socket);
                seal_datagram(self.method, &self.key, &dest, payload)
            }
            Some(w) => w.seal(
                &dest.map_or_else(|| self.target.clone(), target_from_socket),
                payload,
            ),
        }?;                                   // map NativeError → io::Error
        self.socket.send(&packet).await.map(|_| ())
    }

    /// Receive one datagram; `Some(addr)` is the origin the reply header
    /// carried (`None` when that origin is a domain name).
    pub async fn recv(&mut self) -> std::io::Result<Option<(Option<SocketAddr>, Vec<u8>)>> { /* recv + open_datagram / Ss2022ReaderState::open → TargetAddr → Option<SocketAddr> */ }

    /// Classic rows: pure `Arc::clone` of the socket. 2022 rows: the writer
    /// state goes to the writer half, the reader state to the reader half.
    pub fn split(self) -> (SsUdpReader, SsUdpWriter) { /* move, no lock */ }
}
```
/// `bind` a fresh UDP socket to the SS server — the whole dial. The server
/// address goes through `LinkContext::server_socket()`, which honours
/// `params.resolved_ip` and bounds DNS with `timeouts::DIAL` (resolving the
/// host inline here would bypass both).
pub async fn connect_udp(ctx: &LinkContext, method: SsMethod, cfg: &SsConfig) -> Result<SsUdpTunnel, NativeError> {
    let key = Arc::new(password_key(method, &cfg.password)?);
    let server = ctx.server_socket().await?;
    let bind: SocketAddr = if server.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" }.parse().expect(" literal");
    let socket = UdpSocket::bind(bind).await.map_err(|e| NativeError::Dial(e.to_string()))?;
    socket.connect(server).await.map_err(|e| NativeError::Dial(e.to_string()))?;
    let (s2022_writer, s2022_reader) = match method.family {
        SsFamily::Blake3_2022 => (
            Some(Ss2022WriterState::new(method, Arc::clone(&key))),
            Some(Ss2022ReaderState::new(method, Arc::clone(&key))),
        ),
        SsFamily::Classic => (None, None),
    };
    Ok(SsUdpTunnel { socket: Arc::new(socket), method, key, target: ctx.params.target.clone(), s2022_writer, s2022_reader })
}
```

Two tiny private converters live in this module (do NOT add a second dial helper): `fn target_from_socket(sa: SocketAddr) -> TargetAddr { TargetAddr::new(Host::Ip(sa.ip()), sa.port()) }` for `send(Some(..))`/`recv`'s source, and `recv` returns `None` for the `SocketAddr` when the decoded origin address is a **domain** (a `TargetAddr` domain has no `SocketAddr`; replies in practice carry an IP).

Then `protocol/mod.rs`:

```rust
/// True when the link's UDP path dials its own UDP socket instead of riding
/// the TCP-tunnel stream carrier (Shadowsocks' UDP relay is a separate UDP
/// endpoint — there is no in-tunnel framing for it).
#[must_use]
pub(crate) const fn is_udp_dial_link(ctx: &LinkContext) -> bool {
    matches!(ctx.params.protocol, ProtocolConfig::Ss(_))
}

pub enum PacketTunnel { …, Ss(ss::udp::SsUdpTunnel) }
pub enum PacketReader { …, Ss(ss::udp::SsUdpReader) }
pub enum PacketWriter { …, Ss(ss::udp::SsUdpWriter) }
```

and in `chain.rs::connect_chain_udp`, next to the `is_quic_link` arm:

```rust
        if protocol::is_udp_dial_link(&ctx) {
            // Shadowsocks UDP: the dial IS a UDP socket to the server —
            // no security, no transport, no base tunnel. A UDP-dial link
            // cannot ride a chain (mirrors `quic_guard`).
            ss_udp_guard(links, i, base.as_ref())?;
            let ProtocolConfig::Ss(cfg) = &ctx.params.protocol else {
                unreachable!("is_udp_dial_link matches only Ss");
            };
            let method = ss::resolve_method(cfg)?;
            protocol::reject_vless_only_mode(&ctx, method.kind())?;
            return Ok(PacketTunnel::Ss(ss::udp::connect_udp(&ctx, method, cfg).await?));
        }
```

with

```rust
/// An SS UDP link dials its own UDP socket: it must be the only link.
fn ss_udp_guard(links: &[NativeConnectParams], i: usize, base: Option<&BoxStream>) -> Result<(), NativeError> {
    if links.len() != 1 || base.is_some() {
        return Err(NativeError::Config(
            "shadowsocks UDP dials its own UDP socket and cannot ride a proxy chain".into(),
        ));
    }
    let _ = i;
    Ok(())
}
```

- [ ] **Step 4: Write the failing 2022 tests (same task — the tunnel's 2022 fields are filled here)**

Wire (spec §3.2 + §4.1, shadowsocks-rust `udprelay/aead_2022.rs`):

```text
AES methods:
  packet = [AES-ECB(psk, client_session_id u64be ‖ client_packet_id u64be)]        (16 B)
         ‖ seal(nonce = separate_header[4..16],
                [type=0 | ts u64be | pad_len u16be | padding | ATYP|addr|port | payload])
  subkey = blake3::derive_key("shadowsocks 2022 session subkey", psk ‖ client_session_id bytes)
server → client (client must parse):
  [AES-ECB(psk, server_session_id ‖ server_packet_id)] ‖ seal(nonce,
    [type=1 | ts | client_session_id 8B | pad_len | padding | addr | port | payload])
ChaCha method:
  [24B random nonce] ‖ XChaCha20Poly1305(psk directly, [client_session_id ‖ client_packet_id ‖ type | ts | pad_len | padding | addr | port | payload])
```

```rust
#[test]
fn separate_header_is_aes_ecb_of_ids_with_the_psk() {
    let method = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
    let psk = [0x03u8; 32];
    let header = separate_header_aes(method.aead, &psk, 0x0102_0304_0506_0708, 0x11);
    // AES-ECB is a permutation: the two halves are not identity.
    assert_ne!(header, 0x0102_0304_0506_0708u64.to_be_bytes());
    let mut plain = [0u8; 16];
    aes_ecb_decrypt(method.aead, &psk, &header, &mut plain);
    assert_eq!(&plain[..8], &0x0102_0304_0506_0708u64.to_be_bytes());
    assert_eq!(&plain[8..], &0x11u64.to_be_bytes());
}

#[test]
fn udp_session_subkey_uses_the_client_session_id() {
    let psk = [0x04u8; 32];
    let k1 = udp_session_subkey(&psk, 1);
    let k2 = udp_session_subkey(&psk, 2);
    assert_eq!(k1.len(), 32);
    assert_ne!(k1, k2);
}

#[test]
fn sliding_window_rejects_duplicates_and_old_packets() {
    let mut w = SlidingWindow::new();
    assert!(w.accept(0));
    assert!(!w.accept(0)); // duplicate
    assert!(w.accept(1));
    assert!(w.accept(63));
    assert!(w.accept(64));
    assert!(w.accept(65));
    assert!(!w.accept(64)); // duplicate inside the window
    assert!(w.accept(200)); // jump forward
    assert!(!w.accept(2)); // fell out of the 64-packet window
}

/// The window is PER relay session: after a server restart the new session's
/// ids restart at 0 and must be accepted (a global window would reject every
/// reply — spec §3.2.4's old/current association exists for this).
#[test]
fn each_server_session_has_its_own_window_and_one_old_slot_is_kept() {
    let mut s = ServerSessions::new(7);
    assert_eq!(s.check(100, 0), Some(7)); // resolves/creates the slot
    s.commit(100, 0);
    assert_eq!(s.check(100, 0), None); // duplicate on the same session
    assert_eq!(s.check(200, 0), Some(7), "new server session, fresh window");
    s.commit(200, 0);
    assert_eq!(s.check(100, 1), Some(7), "one old association stays valid");
    s.commit(100, 1);
    assert_eq!(s.check(300, 0), Some(7)); // third session evicts the oldest
    s.commit(300, 0);
    assert_eq!(s.check(100, 2), None, "only one old association is retained");
    assert_eq!(s.check(200, 2), Some(7));
}

/// Window state must not advance on unauthenticated packets: a spoofed
/// high-id datagram is checked (slot created), the body open fails, and the
/// caller never commits — so the real reply that follows is still in-window
/// (spec §3.2.4 forbids updating the window before validation).
#[test]
fn check_without_commit_does_not_advance_the_window() {
    let mut s = ServerSessions::new(7);
    assert_eq!(s.check(100, 5000), Some(7)); // spoofed; open fails ⇒ no commit
    assert_eq!(s.check(100, 0), Some(7), "fresh window still accepts id 0");
    s.commit(100, 0);
    assert_eq!(s.check(100, 0), None, "committed id is a duplicate");
}
```

- [ ] **Step 5: Run to verify they fail**

Run: `cargo test -p xray-tui-native --lib protocol::ss::udp`
Expected: FAIL — the 2022 helpers do not exist.

- [ ] **Step 6: Implement the 2022 codecs**

Add to `udp.rs`:
- `fn aes_ecb_encrypt(aead: SsAead, psk: &[u8], block: &mut [u8; 16])` / `fn aes_ecb_decrypt(..)` via `Aes128`/`Aes256` `encrypt_block`/`decrypt_block` — 2022 has no aes-192 method, so any other cipher is a `NativeError::Config`.
- `fn separate_header_aes(aead, psk, session_id: u64, packet_id: u64) -> [u8; 16]` (AES-ECB over `session_id BE8 ‖ packet_id BE8`), `fn separate_header_nonce(session_id: u64, packet_id: u64) -> [u8; 12]` (the **plaintext** header's `[4..16]`: last 4 bytes of the session id ‖ all 8 bytes of the packet id — spec §3.2.1; NEVER take a nonce off the wire ciphertext at `packet[4..16]`), and `fn udp_session_subkey(key: &[u8], session_id: u64, key_len: usize) -> Zeroizing<Vec<u8>>` (`blake3::derive_key("shadowsocks 2022 session subkey", key ‖ session_id BE8)` **truncated to `key_len`**, exactly like Task 2's `stream_subkey` — the UDP body AEAD takes the key length of the method, so the 32-byte root is wrong for 2022-blake3-aes-128-gcm).
- `struct ServerSessions::new(client_session_id: u64)` (single-user PSK ⇒ exactly one local client session) with a **split check/commit API** — spec §3.2.4: the id MAY be checked right after the separate header decrypts, but the window MUST NOT advance before the body authenticates and the header validates (otherwise a spoofed high-id datagram with a garbage body desyncs the session and every later real reply looks out-of-window):
  - `fn check(&mut self, server_id: u64, packet_id: u64) -> Option<u64>` — resolves (or creates, learning `client_session_id`) the slot, derives/caches that slot's `body_subkey`, runs the membership test only, returns the client session id; no window mutation.
  - `fn body_subkey(&self, server_id: u64) -> Option<&[u8]>` — the cached subkey for the body open.
  - `fn commit(&mut self, server_id: u64, packet_id: u64)` — advances THAT slot's window; called only after the body opened and type/timestamp/client-session validation passed.
  - `SlidingWindow` starts with no committed id (`highest: Option<u64>`), so a fresh slot accepts any first id (a server session always starts at 0 after a restart).
  - Exactly one previous association is retained (spec §3.2.4).
- `Ss2022WriterState { aead, key: Arc<Zeroizing<Vec<u8>>>, client_session_id: u64, packet_id: u64, subkey: Zeroizing<Vec<u8>> }`:
  - `new` draws the 8-byte client session id (`rand::fill_nonsecret`, read as BE) and caches `subkey = udp_session_subkey(key, client_session_id)`.
  - `seal`: AES family → `separate_header_nonce(client_session_id, packet_id)` for the body seal, then `separate_header_aes(..)` prepended as the wire front: `packet = [encrypted separate header][aead.seal(key = subkey, nonce = separate_header_nonce(..), aad = "", body)]`, body = `type=0 ‖ ts u64be ‖ pad_len u16be ‖ padding ‖ encode_addr_port_last(dest) ‖ payload`; packet id increments per packet. ChaCha family → no separate header: `[24B random nonce] ‖ XChaCha20Poly1305(psk directly, nonce, client_session_id BE8 ‖ packet_id BE8 ‖ body)`.
- `Ss2022ReaderState { aead, key: Arc<Zeroizing<Vec<u8>>>, sessions: ServerSessions }`:
  - `open`: AES family → ECB-decrypt `packet[..16]` with the psk to recover the plaintext separate header `(server_session_id, server_packet_id)`; `let Some(client) = sessions.check(server_session_id, server_packet_id) else { return Ok(None) };` `aead.open(key = sessions.body_subkey(server_session_id).unwrap(), nonce = separate_header_nonce(server_session_id, server_packet_id), aad = "", packet[16..])` — an open failure also returns `Ok(None)` **without committing**; parse the body, validate `type == 1`, the timestamp within 30 s, and `client_session_id == client`; only then `sessions.commit(server_session_id, server_packet_id)`. ChaCha family: same check → open → validate → commit order with the psk and the 24-byte leading nonce, ids read from the merged header. The opened body is the server main header `type=1 ‖ ts ‖ client_session_id BE8 ‖ pad_len u16be ‖ padding ‖ addr ‖ port ‖ payload` → `(origin TargetAddr, payload)`. A datagram dropped at any step never ends the session.
- Wire the two states into `SsUdpTunnel` (Task 6) exactly as its struct declares: `send` → `s2022_writer`, `recv` → `s2022_reader`, `split` moves them into `SsUdpWriter`/`SsUdpReader`; classic rows keep `seal_datagram`/`open_datagram` and both options stay `None`.

- [ ] **Step 7: Run the whole UDP module to verify pass**

Run: `cargo test -p xray-tui-native --lib protocol::ss::udp`
Expected: PASS (classic + 2022 tests).

**Then the coordinator commits one task commit** (this plan runs under subagent-driven development: implementers never commit; see the ledger's coordinator-commit ruling).
### Task 7: Capability gate

**Files:**
- Modify: `crates/xray-tui-native/src/capability.rs` (`NATIVE_KINDS`, `kind_supported`, `supported`, new `ss_supported`, tests)

**Interfaces:**
- Consumes: `protocol::ss::method::{SsMethod, password_key}`, `security_supported`, `transport_supported`.
- Produces: `fn ss_supported(kind: ProtocolKind, cfg: &SsConfig) -> bool`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn ss_kinds_are_native_and_legacy_methods_defer() {
    assert!(kind_supported(ProtocolKind::Shadowsocks));
    assert!(kind_supported(ProtocolKind::Shadowsocks2022));
    assert!(supported(ProtocolKind::Shadowsocks, &ss_cfg("aes-128-gcm", "pw")));
    assert!(supported(
        ProtocolKind::Shadowsocks2022,
        &ss_cfg("2022-blake3-aes-256-gcm", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
    ));
    // legacy + unknown methods stay on sing-box
    assert!(!supported(ProtocolKind::Shadowsocks, &ss_cfg("aes-256-cfb", "pw")));
    assert!(!supported(ProtocolKind::Shadowsocks, &ss_cfg("none", "pw")));
}

#[test]
fn ss_kind_and_method_family_must_agree() {
    // a 2022 method on the classic kind (and vice versa) is a mismatch
    assert!(!supported(
        ProtocolKind::Shadowsocks,
        &ss_cfg("2022-blake3-aes-256-gcm", "AAAAAAAAAAAAAAAAAAAAAA==")
    ));
    assert!(!supported(ProtocolKind::Shadowsocks2022, &ss_cfg("aes-128-gcm", "pw")));
}

#[test]
fn ss_plugin_rows_defer() {
    let mut cfg = ss_cfg("aes-128-gcm", "pw");
    cfg.plugin = Some("obfs-local".into());
    assert!(!supported(ProtocolKind::Shadowsocks, &cfg));
}

#[test]
fn ss2022_password_length_is_validated_at_gate_time() {
    assert!(!supported(ProtocolKind::Shadowsocks2022, &ss_cfg("2022-blake3-aes-128-gcm", "c2hvcnQ=")));
    assert!(!supported(ProtocolKind::Shadowsocks2022, &ss_cfg("2022-blake3-aes-128-gcm", "!!!")));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-native --lib capability`
Expected: FAIL.

- [ ] **Step 3: Implement**

```rust
pub const NATIVE_KINDS: &[ProtocolKind] = &[
    ProtocolKind::Vless,
    ProtocolKind::Vmess,
    ProtocolKind::Trojan,
    ProtocolKind::Hysteria2,
    ProtocolKind::Shadowsocks,
    ProtocolKind::Shadowsocks2022,
];
```

Add to the `supported` match:

```rust
        (ProtocolKind::Shadowsocks | ProtocolKind::Shadowsocks2022, ProtocolConfig::Ss(cfg)) => {
            ss_supported(kind, cfg)
        }
```

and:

```rust
/// Shadowsocks row: native AEAD/2022 method, no SIP003 plugin, a fingerprint
/// native parses, implemented transport.
///
/// Method↔kind agreement matters: the two kinds share `SsConfig`, and a KDF
/// mismatch means the wrong key schedule, not a fallback.
fn ss_supported(kind: ProtocolKind, cfg: &SsConfig) -> bool {
    let Some(method) = SsMethod::from_method(&cfg.method) else {
        return false;
    };
    if method.kind() != kind {
        return false;
    }
    if cfg.plugin.is_some() || cfg.plugin_opts.is_some() {
        return false;
    }
    if password_key(method, &cfg.password).is_err() {
        return false;
    }
    security_supported(&cfg.security) && transport_supported(&cfg.transport, cfg.path.as_deref())
}
```

Also extend the existing `NATIVE_KINDS` equality test in the module and the `supported()` doc-comment (the SS UDP path is `connect_udp`, the SOCKS5-proxy UDP gap is unchanged).

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p xray-tui-native --lib capability`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/capability.rs
git commit -m "feat(native): capability gate for shadowsocks + shadowsocks-2022"
```

---

### Task 8: E2E harness + matrix

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/case.rs` (`ProtocolKind::Shadowsocks`, `ss_method` field, `CaseSpec::shadowsocks`, label/server_config/client_params arms)
- Modify: `crates/xray-tui-native/src/e2e/config.rs` (`ss_inbound`, `client_params_ss`, `SS_PASSWORD`, `SS_2022_KEY`)
- Create: `crates/xray-tui-native/tests/shadowsocks.rs`

**Interfaces:**
- Consumes: everything above; `spawn_core`, `run_against`, `probe_udp`.
- Produces: `CaseSpec::shadowsocks(method: &'static str) -> Self`.

Server configs (verified against `infra/conf/shadowsocks.go` and `option/shadowsocks.go`):

```jsonc
// xray (protocol "shadowsocks" serves 2022 methods through the 2022 inbound)
{"inbounds":[{"listen":"127.0.0.1","port":P,"protocol":"shadowsocks",
  "settings":{"method":M,"password":PW,"network":"tcp"}}],
 "streamSettings":{...tls/ws as for trojan...},"outbounds":[{"protocol":"freedom"}]}
// sing-box
{"inbounds":[{"type":"shadowsocks","listen":"127.0.0.1","listen_port":P,
  "method":M,"password":PW, "transport":{...} }],"outbounds":[{"type":"direct"}]}
```

UDP rows set xray's `"network": "tcp,udp"` (sing-box serves UDP by default).

- [ ] **Step 1: Write the failing test**

```rust
//! E2E: native Shadowsocks (classic AEAD + 2022-blake3) matrix.
#![allow(clippy::future_not_send)]
#![cfg(feature = "native-e2e")]

mod common;

use common::{certs, cores, echo, fp, no_tls, pick, tls_echo};
use rstest::rstest;
use xray_tui_native::PacketMode;
use xray_tui_native::e2e::AppKind;
use xray_tui_native::e2e::{
    CaseSpec, Certs, CoreKind, CoreUnderTest, E2eCase, EchoServer, TlsEchoServer, run_against,
};

fn ss(method: &'static str) -> CaseSpec {
    CaseSpec::shadowsocks(method).with_tls(no_tls())
}

#[rstest]
#[case::aead_aes128(ss("aes-128-gcm"))]
#[case::aead_chacha20(ss("chacha20-ietf-poly1305"))]
#[case::aead_xchacha20(ss("xchacha20-ietf-poly1305"))]
#[case::ss2022_aes256(ss("2022-blake3-aes-256-gcm"))]
#[case::ss2022_chacha20(ss("2022-blake3-chacha20-poly1305"))]
#[case::aead_ws(ss("aes-128-gcm").with_network("ws"))]
#[case::aead_tls_chrome(ss("aes-128-gcm").with_tls(fp("chrome")))]
#[case::udp_aead(ss("aes-128-gcm").with_app(AppKind::Udp).with_udp(PacketMode::Raw))]
#[case::udp_2022(ss("2022-blake3-aes-256-gcm").with_app(AppKind::Udp).with_udp(PacketMode::Raw))]
#[tokio::test]
async fn shadowsocks_against_cores(
    #[case] case: CaseSpec,
    #[values(CoreKind::Xray, CoreKind::SingBox)] core: CoreKind,
    cores: &(CoreUnderTest, CoreUnderTest),
    certs: &Certs,
    echo: EchoServer,
    tls_echo: TlsEchoServer,
) {
    let label = case.label();
    run_against(&case, pick(cores, core), certs, &echo, &tls_echo)
        .await
        .unwrap_or_else(|e| panic!("{label}: e2e failed: {e}"));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test shadowsocks`
Expected: FAIL to compile — `CaseSpec::shadowsocks` / `config::ss_inbound` missing.

- [ ] **Step 3: Implement**

`case.rs`: add `Shadowsocks` to `e2e::case::ProtocolKind`, add `ss_method: Option<&'static str>` to `CaseSpec` (init `None` in every existing constructor), add

```rust
    /// A Shadowsocks case: `method` selects the cipher family
    /// (`aes-128-gcm`, `chacha20-ietf-poly1305`, `xchacha20-ietf-poly1305`,
    /// `2022-blake3-aes-256-gcm`, `2022-blake3-chacha20-poly1305`).
    #[must_use]
    pub fn shadowsocks(method: &'static str) -> Self { /* struct literal, ss_method: Some(method) */ }
```

then the `label()` arm (`ProtocolKind::Shadowsocks => "ss"`, plus `/{method}` when set), the `server_config` arm (`config::ss_inbound(core, env, self.tls(), self.network, method, self.app == AppKind::Udp)`), and the `client_params` arm (`config::client_params_ss(port, target, self.tls(), self.network, method)`).

`config.rs`: `ss_inbound` / `client_params_ss` mirroring `trojan_inbound`/`client_params_trojan`, with

```rust
pub const SS_PASSWORD: &str = "ss-test-password";
/// 2022-blake3-aes-256-gcm PSK: 32 zero bytes, base64.
pub const SS_2022_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
/// 16-byte PSK for the aes-128 method rows.
pub const SS_2022_KEY_128: &str = "AAAAAAAAAAAAAAAAAAAAAA==";
fn ss_password(method: &str) -> &'static str {
    if method.starts_with("2022-blake3-aes-128") { SS_2022_KEY_128 }
    else if method.starts_with("2022-blake3") { SS_2022_KEY }
    else { SS_PASSWORD }
}
```

and the client params protocol JSON:

```rust
    let mut protocol_value = serde_json::json!({
        "schema": "Ss",
        "method": method,
        "password": ss_password(method),
        "transport": transport
    });
```
(so `xchacha20-ietf-poly1305` → `xchacha20-ietf-poly1305` and `2022-blake3-chacha20-poly1305` → `2022-blake3-chacha20-poly1305`; the transport mapping copies `client_params_trojan`).

- [ ] **Step 4: Run the matrix**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test shadowsocks`
Expected: PASS — 9 cases × 2 cores = 18 rows (minus any single-core gate).

If `xray` rejects `2022-blake3-chacha20-poly1305` (its accepted list is `shadowaead_2022.List`), do NOT weaken the row: add a public gate to `CaseSpec`

```rust
    /// Restrict a row to one core (used when the server side genuinely
    /// cannot serve the config — e.g. an xray method whitelist gap).
    #[must_use]
    pub const fn with_single_core(mut self, kind: CoreKind) -> Self {
        self.single_core = Some(kind);
        self
    }
```

set exactly that case to `.with_single_core(CoreKind::SingBox)`, and record the reason in the case comment. Any other row that fails must be investigated, not gated.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/e2e/ crates/xray-tui-native/tests/shadowsocks.rs
git commit -m "test(native): shadowsocks + 2022 e2e matrix against both cores"
```

---

### Task 9: Benches

**Files:**
- Create: `crates/xray-tui-native/benches/ss_codec.rs`
- Modify: `crates/xray-tui-native/benches/throughput.rs` (rows)
- Modify: `crates/xray-tui-native/Cargo.toml` (`[[bench]] name = "ss_codec"`)

**Interfaces:**
- Consumes: `xray_tui_native::crypto::crypto::……` — the codec types must be reachable for benches: `SsAead`, `NonceCounter`, `protocol::ss::method::{SsMethod, password_key, stream_subkey}` (all public; `protocol` and `crypto` are `pub mod`).
- Produces: criterion group `ss_codec` with `classic/{cipher}/{seal,open}`, `s2022/{cipher}/{seal,open}`, `kdf/{hkdf_sha1,blake3}`, `udp/{aead,2022}`.

- [ ] **Step 1: Write the bench (it must build without `native-e2e`)**

```rust
//! Hermetic Shadowsocks codec benches: per-op AEAD cost and per-connection KDF
//! cost. No core binary, no network, no feature gate — `cargo criterion
//! --bench ss_codec` always measures.
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use xray_tui_native::crypto::aead::{NonceCounter, SsAead};
use xray_tui_native::protocol::ss::method::{SsMethod, password_key, stream_subkey};

fn criterion_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("ss_codec");
    group.throughput(Throughput::Bytes(16 * 1024));
    for (name, aead) in [
        ("aes-128-gcm", SsAead::Aes128Gcm),
        ("aes-256-gcm", SsAead::Aes256Gcm),
        ("chacha20-ietf-poly1305", SsAead::ChaCha20Poly1305),
        ("xchacha20-ietf-poly1305", SsAead::XChaCha20Poly1305),
    ] {
        let key = vec![0x11u8; aead.key_len()];
        let mut nonce = NonceCounter::new(aead.nonce_len());
        let pt = vec![0xABu8; 0x3FFF];
        group.bench_function(format!("classic/{name}/seal"), |b| {
            b.iter(|| aead.seal(&key, nonce.next(), b"", &pt).unwrap());
        });
        let ct = aead.seal(&key, &vec![0u8; aead.nonce_len()], b"", &pt).unwrap();
        group.bench_function(format!("classic/{name}/open"), |b| {
            b.iter(|| {
                aead.open(&key, &vec![0u8; aead.nonce_len()], b"", &ct)
                    .unwrap()
            });
        });
    }
    group.throughput(Throughput::Elements(1)); // per-connection KDF cost
    let classic = SsMethod::from_method("aes-256-gcm").unwrap();
    let s2022 = SsMethod::from_method("2022-blake3-aes-256-gcm").unwrap();
    let key = password_key(classic, "bench-password").unwrap();
    let psk = password_key(s2022, SS_2022_BENCH_KEY).unwrap();
    group.bench_function("kdf/hkdf_sha1", |b| {
        b.iter(|| stream_subkey(classic, &key, &[0x22u8; 32]));
    });
    group.bench_function("kdf/blake3", |b| {
        b.iter(|| stream_subkey(s2022, &psk, &[0x33u8; 32]));
    });
    group.finish();
}

const SS_2022_BENCH_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
```

Add to `Cargo.toml`:

```toml
# Hermetic codec bench: no core binary, no network, no feature gate.
[[bench]]
name = "ss_codec"
harness = false
```

- [ ] **Step 2: Add the throughput rows**

In `benches/throughput.rs::rows()` (after the trojan row), four rows using a new `CaseSpec::shadowsocks(method).with_tls(Box::new(NoTls))`:

```rust
        Row { name: "ss/tcp/aead-aes-128-gcm", build: || CaseSpec::shadowsocks("aes-128-gcm").with_tls(Box::new(NoTls)), core: CoreKind::Xray },
        Row { name: "ss/tcp/aead-chacha20-ietf-poly1305", build: || CaseSpec::shadowsocks("chacha20-ietf-poly1305").with_tls(Box::new(NoTls)), core: CoreKind::Xray },
        Row { name: "ss/tcp/2022-blake3-aes-256-gcm", build: || CaseSpec::shadowsocks("2022-blake3-aes-256-gcm").with_tls(Box::new(NoTls)), core: CoreKind::Xray },
        Row { name: "ss/tcp/2022-blake3-chacha20-poly1305", build: || CaseSpec::shadowsocks("2022-blake3-chacha20-poly1305").with_tls(Box::new(NoTls)), core: CoreKind::SingBox },
```

- [ ] **Step 3: Run both benches**

```bash
cargo criterion --bench ss_codec          # hermetic: must print measured rows
just bench micro                           # relay + dispatch + throughput-independent rows
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo criterion --bench throughput -- 'ss/tcp'
```
Expected: `ss_codec` reports numbers for every cipher + both KDFs; the throughput rows report MB/s instead of a SKIP.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/benches/ crates/xray-tui-native/Cargo.toml
git commit -m "bench(native): shadowsocks codec + throughput rows"
```

---

### Task 10: Docs

**Files:**
- Modify: `NATIVE_CORE.md` (crate map `protocol/` + `crypto/` rows, capability notes, tier counts, SS axis paragraph, deferral list)
- Modify: `AGENTS.md` (native crate-map line, `NATIVE_KINDS` mentions, decision 20's protocol list)

- [ ] **Step 1: Update `NATIVE_CORE.md`**

Add an SS axis paragraph next to the trojan/hysteria2 axes describing: classic AEAD (HKDF-SHA1 subkey, 0x3FFF chunks, MD5 `EVP_BytesToKey`) + 2022-blake3 (blake3 subkey, standalone headers, request-salt check), UDP in both families, and the deferrals (SIP003 plugins, legacy stream ciphers, 2022 multi-user EIH). Update the `protocol/` and `crypto/` crate-map rows and the tier-3 count sentence.

- [ ] **Step 2: Update `AGENTS.md`**

Replace the four-protocol wording in the native crate-map bullet / decision 20 / capability mentions with the six (add `shadowsocks`, `shadowsocks-2022`), noting the plugin + legacy-cipher gates.

- [ ] **Step 3: Commit**

```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs: native shadowsocks support in the core maps + capability notes"
```

---

### Task 11: Full verification

**Files:** none (verification only).

- [ ] **Step 1: Tier 1 (hermetic)**

```bash
cargo test -p xray-tui-native --features native-e2e --lib
```
Expected: all green, including the ~30 new SS unit tests.

- [ ] **Step 2: Tier 3 (real cores)**

```bash
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e \
  --test shadowsocks --test vless --test vmess --test trojan --test hysteria2
```
Expected: SS rows green; no regression in the existing matrices.

- [ ] **Step 3: Quality gate**

```bash
just quality-gate
```
Expected: fmt, hakari, clippy (pedantic+nursery), nextest, deny, machete, outdated, audit all pass. Fix any clippy finding inline (`clippy::future_not_send` is already allowed file-wide in the e2e tests; the codecs buffer state in structs instead of futures).

- [ ] **Step 4: Bench smoke**

```bash
cargo criterion --bench ss_codec
```
Expected: numbers for every row (this is the recorded baseline).

- [ ] **Step 5: Commit any gate fixes**

```bash
git add -A
git commit -m "chore(native): clippy/fmt fixes for the shadowsocks work"
```
