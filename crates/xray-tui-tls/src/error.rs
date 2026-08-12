//! Error types for the TLS engine.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("spec error: {0}")]
    Spec(String),
    #[error("handshake error: {0}")]
    Handshake(String),
    #[error("certificate verification failed: {0}")]
    Verify(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("not implemented: {feature}")]
    NotImplemented { feature: String },
    #[error("server sent HelloRetryRequest; retry is not supported")]
    HelloRetryRequest,
    /// REALITY was expected but the server flight was not REALITY-authenticated
    /// — a real certificate (transparent proxy / possible MITM or redirection).
    #[error("REALITY: received real certificate (potential MITM or redirection)")]
    RealityFallback,
}

pub type Result<T> = std::result::Result<T, TlsError>;
