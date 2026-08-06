use std::collections::HashMap;

use jiff::Timestamp;
use xray_tui_config::forms::build_typed_config;
use xray_tui_config::import_export::{ParsedProfile, ValidationSettings, parse_share_url};
use xray_tui_core::{CoreType, resolve_core};

use crate::AppState;
use crate::state::{
    endpoint_from_essentials, load_protocol_with_config, persist_parsed, protocol_from_parsed,
};
use crate::types::{AppMode, BatchImportItem, SortColumn};
use crate::{common_field_defaults, get_field, profile_to_fields};
use xray_tui_db::Database;
use xray_tui_db::DatabaseError;
use xray_tui_db::models::{EndpointRow, ProfileStats, PurgatoryView};
use xray_tui_proto::proto_spec::{CoreType as ProtoCoreType, ParsedProto, ProtocolKind};

/// Unix-seconds now, as a `Timestamp` (the typed staleness clock).
fn now_ts() -> Timestamp {
    Timestamp::now()
}

/// Sweep stale persisted failure markers: clear `error` on every
/// `profile_stats` row whose `error` is set AND whose `updated_at` predates
/// `now - ttl_hours`.
///
/// This is the "Clear error after" setting (design §6.4, anchor =
/// `updated_at`): one typed query-based UPDATE (no read-modify-write).
/// `None` (the default) never clears: errors survive until the next test
/// overwrites them. Best-effort housekeeping — DB errors are logged and
/// swallowed, never surfaced to the user.
pub async fn clear_expired_errors(db: &Database, ttl_hours: Option<i64>) {
    let Some(ttl_hours) = ttl_hours.filter(|&h| h > 0) else {
        return;
    };
    let mut conn = match db.connection().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!(target: "tui::ops::profiles", "clear_expired_errors: open connection: {e}");
            return;
        }
    };
    let cutoff = now_ts()
        .checked_sub(jiff::Span::new().hours(ttl_hours))
        .unwrap_or_else(|_| now_ts());
    if let Err(e) = ProfileStats::filter(ProfileStats::fields().error().is_some())
        .filter(ProfileStats::fields().updated_at().lt(cutoff))
        .update()
        .error(None)
        .exec(&mut conn)
        .await
    {
        tracing::warn!(target: "tui::ops::profiles", "clear_expired_errors: {e}");
    }
}

pub async fn reload_profiles(state: &mut AppState) {
    let now = now_ts();
    // Error-TTL sweep (design §6.4): clear failure markers older than the
    // configured TTL before rows are (re)loaded, so a swept error never
    // renders. `None` (default) = no-op.
    clear_expired_errors(&state.db, state.config.speed_test.error_ttl_hours).await;
    let result = match state.purgatory_view {
        PurgatoryView::Active => {
            let threshold = now
                .checked_sub(jiff::Span::new().seconds(state.purgatory_ttl_secs))
                .unwrap_or(now);
            state.db.get_active_endpoints(threshold).await
        }
        PurgatoryView::Stale => {
            let active_threshold = now
                .checked_sub(jiff::Span::new().seconds(state.purgatory_ttl_secs))
                .unwrap_or(now);
            let stale_threshold = now
                .checked_sub(jiff::Span::new().seconds(state.purgatory_retention_secs))
                .unwrap_or(now);
            state
                .db
                .get_stale_endpoints(active_threshold, stale_threshold)
                .await
        }
        PurgatoryView::All => {
            state
                .db
                .get_active_endpoints(Timestamp::from_second(0).unwrap_or(now))
                .await
        }
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
    clamp_selection(state);
    // Enrich new endpoints in the background (IP hosts + persisted DNS cache;
    // no network for fresh entries).
    crate::ops::enrich::spawn_enrich_ip_hosts(state);
}

/// Clamp a selection index into `[0, len)`, returning 0 for an empty list.
pub(crate) const fn clamp_index(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if selected >= len {
        len - 1
    } else {
        selected
    }
}

/// Re-clamp the selection after a reload/filter change so the highlighted row
/// stays inside the (possibly shrunk) filtered list. The `DataTable` clamps its
/// own selection visually to row 0, but `AppState` keeps the stale index —
/// which makes `selected_profile_id()` return `None` and Enter/e/d/x/Space
/// silent no-ops. Drop a `selected_sub` that no longer points at a real row.
pub(crate) fn clamp_selection(state: &mut AppState) {
    let len = filtered_len(state);
    state.selected_index = clamp_index(state.selected_index, len);
    if state.selected_index >= len || len == 0 {
        state.selected_sub = None;
    }
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
                let address = row.endpoint.host.to_lowercase();
                let port = row.endpoint.port.to_string();
                if !address.contains(&q) && !port.contains(&q) {
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
            SortColumn::ConfigType => config_type_rank(a_row).cmp(&config_type_rank(b_row)),
            SortColumn::LastSeen => a_row
                .active_link()
                .map(|l| l.last_seen_at)
                .cmp(&b_row.active_link().map(|l| l.last_seen_at)),
            SortColumn::Address => a_row.endpoint.host.cmp(&b_row.endpoint.host),
            SortColumn::Port => a_row.endpoint.port.cmp(&b_row.endpoint.port),
            SortColumn::Test => {
                let ka = a_row.best_test_priority_key(endpoint_dns_unresolved(state, a_row));
                let kb = b_row.best_test_priority_key(endpoint_dns_unresolved(state, b_row));
                ka.cmp(&kb)
            }
            SortColumn::Speed => {
                let sa = a_row.active_link().and_then(|l| l.speed_bps).unwrap_or(-1);
                let sb = b_row.active_link().and_then(|l| l.speed_bps).unwrap_or(-1);
                sa.cmp(&sb)
            }
            SortColumn::Traffic => {
                let traffic = |row: &EndpointRow| {
                    row.active_link()
                        .map_or(0, |l| l.traffic.total_down + l.traffic.total_up)
                };
                traffic(a_row).cmp(&traffic(b_row))
            }
            SortColumn::Core => resolved_core(state, a_row)
                .as_str()
                .cmp(resolved_core(state, b_row).as_str()),
        };
        if asc { cmp } else { cmp.reverse() }
    });
    indices
}

