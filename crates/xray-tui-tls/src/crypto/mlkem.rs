//! ML-KEM-768 primitives via liboqs.
//!
//! This module wraps `oqs::kem::Algorithm::MlKem768` (ML-KEM-768 / FIPS 203) to provide
//! keypair generation, encapsulation, and decapsulation for TLS 1.3 hybrid
//! curves (`X25519MLKEM768`, `SecP256r1MLKEM768`) and VLESS encryption
//! (`mlkem768x25519plus`).
//!
//! Key sizes (FIPS 203 / RFC 9180):
//! - Public key: 1184 bytes
//! - Secret key: 2400 bytes
//! - Ciphertext: 1088 bytes
//! - Shared secret: 32 bytes
use oqs::kem::{Algorithm, Kem};
use thiserror::Error;
/// ML-KEM-768 public key (1184 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(Vec<u8>);

/// ML-KEM-768 secret key (2400 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(Vec<u8>);

/// ML-KEM-768 ciphertext (1088 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

/// ML-KEM-768 shared secret (32 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecret([u8; 32]);

/// Errors from ML-KEM operations.
#[derive(Debug, Error)]
pub enum MlkemError {
    #[error("keypair generation failed")]
    KeygenFailed,
    #[error("encapsulation failed")]
    EncapsulateFailed,
    #[error("decapsulation failed")]
    DecapsulateFailed,
    #[error("invalid public key length: expected 1184, got {0}")]
    InvalidPublicKeyLength(usize),
    #[error("invalid secret key length: expected 2400, got {0}")]
    InvalidSecretKeyLength(usize),
    #[error("invalid ciphertext length: expected 1088, got {0}")]
    InvalidCiphertextLength(usize),
    #[error("invalid shared secret length: expected 32, got {0}")]
    InvalidSharedSecretLength(usize),
    #[error("liboqs not available: {0}")]
    LiboqsUnavailable(String),
}

