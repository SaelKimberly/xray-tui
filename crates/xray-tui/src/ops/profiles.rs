use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tracing::warn;

use xray_tui_core::protocol::Protocol;
use xray_tui_core::{CoreType, resolve_core};
use xray_tui_config::import_export::{encode_profile_spec, profile_config, ValidationSettings, Profile};
use xray_tui_config::AppConfig;
use crate::ProfileRow;
use xray_tui_db::Database;

use crate::AppState;
use crate::types::*;
use crate::{common_field_defaults, get_field, profile_to_fields, try_send_or_warn};
use crate::state::profile_to_endpoint_protocol;
use xray_tui_proto::proto_spec::ProtoSpec;
use xray_tui_db::models::{ProfileExtension, ServerStat};

pub async fn reload_profiles(state: &mut AppState) {
    match state.db.get_active_endpoints(0).await {
        Ok(rows) => {
            state.profiles = rows;
        }
        Err(e) => {
            state.log_trace("error", "tui", &format!("Failed to load profiles: {e}"));
            state.profiles.clear();
        }
    }
    state.filter_cache_valid.set(false);
}

pub async fn reload_groups(state: &mut AppState) {
    match state.db.get_all_groups().await {
        Ok(groups) => state.groups = groups,
        Err(e) => {
            state.log_trace("error", "tui", &format!("Failed to load groups: {e}"));
            state.groups.clear();
        }
    }
}

pub async fn reload_routing_rules(state: &mut AppState) {
    state.routing_rules = state.db.get_all_routing_rules().await.unwrap_or_default();
}

