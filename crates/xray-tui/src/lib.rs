pub mod ui;

use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use tokio::sync::mpsc;
use xray_tui_config::AppConfig;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{find_binary, resolve_core, BuildParams, CoreManager, ConfigBuilder, CoreType};
use xray_tui_db::models::{DnsSetting, Group, Profile, ProfileExtension, RoutingRule, ServerStat};
use xray_tui_db::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Routing,
    Dns,
    Logs,
    Statistics,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::Profiles,
        Tab::Settings,
        Tab::Routing,
        Tab::Dns,
        Tab::Logs,
        Tab::Statistics,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    ConfigType,
    Remarks,
    Address,
    Port,
    Delay,
    Speed,
    Traffic,
    Core,
}

pub struct ProfileRow {
    pub profile: Profile,
    pub extension: Option<ProfileExtension>,
    pub stats: Option<ServerStat>,
}

pub struct LogLine {
    pub level: String,
    pub message: String,
}
/// Tracks what the UI is currently showing.
#[derive(Debug, Clone)]
pub enum AppMode {
    /// The main profile list
    List,
    /// Adding a new server
    AddServer {
        /// Selected protocol (None while protocol picker shown)
        protocol: Option<Protocol>,
        /// Raw form field values: map of field key -> current input
        fields: Vec<(String, String)>,
        /// Index of the focused field
        focus_index: usize,
    },
    /// Editing an existing server
    EditServer {
        profile_id: String,
        fields: Vec<(String, String)>,
        focus_index: usize,
    },
    /// Import URL from paste
    ImportUrl {
        input: String,
        error: Option<String>,
    },
}


/// Events from the core process manager to the UI event loop.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    Connected(CoreType),
    Disconnected,
    Error(String),
}

pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub profiles: Vec<ProfileRow>,
    pub groups: Vec<Group>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub log_buffer: Vec<LogLine>,
    pub connected_core: Option<CoreType>,
    pub connecting: bool,
    pub connection_error: Option<String>,
    pub core_event_rx: Option<mpsc::UnboundedReceiver<CoreEvent>>,
    pub core_event_tx: Option<mpsc::UnboundedSender<CoreEvent>>,
    pub disconnect_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub multi_select: HashSet<String>,
    pub confirm_delete: Option<String>,
    pub clipboard: Option<String>,
}

impl AppState {
    pub fn new(db: Database, config: AppConfig) -> Self {
        let mut state = Self {
            db,
            config,
            current_tab: Tab::Profiles,
            profiles: Vec::new(),
            groups: Vec::new(),
            selected_group_id: None,
            selected_index: 0,
            sort_column: SortColumn::Remarks,
            sort_ascending: true,
            search_query: String::new(),
            search_focused: false,
            log_buffer: Vec::new(),
            connected_core: None,
            connecting: false,
            connection_error: None,
            core_event_rx: None,
            core_event_tx: None,
            disconnect_tx: None,
            should_quit: false,
            mode: AppMode::List,
            multi_select: HashSet::new(),
            clipboard: None,
            confirm_delete: None,
        };
        state.reload_profiles();
        state.reload_groups();
        state
    }

    pub fn reload_profiles(&mut self) {
        match self.db.get_all_profiles_with_details() {
            Ok(rows) => {
                self.profiles = rows
                    .into_iter()
                    .map(|(profile, extension, stats)| ProfileRow {
                        profile,
                        extension,
                        stats,
                    })
                    .collect();
            }
            Err(e) => {
                self.add_log("error", &format!("Failed to load profiles: {e}"));
                self.profiles.clear();
            }
        }
    }

    pub fn reload_groups(&mut self) {
        match self.db.get_all_groups() {
            Ok(groups) => self.groups = groups,
            Err(e) => {
                self.add_log("error", &format!("Failed to load groups: {e}"));
                self.groups.clear();
            }
        }
    }

