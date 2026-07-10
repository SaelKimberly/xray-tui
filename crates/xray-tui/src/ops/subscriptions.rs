use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use xray_tui_config::import_export::{Profile, ValidationSettings, ValidationSummary};
use xray_tui_db::Database;
use xray_tui_db::models::{Endpoint, Group, ProtocolRow};

use crate::AppState;
use crate::state::profile_to_endpoint_protocol;
use crate::types::{AppMode, CoreEvent};
use crate::{format_now, get_field, try_send_or_warn};

pub fn start_add_group(state: &mut AppState) {
    let fields = vec![
        ("name".into(), String::new()),
        ("subscription_url".into(), String::new()),
        ("user_agent".into(), String::new()),
        ("update_interval".into(), "1h".into()),
        ("core_type".into(), "auto".into()),
    ];
    state.mode = AppMode::AddGroup {
        fields,
        focus_index: 0,
    };
}

pub fn start_edit_group(state: &mut AppState, group_id: &str) {
    let group = if let Some(g) = state.groups.iter().find(|g| g.id == group_id) {
        g.clone()
    } else {
        state.log_trace("error", "tui", "Group not found");
        return;
    };
    let update_interval_value = group.refresh_interval.map_or_else(
        || "1h".into(),
        |mins| {
            humantime::format_duration(std::time::Duration::from_secs(mins as u64 * 60)).to_string()
        },
    );
    let fields = vec![
        ("name".into(), group.name.unwrap_or_default()),
        ("subscription_url".into(), group.url.unwrap_or_default()),
        ("user_agent".into(), group.user_agent.unwrap_or_default()),
        ("update_interval".into(), update_interval_value),
        (
            "core_type".into(),
            group.core_type.unwrap_or_else(|| "auto".into()),
        ),
    ];
    state.mode = AppMode::EditGroup {
        group_id: group_id.into(),
        fields,
        focus_index: 0,
    };
}

pub async fn confirm_add_group(state: &mut AppState) {
    let fields = match &state.mode {
        AppMode::AddGroup { fields, .. } => fields.clone(),
        _ => return,
    };
    let interval: i32 = get_field(&fields, "update_interval")
        .and_then(|v| humantime::parse_duration(&v).ok())
        .map_or(60, |d| (d.as_secs() / 60) as i32);
    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name: get_field(&fields, "name"),
        url: get_field(&fields, "subscription_url"),
        enabled: Some(1),
        user_agent: get_field(&fields, "user_agent"),
        convert_target: None,
        core_type: get_field(&fields, "core_type"),
        sort_order: Some((state.groups.len() + 1) as i32),
        refresh_interval: Some(interval),
        last_refreshed: None,
        status: Some("idle".into()),
        error_message: None,
    };
    if let Err(e) = state.db.insert_group(&group).await {
        state.log_trace("error", "tui", &format!("Failed to add group: {e}"));
        return;
    }
    state.log_trace(
        "info",
        "tui",
        &format!(
            "Group '{}' added",
            group.name.as_deref().unwrap_or("unnamed")
        ),
    );
    state.mode = AppMode::List;
    state.reload_groups().await;
}

pub async fn confirm_edit_group(state: &mut AppState) {
    let (group_id, fields) = match &state.mode {
        AppMode::EditGroup {
            group_id, fields, ..
        } => (group_id.clone(), fields.clone()),
        _ => return,
    };
    let mut group = if let Some(g) = state.groups.iter().find(|g| g.id == group_id) {
        g.clone()
    } else {
        state.log_trace("error", "tui", "Group not found");
        return;
    };
    group.name = get_field(&fields, "name");
    group.url = get_field(&fields, "subscription_url");
    group.user_agent = get_field(&fields, "user_agent");
    group.core_type = get_field(&fields, "core_type");
    let interval: i32 = get_field(&fields, "update_interval")
        .and_then(|v| humantime::parse_duration(&v).ok())
        .map_or(60, |d| (d.as_secs() / 60) as i32);
    group.refresh_interval = Some(interval);
    if let Err(e) = state.db.update_group(&group).await {
        state.log_trace("error", "tui", &format!("Failed to update group: {e}"));
        return;
    }
    state.log_trace("info", "tui", "Group updated");
    state.mode = AppMode::List;
    state.reload_groups().await;
}

pub async fn delete_group(state: &mut AppState, group_id: &str) {
    if let Err(e) = state.db.delete_group(group_id).await {
        state.log_trace("error", "tui", &format!("Failed to delete group: {e}"));
        return;
    }
    let _ = state.db.purge_expired(0).await;
    state.log_trace("info", "tui", "Group deleted");
    state.selected_group_id = None;
    state.confirmation = None;
    state.reload_groups().await;
    state.reload_profiles().await;
}

pub async fn clear_group(state: &mut AppState, group_id: &str) {
    match state.db.clear_group(group_id).await {
        Ok(count) => {
            state.log_trace(
                "info",
                "tui",
                &format!("Cleared {count} profiles from group"),
            );
        }
        Err(e) => {
            state.log_trace("error", "tui", &format!("Failed to clear group: {e}"));
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
        state.log_trace("error", "tui", "Group not found");
        return;
    };
    let url = match &group.url {
        Some(u) if !u.is_empty() => u.clone(),
        _ => {
            state.log_trace("warn", "tui", "Group has no subscription URL");
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
            tracing::error!(target: "tui", "Subscription update timed out after 120s");
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
        target: "tui",
        "Parsed {} profiles, {} errors from subscription",
        profiles.len(),
        summary.total_errors,
    );
    if profiles.is_empty() {
        tracing::info!(target: "tui", "Subscription returned 0 usable profiles — all URLs may have failed validation");
    }
    let pairs: Vec<(Endpoint, Vec<ProtocolRow>)> = profiles
        .into_iter()
        .map(|p| {
            let profile = Profile::from(&p);
            let (endpoint, protocol) = profile_to_endpoint_protocol(&profile);
            (endpoint, vec![protocol])
        })
        .collect();
    tracing::info!(
        target: "tui",
        "Starting DB upsert for {} profiles",
        pairs.len()
    );
    if let Err(e) = db.subscription_upsert(&group_id, &pairs).await {
        tracing::error!(target: "tui", "DB upsert failed: {e}");
        return (group_id, 0, summary, Some(format!("DB upsert: {e}")));
    }
    tracing::info!(target: "tui", "DB upsert succeeded");

    // Update group metadata (last_refreshed, status) — merged from old Subscription
    if let Ok(groups) = db.get_all_groups().await
        && let Some(mut grp) = groups.into_iter().find(|g| g.id == group_id)
    {
        grp.last_refreshed = Some(format_now());
        grp.status = Some("ok".into());
        grp.error_message = None;
        let _ = db.update_group(&grp).await;
    }

    (group_id, pairs.len(), summary, None)
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
