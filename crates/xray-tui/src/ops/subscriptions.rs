use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use xray_tui_config::import_export::{ValidationSettings, ValidationSummary};
use xray_tui_db::Database;
use xray_tui_db::models::{Group, GroupCoreType, GroupStatus};

use crate::AppState;

use crate::types::{CoreEvent, SplitRightPane};
use crate::{get_field, try_send_or_warn};

/// Map the typed group core enum to the form's select values.
const fn group_core_to_str(c: GroupCoreType) -> &'static str {
    match c {
        GroupCoreType::Auto => "auto",
        GroupCoreType::Xray => "Xray",
        GroupCoreType::SingBox => "SingBox",
    }
}

/// Map the form's select value to the typed group core (auto → `None`, the
/// model's "unset" state).
fn group_core_from_str(s: &str) -> Option<GroupCoreType> {
    match s {
        // Accept the select's capitalized values ("Xray"/"SingBox") in
        // addition to the legacy lowercase spellings — the group form writes
        // the capitalized strings, so rejecting them silently reset every
        // explicit override to auto.
        "xray" | "Xray" => Some(GroupCoreType::Xray),
        "sing-box" | "singbox" | "SingBox" => Some(GroupCoreType::SingBox),
        _ => None,
    }
}

pub fn start_add_group(state: &mut AppState) {
    let fields = vec![
        ("name".into(), String::new()),
        ("subscription_url".into(), String::new()),
        ("user_agent".into(), String::new()),
        ("update_interval".into(), "1h".into()),
        ("core_type".into(), "auto".into()),
    ];
    if let crate::AppMode::Settings {
        mode: crate::SettingsMode::Split { right, .. },
    } = &mut state.mode
    {
        *right = SplitRightPane::GroupForm {
            group_id: None,
            fields,
            focus_index: 0,
            form_errors: HashMap::new(),
        };
    }
}

pub fn start_edit_group(state: &mut AppState, group_id: &str) {
    let group = if let Some(g) = state.groups.iter().find(|g| g.id == group_id) {
        g.clone()
    } else {
        state.log_trace("error", "tui::ops::subscriptions", "Group not found");
        return;
    };
    let update_interval_value = group.refresh_interval.map_or_else(
        || "1h".into(),
        |mins| {
            humantime::format_duration(std::time::Duration::from_secs(mins as u64 * 60)).to_string()
        },
    );
    let fields = vec![
        ("name".into(), group.name.clone().unwrap_or_default()),
        (
            "subscription_url".into(),
            group.url.clone().unwrap_or_default(),
        ),
        (
            "user_agent".into(),
            group.user_agent.clone().unwrap_or_default(),
        ),
        ("update_interval".into(), update_interval_value),
        (
            "core_type".into(),
            group
                .core_type
                .map_or("auto", group_core_to_str)
                .to_string(),
        ),
    ];
    if let crate::AppMode::Settings {
        mode: crate::SettingsMode::Split { right, .. },
    } = &mut state.mode
    {
        *right = SplitRightPane::GroupForm {
            group_id: Some(group_id.into()),
            fields,
            focus_index: 0,
            form_errors: HashMap::new(),
        };
    }
}

