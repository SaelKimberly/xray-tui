//! FastPingAdapter trait + FastPingManager with adapter registry.
//! Each sub-module implements the trait for one transport.

#[cfg(feature = "quic-ping")]
mod quic;
mod tcp;
mod udp;

use super::{PingCapability, PingError};
use crate::protocol::Protocol;
use std::time::Duration;

/// Transport-level ping adapter. Each impl handles one transport method
/// (TCP, UDP, QUIC) and declares which protocols it supports.
///
/// Design rule: an adapter MUST NOT support any protocol whose `address:port`
/// is not extractable from profile metadata — such protocols get no adapter
/// match and fall through to RealPingManager.
#[async_trait::async_trait]
pub trait FastPingAdapter: Send + Sync + std::fmt::Debug {
    fn transport(&self) -> PingCapability;
    fn name(&self) -> &'static str;
    /// Whether this adapter can ping the given protocol.
    fn supports(&self, protocol: Protocol) -> bool;
    /// Run a ping against `addr:port`, returning latency on success.
    async fn ping(&self, addr: &str, port: u16, timeout: Duration) -> Result<Duration, PingError>;
}

#[cfg(feature = "quic-ping")]
pub use quic::QuicPingAdapter;
pub use tcp::TcpPingAdapter;
pub use udp::UdpPingAdapter;

/// Manages a registry of FastPingAdapters and dispatches to the first match.
/// Built-in adapters are registered by default; new ones can be added via `register()`.
pub struct FastPingManager {
    adapters: Vec<Box<dyn FastPingAdapter>>,
    timeout: Duration,
}

impl std::fmt::Debug for FastPingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastPingManager")
            .field(
                "adapters",
                &self.adapters.iter().map(|a| a.name()).collect::<Vec<_>>(),
            )
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl FastPingManager {
    pub fn new(timeout: Duration) -> Self {
        let mut mgr = Self {
            adapters: Vec::new(),
            timeout,
        };
        mgr.register(Box::new(TcpPingAdapter));
        mgr.register(Box::new(UdpPingAdapter));
        #[cfg(feature = "quic-ping")]
        mgr.register(Box::new(QuicPingAdapter));
        mgr
    }

    pub fn register(&mut self, adapter: Box<dyn FastPingAdapter>) {
        self.adapters.push(adapter);
    }

    /// Find the first adapter that supports this protocol.
    pub fn adapter_for(&self, protocol: Protocol) -> Option<&dyn FastPingAdapter> {
        self.adapters
            .iter()
            .find(|a| a.supports(protocol))
            .map(|b| b.as_ref())
    }

    /// Capability for a protocol — for TUI indicator display.
    pub fn capability_for(&self, config_type: i32) -> PingCapability {
        let protocol = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        self.adapter_for(protocol)
            .map_or(PingCapability::None, |a| a.transport())
    }

    /// Run fast ping. Returns Ok(latency) or Err(PingError::NotSupported) if no adapter.
    pub async fn ping(
        &self,
        config_type: i32,
        addr: &str,
        port: u16,
    ) -> Result<Duration, PingError> {
        let protocol = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        match self.adapter_for(protocol) {
            Some(adapter) => adapter.ping(addr, port, self.timeout).await,
            None => Err(PingError::NotSupported),
        }
    }
}
