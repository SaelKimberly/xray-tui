use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use xray_tui_core::CoreType;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::models::{
    DELAY_SOURCE_FAST, DELAY_SOURCE_REAL, DELAY_SOURCE_UDP, EndpointRow, ProfileExtension,
    ServerStat,
};

use crate::AppState;
use crate::format_now;
use crate::ops::profiles::{endpoint_dns_unresolved, session_rounds};
use crate::types::{AppMode, CoreEvent, SettingsMode, SplitRightPane};

/// Format a profile ID as a URL-like hex identifier.
fn fmt_profile_id(id: i64) -> String {
    format!("xray-tui://{id:x}")
}

/// Find the endpoint row whose protocols list owns `protocol_id`
/// (a `ProtocolRow` id). Endpoint ids (stable hashes of host:port) are
/// unrelated to protocol ids, so the match scans `r.protocols`.
#[must_use]
pub(crate) fn endpoint_row_for_protocol(
    endpoints: &mut [EndpointRow],
    protocol_id: i64,
) -> Option<&mut EndpointRow> {
    endpoints
        .iter_mut()
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
}

/// Poll core event channel and update state accordingly.
///
/// Returns `true` when anything was handled (an event consumed, or a finished
/// batch-progress bar cleared), so the caller can trigger an immediate redraw
/// instead of waiting for the idle refresh cadence.
pub async fn poll_core_events(state: &mut AppState) -> bool {
    // Clean up stale batch_progress when the task has finished silently
    // (e.g., create_ping_batch returned 0, no TestTypeUpdate events were ever sent).
    let mut handled = if state.batch_progress.is_some()
        && state.testing_profiles.is_empty()
        && state
            .batch_progress
            .as_ref()
            .map(|p| p.0.load(Ordering::Relaxed))
            == Some(0)
    {
        state.batch_progress = None;
        true
    } else {
        false
    };
    while let Some(rx) = state.core_event_rx.as_mut() {
        let event = match rx.try_recv() {
            Ok(event) => event,
            Err(
                tokio::sync::mpsc::error::TryRecvError::Empty
                | tokio::sync::mpsc::error::TryRecvError::Disconnected,
            ) => break,
        };
        handled = true;
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
                // Update in-memory endpoint row to avoid full reload.
                // protocol_id is a ProtocolRow id — match the row whose
                // protocols list owns it (never the endpoint id).
                if let Some(row) = endpoint_row_for_protocol(&mut state.endpoints, protocol_id) {
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
                    .map_or((0, false), |r| {
                        (r.endpoint.id, r.endpoint.host_type == "dns")
                    });
                let ip_info_clone = ip_info.clone();

                // Feed the per-endpoint ping round used by the profiles Test
                // column `[fast]`/`[real]` labels. A protocol succeeds on any
                // result without error; failures accumulate per round.
                let round = match test_type {
                    TestType::TcpPing => {
                        Some(&mut state.ping_status.entry(ep_id).or_default().fast)
                    }
                    TestType::RealPing => {
                        Some(&mut state.ping_status.entry(ep_id).or_default().real)
                    }
                    TestType::SpeedTest | TestType::UdpTest => None,
                };
                if let Some(round) = round {
                    round.seen.insert(protocol_id);
                    // A stopped test is not a failure: the stop path emits
                    // `error: Some("Cancelled")` for sessions that never ran,
                    // and counting them failed would paint `[fast]`/`[real]`
                    // red on endpoints whose tests were merely abandoned.
                    if error.is_some() && error.as_deref() != Some("Cancelled") {
                        round.failed.insert(protocol_id);
                    } else {
                        round.failed.remove(&protocol_id);
                    }
                }

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
                                    delay_source: None,
                                    protocol_row: Default::default(),
                                }
                            });
                            match test_type {
                                TestType::RealPing => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                    ext.delay_source = Some(DELAY_SOURCE_REAL);
                                    ext.ip_info = ip_info;
                                }
                                TestType::TcpPing | TestType::UdpTest => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                    ext.delay_source = Some(if test_type == TestType::UdpTest {
                                        DELAY_SOURCE_UDP
                                    } else {
                                        DELAY_SOURCE_FAST
                                    });
                                }
                                TestType::SpeedTest => {
                                    ext.speed = speed_bps.map(|v| v as i64);
                                }
                            }
                            if error.is_none() {
                                let _ = state.db.upsert_profile_extension(ext).await;
                            }
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

                // Live sub-table ordering: re-sort the endpoint's protocols by
                // test priority after every fast/real ping result. `selected_sub`
                // follows its protocol to its new index (only when this result's
                // endpoint is the one currently selected).
                if matches!(test_type, TestType::TcpPing | TestType::RealPing) {
                    let keep = if state.selected_profile_id() == Some(ep_id) {
                        state.selected_sub.and_then(|n| {
                            state
                                .endpoints
                                .iter()
                                .find(|r| r.endpoint.id == ep_id)
                                .and_then(|r| r.protocols.get(n).map(|p| p.id))
                        })
                    } else {
                        None
                    };
                    let dns_unresolved = state
                        .endpoints
                        .iter()
                        .find(|r| r.endpoint.id == ep_id)
                        .is_some_and(|r| endpoint_dns_unresolved(state, r));
                    let rounds = state
                        .endpoints
                        .iter()
                        .find(|r| r.endpoint.id == ep_id)
                        .and_then(|r| session_rounds(&state.ping_status, r));
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id == ep_id)
                    {
                        row.sort_protocols_by_test_priority(dns_unresolved, rounds);
                        if let Some(pid) = keep {
                            state.selected_sub = row.protocols.iter().position(|p| p.id == pid);
                        }
                    }
                    state.filter_cache_valid.set(false);
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
                // Before the merge: was this endpoint's DNS unresolved?
                let was_resolved = state
                    .endpoint_info
                    .get(&endpoint_id)
                    .is_some_and(|i| !i.resolved_ips.is_empty());
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
                        // Keep the current flags when the incoming event
                        // carries defaults (seed pass before whitelist load).
                        entry.host_features =
                            merge_host_features(entry.host_features, info.host_features);
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
                                    .map(std::string::ToString::to_string)
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
                            .update_endpoint_resolution(
                                endpoint_id,
                                Some(&resolved_as),
                                resolved_at,
                            )
                            .await
                        {
                            tracing::warn!(
                                target: "tui::ops::events",
                                "update_endpoint_resolution failed: {e}"
                            );
                        }
                    });
                }

                // DNS flip (unresolved -> resolved): lift the endpoint's
                // protocols out of the name (bottom) tier.
                let is_resolved = state
                    .endpoint_info
                    .get(&endpoint_id)
                    .is_some_and(|i| !i.resolved_ips.is_empty());
                if !was_resolved && is_resolved {
                    let rounds = state
                        .ping_status
                        .get(&endpoint_id)
                        .map(|ps| (&ps.fast.failed, &ps.real.failed));
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id == endpoint_id)
                    {
                        row.sort_protocols_by_test_priority(false, rounds);
                    }
                    state.filter_cache_valid.set(false);
                }
            }
        }
    }
    handled
}