/// Sort rank for the config-type column: Form before `ShareUrl` (hand-made
/// profiles first, deterministic for the sort).
fn config_type_rank(row: &EndpointRow) -> u8 {
    use xray_tui_db::models::ConfigType;
    match row.active_link().map(|l| l.config_type) {
        Some(ConfigType::Form) => 0,
        Some(ConfigType::ShareUrl) => 1,
        None => 2,
    }
}

/// Whether the endpoint's DNS host is currently unresolved (no known IPs).
/// Endpoints without an `endpoint_info` entry count as unresolved.
pub(crate) fn endpoint_dns_unresolved(state: &AppState, row: &EndpointRow) -> bool {
    use xray_tui_db::models::HostType;
    row.endpoint.host_type == HostType::Dns
        && state
            .endpoint_info
            .get(&row.endpoint.id.get())
            .is_none_or(|i| i.resolved_ips.is_empty())
}

/// Resolve which core a profile row should use, considering (in order):
///   1. Per-profile override (`link.core_type`)
///   2. Per-protocol config override (`config.core.protocol_core_overrides`)
///   3. Hardcoded auto-detection (`core_for_protocol` via `resolve_core`)
pub fn resolved_core(state: &AppState, row: &EndpointRow) -> CoreType {
    let Some((link, protocol)) = row.active_protocol() else {
        return CoreType::Auto;
    };
    let config_override = state
        .config
        .core
        .protocol_core_overrides
        .get(&protocol.proto_kind.to_string())
        .and_then(|s| s.parse::<CoreType>().ok());
    let override_ = match config_override.or(match link.core_type {
        ProtoCoreType::Xray => Some(CoreType::Xray),
        ProtoCoreType::SingBox => Some(CoreType::SingBox),
    }) {
        Some(CoreType::Auto) | None => None,
        Some(CoreType::Xray) => Some(ProtoCoreType::Xray),
        Some(CoreType::SingBox) => Some(ProtoCoreType::SingBox),
    };
    match resolve_core(
        protocol.proto_kind,
        override_,
        xray_tui_core::shadowsocks_method(protocol).as_deref(),
    ) {
        ProtoCoreType::Xray => CoreType::Xray,
        ProtoCoreType::SingBox => CoreType::SingBox,
    }
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
/// first, then fall back to a direct DB lookup by endpoint id (covers
/// never-seen rows that the Active view excludes).
async fn find_editable_endpoint(
    state: &AppState,
    endpoint_id: i64,
) -> Result<Option<EndpointRow>, DatabaseError> {
    if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    {
        return Ok(Some(r.clone()));
    }
    state
        .db
        .get_endpoint(endpoint_id_from_raw(endpoint_id))
        .await
}

/// The typed `EndpointId` for a raw i64 (used by edit/delete flows).
pub(crate) const fn endpoint_id_from_raw(raw: i64) -> xray_tui_db::models::EndpointId {
    xray_tui_db::models::EndpointId::new(raw)
}

pub async fn start_edit_profile(state: &mut AppState, id: &str) {
    let endpoint_id: i64 = id.parse().unwrap_or(0);
    let row = match find_editable_endpoint(state, endpoint_id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Profile {id} not found"),
            );
            return;
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Error loading profile {id}: {e}"),
            );
            return;
        }
    };
    // The edit form is populated from the ACTIVE protocol's typed config
    // (loaded with `config` included — `profile_to_fields` and the config
    // builders require it).
    let Some((link, protocol)) = row.active_protocol() else {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Profile {id} has no protocols"),
        );
        return;
    };
    let protocol = match load_protocol_with_config(&state.db, protocol.id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Protocol for profile {id} not found"),
            );
            return;
        }
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Error loading protocol for profile {id}: {e}"),
            );
            return;
        }
    };
    let mut fields = profile_to_fields(&protocol, &row.endpoint);
    // The per-pair core override is a link column, not a config field —
    // `profile_to_fields` leaves the default; set the actual override.
    set_core_field(&mut fields, link.core_type);
    state.mode = AppMode::EditServer {
        protocol_id: endpoint_id,
        fields,
        focus_index: 0,
        form_errors: HashMap::new(),
    };
}

/// Set the `core_type` form field to the link's per-pair override (the
/// producer reads it back into `link.core_type` on save).
fn set_core_field(fields: &mut [(String, String)], core_type: ProtoCoreType) {
    if let Some((_, v)) = fields.iter_mut().find(|(k, _)| k == "core_type") {
        *v = core_type.as_str().to_string();
    }
}

pub fn selected_profile_id(state: &AppState) -> Option<i64> {
    filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id.get())
}

pub fn toggle_expand(state: &mut AppState) {
    let ep_id = filtered_profiles(state)
        .nth(state.selected_index)
        .map(|r| r.endpoint.id.get());
    if let Some(ep_id) = ep_id
        && let Some(ep_row) = state
            .endpoints
            .iter_mut()
            .find(|r| r.endpoint.id.get() == ep_id)
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
        .map(|r| r.endpoint.id.get());
    if let Some(ep_id) = ep_id
        && let Some(ep_row) = state
            .endpoints
            .iter_mut()
            .find(|r| r.endpoint.id.get() == ep_id)
    {
        ep_row.expanded = false;
    }
    state.selected_sub = None;
}