pub async fn confirm_add_group(state: &mut AppState) {
    let fields = match &state.mode {
        crate::AppMode::Settings {
            mode:
                crate::SettingsMode::Split {
                    right: SplitRightPane::GroupForm { fields, .. },
                    ..
                },
        } => fields.clone(),
        _ => return,
    };
    let interval: i64 = get_field(&fields, "update_interval")
        .and_then(|v| humantime::parse_duration(&v).ok())
        .map_or(60, |d| (d.as_secs() / 60) as i64);
    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name: get_field(&fields, "name"),
        url: get_field(&fields, "subscription_url"),
        enabled: true,
        user_agent: get_field(&fields, "user_agent"),
        convert_target: None,
        core_type: get_field(&fields, "core_type")
            .as_deref()
            .and_then(group_core_from_str),
        sort_order: Some((state.groups.len() + 1) as i32),
        refresh_interval: Some(interval),
        last_refreshed: None,
        status: None,
        error_message: None,
    };
    if let Err(e) = state.db.upsert_group(&group).await {
        state.log_trace(
            "error",
            "tui::ops::subscriptions",
            &format!("Failed to add group: {e}"),
        );
        return;
    }
    state.log_trace(
        "info",
        "tui::ops::subscriptions",
        &format!(
            "Group '{}' added",
            group.name.as_deref().unwrap_or("unnamed")
        ),
    );
    state.reload_groups().await;
    if let crate::AppMode::Settings {
        mode: crate::SettingsMode::Split { right, .. },
    } = &mut state.mode
    {
        *right = SplitRightPane::GroupList {
            selected: 0,
            selected_mask: vec![false; state.groups.len()],
        };
    }
}
pub async fn confirm_edit_group(state: &mut AppState) {
    let (group_id_opt, fields) = match &state.mode {
        crate::AppMode::Settings {
            mode:
                crate::SettingsMode::Split {
                    right:
                        SplitRightPane::GroupForm {
                            group_id, fields, ..
                        },
                    ..
                },
        } => (group_id.clone(), fields.clone()),
        _ => return,
    };
    let Some(group_id) = group_id_opt else {
        return;
    };
    let Some(mut group) = state.groups.iter().find(|g| g.id == group_id).cloned() else {
        state.log_trace("error", "tui::ops::subscriptions", "Group not found");
        return;
    };
    group.name = get_field(&fields, "name");
    group.url = get_field(&fields, "subscription_url");
    group.user_agent = get_field(&fields, "user_agent");
    group.core_type = get_field(&fields, "core_type")
        .as_deref()
        .and_then(group_core_from_str);
    let interval: i64 = get_field(&fields, "update_interval")
        .and_then(|v| humantime::parse_duration(&v).ok())
        .map_or(60, |d| (d.as_secs() / 60) as i64);
    group.refresh_interval = Some(interval);
    if let Err(e) = state.db.upsert_group(&group).await {
        state.log_trace(
            "error",
            "tui::ops::subscriptions",
            &format!("Failed to update group: {e}"),
        );
        return;
    }
    state.log_trace("info", "tui::ops::subscriptions", "Group updated");
    state.reload_groups().await;
    if let crate::AppMode::Settings {
        mode: crate::SettingsMode::Split { right, .. },
    } = &mut state.mode
    {
        *right = SplitRightPane::GroupList {
            selected: 0,
            selected_mask: vec![false; state.groups.len()],
        };
    }
}

pub async fn delete_group(state: &mut AppState, group_id: &str) {
    if let Err(e) = state.db.delete_group(group_id).await {
        state.log_trace(
            "error",
            "tui::ops::subscriptions",
            &format!("Failed to delete group: {e}"),
        );
        return;
    }
    state.log_trace("info", "tui::ops::subscriptions", "Group deleted");
    state.confirmation = None;
    state.reload_groups().await;
    state.reload_profiles().await;
}

pub async fn clear_group(state: &mut AppState, group_id: &str) {
    match state.db.clear_group_endpoints(group_id).await {
        Ok(count) => {
            state.log_trace(
                "info",
                "tui::ops::subscriptions",
                &format!("Cleared {count} profiles from group"),
            );
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::subscriptions",
                &format!("Failed to clear group: {e}"),
            );
        }
    }
    state.confirmation = None;
    state.reload_profiles().await;
}

pub fn update_group_subscriptions(state: &mut AppState, group_id: &str) {
    if state.updating_groups.contains(group_id) {
        return;
    }
    let group = if let Some(g) = state.groups.iter().find(|g| g.id == group_id) {
        g.clone()
    } else {
        state.log_trace("error", "tui::ops::subscriptions", "Group not found");
        return;
    };
    let url = match &group.url {
        Some(u) if !u.is_empty() => u.clone(),
        _ => {
            state.log_trace(
                "warn",
                "tui::ops::subscriptions",
                "Group has no subscription URL",
            );
            return;
        }
    };

    state.updating_groups.insert(group_id.to_string());
    let gid = group_id.to_string();
    let tx = state.core_event_tx.clone();
    let user_agent = group.user_agent.unwrap_or_else(|| "xray-tui/0.1".into());
    let db = state.db.clone();
    let validation: ValidationSettings = state.config.parsing.clone().into();
    tokio::spawn(async move {
        update_one_group(url, user_agent, gid, db, validation, tx).await;
    });
}