    pub fn filtered_profiles(&self) -> Vec<&ProfileRow> {
        let mut filtered: Vec<&ProfileRow> = self.profiles.iter().filter(|row| {
            // Group filter
            if let Some(group_id) = &self.selected_group_id
                && row.profile.group_id.as_deref() != Some(group_id.as_str())
            {
                return false;
            }
            if !self.search_query.is_empty() {
                let q = self.search_query.to_lowercase();
                let remarks = row.profile.remarks.as_deref().unwrap_or("");
                let address = row.profile.address.as_deref().unwrap_or("");
                let port = row.profile.port.map(|p| p.to_string()).unwrap_or_default();
                if !remarks.to_lowercase().contains(&q)
                    && !address.to_lowercase().contains(&q)
                    && !port.contains(&q)
                {
                    return false;
                }
            }
            true
        }).collect();

        let asc = self.sort_ascending;
        filtered.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::ConfigType => a.profile.config_type.cmp(&b.profile.config_type),
                SortColumn::Remarks => {
                    a.profile.remarks.as_deref().unwrap_or("")
                        .cmp(b.profile.remarks.as_deref().unwrap_or(""))
                }
                SortColumn::Address => {
                    a.profile.address.as_deref().unwrap_or("")
                        .cmp(b.profile.address.as_deref().unwrap_or(""))
                }
                SortColumn::Port => {
                    let pa = a.profile.port.unwrap_or(0);
                    let pb = b.profile.port.unwrap_or(0);
                    pa.cmp(&pb)
                }
                SortColumn::Delay => {
                    let da = a.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    let db = b.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    da.cmp(&db)
                }
                SortColumn::Speed => {
                    let sa = a.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    let sb = b.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    sa.cmp(&sb)
                }
                SortColumn::Traffic => {
                    let ta = a.stats.as_ref().map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0)).unwrap_or(0);
                    let tb = b.stats.as_ref().map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0)).unwrap_or(0);
                    ta.cmp(&tb)
                }
                SortColumn::Core => {
                    let resolve = |row: &&ProfileRow| -> String {
                        let protocol = Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
                        let core = resolve_core(protocol, Some(
                            CoreType::from_str(&row.profile.core_type).unwrap_or(CoreType::Auto),
                        ));
                        core.to_string()
                    };
                    resolve(a).cmp(&resolve(b))
                }
            };
            if asc { cmp } else { cmp.reverse() }
        });
        filtered
    }

    pub fn add_log(&mut self, level: &str, message: &str) {
        self.log_buffer.push(LogLine {
            level: level.to_owned(),
            message: message.to_owned(),
        });
        if self.log_buffer.len() > 1000 {
            self.log_buffer.remove(0);
        }
    }
    // ── CRUD operations ──────────────────────────────────────────────────

    pub fn start_add_server(&mut self) {
        let fields = common_field_defaults();
        self.mode = AppMode::AddServer {
            protocol: None,
            fields,
            focus_index: 0,
        };
    }

    pub fn start_edit_profile(&mut self, id: &str) {
        match self.db.get_profile(id) {
            Ok(Some(profile)) => {
                let fields = profile_to_fields(&profile);
                self.mode = AppMode::EditServer {
                    profile_id: id.to_string(),
                    fields,
                    focus_index: 0,
                };
            }
            Ok(None) => self.add_log("error", &format!("Profile {id} not found")),
            Err(e) => self.add_log("error", &format!("Failed to load profile: {e}")),
        }
    }

    #[allow(dead_code)]
    fn selected_profile(&self) -> Option<&Profile> {
        let filtered = self.filtered_profiles();
        filtered.get(self.selected_index).map(|r| &r.profile)
    }

    fn selected_profile_id(&self) -> Option<String> {
        let filtered = self.filtered_profiles();
        filtered.get(self.selected_index).map(|r| r.profile.id.clone())
    }

    fn fields_to_profile(&self, protocol: Protocol, fields: &[(String, String)]) -> Profile {
        let id = uuid::Uuid::new_v4().to_string();
        let now = iso_now();
        let mut profile = Profile {
            id,
            config_type: protocol.to_i32(),
            core_type: "auto".into(),
            remarks: None,
            address: None,
            port: None,
            user_id: None,
            security: None,
            network: None,
            stream_settings: None,
            protocol_settings: None,
            is_sub: Some(0),
            sub_id: None,
            group_id: None,
            sort_order: None,
            is_active: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };
        let mut stream_map = serde_json::Map::new();
        let mut proto_map = serde_json::Map::new();

        for (key, value) in fields {
            if value.is_empty() {
                continue;
            }
            match key.as_str() {
                "remarks" => profile.remarks = Some(value.clone()),
                "address" => profile.address = Some(value.clone()),
                "port" => profile.port = value.parse::<i32>().ok(),
                "core_type" => profile.core_type = value.clone(),
                "user_id" | "password" | "uuid" => profile.user_id = Some(value.clone()),
                "security" => profile.security = Some(value.clone()),
                "network" => profile.network = Some(value.clone()),
                // Stream settings (dot-separated keys)
                _ if key.starts_with("tls.") || key.starts_with("ws.") || key.starts_with("grpc.")
                    || key.starts_with("reality.") || key.starts_with("tcp.")
                    || *key == "sni" || *key == "alpn" || *key == "fingerprint"
                    || *key == "allow_insecure" => {
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
                // Protocol settings fallback
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

        if !stream_map.is_empty() {
            profile.stream_settings = Some(serde_json::to_string(&stream_map).unwrap_or_default());
        }
        if !proto_map.is_empty() {
            profile.protocol_settings = Some(serde_json::to_string(&proto_map).unwrap_or_default());
        }
        profile
    }

    pub fn confirm_add_server(&mut self) {
        let (protocol, fields) = match &self.mode {
            AppMode::AddServer { protocol: Some(p), fields, .. } => (*p, fields.clone()),
            _ => {
                self.add_log("error", "Cannot confirm: no protocol selected");
                return;
            }
        };
        let profile = self.fields_to_profile(protocol, &fields);
        if let Err(e) = self.db.insert_profile(&profile) {
            self.add_log("error", &format!("Failed to add server: {e}"));
            return;
        }
        self.add_log("info", &format!("Added server: {}", profile.remarks.as_deref().unwrap_or("unnamed")));
        self.mode = AppMode::List;
        self.reload_profiles();
    }

    pub fn confirm_edit_server(&mut self) {
        let (profile_id, fields) = match &self.mode {
            AppMode::EditServer { profile_id, fields, .. } => (profile_id.clone(), fields.clone()),
            _ => return,
        };
        let mut profile = match self.db.get_profile(&profile_id) {
            Ok(Some(p)) => p,
            _ => {
                self.add_log("error", "Profile not found for edit");
                return;
            }
        };
        // Infer protocol from existing profile
        let protocol = Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
        // Rebuild from form fields
        let new_profile = self.fields_to_profile(protocol, &fields);
        profile.remarks = new_profile.remarks;
        profile.address = new_profile.address;
        profile.port = new_profile.port;
        profile.core_type = new_profile.core_type;
        profile.user_id = new_profile.user_id;
        profile.security = new_profile.security;
        profile.network = new_profile.network;
        profile.stream_settings = new_profile.stream_settings;
        profile.protocol_settings = new_profile.protocol_settings;
        profile.updated_at = Some(iso_now());

        if let Err(e) = self.db.update_profile(&profile) {
            self.add_log("error", &format!("Failed to update server: {e}"));
            return;
        }
        self.add_log("info", "Server updated");
        self.mode = AppMode::List;
        self.reload_profiles();
    }

    pub fn cancel_form(&mut self) {
        self.mode = AppMode::List;
    }

    pub fn delete_profile(&mut self, id: &str) {
        if let Err(e) = self.db.delete_profile(id) {
            self.add_log("error", &format!("Failed to delete profile: {e}"));
            return;
        }
        self.add_log("info", "Profile deleted");
        self.confirm_delete = None;
        self.multi_select.remove(id);
        self.reload_profiles();
    }

    pub fn clone_profile(&mut self, id: &str) {
        let new_id = uuid::Uuid::new_v4().to_string();
        if let Err(e) = self.db.clone_profile(id, &new_id) {
            self.add_log("error", &format!("Failed to clone profile: {e}"));
            return;
        }
        self.add_log("info", "Profile cloned");
        self.reload_profiles();
    }

    pub fn toggle_multi_select(&mut self, id: &str) {
        if !self.multi_select.insert(id.to_string()) {
            self.multi_select.remove(id);
        }
    }

    pub fn import_url(&mut self, url: &str) {
        match xray_tui_config::import_export::parse_share_url(url) {
            Ok(profile) => {
                let protocol = Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
                let fields = profile_to_fields(&profile);
                self.mode = AppMode::AddServer {
                    protocol: Some(protocol),
                    fields,
                    focus_index: 0,
                };
                self.add_log("info", "URL imported successfully");
            }
            Err(e) => {
                self.mode = AppMode::ImportUrl {
                    input: url.to_string(),
                    error: Some(e.to_string()),
                };
            }
        }
    }

    pub fn move_profile_up(&mut self) {
        let id = match self.selected_profile_id() {
            Some(id) => id,
            None => return,
        };
        let filtered = self.filtered_profiles();
        let idx = filtered.iter().position(|r| r.profile.id == id);
        let idx = match idx {
            Some(i) if i > 0 => i,
            _ => return,
        };
        let prev_id = &filtered[idx - 1].profile.id;
        let a_order = filtered[idx].profile.sort_order.unwrap_or(0);
        let b_order = filtered[idx - 1].profile.sort_order.unwrap_or(0);
        if let Err(e) = self.db.reorder_profiles(&[(id.clone(), b_order), (prev_id.clone(), a_order)]) {
            self.add_log("error", &format!("Failed to reorder: {e}"));
        }
        self.reload_profiles();
    }

    pub fn move_profile_down(&mut self) {
        let id = match self.selected_profile_id() {
            Some(id) => id,
            None => return,
        };
        let filtered = self.filtered_profiles();
        let idx = match filtered.iter().position(|r| r.profile.id == id) {
            Some(i) if i < filtered.len() - 1 => i,
            _ => return,
        };
        let next_id = &filtered[idx + 1].profile.id;
        let a_order = filtered[idx].profile.sort_order.unwrap_or(0);
        let b_order = filtered[idx + 1].profile.sort_order.unwrap_or(0);
        if let Err(e) = self.db.reorder_profiles(&[(id.clone(), b_order), (next_id.clone(), a_order)]) {
            self.add_log("error", &format!("Failed to reorder: {e}"));
        }
        self.reload_profiles();
    }

    pub fn set_active(&mut self, id: &str) {
        if let Err(e) = self.db.update_profile_active(id) {
            self.add_log("error", &format!("Failed to set active: {e}"));
            return;
        }
        self.reload_profiles();
    }

    // ── Core connection management ──────────────────────────────────────

    /// Connect to the selected profile by id.
    pub fn connect_to_profile(&mut self, profile_id: &str) {
        if self.connecting {
            return;
        }

        let profile = match self.profiles.iter().find(|r| r.profile.id == profile_id) {
            Some(r) => r.profile.clone(),
            None => {
                self.add_log("error", "Profile not found for connection");
                return;
            }
        };

        let protocol = match Protocol::try_from_i32(profile.config_type) {
            Some(p) => p,
            None => {
                self.add_log("error", &format!("Unknown protocol: {}", profile.config_type));
                return;
            }
        };

        let profile_override = profile.core_type.parse::<CoreType>().ok();
        let core_type = resolve_core(protocol, profile_override);

        // If already connected/disconnecting, send stop signal first
        if let Some(tx) = self.disconnect_tx.take() {
            let _ = tx.send(());
        }

        // Create disconnect signal channel
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        self.disconnect_tx = Some(stop_tx);
        self.connecting = true;
        self.connection_error = None;

        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => {
                self.connecting = false;
                self.add_log("error", "Core event channel not initialized");
                return;
            }
        };

        let params = BuildParams {
            log_level: self.config.core.log_level.clone(),
            socks_port: self.config.inbound.socks_port,
            http_port: self.config.inbound.http_port,
            listen: self.config.inbound.listen.clone(),
            sniffing: self.config.inbound.sniffing,
        };

        // Default DNS and routing for first pass
        let dns = DnsSetting {
            id: "default".to_string(),
            name: None,
            servers: None,
            hosts: None,
            query_strategy: None,
            disable_cache: None,
            disable_fallback: None,
            client_ip: None,
        };
        let routing: Vec<RoutingRule> = vec![];

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join("xray-tui");
        let bin_dir = config_dir.join("bin").join(core_type.to_string());
        let bin_configs_dir = config_dir.join("binConfigs");

        // Need to move params into the async block

        tokio::spawn(async move {
            // 1. Build config
            let backend_config = match ConfigBuilder::build(
                &profile, core_type, &params, &routing, &dns,
            ) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(CoreEvent::Error(format!("Config build failed: {e}")));
                    return;
                }
            };

            // 2. Find binary
            let bin_path = match find_binary(core_type, &bin_dir) {
                Some(p) => p,
                None => {
                    let _ = tx.send(CoreEvent::Error(
                        "Core binary not found. Place it in ~/.config/xray-tui/bin/ or install in PATH."
                            .to_string(),
                    ));
                    return;
                }
            };

            // 3. Start core
            let mut manager = CoreManager::new(bin_configs_dir);
            if let Err(e) = manager.start(core_type, &backend_config, &bin_path).await {
                let _ = tx.send(CoreEvent::Error(format!("Failed to start core: {e}")));
                return;
            }

            // 4. Signal connected
            let _ = tx.send(CoreEvent::Connected(core_type));

            // 5. Wait for stop signal
            let _ = stop_rx.await;

            // 6. Stop core
            let _ = manager.stop().await;

            // 7. Signal disconnected
            let _ = tx.send(CoreEvent::Disconnected);
        });
    }

    /// Disconnect the currently running core.
    pub fn disconnect(&mut self) {
        if let Some(tx) = self.disconnect_tx.take() {
            let _ = tx.send(());
        }
        self.connected_core = None;
        self.connecting = false;
        self.add_log("info", "Disconnected");
    }

    /// Poll core event channel and update state accordingly.
    pub fn poll_core_events(&mut self) {
        while let Some(rx) = self.core_event_rx.as_mut() {
            let event = match rx.try_recv() {
                Ok(event) => event,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            };
            match event {
                CoreEvent::Connected(core_type) => {
                    self.connected_core = Some(core_type);
                    self.connecting = false;
                    self.connection_error = None;
                    self.add_log("info", &format!("Connected [{core_type}]"));
                }
                CoreEvent::Disconnected => {
                    // Ignore stale Disconnected if already reconnecting
                    if !self.connecting {
                        self.connected_core = None;
                        self.add_log("info", "Core process stopped");
                    }
                }
                CoreEvent::Error(err) => {
                    self.connection_error = Some(err.clone());
                    self.connecting = false;
                    self.connected_core = None;
                    self.add_log("error", &format!("Connection error: {err}"));
                }
            }
        }
    }
}

