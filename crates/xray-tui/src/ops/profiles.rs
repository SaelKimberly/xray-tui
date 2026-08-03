use std::collections::HashMap;
use std::str::FromStr;

use crate::EndpointRow;
use xray_tui_config::import_export::{
    Profile, ValidationSettings, encode_profile_spec,
};
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{CoreType, resolve_core};

use crate::AppState;
use crate::state::profile_to_endpoint_protocol;
use crate::types::{AppMode, BatchImportItem, SortColumn};
use crate::{common_field_defaults, profile_to_fields};
use xray_tui_db::models::{ProfileExtension, PurgatoryView, ServerStat};

pub async fn reload_profiles(state: &mut AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let result = match state.purgatory_view {
        PurgatoryView::Active => {
            let threshold = now - state.purgatory_ttl_secs;
            state.db.get_active_endpoints(threshold).await
        }
        PurgatoryView::Stale => {
            let active_threshold = now - state.purgatory_ttl_secs;
            let stale_threshold = now - state.purgatory_retention_secs;
            state
                .db
                .get_stale_endpoints(active_threshold, stale_threshold)
                .await
        }
        PurgatoryView::All => state.db.get_active_endpoints(0).await,
    };
    match result {
        Ok(rows) => {
            state.endpoints = rows;
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to load profiles: {e}"),
            );
            state.endpoints.clear();
        }
    }
    state.filter_cache_valid.set(false);
    // Enrich new endpoints in the background (IP hosts + persisted DNS cache;
    // no network for fresh entries).
    crate::ops::enrich::spawn_enrich_ip_hosts(state);
}

pub async fn reload_groups(state: &mut AppState) {
    match state.db.get_all_groups().await {
        Ok(groups) => state.groups = groups,
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to load groups: {e}"),
            );
            state.groups.clear();
        }
    }
}

pub async fn reload_routing_rules(state: &mut AppState) {
    state.routing_rules = state.db.get_all_routing_rules().await.unwrap_or_default();
}

pub fn filtered_profiles(state: &AppState) -> impl Iterator<Item = &EndpointRow> {
    if !state.filter_cache_valid.get() {
        let indices = compute_filtered_indices(state);
        *state.cached_filtered_indices.borrow_mut() = indices;
        state.filter_cache_valid.set(true);
    }
    let indices: Vec<usize> = state.cached_filtered_indices.borrow().clone();
    indices.into_iter().map(move |i| &state.endpoints[i])
}

pub fn filtered_len(state: &AppState) -> usize {
    if !state.filter_cache_valid.get() {
        let indices = compute_filtered_indices(state);
        *state.cached_filtered_indices.borrow_mut() = indices;
        state.filter_cache_valid.set(true);
    }
    state.cached_filtered_indices.borrow().len()
}