/// One group's update cycle: fetch + parse + persist under an overall
/// timeout, then deliver `SubscriptionsUpdated` (timeout or not). The
/// `updating_groups` spinner entry is removed by the event handler; the
/// 30-minute cap exists so a 100k-URL import is never aborted mid-parse
/// (the HTTP client itself times out after 30s).
async fn update_one_group(
    url: String,
    user_agent: String,
    gid: String,
    db: Arc<Database>,
    validation: ValidationSettings,
    tx: Option<tokio::sync::mpsc::Sender<CoreEvent>>,
) {
    let result = tokio::time::timeout(
        std::time::Duration::from_mins(30),
        do_update_subscription(url, user_agent, gid.clone(), db, validation),
    )
    .await;
    if let Ok(inner) = result {
        if let Some(tx) = &tx {
            try_send_or_warn(
                tx,
                CoreEvent::SubscriptionsUpdated {
                    group_id: inner.0,
                    count: inner.1,
                    summary: inner.2,
                    error: inner.3,
                },
                "subs_updated",
            );
        }
    } else {
        tracing::error!(target: "tui::ops::subscriptions", "Subscription update timed out after 30m");
        if let Some(tx) = &tx {
            try_send_or_warn(
                tx,
                CoreEvent::SubscriptionsUpdated {
                    group_id: gid.clone(),
                    count: 0,
                    summary: ValidationSummary::default(),
                    error: Some("Subscription update timed out after 30m".into()),
                },
                "subs_timeout",
            );
        }
    }
}

async fn do_update_subscription(
    url: String,
    user_agent: String,
    group_id: String,
    db: Arc<Database>,
    validation: ValidationSettings,
) -> (String, usize, ValidationSummary, Option<String>) {
    // Warn on HTTP (non-HTTPS) subscription URLs
    if url.starts_with("http://") {
        tracing::warn!(
            target: "tui::ops::subscriptions",
            "Subscription URL uses HTTP, traffic is not encrypted"
        );
    }
    let (count, summary) = crate::ops::stream_import::import_http_subscription(
        &url,
        &user_agent,
        &db,
        Some(&group_id),
        &validation,
    )
    .await;
    tracing::info!(target: "tui::ops::subscriptions", "DB upsert succeeded: {count} links, {} errors", summary.total_errors);

    // Update group metadata (last_refreshed, status)
    if let Ok(groups) = db.get_all_groups().await
        && let Some(mut grp) = groups.into_iter().find(|g| g.id == group_id)
    {
        grp.last_refreshed = Some(jiff::Timestamp::now());
        grp.status = Some(GroupStatus::Ok);
        grp.error_message = None;
        let _ = db.upsert_group(&grp).await;
    }

    (group_id, count, summary, None)
}

/// URLs per parse+persist batch (one DB transaction per batch).
pub const PERSIST_CHUNK: usize = 500;

