//! Crypto primitives: TLS 1.3 key schedule, AEAD record keys, and X25519.
//!
//! Ring port of `thirdparty/tls-fingerprint/src/crypto.rs` (`aws-lc-rs` →
//! ring 0.17): same key-schedule flow (`EarlySecret` → `HandshakeSecret` →
//! `MasterSecret`, client/server handshake + application traffic secrets),
//! same `HKDF-Expand-Label` encoding, and the same AEAD record-key shape.
//! PRKs are passed around as raw bytes (ring hides them inside `Prk`), so
//! every derived value is directly observable and testable against the
//! RFC 8448 vectors.

pub mod fingerprint;

use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey},
    digest,
    hkdf::{self, Prk},
    hmac,
};
use x25519_dalek::StaticSecret;

use crate::error::{Result, TlsError};

// ── Cipher suite identifiers ───────────────────────────────────────────────

/// The TLS 1.3 cipher suites this engine can speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherSuiteId {
    /// `TLS_AES_128_GCM_SHA256` (0x1301).
    Aes128GcmSha256,
    /// `TLS_AES_256_GCM_SHA384` (0x1302).
    Aes256GcmSha384,
    /// `TLS_CHACHA20_POLY1305_SHA256` (0x1303).
    Chacha20Poly1305Sha256,
}

impl CipherSuiteId {
    /// Maps a wire cipher-suite value to a variant.
    #[must_use]
    pub const fn from_u16(v: u16) -> Option<Self> {
        match v {
            0x1301 => Some(Self::Aes128GcmSha256),
            0x1302 => Some(Self::Aes256GcmSha384),
            0x1303 => Some(Self::Chacha20Poly1305Sha256),
            _ => None,
        }
    }

    /// The wire value of this cipher suite.
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::Aes128GcmSha256 => 0x1301,
            Self::Aes256GcmSha384 => 0x1302,
            Self::Chacha20Poly1305Sha256 => 0x1303,
        }
    }

    /// The transcript hash algorithm for this suite.
    #[must_use]
    pub const fn digest(self) -> &'static digest::Algorithm {
        match self {
            Self::Aes128GcmSha256 | Self::Chacha20Poly1305Sha256 => &digest::SHA256,
            Self::Aes256GcmSha384 => &digest::SHA384,
        }
    }

    /// The AEAD algorithm for this suite.
    #[must_use]
    pub const fn aead(self) -> &'static ring::aead::Algorithm {
        match self {
            Self::Aes128GcmSha256 => &ring::aead::AES_128_GCM,
            Self::Aes256GcmSha384 => &ring::aead::AES_256_GCM,
            Self::Chacha20Poly1305Sha256 => &ring::aead::CHACHA20_POLY1305,
        }
    }

    /// AEAD key length in bytes.
    #[must_use]
    pub const fn key_len(self) -> usize {
        match self {
            Self::Aes128GcmSha256 => 16,
            Self::Aes256GcmSha384 | Self::Chacha20Poly1305Sha256 => 32,
        }
    }

    /// Hash length in bytes.
    #[must_use]
    pub const fn hash_len(self) -> usize {
        match self {
            Self::Aes128GcmSha256 | Self::Chacha20Poly1305Sha256 => 32,
            Self::Aes256GcmSha384 => 48,
        }
    }

    /// The HKDF algorithm underlying this suite's hash.
    const fn hkdf_alg(self) -> hkdf::Algorithm {
        match self {
            Self::Aes128GcmSha256 | Self::Chacha20Poly1305Sha256 => hkdf::HKDF_SHA256,
            Self::Aes256GcmSha384 => hkdf::HKDF_SHA384,
        }
    }

    /// The HMAC algorithm underlying this suite's hash.
    const fn hmac_alg(self) -> hmac::Algorithm {
        match self {
            Self::Aes128GcmSha256 | Self::Chacha20Poly1305Sha256 => hmac::HMAC_SHA256,
            Self::Aes256GcmSha384 => hmac::HMAC_SHA384,
        }
    }
}

// ── X25519 key pair ────────────────────────────────────────────────────────

/// An ephemeral X25519 key pair.
///
/// The private half lives in `x25519-dalek` (`StaticSecret`): ring 0.17's
/// `EphemeralPrivateKey` is single-use (agreement consumes it) and cannot be
/// serialized, but REALITY must agree **twice** with the same scalar — once
/// with the server's static `pbk` for the auth key, once with the server's
/// ephemeral keyshare for the TLS 1.3 key schedule (xtls/reality 2025-10+
/// no longer uses the static key as the TLS keyshare). Agreement is
/// therefore repeatable.
pub struct X25519KeyPair {
    private: StaticSecret,
    public_key: [u8; 32],
}

