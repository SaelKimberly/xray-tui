//! Pure cryptographic algorithm codecs.
//!
//! Rule: `crypto/` owns ALGORITHMS, layers own DECISIONS. No I/O policy and
//! no transport logic lives here. Packet-level obfuscation (e.g. Hysteria2
//! Salamander) is implemented here but consumed inside the QUIC transport's
//! packet handling — it is never forced into the byte-stream seam.
//!
//! Current state: JA3/JA4 fingerprint encoders are implemented (tier-1
//! fingerprint tripwire). Everything else is a documented skeleton waiting
//! for its protocol milestone (M2+).

pub mod aead;
pub mod fingerprint;
pub mod kdf;
pub mod legacy_stream;
pub mod salamander;