/// Parse URLs in bounded batches and persist each batch's rows with the
/// db-crate bulk upserts.
///
/// One transaction per batch instead of one autocommit per row — a 7000-URL
/// feed previously issued ~28k implicit commits, pegging the `NVMe` and
/// starving the UI. Rows are deduped across the WHOLE import via deterministic
/// ids before the bulk calls, so repeated protocol configs and duplicate
/// (host, port) lines upsert once per run.
///
/// Batch failures are logged with the URL index range and skipped — same
/// log-and-continue semantics the old per-profile loop had, coarser.
/// Returns `(links persisted, whole-run summary)`.
pub async fn persist_parsed_urls(
    db: &Arc<Database>,
    urls: &[String],
    group_id: Option<&str>,
    validation: &ValidationSettings,
) -> (usize, ValidationSummary) {
    const PROGRESS_EVERY: usize = 5000;
    let total = urls.len();

    let mut summary = ValidationSummary::default();
    let mut count = 0usize;
    // Deterministic-id dedup sets: a protocol row is shared across endpoints
    // (identity excludes host/port), so subscription feeds that repeat one
    // protocol config over hundreds of server URLs collapse to one upsert.
    let mut seen_protocols = std::collections::HashSet::new();
    let mut seen_endpoints = std::collections::HashSet::new();
    let mut seen_links = std::collections::HashSet::new();

    for (chunk_idx, chunk) in urls.chunks(PERSIST_CHUNK).enumerate() {
        let (profiles, batch_summary) =
            xray_tui_config::subscription::parse_url_batch(chunk, validation);
        summary.merge(&batch_summary);

        let mut endpoints: Vec<xray_tui_db::models::Endpoint> = Vec::new();
        let mut protocols: Vec<xray_tui_db::models::Protocol> = Vec::new();
        let mut links: Vec<xray_tui_db::models::ProfileStats> = Vec::new();
        let mut group_links: Vec<xray_tui_db::models::EndpointGroup> = Vec::new();
        let mut chunk_links = 0usize;

        for profile in &profiles {
            let parsed = &profile.parsed;
            if parsed.endpoints.is_empty() {
                continue;
            }
            let protocol = crate::state::protocol_from_parsed(parsed);
            if seen_protocols.insert(protocol.id.get()) {
                protocols.push(protocol.clone());
            }
            for ep in &parsed.endpoints {
                let endpoint = crate::state::endpoint_from_essentials(ep);
                let link = crate::state::link_from_parsed_with_id(parsed, protocol.id, endpoint.id);
                if seen_links.insert((link.protocol_id.get(), link.endpoint_id.get())) {
                    if seen_endpoints.insert(endpoint.id.get()) {
                        endpoints.push(endpoint.clone());
                    }
                    links.push(link.clone());
                    if let Some(gid) = group_id {
                        group_links.push(xray_tui_db::models::EndpointGroup {
                            endpoint_id: endpoint.id,
                            group_id: gid.to_owned(),
                            last_seen_at: link.last_seen_at,
                            sort_order: None,
                            endpoint: toasty::Deferred::default(),
                            group: toasty::Deferred::default(),
                        });
                    }
                    chunk_links += 1;
                }
            }
        }

        let url_range = (
            chunk_idx * PERSIST_CHUNK,
            chunk_idx * PERSIST_CHUNK + chunk.len(),
        );
        // One transaction for the WHOLE chunk: the four bulk families share
        // one commit, and a busy error retries the whole chunk. Row Vecs are
        // moved into Arc slices so each retry attempt clones the Arc
        // (refcount bump) instead of deep-copying the chunk contents.
        let endpoints = Arc::from(endpoints);
        let protocols = Arc::from(protocols);
        let links = Arc::from(links);
        let group_links = Arc::from(group_links);
        let persist = || {
            let (endpoints, protocols, links, group_links) = (
                Arc::clone(&endpoints),
                Arc::clone(&protocols),
                Arc::clone(&links),
                Arc::clone(&group_links),
            );
            async move {
                let mut conn = db.connection().await?;
                let mut tx = conn.transaction().await?;
                xray_tui_db::upsert_endpoints_bulk(&mut tx, &endpoints).await?;
                xray_tui_db::upsert_protocols_bulk(&mut tx, &protocols).await?;
                xray_tui_db::upsert_links_bulk(&mut tx, &links).await?;
                xray_tui_db::upsert_endpoint_group_links_bulk(&mut tx, &group_links).await?;
                tx.commit().await?;
                Ok(())
            }
        };
        if let Err(e) = xray_tui_db::retry_on_busy(persist, 5).await {
            tracing::error!(
                target: "tui::ops::subscriptions",
                "bulk persist failed for URLs [{}..{}): {e}",
                url_range.0,
                url_range.1,
            );
        } else {
            count += chunk_links;
        }

        if count != 0 && count / PROGRESS_EVERY != (count - chunk_links) / PROGRESS_EVERY {
            tracing::info!(target: "tui::ops::subscriptions", "Imported {count}/{total} links from subscription");
        }
        tokio::task::yield_now().await;
    }

    (count, summary)
}