impl X25519KeyPair {
    /// Generates a fresh key pair using `rng`.
    pub fn generate(rng: &dyn ring::rand::SecureRandom) -> Result<Self> {
        let mut seed = [0u8; 32];
        rng.fill(&mut seed)
            .map_err(|_| TlsError::Crypto("X25519 seed generation failed".into()))?;
        Ok(Self::from_seed(seed))
    }

    /// Builds a key pair from a raw 32-byte private seed (RFC 7748 vectors,
    /// deterministic tests). The seed is clamped at use, like both cores.
    #[must_use]
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let private = StaticSecret::from(seed);
        let public = x25519_dalek::PublicKey::from(&private);
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(public.as_bytes());
        Self {
            private,
            public_key,
        }
    }

    /// The 32-byte X25519 public key.
    #[must_use]
    pub const fn public_key(&self) -> [u8; 32] {
        self.public_key
    }

    /// Computes the 32-byte ECDHE shared secret with `peer`. Repeatable —
    /// REALITY agrees with the static `pbk` and the server keyshare.
    pub fn agree(&self, peer: &[u8; 32]) -> Result<[u8; 32]> {
        let peer = x25519_dalek::PublicKey::from(*peer);
        let shared = self.private.diffie_hellman(&peer);
        let mut out = [0u8; 32];
        out.copy_from_slice(shared.as_bytes());
        Ok(out)
    }
}

// ── HKDF helpers ───────────────────────────────────────────────────────────

/// Output length for `Prk::expand` — required by ring's `KeyType`.
struct ExpandLen(usize);

impl hkdf::KeyType for ExpandLen {
    fn len(&self) -> usize {
        self.0
    }
}

/// Hash of the empty string for `suite`, used as the context of the
/// `"derived"` expansion steps (RFC 8446 §7.1).
#[must_use]
pub fn empty_hash(suite: CipherSuiteId) -> Vec<u8> {
    transcript_digest(suite, &[])
}

/// Transcript-Hash of `data` for `suite`.
fn transcript_digest(suite: CipherSuiteId, data: &[u8]) -> Vec<u8> {
    digest::digest(suite.digest(), data).as_ref().to_vec()
}

// ── TLS 1.3 key schedule ───────────────────────────────────────────────────

/// Incremental TLS 1.3 key schedule.
///
/// Mirrors the `tls-fingerprint` flow: `new` seeds the `EarlySecret`, then
/// `handshake_secret` → `handshake_traffic_secrets`, then `master_secret` →
/// `app_traffic_secrets`. `add_transcript` feeds handshake messages as they
/// arrive; `derive_secret` hashes whatever transcript has accumulated.
pub struct KeySchedule {
    suite: CipherSuiteId,
    transcript: Vec<u8>,
}

impl KeySchedule {
    /// A fresh schedule for `suite` with an empty transcript.
    #[must_use]
    pub const fn new(suite: CipherSuiteId) -> Self {
        Self {
            suite,
            transcript: Vec::new(),
        }
    }

    /// Appends one handshake message to the transcript.
    pub fn add_transcript(&mut self, hs_msg: &[u8]) {
        self.transcript.extend_from_slice(hs_msg);
    }

    /// Transcript-Hash of everything added so far.
    #[must_use]
    pub fn transcript_hash(&self) -> Vec<u8> {
        transcript_digest(self.suite, &self.transcript)
    }

