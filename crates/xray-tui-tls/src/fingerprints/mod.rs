//! Flexible TLS fingerprint selection (see
//! `docs/superpowers/specs/2026-08-24-tls-fingerprint-design.md`).

pub mod error;
pub mod query;

pub use error::FingerprintError;
pub use query::{Browser, Device, Fingerprint, Os};
