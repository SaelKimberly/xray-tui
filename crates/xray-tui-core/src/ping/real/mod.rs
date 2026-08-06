//! RealPingManager: launches a temporary binary core to test a profile.
//! Profile config is built via ConfigBuilder, the core runs briefly, then
//! HTTP requests are sent through the SOCKS5 proxy to measure real latency.

mod pool;
mod singbox;
mod xray;

pub use pool::CorePool;
pub use pool::SinglePingReq;

use super::PingResult;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use xray_tui_db::models::{Endpoint, ProfileStats, Protocol as DbProtocol};

/// Configuration and context for running real pings via temporary core instances.
#[derive(Clone, Debug)]
pub struct RealPingManager {
    pub ping_url: String,
    pub ip_api_url: String,
    pub timeout: Duration,
    pub retries: u32,
    pub proxy_addr: String,
    pub base_proxy_port: u16,
    pub next_ping_port: Arc<AtomicU16>,
    pub bin_dir: PathBuf,
    pub bin_configs_dir: PathBuf,
}

impl RealPingManager {
    /// Allocate next available port for a temp core.
    /// Starts from `base_proxy_port` and increments atomically.
    /// Shared across all concurrent tasks via `Arc`.
    pub fn allocate_port(&self) -> u16 {
        self.next_ping_port.fetch_add(1, Ordering::Relaxed)
    }

    /// Run real ping for a single profile. The core is taken from
    /// `link.core_type` — the per-pair override resolved at parse time.
    pub async fn real_ping(
        &self,
        endpoint: &Endpoint,
        link: &ProfileStats,
        protocol: &DbProtocol,
    ) -> PingResult {
        match link.core_type {
            xray_tui_proto::proto_spec::CoreType::Xray => {
                xray::real_ping(endpoint, link, protocol, self).await
            }
            xray_tui_proto::proto_spec::CoreType::SingBox => {
                singbox::real_ping(endpoint, link, protocol, self).await
            }
        }
    }
}

/// Legacy `config_type` integer for `ProfileKey`, derived from the typed
/// `proto_kind`.
pub(super) fn config_type(protocol: &DbProtocol) -> i32 {
    protocol.proto_kind.to_i32()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU16, Ordering};

    #[test]
    fn allocate_port_increments() {
        let port = Arc::new(AtomicU16::new(1000));
        let p1 = port.fetch_add(1, Ordering::Relaxed);
        let p2 = port.fetch_add(1, Ordering::Relaxed);
        assert_eq!(p1, 1000);
        assert_eq!(p2, 1001);
    }

    #[test]
    fn allocate_port_wraps_at_u16_max() {
        let port = AtomicU16::new(u16::MAX);
        let p1 = port.fetch_add(1, Ordering::Relaxed);
        assert_eq!(p1, u16::MAX);
        assert_eq!(port.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn concurrent_allocation_unique() {
        let port = Arc::new(AtomicU16::new(30000));
        let mut handles = Vec::new();
        for _ in 0..10 {
            let p = port.clone();
            handles.push(tokio::spawn(
                async move { p.fetch_add(1, Ordering::Relaxed) },
            ));
        }
        let mut results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        results.sort();
        results.dedup();
        assert_eq!(results.len(), 10, "duplicate ports allocated");
    }
}
