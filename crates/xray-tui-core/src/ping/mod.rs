mod adapters;

#[cfg(feature = "quic-ping")]
pub use adapters::QuicPingAdapter;
pub use adapters::{FastPingAdapter, FastPingManager, TcpPingAdapter, UdpPingAdapter};

use std::fmt;

/// The transport used for a fast ping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingCapability {
    Tcp,
    Udp,
    Quic,
    None,
}

impl PingCapability {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Quic => "QUIC",
            Self::None => "\u{2014}",
        }
    }
}

/// Error for ping operations.
#[derive(Debug, Clone)]
pub enum PingError {
    Io(String),
    Timeout(std::time::Duration),
    NotSupported,
    Other(String),
}

impl From<crate::speed_test::SpeedTestError> for PingError {
    fn from(e: crate::speed_test::SpeedTestError) -> Self {
        match e {
            crate::speed_test::SpeedTestError::Timeout(d) => PingError::Timeout(d),
            crate::speed_test::SpeedTestError::Io(e) => PingError::Io(e.to_string()),
            other => PingError::Other(other.to_string()),
        }
    }
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(s) => write!(f, "IO: {s}"),
            Self::Timeout(d) => write!(f, "timeout after {d:?}"),
            Self::NotSupported => write!(f, "not supported by any adapter"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}