fn compute_filtered_indices(state: &AppState) -> Vec<usize> {
    let mut indices: Vec<usize> = state
        .endpoints
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            if !state.search_query.is_empty() {
                let q = state.search_query.to_lowercase();
                let address = row.endpoint.host.clone();
                let port = row.endpoint.port.to_string();
                if !address.to_lowercase().contains(&q) && !port.contains(&q) {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    let asc = state.sort_ascending;
    indices.sort_by(|&a, &b| {
        let a_row = &state.endpoints[a];
        let b_row = &state.endpoints[b];
        let cmp = match state.sort_column {
            SortColumn::ConfigType => a_row
                .active_protocol()
                .config_type
                .cmp(&b_row.active_protocol().config_type),
            SortColumn::LastSeen => a_row
                .active_protocol()
                .last_seen_at
                .cmp(&b_row.active_protocol().last_seen_at),
            SortColumn::Address => a_row.endpoint.host.cmp(&b_row.endpoint.host),
            SortColumn::Port => a_row.endpoint.port.cmp(&b_row.endpoint.port),
            SortColumn::Delay => {
                let da = a_row
                    .extensions
                    .get(&a_row.active_protocol().id)
                    .and_then(|e| e.delay)
                    .unwrap_or(-1);
                let db = b_row
                    .extensions
                    .get(&b_row.active_protocol().id)
                    .and_then(|e| e.delay)
                    .unwrap_or(-1);
                da.cmp(&db)
            }
            SortColumn::Speed => {
                let sa = a_row
                    .extensions
                    .get(&a_row.active_protocol().id)
                    .and_then(|e| e.speed)
                    .unwrap_or(-1);
                let sb = b_row
                    .extensions
                    .get(&b_row.active_protocol().id)
                    .and_then(|e| e.speed)
                    .unwrap_or(-1);
                sa.cmp(&sb)
            }
            SortColumn::Traffic => {
                let ta = a_row
                    .stats
                    .get(&a_row.active_protocol().id)
                    .map_or(0, |s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0));
                let tb = b_row
                    .stats
                    .get(&b_row.active_protocol().id)
                    .map_or(0, |s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0));
                ta.cmp(&tb)
            }
            SortColumn::Core => {
                let resolve = |row: &EndpointRow| -> String {
                    let protocol = Protocol::try_from_i32(row.active_protocol().config_type)
                        .unwrap_or(Protocol::Custom);
                    let core = resolve_core(
                        protocol,
                        Some(
                            CoreType::from_str(&row.active_protocol().core_type)
                                .unwrap_or(CoreType::Auto),
                        ),
                    );
                    core.to_string()
                };
                resolve(a_row).cmp(&resolve(b_row))
            }
        };
        if asc { cmp } else { cmp.reverse() }
    });
    indices
}

pub fn resolved_core(state: &AppState, row: &EndpointRow) -> CoreType {
    let protocol =
        Protocol::try_from_i32(row.active_protocol().config_type).unwrap_or(Protocol::Custom);
    let profile_override = row.active_protocol().core_type.parse::<CoreType>().ok();
    let config_override = state
        .config
        .core
        .protocol_core_overrides
        .get(&protocol.to_string())
        .and_then(|s| s.parse::<CoreType>().ok());
    resolve_core(protocol, config_override.or(profile_override))
}

pub fn start_add_server(state: &mut AppState) {
    let fields = common_field_defaults();
    state.mode = AppMode::AddServer {
        protocol: None,
        fields,
        focus_index: 0,
        form_errors: HashMap::new(),
    };
}

/// Resolve an endpoint row for editing: check the currently loaded view
/// first, then fall back to every endpoint. The profiles list shows rows up
/// to `purgatory_ttl` old, so a fixed 1-day DB threshold would reject
/// visible-but-stale profiles ("Profile not found" while on screen).
async fn find_editable_endpoint(
    state: &AppState,
    protocol_id: i64,
) -> Option<EndpointRow> {
    if let Some(r) = state.endpoints.iter().find(|r| r.endpoint.id == protocol_id) {
        return Some(r.clone());
    }
    state
        .db
        .get_active_endpoints(0)
        .await
        .ok()
        .and_then(|rows| rows.into_iter().find(|r| r.endpoint.id == protocol_id))
}

pub async fn start_edit_profile(state: &mut AppState, id: &str) {
    let protocol_id: i64 = id.parse().unwrap_or(0);
    if find_editable_endpoint(state, protocol_id).await.is_some() {
        state.mode = AppMode::EditServer {
            protocol_id,
            fields: Vec::new(),
            focus_index: 0,
            form_errors: HashMap::new(),
        };
    } else {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Profile {id} not found"),
        );
    }
}

pub fn selected_profile_id(state: &AppState) -> Option<i64> {
    filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id)
}