/// Merge whitelist feature flags from an `EndpointInfoUpdated` event into the
/// cached entry, guarding against a seed phase-2 event clobbering real values.
///
/// `spawn_dns_resolve` captures the whitelist checker at spawn time; on first
/// launch it is `None`, so the seed event carries `HostFeatures::default()`.
/// If that event lands after the whitelist pass, its default flags would wipe
/// the real ones for the session — keep the current value in that case.
#[must_use]
pub(crate) fn merge_host_features(
    current: xray_tui_host_features::HostFeatures,
    incoming: xray_tui_host_features::HostFeatures,
) -> xray_tui_host_features::HostFeatures {
    if incoming == xray_tui_host_features::HostFeatures::default() {
        current
    } else {
        incoming
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use toasty::Deferred;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{Endpoint, ProtocolRow};

    use super::*;

    /// EndpointRow fixture: one endpoint owning one protocol.
    fn row_with_protocol(endpoint_id: i64, protocol_id: i64) -> EndpointRow {
        EndpointRow {
            endpoint: Endpoint {
                id: endpoint_id,
                host: format!("h{endpoint_id}.example"),
                host_type: "ipv4".to_string(),
                port: 443,
                port_spec_str: None,
                parent_id: None,
                last_source: None,
                created_at: 0,
                manual_protocol_override: None,
                resolved_as: None,
                resolved_at: None,
            },
            protocols: vec![ProtocolRow {
                id: protocol_id,
                endpoint_id,
                sig: 0,
                cred_hash: 0,
                proto_kind: String::new(),
                spec_blob: Vec::new(),
                config_type: 1,
                core_type: "xray".to_string(),
                transport: None,
                security: None,
                last_used_at: None,
                created_at: 0,
                last_seen_at: 0,
                endpoint: Deferred::from(None::<Endpoint>),
                extension: Deferred::from(None::<ProfileExtension>),
                server_stat: Deferred::from(None::<ServerStat>),
            }],
            extensions: HashMap::new(),
            stats: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        }
    }

    /// Multi-protocol endpoint fixture. Protocols get ids `[start..start+n]`.
    fn row_with_protocols(endpoint_id: i64, n: usize, start: i64) -> EndpointRow {
        let mut row = row_with_protocol(endpoint_id, start); // single-proto fixture
        for i in 1..n {
            row.protocols.push(ProtocolRow {
                id: start + i as i64,
                endpoint_id,
                ..row.protocols[0].clone()
            });
        }
        row
    }

    fn set_delay(row: &mut EndpointRow, pid: i64, delay: i32, source: Option<i32>) {
        row.extensions.insert(
            pid,
            ProfileExtension {
                protocol_id: pid,
                delay: Some(delay),
                speed: None,
                sort_order: None,
                ip_info: None,
                delay_source: source,
                protocol_row: Deferred::from(None::<ProtocolRow>),
            },
        );
    }

    async fn event_state() -> (AppState, tokio::sync::mpsc::Sender<CoreEvent>) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        state.core_event_rx = Some(rx);
        (state, tx)
    }

    #[test]
    fn merge_host_features_keeps_existing_when_incoming_is_default() {
        let real = xray_tui_host_features::HostFeatures {
            sni_whitelisted: true,
            ip_whitelisted: true,
            cidr_whitelisted: false,
        };
        // A seed phase-2 event carries `HostFeatures::default()` when the
        // whitelist checker was not loaded at spawn; it must not clobber the
        // flags a later whitelist pass already applied.
        assert_eq!(
            merge_host_features(real, xray_tui_host_features::HostFeatures::default()),
            real
        );
        // A genuine whitelist-pass event still overwrites with real values.
        let other = xray_tui_host_features::HostFeatures {
            sni_whitelisted: false,
            ip_whitelisted: false,
            cidr_whitelisted: true,
        };
        assert_eq!(merge_host_features(real, other), other);
    }

    #[tokio::test]
    async fn poll_core_events_reports_whether_events_were_handled() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        // Isolate from the startup channels (whitelist load, auto-update):
        // replace the rx so only this test can feed events — deterministic.
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        state.core_event_rx = Some(rx);
        // Idle: no events -> false (keeps the refresh-cadence draw path).
        assert!(!state.poll_core_events().await);
        // A real event -> true, and the state change is applied.
        tx.send(CoreEvent::Connected(CoreType::Xray)).await.unwrap();
        assert!(state.poll_core_events().await);
        assert_eq!(state.connected_core, Some(CoreType::Xray));
        // Drained -> idle again.
        assert!(!state.poll_core_events().await);
    }

    #[test]
    fn endpoint_row_for_protocol_matches_protocol_id_not_endpoint_id() {
        // endpoint id 100 has protocol id 7; endpoint id 101 has protocol id 9
        let mut rows = vec![row_with_protocol(100, 7), row_with_protocol(101, 9)];

        assert_eq!(
            endpoint_row_for_protocol(&mut rows, 9).map(|r| r.endpoint.id),
            Some(101)
        );
        assert_eq!(
            endpoint_row_for_protocol(&mut rows, 7).map(|r| r.endpoint.id),
            Some(100)
        );
        // Endpoint ids are not protocol ids — the lookup must not match them.
        assert!(endpoint_row_for_protocol(&mut rows, 100).is_none());
        assert!(endpoint_row_for_protocol(&mut rows, 999).is_none());
    }

    #[tokio::test]
    async fn real_ping_result_restores_sub_table_and_remaps_selection() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 3, 7); // p7, p8, p9
        set_delay(&mut row, 9, 50, Some(DELAY_SOURCE_FAST)); // fast-ok
        row.endpoint.host_type = "ipv4".to_string();
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = Some(2); // points at p9 before the sort
        state.filter_cache_valid.set(false);
        // The SpeedTestResult dedupe guard drops events for protocols not in
        // `testing_profiles`; production seeds it when a ping starts.
        state.testing_profiles.insert(8);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::RealPing,
            latency_ms: Some(120),
            speed_bps: None,
            ip_info: Some("1.2.3.4|US".to_string()),
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // p8 (real 120) above p9 (fast 50) — tier beats latency; p7 untested last.
        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![8, 9, 7]);
        // selected_sub followed p9 to its new index 1.
        assert_eq!(state.selected_sub, Some(1));
        assert_eq!(
            state.endpoints[0].extensions[&8].delay_source,
            Some(DELAY_SOURCE_REAL)
        );
    }

    #[tokio::test]
    async fn real_ping_failure_demotes_below_untested() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, Some(DELAY_SOURCE_REAL)); // stored real-ok
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);
        state.testing_profiles.insert(7);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 7,
            test_type: TestType::RealPing,
            latency_ms: None,
            speed_bps: None,
            ip_info: None,
            error: Some("timeout".to_string()),
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![8, 7]); // fresh real failure sinks below untested
    }

    #[tokio::test]
    async fn fast_ping_result_sorts_sub_table() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 200, Some(DELAY_SOURCE_REAL));
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);
        state.testing_profiles.insert(8);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::TcpPing,
            latency_ms: Some(15),
            speed_bps: None,
            ip_info: None,
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![7, 8]); // real-ok 200 still above fast-ok 15
        assert_eq!(
            state.endpoints[0].extensions[&8].delay_source,
            Some(DELAY_SOURCE_FAST)
        );
    }

    #[tokio::test]
    async fn dns_unresolved_endpoint_sinks_after_result() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, Some(DELAY_SOURCE_REAL));
        row.endpoint.host_type = "dns".to_string();
        state.endpoints = vec![row]; // no endpoint_info entry -> unresolved
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);
        state.testing_profiles.insert(8);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::RealPing,
            latency_ms: Some(10),
            speed_bps: None,
            ip_info: Some("1.2.3.4|US".to_string()),
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // dns tier sinks both protocols to the bottom. Without the dns flag
        // p8 (fresh real-ok, 10ms) would outrank p7 (real-ok, 50ms) -> [8, 7];
        // with it, both fall to tier 5, whose within-tier tiebreak is
        // `last_seen_at desc, then id asc` (Task 2 adjudicated spec) -> [7, 8].
        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![7, 8]);
    }
}
