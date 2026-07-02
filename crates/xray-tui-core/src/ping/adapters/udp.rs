use super::{FastPingAdapter, PingCapability, PingError};
use crate::protocol::Protocol;
use async_trait::async_trait;
use std::time::Duration;

/// Direct UDP ping. Supports protocols where the endpoint accepts UDP traffic.
#[derive(Debug)]
pub struct UdpPingAdapter;

#[async_trait]
impl FastPingAdapter for UdpPingAdapter {
    fn transport(&self) -> PingCapability {
        PingCapability::Udp
    }

    fn name(&self) -> &'static str {
        "UDP"
    }

    fn supports(&self, protocol: Protocol) -> bool {
        matches!(protocol, Protocol::WireGuard | Protocol::ShadowsocksR)
    }

    async fn ping(&self, addr: &str, port: u16, timeout: Duration) -> Result<Duration, PingError> {
        Ok(crate::speed_test::udp_ping(addr, port, timeout).await?)
    }
}
