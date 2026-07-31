//! RealPingManager: launches a temporary binary core to test a profile.
//! Profile config is built via ConfigBuilder, the core runs briefly, then
//! HTTP requests are sent through the SOCKS5 proxy to measure real latency.

mod pool;
mod singbox;
mod xray;

pub use pool::CorePool;

use super::PingResult;
use crate::core_type::CoreType;
use crate::protocol::Protocol;
use crate::protocol_core_mapping::resolve_core;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

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

    /// Run real ping for a single profile. Builds config for the resolved core,
    pub async fn real_ping(
        &self,
        endpoint: &xray_tui_db::models::Endpoint,
        protocol: &xray_tui_db::models::ProtocolRow,
        config_type: i32,
    ) -> PingResult {
        let proto = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        let core = resolve_core(proto, None);
        match core {
            CoreType::Xray => xray::real_ping(endpoint, protocol, self).await,
            CoreType::SingBox => singbox::real_ping(endpoint, protocol, self).await,
            CoreType::Auto => {
                tracing::warn!("resolve_core returned Auto, falling back to xray");
                xray::real_ping(endpoint, protocol, self).await
            }
        }
    }
}
