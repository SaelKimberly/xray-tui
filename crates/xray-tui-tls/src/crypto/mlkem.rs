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
use zeroize::{ZeroizeOnDrop, Zeroizing};
/// ML-KEM-768 public key (1184 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey([u8; 1184]);

/// ML-KEM-768 secret key (2400 bytes).
///
/// Wipes on drop: this is the decapsulation key, so a stale heap copy is
/// enough to recover every shared secret negotiated with it. `Debug` prints
/// the length only — the derive would dump the key into any log line.
#[derive(Clone, PartialEq, Eq, ZeroizeOnDrop)]
pub struct SecretKey([u8; 2400]);

impl core::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SecretKey(<{} bytes redacted>)", self.0.len())
    }
}

/// ML-KEM-768 ciphertext (1088 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext([u8; 1088]);

/// ML-KEM-768 shared secret (32 bytes).
///
/// Wipes on drop; `Debug` prints the length only.
#[derive(Clone, PartialEq, Eq, ZeroizeOnDrop)]
pub struct SharedSecret([u8; 32]);

impl core::fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SharedSecret(<{} bytes redacted>)", self.0.len())
    }
}

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
    /// Create a [`PublicKey`] from raw bytes: copied once into the fixed
    /// array, no `Vec` allocation.
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 1184 {
            return Err(MlkemError::InvalidPublicKeyLength(bytes.len()));
        }
        let mut arr = [0u8; 1184];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Get the raw bytes of the public key.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl SecretKey {
    /// Create a [`SecretKey`] from raw bytes: copied once into the fixed
    /// array, no `Vec` allocation.
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 2400 {
            return Err(MlkemError::InvalidSecretKeyLength(bytes.len()));
        }
        let mut arr = [0u8; 2400];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Get the raw bytes of the secret key.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl Ciphertext {
    /// Create a [`Ciphertext`] from raw bytes: copied once into the fixed
    /// array, no `Vec` allocation.
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, MlkemError> {
        if bytes.len() != 1088 {
            return Err(MlkemError::InvalidCiphertextLength(bytes.len()));
        }
        let mut arr = [0u8; 1088];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Get the raw bytes of the ciphertext.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
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
        let pk_vec = pk.into_vec();
        let sk_vec = Zeroizing::new(sk.into_vec());
        if pk_vec.len() != 1184 || sk_vec.len() != 2400 {
            return Err(MlkemError::KeygenFailed);
        }
        let mut pk_arr = [0u8; 1184];
        let mut sk_arr = [0u8; 2400];
        pk_arr.copy_from_slice(&pk_vec);
        sk_arr.copy_from_slice(&sk_vec);
        Ok((PublicKey(pk_arr), SecretKey(sk_arr)))
    }

    /// Derive a keypair from the 64-byte FIPS 203 keygen seed (`d || z`).
    ///
    /// Deterministic: the same seed always yields the same keypair. This is
    /// the seed form xray's VLESS `mlkem768x25519plus` server config carries
    /// (`decryption` key segment — Go `mlkem.NewDecapsulationKey768`), so
    /// the e2e harness can derive the encapsulation key the client string
    /// must carry from the very seed the server config holds.
    ///
    /// # Errors
    /// [`MlkemError::KeygenFailed`] when liboqs rejects the seed length or
    /// the derandomized keygen fails.
    pub fn keypair_from_seed(seed: &[u8; 64]) -> Result<(PublicKey, SecretKey), MlkemError> {
        let kem = Self::kem()?;
        let seed_ref = kem
            .keypair_seed_from_bytes(seed.as_slice())
            .ok_or(MlkemError::KeygenFailed)?;
        let (pk, sk) = kem
            .keypair_derand(seed_ref)
            .map_err(|_| MlkemError::KeygenFailed)?;
        let pk_vec = pk.into_vec();
        let sk_vec = Zeroizing::new(sk.into_vec());
        if pk_vec.len() != 1184 || sk_vec.len() != 2400 {
            return Err(MlkemError::KeygenFailed);
        }
        let mut pk_arr = [0u8; 1184];
        let mut sk_arr = [0u8; 2400];
        pk_arr.copy_from_slice(&pk_vec);
        sk_arr.copy_from_slice(&sk_vec);
        Ok((PublicKey(pk_arr), SecretKey(sk_arr)))
    }

    /// Encapsulate a shared secret to the given public key.
    ///
    /// Returns ([`Ciphertext`], [`SharedSecret`]) where:
    /// - [`Ciphertext`] is 1088 bytes (to be sent to the key holder)
    /// - [`SharedSecret`] is 32 bytes (the derived secret)
    pub fn encapsulate(pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), MlkemError> {
        let mut ct = [0u8; 1088];
        let mut ss = [0u8; 32];
        Self::encapsulate_into(pk, &mut ct, &mut ss)?;
        // `encapsulate_into` leaves the shared secret in a plain caller
        // buffer; move it into the wiping wrapper here (the zeroize happens
        // on drop, same as before).
        let ss = Zeroizing::new(ss);
        Ok((
            Ciphertext(ct),
            SharedSecret::from_bytes(ss.as_slice()).map_err(|_| MlkemError::EncapsulateFailed)?,
        ))
    }

    /// Encapsulate a shared secret into caller buffers: the ciphertext
    /// lands in `ct_out`, the shared secret in `ss_out`.
    ///
    /// The allocation-free counterpart of [`Self::encapsulate`]: handshake
    /// code sealing straight into a flight buffer skips the intermediate
    /// [`Ciphertext`]/`Vec` copies. `ss_out` is a plain buffer (no wipe
    /// hook) — wrap it in [`Zeroizing`] when it outlives this call.
    pub fn encapsulate_into(
        pk: &PublicKey,
        ct_out: &mut [u8; 1088],
        ss_out: &mut [u8; 32],
    ) -> Result<(), MlkemError> {
        let kem = Self::kem()?;
        let oqs_pk = kem
            .public_key_from_bytes(&pk.0)
            .ok_or(MlkemError::InvalidPublicKeyLength(pk.0.len()))?;
        let (ct, ss) = kem
            .encapsulate(oqs_pk)
            .map_err(|_| MlkemError::EncapsulateFailed)?;
        // liboqs' own `SharedSecret` is a plain `Vec<u8>` with no `Drop`
        // wipe; take ownership of the allocation so it is cleared here.
        let ss = Zeroizing::new(ss.into_vec());
        let ct_vec = ct.into_vec();
        if ct_vec.len() != 1088 || ss.len() != 32 {
            return Err(MlkemError::EncapsulateFailed);
        }
        ct_out.copy_from_slice(&ct_vec);
        ss_out.copy_from_slice(&ss);
        Ok(())
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
        // Same as `encapsulate`: liboqs never wipes its own buffer.
        let ss = Zeroizing::new(ss.into_vec());
        SharedSecret::from_bytes(&ss).map_err(|_| MlkemError::DecapsulateFailed)
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
    fn seed_keypair_is_deterministic_and_roundtrips() {
        let mut seed = [0u8; 64];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = u8::try_from(i).expect("fits u8");
        }
        let (pk1, sk1) = Mlkem768::keypair_from_seed(&seed).expect("seed keygen failed");
        let (pk2, sk2) = Mlkem768::keypair_from_seed(&seed).expect("seed keygen failed");
        assert_eq!(pk1.as_bytes().len(), 1184);
        assert_eq!(sk1.as_bytes().len(), 2400);
        assert_eq!(pk1, pk2, "same seed must yield the same encapsulation key");
        assert_eq!(sk1, sk2);
        // The derived pair is a working ML-KEM-768 pair.
        let (ct, ss1) = Mlkem768::encapsulate(&pk1).expect("encapsulation failed");
        let ss2 = Mlkem768::decapsulate(&sk1, &ct).expect("decapsulation failed");
        assert_eq!(ss1, ss2);
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
