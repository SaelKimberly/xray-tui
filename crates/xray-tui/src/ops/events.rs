use std::net::IpAddr;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use xray_tui_core::CoreType;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::models::{EndpointId, EndpointRow, ErrorInfo, Latency, ProfileErr, TrafficStats};

use crate::AppState;
use crate::ops::profiles::endpoint_dns_unresolved;
use crate::types::{AppMode, CoreEvent, SettingsMode, SplitRightPane};

/// Format a profile ID as a URL-like hex identifier.
fn fmt_profile_id(id: i64) -> String {
    format!("xray-tui://{id:x}")
}

/// Find the endpoint row whose links own `protocol_id` (a `Protocol` row id).
/// Endpoint ids (stable hashes of host:port) are unrelated to protocol ids,
/// so the match scans `r.links`.
#[must_use]
pub(crate) fn endpoint_row_for_protocol(
    endpoints: &mut [EndpointRow],
    protocol_id: i64,
) -> Option<&mut EndpointRow> {
    endpoints
        .iter_mut()
        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
}

/// The persisted `error.kind` bucket for a test type: fast-class tests
/// (TCP/UDP) land in `Fast`, data-plane tests (real ping, speed) in `Real`.
/// Name-resolution failures surface on real attempts and share the real
/// bucket (the tier model already maps `Name` under real-err).
const fn err_kind_for(test_type: TestType) -> ProfileErr {
    match test_type {
        TestType::TcpPing | TestType::UdpTest => ProfileErr::Fast,
        TestType::RealPing | TestType::SpeedTest => ProfileErr::Real,
    }
}