pub fn filtered_profiles(state: &AppState) -> impl Iterator<Item = &ProfileRow> {
    if !state.filter_cache_valid.get() {
        let indices = compute_filtered_indices(state);
        *state.cached_filtered_indices.borrow_mut() = indices;
        state.filter_cache_valid.set(true);
    }
    let indices: Vec<usize> = state.cached_filtered_indices.borrow().clone();
    indices.into_iter().map(move |i| &state.profiles[i])
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
        .profiles
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            if !state.search_query.is_empty() {
                let q = state.search_query.to_lowercase();
                let address = row.endpoint.host.clone();
                let port = row.endpoint.port.to_string();
                if !address.to_lowercase().contains(&q)
                    && !port.contains(&q)
                {
                    let remarks = row.active_protocol().remarks.clone().unwrap_or_default();
                    if !remarks.to_lowercase().contains(&q) {
                        return false;
                    }
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    let asc = state.sort_ascending;
    indices.sort_by(|&a, &b| {
        let a_row = &state.profiles[a];
        let b_row = &state.profiles[b];
        let cmp = match state.sort_column {
            SortColumn::ConfigType => a_row.active_protocol().config_type.cmp(&b_row.active_protocol().config_type),
            SortColumn::Remarks => {
                let a_rem = a_row.active_protocol().remarks.clone().unwrap_or_default();
                let b_rem = b_row.active_protocol().remarks.clone().unwrap_or_default();
                a_rem.cmp(&b_rem)
            }
            SortColumn::Address => a_row.endpoint.host.cmp(&b_row.endpoint.host),
            SortColumn::Port => a_row.endpoint.port.cmp(&b_row.endpoint.port),
            SortColumn::Delay => {
                let da = a_row.extensions.get(&a_row.active_protocol().id).and_then(|e| e.delay).unwrap_or(-1);
                let db = b_row.extensions.get(&b_row.active_protocol().id).and_then(|e| e.delay).unwrap_or(-1);
                da.cmp(&db)
            }
            SortColumn::Speed => {
                let sa = a_row.extensions.get(&a_row.active_protocol().id).and_then(|e| e.speed).unwrap_or(-1);
                let sb = b_row.extensions.get(&b_row.active_protocol().id).and_then(|e| e.speed).unwrap_or(-1);
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
                let resolve = |row: &ProfileRow| -> String {
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

pub fn cycle_group(state: &mut AppState, _dir: i8) {
    if state.groups.is_empty() {
        return;
    }
    let _current_idx = state
        .selected_group_id
        .as_ref()
        .and_then(|id| state.groups.iter().position(|g| g.id == *id));
    let _len = state.groups.len();
    if state.log_scroll != 0 {
        state.log_scroll = state.log_scroll.saturating_add(1);
    }
    if state.log_cache.len() > 10000 {
        let excess = state.log_cache.len() - 10000;
        state.log_cache.drain(0..excess);
        state.log_scroll = state.log_scroll.saturating_sub(excess);
    }
}

pub fn resolved_core(state: &AppState, row: &ProfileRow) -> CoreType {
    let protocol = Protocol::try_from_i32(row.active_protocol().config_type).unwrap_or(Protocol::Custom);
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

pub async fn start_edit_profile(state: &mut AppState, id: &str) {
    let protocol_id: i64 = id.parse().unwrap_or(0);
    match state.db.get_active_endpoints(86400).await {
        Ok(rows) => {
            if let Some(row) = rows.iter().find(|r| r.endpoint.id == protocol_id) {
                state.mode = AppMode::EditServer {
                    protocol_id,
                    fields: Vec::new(),
                    focus_index: 0,
                    form_errors: HashMap::new(),
                };
            } else {
                state.log_trace("error", "tui", &format!("Profile {id} not found"));
            }
        }
        Err(e) => state.log_trace("error", "tui", &format!("Error loading profile {id}: {e}")),
    }
}

pub fn selected_profile_id(state: &AppState) -> Option<i64> {
    filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id)
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
    let mut remarks: Option<String> = None;
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
            "remarks" => {
                remarks = Some(xray_tui_config::import_export::normalize_remark(value));
            }
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

    let protocol_settings_str = if !proto_map.is_empty() {
        Some(serde_json::to_string(&proto_map).unwrap_or_default())
    } else {
        None
    };
    let stream_settings_str = if !stream_map.is_empty() {
        Some(serde_json::to_string(&stream_map).unwrap_or_default())
    } else {
        None
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
        transport: network.clone(),
        security,
        created_at: now as i64,
        remarks: remarks.clone(),
        user_id: user_id.clone(),
        network,
        protocol_settings: protocol_settings_str.clone(),
        stream_settings: stream_settings_str.clone(),
    };
    let mut extra = serde_json::Map::new();
    if let Some(v) = &remarks {
        extra.insert("remarks".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(v) = &user_id {
        extra.insert("user_id".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(v) = &protocol_settings_str {
        if let Ok(val) = serde_json::from_str(v) {
            extra.insert("protocol_settings".into(), val);
        }
    }
    if let Some(v) = &stream_settings_str {
        if let Ok(val) = serde_json::from_str(v) {
            extra.insert("stream_settings".into(), val);
        }
    }
    profile.spec_blob = encode_profile_spec(
        &proto_kind,
        serde_json::to_vec(&extra).unwrap_or_default(),
    );
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
            state.log_trace("error", "tui", "Cannot confirm: no protocol selected");
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
    let group_id = state.selected_group_id.clone().unwrap_or_default();
    let (endpoint, protocol) = profile_to_endpoint_protocol(&profile);
    match state.db.insert_manual_endpoint(&endpoint, &protocol, &group_id).await {
        Ok(()) => {
            let remarks = profile_config(&profile)
                .and_then(|c| c.remarks().map(String::from))
                .unwrap_or_else(|| "unnamed".to_string());
            state.log_trace("info", "tui", &format!("Added server: {remarks}"));
            state.mode = AppMode::List;
            state.profile_gen = state.profile_gen.wrapping_add(1);
            upsert_profile_row(state, profile, None, None, state.selected_group_id.clone());
        }
        Err(e) => {
            state.log_trace("error", "tui", &format!("Failed to add server: {e}"));
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
                protocol_id, fields, ..
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
        state.db
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

    if state.db.get_endpoint(protocol_id).await.ok().flatten().is_none() {
        state.log_trace("error", "tui", "Profile not found for edit");
        return;
    }
    let new_profile = fields_to_profile(protocol, &fields);
    let (endpoint, protocol_row) = profile_to_endpoint_protocol(&new_profile);
    let group_id = state.selected_group_id.clone().unwrap_or_default();
    match state.db.subscription_upsert(&group_id, &[(endpoint, vec![protocol_row])]).await {
        Ok(_ids) => {
            let remarks = profile_config(&new_profile)
                .and_then(|c| c.remarks().map(String::from))
                .unwrap_or_else(|| "unnamed".to_string());
            state.log_trace("info", "tui", &format!("Updated server: {remarks}"));
            state.mode = AppMode::List;
            state.profile_gen = state.profile_gen.wrapping_add(1);
            reload_profiles(state).await;
        }
        Err(e) => {
            state.log_trace("error", "tui", &format!("Failed to update server: {e}"));
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
        state.log_trace("error", "tui", &format!("Failed to delete profile: {e}"));
        return;
    }
    state.log_trace("info", "tui", "Profile deleted");
    state.confirmation = None;
    state.multi_select.remove(&id);
    state.profile_gen = state.profile_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn clone_profile(state: &mut AppState, id: i64) {
    let found = state.profiles.iter().find(|r| r.endpoint.id == id);
    if found.is_none() {
        state.log_trace("error", "tui", &format!("Profile {id} not found for cloning"));
        return;
    }
    state.log_trace("info", "tui", "Profile cloned");
    state.profile_gen = state.profile_gen.wrapping_add(1);
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
            let protocol =
                Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
            let fields = profile_to_fields(&profile);
            state.mode = AppMode::AddServer {
                protocol: Some(protocol),
                fields,
                focus_index: 0,
                form_errors: HashMap::new(),
            };
            state.log_trace("info", "tui", "URL imported successfully");
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
    let group_id = state.selected_group_id.clone().unwrap_or_default();
    for item in items {
        if let Some(profile) = item.profile {
            let (endpoint, protocol) = profile_to_endpoint_protocol(&profile);
            if state.db.insert_manual_endpoint(&endpoint, &protocol, &group_id).await.is_ok() {
                imported += 1;
            } else {
                errors += 1;
            }
        }
    }
    state.log_trace(
        "info",
        "tui",
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
    let filtered: Vec<&ProfileRow> = filtered_profiles(state).collect();
    let idx = filtered.iter().position(|r| r.endpoint.id == id);
    let idx = match idx {
        Some(i) if i > 0 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui", "Profile moved up");
    state.profile_gen = state.profile_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn move_profile_down(state: &mut AppState) {
    let id = match selected_profile_id(state) {
        Some(id) => id,
        None => return,
    };
    let filtered: Vec<&ProfileRow> = filtered_profiles(state).collect();
    let idx = match filtered.iter().position(|r| r.endpoint.id == id) {
        Some(i) if i < filtered.len() - 1 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui", "Profile moved down");
    state.profile_gen = state.profile_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn set_active(state: &mut AppState, id: &str) {
    let pid: i64 = id.parse().unwrap_or(0);
    let group_id = state.selected_group_id.clone().unwrap_or_default();
    if let Err(e) = state.db.set_protocol_override(pid, 0).await {
        state.log_trace("error", "tui", &format!("Failed to set active: {e}"));
        return;
    }
    state.profile_gen = state.profile_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub fn upsert_profile_row(
    state: &mut AppState,
    _profile: Profile,
    _ext: Option<ProfileExtension>,
    _stat: Option<ServerStat>,
    _group_id: Option<String>,
) {
    state.profile_gen = state.profile_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}
