//! TLS 1.3 client engine with browser fingerprint mimicry and REALITY support.
//!
//! This crate provides a ring-based TLS 1.3 client that emulates real browsers,
//! including JA3 fingerprinting, TLS record and handshake handling, certificate
//! verification, and REALITY protocol integration.

pub mod spec;
pub mod profiles;
pub mod hello;
pub mod crypto;
pub mod record;
pub mod handshake;
pub mod verify;
pub mod reality;
pub mod http2;

pub mod error;

/// Byte-stream seam: everything the engine touches is generic over this.
pub trait Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> Stream for T {}