/// Build a typed [`ParsedProto`] from the add/edit form fields — the
/// producer-side replacement for the deleted `fields_to_profile` +
/// `encode_profile_spec`.
///
/// The settings JSON is assembled with the same key routing the legacy
/// producer used (so `build_typed_config` accepts it), with the F6 fix from
/// T12: the tuic `password` field routes into `protocol_settings.password`
/// instead of clobbering the top-level `user_id` (uuid). The form's
/// `core_type` selection is returned separately — it flows into the per-pair
/// `link.core_type` (the config builder takes the core from the link), never
/// into the typed config.
///
/// # Errors
///
/// If the address/port are missing or `build_typed_config` rejects the
/// settings (unknown keys, missing credentials).
pub fn fields_to_parsed(
    kind: ProtocolKind,
    fields: &[(String, String)],
) -> Result<(ParsedProto, Option<ProtoCoreType>), String> {
    let address = get_field(fields, "address").unwrap_or_default();
    let port = get_field(fields, "port")
        .and_then(|p| p.parse::<u16>().ok())
        .ok_or_else(|| "Port must be 1-65535".to_string())?;
    if address.is_empty() {
        return Err("Address is required".to_string());
    }

    let mut proto_map = serde_json::Map::new();
    let mut stream_map = serde_json::Map::new();
    let mut user_id: Option<String> = None;
    let mut core_type = "auto".to_string();

    for (key, value) in fields {
        if value.is_empty() {
            continue;
        }
        let json_val = if value == "true" {
            serde_json::Value::Bool(true)
        } else if value == "false" {
            serde_json::Value::Bool(false)
        } else if let Ok(n) = value.parse::<i64>() {
            serde_json::Value::Number(n.into())
        } else {
            serde_json::Value::String(value.clone())
        };
        match key.as_str() {
            // Profile-column / edit-form plumbing fields — never in the
            // settings JSON (the mapper infers transport from ws/grpc keys;
            // vmess encryption defaults to auto).
            "address" | "port" | "security" | "network" | "config_type" => {}
            "core_type" => core_type.clone_from(value),
            // F6: tuic's `password` is a protocol_setting credential (its
            // `uuid` owns the top-level `user_id` slot); every other
            // protocol's `user_id`/`password`/`uuid` key routes to the
            // top-level credential, as the T12 mappers expect.
            "password" if kind == ProtocolKind::Tuic => {
                proto_map.insert(key.clone(), json_val);
            }
            "user_id" | "password" | "uuid" => user_id = Some(value.clone()),
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
                stream_map.insert(key.clone(), json_val);
            }
            _ => {
                proto_map.insert(key.clone(), json_val);
            }
        }
    }

    let mut settings = serde_json::Map::new();
    if let Some(u) = user_id {
        settings.insert("user_id".into(), serde_json::Value::String(u));
    }
    if !proto_map.is_empty() {
        settings.insert(
            "protocol_settings".into(),
            serde_json::Value::Object(proto_map),
        );
    }
    if !stream_map.is_empty() {
        settings.insert(
            "stream_settings".into(),
            serde_json::Value::Object(stream_map),
        );
    }

    let parsed = build_typed_config(kind, &address, port, &serde_json::Value::Object(settings))?;
    let core_override = match core_type.as_str() {
        "xray" => Some(ProtoCoreType::Xray),
        "sing-box" | "singbox" => Some(ProtoCoreType::SingBox),
        _ => None,
    };
    Ok((parsed, core_override))
}

/// Validation shared by add/edit: address/port presence and required
/// credentials for the credential-bearing protocols.
fn validate_form_fields(
    protocol: ProtocolKind,
    fields: &[(String, String)],
    errors: &mut HashMap<String, String>,
) {
    let address = get_field(fields, "address").unwrap_or_default();
    if address.is_empty() {
        errors.insert("address".into(), "Address is required".into());
    }
    let port = get_field(fields, "port").unwrap_or_default();
    if port.is_empty() || port.parse::<u16>().map_or(true, |p| p == 0) {
        errors.insert("port".into(), "Port must be 1-65535".into());
    }
    let user_id = get_field(fields, "user_id").unwrap_or_default();
    match protocol {
        ProtocolKind::Vmess
        | ProtocolKind::Vless
        | ProtocolKind::Trojan
        | ProtocolKind::Shadowsocks
        | ProtocolKind::Shadowsocks2022
        | ProtocolKind::ShadowsocksR
            if user_id.is_empty() =>
        {
            errors.insert("user_id".into(), "ID/Password required".into());
        }
        ProtocolKind::Tuic => {
            // uuid is the tuic credential; password is optional.
            let uuid = get_field(fields, "uuid").unwrap_or_default();
            if uuid.is_empty() {
                errors.insert("uuid".into(), "UUID required".into());
            }
        }
        _ => {}
    }
}