pub fn toggle_expand(state: &mut AppState) {
    let ep_id = filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id);
    if let Some(ep_id) = ep_id
        && let Some(ep_row) = state.endpoints.iter_mut().find(|r| r.endpoint.id == ep_id)
    {
        ep_row.expanded = !ep_row.expanded;
        if ep_row.expanded {
            // Enter the sub-table on its first row (user decision: expand
            // lands on the first internal protocol).
            state.selected_sub = Some(0);
        } else {
            state.selected_sub = None;
        }
    }
}
pub fn collapse_expand(state: &mut AppState) {
    let ep_id = filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id);
    if let Some(ep_id) = ep_id
        && let Some(ep_row) = state.endpoints.iter_mut().find(|r| r.endpoint.id == ep_id)
    {
        ep_row.expanded = false;
    }
    state.selected_sub = None;
}

fn fields_to_profile(protocol: Protocol, fields: &[(String, String)]) -> Profile {
    let proto_kind = match protocol {
        Protocol::Vmess => "vmess",
        Protocol::Vless => "vless",
        Protocol::Trojan => "trojan",
        Protocol::Shadowsocks => "ss",
        Protocol::Shadowsocks2022 => "ss-2022",
        Protocol::ShadowsocksR => "ssr",
        Protocol::Socks => "socks",
        Protocol::Http => "http",
        Protocol::WireGuard => "wireguard",
        Protocol::Hysteria2 => "hysteria2",
        Protocol::Tuic => "tuic",
        Protocol::Hysteria => "hysteria",
        Protocol::Naive => "naive",
        Protocol::AnyTls => "anytls",
        Protocol::ShadowTls => "shadowtls",
        Protocol::Tor => "tor",
        Protocol::Ssh => "ssh",
        Protocol::Tailscale => "tailscale",
        Protocol::Redirect => "redirect",
        Protocol::TProxy => "tproxy",
        Protocol::Mixed => "mixed",
        Protocol::DokodemoDoor => "dokodemo-door",
        Protocol::Freedom => "freedom",
        Protocol::Blackhole => "blackhole",
        Protocol::Dns => "dns",
        Protocol::Loopback => "loopback",
        Protocol::Custom => "custom",
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let rand_bits = RandomState::new().build_hasher().finish();
    let uid: i64 = (now ^ rand_bits) as i64;
    let sig = uid;
    let cred_hash = 0;
    let mut address = String::new();
    let mut port: i32 = 0;
    let mut user_id: Option<String> = None;
    let mut security: Option<String> = None;
    let mut network: Option<String> = None;
    let mut core_type = "auto".to_string();
    let mut stream_map = serde_json::Map::new();
    let mut proto_map = serde_json::Map::new();

    for (key, value) in fields {
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "address" => address = value.clone(),
            "port" => port = value.parse::<i32>().unwrap_or(0),
            "core_type" => core_type.clone_from(value),
            "user_id" | "password" | "uuid" => user_id = Some(value.clone()),
            "security" => security = Some(value.clone()),
            "network" => network = Some(value.clone()),
            _ if key.starts_with("tls.")
                || key.starts_with("ws.")
                || key.starts_with("grpc.")
                || key.starts_with("reality.")
                || key.starts_with("tcp.")
                || *key == "sni"
                || *key == "alpn"
                || *key == "fingerprint"
                || *key == "allow_insecure" =>
            {
                let json_val = if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else if let Ok(n) = value.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else {
                    serde_json::Value::String(value.clone())
                };
                stream_map.insert(key.clone(), json_val);
            }
            _ => {
                let json_val = if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else if let Ok(n) = value.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else {
                    serde_json::Value::String(value.clone())
                };
                proto_map.insert(key.clone(), json_val);
            }
        }
    }

    let protocol_settings_str = if proto_map.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&proto_map).unwrap_or_default())
    };
    let stream_settings_str = if stream_map.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&stream_map).unwrap_or_default())
    };

    let mut profile = Profile {
        id: uid,
        sig,
        cred_hash,
        proto_kind: proto_kind.to_string(),
        spec_blob: Vec::new(),
        config_type: protocol.to_i32(),
        core_type,
        address,
        port,
        transport: network,
        security,
        created_at: now as i64,
        remarks: None,
    };
    let mut extra = serde_json::Map::new();
    if let Some(v) = &user_id {
        extra.insert("user_id".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(v) = &protocol_settings_str
        && let Ok(val) = serde_json::from_str(v)
    {
        extra.insert("protocol_settings".into(), val);
    }
    if let Some(v) = &stream_settings_str
        && let Ok(val) = serde_json::from_str(v)
    {
        extra.insert("stream_settings".into(), val);
    }
    profile.spec_blob =
        encode_profile_spec(proto_kind, serde_json::to_vec(&extra).unwrap_or_default());
    profile
}

pub async fn confirm_add_server(state: &mut AppState) {
    let (protocol, address, port, user_id) = {
        let (p, fields) = if let AppMode::AddServer {
            protocol: Some(p),
            fields,
            ..
        } = &state.mode
        {
            (*p, fields)
        } else {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                "Cannot confirm: no protocol selected",
            );
            return;
        };
        let addr = fields
            .iter()
            .find(|(k, _)| k == "address")
            .map_or("", |(_, v)| v.as_str());
        let prt = fields
            .iter()
            .find(|(k, _)| k == "port")
            .map_or("", |(_, v)| v.as_str());
        let uid = fields
            .iter()
            .find(|(k, _)| k == "user_id")
            .map_or("", |(_, v)| v.as_str());
        (p, addr.to_owned(), prt.to_owned(), uid.to_owned())
    };

    let mut errors: HashMap<String, String> = HashMap::new();
    if address.is_empty() {
        errors.insert("address".into(), "Address is required".into());
    }
    if port.is_empty() || port.parse::<u16>().map_or(true, |p| p == 0) {
        errors.insert("port".into(), "Port must be 1-65535".into());
    }
    match protocol {
        Protocol::Vmess | Protocol::Vless | Protocol::Trojan if user_id.is_empty() => {
            errors.insert("user_id".into(), "ID/Password required".into());
        }
        _ => {}
    }

    if !errors.is_empty() {
        if let AppMode::AddServer {
            ref mut form_errors,
            ..
        } = state.mode
        {
            *form_errors = errors;
        }
        return;
    }

    let fields = match &mut state.mode {
        AppMode::AddServer { fields, .. } => std::mem::take(fields),
        _ => unreachable!(),
    };

    let profile = fields_to_profile(protocol, &fields);
    let group_id = state.first_group_id();
    let (endpoint, protocol) = profile_to_endpoint_protocol(&profile);
    match state
        .db
        .insert_manual_endpoint(&endpoint, &protocol, &group_id)
        .await
    {
        Ok(()) => {
            let addr = format!("{}:{}", profile.address, profile.port);
            state.log_trace(
                "info",
                "tui::ops::profiles",
                &format!("Added server: {addr}"),
            );
            state.mode = AppMode::List;
            state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
            upsert_profile_row(state, profile, None, None, Some(state.first_group_id()));
            reload_profiles(state).await;
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to add server: {e}"),
            );
            if let AppMode::AddServer {
                fields: ref mut f, ..
            } = state.mode
            {
                *f = fields;
            }
        }
    }
}