pub fn update_all_subscriptions(state: &mut AppState) {
    // One shared pipeline: a group with a URL runs sequentially. Parallel
    // group herds multiplied write contention (each group previously spawned
    // its own upsert task burst); the per-group `updating_groups` guard still
    // prevents double-updates of the same group.
    let groups: Vec<(String, String, Option<String>)> = state
        .groups
        .iter()
        .filter(|g| g.url.as_deref().is_some_and(|u| !u.is_empty()))
        .filter(|g| !state.updating_groups.contains(&g.id))
        .map(|g| {
            (
                g.id.clone(),
                g.url.clone().unwrap_or_default(),
                g.user_agent.clone(),
            )
        })
        .collect();
    if groups.is_empty() {
        return;
    }

    let tx = state.core_event_tx.clone();
    let db = state.db.clone();
    let validation: ValidationSettings = state.config.parsing.clone().into();
    for (gid, _url, _ua) in &groups {
        state.updating_groups.insert(gid.clone());
    }
    tokio::spawn(async move {
        for (gid, url, ua) in groups {
            let user_agent = ua.unwrap_or_else(|| "xray-tui/0.1".into());
            update_one_group(
                url,
                user_agent,
                gid,
                db.clone(),
                validation.clone(),
                tx.clone(),
            )
            .await;
        }
    });
}