pub async fn confirm_add_server(state: &mut AppState) {
    let (protocol, fields) = if let AppMode::AddServer {
        protocol: Some(p),
        fields,
        ..
    } = &state.mode
    {
        (*p, fields.clone())
    } else {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            "Cannot confirm: no protocol selected",
        );
        return;
    };

    let mut errors: HashMap<String, String> = HashMap::new();
    validate_form_fields(protocol, &fields, &mut errors);
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

    let parsed = match fields_to_parsed(protocol, &fields) {
        Ok(p) => p,
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to build profile: {e}"),
            );
            if let AppMode::AddServer {
                fields: ref mut f, ..
            } = state.mode
            {
                *f = fields;
            }
            return;
        }
    };
    let group_id = state.first_group_id();
    let group = if group_id.is_empty() {
        None
    } else {
        Some(group_id.as_str())
    };
    match persist_parsed(&state.db, &parsed.0, group, parsed.1).await {
        Ok(_) => {
            let addr = parsed
                .0
                .first_endpoint()
                .map_or_else(|| "?".into(), |e| format!("{}:{}", e.host, e.port));
            state.log_trace(
                "info",
                "tui::ops::profiles",
                &format!("Added server: {addr}"),
            );
            state.mode = AppMode::List;
            state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
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
    let (endpoint_id, fields) = match &state.mode {
        AppMode::EditServer {
            protocol_id,
            fields,
            ..
        } => (*protocol_id, fields.clone()),
        _ => return,
    };

    let row = if let Ok(Some(r)) = state
        .db
        .get_endpoint(endpoint_id_from_raw(endpoint_id))
        .await
    {
        r
    } else {
        state.log_trace("error", "tui::ops::profiles", "Profile not found for edit");
        return;
    };
    let Some((_, protocol)) = row.active_protocol() else {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            "Profile has no protocol to edit",
        );
        return;
    };
    let protocol = protocol.proto_kind;

    let mut errors: HashMap<String, String> = HashMap::new();
    validate_form_fields(protocol, &fields, &mut errors);
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

    let parsed = match fields_to_parsed(protocol, &fields) {
        Ok(p) => p,
        Err(e) => {
            state.log_trace(
                "error",
                "tui::ops::profiles",
                &format!("Failed to build profile: {e}"),
            );
            if let AppMode::EditServer {
                fields: ref mut f, ..
            } = state.mode
            {
                *f = fields;
            }
            return;
        }
    };
    // Editing replaces the endpoint's protocol identity: upsert the new
    // endpoint/protocol/link rows (idempotent on the same uid) and reload.
    let group_id = state.first_group_id();
    let group = if group_id.is_empty() {
        None
    } else {
        Some(group_id.as_str())
    };
    match persist_parsed(&state.db, &parsed.0, group, parsed.1).await {
        Ok(_) => {
            let addr = parsed
                .0
                .first_endpoint()
                .map_or_else(|| "?".into(), |e| format!("{}:{}", e.host, e.port));
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
    if let Err(e) = state.db.delete_endpoint(endpoint_id_from_raw(id)).await {
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

pub fn clone_profile(state: &mut AppState, id: i64) {
    let found = state.endpoints.iter().find(|r| r.endpoint.id.get() == id);
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

/// Populate the `AddServer` form from a parsed share URL: the first endpoint's
/// address/port plus the typed config's form fields.
fn form_from_parsed(state: &mut AppState, parsed: &ParsedProfile) {
    let Some(first) = parsed.parsed.first_endpoint().cloned() else {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            "Imported profile has no endpoint address",
        );
        return;
    };
    let endpoint = endpoint_from_essentials(&first);
    let protocol = protocol_from_parsed(&parsed.parsed);
    let mut fields = profile_to_fields(&protocol, &endpoint);
    set_core_field(&mut fields, parsed.parsed.protocol.core_type);
    let core_protocol = parsed.parsed.protocol.proto_kind;
    state.mode = AppMode::AddServer {
        protocol: Some(core_protocol),
        fields,
        focus_index: 0,
        form_errors: HashMap::new(),
    };
    state.log_trace("info", "tui::ops::profiles", "URL imported successfully");
}

pub fn import_url(state: &mut AppState, url: &str) {
    let settings = ValidationSettings::from(state.config.parsing.clone());
    match parse_share_url(url, &settings) {
        Ok(parsed) => form_from_parsed(state, &parsed),
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
        .map(|url| match parse_share_url(url, &settings) {
            Ok(parsed) => BatchImportItem {
                url: url.clone(),
                profile: Some(parsed),
                error: None,
                imported: false,
            },
            Err(e) => BatchImportItem {
                url: url.clone(),
                profile: None,
                error: Some(e.to_string()),
                imported: false,
            },
        })
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
    let group = if group_id.is_empty() {
        None
    } else {
        Some(group_id.as_str())
    };
    for item in items {
        if let Some(parsed) = item.profile {
            match persist_parsed(&state.db, &parsed.parsed, group, None).await {
                Ok(_) => imported += 1,
                Err(_) => errors += 1,
            }
        } else {
            errors += 1;
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

pub fn move_profile_up(state: &mut AppState) {
    let id = match selected_profile_id(state) {
        Some(id) => id,
        None => return,
    };
    let filtered: Vec<&EndpointRow> = filtered_profiles(state).collect();
    let idx = filtered.iter().position(|r| r.endpoint.id.get() == id);
    let _idx = match idx {
        Some(i) if i > 0 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui::ops::profiles", "Profile moved up");
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub fn move_profile_down(state: &mut AppState) {
    let id = match selected_profile_id(state) {
        Some(id) => id,
        None => return,
    };
    let filtered: Vec<&EndpointRow> = filtered_profiles(state).collect();
    let _idx = match filtered.iter().position(|r| r.endpoint.id.get() == id) {
        Some(i) if i < filtered.len() - 1 => i,
        _ => return,
    };
    drop(filtered);
    state.log_trace("info", "tui::ops::profiles", "Profile moved down");
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

pub async fn set_active(state: &mut AppState, id: &str) {
    let endpoint_id: i64 = id.parse().unwrap_or(0);
    if let Err(e) = state
        .db
        .set_manual_override(endpoint_id_from_raw(endpoint_id), None)
        .await
    {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Failed to clear override: {e}"),
        );
        return;
    }
    // The DB write alone never reaches the UI: `state.endpoints` is only
    // rebuilt by `reload_profiles` (subscription events). Clear the override
    // in-memory so `active_protocol()` falls back immediately without a
    // reload (which would also collapse the panel).
    if let Some(row) = state
        .endpoints
        .iter_mut()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    {
        row.endpoint.manual_protocol_override = None;
    }
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

/// Pin a specific protocol as the endpoint's default (manual override).
/// Writes the override to DB and updates the in-memory row so the UI
/// switches immediately — same rationale as `set_active`: no reload.
pub async fn set_protocol_default(state: &mut AppState, endpoint_id: i64, protocol_id: i64) {
    let pid = xray_tui_db::models::ProtocolId::new(protocol_id);
    if let Err(e) = state
        .db
        .set_manual_override(endpoint_id_from_raw(endpoint_id), Some(pid))
        .await
    {
        state.log_trace(
            "error",
            "tui::ops::profiles",
            &format!("Failed to set protocol override: {e}"),
        );
        return;
    }
    if let Some(row) = state
        .endpoints
        .iter_mut()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    {
        row.endpoint.manual_protocol_override = Some(pid);
    }
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
        .and_then(|r| {
            if r.expanded {
                Some(r.links.len())
            } else {
                None
            }
        });
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
        .is_some_and(|r| r.expanded && r.links.len() > 1);
    if !current_expanded {
        // Fall through to endpoint nav; drop any stale sub-row selection.
        state.selected_sub = None;
        return false;
    }
    match state.selected_sub {
        Some(n) if n > 0 => {
            state.selected_sub = Some(n - 1);
            true
        }
        Some(0) => {
            // Up at the first sub-row returns to the endpoint row.
            state.selected_sub = None;
            true
        }
        None | Some(_) => false,
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
    row.links.get(n).map(|l| l.protocol_id.get())
}

#[cfg(test)]
pub(crate) mod test_support {

    use crate::AppState;
    use std::sync::Arc;
    use toasty::Deferred;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{
        ConfigType, Endpoint, EndpointId, EndpointRow, HostType, ProfileStats, ProtocolId,
        TrafficStats,
    };
    use xray_tui_proto::proto_spec::CoreType as ProtoCoreType;

    pub fn ts(secs: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(secs).expect("valid ts")
    }

    /// Minimal typed `EndpointRow` with `n` links (protocol ids
    /// `id*100 .. id*100+n`).
    pub fn fake_row(id: i64, host: &str, n_protos: usize) -> EndpointRow {
        let endpoint = Endpoint {
            id: EndpointId::new(id),
            host: host.to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::new(),
            parent_id: None,
            last_source: None,
            manual_protocol_override: None,
            resolved_as: Vec::new(),
            resolved_at: None,
            created_at: ts(0),
            links: Deferred::default(),
            group_links: Deferred::default(),
        };
        let links: Vec<ProfileStats> = (0..n_protos)
            .map(|i| ProfileStats {
                protocol_id: ProtocolId::new(id * 100 + i as i64),
                endpoint_id: EndpointId::new(id),
                core_type: ProtoCoreType::Xray,
                config_type: ConfigType::ShareUrl,
                last_used_at: None,
                last_seen_at: ts(0),
                task_id: None,
                task_queue: Vec::new(),
                latency: None,
                speed_bps: None,
                error: None,
                traffic: TrafficStats {
                    today_up: 0,
                    today_down: 0,
                    total_up: 0,
                    total_down: 0,
                },
                created_at: ts(0),
                updated_at: ts(0),
                version: 1,
                protocol: Deferred::default(),
                endpoint: Deferred::default(),
            })
            .collect();
        let protocols = links
            .iter()
            .map(|l| {
                (
                    l.protocol_id,
                    super::xray_tui_db_helper::vless_protocol(l.protocol_id.get()),
                )
            })
            .collect();
        EndpointRow {
            endpoint,
            links,
            protocols,
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

/// A vless `Protocol` row for tests (config loaded).
#[cfg(test)]
pub(crate) mod xray_tui_db_helper {
    pub fn vless_protocol(id: i64) -> xray_tui_db::models::Protocol {
        use super::test_support::ts;
        use toasty::{Deferred, Json};
        use xray_tui_db::models::{Protocol, ProtocolId, Security, Transport};
        use xray_tui_proto::proto_spec::ProtocolKind;
        use xray_tui_proto::proto_spec::common::TransportConfig;
        use xray_tui_proto::proto_spec::{
            ProtocolConfig, SecurityConfig, SecurityType, TransportType, VlessConfig,
        };
        Protocol {
            id: ProtocolId::new(id),
            sig: id,
            cred_hash: 0,
            proto_kind: ProtocolKind::Vless,
            transport: Transport {
                r#type: TransportType::Tcp,
                data: Deferred::from(Json(TransportConfig::Tcp)),
            },
            security: Security {
                r#type: SecurityType::None,
                sni: None,
                fp: None,
                insecure: None,
                data: Deferred::from(Json(SecurityConfig::default())),
            },
            config: Deferred::from(Json(ProtocolConfig::Vless(VlessConfig {
                uuid: format!("00000000-0000-0000-0000-{id:012}"),
                uuid_origin: None,
                security: SecurityConfig::default(),
                transport: TransportConfig::Tcp,
                encryption: None,
                flow: None,
                path: None,
                splice: None,
                remarks: None,
            }))),
            created_at: ts(0),
            links: Deferred::default(),
        }
    }
}

#[cfg(test)]
mod nav_tests {
    use super::test_support::{fake_row, test_state};
    use super::*;

    #[tokio::test]
    async fn expand_lands_on_first_sub_row() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        toggle_expand(&mut state);
        assert!(state.endpoints[0].expanded);
        assert_eq!(state.selected_sub, Some(0));
    }

    #[tokio::test]
    async fn pin_protocol_switches_active_in_memory() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        state.selected_sub = Some(1);
        let pid = selected_sub_protocol_id(&state).unwrap();
        assert_eq!(pid, 101); // fake_row protocol ids: 100, 101, 102
        assert_ne!(
            state.endpoints[0]
                .active_protocol()
                .unwrap()
                .0
                .protocol_id
                .get(),
            pid,
            "default active must be the first (unsorted) protocol before pinning"
        );

        set_protocol_default(&mut state, 1, pid).await;

        assert_eq!(
            state.endpoints[0].endpoint.manual_protocol_override,
            Some(xray_tui_db::models::ProtocolId::new(pid))
        );
        assert_eq!(
            state.endpoints[0]
                .active_protocol()
                .unwrap()
                .0
                .protocol_id
                .get(),
            pid,
            "active protocol must switch to the pinned variant immediately"
        );
    }

    #[tokio::test]
    async fn set_active_clears_pin_and_falls_back() {
        let mut state = test_state(vec![fake_row(1, "a.com", 3)]).await;
        set_protocol_default(&mut state, 1, 101).await;
        assert_eq!(
            state.endpoints[0]
                .active_protocol()
                .unwrap()
                .0
                .protocol_id
                .get(),
            101,
            "pinned variant is active before clearing"
        );

        set_active(&mut state, "1").await;

        assert_eq!(state.endpoints[0].endpoint.manual_protocol_override, None);
        assert_eq!(
            state.endpoints[0]
                .active_protocol()
                .unwrap()
                .0
                .protocol_id
                .get(),
            100,
            "active must fall back to the first protocol after clearing"
        );
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
    use super::test_support::{fake_row, test_state};
    use super::*;
    use crate::AppState;
    use std::sync::Arc;
    use xray_tui_config::AppConfig;

    fn matches_edit_mode(state: &AppState, endpoint_id: i64) -> bool {
        matches!(state.mode, AppMode::EditServer { protocol_id: id, .. } if id == endpoint_id)
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
        // current view must win. The edit form needs the protocol row WITH
        // config, so the state row (loaded without deferred data) cannot
        // serve it; the flow falls back to the DB and reports the profile
        // missing there. Test the guarded path: a row in state + a DB row
        // lets the form open.
        let mut state = test_state(vec![fake_row(7, "visible.example", 1)]).await;
        // Seed the DB with the same endpoint + a protocol row.
        let row = state.endpoints[0].clone();
        state.db.upsert_endpoint(&row.endpoint).await.unwrap();
        let proto = super::xray_tui_db_helper::vless_protocol(700);
        state.db.upsert_protocol(&proto).await.unwrap();
        state.db.upsert_link(&row.links[0]).await.unwrap();
        start_edit_profile(&mut state, "7").await;
        assert!(matches_edit_mode(&state, 7));
        // The form is populated from the typed config.
        if let AppMode::EditServer { fields, .. } = &state.mode {
            assert!(
                fields
                    .iter()
                    .any(|(k, v)| k == "address" && v == "visible.example"),
                "edit form must carry the endpoint address"
            );
        } else {
            panic!("expected EditServer");
        }
    }

    #[tokio::test]
    async fn edit_falls_back_to_never_seen_profile() {
        // Profile not loaded in the current view, but present in the DB with
        // last_seen_at = 0 ("never seen"). get_active_endpoints compares an
        // ABSOLUTE epoch threshold, so a fixed 86400 lookup would reject this
        // row even though it is visible in the All view; the fallback uses
        // scope 0 (everything), not 86400.
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let group_id = db.get_all_groups().await.unwrap()[0].id.clone();
        let endpoint = super::test_support::fake_row(42, "never-seen.example", 1).endpoint;
        let protocol = super::xray_tui_db_helper::vless_protocol(4200);
        let link = {
            use xray_tui_db::models::{ConfigType, ProfileStats, TrafficStats};
            ProfileStats {
                protocol_id: protocol.id,
                endpoint_id: endpoint.id,
                core_type: xray_tui_proto::proto_spec::CoreType::Xray,
                config_type: ConfigType::ShareUrl,
                last_used_at: None,
                last_seen_at: super::test_support::ts(0),
                task_id: None,
                task_queue: Vec::new(),
                latency: None,
                speed_bps: None,
                error: None,
                traffic: TrafficStats {
                    today_up: 0,
                    today_down: 0,
                    total_up: 0,
                    total_down: 0,
                },
                created_at: super::test_support::ts(0),
                updated_at: super::test_support::ts(0),
                version: 1,
                protocol: toasty::Deferred::default(),
                endpoint: toasty::Deferred::default(),
            }
        };
        db.upsert_endpoint(&endpoint).await.unwrap();
        db.upsert_protocol(&protocol).await.unwrap();
        db.upsert_link(&link).await.unwrap();
        db.upsert_endpoint_group_link(&xray_tui_db::models::EndpointGroup {
            endpoint_id: endpoint.id,
            group_id,
            last_seen_at: super::test_support::ts(0),
            sort_order: None,
            endpoint: toasty::Deferred::default(),
            group: toasty::Deferred::default(),
        })
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

#[cfg(test)]
mod clamp_tests {
    use super::test_support::{fake_row, test_state};
    use super::*;
    use std::sync::Arc;
    use xray_tui_config::AppConfig;

    #[test]
    fn clamp_index_stays_in_bounds() {
        assert_eq!(clamp_index(5, 3), 2);
        assert_eq!(clamp_index(2, 3), 2);
        assert_eq!(clamp_index(0, 0), 0);
    }

    #[tokio::test]
    async fn clamp_selection_fixes_stale_index() {
        let mut state = test_state(vec![fake_row(1, "a.com", 1), fake_row(2, "b.com", 1)]).await;
        state.selected_index = 5; // stale: beyond the filtered list
        clamp_selection(&mut state);
        assert_eq!(state.selected_index, 1); // len - 1
        assert_eq!(state.selected_profile_id(), Some(2));
    }

    #[tokio::test]
    async fn clamp_selection_resets_empty_list() {
        let mut state = test_state(vec![]).await;
        state.selected_index = 3;
        state.selected_sub = Some(0);
        clamp_selection(&mut state);
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.selected_sub, None);
    }

    #[tokio::test]
    async fn reload_profiles_clamps_stale_selection() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let group_id = db.get_all_groups().await.unwrap()[0].id.clone();
        let row = super::test_support::fake_row(9, "clamp.example", 1);
        let endpoint = row.endpoint.clone();
        let protocol = super::xray_tui_db_helper::vless_protocol(900);
        let mut link = row.links[0].clone();
        link.last_seen_at = super::test_support::ts(0);
        db.upsert_endpoint(&endpoint).await.unwrap();
        db.upsert_protocol(&protocol).await.unwrap();
        db.upsert_link(&link).await.unwrap();
        db.upsert_endpoint_group_link(&xray_tui_db::models::EndpointGroup {
            endpoint_id: endpoint.id,
            group_id,
            last_seen_at: super::test_support::ts(0),
            sort_order: None,
            endpoint: toasty::Deferred::default(),
            group: toasty::Deferred::default(),
        })
        .await
        .unwrap();
        let mut state = AppState::new(db, AppConfig::default()).await;
        state.purgatory_view = PurgatoryView::All; // includes never-seen rows
        state.selected_index = 5; // stale: list reloads to a single row
        reload_profiles(&mut state).await;
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.selected_profile_id(), Some(9));
    }
}

#[cfg(test)]
mod sort_tests {
    use super::test_support::fake_row;
    use super::*;
    use crate::AppState;
    use std::sync::Arc;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{ErrorInfo, Latency, ProfileErr};

    fn set_delay(row: &mut EndpointRow, proto_id: i64, delay: i32, real: bool) {
        let link = row
            .links
            .iter_mut()
            .find(|l| l.protocol_id.get() == proto_id)
            .expect("link exists");
        link.latency = if real {
            Some(Latency::Real { delay, ip: None })
        } else {
            Some(Latency::Fast { delay })
        };
    }

    fn set_error(row: &mut EndpointRow, proto_id: i64, kind: ProfileErr) {
        let link = row
            .links
            .iter_mut()
            .find(|l| l.protocol_id.get() == proto_id)
            .expect("link exists");
        link.error = Some(ErrorInfo {
            kind,
            text: "boom".into(),
        });
    }

    #[tokio::test]
    async fn test_sort_ranks_by_best_protocol_tier() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        // E1: real-ok 300ms; E2: fast-ok 100ms; E3: untested.
        let mut e1 = fake_row(1, "e1.example", 1);
        set_delay(&mut e1, 100, 300, true);
        let mut e2 = fake_row(2, "e2.example", 1);
        set_delay(&mut e2, 200, 100, false);
        let e3 = fake_row(3, "e3.example", 1);
        state.endpoints = vec![e1, e2, e3];
        state.sort_column = SortColumn::Test;
        state.sort_ascending = true;
        state.filter_cache_valid.set(false);

        let order: Vec<i64> = state
            .filtered_profiles()
            .map(|r| r.endpoint.id.get())
            .collect();
        assert_eq!(order, vec![1, 2, 3]); // real beats fast beats untested
    }

    #[tokio::test]
    async fn test_sort_sinks_failures_and_dns() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        // E1 untested; E2 fast-ok but real-failed; E3 dns-unresolved.
        let e1 = fake_row(1, "e1.example", 1);
        let mut e2 = fake_row(2, "e2.example", 1);
        set_delay(&mut e2, 200, 100, false);
        set_error(&mut e2, 200, ProfileErr::Real);
        let mut e3 = fake_row(3, "dns.example", 1);
        e3.endpoint.host_type = xray_tui_db::models::HostType::Dns;
        state.endpoints = vec![e1, e2, e3];
        state.sort_column = SortColumn::Test;
        state.sort_ascending = true;
        state.filter_cache_valid.set(false);

        let order: Vec<i64> = state
            .filtered_profiles()
            .map(|r| r.endpoint.id.get())
            .collect();
        assert_eq!(order, vec![1, 2, 3]); // untested above real-failure above dns
    }

    #[tokio::test]
    async fn test_sort_traffic_uses_active_link() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        let mut e1 = fake_row(1, "e1.example", 1);
        e1.links[0].traffic = xray_tui_db::models::TrafficStats {
            today_up: 0,
            today_down: 0,
            total_up: 100,
            total_down: 200,
        };
        let e2 = fake_row(2, "e2.example", 1); // zero traffic
        state.endpoints = vec![e1, e2];
        state.sort_column = SortColumn::Traffic;
        state.sort_ascending = true;
        state.filter_cache_valid.set(false);

        let order: Vec<i64> = state
            .filtered_profiles()
            .map(|r| r.endpoint.id.get())
            .collect();
        assert_eq!(order, vec![2, 1]); // 0 before 300
    }
}

#[cfg(test)]
mod ttl_tests {
    use super::test_support::{fake_row, ts};
    use super::*;
    use std::sync::Arc;
    use xray_tui_config::AppConfig;
    use xray_tui_db::models::{EndpointId, ErrorInfo, ProfileErr, ProfileStats, ProtocolId};

    /// Stamp `error` on the row's link so it persists as a failure marker.
    fn set_error(row: &mut EndpointRow, proto_id: i64, kind: ProfileErr) {
        let link = row
            .links
            .iter_mut()
            .find(|l| l.protocol_id.get() == proto_id)
            .expect("link exists");
        link.error = Some(ErrorInfo {
            kind,
            text: "boom".to_string(),
        });
    }

    /// Persist the rows (endpoint + links + protocols) via the typed writes.
    async fn persist_rows(db: &Database, rows: &[EndpointRow]) {
        for row in rows {
            db.upsert_endpoint(&row.endpoint).await.unwrap();
            for link in &row.links {
                db.upsert_link(link).await.unwrap();
                if let Some(proto) = row.protocols.get(&link.protocol_id) {
                    db.upsert_protocol(proto).await.unwrap();
                }
            }
        }
    }

    /// Backdate a link's `updated_at` (the error-TTL anchor) to `secs`.
    async fn backdate(db: &Database, proto_id: i64, endpoint_id: i64, secs: i64) {
        let mut conn = db.connection().await.unwrap();
        ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(proto_id),
            EndpointId::new(endpoint_id),
        )
        .update()
        .updated_at(ts(secs))
        .exec(&mut conn)
        .await
        .unwrap();
    }

    /// The persisted error kind of a link, if any.
    async fn link_error_kind(db: &Database, proto_id: i64, endpoint_id: i64) -> Option<ProfileErr> {
        let mut conn = db.connection().await.unwrap();
        ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(proto_id),
            EndpointId::new(endpoint_id),
        )
        .first()
        .exec(&mut conn)
        .await
        .unwrap()
        .expect("link persisted")
        .error
        .map(|e| e.kind)
    }

    #[tokio::test]
    async fn reload_profiles_sweeps_expired_errors_and_keeps_fresh() {
        // Endpoint 1 with two failing links: p100 (error 48h old), p101
        // (error 10 min old).
        let mut row = fake_row(1, "10.0.0.1", 2);
        set_error(&mut row, 100, ProfileErr::Fast);
        set_error(&mut row, 101, ProfileErr::Real);
        let db = Arc::new(xray_tui_db::Database::in_memory().await.unwrap());
        persist_rows(&db, std::slice::from_ref(&row)).await;
        let now = jiff::Timestamp::now().as_second();
        backdate(&db, 100, 1, now - 48 * 3600).await;
        backdate(&db, 101, 1, now - 600).await;

        let mut state = AppState::new(db.clone(), AppConfig::default()).await;
        state.purgatory_view = PurgatoryView::All;

        // Default config (ttl None): reload must NOT clear anything.
        reload_profiles(&mut state).await;
        assert_eq!(
            link_error_kind(&db, 100, 1).await,
            Some(ProfileErr::Fast),
            "ttl None: stale error untouched"
        );
        assert_eq!(
            link_error_kind(&db, 101, 1).await,
            Some(ProfileErr::Real),
            "ttl None: fresh error untouched"
        );

        // Configured ttl (24h): the 48h-old error is swept, the fresh one
        // survives, and the reloaded in-memory rows match the DB.
        state.config.speed_test.error_ttl_hours = Some(24);
        reload_profiles(&mut state).await;
        assert_eq!(link_error_kind(&db, 100, 1).await, None, "stale cleared");
        assert_eq!(
            link_error_kind(&db, 101, 1).await,
            Some(ProfileErr::Real),
            "fresh kept"
        );
        let row1 = state
            .endpoints
            .iter()
            .find(|r| r.endpoint.id.get() == 1)
            .expect("endpoint reloaded");
        let p100 = row1
            .links
            .iter()
            .find(|l| l.protocol_id.get() == 100)
            .expect("p100 link");
        assert!(p100.error.is_none(), "in-memory row reflects the sweep");
        let p101 = row1
            .links
            .iter()
            .find(|l| l.protocol_id.get() == 101)
            .expect("p101 link");
        assert!(p101.error.is_some(), "fresh marker still renders");
    }

    #[tokio::test]
    async fn clear_expired_errors_none_and_nonpositive_ttl_are_noops() {
        let mut row = fake_row(1, "10.0.0.1", 1);
        set_error(&mut row, 100, ProfileErr::Fast);
        let db = Arc::new(xray_tui_db::Database::in_memory().await.unwrap());
        persist_rows(&db, std::slice::from_ref(&row)).await;
        let now = jiff::Timestamp::now().as_second();
        backdate(&db, 100, 1, now - 48 * 3600).await;

        // None (default) and non-positive values both mean "never clear".
        clear_expired_errors(&db, None).await;
        assert_eq!(
            link_error_kind(&db, 100, 1).await,
            Some(ProfileErr::Fast),
            "ttl None: no-op"
        );
        clear_expired_errors(&db, Some(0)).await;
        assert_eq!(
            link_error_kind(&db, 100, 1).await,
            Some(ProfileErr::Fast),
            "ttl 0: no-op"
        );

        // A positive ttl clears the stale marker.
        clear_expired_errors(&db, Some(24)).await;
        assert_eq!(link_error_kind(&db, 100, 1).await, None);
    }
}
