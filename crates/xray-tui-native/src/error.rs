use std::time::Duration;

use xray_tui_proto::proto_spec::ProtocolKind;

/// Errors from the native proxy core.
///
/// Every network step (dial, transport upgrade, security handshake, protocol
/// handshake, tunnel I/O) is wrapped in `tokio::time::timeout`; a deadline
/// expiry surfaces as [`NativeError::Timeout`].
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("invalid or unsupported config: {0}")]
    Config(String),
    #[error("server dial failed: {0}")]
    Dial(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("REALITY error: {0}")]
    Reality(String),
    #[error("protocol {kind} error: {detail}")]
    Protocol { kind: ProtocolKind, detail: String },
    #[error("not implemented: {feature}")]
    NotImplemented { feature: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("timeout on {step} (limit {limit:?})")]
    Timeout { step: &'static str, limit: Duration },
}

/// Named deadline limits, applied around every network step.
pub mod timeouts {
    use std::time::Duration;

    pub const DIAL: Duration = Duration::from_secs(10);
    pub const TRANSPORT: Duration = Duration::from_secs(10);
    pub const SECURITY: Duration = Duration::from_secs(10);
    pub const PROTOCOL: Duration = Duration::from_secs(10);
    pub const TUNNEL_READ: Duration = Duration::from_secs(30);
}
