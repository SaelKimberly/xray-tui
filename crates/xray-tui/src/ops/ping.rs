use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use tokio::sync::{Semaphore, mpsc};

use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{BuildParams, ConfigBuilder, CoreManager, CoreType, find_binary, resolve_core};
use xray_tui_db::Database;
use xray_tui_db::models::{DnsSetting, PingResultUpdate, ProfileExtension};

use crate::AppState;
use crate::try_send_or_warn;
use crate::types::{CoreEvent, EndpointRow};

/// Start TCP ping on the given profile. Returns immediately; result arrives via `CoreEvent`.
pub fn start_tcp_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        state.log_trace("warn", "tui", "Test already in progress for this profile");
        return;
    }
    // Find the profile and extract address:port
    let row = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id == protocol_id)
    {
        r
    } else {
        state.log_trace("error", "tui", "Profile not found for TCP ping");
        return;
    };
    if row.endpoint.host.is_empty() {
        state.log_trace("error", "tui", "Profile has no address");
        return;
    }
    let addr = row.endpoint.host.clone();
    let port = if row.endpoint.port > 0 && row.endpoint.port <= 65535 {
        row.endpoint.port as u16
    } else {
        state.log_trace("error", "tui", "Profile has invalid port");
        return;
    };

    let tx = if let Some(tx) = &state.core_event_tx {
        tx.clone()
    } else {
        state.log_trace("error", "tui", "Core event channel not initialized");
        return;
    };

    state.testing_details.insert(protocol_id, TestType::TcpPing);
    state.testing_profiles.insert(protocol_id);
    let config_type = row.active_protocol().config_type;
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

/// Start real ping (HTTP through proxy) by starting a temporary core for the profile.
pub fn start_real_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }

    // Find profile row and resolve core
    let endpoint;
    let protocol = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id == protocol_id)
    {
        endpoint = r.endpoint.clone();
        r.active_protocol().clone()
    } else {
        state.log_trace("error", "tui", "Profile not found for real ping");
        return;
    };
    let p = protocol;
    let resolved_protocol = Protocol::try_from_i32(p.config_type).unwrap_or(Protocol::Custom);
    let core_override = state
        .config
        .core
        .protocol_core_overrides
        .get(resolved_protocol.to_string().as_str())
        .and_then(|s| s.parse::<CoreType>().ok());
    let core_type = resolve_core(resolved_protocol, core_override);
    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state
        .testing_details
        .insert(protocol_id, TestType::RealPing);
    state.testing_profiles.insert(protocol_id);

    // Build params for the temp core
    let params = BuildParams {
        v2ray_api_enabled: false,
        clash_api_enabled: false,
        log_level: "error".to_string(),
        socks_port: 0, // overridden by temp allocation
        http_port: None,
        listen: state.config.inbound.listen.clone(),
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

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;
    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout_dur = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        // 1. Create temp config directory
        let temp_id = uuid::Uuid::new_v4().to_string();
        let temp_dir = bin_configs_dir.join(&temp_id);
        if let Err(e) = tokio::fs::create_dir_all(&temp_dir).await {
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some(format!("Failed to create temp dir: {e}")),
                },
                "real_ping_tempdir_err",
            );
            return;
        }

        // 2. Build config
        let backend_config =
            match ConfigBuilder::build(&endpoint, &p, core_type, &params, &[], &dns) {
                Ok(c) => c,
                Err(e) => {
                    try_send_or_warn(
                        &tx,
                        CoreEvent::SpeedTestResult {
                            protocol_id,
                            test_type: TestType::RealPing,
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: Some(format!("Config build failed: {e}")),
                        },
                        "real_ping_config_err",
                    );
                    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                    return;
                }
            };

        // 3. Find binary
        let bin_path = if let Some(p) = find_binary(core_type, &bin_dir) {
            p
        } else {
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("Core binary not found".to_string()),
                },
                "real_ping_binary_err",
            );
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
            return;
        };

        // 4. Start core (discard log lines from temp core)
        let (log_line_tx, mut _log_line_rx) = mpsc::channel::<String>(512);
        let mut manager = CoreManager::with_log_channel(temp_dir.clone(), log_line_tx);
        if let Err(e) = manager.start(core_type, &backend_config, &bin_path, None).await {
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some(format!("Failed to start core: {e}")),
                },
                "real_ping_start_err",
            );
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
            return;
        }

        // 5. Wait for core readiness
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // 6. Run real ping through the temp core's proxy
        let result = xray_tui_core::speed_test::real_ping(
            &proxy_addr,
            proxy_port,
            &ping_url,
            &ip_api_url,
            timeout_dur,
            retries,
        )
        .await;

        let (latency_ms, ip_info, error) = match result {
            Ok(rp_result) => (Some(rp_result.latency_ms), rp_result.ip_info, None),
            Err(e) => (None, None, Some(e.to_string())),
        };

        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::RealPing,
                latency_ms,
                speed_bps: None,
                ip_info,
                error,
            },
            "real_ping_result",
        );

        // 7. Stop core and clean up
        let _ = manager.stop().await;
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
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
            "tui",
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
            "tui",
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

/// Batch TCP ping all visible (filtered) profiles — delegates to `start_batch_sieve`.
pub fn start_batch_ping(state: &mut AppState) {
    start_batch_sieve(state, false);
}

/// Batch TCP ping all visible profiles, then real ping TCP-successful targets via temp core.
pub fn start_batch_then_real_ping(state: &mut AppState) {
    start_batch_sieve(state, true);
}