impl PublicKey {
    /// Create a [`PublicKey`] from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 1184 {
            return Err(MlkemError::InvalidPublicKeyLength(bytes.len()));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Get the raw bytes of the public key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl SecretKey {
    /// Create a [`SecretKey`] from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 2400 {
            return Err(MlkemError::InvalidSecretKeyLength(bytes.len()));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Get the raw bytes of the secret key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl Ciphertext {
    /// Create a [`Ciphertext`] from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 1088 {
            return Err(MlkemError::InvalidCiphertextLength(bytes.len()));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Get the raw bytes of the ciphertext.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl SharedSecret {
    /// Create a [`SharedSecret`] from raw bytes.
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 32 {
            return Err(MlkemError::InvalidSharedSecretLength(bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Get the raw bytes of the shared secret.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// ML-KEM-768 primitive operations.
pub struct Mlkem768;

impl Mlkem768 {
    fn kem() -> Result<Kem, MlkemError> {
        Kem::new(Algorithm::MlKem768).map_err(|_| MlkemError::KeygenFailed)
    }

    /// Generate a new ML-KEM-768 keypair.
    ///
    /// Returns ([`PublicKey`], [`SecretKey`]) where:
    /// - [`PublicKey`] is 1184 bytes (encapsulation key)
    /// - [`SecretKey`] is 2400 bytes (decapsulation key)
    pub fn generate_keypair() -> Result<(PublicKey, SecretKey), MlkemError> {
        let kem = Self::kem()?;
        let (pk, sk) = kem.keypair().map_err(|_| MlkemError::KeygenFailed)?;
        Ok((PublicKey(pk.into_vec()), SecretKey(sk.into_vec())))
    }

    /// Encapsulate a shared secret to the given public key.
    ///
    /// Returns ([`Ciphertext`], [`SharedSecret`]) where:
    /// - [`Ciphertext`] is 1088 bytes (to be sent to the key holder)
    /// - [`SharedSecret`] is 32 bytes (the derived secret)
    pub fn encapsulate(pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), MlkemError> {
        let kem = Self::kem()?;
        let oqs_pk = kem
            .public_key_from_bytes(&pk.0)
            .ok_or(MlkemError::InvalidPublicKeyLength(pk.0.len()))?;
        let (ct, ss) = kem
            .encapsulate(oqs_pk)
            .map_err(|_| MlkemError::EncapsulateFailed)?;
        Ok((
            Ciphertext(ct.into_vec()),
            SharedSecret::from_bytes(ss.as_ref()).map_err(|_| MlkemError::EncapsulateFailed)?,
        ))
    }

    /// Decapsulate a shared secret from the given ciphertext using the secret key.
    ///
    /// Returns [`SharedSecret`] (32 bytes).
    pub fn decapsulate(sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, MlkemError> {
        let kem = Self::kem()?;
        let oqs_sk = kem
            .secret_key_from_bytes(&sk.0)
            .ok_or(MlkemError::InvalidSecretKeyLength(sk.0.len()))?;
        let oqs_ct = kem
            .ciphertext_from_bytes(&ct.0)
            .ok_or(MlkemError::InvalidCiphertextLength(ct.0.len()))?;
        let ss = kem
            .decapsulate(oqs_sk, oqs_ct)
            .map_err(|_| MlkemError::DecapsulateFailed)?;
        SharedSecret::from_bytes(ss.as_ref()).map_err(|_| MlkemError::DecapsulateFailed)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_roundtrip() {
        let (pk, sk) = Mlkem768::generate_keypair().expect("keypair generation failed");
        assert_eq!(pk.as_bytes().len(), 1184);
        assert_eq!(sk.as_bytes().len(), 2400);
    }

    #[test]
    fn encapsulate_decapsulate_shared_secret() {
        let (pk, sk) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let (ct, ss1) = Mlkem768::encapsulate(&pk).expect("encapsulation failed");
        let ss2 = Mlkem768::decapsulate(&sk, &ct).expect("decapsulation failed");

        assert_eq!(ct.as_bytes().len(), 1088);
        assert_eq!(ss1.as_bytes().len(), 32);
        assert_eq!(ss2.as_bytes().len(), 32);
        assert_eq!(ss1, ss2, "shared secrets must match");
    }

    #[test]
    fn multiple_encapsulations_produce_different_ciphertexts() {
        let (pk, sk) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let (ct1, ss1) = Mlkem768::encapsulate(&pk).expect("encapsulation 1 failed");
        let (ct2, ss2) = Mlkem768::encapsulate(&pk).expect("encapsulation 2 failed");

        // Each encapsulation should produce a different ciphertext (IND-CCA2)
        assert_ne!(ct1, ct2, "ciphertexts should differ");
        // But decapsulation with the same secret key should recover the respective shared secrets
        let ss1_dec = Mlkem768::decapsulate(&sk, &ct1).expect("decapsulation 1 failed");
        let ss2_dec = Mlkem768::decapsulate(&sk, &ct2).expect("decapsulation 2 failed");
        assert_eq!(ss1, ss1_dec);
        assert_eq!(ss2, ss2_dec);
    }

    #[test]
    fn public_key_serialization() {
        let (pk, _) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let pk_bytes = pk.as_bytes();
        let pk2 = PublicKey::from_bytes(pk_bytes).expect("public key deserialization failed");
        assert_eq!(pk, pk2);
    }

    #[test]
    fn secret_key_serialization() {
        let (_, sk) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let sk_bytes = sk.as_bytes();
        let sk2 = SecretKey::from_bytes(sk_bytes).expect("secret key deserialization failed");
        assert_eq!(sk, sk2);
    }

    #[test]
    fn ciphertext_serialization() {
        let (pk, _) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let (ct, _) = Mlkem768::encapsulate(&pk).expect("encapsulation failed");
        let ct_bytes = ct.as_bytes();
        let ct2 = Ciphertext::from_bytes(ct_bytes).expect("ciphertext deserialization failed");
        assert_eq!(ct, ct2);
    }

    #[test]
    fn shared_secret_serialization() {
        let (pk, _sk) = Mlkem768::generate_keypair().expect("keypair generation failed");
        let (_, ss) = Mlkem768::encapsulate(&pk).expect("encapsulation failed");
        let ss_bytes = ss.as_bytes();
        let ss2 = SharedSecret::from_bytes(ss_bytes).expect("shared secret deserialization failed");
        assert_eq!(ss, ss2);
    }

    #[test]
    fn invalid_public_key_length() {
        let err = PublicKey::from_bytes(&[0u8; 100]).unwrap_err();
        assert!(matches!(err, MlkemError::InvalidPublicKeyLength(100)));
    }

    #[test]
    fn invalid_secret_key_length() {
        let err = SecretKey::from_bytes(&[0u8; 100]).unwrap_err();
        assert!(matches!(err, MlkemError::InvalidSecretKeyLength(100)));
    }

    #[test]
    fn invalid_ciphertext_length() {
        let err = Ciphertext::from_bytes(&[0u8; 100]).unwrap_err();
        assert!(matches!(err, MlkemError::InvalidCiphertextLength(100)));
    }

    #[test]
    fn invalid_shared_secret_length() {
        let err = SharedSecret::from_bytes(&[0u8; 100]).unwrap_err();
        assert!(matches!(err, MlkemError::InvalidSharedSecretLength(100)));
    }
}