pub async fn confirm_edit_server(state: &mut AppState) {
    let (protocol_id, address, port, user_id) = {
        let (pid, fields) = match &state.mode {
            AppMode::EditServer {
                protocol_id,
                fields,
                ..
            } => (protocol_id, fields),
            _ => return,
        };
        let addr = fields
            .iter()
            .find(|(k, _)| k == "address")
            .map_or("", |(_, v)| v.as_str());
        let prt = fields
            .iter()
            .find(|(k, _)| k == "port")
            .map_or("", |(_, v)| v.as_str());
        let uid = fields
            .iter()
            .find(|(k, _)| k == "user_id")
            .map_or("", |(_, v)| v.as_str());
        (*pid, addr.to_owned(), prt.to_owned(), uid.to_owned())
    };

    let mut errors: HashMap<String, String> = HashMap::new();
    if address.is_empty() {
        errors.insert("address".into(), "Address is required".into());
    }
    if port.is_empty() || port.parse::<u16>().map_or(true, |p| p == 0) {
        errors.insert("port".into(), "Port must be 1-65535".into());
    }
    let protocol = Protocol::try_from_i32(
        state
            .db
            .get_endpoint(protocol_id)
            .await
            .ok()
            .flatten()
            .map_or(0, |row| row.active_protocol().config_type),
    )
    .unwrap_or(Protocol::Custom);
    match protocol {
        Protocol::Vmess | Protocol::Vless | Protocol::Trojan if user_id.is_empty() => {
            errors.insert("user_id".into(), "ID/Password required".into());
        }
        _ => {}
    }

    if !errors.is_empty() {
        if let AppMode::EditServer {
            ref mut form_errors,
            ..
        } = state.mode
        {
            *form_errors = errors;
        }
        return;
    }

    let fields = match &mut state.mode {
        AppMode::EditServer { fields, .. } => std::mem::take(fields),
        _ => unreachable!(),
    };

    if state
        .db
        .get_endpoint(protocol_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        state.log_trace("error", "tui::ops::profiles", "Profile not found for edit");
        return;
    }
    let new_profile = fields_to_profile(protocol, &fields);
    let (endpoint, protocol_row) = profile_to_endpoint_protocol(&new_profile);
    let group_id = state.first_group_id();
    match state
        .db
        .subscription_upsert(&group_id, &[(endpoint, vec![protocol_row])])
        .await
    {
        Ok(_ids) => {
            let addr = format!("{}:{}", new_profile.address, new_profile.port);
            state.log_trace(
                "info",
                "tui::ops::profiles",
                &format!("Updated server: {addr}"),
            );
            state.mode = AppMode::List;
            state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
            reload_profiles(state).await;
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to update server: {e}"),
            );
            if let AppMode::EditServer {
                fields: ref mut f, ..
            } = state.mode
            {
                *f = fields;
            }
        }
    }
}