    /// HKDF-Extract(salt, ikm) → PRK bytes.
    ///
    /// HKDF-Extract(salt, ikm) = HMAC(salt, ikm); ring's `Prk` hides its
    /// bytes, so the HMAC is computed directly — identical to what ring's
    /// `Salt::extract` does internally.
    pub fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let key = hmac::Key::new(self.suite.hmac_alg(), salt);
        Ok(hmac::sign(&key, ikm).as_ref().to_vec())
    }

    /// HKDF-Expand-Label(prk, label, ctx, len) per RFC 8446 §7.1.
    pub fn hkdf_expand_label(
        &self,
        prk: &[u8],
        label: &str,
        ctx: &[u8],
        len: usize,
    ) -> Result<Vec<u8>> {
        let len16 = u16::try_from(len)
            .map_err(|_| TlsError::Crypto("HKDF-Expand-Label length too large".into()))?;
        let full_label = format!("tls13 {label}");
        let label_bytes = full_label.as_bytes();

        // HkdfLabel: uint16 length || uint8 label_len || label ||
        //            uint8 ctx_len || ctx
        let mut info = Vec::with_capacity(2 + 1 + label_bytes.len() + 1 + ctx.len());
        info.extend_from_slice(&len16.to_be_bytes());
        info.push(
            u8::try_from(label_bytes.len())
                .map_err(|_| TlsError::Crypto("HKDF-Expand-Label label too long".into()))?,
        );
        info.extend_from_slice(label_bytes);
        info.push(
            u8::try_from(ctx.len())
                .map_err(|_| TlsError::Crypto("HKDF-Expand-Label context too long".into()))?,
        );
        info.extend_from_slice(ctx);

        let prk = Prk::new_less_safe(self.suite.hkdf_alg(), prk);
        let info_slice: &[u8] = &info;
        let info_arr = [info_slice];
        let okm = prk
            .expand(&info_arr, ExpandLen(len))
            .map_err(|_| TlsError::Crypto(format!("HKDF-Expand-Label({label}) failed")))?;
        let mut out = vec![0u8; len];
        okm.fill(&mut out)
            .map_err(|_| TlsError::Crypto("HKDF fill failed".into()))?;
        Ok(out)
    }

    /// `Derive-Secret(prk, label)` = `HKDF-Expand-Label(prk, label,
    /// Hash(transcript), hash_len)`.
    pub fn derive_secret(&self, prk: &[u8], label: &str) -> Result<Vec<u8>> {
        let h = self.transcript_hash();
        self.hkdf_expand_label(prk, label, &h, self.suite.hash_len())
    }

    /// `handshake_secret` = `HKDF-Extract(Derive-Secret(early_secret,
    /// "derived", ""), shared_secret)`.
    pub fn handshake_secret(&self, shared_secret: &[u8]) -> Result<Vec<u8>> {
        let hash_len = self.suite.hash_len();
        let zeros = vec![0u8; hash_len];
        let early = self.hkdf_extract(&zeros, &zeros)?;
        let derived = self.hkdf_expand_label(&early, "derived", &empty_hash(self.suite), hash_len)?;
        self.hkdf_extract(&derived, shared_secret)
    }

    /// `(c hs traffic, s hs traffic)` from the handshake secret and the
    /// accumulated CH..SH transcript.
    pub fn handshake_traffic_secrets(&self, hs_secret: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let c = self.derive_secret(hs_secret, "c hs traffic")?;
        let s = self.derive_secret(hs_secret, "s hs traffic")?;
        Ok((c, s))
    }

    /// `master_secret` = `HKDF-Extract(Derive-Secret(hs_secret, "derived",
    /// ""), 0^hash_len)`.
    pub fn master_secret(&self, hs_secret: &[u8]) -> Result<Vec<u8>> {
        let hash_len = self.suite.hash_len();
        let derived = self.hkdf_expand_label(hs_secret, "derived", &empty_hash(self.suite), hash_len)?;
        let zeros = vec![0u8; hash_len];
        self.hkdf_extract(&derived, &zeros)
    }

    /// `(c ap traffic, s ap traffic)` from the master secret and the
    /// accumulated CH..server Finished transcript.
    pub fn app_traffic_secrets(&self, master: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let c = self.derive_secret(master, "c ap traffic")?;
        let s = self.derive_secret(master, "s ap traffic")?;
        Ok((c, s))
    }

    /// `Finished` key = `HKDF-Expand-Label(traffic_secret, "finished", "",
    /// hash_len)`.
    pub fn finished_key(&self, traffic_secret: &[u8]) -> Result<Vec<u8>> {
        self.hkdf_expand_label(traffic_secret, "finished", &[], self.suite.hash_len())
    }

    /// `Finished` `verify_data` = `HMAC(finished_key,
    /// Transcript-Hash(transcript))`.
    #[must_use]
    pub fn finished_mac(&self, finished_key: &[u8]) -> Vec<u8> {
        let th = self.transcript_hash();
        let key = hmac::Key::new(self.suite.hmac_alg(), finished_key);
        hmac::sign(&key, &th).as_ref().to_vec()
    }
}

// ── AEAD record keys ───────────────────────────────────────────────────────

/// An AEAD record key; the per-record nonce is
/// `write_iv XOR sequence number` (RFC 8446 §5.3).
#[derive(Clone)]
pub struct AeadKey {
    key: LessSafeKey,
    iv: [u8; 12],
}

