use super::super::{PingResult, ProfileKey};
use super::RealPingManager;
use crate::bin_manager::find_binary;
use crate::config_builder::{BuildParams, ConfigBuilder};
use crate::process::CoreManager;
use crate::protocol::Protocol;
use crate::protocol_core_mapping::resolve_core;
use tokio::sync::mpsc;
use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow};

/// Run a real ping through a temp xray-core instance.
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

        let proxy_port = ctx.allocate_port();

        let params = BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: proxy_port,
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

        let (log_line_tx, mut log_rx) = mpsc::channel(512);
        // Spawn reader to capture xray-core stderr for diagnostics
        // (includes config validation errors on exit code 23)
        let log_dir = temp_dir_path.clone();
        tokio::spawn(async move {
            while let Some(line) = log_rx.recv().await {
                tracing::warn!(target: "core::real_ping", dir=%log_dir.display(), "{line}");
            }
        });
        let mut manager = CoreManager::with_log_channel(temp_dir_path.clone(), log_line_tx);
        manager
            .start(resolved_core, &backend_config, &bin_path, None)
            .await
            .map_err(|e| format!("Core start: {e}"))?;

        let _ = crate::speed_test::wait_for_socks5(
            &proxy_addr,
            proxy_port,
            std::time::Duration::from_secs(5),
        )
        .await;

        let rp_result = crate::speed_test::real_ping(
            &proxy_addr,
            proxy_port,
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
