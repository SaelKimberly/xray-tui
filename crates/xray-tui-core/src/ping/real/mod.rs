//! RealPingManager: launches a temporary binary core to test a profile.
//! Profile config is built via ConfigBuilder, the core runs briefly, then
//! HTTP requests are sent through the SOCKS5 proxy to measure real latency.

mod singbox;
mod xray;

use super::PingResult;
use crate::core_type::CoreType;
use crate::protocol::Protocol;
use crate::protocol_core_mapping::resolve_core;
use std::path::PathBuf;
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
    pub bin_dir: PathBuf,
    pub bin_configs_dir: PathBuf,
}

impl RealPingManager {
    /// Run real ping for a single profile. Builds config for the resolved core,
    /// launches a temp binary, waits for SOCKS5 proxy, runs HTTP real_ping(), stops.
    pub async fn real_ping(
        &self,
        profile: &xray_tui_db::models::Profile,
        config_type: i32,
    ) -> PingResult {
        let protocol = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        let core = resolve_core(protocol, None);
        match core {
            CoreType::Xray => xray::real_ping(profile, self).await,
            CoreType::SingBox => singbox::real_ping(profile, self).await,
            CoreType::Auto => {
                tracing::warn!("resolve_core returned Auto, falling back to xray");
                xray::real_ping(profile, self).await
            }
        }
    }
}
