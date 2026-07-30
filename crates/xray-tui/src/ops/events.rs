use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use xray_tui_core::CoreType;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::models::{ProfileExtension, ServerStat};

use crate::AppState;
use crate::format_now;
use crate::types::{AppMode, CoreEvent, SettingsMode, SplitRightPane};

/// Poll core event channel and update state accordingly.
pub async fn poll_core_events(state: &mut AppState) {
    // Clean up stale batch_progress when the task has finished silently
    // (e.g., create_ping_batch returned 0, no TestTypeUpdate events were ever sent).
    if state.batch_progress.is_some()
        && state.testing_profiles.is_empty()
        && state
            .batch_progress
            .as_ref()
            .map(|p| p.0.load(Ordering::Relaxed))
            == Some(0)
    {
        state.batch_progress = None;
    }
    while let Some(rx) = state.core_event_rx.as_mut() {
        let event = match rx.try_recv() {
            Ok(event) => event,
            Err(
                tokio::sync::mpsc::error::TryRecvError::Empty
                | tokio::sync::mpsc::error::TryRecvError::Disconnected,
            ) => break,
        };
        match event {
            CoreEvent::Connected(core_type) => {
                state.connected_core = Some(core_type);
                state.connecting = false;
                state.connection_error = None;
                state.log_trace("info", "core", &format!("Connected [{core_type}]"));
            }
            CoreEvent::Disconnected => {
                // Ignore stale Disconnected if already reconnecting
                if !state.connecting {
                    state.connected_core = None;
                    state.connected_protocol_id = None;
                    state.log_trace("info", "core", "Core process stopped");
                }
            }
            CoreEvent::Error(err) => {
                state.connection_error = Some(err.clone());
                state.connecting = false;
                state.connected_core = None;
                state.log_trace("error", "core", &format!("Connection error: {err}"));
                state.connected_protocol_id = None;
            }
            CoreEvent::StatsError(msg) => {
                state.connection_error = Some(msg.clone());
                state.log_trace("warning", "core", &format!("Stats error: {msg}"));
            }
            CoreEvent::StatsUpdate {
                protocol_id,
                today_up,
                today_down,
                total_up,
                total_down,
            } => {
                state.connection_error = None;
                let stats = ServerStat {
                    protocol_id,
                    today_up: Some(today_up),
                    today_down: Some(today_down),
                    total_up: Some(total_up),
                    total_down: Some(total_down),
                    last_updated: Some(format_now()),
                    protocol_row: Default::default(),
                };
                if let Err(e) = state.db.upsert_server_stats(&stats).await {
                    state.log_trace("error", "tui", &format!("Failed to save stats: {e}"));
                }
                // Update in-memory endpoint row to avoid full reload
                if let Some(row) = state
                    .endpoints
                    .iter_mut()
                    .find(|r| r.endpoint.id == protocol_id)
                {
                    row.stats.insert(protocol_id, stats);
                }
                state.current_traffic_up = total_up;
                state.current_traffic_down = total_down;
            }
            CoreEvent::SysStatsUpdate(stats) => {
                state.current_memory = stats.alloc;
                state.system_stats = Some(stats);
            }
            CoreEvent::LogLine { .. } => {}
            CoreEvent::TuiLog { target, level, message } => {
                let level = level.to_lowercase();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64;
                state.log_cache.push_back(crate::LogLine {
                    level,
                    target,
                    message,
                    timestamp_nanos: now,
                });
                if state.log_cache.len() > 10_000 {
                    state.log_cache.pop_front();
                }
            }
            CoreEvent::SubscriptionsUpdated {
                group_id,
                count,
                error,
                summary,
            } => {
                state.updating_groups.remove(&group_id);
                if summary.total_errors > 0 || summary.security_warning_count > 0 {
                    let msg = format!(
                        "Subscription validation: {} errors (missing fields: {}, host validation: {}, security warnings: {}, other: {})",
                        summary.total_errors,
                        summary.missing_field_count,
                        summary.host_validation_count,
                        summary.security_warning_count,
                        summary.other_count,
                    );
                    state.log_trace("warn", "subscription", &msg);
                }
                if let Some(err) = error {
                    state.log_trace(
                        "error",
                        "subscription",
                        &format!("Subscription update failed: {err}"),
                    );
                } else {
                    state.log_trace(
                        "info",
                        "subscription",
                        &format!("Subscription updated: {count} profiles"),
                    );
                }
                state.reload_profiles().await;
                state.reload_groups().await;
            }
            CoreEvent::TestTypeUpdate {
                protocol_id,
                test_type,
            } => {
                state.testing_details.insert(protocol_id, test_type);
                state.testing_profiles.insert(protocol_id);
            }
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type,
                latency_ms,
                speed_bps,
                ip_info,
                error,
            } => {
                state.testing_profiles.remove(&protocol_id);
                state.testing_details.remove(&protocol_id);

                let name = {
                    let row = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id == protocol_id);
                    match row {
                        Some(row) => {
                            let ext = row.extensions.entry(protocol_id).or_insert_with(|| {
                                ProfileExtension {
                                    protocol_id,
                                    delay: None,
                                    speed: None,
                                    sort_order: None,
                                    ip_info: None,
                                    protocol_row: Default::default(),
                                }
                            });
                            match test_type {
                                TestType::RealPing => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                    ext.ip_info = ip_info;
                                }
                                TestType::TcpPing | TestType::UdpTest => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                }
                                TestType::SpeedTest => {
                                    ext.speed =
                                        speed_bps.map(|v| std::cmp::min(v, i32::MAX as u64) as i32);
                                }
                            }
                            let _ = state.db.upsert_profile_extension(ext).await;
                            row.active_protocol()
                                .remarks
                                .clone()
                                .unwrap_or_else(|| protocol_id.to_string())
                        }
                        None => protocol_id.to_string(),
                    }
                };

                if let Some(ref err) = error {
                    state.log_trace(
                        "warn",
                        "speedtest",
                        &format!("{test_type:?} failed for {name}: {err}"),
                    );
                } else {
                    let latency_str = latency_ms.map(|ms| format!("{ms}ms")).unwrap_or_default();
                    let speed_str = speed_bps.map(|bps| format!("{bps}bps")).unwrap_or_default();
                    let detail = if !speed_str.is_empty() {
                        speed_str
                    } else if !latency_str.is_empty() {
                        latency_str
                    } else {
                        "success".to_string()
                    };
                    state.log_trace(
                        "info",
                        "speedtest",
                        &format!("{test_type:?} {name}: {detail}"),
                    );
                }

                // Update tracking fields for actions log
                match test_type {
                    TestType::TcpPing => state.last_test_tcp = latency_ms,
                    TestType::RealPing => state.last_test_real = latency_ms,
                    TestType::SpeedTest => state.last_test_speed = speed_bps,
                    TestType::UdpTest => {} // no tracking for UDP
                }

                // Auto-reset stop flag when all tests complete
                if state.testing_profiles.is_empty() {
                    state.speed_test_stop.store(false, Ordering::Relaxed);
                    state.batch_progress = None;
                }
            }
            CoreEvent::UpdateCheckResult {
                core_type,
                current_version,
                latest_version,
                error,
            } => {
                let status = state.update_status.entry(core_type).or_default();
                status.current_version.clone_from(&current_version);
                status.latest_version.clone_from(&latest_version);
                status.update_available = {
                    #[allow(
                        clippy::option_if_let_else,
                        reason = "business logic with nested version comparison clearer as match"
                    )]
                    match &current_version {
                        // Not installed but latest known → install available
                        None => latest_version.is_some(),
                        // Both known → compare versions
                        Some(cur_str) => match &latest_version {
                            Some(latest_str) => {
                                let cur = xray_tui_core::updater::parse_version(cur_str);
                                let latest = xray_tui_core::updater::parse_version(latest_str);
                                match (cur, latest) {
                                    (Some(c), Some(l)) => xray_tui_core::updater::is_newer(&c, &l),
                                    _ => false,
                                }
                            }
                            None => false,
                        },
                    }
                };
                status.error = error;
                if let Some(ref ver) = latest_version {
                    match core_type {
                        CoreType::Xray => {
                            state.config.updates.xray_latest_known = Some(ver.clone());
                        }
                        CoreType::SingBox => {
                            state.config.updates.sing_box_latest_known = Some(ver.clone());
                        }
                        CoreType::Auto => {}
                    }
                }
                // Refresh form snapshots if currently viewing the updates form
                if let AppMode::Settings {
                    mode:
                        SettingsMode::Split {
                            right:
                                SplitRightPane::UpdateForm {
                                    status_xray,
                                    status_singbox,
                                },
                            ..
                        },
                } = &mut state.mode
                {
                    *status_xray = state
                        .update_status
                        .get(&CoreType::Xray)
                        .cloned()
                        .unwrap_or_default();
                    *status_singbox = state
                        .update_status
                        .get(&CoreType::SingBox)
                        .cloned()
                        .unwrap_or_default();
                }
            }
            CoreEvent::UpdateDownloadProgress {
                core_type,
                downloaded,
                total,
            } => {
                if let Some(status) = state.update_status.get_mut(&core_type) {
                    status.download_progress = Some((downloaded, total));
                }
            }
            CoreEvent::UpdateCompleted {
                core_type,
                old_version,
                new_version,
                success,
                error,
            } => {
                let status = state.update_status.entry(core_type).or_default();
                status.downloading = false;
                status.download_progress = None;
                if success {
                    status.current_version = Some(new_version.clone());
                    status.update_available = false;
                    state.log_trace(
                        "info",
                        "tui",
                        &format!(
                            "{core_type} updated: {} → {}",
                            old_version.as_deref().unwrap_or("none"),
                            new_version,
                        ),
                    );
                } else {
                    status.error.clone_from(&error);
                    state.log_trace(
                        "error",
                        "tui",
                        &format!("{core_type} update failed: {error:?}"),
                    );
                }
                // Refresh form snapshots if currently viewing the updates form
                if let AppMode::Settings {
                    mode:
                        SettingsMode::Split {
                            right:
                                SplitRightPane::UpdateForm {
                                    status_xray,
                                    status_singbox,
                                },
                            ..
                        },
                } = &mut state.mode
                {
                    *status_xray = state
                        .update_status
                        .get(&CoreType::Xray)
                        .cloned()
                        .unwrap_or_default();
                    *status_singbox = state
                        .update_status
                        .get(&CoreType::SingBox)
                        .cloned()
                        .unwrap_or_default();
                }
            }
            CoreEvent::BatchProgress { .. } => {}
        }
    }
}
