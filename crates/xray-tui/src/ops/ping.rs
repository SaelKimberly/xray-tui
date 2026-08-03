use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use tokio::sync::{Semaphore, mpsc};

use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{
    BuildParams, ConfigBuilder, CorePool, CoreType, MultiInboundItem, RealCoreManager, find_binary,
    resolve_core, wait_for_socks5,
};
use xray_tui_db::Database;
use xray_tui_db::models::{DnsSetting, PingResultUpdate, ProfileExtension};

use crate::AppState;
use crate::try_send_or_warn;
use crate::types::{CoreEvent, EndpointRow};

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
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
    {
        r
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for TCP ping");
        return;
    };
    let Some(proto) = row.protocols.iter().find(|p| p.id == protocol_id) else {
        state.log_trace("error", "tui::ops::ping", "Protocol not found for TCP ping");
        return;
    };
    if row.endpoint.host.is_empty() {
        state.log_trace("error", "tui::ops::ping", "Profile has no address");
        return;
    }
    let addr = row.endpoint.host.clone();
    let port = if row.endpoint.port > 0 && row.endpoint.port <= 65535 {
        row.endpoint.port as u16
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
    let config_type = proto.config_type;
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

/// Start real ping (HTTP through proxy) using a pooled warm core for single-ping reuse.
///
/// The pool is created lazily on first use. Subsequent single pings reuse the same core:
/// sing-box via SIGHUP reload, xray-core via stop+restart. Batch real ping bypasses
/// the pool entirely — it uses the multi-inbound approach instead.
pub fn start_real_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }

    // Find profile row by REAL protocol id (sub-row pings target the exact
    // protocol; different credentials → different exits).
    let endpoint;
    let protocol = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
    {
        let Some(p) = r.protocols.iter().find(|p| p.id == protocol_id) else {
            state
                .log_trace("error", "tui::ops::ping", "Protocol not found for real ping");
            return;
        };
        endpoint = r.endpoint.clone();
        p.clone()
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
    let pool = if let Some(p) = &state.core_pool {
        p.clone()
    } else {
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
    };

    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            protocol_id,
            test_type: TestType::RealPing,
        });

        let result = pool
            .ping(
                &endpoint,
                &protocol,
                &ping_url,
                &ip_api_url,
                timeout,
                retries,
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

/// Signal all running batch speed tests to stop.
pub fn stop_speed_test(state: &mut AppState) {
    state.speed_test_stop.store(true, Ordering::Relaxed);
}

/// Batch TCP ping all visible (filtered) profiles — delegates to `start_sieve`.
pub fn start_batch_ping(state: &mut AppState) {
    let profile_order = state
        .filtered_profiles()
        .enumerate()
        .map(|(i, r)| (r.active_protocol().id, i as i32))
        .collect();
    start_sieve(state, false, profile_order, true);
}

/// Batch TCP ping all visible profiles, then real ping TCP-successful targets via temp core.
pub fn start_batch_then_real_ping(state: &mut AppState) {
    let profile_order = state
        .filtered_profiles()
        .enumerate()
        .map(|(i, r)| (r.active_protocol().id, i as i32))
        .collect();
    start_sieve(state, true, profile_order, true);
}

/// Fast-ping every protocol of the selected endpoint (endpoint-scoped batch).
/// Collapsed endpoint rows with >1 protocols dispatch here — different
/// credentials can route differently, so each protocol gets its own result.
pub fn start_endpoint_batch_ping(state: &mut AppState) {
    let profile_order = selected_endpoint_order(state);
    start_sieve(state, false, profile_order, false);
}

/// Real-ping every protocol of the selected endpoint. `dedup_endpoints=false`
/// so all protocols get real-pinged (their exit IPs may differ).
pub fn start_endpoint_batch_real_ping(state: &mut AppState) {
    let profile_order = selected_endpoint_order(state);
    start_sieve(state, true, profile_order, false);
}

/// `(protocol_id, sort_order)` for every protocol of the currently selected
/// endpoint row (all of them — not just the active one).
fn selected_endpoint_order(state: &AppState) -> Vec<(i64, i32)> {
    let Some(ep_id) = state.selected_profile_id() else {
        return Vec::new();
    };
    let Some(row) = state.endpoints.iter().find(|r| r.endpoint.id == ep_id) else {
        return Vec::new();
    };
    row.protocols
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id, i as i32))
        .collect()
}

