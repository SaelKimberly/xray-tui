use super::super::{PingResult, ProfileKey};
use super::RealPingManager;
use crate::bin_manager::find_binary;
use crate::config_builder::{BuildParams, ConfigBuilder};
use crate::process::CoreManager;
use crate::protocol::Protocol;
use crate::protocol_core_mapping::resolve_core;
use tokio::sync::mpsc;
use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow};

/// Run a real ping through a temp sing-box instance.
/// ConfigBuilder::build handles both cores, so the flow is identical
/// to xray.rs. Separation exists for future divergence (different startup
/// flags, API ports, etc.).
pub(super) async fn real_ping(
    endpoint: &Endpoint,
    protocol: &ProtocolRow,
    ctx: &RealPingManager,
) -> PingResult {
    let endpoint = endpoint.clone();
    let protocol = protocol.clone();
    let proxy_addr = ctx.proxy_addr.clone();
    let ping_url = ctx.ping_url.clone();
    let ip_api_url = ctx.ip_api_url.clone();
    let timeout = ctx.timeout;
    let retries = ctx.retries;
    let bin_dir = ctx.bin_dir.clone();
    let bin_configs_dir = ctx.bin_configs_dir.clone();
    let r#type = protocol.config_type;

    let outcome = async {
        let temp_dir =
            tempfile::TempDir::new_in(&bin_configs_dir).map_err(|e| format!("Temp dir: {e}"))?;
        let temp_dir_path = temp_dir.path().to_path_buf();

        let proto = Protocol::try_from_i32(r#type).unwrap_or(Protocol::Custom);
        let resolved_core = resolve_core(proto, None);

        let params = BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: ctx.base_proxy_port,
            http_port: None,
            listen: proxy_addr.clone(),
            sniffing: false,
            clash_api_port: None,
            mux: None,
            clash_mixin: None,
            skip_cert_verify: false,
        };

        let dns = DnsSetting {
            id: "default".to_string(),
            name: None,
            servers: None,
            hosts: None,
            query_strategy: None,
            disable_cache: None,
            disable_fallback: None,
            client_ip: None,
        };

        let backend_config =
            ConfigBuilder::build(&endpoint, &protocol, resolved_core, &params, &[], &dns)
                .map_err(|_| "Build config failed".to_string())?;

        let bin_path =
            find_binary(resolved_core, &bin_dir).ok_or_else(|| "Binary not found".to_string())?;

        let (log_line_tx, _log_rx) = mpsc::channel(512);
        let mut manager = CoreManager::with_log_channel(temp_dir_path.clone(), log_line_tx);
        manager
            .start(resolved_core, &backend_config, &bin_path, None)
            .await
            .map_err(|e| format!("Core start: {e}"))?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let rp_result = crate::speed_test::real_ping(
            &proxy_addr,
            ctx.base_proxy_port,
            &ping_url,
            &ip_api_url,
            timeout,
            retries,
        )
        .await;

        let _ = manager.stop().await;
        rp_result.map_err(|e| format!("Real ping: {e}"))
    }
    .await;

    match outcome {
        Ok(rp) => PingResult {
            profile_key: ProfileKey {
                config_type: r#type,
                address: endpoint.host.clone(),
                port: endpoint.port as u16,
            },
            latency_ms: Some(rp.latency_ms),
            ip_info: rp.ip_info,
            error: None,
        },
        Err(e) => PingResult {
            profile_key: ProfileKey {
                config_type: r#type,
                address: endpoint.host.clone(),
                port: endpoint.port as u16,
            },
            latency_ms: None,
            ip_info: None,
            error: Some(e),
        },
    }
}
