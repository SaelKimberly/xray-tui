//! AEAD body ciphers: AES-128/192/256-GCM and ChaCha20/XChaCha20-Poly1305.
//!
//! Consumers: `protocol/ss` (classic AEAD + 2022-blake3, Tasks 3/4/6). The
//! VMess record codec (`protocol/vmess/stream.rs`) drives the same RustCrypto
//! primitives directly for its own record framing.
//! Backend: RustCrypto (`aes-gcm`, `chacha20poly1305`) — not ring: both are
//! already in the tree with the `zeroize` feature, and their explicit-nonce
//! APIs are what the Shadowsocks counters need. References: RFC 8439,
//! NIST SP 800-38D, draft-irtf-cfrg-xchacha.

use aes_gcm::aead::Payload;
use aes_gcm::{Aes128Gcm, Aes256Gcm};
use chacha20poly1305::{ChaCha20Poly1305, XChaCha20Poly1305};
use zeroize::Zeroizing;

use crate::error::NativeError;
use xray_tui_proto::proto_spec::ProtocolKind;

/// AES-192-GCM. `aes-gcm` 0.11 exports only the 128/256-bit aliases, so the
/// 192-bit key size is spelled out from `AesGcm<Aes, NonceSize, TagSize>`.
type Aes192Gcm = aes_gcm::AesGcm<aes::Aes192, aes_gcm::aead::consts::U12>;

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
        if self.key_len() > 16 {
            self.key_len()
        } else {
            16
        }
    }

    /// Nonce length: 24 for XChaCha20-Poly1305, 12 for every other method.
    #[must_use]
    pub const fn nonce_len(self) -> usize {
        match self {
            Self::XChaCha20Poly1305 => 24,
            _ => 12,
        }
    }

    /// Poly1305/GCM tag length — 16 bytes for the whole Shadowsocks set.
    #[must_use]
    pub const fn tag_len(self) -> usize {
        16
    }

    /// Seal `plaintext` under `key`/`nonce`, authenticating `aad`; the tag is
    /// appended to the returned ciphertext.
    pub fn seal(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, NativeError> {
        match self {
            Self::Aes128Gcm => seal_with::<Aes128Gcm>(key, nonce, aad, plaintext),
            Self::Aes192Gcm => seal_with::<Aes192Gcm>(key, nonce, aad, plaintext),
            Self::Aes256Gcm => seal_with::<Aes256Gcm>(key, nonce, aad, plaintext),
            Self::ChaCha20Poly1305 => seal_with::<ChaCha20Poly1305>(key, nonce, aad, plaintext),
            Self::XChaCha20Poly1305 => seal_with::<XChaCha20Poly1305>(key, nonce, aad, plaintext),
        }
    }

    /// Open `ciphertext` (tag trailing) under `key`/`nonce`, requiring `aad`
    /// to match the value sealed with it.
    pub fn open(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, NativeError> {
        match self {
            Self::Aes128Gcm => open_with::<Aes128Gcm>(key, nonce, aad, ciphertext),
            Self::Aes192Gcm => open_with::<Aes192Gcm>(key, nonce, aad, ciphertext),
            Self::Aes256Gcm => open_with::<Aes256Gcm>(key, nonce, aad, ciphertext),
            Self::ChaCha20Poly1305 => open_with::<ChaCha20Poly1305>(key, nonce, aad, ciphertext),
            Self::XChaCha20Poly1305 => open_with::<XChaCha20Poly1305>(key, nonce, aad, ciphertext),
        }
    }
}

/// Every cipher here is a RustCrypto `aead::Aead` (`aes-gcm` and
/// `chacha20poly1305` share the 0.6 trait), so one generic pair covers all
/// five — key/nonce widths come from the algorithm's own `KeyInit`/`AeadCore`.
fn seal_with<C>(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, NativeError>
where
    C: aes_gcm::aead::Aead + aes_gcm::aead::KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| bad_key())?;
    let nonce: &aes_gcm::aead::Nonce<C> = nonce.try_into().map_err(|_| bad_nonce())?;
    cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| cipher_error("AEAD seal failed"))
}

fn open_with<C>(
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, NativeError>
where
    C: aes_gcm::aead::Aead + aes_gcm::aead::KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| bad_key())?;
    let nonce: &aes_gcm::aead::Nonce<C> = nonce.try_into().map_err(|_| bad_nonce())?;
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| cipher_error("AEAD open failed (tampered or wrong key)"))
}

fn bad_key() -> NativeError {
    cipher_error("bad AEAD key length")
}

fn bad_nonce() -> NativeError {
    cipher_error("bad AEAD nonce length")
}

fn cipher_error(detail: &str) -> NativeError {
    NativeError::Protocol {
        kind: ProtocolKind::Shadowsocks,
        detail: detail.to_owned(),
    }
}

/// Little-endian counter over a fixed-width nonce, handed out once per AEAD
/// operation — the classic-AEAD and 2022 TCP/stream nonce.
///
/// `next` yields the all-zero nonce first and bumps the counter after each
/// hand-out, wrapping at the top byte. Widths used in the tree are 12 (GCM,
/// ChaCha20-Poly1305) and 24 (XChaCha20-Poly1305).
pub struct NonceCounter {
    buf: Zeroizing<[u8; 24]>,
    len: usize,
    /// Set once the initial all-zero nonce has been handed out: from then on
    /// every `next` bumps the counter before returning it.
    handed_out: bool,
}

impl NonceCounter {
    #[must_use]
    pub fn new(len: usize) -> Self {
        // `assert!`, not `debug_assert!`: an over-wide nonce would otherwise
        // panic on the first hand-out, far from the constructor that accepted
        // it — and only in release builds.
        assert!(
            len <= 24,
            "NonceCounter width must be <= 24 bytes, got {len}"
        );
        Self {
            buf: Zeroizing::new([0u8; 24]),
            len,
            handed_out: false,
        }
    }

    /// The nonce for the next AEAD operation, advancing the counter.
    #[must_use]
    pub fn next(&mut self) -> &[u8] {
        let nonce = &mut self.buf[..self.len];
        if self.handed_out {
            bump(nonce);
        }
        self.handed_out = true;
        nonce
    }
}

/// Little-endian increment over the whole buffer; the carry out of the top
/// byte is dropped, so the counter wraps.
fn bump(nonce: &mut [u8]) {
    for byte in nonce.iter_mut() {
        let (bumped, carry) = byte.overflowing_add(1);
        *byte = bumped;
        if !carry {
            break;
        }
    }
}

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
    /// nonce buffer. Guards the increment width (XChaCha's 24 bytes).
    #[test]
    fn nonce_counter_is_little_endian_and_wraps() {
        let mut c = NonceCounter::new(12);
        assert_eq!(c.next(), &[0u8; 12]);
        assert_eq!(c.next(), &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut c = NonceCounter::new(0);
        assert_eq!(c.next(), &[] as &[u8]);
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
            let _ = c.next();
        }
        assert_eq!(c.next(), &[0x00, 0x01]); // 256 as u16 LE
        let mut c = NonceCounter::new(1);
        for _ in 0..256 {
            let _ = c.next();
        }
        assert_eq!(c.next(), &[0x00]); // 256 truncated to one byte
    }
}