/// Flush accumulated `PingResultUpdate`s to DB. Called at page boundaries and batch end.
/// Returns `true` if the flush succeeded, `false` on failure (logged).
async fn batch_upsert_buffer(
    batch_id: &str,
    db: &Arc<Database>,
    buffer: &mut Vec<PingResultUpdate>,
) -> bool {
    if buffer.is_empty() {
        return true;
    }
    let batch = std::mem::take(buffer);
    let extensions: Vec<ProfileExtension> = batch
        .iter()
        .filter_map(|r| {
            r.latency_ms.map(|ms| ProfileExtension {
                protocol_id: r.protocol_id,
                delay: Some(ms),
                speed: None,
                sort_order: None,
                ip_info: r.ip_info.clone(),
                protocol_row: Default::default(),
            })
        })
        .collect();
    match db
        .batch_flush_ping_buffer(batch_id, &batch, &extensions)
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(
                target: "tui::ops::ping",
                "batch_flush_ping_buffer failed: {e}",
            );
            false
        }
    }
}

/// Dispatch a batch of real-ping sessions through multi-inbound config.
/// Groups profiles by `core_type` (xray/sing-box), builds multi-inbound config,
/// starts one core per group, fires concurrent HTTP pings, collects results.
/// Tracks `completed_endpoints` for wave-ordering skip logic.
/// Returns number of flushed results.
async fn dispatch_real_ping_batch(
    sessions: &[xray_tui_db::models::PingSession],
    db: &Arc<Database>,
    tx: &tokio::sync::mpsc::Sender<CoreEvent>,
    progress: &(AtomicU16, AtomicU16),
    completed_endpoints: &Arc<std::sync::Mutex<HashSet<(String, u16)>>>,
    dedup_endpoints: bool,
    batch_id: &str,
    base_proxy_port: u16,
    proxy_addr: &str,
    bin_dir: &std::path::Path,
    bin_configs_dir: &std::path::Path,
    real_ping_concurrency: usize,
    real_ping_timeout: std::time::Duration,
    real_ping_retries: u32,
    ping_url: &str,
    ip_api_url: &str,
    stop_flag: &AtomicBool,
    buffer: &mut Vec<PingResultUpdate>,
    log_tx: Option<tokio::sync::mpsc::Sender<String>>,
) -> u16 {
    let mut xray_items: Vec<(
        xray_tui_db::models::EndpointRow,
        xray_tui_db::models::PingSession,
        u16,
    )> = Vec::new();
    let mut singbox_items: Vec<(
        xray_tui_db::models::EndpointRow,
        xray_tui_db::models::PingSession,
        u16,
    )> = Vec::new();

    let mut port_counter = base_proxy_port + 1;
    for session in sessions {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        let profile = if let Ok(Some(p)) = db.get_endpoint_by_protocol_id(session.protocol_id).await
        {
            p
        } else {
            buffer.push(PingResultUpdate {
                session_id: session.id.clone(),
                protocol_id: session.protocol_id,
                status: "failed".to_string(),
                ping_type: "real".to_string(),
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some("Endpoint not found".to_string()),
            });
            let _ = tx.try_send(CoreEvent::SpeedTestResult {
                protocol_id: session.protocol_id,
                test_type: TestType::RealPing,
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some("Endpoint not found".to_string()),
            });
            continue;
        };
        let proto = Protocol::try_from_i32(session.config_type).unwrap_or(Protocol::Custom);
        let core_type = resolve_core(proto, None);
        let assigned_port = port_counter;
        port_counter += 1;

        match core_type {
            CoreType::Xray => xray_items.push((profile, session.clone(), assigned_port)),
            CoreType::SingBox => singbox_items.push((profile, session.clone(), assigned_port)),
            CoreType::Auto => xray_items.push((profile, session.clone(), assigned_port)),
        }
    }

    for (core_type, items) in [
        (CoreType::Xray, &xray_items),
        (CoreType::SingBox, &singbox_items),
    ] {
        if items.is_empty() || stop_flag.load(Ordering::Relaxed) {
            continue;
        }

        let multi_items: Vec<MultiInboundItem> = items
            .iter()
            .map(|(profile, _session, port)| MultiInboundItem {
                endpoint: &profile.endpoint,
                protocol: profile.active_protocol(),
                assigned_port: *port,
            })
            .collect();

        let base_params = BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: 0,
            http_port: None,
            listen: proxy_addr.to_string(),
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
            cache_ttl_secs: None,
        };

        let backend_config =
            match ConfigBuilder::build_multi(&multi_items, core_type, &base_params, &dns) {
                Ok(c) => c,
                Err(e) => {
                    for (_profile, session, _port) in items {
                        buffer.push(PingResultUpdate {
                            session_id: session.id.clone(),
                            protocol_id: session.protocol_id,
                            status: "failed".to_string(),
                            ping_type: "real".to_string(),
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: Some(format!("Config build: {e}")),
                        });
                    }
                    continue;
                }
            };

        let bin_path = if let Some(p) = find_binary(core_type, bin_dir) {
            p
        } else {
            for (_profile, session, _port) in items {
                buffer.push(PingResultUpdate {
                    session_id: session.id.clone(),
                    protocol_id: session.protocol_id,
                    status: "failed".to_string(),
                    ping_type: "real".to_string(),
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("Binary not found".to_string()),
                });
            }
            continue;
        };

        let log_tx = if let Some(ref tx) = log_tx {
            tx.clone()
        } else {
            let (noop_tx, mut noop_rx) = tokio::sync::mpsc::channel::<String>(256);
            tokio::spawn(async move { while noop_rx.recv().await.is_some() {} });
            noop_tx
        };
        let mut manager = RealCoreManager::new(bin_configs_dir.to_path_buf(), log_tx);

        if let Err(e) = manager
            .start(core_type, &backend_config, &bin_path, None)
            .await
        {
            for (_profile, session, _port) in items {
                buffer.push(PingResultUpdate {
                    session_id: session.id.clone(),
                    protocol_id: session.protocol_id,
                    status: "failed".to_string(),
                    ping_type: "real".to_string(),
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some(format!("Core start: {e}")),
                });
            }
            continue;
        }

        for (_profile, _session, port) in items {
            let _ = wait_for_socks5(proxy_addr, *port, std::time::Duration::from_secs(5)).await;
        }

        let sem = Arc::new(Semaphore::new(real_ping_concurrency));
        let mut handles = Vec::with_capacity(items.len());
        for (_profile, session, port) in items {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            let permit = match Arc::clone(&sem).acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let tx = tx.clone();
            let session = session.clone();
            let port = *port;
            let proxy_addr = proxy_addr.to_string();
            let ping_url = ping_url.to_string();
            let ip_api_url = ip_api_url.to_string();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                    protocol_id: session.protocol_id,
                    test_type: TestType::RealPing,
                });
                let rp_result = xray_tui_core::speed_test::real_ping(
                    &proxy_addr,
                    port,
                    &ping_url,
                    &ip_api_url,
                    real_ping_timeout,
                    real_ping_retries,
                )
                .await;
                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                    protocol_id: session.protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: rp_result.as_ref().ok().map(|r| r.latency_ms),
                    speed_bps: None,
                    ip_info: rp_result.as_ref().ok().and_then(|r| r.ip_info.clone()),
                    error: rp_result
                        .as_ref()
                        .err()
                        .map(std::string::ToString::to_string),
                });
                (session, rp_result)
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((session, Ok(rp_result))) => {
                    // Endpoint succeeded — mark so Phase 1 and future queries
                    // skip remaining profiles on it (all-visible batches only;
                    // endpoint-scoped batches must real-ping every protocol).
                    if dedup_endpoints
                        && let Some(ref addr) = session.address
                    {
                        let ep_port = session.port.unwrap_or(0) as u16;
                        completed_endpoints
                            .lock()
                            .unwrap()
                            .insert((addr.clone(), ep_port));
                    }
                    buffer.push(PingResultUpdate {
                        session_id: session.id.clone(),
                        protocol_id: session.protocol_id,
                        status: "completed".to_string(),
                        ping_type: "real".to_string(),
                        latency_ms: Some(rp_result.latency_ms as i32),
                        speed_bps: None,
                        ip_info: rp_result.ip_info.clone(),
                        error: None,
                    });
                }
                Ok((session, Err(e))) => {
                    buffer.push(PingResultUpdate {
                        session_id: session.id.clone(),
                        protocol_id: session.protocol_id,
                        status: "failed".to_string(),
                        ping_type: "real".to_string(),
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(e.to_string()),
                    });
                }
                Err(_) => {}
            }
        }

        let _ = manager.stop().await;
    }

    let flushed = buffer.len() as u16;
    batch_upsert_buffer(batch_id, db, buffer).await;
    // Clean up sessions stranded by race: demoted to real/queued while
    // endpoint already completed in this batch dispatch
    if let Err(e) = db.cancel_stranded_real_pings(batch_id).await {
        tracing::warn!(target: "ops::ping", "cancel_stranded_real_pings: {e}");
    }
    progress.1.fetch_add(flushed, Ordering::Relaxed);
    flushed
}

/// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
/// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
/// `profile_order` is the explicit `(protocol_id, sort_order)` list to test;
/// `dedup_endpoints` gates the real-ping endpoint dedup (all-visible batches
/// dedup; endpoint-scoped batches real-ping every protocol of one endpoint).
#[allow(clippy::needless_collect)]
pub fn start_sieve(
    state: &mut AppState,
    real_ping_enabled: bool,
    profile_order: Vec<(i64, i32)>,
    dedup_endpoints: bool,
) {
    if profile_order.is_empty() {
        state.log_trace(
            "info",
            "tui::ops::ping",
            if real_ping_enabled {
                "No profiles to test"
            } else {
                "No profiles to ping"
            },
        );
        return;
    }

    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    let state_log_tx = state.core_log_tx.clone();
    let db = state.db.clone();
    let stop_flag = state.speed_test_stop.clone();
    let batch_id = uuid::Uuid::new_v4().to_string();
    let page_size = state.config.speed_test.batch_page_size.max(1);

    let fast_timeout = *state.config.speed_test.tcp_timeout_secs;
    let real_ping_timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;
    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let proxy_addr = state.config.inbound.listen.clone();
    let base_proxy_port = state.config.inbound.socks_port;

    let config_dir_path = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir_path.join("bin");
    let bin_configs_dir = config_dir_path.join("binConfigs");
    let real_ping_concurrency = state.config.speed_test.real_ping_concurrency.max(1);

    // Phase 2 wave-ordered window: profiles to accumulate before waking Phase 2
    let real_ping_window = state.config.speed_test.real_ping_window.max(1);

    // Set up shared batch progress for status bar display
    #[allow(clippy::cast_possible_truncation)]
    let total_count = profile_order.len() as u16;
    let progress = Arc::new((AtomicU16::new(total_count), AtomicU16::new(0)));
    state.batch_progress = Some(progress.clone());
    tokio::spawn(async move {
        // 1. Snapshot visible profiles into ping_sessions table
        let count = db
            .create_ping_batch(&batch_id, None, Some(&profile_order))
            .await;
        let count = match count {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(target: "tui::ops::ping", "create_ping_batch failed: {e}");
                progress.0.store(0, Ordering::Relaxed);
                let _ = tx.try_send(CoreEvent::BatchProgress {
                    total: 0,
                    completed: 0,
                });
                return;
            }
        };
        if count == 0 {
            tracing::warn!(target: "tui::ops::ping", "create_ping_batch returned 0 — no matching profile_cores?");
            progress.0.store(0, Ordering::Relaxed);
            let _ = tx.try_send(CoreEvent::BatchProgress {
                total: 0,
                completed: 0,
            });
            return;
        }
        // Adjust progress total to match actual inserted count
        #[allow(clippy::cast_possible_truncation)]
        if (count as u16) != total_count {
            progress.0.store(count as u16, Ordering::Relaxed);
        }

        let mut buffer: Vec<PingResultUpdate> = Vec::new();
        let mut stall_count = 0u32;
        // Phase 1 dedup cache: key=(addr, port) → (latency_ms, error)
        let mut fast_cache: std::collections::HashMap<
            (String, u16),
            (Option<i32>, Option<String>),
        > = std::collections::HashMap::new();

        // 2. Initialize fast ping manager (Phase 2 uses multi-inbound directly, no RealPingManager)
        let fmgr = xray_tui_core::FastPingManager::new(fast_timeout);

        // ── Shared state for concurrent Phase 1+2 ──
        // Endpoints that had a successful real ping — no more profiles on these need testing.
        let completed_endpoints: Arc<std::sync::Mutex<HashSet<(String, u16)>>> =
            Arc::new(std::sync::Mutex::new(HashSet::new()));
        let mut accumulated_successes = 0usize;

        // Phase 2 consumer — spawned first so it's ready for wake-up from Phase 1
        let mut phase2_handle: Option<(mpsc::Sender<()>, tokio::task::JoinHandle<()>)> =
            if real_ping_enabled {
                let (phase2_tx, mut phase2_rx) = mpsc::channel::<()>(1);
                let phase2_completed = completed_endpoints.clone();
                let phase2_db = db.clone();
                let phase2_tx_ev = tx.clone();
                let phase2_stop = stop_flag.clone();
                let phase2_progress = progress.clone();
                let phase2_window = real_ping_window;
                let _phase2_page_size = page_size;
                let phase2_timeout = real_ping_timeout;
                let phase2_retries = retries;
                let phase2_ping_url = ping_url.clone();
                let phase2_ip_api_url = ip_api_url.clone();
                let phase2_proxy_addr = proxy_addr.clone();
                let phase2_base_port = base_proxy_port;
                let phase2_bin_dir = bin_dir.clone();
                let phase2_configs_dir = bin_configs_dir.clone();
                let phase2_concurrency = real_ping_concurrency;
                let phase2_batch_id = batch_id.clone();
                let phase2_dedup_endpoints = dedup_endpoints;
                let phase2_log_tx = state_log_tx.clone();

                let handle = tokio::spawn(async move {
                    let mut buffer: Vec<PingResultUpdate> = Vec::new();

                    loop {
                        if phase2_rx.recv().await == Some(()) {
                            // Woken up — fall through to poll
                        } else {
                            // Sender dropped — final drain
                            loop {
                                if phase2_stop.load(Ordering::Relaxed) {
                                    break;
                                }
                                let sessions = match phase2_db
                                    .get_batch_for_real_ping(
                                        &phase2_batch_id,
                                        phase2_window,
                                        phase2_dedup_endpoints,
                                    )
                                    .await
                                {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!(target: "tui::ops::ping", "get_batch_for_real_ping failed: {e}");
                                        break;
                                    }
                                };
                                if sessions.is_empty() {
                                    break;
                                }
                                let flushed = dispatch_real_ping_batch(
                                    &sessions,
                                    &phase2_db,
                                    &phase2_tx_ev,
                                    &phase2_progress,
                                    &phase2_completed,
                                    phase2_dedup_endpoints,
                                    &phase2_batch_id,
                                    phase2_base_port,
                                    &phase2_proxy_addr,
                                    &phase2_bin_dir,
                                    &phase2_configs_dir,
                                    phase2_concurrency,
                                    phase2_timeout,
                                    phase2_retries,
                                    &phase2_ping_url,
                                    &phase2_ip_api_url,
                                    &phase2_stop,
                                    &mut buffer,
                                    phase2_log_tx.clone(),
                                )
                                .await;
                                if flushed == 0 {
                                    break;
                                }
                            }
                            break;
                        }

                        // Normal poll (after wake-up)
                        loop {
                            if phase2_stop.load(Ordering::Relaxed) {
                                break;
                            }
                            let sessions = match phase2_db
                                .get_batch_for_real_ping(
                                    &phase2_batch_id,
                                    phase2_window,
                                    phase2_dedup_endpoints,
                                )
                                .await
                            {
                                Ok(s) => s,
                                Err(e) => {
                                    tracing::error!(target: "tui::ops::ping", "get_batch_for_real_ping failed: {e}");
                                    break;
                                }
                            };
                            if sessions.is_empty() {
                                break;
                            }
                            let flushed = dispatch_real_ping_batch(
                                &sessions,
                                &phase2_db,
                                &phase2_tx_ev,
                                &phase2_progress,
                                &phase2_completed,
                                phase2_dedup_endpoints,
                                &phase2_batch_id,
                                phase2_base_port,
                                &phase2_proxy_addr,
                                &phase2_bin_dir,
                                &phase2_configs_dir,
                                phase2_concurrency,
                                phase2_timeout,
                                phase2_retries,
                                &phase2_ping_url,
                                &phase2_ip_api_url,
                                &phase2_stop,
                                &mut buffer,
                                phase2_log_tx.clone(),
                            )
                            .await;
                            if flushed == 0 {
                                break;
                            }
                        }
                    }

                    // Final buffer flush for any remaining Phase 2 results
                    batch_upsert_buffer(&phase2_batch_id, &phase2_db, &mut buffer).await;
                });

                Some((phase2_tx, handle))
            } else {
                None
            };

        // Sender clone for Phase 1 to wake up Phase 2
        let phase2_sender: Option<mpsc::Sender<()>> =
            phase2_handle.as_ref().map(|(tx, _)| tx.clone());

        // ── Phase 1: Fast Ping ──
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let sessions = db
                .get_batch_page_ready_for_fast_ping(&batch_id, page_size)
                .await;
            let Ok(sessions) = sessions else {
                tracing::error!(target: "tui::ops::ping", "get_batch_page_ready_for_fast_ping failed");
                progress.0.store(0, Ordering::Relaxed);
                break;
            };
            if sessions.is_empty() {
                break;
            }

            let mut demoted_count = 0u16;
            for session in &sessions {
                let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                    protocol_id: session.protocol_id,
                    test_type: TestType::TcpPing,
                });

                let addr = match &session.address {
                    Some(a) => a.clone(),
                    None => continue,
                };
                let port = session.port.unwrap_or(0) as u16;
                if port == 0 {
                    continue;
                }

                // Track sessions demoted to Phase 2 (no buffer push this page)
                let mut demoted_here = false;

                // Dedup: skip ping if (addr, port) already tested in this page
                let cache_key = (addr.clone(), port);
                let pushed = if let Some((cached_ms, cached_err)) = fast_cache.get(&cache_key) {
                    // Reuse cached result
                    if let Some(ms) = cached_ms {
                        if real_ping_enabled {
                            let ep_done = completed_endpoints
                                .lock()
                                .unwrap()
                                .contains(&(addr.clone(), port));
                            if ep_done {
                                // Endpoint already succeeded in Phase 2 — cancel this profile
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "cancelled".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Endpoint already tested in Real Ping".to_string()),
                                });
                                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                    protocol_id: session.protocol_id,
                                    test_type: TestType::TcpPing,
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Endpoint already tested in Real Ping".to_string()),
                                });
                                demoted_here = false;
                                false
                            } else {
                                // Demote to real ping
                                let _ = db
                                    .update_session_ping_type(&session.id, "real", "queued")
                                    .await;
                                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                    protocol_id: session.protocol_id,
                                    test_type: TestType::TcpPing,
                                    latency_ms: Some(*ms as u64),
                                    speed_bps: None,
                                    ip_info: None,
                                    error: None,
                                });
                                demoted_here = true;
                                false
                            }
                        } else {
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                protocol_id: session.protocol_id,
                                status: "completed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: Some(*ms),
                                speed_bps: None,
                                ip_info: None,
                                error: None,
                            });
                            true
                        }
                    } else {
                        buffer.push(PingResultUpdate {
                            session_id: session.id.clone(),
                            protocol_id: session.protocol_id,
                            status: "failed".to_string(),
                            ping_type: "fast".to_string(),
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: cached_err.clone(),
                        });
                        true
                    }
                } else {
                    match fmgr.ping(session.config_type, &addr, port).await {
                        Ok(dur) => {
                            #[allow(clippy::cast_possible_truncation)]
                            let ms = dur.as_millis() as i32;
                            if real_ping_enabled {
                                let ep_done = completed_endpoints
                                    .lock()
                                    .unwrap()
                                    .contains(&(addr.clone(), port));
                                if ep_done {
                                    // Endpoint already succeeded in Phase 2 — cancel
                                    buffer.push(PingResultUpdate {
                                        session_id: session.id.clone(),
                                        protocol_id: session.protocol_id,
                                        status: "cancelled".to_string(),
                                        ping_type: "fast".to_string(),
                                        latency_ms: None,
                                        speed_bps: None,
                                        ip_info: None,
                                        error: Some(
                                            "Endpoint already tested in Real Ping".to_string(),
                                        ),
                                    });
                                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                        protocol_id: session.protocol_id,
                                        test_type: TestType::TcpPing,
                                        latency_ms: None,
                                        speed_bps: None,
                                        ip_info: None,
                                        error: Some(
                                            "Endpoint already tested in Real Ping".to_string(),
                                        ),
                                    });
                                    fast_cache.insert(cache_key, (Some(ms), None));
                                    demoted_here = false;
                                    false
                                } else {
                                    // Demote to real ping
                                    let _ = db
                                        .update_session_ping_type(&session.id, "real", "queued")
                                        .await;
                                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                        protocol_id: session.protocol_id,
                                        test_type: TestType::TcpPing,
                                        latency_ms: Some(ms as u64),
                                        speed_bps: None,
                                        ip_info: None,
                                        error: None,
                                    });
                                    fast_cache.insert(cache_key, (Some(ms), None));
                                    demoted_here = true;
                                    false
                                }
                            } else {
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "completed".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: Some(ms),
                                    speed_bps: None,
                                    ip_info: None,
                                    error: None,
                                });
                                fast_cache.insert(cache_key, (Some(ms), None));
                                true
                            }
                        }
                        Err(xray_tui_core::ping::PingError::NotSupported) => {
                            if real_ping_enabled {
                                // Check if endpoint already completed in Phase 2
                                let ep_done = completed_endpoints
                                    .lock()
                                    .unwrap()
                                    .contains(&(addr.clone(), port));
                                if ep_done {
                                    buffer.push(PingResultUpdate {
                                        session_id: session.id.clone(),
                                        protocol_id: session.protocol_id,
                                        status: "cancelled".to_string(),
                                        ping_type: "fast".to_string(),
                                        latency_ms: None,
                                        speed_bps: None,
                                        ip_info: None,
                                        error: Some(
                                            "Endpoint already tested in Real Ping".to_string(),
                                        ),
                                    });
                                    demoted_here = false;
                                    false
                                } else {
                                    // Demote to real ping for Phase 2 — no buffer push
                                    let _ = db
                                        .update_session_ping_type(&session.id, "real", "queued")
                                        .await;
                                    // Don't cache — NotSupported is protocol-specific
                                    demoted_here = true;
                                    false
                                }
                            } else {
                                // No real ping phase — emit Cancelled immediately
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "cancelled".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Not supported by fast ping".to_string()),
                                });
                                true
                            }
                        }
                        Err(e) => {
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                protocol_id: session.protocol_id,
                                status: "failed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: None,
                                speed_bps: None,
                                ip_info: None,
                                error: Some(e.to_string()),
                            });
                            fast_cache.insert(cache_key, (None, Some(e.to_string())));
                            true
                        }
                    } // end else (non-cached path)
                };
                // Send SpeedTestResult if we pushed to buffer
                // (demoted sessions already emitted above)
                if pushed {
                    let last = buffer.last().unwrap();
                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                        protocol_id: session.protocol_id,
                        test_type: TestType::TcpPing,
                        latency_ms: last.latency_ms.map(|v| v as u64),
                        speed_bps: None,
                        ip_info: None,
                        error: last.error.clone(),
                    });
                }
                if demoted_here {
                    demoted_count += 1;
                    accumulated_successes += 1;
                }
            }
            // Flush buffer at page boundary
            let flushed = buffer.len() as u16;
            let flush_ok = batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);

            // Stall guard: break if too many consecutive iterations with NO work
            // (demoted sessions count as progress — they'll be picked up by Phase 2).
            if flushed == 0 && demoted_count == 0 {
                stall_count += 1;
            } else if !flush_ok {
                stall_count += 1;
            } else {
                stall_count = 0;
            }
            if stall_count > 3 {
                tracing::warn!(
                    target: "tui::ops::ping",
                    "Phase 1 stall guard triggered after {stall_count} iterations",
                );
                break;
            }
            // Wake up Phase 2 if enough demoted sessions accumulated
            if accumulated_successes >= real_ping_window {
                if let Some(ref tx) = phase2_sender {
                    let _ = tx.try_send(());
                }
                accumulated_successes = 0;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // After Phase 1: mark any remaining 'queued' fast sessions as failed
        // (orphan profiles with no endpoint or empty address/port)
        if let Ok(remaining) = db
            .get_batch_page_ready_for_fast_ping(&batch_id, 65536)
            .await
        {
            for session in &remaining {
                buffer.push(PingResultUpdate {
                    session_id: session.id.clone(),
                    protocol_id: session.protocol_id,
                    status: "failed".to_string(),
                    ping_type: "fast".to_string(),
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("No address/port".to_string()),
                });
            }
            let flushed = remaining.len() as u16;
            batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);
        }

        // ── Phase 2: Real Ping — running concurrently as spawned task ──
        // Signal Phase 2 to drain (drop sender) and wait for completion.
        if let Some((_sender, handle)) = phase2_handle.take() {
            drop(_sender);
            let _ = handle.await;
        }

        // Final flush + cancel remaining + cleanup
        if stop_flag.load(Ordering::Relaxed) {
            // Emit Cancelled events for any non-terminal sessions
            if let Ok(sessions) = db.get_ping_sessions_by_batch(&batch_id, 65536, 0).await {
                for session in &sessions {
                    if session.status == "completed"
                        || session.status == "failed"
                        || session.status == "cancelled"
                    {
                        continue;
                    }
                    let test_type = if session.ping_type == "real" {
                        TestType::RealPing
                    } else {
                        TestType::TcpPing
                    };
                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                        protocol_id: session.protocol_id,
                        test_type,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some("Cancelled".to_string()),
                    });
                }
            }
            let _ = db.cancel_ping_batch(&batch_id).await;
        }
        let flushed = buffer.len() as u16;
        batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
        progress.1.fetch_add(flushed, Ordering::Relaxed);
        let _ = db.cleanup_ping_batch(&batch_id).await;
        progress.0.store(0, Ordering::Relaxed); // signal batch is done; enables stale cleanup
        let _ = tx.try_send(CoreEvent::BatchProgress {
            total: 0,
            completed: 0,
        });
    });
}

/// Remove profiles whose extension.delay == Some(-1) (failed TCP ping).
pub async fn remove_failed_servers(state: &mut AppState) {
    let to_remove: Vec<i64> = state
        .endpoints
        .iter()
        .filter(|r| r.extensions.values().any(|e| e.delay == Some(-1)))
        .map(|r| r.endpoint.id)
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
