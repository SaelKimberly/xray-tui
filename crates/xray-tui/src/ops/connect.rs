use std::path::Path;

use tokio::sync::mpsc;

use xray_tui_core::grpc_client;
use xray_tui_core::{
    BuildParams, CLASH_API_PORT, ConfigBuilder, CoreType, RealCoreManager,
    config_builder::clash_mixin::parse_clash_mixin, find_binary,
};
use xray_tui_db::models::{DnsSetting, EndpointId, RoutingRule};
use xray_tui_proto::proto_spec::CoreType as ProtoCoreType;

use crate::AppState;
use crate::parse_core_log_line;
use crate::types::CoreEvent;
use crate::{ClashTraffic, try_send_or_warn};
use futures_util::StreamExt;

/// Map one sing-box clash `/traffic` line to its `StatsUpdate` event.
///
/// sing-box emits one JSON line per second with that second's traffic delta
/// since the previous line, so the event carries the per-line delta — the
/// same semantics as the xray gRPC poller (`query_stats(..., reset = true)`).
/// The events handler accumulates deltas into the persisted row; sending a
/// cumulative session total here would double-count on every event.
fn clash_stats_event(protocol_id: i64, t: &ClashTraffic) -> CoreEvent {
    CoreEvent::StatsUpdate {
        protocol_id,
        today_up: t.up,
        today_down: t.down,
        total_up: t.up,
        total_down: t.down,
    }
}