impl AeadKey {
    /// Derives a record key and write IV from a traffic secret:
    /// `HKDF-Expand-Label(secret, "key", "", key_len)` and
    /// `HKDF-Expand-Label(secret, "iv", "", 12)`.
    pub fn new(suite: CipherSuiteId, secret: &[u8]) -> Result<Self> {
        let ks = KeySchedule::new(suite);
        let key_bytes = ks.hkdf_expand_label(secret, "key", &[], suite.key_len())?;
        let iv_vec = ks.hkdf_expand_label(secret, "iv", &[], 12)?;
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_vec);
        Self::from_key_iv(suite, &key_bytes, iv)
    }

    /// Builds a record key directly from raw key bytes with a zero IV
    /// (same material → identical keys; for tests and raw-material paths).
    pub fn from_key_bytes(suite: CipherSuiteId, key_bytes: &[u8]) -> Result<Self> {
        Self::from_key_iv(suite, key_bytes, [0u8; 12])
    }

    /// Returns a key with identical key material (same key bytes, same IV);
    /// used when both directions of a stream share one key, e.g. tests.
    #[must_use]
    pub fn clone_key(&self) -> Self {
        Self {
            key: self.key.clone(),
            iv: self.iv,
        }
    }

    /// Builds a record key from raw key bytes and an explicit write IV.
    pub fn from_key_iv(suite: CipherSuiteId, key_bytes: &[u8], iv: [u8; 12]) -> Result<Self> {
        let unbound = UnboundKey::new(suite.aead(), key_bytes)
            .map_err(|_| TlsError::Crypto("failed to create AEAD key".into()))?;
        Ok(Self {
            key: LessSafeKey::new(unbound),
            iv,
        })
    }

    /// TLS 1.3 record nonce = `write_iv` XOR the sequence number encoded as
    /// a right-aligned 12-byte big-endian value (RFC 8446 §5.3).
    fn make_nonce(&self, seq: u64) -> [u8; 12] {
        let mut n = self.iv;
        let seq_bytes = seq.to_be_bytes();
        for i in 0..8 {
            n[4 + i] ^= seq_bytes[i];
        }
        n
    }

    /// Encrypts `plaintext` under `(seq, aad)`, appending the AEAD tag.
    pub fn seal(&self, seq: u64, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key(self.make_nonce(seq));
        let mut in_out = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::from(aad), &mut in_out)
            .map_err(|_| TlsError::Crypto("AEAD seal failed".into()))?;
        Ok(in_out)
    }

    /// Decrypts and authenticates `ciphertext` under `(seq, aad)`.
    pub fn open(&self, seq: u64, aad: &[u8], ciphertext: &mut [u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key(self.make_nonce(seq));
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::from(aad), ciphertext)
            .map_err(|_| {
                TlsError::Crypto("AEAD open failed — wrong key or corrupted record".into())
            })?;
        Ok(plaintext.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{empty_hash, AeadKey, CipherSuiteId, KeySchedule, X25519KeyPair};

    /// Decodes a hex string into bytes (test helper).
    fn decode_hex(s: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(s.len() / 2);
        let mut hi = None;
        for b in s.bytes() {
            let v = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => panic!("invalid hex digit: {b:#x}"),
            };
            match hi {
                None => hi = Some(v),
                Some(h) => {
                    out.push((h << 4) | v);
                    hi = None;
                }
            }
        }
        assert_eq!(hi, None, "hex string must have even length");
        out
    }

    /// `decode_hex` then hex-encode back (assert helper).
    fn hex(bytes: &[u8]) -> String {
        use core::fmt::Write;
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            let _ = write!(s, "{b:02x}");
        }
        s
    }

    // RFC 8448 §3 vectors — TLS13-AES-128-GCM-SHA256, simple 1-RTT
    // handshake. Message payloads are the handshake-message bytes (the
    // record layer is stripped); secrets are the published values.

    const CLIENT_HELLO: &str = concat!(
        "010000c00303cb34ecb1e78163ba1c38c6dacb196a6dffa21a8d9912ec18a2ef6283024dece700000613011303130201",
        "0000910000000b0009000006736572766572ff01000100000a00140012001d0017001800190100010101020103010400",
        "230000003300260024001d002099381de560e4bd43d23d8e435a7dbafeb3c06e51c13cae4d5413691e529aaf2c002b00",
        "03020304000d0020001e040305030603020308040805080604010501060102010402050206020202002d00020101001c",
        "00024001",
    );
    const SERVER_HELLO: &str = concat!(
        "020000560303a6af06a4121860dc5e6e60249cd34c95930c8ac5cb1434dac155772ed3e2692800130100002e00330024",
        "001d0020c9828876112095fe66762bdbf7c672e156d6cc253b833df1dd69b1b04e751f0f002b00020304",
    );
    const ENCRYPTED_EXTENSIONS: &str =
        "080000240022000a00140012001d00170018001901000101010201030104001c0002400100000000";
    const CERTIFICATE: &str = concat!(
        "0b0001b9000001b50001b0308201ac30820115a003020102020102300d06092a864886f70d01010b0500300e310c300a",
        "06035504031303727361301e170d3136303733303031323335395a170d3236303733303031323335395a300e310c300a",
        "0603550403130372736130819f300d06092a864886f70d010101050003818d0030818902818100b4bb498f8279303d98",
        "0836399b36c6988c0c68de55e1bdb826d3901a2461eafd2de49a91d015abbc9a95137ace6c1af19eaa6af98c7ced4312",
        "0998e187a80ee0ccb0524b1b018c3e0b63264d449a6d38e22a5fda430846748030530ef0461c8ca9d9efbfae8ea6d1d0",
        "3e2bd193eff0ab9a8002c47428a6d35a8d88d79f7f1e3f0203010001a31a301830090603551d1304023000300b060355",
        "1d0f0404030205a0300d06092a864886f70d01010b05000381810085aad2a0e5b9276b908c65f73a7267170618a54c5f",
        "8a7b337d2df7a594365417f2eae8f8a58c8f8172f9319cf36b7fd6c55b80f21a03015156726096fd335e5e67f2dbf102",
        "702e608ccae6bec1fc63a42a99be5c3eb7107c3c54e9b9eb2bd5203b1c3b84e0a8b2f759409ba3eac9d91d402dcc0cc8",
        "f8961229ac9187b42b4de10000",
    );
    const CERTIFICATE_VERIFY: &str = concat!(
        "0f000084080400805a747c5d88fa9bd2e55ab085a61015b7211f824cd484145ab3ff52f1fda8477b0b7abc90db78e2d3",
        "3a5c141a078653fa6bef780c5ea248eeaaa785c4f394cab6d30bbe8d4859ee511f602957b15411ac027671459e46445c",
        "9ea58c181e818e95b8c3fb0bf3278409d3be152a3da5043e063dda65cdf5aea20d53dfacd42f74f3",
    );
    const SERVER_FINISHED: &str =
        "140000209b9b141d906337fbd2cbdce71df4deda4ab42c309572cb7fffee5454b78f0718";

    const EARLY_SECRET: &str = "33ad0a1c607ec03b09e6cd9893680ce210adf300aa1f2660e1b22e10f170f92a";
    const DERIVED_FROM_EARLY: &str =
        "6f2615a108c702c5678f54fc9dbab69716c076189c48250cebeac3576c3611ba";
    const ECDHE_SHARED_SECRET: &str =
        "8bd4054fb55b9d63fdfbacf9f04b9f0d35e6d63f537563efd46272900f89492d";
    const HANDSHAKE_SECRET: &str =
        "1dc826e93606aa6fdc0aadc12f741b01046aa6b99f691ed221a9f0ca043fbeac";
    const TRANSCRIPT_CH_SH: &str =
        "860c06edc07858ee8e78f0e7428c58edd6b43f2ca3e6e95f02ed063cf0e1cad8";
    const CLIENT_HS_TRAFFIC: &str =
        "b3eddb126e067f35a780b3abf45e2d8f3b1a950738f52e9600746a0e27a55a21";
    const SERVER_HS_TRAFFIC: &str =
        "b67b7d690cc16c4e75e54213cb2d37b4e9c912bcded9105d42befd59d391ad38";
    const SERVER_FINISHED_KEY: &str =
        "008d3b66f816ea559f96b537e885c31fc068bf492c652f01f288a1d8cdc19fc8";
    const SERVER_FINISHED_VERIFY: &str =
        "9b9b141d906337fbd2cbdce71df4deda4ab42c309572cb7fffee5454b78f0718";
    const DERIVED_FROM_HS: &str =
        "43de77e0c77713859a944db9db2590b53190a65b3ee2e4f12dd7a0bb7ce254b4";
    const MASTER_SECRET: &str =
        "18df06843d13a08bf2a449844c5f8a478001bc4d4c627984d5a41da8d0402919";
    const TRANSCRIPT_CH_FINISHED: &str =
        "9608102a0f1ccc6db6250b7b7e417b1a000eaada3daae4777a7686c9ff83df13";
    const CLIENT_AP_TRAFFIC: &str =
        "9e40646ce79a7f9dc05af8889bce6552875afa0b06df0087f792ebb7c17504a5";
    const SERVER_AP_TRAFFIC: &str =
        "a11af9f05531f856ad47116b45a950328204b4f44bfb6b3a4b4f1f3fcb631643";
    const EXPORTER_MASTER: &str =
        "fe22f881176eda18eb8f44529e6792c50c9a3f89452f68d8ae311b4309d3cf50";
    const SERVER_AP_KEY: &str = "9f02283b6c9c07efc26bb9f2ac92e356";
    const CLIENT_HS_KEY: &str = "dbfaa693d1762c5b666af5d950258d01";

    /// The full RFC 8448 §3 key schedule: early → handshake → master and
    /// the handshake/application traffic secrets.
    #[test]
    fn hkdf_expand_label_matches_rfc8448() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let mut ks = KeySchedule::new(suite);

        // early_secret = HKDF-Extract(0^32, 0^32).
        let zeros = [0u8; 32];
        let early = ks.hkdf_extract(&zeros, &zeros).unwrap();
        assert_eq!(hex(&early), EARLY_SECRET);

        // Derive-Secret(early, "derived", "") with Hash("") as context.
        let derived = ks
            .hkdf_expand_label(&early, "derived", &empty_hash(suite), 32)
            .unwrap();
        assert_eq!(hex(&derived), DERIVED_FROM_EARLY);

        // handshake_secret = HKDF-Extract(derived, ecdhe).
        let ecdhe = decode_hex(ECDHE_SHARED_SECRET);
        let hs = ks.hkdf_extract(&derived, &ecdhe).unwrap();
        assert_eq!(hex(&hs), HANDSHAKE_SECRET);

        // Transcript CH..SH feeds the handshake traffic secrets.
        ks.add_transcript(&decode_hex(CLIENT_HELLO));
        ks.add_transcript(&decode_hex(SERVER_HELLO));
        assert_eq!(hex(&ks.transcript_hash()), TRANSCRIPT_CH_SH);

        let (c_hs, s_hs) = ks.handshake_traffic_secrets(&hs).unwrap();
        assert_eq!(hex(&c_hs), CLIENT_HS_TRAFFIC);
        assert_eq!(hex(&s_hs), SERVER_HS_TRAFFIC);

        // The intermediate "derived" expansion the master step consumes.
        let derived_hs = ks
            .hkdf_expand_label(&hs, "derived", &empty_hash(suite), 32)
            .unwrap();
        assert_eq!(hex(&derived_hs), DERIVED_FROM_HS);

        // master_secret.
        let master = ks.master_secret(&hs).unwrap();
        assert_eq!(hex(&master), MASTER_SECRET);

        // Transcript CH..server Finished feeds the app traffic secrets.
        ks.add_transcript(&decode_hex(ENCRYPTED_EXTENSIONS));
        ks.add_transcript(&decode_hex(CERTIFICATE));
        ks.add_transcript(&decode_hex(CERTIFICATE_VERIFY));
        ks.add_transcript(&decode_hex(SERVER_FINISHED));
        assert_eq!(hex(&ks.transcript_hash()), TRANSCRIPT_CH_FINISHED);

        let (c_ap, s_ap) = ks.app_traffic_secrets(&master).unwrap();
        assert_eq!(hex(&c_ap), CLIENT_AP_TRAFFIC);
        assert_eq!(hex(&s_ap), SERVER_AP_TRAFFIC);

        // Server application write key and exporter master secret.
        let s_ap_key = ks.hkdf_expand_label(&s_ap, "key", &[], 16).unwrap();
        assert_eq!(hex(&s_ap_key), SERVER_AP_KEY);
        let exp_master = ks
            .hkdf_expand_label(&master, "exp master", &ks.transcript_hash(), 32)
            .unwrap();
        assert_eq!(hex(&exp_master), EXPORTER_MASTER);

        // Hash("") — the empty_hash the "derived" steps use.
        assert_eq!(
            hex(&empty_hash(suite)),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// HKDF-Expand-Label known answers from RFC 8448 (key/finished).
    #[test]
    fn expand_label_known_answer() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let ks = KeySchedule::new(suite);

        // finished_key = HKDF-Expand-Label(s_hs_traffic, "finished", "", 32).
        let s_hs = decode_hex(SERVER_HS_TRAFFIC);
        let finished_key = ks.hkdf_expand_label(&s_hs, "finished", &[], 32).unwrap();
        assert_eq!(hex(&finished_key), SERVER_FINISHED_KEY);

        // client handshake write key = HKDF-Expand-Label(c_hs_traffic, "key", "", 16).
        let c_hs = decode_hex(CLIENT_HS_TRAFFIC);
        let key = ks.hkdf_expand_label(&c_hs, "key", &[], 16).unwrap();
        assert_eq!(hex(&key), CLIENT_HS_KEY);
        // client handshake write IV = HKDF-Expand-Label(c_hs_traffic, "iv", "", 12).
        let iv = ks.hkdf_expand_label(&c_hs, "iv", &[], 12).unwrap();
        assert_eq!(hex(&iv), "5bd3c71b836e0b76bb73265f");
    }

    /// Server `Finished` `verify_data` = `HMAC(finished_key, Hash(CH..CV))`.
    #[test]
    fn finished_matches_rfc8448() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let mut ks = KeySchedule::new(suite);
        ks.add_transcript(&decode_hex(CLIENT_HELLO));
        ks.add_transcript(&decode_hex(SERVER_HELLO));
        ks.add_transcript(&decode_hex(ENCRYPTED_EXTENSIONS));
        ks.add_transcript(&decode_hex(CERTIFICATE));
        ks.add_transcript(&decode_hex(CERTIFICATE_VERIFY));

        let finished_key = ks.finished_key(&decode_hex(SERVER_HS_TRAFFIC)).unwrap();
        assert_eq!(hex(&finished_key), SERVER_FINISHED_KEY);
        assert_eq!(hex(&ks.finished_mac(&finished_key)), SERVER_FINISHED_VERIFY);
    }

    /// AEAD seal/open round-trip with a fixed key (brief's test).
    #[test]
    fn aead_roundtrip() {
        let key = AeadKey::new(CipherSuiteId::Aes128GcmSha256, &[0x11; 16]).unwrap();
        let mut ct = key
            .seal(0, &[0x17, 0x03, 0x03, 0x00, 0x10], b"hello world")
            .unwrap();
        let pt = key
            .open(0, &[0x17, 0x03, 0x03, 0x00, 0x10], &mut ct)
            .unwrap();
        assert_eq!(pt, b"hello world");
    }

    /// Keys built from the same material seal identically; raw key bytes
    /// work through `from_key_bytes` (Task 7 needs two keys, one object).
    #[test]
    fn aead_same_material_matches() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let a = AeadKey::new(suite, &[0x11; 16]).unwrap();
        let b = AeadKey::new(suite, &[0x11; 16]).unwrap();
        let ct_a = a.seal(7, b"aad-test", b"payload").unwrap();
        let ct_b = b.seal(7, b"aad-test", b"payload").unwrap();
        assert_eq!(ct_a, ct_b);

        let c = AeadKey::from_key_bytes(suite, &[0x11; 16]).unwrap();
        let d = AeadKey::from_key_bytes(suite, &[0x11; 16]).unwrap();
        let mut ct_c = c.seal(3, b"aad", b"payload").unwrap();
        assert_eq!(d.open(3, b"aad", &mut ct_c).unwrap(), b"payload");
    }

    /// `new` (traffic-secret derivation) and `from_key_iv` (explicit key +
    /// IV from the same expansions) build interoperable keys.
    #[test]
    fn new_matches_from_key_iv() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let secret = [0x42; 32];
        let ks = KeySchedule::new(suite);
        let key_bytes = ks.hkdf_expand_label(&secret, "key", &[], 16).unwrap();
        let iv_vec = ks.hkdf_expand_label(&secret, "iv", &[], 12).unwrap();
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_vec);

        let a = AeadKey::new(suite, &secret).unwrap();
        let b = AeadKey::from_key_iv(suite, &key_bytes, iv).unwrap();
        let mut ct = a.seal(5, b"aad", b"payload").unwrap();
        assert_eq!(b.open(5, b"aad", &mut ct).unwrap(), b"payload");
    }

    /// Nonce = `write_iv` XOR right-aligned 12-byte big-endian seq (RFC 8446
    /// §5.3, reference `make_nonce` formula).
    #[test]
    fn nonce_is_iv_xor_seq() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let iv = [0x5d, 0x31, 0x3e, 0xb2, 0x67, 0x12, 0x76, 0xee, 0x13, 0x00, 0x0b, 0x30];
        let key = AeadKey::from_key_iv(suite, &[0x11; 16], iv).unwrap();

        assert_eq!(key.make_nonce(0), iv);
        let mut expected = iv;
        for i in 0..8 {
            expected[4 + i] ^= (1u64).to_be_bytes()[i];
        }
        assert_eq!(key.make_nonce(1), expected);
        // seq 2^32 must flip the first nonce octet of the 8-byte field.
        let mut expected = iv;
        let seq = 1u64 << 32;
        for i in 0..8 {
            expected[4 + i] ^= seq.to_be_bytes()[i];
        }
        assert_eq!(key.make_nonce(seq), expected);
    }

    /// A different key or sequence number fails authentication.
    #[test]
    fn aead_wrong_key_fails() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let key = AeadKey::from_key_bytes(suite, &[0x11; 16]).unwrap();
        let other = AeadKey::from_key_bytes(suite, &[0x22; 16]).unwrap();
        let mut ct = key.seal(0, b"aad", b"secret").unwrap();
        assert!(other.open(0, b"aad", &mut ct).is_err());
        assert!(key.open(1, b"aad", &mut ct).is_err());
    }

    /// `CHACHA20_POLY1305` round-trip sanity.
    #[test]
    fn aead_chacha_roundtrip() {
        let key =
            AeadKey::from_key_bytes(CipherSuiteId::Chacha20Poly1305Sha256, &[0x33; 32]).unwrap();
        let mut ct = key.seal(0, b"aad", b"chacha payload").unwrap();
        assert_eq!(key.open(0, b"aad", &mut ct).unwrap(), b"chacha payload");
    }

    /// Two fresh X25519 pairs agree on the same shared secret.
    #[test]
    fn x25519_agrees() {
        let a = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
        let b = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
        let sa = a.agree(&b.public_key()).unwrap();
        let sb = b.agree(&a.public_key()).unwrap();
        assert_eq!(sa, sb);
        assert_eq!(a.public_key().len(), 32);
    }

    /// Agreement is repeatable: REALITY agrees with the static pbk AND the
    /// server's ephemeral keyshare from the same pair.
    #[test]
    fn x25519_agree_is_repeatable() {
        let a = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
        let b = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
        let c = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).unwrap();
        let ab = a.agree(&b.public_key()).unwrap();
        let ac = a.agree(&c.public_key()).unwrap();
        assert_eq!(a.agree(&b.public_key()).unwrap(), ab);
        assert_ne!(ab, ac);
    }

    /// RFC 7748 §6.1: the fixed Alice scalar derives Bob's known shared
    /// secret (ring cannot push raw scalars; x25519-dalek can).
    #[test]
    fn x25519_rfc7748_alice_vector() {
        let alice_seed: [u8; 32] = [
            0x77, 0x07, 0x6d, 0x0a, 0x73, 0x18, 0xa5, 0x7d, 0x3c, 0x16, 0xc1, 0x72, 0x51, 0xb2,
            0x66, 0x45, 0xdf, 0x4c, 0x2f, 0x87, 0xeb, 0xc0, 0x99, 0x2a, 0xb1, 0x77, 0xfb, 0xa5,
            0x1d, 0xb9, 0x2c, 0x2a,
        ];
        let bob_pub: [u8; 32] = [
            0xde, 0x9e, 0xdb, 0x7d, 0x7b, 0x7d, 0xc1, 0xb4, 0xd3, 0x5b, 0x61, 0xc2, 0xec, 0xe4,
            0x35, 0x37, 0x3f, 0x83, 0x43, 0xc8, 0x5b, 0x78, 0x67, 0x4d, 0xad, 0xfc, 0x7e, 0x14,
            0x6f, 0x88, 0x2b, 0x4f,
        ];
        let alice = X25519KeyPair::from_seed(alice_seed);
        assert_eq!(
            alice.public_key().to_vec(),
            vec![
                0x85, 0x20, 0xf0, 0x09, 0x89, 0x30, 0xa7, 0x54, 0x74, 0x8b, 0x7d, 0xdc, 0xb4,
                0x3e, 0xf7, 0x5a, 0x0d, 0xbf, 0x3a, 0x0d, 0x26, 0x38, 0x1a, 0xf4, 0xeb, 0xa4,
                0xa9, 0x8e, 0xaa, 0x9b, 0x4e, 0x6a,
            ]
        );
        let shared = alice.agree(&bob_pub).unwrap();
        assert_eq!(
            shared.to_vec(),
            vec![
                0x4a, 0x5d, 0x9d, 0x5b, 0xa4, 0xce, 0x2d, 0xe1, 0x72, 0x8e, 0x3b, 0xf4, 0x80,
                0x35, 0x0f, 0x25, 0xe0, 0x7e, 0x21, 0xc9, 0x47, 0xd1, 0x9e, 0x33, 0x76, 0xf0,
                0x9b, 0x3c, 0x1e, 0x16, 0x17, 0x42,
            ]
        );
    }

    /// Cipher-suite wire mapping and lengths.
    #[test]
    fn cipher_suite_ids() {
        assert_eq!(
            CipherSuiteId::from_u16(0x1301),
            Some(CipherSuiteId::Aes128GcmSha256)
        );
        assert_eq!(
            CipherSuiteId::from_u16(0x1302),
            Some(CipherSuiteId::Aes256GcmSha384)
        );
        assert_eq!(
            CipherSuiteId::from_u16(0x1303),
            Some(CipherSuiteId::Chacha20Poly1305Sha256)
        );
        assert_eq!(CipherSuiteId::from_u16(0x1304), None);
        assert_eq!(CipherSuiteId::Aes128GcmSha256.to_u16(), 0x1301);
        assert_eq!(CipherSuiteId::Aes256GcmSha384.to_u16(), 0x1302);
        assert_eq!(CipherSuiteId::Chacha20Poly1305Sha256.to_u16(), 0x1303);
        assert_eq!(CipherSuiteId::Aes128GcmSha256.key_len(), 16);
        assert_eq!(CipherSuiteId::Aes256GcmSha384.key_len(), 32);
        assert_eq!(CipherSuiteId::Chacha20Poly1305Sha256.key_len(), 32);
        assert_eq!(CipherSuiteId::Aes128GcmSha256.hash_len(), 32);
        assert_eq!(CipherSuiteId::Aes256GcmSha384.hash_len(), 48);
        assert_eq!(CipherSuiteId::Chacha20Poly1305Sha256.hash_len(), 32);
    }
}
