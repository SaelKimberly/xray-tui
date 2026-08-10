//! Offline TLS fingerprint encoders (JA3/JA4).
//!
//! JA3 and JA4 are computed purely from `ClientHello` fields — no network
//! needed. They tripwire extension-order drift on every commit. The
//! authoritative grader check (tier 2, tls.peet.ws) ships with the
//! fingerprint engine milestone M2.

pub mod ja3;
pub mod ja4;