/// Connect to a profile by starting the appropriate core (xray-core or sing-box).
pub fn connect_to_profile(state: &mut AppState, endpoint_id: i64) {
    if state.connecting {
        return;
    }

    let (endpoint, link, protocol_id) = {
        let Some(row) = state
            .endpoints
            .iter()
            .find(|r| r.endpoint.id.get() == endpoint_id)
        else {
            state.log_trace(
                "error",
                "tui::ops::connect",
                "Profile not found for connection",
            );
            return;
        };
        let Some((link, _protocol)) = row.active_protocol() else {
            // T8+9: linkless endpoints are valid rows — nothing to connect.
            state.log_trace(
                "error",
                "tui::ops::connect",
                "Endpoint has no protocol links to connect",
            );
            return;
        };
        (row.endpoint.clone(), link.clone(), link.protocol_id)
    };

    // The link's `core_type` is the per-pair override resolved at parse time
    // (never Auto) — it drives both the backend build and the params flags.
    let core_type = match link.core_type {
        ProtoCoreType::Xray => CoreType::Xray,
        ProtoCoreType::SingBox => CoreType::SingBox,
    };

    // If already connected/disconnecting, send stop signal first
    if let Some(tx) = state.disconnect_tx.take() {
        let _ = tx.send(());
    }

    // Create disconnect signal channel
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    state.disconnect_tx = Some(stop_tx);
    state.connecting = true;
    state.connected_protocol_id = Some(endpoint_id);
    state.connection_error = None;

    // Deferred-resolution trigger: connect does real networking, so resolve
    // the inbound host now (force — bypasses the TTL cache).
    crate::ops::enrich::spawn_dns_resolve(state, endpoint.id.get(), true);

    // "Last Used" = connect initiation: DB write + in-memory row so the
    // sub-table refreshes without a reload. `update_last_used` sets both
    // `last_used_at` and `last_seen_at` to the same ts (old semantics).
    let now = jiff::Timestamp::now();
    let lu_endpoint_id = EndpointId::new(endpoint.id.get());
    let lu_db = state.db.clone();
    tokio::spawn(async move {
        if let Err(e) = lu_db
            .update_last_used(protocol_id, lu_endpoint_id, now)
            .await
        {
            tracing::warn!(target: "tui::ops::connect", "update_last_used failed: {e}");
        }
    });
    if let Some(r) = state
        .endpoints
        .iter_mut()
        .find(|r| r.endpoint.id == endpoint.id)
        && let Some(link) = r.links.iter_mut().find(|l| l.protocol_id == protocol_id)
    {
        link.last_used_at = Some(now);
        // Mirror the DB write so the sub-table's displayed last_seen matches
        // the DB staleness classification between reloads.
        link.last_seen_at = now;
    }
    let tx = if let Some(tx) = &state.core_event_tx {
        tx.clone()
    } else {
        state.connecting = false;
        state.log_trace(
            "error",
            "tui::ops::connect",
            "Core event channel not initialized",
        );
        return;
    };

    let params = BuildParams {
        v2ray_api_enabled: matches!(link.core_type, ProtoCoreType::Xray),
        clash_api_enabled: matches!(link.core_type, ProtoCoreType::SingBox),
        log_level: state.config.core.log_level.clone(),
        socks_port: state.config.inbound.socks_port,
        http_port: state.config.inbound.http_port,
        listen: state.config.inbound.listen.clone(),
        sniffing: state.config.inbound.sniffing,
        clash_api_port: state.config.clash_api_port,
        mux: if state.config.mux.enabled {
            Some(serde_json::json!({
                "protocol": state.config.mux.protocol,
                "max_connections": state.config.mux.max_connections,
                "min_streams": state.config.mux.min_streams,
                "max_streams": state.config.mux.max_streams,
                "padding": state.config.mux.padding,
            }))
        } else {
            None
        },
        clash_mixin: state
            .config
            .clash_mixin
            .as_deref()
            .and_then(parse_clash_mixin),
        skip_cert_verify: state.config.core.skip_cert_verify,
    };

    // Routing comes from the already-loaded rules; DNS settings are loaded
    // inside the task (async read) with a typed default fallback.
    let routing: Vec<RoutingRule> = state.routing_rules.clone();
    let db = state.db.clone();

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");

    // Create log forwarding channel
    let (log_line_tx, mut log_line_rx) = mpsc::channel::<String>(512);
    let state_log_sender = state.log_sender_tx.clone();
    let handle = tokio::spawn(async move {
        // 0. Load the Protocol row with config included (the EndpointRow
        //    list ships unloaded deferred JSON).
        let protocol = match crate::state::load_protocol_with_config(&db, protocol_id).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::Error("Protocol row not found for connection".to_string()),
                    "protocol_not_found",
                );
                return;
            }
            Err(e) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::Error(format!("Failed to load protocol: {e}")),
                    "protocol_load_error",
                );
                return;
            }
        };
        let dns = match db.get_dns_settings().await {
            Ok(Some(d)) => d,
            _ => DnsSetting {
                id: "default".to_string(),
                name: None,
                servers: Vec::new(),
                hosts: Vec::new(),
                query_strategy: None,
                disable_cache: false,
                disable_fallback: false,
                client_ip: None,
                cache_ttl_secs: None,
            },
        };

        // 1. Build config
        let backend_config =
            match ConfigBuilder::build(&endpoint, &link, &protocol, &params, &routing, &dns) {
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
        let mut manager = RealCoreManager::new(bin_configs_dir, log_line_tx);
        if let Err(e) = manager
            .start(
                core_type,
                &backend_config,
                &bin_path,
                params.clash_mixin.as_ref(),
            )
            .await
        {
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

        let profile_id = protocol_id.get();

        if core_type == CoreType::Xray {
            // === gRPC polling loop (xray-core) ===
            let provider = match grpc_client::create_stats_provider().await {
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
                                Ok(rows) => {
                                    let mut today_up = 0i64;
                                    let mut today_down = 0i64;
                                    for stat in &rows {
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
                                                // Per-line deltas (sing-box
                                                // emits each second's traffic
                                                // since the last line) — the
                                                // same delta semantics as the
                                                // xray gRPC poller, so the
                                                // stats handler's accumulation
                                                // never double-counts.
                                                try_send_or_warn(
                                                    &tx,
                                                    clash_stats_event(profile_id, &t),
                                                    "clash_stats_update",
                                                );
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
    state.log_trace("info", "core::process", "Disconnected");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clash_traffic_lines_map_to_per_line_deltas() {
        // sing-box `/traffic` emits one JSON line per second carrying that
        // second's delta. Two lines must produce two delta events (100, 50) —
        // never the running session total (100, 150) — or the stats handler's
        // accumulation would double-count on every event.
        let lines = [
            br#"{"up":100,"down":200}"#.as_slice(),
            br#"{"up":50,"down":75}"#.as_slice(),
        ];
        let events: Vec<CoreEvent> = lines
            .iter()
            .filter_map(|line| serde_json::from_slice::<ClashTraffic>(line).ok())
            .map(|t| clash_stats_event(7, &t))
            .collect();
        assert_eq!(events.len(), 2, "both lines parsed");
        let fields: Vec<(i64, i64, i64, i64, i64)> = events
            .iter()
            .map(|e| match e {
                CoreEvent::StatsUpdate {
                    protocol_id,
                    today_up,
                    today_down,
                    total_up,
                    total_down,
                } => (*protocol_id, *today_up, *today_down, *total_up, *total_down),
                other => panic!("expected StatsUpdate, got {other:?}"),
            })
            .collect();
        assert_eq!(
            fields,
            vec![(7, 100, 200, 100, 200), (7, 50, 75, 50, 75)],
            "each event carries only that line's delta"
        );
    }
}