/// Flush accumulated `PingResultUpdate`s to DB. Called at page boundaries and batch end.
async fn batch_upsert_buffer(
    batch_id: &str,
    db: &Arc<Database>,
    buffer: &mut Vec<PingResultUpdate>,
) {
    if buffer.is_empty() {
        return;
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
    let _ = db
        .batch_flush_ping_buffer(batch_id, &batch, &extensions)
        .await;
}

/// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
#[allow(clippy::needless_collect)]
pub fn start_batch_sieve(state: &mut AppState, real_ping_enabled: bool) {
    let visible: Vec<&EndpointRow> = state.filtered_profiles().collect();
    if visible.is_empty() {
        state.log_trace(
            "info",
            "tui",
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

    // Set up shared batch progress for status bar display
    let total_count = visible.len() as u16;
    let progress = Arc::new((AtomicU16::new(total_count), AtomicU16::new(0)));
    state.batch_progress = Some(progress.clone());

    tokio::spawn(async move {
        // 1. Snapshot visible profiles into ping_sessions table
        let count = db.create_ping_batch(&batch_id, None).await;
        let count = match count {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(target: "tui", "create_ping_batch failed: {e}");
                progress.0.store(0, Ordering::Relaxed);
                return;
            }
        };
        if count == 0 {
            tracing::warn!(target: "tui", "create_ping_batch returned 0 — no matching profile_cores?");
            progress.0.store(0, Ordering::Relaxed);
            return;
        }

        let mut buffer: Vec<PingResultUpdate> = Vec::new();

        // 2. Initialize managers
        let fmgr = xray_tui_core::FastPingManager::new(fast_timeout);
        let rmgr = xray_tui_core::RealPingManager {
            ping_url,
            ip_api_url,
            timeout: real_ping_timeout,
            retries,
            proxy_addr,
            base_proxy_port,
            bin_dir,
            bin_configs_dir,
        };

        // ── Phase 1: Fast Ping ──
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let sessions = db
                .get_batch_page_ready_for_fast_ping(&batch_id, page_size)
                .await;
            let Ok(sessions) = sessions else {
                tracing::error!(target: "tui", "get_batch_page_ready_for_fast_ping failed");
                progress.0.store(0, Ordering::Relaxed);
                break;
            };
            if sessions.is_empty() {
                break;
            }

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

                match fmgr.ping(session.config_type, &addr, port).await {
                    Ok(dur) => {
                        #[allow(clippy::cast_possible_truncation)]
                        let ms = dur.as_millis() as i32;
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
                    }
                    Err(xray_tui_core::ping::PingError::NotSupported) => {
                        if real_ping_enabled {
                            // Demote to real ping for Phase 2
                            let _ = db
                                .update_session_ping_type(&session.id, "real", "queued")
                                .await;
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
                            let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                protocol_id: session.protocol_id,
                                test_type: TestType::TcpPing,
                                latency_ms: None,
                                speed_bps: None,
                                ip_info: None,
                                error: Some("Not supported by fast ping".to_string()),
                            });
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
                    }
                }
                // Send SpeedTestResult to TUI for immediate feedback
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
            // Flush buffer at page boundary
            let flushed = buffer.len() as u16;
            batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // ── Phase 2: Real Ping (only if enabled) ──
        if real_ping_enabled {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                let sessions = db
                    .get_batch_page_ready_for_real_ping(&batch_id, page_size)
                    .await;
                let Ok(sessions) = sessions else {
                    tracing::error!(target: "tui", "get_batch_page_ready_for_real_ping failed");
                    progress.0.store(0, Ordering::Relaxed);
                    break;
                };
                if sessions.is_empty() {
                    break;
                }

                let sem = Arc::new(Semaphore::new(real_ping_concurrency));
                let mut handles = Vec::with_capacity(sessions.len());
                for session in &sessions {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    // Load profile for this session
                    let profile = match db.get_endpoint(session.protocol_id).await {
                        Ok(Some(p)) => p,
                        _ => continue,
                    };
                    let permit = match Arc::clone(&sem).acquire_owned().await {
                        Ok(p) => p,
                        Err(_) => break,
                    };
                    let rmgr = rmgr.clone();
                    let tx = tx.clone();
                    let session = session.clone();

                    handles.push(tokio::spawn(async move {
                        let _permit = permit;
                        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                            protocol_id: session.protocol_id,
                            test_type: TestType::RealPing,
                        });
                        let result = rmgr
                            .real_ping(
                                &profile.endpoint,
                                profile.active_protocol(),
                                session.config_type,
                            )
                            .await;
                        let _ = tx.try_send(CoreEvent::SpeedTestResult {
                            protocol_id: session.protocol_id,
                            test_type: TestType::RealPing,
                            latency_ms: result.latency_ms,
                            speed_bps: None,
                            ip_info: result.ip_info.clone(),
                            error: result.error.clone(),
                        });
                        PingResultUpdate {
                            session_id: session.id.clone(),
                            protocol_id: session.protocol_id,
                            status: if result.error.is_none() {
                                "completed".to_string()
                            } else {
                                "failed".to_string()
                            },
                            ping_type: "real".to_string(),
                            latency_ms: result.latency_ms.map(|v| v as i32),
                            speed_bps: None,
                            ip_info: result.ip_info.clone(),
                            error: result.error,
                        }
                    }));
                }

                // Collect results
                for handle in handles {
                    if let Ok(update) = handle.await {
                        buffer.push(update);
                    }
                }
                let flushed = buffer.len() as u16;
                batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
                progress.1.fetch_add(flushed, Ordering::Relaxed);
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
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
    state.log_trace("info", "tui", &format!("Removed {count} failed server(s)"));
}
