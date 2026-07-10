use std::path::Path;

use tokio::sync::mpsc;

use xray_tui_config::import_export::Profile;
use xray_tui_core::grpc_client;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{
    BuildParams, CLASH_API_PORT, ConfigBuilder, CoreManager, CoreType, find_binary, resolve_core,
};
use xray_tui_db::models::{DnsSetting, RoutingRule};

use crate::AppState;
use crate::parse_core_log_line;
use crate::types::CoreEvent;
use crate::{ClashTraffic, try_send_or_warn};
use futures_util::StreamExt;

/// Connect to a profile by starting the appropriate core (xray-core or sing-box).
pub fn connect_to_profile(state: &mut AppState, protocol_id: i64) {
    if state.connecting {
        return;
    }

    let (endpoint, protocol_row, profile, profile_override) = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id == protocol_id)
    {
        let p = r.active_protocol();
        let profile = Profile {
            id: p.id,
            sig: p.sig,
            cred_hash: p.cred_hash,
            proto_kind: p.proto_kind.clone(),
            spec_blob: p.spec_blob.clone(),
            config_type: p.config_type,
            core_type: p.core_type.clone(),
            address: r.endpoint.host.clone(),
            port: r.endpoint.port,
            transport: p.transport.clone(),
            security: p.security.clone(),
            created_at: p.created_at,
            remarks: p.remarks.clone(),
        };
        let core_override = p.core_type.parse::<CoreType>().ok();
        (r.endpoint.clone(), p.clone(), profile, core_override)
    } else {
        state.log_trace("error", "tui", "Profile not found for connection");
        return;
    };

    let protocol = if let Some(p) = Protocol::try_from_i32(profile.config_type) {
        p
    } else {
        state.log_trace(
            "error",
            "tui",
            &format!("Unknown protocol: {}", profile.config_type),
        );
        return;
    };

    let core_type = resolve_core(protocol, profile_override);

    // If already connected/disconnecting, send stop signal first
    if let Some(tx) = state.disconnect_tx.take() {
        let _ = tx.send(());
    }

    // Create disconnect signal channel
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    state.disconnect_tx = Some(stop_tx);
    state.connecting = true;
    state.connected_protocol_id = Some(protocol_id);
    state.connection_error = None;
    let tx = if let Some(tx) = &state.core_event_tx {
        tx.clone()
    } else {
        state.connecting = false;
        state.log_trace("error", "tui", "Core event channel not initialized");
        return;
    };

    let params = BuildParams {
        v2ray_api_enabled: matches!(core_type, CoreType::Xray),
        clash_api_enabled: matches!(core_type, CoreType::SingBox),
        log_level: state.config.core.log_level.clone(),
        socks_port: state.config.inbound.socks_port,
        http_port: state.config.inbound.http_port,
        listen: state.config.inbound.listen.clone(),
        sniffing: state.config.inbound.sniffing,
        clash_api_port: state.config.clash_api_port,
        mux: if state.config.mux.enabled { Some(serde_json::json!({
            "protocol": state.config.mux.protocol,
            "max_connections": state.config.mux.max_connections,
            "min_streams": state.config.mux.min_streams,
            "max_streams": state.config.mux.max_streams,
            "padding": state.config.mux.padding,
        })) } else { None },

        skip_cert_verify: state.config.core.skip_cert_verify,
    };

    // Default DNS and routing for first pass
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
    let routing: Vec<RoutingRule> = vec![];

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");

    // Create log forwarding channel
    let (log_line_tx, mut log_line_rx) = mpsc::channel::<String>(512);
    let state_log_sender = state.log_sender_tx.clone();
    let handle = tokio::spawn(async move {
        // 1. Build config
        let backend_config = match ConfigBuilder::build(
            &endpoint,
            &protocol_row,
            core_type,
            &params,
            &routing,
            &dns,
        ) {
            Ok(c) => c,
            Err(e) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::Error(format!("Config build failed: {e}")),
                    "config_build_error",
                );
                return;
            }
        };

        // 2. Find binary
        let bin_path = if let Some(p) = find_binary(core_type, &bin_dir) {
            p
        } else {
            try_send_or_warn(&tx, CoreEvent::Error(
                "Core binary not found. Place it in ~/.config/xray-tui/bin/ or install in PATH."
                    .to_string(),
            ), "binary_not_found");
            return;
        };

        // 3. Start core
        let mut manager = CoreManager::with_log_channel(bin_configs_dir, log_line_tx);
        if let Err(e) = manager.start(core_type, &backend_config, &bin_path).await {
            try_send_or_warn(
                &tx,
                CoreEvent::Error(format!("Failed to start core: {e}")),
                "core_start_error",
            );
            return;
        }

        // 3b. Readiness probe: wait for the core's proxy port to open
        let proxy_port = params.socks_port;
        if proxy_port > 0 {
            let probe_start = std::time::Instant::now();
            loop {
                if probe_start.elapsed() > std::time::Duration::from_secs(10) {
                    break;
                }
                match tokio::net::TcpStream::connect(("127.0.0.1", proxy_port)).await {
                    Ok(_) => break,
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(200)).await,
                }
            }
        }

        // 4. Signal connected
        try_send_or_warn(&tx, CoreEvent::Connected(core_type), "connected");

        // Forward stderr log lines as CoreEvent::LogLine
        let log_tx = tx.clone();
        let log_sender = state_log_sender.clone();
        tokio::spawn(async move {
            while let Some(line) = log_line_rx.recv().await {
                let (level, target, message, ts_nanos) = parse_core_log_line(&line, core_type);
                let timestamp_nanos = ts_nanos.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as i64
                });
                // Write to log storage via non-blocking channel (batched async writer)
                if let Some(ref sender) = log_sender {
                    let _ = sender.send(xray_tui_core::log_heed::LogMessage {
                        level: level.clone(),
                        target: target.clone(),
                        message: message.clone(),
                        timestamp_nanos: timestamp_nanos as u64,
                    });
                }
                // Forward to TUI
                try_send_or_warn(
                    &log_tx,
                    CoreEvent::LogLine {
                        level,
                        target,
                        message,
                        timestamp_nanos,
                    },
                    "log_line",
                );
            }
        });

        let profile_id = profile.id;

        if core_type == CoreType::Xray {
            // === gRPC polling loop (xray-core) ===
            let provider = match grpc_client::create_stats_provider(CoreType::Xray).await {
                Ok(p) => Some(p),
                Err(e) => {
                    try_send_or_warn(
                        &tx,
                        CoreEvent::StatsError(format!("Stats API unavailable: {e}")),
                        "stats_api_unavailable",
                    );
                    None
                }
            };
            let poll_interval = std::time::Duration::from_secs(3);
            let mut ticker = tokio::time::interval(poll_interval);
            ticker.tick().await;
            let mut sys_tick_counter = 0u8;
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = ticker.tick() => {
                        if let Some(ref provider) = provider {
                            match provider.query_stats("outbound>>>*>>>traffic>>>*", true).await {
                                Ok(stats) => {
                                    let mut today_up = 0i64;
                                    let mut today_down = 0i64;
                                    for stat in &stats {
                                        if stat.name.contains(">>>uplink") {
                                            today_up += stat.value;
                                        } else if stat.name.contains(">>>downlink") {
                                            today_down += stat.value;
                                        }
                                    }
                                    try_send_or_warn(&tx, CoreEvent::StatsUpdate {
                                        protocol_id: profile_id,
                                        today_up,
                                        today_down,
                                        total_up: today_up,
                                        total_down: today_down,
                                    }, "stats_update");
                                }
                                Err(e) => {
                                try_send_or_warn(&tx, CoreEvent::StatsError(format!("Stats query failed: {e}")), "stats_query_err");
                                }
                            }
                            // sys stats every 3rd tick (~9s)
                            sys_tick_counter += 1;
                            if sys_tick_counter >= 3 {
                                sys_tick_counter = 0;
                                match provider.get_sys_stats().await {
                                    Ok(sys) => { try_send_or_warn(&tx, CoreEvent::SysStatsUpdate(sys), "sys_stats_update"); }
                                    Err(e) => {
                                        try_send_or_warn(&tx, CoreEvent::StatsError(format!("Sys stats query failed: {e}")), "sys_stats_err");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // === Sing-box Clash API /traffic streaming ===
            let clash_port = params.clash_api_port.unwrap_or(CLASH_API_PORT);
            let url = format!("http://127.0.0.1:{clash_port}/traffic");

            match reqwest::Client::new().get(&url).send().await {
                Ok(resp) => {
                    let mut stream = Box::pin(resp.bytes_stream());
                    let mut buf = Vec::new();
                    let mut session_up: i64 = 0;
                    let mut session_down: i64 = 0;
                    loop {
                        tokio::select! {
                            _ = &mut stop_rx => break,
                            chunk = stream.next() => {
                                match chunk {
                                    Some(Ok(bytes)) => {
                                        buf.extend_from_slice(&bytes);
                                        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                                            let line = buf.drain(..=pos).collect::<Vec<_>>();
                                            let trimmed = line.as_slice().trim_ascii();
                                            if let Ok(t) = serde_json::from_slice::<ClashTraffic>(trimmed) {
                                                session_up += t.up;
                                                session_down += t.down;
                                                try_send_or_warn(&tx, CoreEvent::StatsUpdate {
                                                    protocol_id: profile_id,
                                                    today_up: session_up,
                                                    today_down: session_down,
                                                    total_up: session_up,
                                                    total_down: session_down,
                                                }, "clash_stats_update");
                                            }
                                        }
                                    }
                                    Some(Err(e)) => {
                                        try_send_or_warn(&tx, CoreEvent::StatsError(
                                            format!("Clash API stream error: {e}")
                                        ), "clash_stream_err");
                                        break;
                                    }
                                    None => break,
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    try_send_or_warn(
                        &tx,
                        CoreEvent::StatsError(format!(
                            "Clash API unavailable (is sing-box running?): {e}"
                        )),
                        "clash_unavailable",
                    );
                }
            }
        }

        // 6. Stop core
        let _ = manager.stop().await;

        // 7. Signal disconnected
        try_send_or_warn(&tx, CoreEvent::Disconnected, "disconnected");
    });
    state.core_task_handle = Some(handle);
}

/// Disconnect the currently running core.
pub fn disconnect(state: &mut AppState) {
    if let Some(tx) = state.disconnect_tx.take() {
        let _ = tx.send(());
    }
    if let Some(handle) = state.core_task_handle.take() {
        handle.abort();
    }
    state.connected_core = None;
    state.connected_protocol_id = None;
    state.connecting = false;
    state.log_trace("info", "core", "Disconnected");
}