/// Poll core event channel and update state accordingly.
///
/// Returns `true` when anything was handled (an event consumed, or a finished
/// batch-progress bar cleared), so the caller can trigger an immediate redraw
/// instead of waiting for the idle refresh cadence.
pub async fn poll_core_events(state: &mut AppState) -> bool {
    // Clean up stale batch_progress when a batch task has finished silently
    // (the batch pipeline is removed in T17; T19 rebuilds it).
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
                // protocol_id is a Protocol row id — match the row whose links
                // own it (never the endpoint id). Patch the link's traffic
                // in-memory and persist via `upsert_link` (the gRPC stats
                // poller writes these).
                if let Some(row) = endpoint_row_for_protocol(&mut state.endpoints, protocol_id)
                    && let Some(link) = row
                        .links
                        .iter_mut()
                        .find(|l| l.protocol_id.get() == protocol_id)
                {
                    link.traffic = TrafficStats {
                        today_up,
                        today_down,
                        total_up,
                        total_down,
                    };
                    if let Err(e) = state.db.upsert_link(link).await {
                        state.log_trace(
                            "error",
                            "tui::ops::events",
                            &format!("Failed to save stats: {e}"),
                        );
                    }
                }
                state.current_traffic_up = total_up;
                state.current_traffic_down = total_down;
            }
            CoreEvent::SysStatsUpdate(sys_stats) => {
                state.current_memory = sys_stats.alloc;
                state.system_stats = Some(sys_stats);
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
                    .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
                    .map_or((0, false), |r| {
                        use xray_tui_db::models::HostType;
                        (r.endpoint.id.get(), r.endpoint.host_type == HostType::Dns)
                    });
                let ip_info_clone = ip_info.clone();

                let name = {
                    let row = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id));
                    match row {
                        Some(row) => {
                            if let Some(link) = row
                                .links
                                .iter_mut()
                                .find(|l| l.protocol_id.get() == protocol_id)
                            {
                                // A stopped test is not a failure: the stop path
                                // emits `error: Some("Cancelled")` for sessions
                                // that never ran, and persisting a failure
                                // marker would paint `[fast]`/`[real]` red on
                                // endpoints whose tests were merely abandoned.
                                if error.is_some() && error.as_deref() != Some("Cancelled") {
                                    // Persisted failure marker — the profiles
                                    // Test column renders `[fast]`/`[real]`
                                    // from `link.error.kind` (round maps
                                    // removed in T17).
                                    link.error = Some(ErrorInfo {
                                        kind: err_kind_for(test_type),
                                        text: error.clone().unwrap_or_default(),
                                    });
                                    if let Err(e) = state.db.upsert_link(link).await {
                                        state.log_trace(
                                            "error",
                                            "tui::ops::events",
                                            &format!("Failed to save ping error: {e}"),
                                        );
                                    }
                                } else if error.is_none() {
                                    // Success: record the measurement and clear
                                    // any previous failure marker.
                                    match test_type {
                                        TestType::RealPing => {
                                            link.latency = latency_ms.map(|ms| Latency::Real {
                                                delay: ms as i32,
                                                ip: ip_info
                                                    .as_deref()
                                                    .and_then(|s| s.split('|').next())
                                                    .map(str::trim)
                                                    .filter(|s| !s.is_empty())
                                                    .map(str::to_string),
                                            });
                                        }
                                        TestType::TcpPing | TestType::UdpTest => {
                                            link.latency = latency_ms
                                                .map(|ms| Latency::Fast { delay: ms as i32 });
                                        }
                                        TestType::SpeedTest => {
                                            link.speed_bps = speed_bps.map(|v| v as i64);
                                        }
                                    }
                                    link.error = None;
                                    if let Err(e) = state.db.upsert_link(link).await {
                                        state.log_trace(
                                            "error",
                                            "tui::ops::events",
                                            &format!("Failed to save ping result: {e}"),
                                        );
                                    }
                                }
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

                // Live sub-table ordering: re-sort the endpoint's links by test
                // priority after every fast/real ping result. `selected_sub`
                // follows its protocol to its new index (only when this
                // result's endpoint is the one currently selected).
                if matches!(test_type, TestType::TcpPing | TestType::RealPing) {
                    let keep = if state.selected_profile_id() == Some(ep_id) {
                        state.selected_sub.and_then(|n| {
                            state
                                .endpoints
                                .iter()
                                .find(|r| r.endpoint.id.get() == ep_id)
                                .and_then(|r| r.links.get(n).map(|l| l.protocol_id))
                        })
                    } else {
                        None
                    };
                    let dns_unresolved = state
                        .endpoints
                        .iter()
                        .find(|r| r.endpoint.id.get() == ep_id)
                        .is_some_and(|r| endpoint_dns_unresolved(state, r));
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id.get() == ep_id)
                    {
                        row.sort_links_by_test_priority(dns_unresolved);
                        if let Some(pid) = keep {
                            state.selected_sub =
                                row.links.iter().position(|l| l.protocol_id == pid);
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
                let mut persist: Option<(Vec<String>, jiff::Timestamp)> = None;
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
                                    .collect::<Vec<_>>(),
                                jiff::Timestamp::from_second(entry.resolved_at_secs.unwrap_or(0))
                                    .unwrap_or_else(|_| jiff::Timestamp::now()),
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
                // re-resolve; the TTL gate applies across restarts. The typed
                // write also refreshes the resolved-IP child endpoints (port
                // 443, per the T10 decision).
                if let Some((resolved_as, resolved_at)) = persist {
                    let db = state.db.clone();
                    let ip_addrs: Vec<IpAddr> =
                        resolved_as.iter().filter_map(|s| s.parse().ok()).collect();
                    tokio::spawn(async move {
                        let eid = EndpointId::new(endpoint_id);
                        if let Err(e) = db
                            .update_endpoint_resolution(eid, resolved_as, resolved_at)
                            .await
                        {
                            tracing::warn!(
                                target: "tui::ops::events",
                                "update_endpoint_resolution failed: {e}"
                            );
                        }
                        if let Err(e) = db.upsert_resolved_ip_children(eid, &ip_addrs).await {
                            tracing::warn!(
                                target: "tui::ops::events",
                                "upsert_resolved_ip_children failed: {e}"
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
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id.get() == endpoint_id)
                    {
                        row.sort_links_by_test_priority(false);
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
    use std::sync::Arc;

    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{EndpointRow, HostType, Latency};

    use crate::ops::profiles::test_support::fake_row;
    use crate::types::EndpointInfo;

    use super::*;

    /// Multi-protocol endpoint fixture. Protocol ids `[start..start+n]`.
    fn row_with_protocols(endpoint_id: i64, n: usize, start: i64) -> EndpointRow {
        let mut row = fake_row(endpoint_id, &format!("h{endpoint_id}.example"), n);
        // Rebase the link protocol ids onto `start` so tests control the ids.
        for (i, link) in row.links.iter_mut().enumerate() {
            let new_pid = start + i as i64;
            let protocol = row.protocols.remove(&link.protocol_id).unwrap_or_else(|| {
                crate::ops::profiles::xray_tui_db_helper::vless_protocol(new_pid)
            });
            link.protocol_id = xray_tui_db::models::ProtocolId::new(new_pid);
            row.protocols.insert(link.protocol_id, protocol);
        }
        row
    }

    /// Pre-set a link's latency (fast or real) before a result event lands.
    fn set_delay(row: &mut EndpointRow, pid: i64, delay: i32, real: bool) {
        let link = row
            .links
            .iter_mut()
            .find(|l| l.protocol_id.get() == pid)
            .expect("link exists");
        link.latency = if real {
            Some(Latency::Real { delay, ip: None })
        } else {
            Some(Latency::Fast { delay })
        };
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
        // endpoint id 100 has link protocol 7; endpoint id 101 has link 9
        let mut rows = vec![row_with_protocols(100, 1, 7), row_with_protocols(101, 1, 9)];

        assert_eq!(
            endpoint_row_for_protocol(&mut rows, 9).map(|r| r.endpoint.id.get()),
            Some(101)
        );
        assert_eq!(
            endpoint_row_for_protocol(&mut rows, 7).map(|r| r.endpoint.id.get()),
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
        set_delay(&mut row, 9, 50, false); // fast-ok
        row.endpoint.host_type = HostType::Ipv4;
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
        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![8, 9, 7]);
        // selected_sub followed p9 to its new index 1.
        assert_eq!(state.selected_sub, Some(1));
        // p8's latency persisted as a Real measurement (with the ip part).
        assert!(matches!(
            state.endpoints[0].links[0].latency,
            Some(Latency::Real { delay: 120, ip: Some(ref s) }) if s == "1.2.3.4"
        ));
    }

    #[tokio::test]
    async fn real_ping_failure_demotes_below_untested() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, true); // stored real-ok
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

        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![8, 7]); // fresh real failure sinks below untested
        // The failure persisted as a Real-kind error marker.
        assert_eq!(
            state.endpoints[0].links[1].error.as_ref().map(|e| e.kind),
            Some(ProfileErr::Real)
        );
    }

    #[tokio::test]
    async fn fast_ping_result_sorts_sub_table() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 200, true); // real-ok
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

        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![7, 8]); // real-ok 200 still above fast-ok 15
        assert!(matches!(
            state.endpoints[0].links[1].latency,
            Some(Latency::Fast { delay: 15 })
        ));
    }

    #[tokio::test]
    async fn udp_result_does_not_resort_and_records_udp_latency() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 200, false); // fast-ok
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);
        state.testing_profiles.insert(8);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::UdpTest,
            latency_ms: Some(50),
            speed_bps: None,
            ip_info: None,
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // UDP must not trigger a re-sort: p8 (udp-ok 50ms) stays below p7
        // (fast-ok 200ms) in the CURRENT (unsorted) order.
        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![7, 8]);
        // Latency persisted (fast-class semantics).
        assert!(matches!(
            state.endpoints[0].links[1].latency,
            Some(Latency::Fast { delay: 50 })
        ));
    }

    #[tokio::test]
    async fn dns_unresolved_endpoint_sinks_after_result() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, true); // real-ok
        row.endpoint.host_type = HostType::Dns;
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

        // dns tier sinks both protocols to the bottom. Within the dns tier the
        // tiebreak is `last_seen_at desc, then id asc`; both links share
        // last_seen ts(0), so id asc wins -> [7, 8].
        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![7, 8]);
    }

    #[tokio::test]
    async fn cancelled_real_ping_does_not_rank_untested_as_real_ok() {
        let (mut state, tx) = event_state().await;
        let row = row_with_protocols(100, 2, 7); // p7, p8, both untested
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);
        state.testing_profiles.insert(8);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::RealPing,
            latency_ms: None,
            speed_bps: None,
            ip_info: None,
            error: Some("Cancelled".to_string()),
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // A Cancelled event mutates nothing: no latency, no error marker —
        // p8 stays untested and is never hoisted to the real-ok tier.
        let link = state.endpoints[0]
            .links
            .iter()
            .find(|l| l.protocol_id.get() == 8)
            .unwrap();
        assert!(link.latency.is_none());
        assert!(link.error.is_none());
        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![7, 8]); // order unchanged, still untested tier
    }

    #[tokio::test]
    async fn endpoint_info_flip_to_resolved_restores_priority_order() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, false); // fast-ok
        set_delay(&mut row, 8, 10, true); // real-ok
        row.endpoint.host_type = HostType::Dns;
        state.endpoints = vec![row]; // no endpoint_info entry -> unresolved
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);

        tx.send(CoreEvent::EndpointInfoUpdated {
            endpoint_id: 100,
            info: EndpointInfo {
                resolved_ips: vec!["1.2.3.4".parse().unwrap()],
                ..EndpointInfo::default()
            },
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // Unresolved: both sink to the dns tier ([7, 8] by id). Once the DNS
        // flip resolves, p8 (real-ok) rises above p7 (fast-ok) -> [8, 7].
        let ids: Vec<i64> = state.endpoints[0]
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect();
        assert_eq!(ids, vec![8, 7]);
    }
}
