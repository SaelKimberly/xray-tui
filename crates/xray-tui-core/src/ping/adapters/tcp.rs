use super::{FastPingAdapter, PingCapability, PingError};
use crate::protocol::Protocol;
use async_trait::async_trait;
use std::time::Duration;

/// TCP handshake ping. Supports all TCP-based proxy protocols.
#[derive(Debug)]
pub struct TcpPingAdapter;

#[async_trait]
impl FastPingAdapter for TcpPingAdapter {
    fn transport(&self) -> PingCapability {
        PingCapability::Tcp
    }

    fn name(&self) -> &'static str {
        "TCP"
    }

    fn supports(&self, protocol: Protocol) -> bool {
        matches!(
            protocol,
            Protocol::Vmess
                | Protocol::Vless
                | Protocol::Shadowsocks
                | Protocol::Shadowsocks2022
                | Protocol::Socks
                | Protocol::Http
                | Protocol::Trojan
                | Protocol::DokodemoDoor
                | Protocol::Custom
                | Protocol::Naive
                | Protocol::AnyTls
                | Protocol::ShadowTls
                | Protocol::Tor
                | Protocol::Ssh
                | Protocol::ShadowsocksR
                | Protocol::Redirect
                | Protocol::TProxy
                | Protocol::Mixed
        )
    }

    async fn ping(&self, addr: &str, port: u16, timeout: Duration) -> Result<Duration, PingError> {
        Ok(crate::speed_test::tcp_ping(addr, port, timeout).await?)
    }
}