fn common_field_defaults() -> Vec<(String, String)> {
    vec![
        ("remarks".to_string(), String::new()),
        ("address".to_string(), String::new()),
        ("port".to_string(), "443".to_string()),
        ("core_type".to_string(), "auto".to_string()),
    ]
}

fn profile_to_fields(profile: &Profile) -> Vec<(String, String)> {
    let mut fields = common_field_defaults();
    if let Some(v) = &profile.remarks {
        set_field(&mut fields, "remarks", v);
    }
    if let Some(v) = &profile.address {
        set_field(&mut fields, "address", v);
    }
    if let Some(v) = profile.port {
        set_field(&mut fields, "port", &v.to_string());
    }
    set_field(&mut fields, "core_type", &profile.core_type);
    if let Some(v) = &profile.user_id {
        set_field(&mut fields, "user_id", v);
    }
    if let Some(v) = &profile.security {
        set_field(&mut fields, "security", v);
    }
    if let Some(v) = &profile.network {
        set_field(&mut fields, "network", v);
    }

    // Flatten stream_settings into dotted keys
    if let Some(ss) = &profile.stream_settings
        && let Ok(obj) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ss) {
            for (k, v) in obj {
                let val = match &v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => continue,
                };
                fields.push((k, val));
            }
        }

    // Flatten protocol_settings
    if let Some(ps) = &profile.protocol_settings
        && let Ok(obj) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ps) {
            for (k, v) in obj {
                let val = match &v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => continue,
                };
                // Skip duplicates already set explicitly
                if !fields.iter().any(|(fk, _)| fk == &k) {
                    fields.push((k, val));
                }
            }
        }

    fields
}

fn set_field(fields: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = fields.iter_mut().find(|(k, _)| k == key) {
        *existing = value.to_string();
    } else {
        fields.push((key.to_string(), value.to_string()));
    }
}

fn iso_now() -> String {
    // Simple ISO 8601 without chrono dependency
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Format as YYYY-MM-DDTHH:MM:SSZ
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Simple date calculation from Unix epoch (1970-01-01)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
