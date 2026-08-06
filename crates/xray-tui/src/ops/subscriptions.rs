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
fn group_core_to_str(c: GroupCoreType) -> &'static str {
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
        "xray" => Some(GroupCoreType::Xray),
        "sing-box" | "singbox" => Some(GroupCoreType::SingBox),
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
                .map(group_core_to_str)
                .unwrap_or("auto")
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
    let group_id = match group_id_opt {
        Some(id) => id,
        None => return,
    };
    let mut group = if let Some(g) = state.groups.iter().find(|g| g.id == group_id) {
        g.clone()
    } else {
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
        let result = tokio::time::timeout(
            std::time::Duration::from_mins(2),
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
            tracing::error!(target: "tui::ops::subscriptions", "Subscription update timed out after 120s");
            if let Some(tx) = &tx {
                try_send_or_warn(
                    tx,
                    CoreEvent::SubscriptionsUpdated {
                        group_id: gid.clone(),
                        count: 0,
                        summary: ValidationSummary::default(),
                        error: Some("Subscription update timed out after 120s".into()),
                    },
                    "subs_timeout",
                );
            }
        }
    });
}

async fn do_update_subscription(
    url: String,
    user_agent: String,
    group_id: String,
    db: Arc<Database>,
    validation: ValidationSettings,
) -> (String, usize, ValidationSummary, Option<String>) {
    let client = match reqwest::Client::builder()
        .user_agent(&user_agent)
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                group_id,
                0,
                ValidationSummary::default(),
                Some(e.to_string()),
            );
        }
    };

    // Warn on HTTP (non-HTTPS) subscription URLs
    if url.starts_with("http://") {
        tracing::warn!(
            target: "tui::ops::subscriptions",
            "Subscription URL uses HTTP, traffic is not encrypted"
        );
    }

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                group_id,
                0,
                ValidationSummary::default(),
                Some(format!("HTTP: {e}")),
            );
        }
    };
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                group_id,
                0,
                ValidationSummary::default(),
                Some(format!("Body: {e}")),
            );
        }
    };
    let (profiles, summary) =
        match xray_tui_config::subscription::parse_subscription_data(&bytes, &validation) {
            Ok((p, s)) => (p, s),
            Err(e) => return (group_id, 0, ValidationSummary::default(), Some(e)),
        };
    tracing::info!(
        target: "tui::ops::subscriptions",
        "Parsed {} profiles, {} errors from subscription",
        profiles.len(),
        summary.total_errors,
    );
    if profiles.is_empty() {
        tracing::info!(target: "tui::ops::subscriptions", "Subscription returned 0 usable profiles — all URLs may have failed validation");
    }
    // Persist every parsed profile with the typed upserts. Dedup is natural:
    // endpoint ids (`stable_hash(host, port)`) and protocol ids (`uid()`) are
    // deterministic, so re-imports of the same (endpoint id, uid) pair update
    // the existing rows instead of duplicating. Profiles missing from this
    // fetch keep their old `last_seen_at` and age into the Stale view —
    // preserving the old move-orphans-to-purgatory semantics through the
    // typed staleness clock (purge_expired reclaims them after retention).
    let mut count = 0usize;
    for parsed in &profiles {
        match crate::state::persist_parsed(&db, &parsed.parsed, Some(&group_id), None).await {
            Ok(n) => count += n,
            Err(e) => {
                tracing::error!(target: "tui::ops::subscriptions", "profile upsert failed: {e}");
            }
        }
    }
    tracing::info!(target: "tui::ops::subscriptions", "DB upsert succeeded");

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

pub fn update_all_subscriptions(state: &mut AppState) {
    let group_ids: Vec<String> = state
        .groups
        .iter()
        .filter(|g| g.url.as_deref().is_some_and(|u| !u.is_empty()))
        .map(|g| g.id.clone())
        .collect();
    for gid in group_ids {
        update_group_subscriptions(state, &gid);
    }
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
            let due_groups = if let Ok(g) = db.get_groups_due_update().await {
                g
            } else {
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
