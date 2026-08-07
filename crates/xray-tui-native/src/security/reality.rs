//! REALITY client stub — contract only (M3 milestone).
//!
//! Wire contract (modeled on `shoes/src/reality/`, MIT):
//! 1. X25519 keypair; `client_random` 32 B.
//! 2. `shared` = `ECDH(client_priv`, `server_pub` from `pbk`).
//! 3. `auth_key` = HKDF-SHA256(shared, salt = random[0..20], info b"REALITY").
//! 4. `SessionId` plaintext 16 B = version(1,8,0) + pad(1) + timestamp u32 BE + `short_id(8)`.
//! 5. `ClientHello`: X25519 keyshare, SNI steal target, suites 0x1301-03, ALPN h2+http/1.1.
//! 6. AAD = hello with `SessionId` zeroed; nonce = random[20..32]; `SessionId` =
//!    AES-256-GCM(auth_key, nonce, plaintext, aad).
//! 7. Restore `SessionId` (builder-returned range, never hardcoded offset).
//! 8. Server auth: cert signature field == HMAC-SHA512(auth_key, `ed25519_pub`),
//!    Ed25519 `CertificateVerify` over transcript. No PKI chain check.
//! 9. TLS1.3 key schedule + Finished, then app records.
//!
//! Crypto (ring): X25519 agreement, HKDF-SHA256, AES-256-GCM,
//! SHA-256/384/512 + HMAC, Ed25519 verify — full coverage, no aws-lc-rs.

use std::sync::Arc;

use crate::error::NativeError;

/// A `ClientHello` layed out for REALITY: fingerprint-shaped extensions that
/// the reality connector then injects keyshare/SNI/random/SessionId into.
pub trait HelloProvisioner: Send + Sync {
    /// Returns the bare `ClientHello` plus the `SessionId` byte range.
    /// The returned range is computed by the builder — never hardcoded.
    fn provision(&self, params: &HelloProvisionParams) -> Result<ProvisionedHello, NativeError>;
}

/// Inputs the reality connector hands to a provisioner.
#[derive(Debug, Clone)]
pub struct HelloProvisionParams {
    pub server_name: String,
    pub alpn: Vec<Vec<u8>>,
}

/// A provisioned `ClientHello` (handshake bytes, no record header).
#[derive(Debug, Clone)]
pub struct ProvisionedHello {
    pub handshake_bytes: Vec<u8>,
    /// Byte range of the `SessionId` field within `handshake_bytes`.
    pub session_id_range: std::ops::Range<usize>,
}

/// Fixed Chrome-133-shaped provisioner (first engine; ported from shoes).
pub struct FixedChrome133;

impl HelloProvisioner for FixedChrome133 {
    // Implemented in M3 with the reality ring port; M1 = unreachable stub.
    fn provision(&self, _params: &HelloProvisionParams) -> Result<ProvisionedHello, NativeError> {
        Err(NativeError::NotImplemented {
            feature: "reality hello provisioner".into(),
        })
    }
}

/// Chosen provisioner for a connect.
#[derive(Clone, Default)]
pub enum HelloProvisionerChoice {
    #[default]
    FixedChrome133,
    Custom(Arc<dyn HelloProvisioner>),
}