/// Start a background task to check and update subscriptions.
pub fn spawn_auto_update(state: &mut AppState) {
    let Some(tx) = state.core_event_tx.clone() else {
        return;
    };
    let db = state.db.clone();
    let validation: ValidationSettings = state.config.parsing.clone().into();
    let shutdown = state.shutdown_token.clone();
    tokio::spawn(async move {
        // Check shutdown before first sleep
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(10)) => {},
            () = async { while !shutdown.load(Ordering::Relaxed) { tokio::time::sleep(Duration::from_millis(100)).await; } } => return,
        }
        loop {
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            let Ok(due_groups) = db.get_groups_due_update().await else {
                tokio::select! {
                    () = tokio::time::sleep(Duration::from_mins(1)) => {},
                    () = async { while !shutdown.load(Ordering::Relaxed) { tokio::time::sleep(Duration::from_millis(100)).await; } } => return,
                }
                continue;
            };
            for group in &due_groups {
                if shutdown.load(Ordering::Relaxed) {
                    return;
                }
                let url = match &group.url {
                    Some(u) => u.clone(),
                    None => continue,
                };
                let ua = group
                    .user_agent
                    .clone()
                    .unwrap_or_else(|| "xray-tui/0.1".into());
                let gid = group.id.clone();
                let result =
                    do_update_subscription(url, ua, gid.clone(), db.clone(), validation.clone())
                        .await;
                try_send_or_warn(
                    &tx,
                    CoreEvent::SubscriptionsUpdated {
                        group_id: result.0,
                        count: result.1,
                        summary: result.2,
                        error: result.3,
                    },
                    "auto_subs_updated",
                );
            }
            tokio::select! {
                () = tokio::time::sleep(Duration::from_mins(1)) => {},
                () = async { while !shutdown.load(Ordering::Relaxed) { tokio::time::sleep(Duration::from_millis(100)).await; } } => return,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_core_select_round_trip() {
        // The group form's Core Type select (ui/settings.rs OPTIONS) offers
        // "Auto"/"Xray"/"SingBox". Every option must parse to the right core
        // instead of silently degrading to auto (F1: the capitalized values
        // were rejected, so explicit Xray/SingBox overrides reset to auto on
        // save — and editing such a group showed the override as lost).
        for (select, expected) in [
            ("Auto", None),
            ("Xray", Some(GroupCoreType::Xray)),
            ("SingBox", Some(GroupCoreType::SingBox)),
        ] {
            assert_eq!(
                group_core_from_str(select),
                expected,
                "from_str({select:?})"
            );
        }
        // Emitter output must parse back to the same core. Auto's emitter
        // spelling is "auto", which maps to the model's unset state (None).
        for (core, expected) in [
            (GroupCoreType::Auto, None),
            (GroupCoreType::Xray, Some(GroupCoreType::Xray)),
            (GroupCoreType::SingBox, Some(GroupCoreType::SingBox)),
        ] {
            assert_eq!(
                group_core_from_str(group_core_to_str(core)),
                expected,
                "round-trip {core:?}"
            );
        }
        // Legacy lowercase spellings remain accepted.
        assert_eq!(group_core_from_str("xray"), Some(GroupCoreType::Xray));
        assert_eq!(
            group_core_from_str("sing-box"),
            Some(GroupCoreType::SingBox)
        );
        assert_eq!(group_core_from_str("singbox"), Some(GroupCoreType::SingBox));
    }

    fn valid_vmess_url(host: &str) -> String {
        let qr = serde_json::json!({
            "v": "2", "ps": "test", "add": host, "port": "443",
            "id": "550e8400-e29b-41d4-a716-446655440000", "aid": "0", "scy": "auto",
            "net": "tcp", "type": "none", "host": "", "path": "", "tls": "",
            "sni": "", "alpn": "", "fp": "", "insecure": "0",
        });
        let b64 = base64_simd::STANDARD.encode_to_string(serde_json::to_string(&qr).unwrap());
        format!("vmess://{b64}")
    }

    #[tokio::test]
    async fn persist_parsed_urls_dedups_and_persists_group_links() {
        use xray_tui_db::models::EndpointId;
        let db = Arc::new(Database::in_memory().await.expect("in-memory db"));
        let validation = ValidationSettings::default();

        // Two identical URLs (same host+protocol), one distinct host, one
        // garbage URL. The two identical URLs dedup to ONE link (same
        // endpoint id + protocol id), the distinct host is a second link;
        // both share ONE protocol row (identity excludes host/port).
        let urls = vec![
            valid_vmess_url("1.2.3.4"),
            valid_vmess_url("1.2.3.4"),
            valid_vmess_url("5.6.7.8"),
            "vmess://!!!not-base64!!!".to_string(),
        ];

        let (count, summary) = persist_parsed_urls(&db, &urls, Some("g1"), &validation).await;
        assert_eq!(count, 2, "duplicate (host, protocol) collapses");
        assert_eq!(summary.other_count, 1, "garbage URL counted");
        assert_eq!(summary.total_errors, 1);

        // One endpoint per unique host, both linked to the group.
        let rows = db
            .get_active_endpoints_by_group("g1", jiff::Timestamp::from_second(0).unwrap())
            .await
            .expect("group read");
        assert_eq!(rows.len(), 2, "one row per unique endpoint");
        assert_eq!(rows[0].links.len(), 1);
        assert_eq!(rows[1].links.len(), 1);

        // Rerun the same list: counts identical, no duplicate rows.
        let (count2, _) = persist_parsed_urls(&db, &urls, Some("g1"), &validation).await;
        assert_eq!(count2, 2, "idempotent rerun");
        let rows2 = db
            .get_active_endpoints_by_group("g1", jiff::Timestamp::from_second(0).unwrap())
            .await
            .expect("group read 2");
        assert_eq!(rows2.len(), 2, "no row duplication on rerun");
        assert_eq!(rows2[0].links.len(), 1, "no link duplication on rerun");

        // No group id: links persist without group membership rows.
        let db2 = Arc::new(Database::in_memory().await.expect("in-memory db"));
        let (count3, _) = persist_parsed_urls(&db2, &urls, None, &validation).await;
        assert_eq!(count3, 2);
        assert!(
            db2.get_active_endpoints_by_group("g1", jiff::Timestamp::from_second(0).unwrap())
                .await
                .expect("no group rows")
                .is_empty(),
            "group_id=None must not create group links"
        );
        let _ = EndpointId::new(1); // import reference for the models path
    }
}