pub async fn delete_profile(state: &mut AppState, id: i64) {
    if let Err(e) = state.db.delete_endpoint(id).await {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Failed to delete profile: {e}"),
        );
        return;
    }
    state.log_trace("info", "tui::ops::profiles", "Profile deleted");
    state.confirmation = None;
    state.multi_select.remove(&id);
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn clone_profile(state: &mut AppState, id: i64) {
    let found = state.endpoints.iter().find(|r| r.endpoint.id == id);
    if found.is_none() {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Profile {id} not found for cloning"),
        );
        return;
    }
    state.log_trace("info", "tui::ops::profiles", "Profile cloned");
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub fn toggle_multi_select(state: &mut AppState, id: i64) {
    if !state.multi_select.insert(id) {
        state.multi_select.remove(&id);
    }
}

pub fn import_url(state: &mut AppState, url: &str) {
    let settings = ValidationSettings::from(state.config.parsing.clone());
    match xray_tui_config::import_export::parse_share_url(url, &settings) {
        Ok(parsed) => {
            let profile = Profile::from(&parsed);
            let protocol = Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
            let fields = profile_to_fields(&profile);
            state.mode = AppMode::AddServer {
                protocol: Some(protocol),
                fields,
                focus_index: 0,
                form_errors: HashMap::new(),
            };
            state.log_trace("info", "tui::ops::profiles", "URL imported successfully");
        }
        Err(e) => {
            state.mode = AppMode::ImportUrl {
                input: url.to_string(),
                error: Some(e.to_string()),
            };
        }
    }
}

pub fn start_batch_import(state: &mut AppState, urls: &[String]) {
    let settings = ValidationSettings::from(state.config.parsing.clone());
    let results: Vec<BatchImportItem> = urls
        .iter()
        .map(
            |url| match xray_tui_config::import_export::parse_share_url(url, &settings) {
                Ok(parsed) => {
                    let profile = Profile::from(&parsed);
                    BatchImportItem {
                        url: url.clone(),
                        profile: Some(profile),
                        error: None,
                        imported: false,
                    }
                }
                Err(e) => BatchImportItem {
                    url: url.clone(),
                    profile: None,
                    error: Some(e.to_string()),
                    imported: false,
                },
            },
        )
        .collect();
    state.mode = AppMode::BatchImport { results, scroll: 0 };
}

