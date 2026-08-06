use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{CorePool, SinglePingReq};

use crate::AppState;
use crate::state::{link_is_failed, load_protocol_with_config};
use crate::try_send_or_warn;
use crate::types::CoreEvent;

/// Start TCP ping on the given profile. Returns immediately; result arrives via `CoreEvent`.
pub fn start_tcp_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Test already in progress for this profile",
        );
        return;
    }
    // Find the profile and extract address:port. Lookup is by REAL protocol
    // id so sub-row pings target the exact protocol (not the endpoint).
    let row = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
    {
        r
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for TCP ping");
        return;
    };
    let Some(link) = row
        .links
        .iter()
        .find(|l| l.protocol_id.get() == protocol_id)
    else {
        state.log_trace("error", "tui::ops::ping", "Protocol not found for TCP ping");
        return;
    };
    let Some(proto) = row.protocols.get(&link.protocol_id) else {
        state.log_trace(
            "error",
            "tui::ops::ping",
            "Protocol row not found for TCP ping",
        );
        return;
    };
    if row.endpoint.host.is_empty() {
        state.log_trace("error", "tui::ops::ping", "Profile has no address");
        return;
    }
    let addr = row.endpoint.host.clone();
    let port = if row.endpoint.port > 0 {
        row.endpoint.port
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile has invalid port");
        return;
    };

    let tx = if let Some(tx) = &state.core_event_tx {
        tx.clone()
    } else {
        state.log_trace(
            "error",
            "tui::ops::ping",
            "Core event channel not initialized",
        );
        return;
    };

    state.testing_details.insert(protocol_id, TestType::TcpPing);
    state.testing_profiles.insert(protocol_id);
    // The fast-ping adapters dispatch on the core Protocol kind.
    let config_type = Protocol::from(proto.proto_kind).to_i32();
    let timeout_dur = *state.config.speed_test.tcp_timeout_secs;

    tokio::spawn(async move {
        let fmgr = xray_tui_core::FastPingManager::new(timeout_dur);
        let result = fmgr.ping(config_type, &addr, port).await;
        let (latency_ms, error) = match result {
            Ok(dur) => (Some(dur.as_millis() as u64), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::TcpPing,
                latency_ms,
                speed_bps: None,
                ip_info: None,
                error,
            },
            "tcp_ping_result",
        );
    });
}

/// Get the shared core pool, creating it lazily on first use.
///
/// Single real pings draw ports from this pool's single allocator, so a warm
/// pooled core and any future batch core can never collide on a port.
fn get_or_create_pool(state: &mut AppState) -> Arc<CorePool> {
    if let Some(p) = &state.core_pool {
        return p.clone();
    }
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");
    let proxy_addr = state.config.inbound.listen.clone();
    let base_port = state.config.inbound.socks_port;
    let pool = Arc::new(CorePool::new(
        bin_dir,
        bin_configs_dir,
        proxy_addr,
        base_port,
    ));
    state.core_pool = Some(pool.clone());
    pool
}

