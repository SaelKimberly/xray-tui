use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use xray_tui_core::CoreType;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::models::{ProfileExtension, ServerStat};

use crate::AppState;
use crate::format_now;
use crate::types::{AppMode, CoreEvent, SettingsMode, SplitRightPane};

/// Format a profile ID as a URL-like hex identifier.
fn fmt_profile_id(id: i64) -> String {
    format!("xray-tui://{id:x}")
}

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
                state.log_trace("info", "core::process", &format!("Connected [{core_type}]"));
            }
            CoreEvent::Disconnected => {
                // Ignore stale Disconnected if already reconnecting
                if !state.connecting {
                    state.connected_core = None;
                    state.connected_protocol_id = None;
                    state.log_trace("info", "core::process", "Core process stopped");
                }
            }
            CoreEvent::Error(err) => {
                state.connection_error = Some(err.clone());
                state.connecting = false;
                state.connected_core = None;
                state.log_trace(
                    "error",
                    "core::process",
                    &format!("Connection error: {err}"),
                );
                state.connected_protocol_id = None;
            }
            CoreEvent::StatsError(msg) => {
                state.connection_error = Some(msg.clone());
                state.log_trace("warning", "core::process", &format!("Stats error: {msg}"));
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
                    state.log_trace(
                        "error",
                        "tui::ops::events",
                        &format!("Failed to save stats: {e}"),
                    );
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
            CoreEvent::TuiLog {
                target,
                level,
                message,
            } => {
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
                    state.log_trace("warn", "tui::ops::subscriptions", &msg);
                }
                if let Some(err) = error {
                    state.log_trace(
                        "error",
                        "tui::ops::subscriptions",
                        &format!("Subscription update failed: {err}"),
                    );
                } else {
                    state.log_trace(
                        "info",
                        "tui::ops::subscriptions",
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
                // Guard against duplicate events for the same protocol_id
                if !state.testing_profiles.remove(&protocol_id) {
                    // Already processed — skip
                    continue;
                }
                state.testing_details.remove(&protocol_id);

                // Capture endpoint context before the mutable row borrow below.
                let (ep_id, ep_host_is_dns) = state
                    .endpoints
                    .iter()
                    .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
                    .map(|r| (r.endpoint.id, r.endpoint.host_type == "dns"))
                    .unwrap_or((0, false));
                let ip_info_clone = ip_info.clone();

                let name = {
                    let row = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id));
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
                                    ext.speed = speed_bps.map(|v| v as i64);
                                }
                            }
                            let _ = state.db.upsert_profile_extension(ext).await;
                            fmt_profile_id(protocol_id)
                        }
                        None => fmt_profile_id(protocol_id),
                    }
                };

                if let Some(ref err) = error {
                    state.log_trace(
                        "warn",
                        "tui::ops::speedtest",
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
                        "tui::ops::speedtest",
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

                // Real ping happened — record the exit IP + country on the
                // endpoint; DNS-host endpoints get their inbound resolved too
                // (deferred-resolution trigger: real networking occurred).
                if ip_info_clone.is_some() {
                    crate::ops::enrich::spawn_outbound_enrich(state, protocol_id, ip_info_clone);
                }
                if ep_host_is_dns {
                    crate::ops::enrich::spawn_dns_resolve(state, ep_id, false);
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
                        "tui::ops::events",
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
                        "tui::ops::events",
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
            CoreEvent::BatchProgress {
                total,
                completed: _,
            } => {
                if total == 0 {
                    state.batch_progress = None;
                }
            }
            CoreEvent::HostFeaturesLoaded(checker) => {
                state.host_features = Some(checker);
                state.log_trace("info", "tui::state", "Whitelist loaded");
                // Refresh whitelist features for every endpoint (never
                // persisted — cached entries must track the current files).
                crate::ops::enrich::spawn_whitelist_pass(state);
            }
            CoreEvent::EndpointInfoUpdated { endpoint_id, info } => {
                // Merge by field group so concurrent enrichment (resolution /
                // whitelist / outbound) does not clobber each other. Events
                // with empty resolved_ips (failed lookup) carry nothing
                // mergable and must NOT materialize an entry — an empty entry
                // would block the startup seeding pass and make
                // `should_resolve` treat the endpoint as a never-retried IP
                // host.
                let mut persist: Option<(String, Option<i64>)> = None;
                if !info.resolved_ips.is_empty()
                    || info.sni_whitelisted.is_some()
                    || info.outbound_ip.is_some()
                    || info.resolved_at_secs.is_some()
                {
                    let entry = state.endpoint_info.entry(endpoint_id).or_default();
                    if !info.resolved_ips.is_empty() {
                        let had_resolved_at = entry.resolved_at_secs;
                        entry.resolved_ips = info.resolved_ips;
                        entry.resolved_at_secs = info.resolved_at_secs;
                        // country is derived (mmdb) — a lookup failure in a
                        // re-resolution returns None; keep the known value
                        // rather than clearing it.
                        entry.country = info.country.or_else(|| entry.country.clone());
                        entry.host_features = info.host_features;
                        // Persist only when the resolution itself changed
                        // (whitelist-pass copies carry the same timestamp and
                        // must not re-write).
                        if entry.resolved_at_secs.is_some()
                            && had_resolved_at != entry.resolved_at_secs
                        {
                            persist = Some((
                                entry
                                    .resolved_ips
                                    .iter()
                                    .map(|ip| ip.to_string())
                                    .collect::<Vec<_>>()
                                    .join(","),
                                entry.resolved_at_secs,
                            ));
                        }
                    } else if info.resolved_at_secs.is_some() {
                        // Failed lookup (empty result): record the attempt so
                        // re-resolution is TTL-gated instead of re-running on
                        // every trigger. `x` (force) still re-attempts.
                        entry.resolved_at_secs = info.resolved_at_secs;
                    }
                    if info.sni_whitelisted.is_some() {
                        entry.sni_whitelisted = info.sni_whitelisted;
                    }
                    if info.outbound_ip.is_some() {
                        entry.outbound_ip = info.outbound_ip;
                        entry.outbound_country = info.outbound_country;
                    }
                }
                // Persist DNS resolutions (DNS hosts only) so launches don't
                // re-resolve; the TTL gate applies across restarts.
                if let Some((resolved_as, resolved_at)) = persist {
                    let db = state.db.clone();
                    tokio::spawn(async move {
                        if let Err(e) = db
                            .update_endpoint_resolution(endpoint_id, Some(&resolved_as), resolved_at)
                            .await
                        {
                            tracing::warn!(
                                target: "tui::ops::events",
                                "update_endpoint_resolution failed: {e}"
                            );
                        }
                    });
                }
            }
        }
    }
}