pub async fn confirm_batch_import(state: &mut AppState) {
    let items = match &mut state.mode {
        AppMode::BatchImport { results, .. } => std::mem::take(results),
        _ => return,
    };
    let mut imported = 0usize;
    let mut errors = 0usize;
    let group_id = state.first_group_id();
    for item in items {
        if let Some(profile) = item.profile {
            let (endpoint, protocol) = profile_to_endpoint_protocol(&profile);
            if state
                .db
                .insert_manual_endpoint(&endpoint, &protocol, &group_id)
                .await
                .is_ok()
            {
                imported += 1;
            } else {
                errors += 1;
            }
        }
    }
    state.log_trace(
        "info",
        "tui::ops::profiles",
        &format!("Batch import: {imported} imported, {errors} errors"),
    );
    state.mode = AppMode::List;
    reload_profiles(state).await;
}

pub async fn move_profile_up(state: &mut AppState) {
    let id = match selected_profile_id(state) {
        Some(id) => id,
        None => return,
    };
    let filtered: Vec<&EndpointRow> = filtered_profiles(state).collect();
    let idx = filtered.iter().position(|r| r.endpoint.id == id);
    let _idx = match idx {
        Some(i) if i > 0 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui::ops::profiles", "Profile moved up");
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn move_profile_down(state: &mut AppState) {
    let id = match selected_profile_id(state) {
        Some(id) => id,
        None => return,
    };
    let filtered: Vec<&EndpointRow> = filtered_profiles(state).collect();
    let _idx = match filtered.iter().position(|r| r.endpoint.id == id) {
        Some(i) if i < filtered.len() - 1 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui::ops::profiles", "Profile moved down");
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn set_active(state: &mut AppState, id: &str) {
    let pid: i64 = id.parse().unwrap_or(0);
    if let Err(e) = state.db.clear_protocol_override(pid).await {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Failed to clear override: {e}"),
        );
        return;
    }
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub fn upsert_profile_row(
    state: &mut AppState,
    _profile: Profile,
    _ext: Option<ProfileExtension>,
    _stat: Option<ServerStat>,
    _group_id: Option<String>,
) {
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

/// Cycle purgatory view: Active → Stale → All → Active
pub const fn cycle_purgatory_view(state: &mut AppState) {
    state.purgatory_view = match state.purgatory_view {
        PurgatoryView::Active => PurgatoryView::Stale,
        PurgatoryView::Stale => PurgatoryView::All,
        PurgatoryView::All => PurgatoryView::Active,
    };
}

/// Navigate protocol sub-rows when on an expanded endpoint.
/// Returns true if sub-row navigation was handled (caller should stop).
pub fn nav_protocol_down(state: &mut AppState) -> bool {
    // Extract data before any mutable access.
    let expanded_count = filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| if r.expanded { Some(r.protocols.len()) } else { None })
        .flatten();
    let Some(proto_count) = expanded_count else {
        // Not on an expandable endpoint: fall through to endpoint nav and
        // drop any stale sub-row selection.
        state.selected_sub = None;
        return false;
    };
    if proto_count <= 1 {
        state.selected_sub = None;
        return false;
    }
    match state.selected_sub {
        None => {
            // Full row of an expanded endpoint (reached via up-overflow):
            // re-enter the sub-table on its first row.
            state.selected_sub = Some(0);
            true
        }
        Some(n) if n + 1 < proto_count => {
            state.selected_sub = Some(n + 1);
            true
        }
        Some(_) => {
            // Last sub-row: down overflows to the next profile (full row).
            state.selected_sub = None;
            false
        }
    }
}

/// Navigate protocol sub-rows when on an expanded endpoint.
/// Returns true if sub-row navigation was handled (caller should stop).
pub fn nav_protocol_up(state: &mut AppState) -> bool {
    // Sub-rows live inside the expanded endpoint's panel — navigation never
    // crosses endpoint boundaries. Up at the first sub-row returns to the
    // endpoint row; Up on the endpoint row falls through to endpoint nav.
    let current_expanded = filtered_profiles(state)
        .nth(state.selected_index)
        .is_some_and(|r| r.expanded && r.protocols.len() > 1);
    if !current_expanded {
        // Fall through to endpoint nav; drop any stale sub-row selection.
        state.selected_sub = None;
        return false;
    }
    match state.selected_sub {
        None => false,
        Some(n) if n > 0 => {
            state.selected_sub = Some(n - 1);
            true
        }
        Some(0) => {
            // Up at the first sub-row returns to the endpoint row.
            state.selected_sub = None;
            true
        }
        Some(_) => false,
    }
}

/// Check whether the current selection is on a protocol sub-row.
pub const fn is_on_sub_row(state: &AppState) -> bool {
    state.selected_sub.is_some()
}

/// Get the protocol ID for the currently selected sub-row, if any.
pub fn selected_sub_protocol_id(state: &AppState) -> Option<i64> {
    let n = state.selected_sub?;
    let row = filtered_profiles(state).nth(state.selected_index)?;
    row.protocols.get(n).map(|p| p.id)
}

#[cfg(test)]
mod test_support {
    use super::*;
    use crate::AppState;
    use std::collections::HashMap;
    use std::sync::Arc;
    use toasty::Deferred;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{Endpoint, EndpointRow, ProfileExtension, ProtocolRow, ServerStat};

    pub fn fake_row(id: i64, host: &str, n_protos: usize) -> EndpointRow {
        EndpointRow {
            endpoint: Endpoint {
                id,
                host: host.to_string(),
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
            protocols: (0..n_protos)
                .map(|i| ProtocolRow {
                    id: id * 100 + i as i64,
                    endpoint_id: id,
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
                })
                .collect(),
            extensions: HashMap::new(),
            stats: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        }
    }

    pub async fn test_state(rows: Vec<EndpointRow>) -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        state.endpoints = rows;
        state.filter_cache_valid.set(false);
        state.selected_index = 0;
        state.selected_sub = None;
        state
    }
}

#[cfg(test)]
mod nav_tests {
    use super::*;
    use super::test_support::{fake_row, test_state};

    #[tokio::test]
    async fn expand_lands_on_first_sub_row() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        toggle_expand(&mut state);
        assert!(state.endpoints[0].expanded);
        assert_eq!(state.selected_sub, Some(0));
    }

    #[tokio::test]
    async fn down_walks_subs_then_overflows_to_next_profile() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3), fake_row(2, "b.com", 1)]).await;
        toggle_expand(&mut state); // sub 0
        assert!(nav_protocol_down(&mut state));
        assert_eq!(state.selected_sub, Some(1));
        assert!(nav_protocol_down(&mut state));
        assert_eq!(state.selected_sub, Some(2));
        // Overflow: last sub-row down → full row, caller moves to next profile.
        assert!(!nav_protocol_down(&mut state));
        assert_eq!(state.selected_sub, None);
        assert_eq!(state.selected_index, 0);
    }

    #[tokio::test]
    async fn up_at_first_sub_returns_to_full_row() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        toggle_expand(&mut state);
        assert!(nav_protocol_up(&mut state));
        assert_eq!(state.selected_sub, None);
        // Up on the full row falls through to endpoint nav (caller moves up).
        assert!(!nav_protocol_up(&mut state));
        assert_eq!(state.selected_sub, None);
    }

    #[tokio::test]
    async fn down_from_full_row_reenters_sub_table() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        toggle_expand(&mut state);
        nav_protocol_up(&mut state); // back to full row
        assert_eq!(state.selected_sub, None);
        assert!(nav_protocol_down(&mut state));
        assert_eq!(state.selected_sub, Some(0));
    }

    #[tokio::test]
    async fn collapsed_endpoint_down_moves_directly_and_clears_stale_sub() {
        let mut state = test_state(vec![fake_row(1, "a.com", 2), fake_row(2, "b.com", 1)]).await;
        state.selected_sub = Some(1); // stale sub selection
        assert!(!nav_protocol_down(&mut state));
        assert_eq!(state.selected_sub, None);
        assert_eq!(state.selected_index, 0);
        // Up on collapsed: same, stale cleared.
        state.selected_sub = Some(0);
        assert!(!nav_protocol_up(&mut state));
        assert_eq!(state.selected_sub, None);
    }
}