/// Start real ping (HTTP through proxy) using a pooled warm core for
/// single-ping reuse.
///
/// The pool is created lazily on first use. Subsequent single pings reuse the
/// same core: sing-box via SIGHUP reload, xray-core via stop+restart. The
/// `Protocol` row is re-loaded WITH its deferred `config` included inside the
/// spawned task — the pool's `ConfigBuilder::build` refuses unloaded configs.
pub fn start_real_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }

    // Find profile row by REAL protocol id (sub-row pings target the exact
    // protocol; different credentials → different exits).
    let endpoint;
    let link;
    let protocol_id_typed;
    if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
    {
        let Some(l) = r.links.iter().find(|l| l.protocol_id.get() == protocol_id) else {
            state.log_trace(
                "error",
                "tui::ops::ping",
                "Protocol not found for real ping",
            );
            return;
        };
        endpoint = r.endpoint.clone();
        link = l.clone();
        protocol_id_typed = l.protocol_id;
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for real ping");
        return;
    };

    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state
        .testing_details
        .insert(protocol_id, TestType::RealPing);
    state.testing_profiles.insert(protocol_id);

    // Lazily create the core pool on first use
    let pool = get_or_create_pool(state);
    let db = state.db.clone();

    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            protocol_id,
            test_type: TestType::RealPing,
        });

        // Load the protocol row WITH its config included (the config builders
        // and `shadowsocks_method` refuse unloaded configs).
        let protocol = match load_protocol_with_config(&db, protocol_id_typed).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        protocol_id,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some("Protocol row not found for real ping".to_string()),
                    },
                    "real_ping_protocol_missing",
                );
                return;
            }
            Err(e) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        protocol_id,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(format!("Failed to load protocol: {e}")),
                    },
                    "real_ping_protocol_load",
                );
                return;
            }
        };

        let result = pool
            .ping(
                &endpoint,
                &link,
                &protocol,
                SinglePingReq {
                    ping_url: &ping_url,
                    ip_api_url: &ip_api_url,
                    timeout,
                    retries,
                },
            )
            .await;

        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::RealPing,
                latency_ms: result.latency_ms,
                speed_bps: None,
                ip_info: result.ip_info,
                error: result.error,
            },
            "real_ping_result",
        );
    });
}

/// Start speed test (download through proxy) on the given profile.
pub fn start_speed_test(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }
    if state.connected_core.is_none() {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Core not connected — proxy required for speed test",
        );
        return;
    }
    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state
        .testing_details
        .insert(protocol_id, TestType::SpeedTest);
    state.testing_profiles.insert(protocol_id);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;
    let test_url = "http://cachefly.cachefly.net/1mb.test".to_string();
    let min_dur = std::time::Duration::from_secs(3);
    let max_dur = std::time::Duration::from_secs(10);

    tokio::spawn(async move {
        let result = xray_tui_core::speed_test::speed_test(
            &proxy_addr,
            proxy_port,
            &test_url,
            min_dur,
            max_dur,
        )
        .await;
        let (speed_bps, error) = match result {
            Ok(bps) => (Some(bps), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::SpeedTest,
                latency_ms: None,
                speed_bps,
                ip_info: None,
                error,
            },
            "speed_test_result",
        );
    });
}

/// Start UDP test through the connected proxy.
pub fn start_udp_test(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }
    if state.connected_core.is_none() {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Core not connected — proxy required for UDP test",
        );
        return;
    }
    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state.testing_details.insert(protocol_id, TestType::UdpTest);
    state.testing_profiles.insert(protocol_id);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;

    tokio::spawn(async move {
        let result = xray_tui_core::speed_test::udp_test(
            &proxy_addr,
            proxy_port,
            std::time::Duration::from_secs(5),
        )
        .await;
        let (latency_ms, error) = match result {
            Ok(dur) => (Some(dur.as_millis() as u64), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::UdpTest,
                latency_ms,
                speed_bps: None,
                ip_info: None,
                error,
            },
            "udp_test_result",
        );
    });
}

/// Signal all running tests to stop.
pub fn stop_speed_test(state: &mut AppState) {
    state.speed_test_stop.store(true, Ordering::Relaxed);
}

/// Remove endpoints whose links carry a persisted failure marker (the old
/// `extension.delay == Some(-1)` sweep, now driven by `ProfileStats.error`).
pub async fn remove_failed_servers(state: &mut AppState) {
    let to_remove: Vec<i64> = state
        .endpoints
        .iter()
        .filter(|r| r.links.iter().any(link_is_failed))
        .map(|r| r.endpoint.id.get())
        .collect();
    let count = to_remove.len();
    for id in to_remove {
        crate::ops::profiles::delete_profile(state, id).await;
    }
    state.multi_select.clear();
    state.log_trace(
        "info",
        "tui::ops::ping",
        &format!("Removed {count} failed server(s)"),
    );
}