#[cfg(test)]
mod edit_tests {
    use super::*;
    use super::test_support::{fake_row, test_state};
    use crate::AppState;
    use std::sync::Arc;
    use toasty::Deferred;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{Endpoint, EndpointRow, ProfileExtension, ProtocolRow, ServerStat};

    fn matches_edit_mode(state: &AppState, protocol_id: i64) -> bool {
        matches!(state.mode, AppMode::EditServer { protocol_id: id, .. } if id == protocol_id)
    }

    fn assert_not_edit_mode(state: &AppState) {
        assert!(
            !matches!(state.mode, AppMode::EditServer { .. }),
            "expected no EditServer mode, got {:?}",
            state.mode
        );
    }

    #[tokio::test]
    async fn edit_resolves_against_current_view() {
        // The profile is on screen (state.endpoints) but absent from the DB —
        // a visible, manually-added profile that was never re-imported. The
        // old fixed 1-day DB lookup rejected it; the current view must win.
        let mut state = test_state(vec![fake_row(7, "visible.example", 1)]).await;
        start_edit_profile(&mut state, "7").await;
        assert!(matches_edit_mode(&state, 7));
    }

    #[tokio::test]
    async fn edit_falls_back_to_every_endpoint_for_stale_profile() {
        // Profile not loaded in the current view, but present in the DB with
        // last_seen_at older than the old 1-day threshold. The fallback must
        // use scope 0 (everything), not a fixed 86400s window.
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let group_id = db.get_all_groups().await.unwrap()[0].id.clone();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let endpoint = Endpoint {
            id: 42,
            host: "stale.example".to_string(),
            host_type: "dns".to_string(),
            port: 443,
            port_spec_str: None,
            parent_id: None,
            last_source: None,
            created_at: now - 2 * 86400,
            manual_protocol_override: None,
            resolved_as: None,
            resolved_at: None,
        };
        let protocol = ProtocolRow {
            id: 4200,
            endpoint_id: 42,
            sig: 0,
            cred_hash: 0,
            proto_kind: "vless".to_string(),
            spec_blob: Vec::new(),
            config_type: 1,
            core_type: "xray".to_string(),
            transport: None,
            security: None,
            last_used_at: Some(now - 2 * 86400),
            created_at: now - 2 * 86400,
            last_seen_at: now - 2 * 86400,
            endpoint: Deferred::from(None::<Endpoint>),
            extension: Deferred::from(None::<ProfileExtension>),
            server_stat: Deferred::from(None::<ServerStat>),
        };
        db.insert_manual_endpoint(&endpoint, &protocol, &group_id)
            .await
            .unwrap();
        let mut state = AppState::new(db, AppConfig::default()).await;
        state.endpoints = Vec::new(); // profile not loaded in the current view
        start_edit_profile(&mut state, "42").await;
        assert!(matches_edit_mode(&state, 42));
    }

    #[tokio::test]
    async fn edit_rejects_unknown_profile() {
        let mut state = test_state(vec![fake_row(1, "a.com", 1)]).await;
        start_edit_profile(&mut state, "999").await;
        assert_not_edit_mode(&state);
    }
}
