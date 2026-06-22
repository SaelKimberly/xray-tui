pub mod ui;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use xray_tui_config::AppConfig;
use xray_tui_core::grpc_client;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{BuildParams, CLASH_API_PORT, ConfigBuilder, CoreManager, CoreType, find_binary, resolve_core};
use xray_tui_db::Database;
use xray_tui_db::models::{
    ALL_GROUP_ID, DnsSetting, GRAVEYARD_GROUP_ID, Group, Profile, ProfileExtension, RoutingRule, ServerStat,
    Subscription,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Logs,
    Statistics,
}

impl Tab {
    pub const ALL: &[Tab] = &[Tab::Profiles, Tab::Settings, Tab::Logs, Tab::Statistics];
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

/// Sub-modes for the Settings panel.
#[derive(Debug, Clone)]
pub enum SettingsMode {
    Menu { selected: usize },
    CoreForm { fields: Vec<(String, String)>, focus_index: usize },
    GuiForm { fields: Vec<(String, String)>, focus_index: usize },
    InboundForm { fields: Vec<(String, String)>, focus_index: usize },
    RoutingList { selected: usize },
    RoutingForm { rule_id: Option<String>, fields: Vec<(String, String)>, focus_index: usize },
    DnsForm { fields: Vec<(String, String)>, focus_index: usize },
    SystemProxyForm { fields: Vec<(String, String)>, focus_index: usize },
    TunForm { fields: Vec<(String, String)>, focus_index: usize },
    MuxForm { fields: Vec<(String, String)>, focus_index: usize },
    StatsForm { fields: Vec<(String, String)>, focus_index: usize },
    UpdateForm { status_xray: BackendUpdateStatus, status_singbox: BackendUpdateStatus },
}

/// Identifies which section of the Settings panel is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Core,
    Gui,
    Inbound,
    Routing,
    Dns,
    SystemProxy,
    Tun,
    Mux,
    Stats,
    Updates,
}
/// Tracks what the UI is currently showing.
#[derive(Debug, Clone)]
pub enum AppMode {
    /// The main profile list
    List,
    /// Help overlay
    Help,
    Settings { mode: SettingsMode },
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
    /// Managing subscription groups
    ManageGroups { selected: usize },
    /// Adding a new subscription group
    AddGroup {
        fields: Vec<(String, String)>,
        focus_index: usize,
    },
    /// Editing an existing subscription group
    EditGroup {
        group_id: String,
        fields: Vec<(String, String)>,
        focus_index: usize,
    },
    /// Speed test menu overlay
    SpeedTestMenu { selected: usize },
    /// Batch import multiple share URLs
    BatchImport {
        /// Parsed/split profiles for each URL
        results: Vec<BatchImportItem>,
        /// Current scroll position
        scroll: usize,
    },
}

/// A single item in a batch import list.
#[derive(Debug, Clone)]
pub struct BatchImportItem {
    pub url: String,
    pub profile: Option<Profile>,
    pub error: Option<String>,
    pub imported: bool,
}

/// Parsed line from sing-box Clash API `/traffic` streaming endpoint.
#[derive(serde::Deserialize)]
struct ClashTraffic {
    up: i64,
    down: i64,
}

/// Events from the core process manager to the UI event loop.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    Connected(CoreType),
    Disconnected,
    Error(String),
    /// Non-fatal stats error — keeps connected_core intact
    StatsError(String),
    StatsUpdate {
        profile_id: String,
        today_up: i64,
        today_down: i64,
        total_up: i64,
        total_down: i64,
    },
    SysStatsUpdate(grpc_client::SysStats),
    SubscriptionsUpdated {
        group_id: String,
        count: usize,
        error: Option<String>,
    },
    /// Result from a speed test operation
    SpeedTestResult {
        profile_id: String,
        test_type: TestType,
        latency_ms: Option<u64>,
        speed_bps: Option<u64>,
        error: Option<String>,
    },
    /// Result of a version check for a proxy backend.
    UpdateCheckResult {
        core_type: CoreType,
        current_version: Option<String>,
        latest_version: Option<String>,
        error: Option<String>,
    },
    /// Result of a download+install operation.
    UpdateCompleted {
        core_type: CoreType,
        old_version: Option<String>,
        new_version: String,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteProfile(String),
    DeleteGroup(String),
    ClearGroup(String),
}
#[derive(Debug, Clone, Default)]
pub struct BackendUpdateStatus {
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub downloading: bool,
    pub download_progress: Option<(u64, u64)>,
    pub error: Option<String>,
}


pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub profiles: Vec<ProfileRow>,
    /// Cached filtered/sorted profile indices for performance.
    pub cached_filtered_indices: RefCell<Vec<usize>>,
    pub filter_cache_valid: Cell<bool>,
    pub groups: Vec<Group>,
    pub subscriptions: Vec<Subscription>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub log_buffer: Vec<LogLine>,
    pub connected_core: Option<CoreType>,
    pub connecting: bool,
    pub system_stats: Option<grpc_client::SysStats>,
    pub connection_error: Option<String>,
    pub core_event_rx: Option<mpsc::UnboundedReceiver<CoreEvent>>,
    pub core_event_tx: Option<mpsc::UnboundedSender<CoreEvent>>,
    pub disconnect_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub should_quit: bool,
    pub mode: AppMode,
    /// Previous mode before opening help overlay
    pub previous_mode: Option<Box<AppMode>>,
    pub multi_select: HashSet<String>,
    pub clipboard: Option<String>,
    pub confirmation: Option<ConfirmAction>,
    pub updating_groups: HashSet<String>,
    /// Profile IDs currently being tested
    pub testing_profiles: HashSet<String>,
    /// Progress for batch tests: (completed, total)
    pub test_progress: Option<(usize, usize)>,
    /// Cached update status for both backends.
    pub update_status: HashMap<CoreType, BackendUpdateStatus>,
}

/// Internal helper for batch ping deduplication.
struct UniqueTarget {
    key: (String, u16),
    profile_ids: Vec<String>,
}

impl AppState {
    pub fn new(db: Database, config: AppConfig) -> Self {
        let mut state = Self {
            db,
            config,
            current_tab: Tab::Profiles,
            update_status: HashMap::new(),
            profiles: Vec::new(),
            cached_filtered_indices: RefCell::new(Vec::new()),
            filter_cache_valid: Cell::new(true),
            groups: Vec::new(),
            subscriptions: Vec::new(),
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
            previous_mode: None,
            multi_select: HashSet::new(),
            clipboard: None,
            confirmation: None,
            updating_groups: HashSet::new(),
            testing_profiles: HashSet::new(),
            test_progress: None,
            system_stats: None,
        };
        state.reload_profiles();
        state.reload_groups();
        state.subscriptions = state.db.get_all_subscriptions().unwrap_or_default();
        state.spawn_auto_update();
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
        self.filter_cache_valid.set(false);
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
        if !self.filter_cache_valid.get() {
            let indices = self.compute_filtered_indices();
            *self.cached_filtered_indices.borrow_mut() = indices;
            self.filter_cache_valid.set(true);
        }
        let indices = self.cached_filtered_indices.borrow();
        indices.iter().map(|&i| &self.profiles[i]).collect()
    }

    fn compute_filtered_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .profiles
            .iter()
            .enumerate()
            .filter(|(_, row)| {
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
            })
            .map(|(i, _)| i)
            .collect();

        let asc = self.sort_ascending;
        indices.sort_by(|&a, &b| {
            let a_row = &self.profiles[a];
            let b_row = &self.profiles[b];
            let cmp = match self.sort_column {
                SortColumn::ConfigType => {
                    a_row.profile.config_type.cmp(&b_row.profile.config_type)
                }
                SortColumn::Remarks => a_row
                    .profile
                    .remarks
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b_row.profile.remarks.as_deref().unwrap_or("")),
                SortColumn::Address => a_row
                    .profile
                    .address
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b_row.profile.address.as_deref().unwrap_or("")),
                SortColumn::Port => {
                    let pa = a_row.profile.port.unwrap_or(0);
                    let pb = b_row.profile.port.unwrap_or(0);
                    pa.cmp(&pb)
                }
                SortColumn::Delay => {
                    let da = a_row.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    let db = b_row.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    da.cmp(&db)
                }
                SortColumn::Speed => {
                    let sa = a_row.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    let sb = b_row.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    sa.cmp(&sb)
                }
                SortColumn::Traffic => {
                    let ta = a_row
                        .stats
                        .as_ref()
                        .map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0))
                        .unwrap_or(0);
                    let tb = b_row
                        .stats
                        .as_ref()
                        .map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0))
                        .unwrap_or(0);
                    ta.cmp(&tb)
                }
                SortColumn::Core => {
                    let resolve = |row: &ProfileRow| -> String {
                        let protocol = Protocol::try_from_i32(row.profile.config_type)
                            .unwrap_or(Protocol::Custom);
                        let core = resolve_core(
                            protocol,
                            Some(
                                CoreType::from_str(&row.profile.core_type)
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

    pub fn filtered_len(&self) -> usize {
        if !self.filter_cache_valid.get() {
            let indices = self.compute_filtered_indices();
            *self.cached_filtered_indices.borrow_mut() = indices;
            self.filter_cache_valid.set(true);
        }
        self.cached_filtered_indices.borrow().len()
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
        filtered
            .get(self.selected_index)
            .map(|r| r.profile.id.clone())
    }

    fn fields_to_profile(&self, protocol: Protocol, fields: &[(String, String)]) -> Profile {
        let id = uuid::Uuid::new_v4().to_string();
        let now = format_now();
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
            sub_uid: None,
        };
        let mut stream_map = serde_json::Map::new();
        let mut proto_map = serde_json::Map::new();

        for (key, value) in fields {
            if value.is_empty() {
                continue;
            }
            match key.as_str() {
                "remarks" => profile.remarks = Some(xray_tui_config::import_export::normalize_remark(value)),
                "address" => profile.address = Some(value.clone()),
                "port" => profile.port = value.parse::<i32>().ok(),
                "core_type" => profile.core_type = value.clone(),
                "user_id" | "password" | "uuid" => profile.user_id = Some(value.clone()),
                "security" => profile.security = Some(value.clone()),
                "network" => profile.network = Some(value.clone()),
                // Stream settings (dot-separated keys)
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
        profile.sub_uid = Some(profile.compute_sub_uid() as i64);
        profile
    }

    pub fn confirm_add_server(&mut self) {
        let (protocol, fields) = match &self.mode {
            AppMode::AddServer {
                protocol: Some(p),
                fields,
                ..
            } => (*p, fields.clone()),
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
        self.add_log(
            "info",
            &format!(
                "Added server: {}",
                profile.remarks.as_deref().unwrap_or("unnamed")
            ),
        );
        self.mode = AppMode::List;
        self.reload_profiles();
    }

    pub fn confirm_edit_server(&mut self) {
        let (profile_id, fields) = match &self.mode {
            AppMode::EditServer {
                profile_id, fields, ..
            } => (profile_id.clone(), fields.clone()),
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
        profile.sub_uid = Some(profile.compute_sub_uid() as i64);
        profile.updated_at = Some(format_now());

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

    // ── Settings helpers ──────────────────────────────────────────────────

    pub fn enter_settings(&mut self) {
        self.mode = AppMode::Settings {
            mode: SettingsMode::Menu { selected: 0 },
        };
    }

    fn build_settings_fields(&self, section: SettingsSection) -> Vec<(String, String)> {
        use crate::SettingsSection::*;
        match section {
            Core => {
                vec![
                    ("xray_path".into(), self.config.core.xray_path.clone().unwrap_or_default()),
                    ("sing_box_path".into(), self.config.core.sing_box_path.clone().unwrap_or_default()),
                    ("default_core".into(), format!("{:?}", self.config.core.core_type.unwrap_or(xray_tui_core::CoreType::Auto))),
                    ("log_level".into(), self.config.core.log_level.clone()),
                ]
            }
            Gui => {
                vec![
                    ("language".into(), self.config.gui.language.clone()),
                    ("theme".into(), self.config.gui.theme.clone().unwrap_or_default()),
                    ("refresh_interval".into(), self.config.gui.refresh_interval_secs.to_string()),
                ]
            }
            Inbound => {
                vec![
                    ("socks_port".into(), self.config.inbound.socks_port.to_string()),
                    ("http_port".into(), self.config.inbound.http_port.map(|p| p.to_string()).unwrap_or_default()),
                    ("mixed_port".into(), self.config.inbound.mixed_port.map(|p| p.to_string()).unwrap_or_default()),
                    ("listen".into(), self.config.inbound.listen.clone()),
                    ("sniffing".into(), if self.config.inbound.sniffing { "true".into() } else { "false".into() }),
                ]
            }
            Dns => {
                if let Ok(Some(dns)) = self.db.get_dns_settings() {
                    vec![
                        ("servers".into(), dns.servers.unwrap_or_default()),
                        ("hosts".into(), dns.hosts.unwrap_or_default()),
                        ("query_strategy".into(), dns.query_strategy.unwrap_or_default()),
                        ("disable_cache".into(), if dns.disable_cache.unwrap_or(0) != 0 { "true".into() } else { "false".into() }),
                        ("disable_fallback".into(), if dns.disable_fallback.unwrap_or(0) != 0 { "true".into() } else { "false".into() }),
                        ("client_ip".into(), dns.client_ip.unwrap_or_default()),
                    ]
                } else {
                    vec![
                        ("servers".into(), String::new()),
                        ("hosts".into(), String::new()),
                        ("query_strategy".into(), String::new()),
                        ("disable_cache".into(), "false".into()),
                        ("disable_fallback".into(), "false".into()),
                        ("client_ip".into(), String::new()),
                    ]
                }
            }
            SystemProxy => {
                vec![
                    ("enabled".into(), if self.config.system_proxy.enabled { "true".into() } else { "false".into() }),
                    ("http_port".into(), self.config.system_proxy.http_port.map(|p| p.to_string()).unwrap_or_default()),
                    ("socks_port".into(), self.config.system_proxy.socks_port.map(|p| p.to_string()).unwrap_or_default()),
                    ("bypass".into(), self.config.system_proxy.bypass.clone().unwrap_or_default()),
                ]
            }
            Tun => {
                vec![
                    ("enabled".into(), if self.config.tun.enabled { "true".into() } else { "false".into() }),
                    ("interface_name".into(), self.config.tun.interface_name.clone().unwrap_or_default()),
                    ("mtu".into(), self.config.tun.mtu.map(|v| v.to_string()).unwrap_or_default()),
                ]
            }
            Mux => {
                vec![
                    ("enabled".into(), if self.config.mux.enabled { "true".into() } else { "false".into() }),
                    ("concurrency".into(), self.config.mux.concurrency.map(|v| v.to_string()).unwrap_or_default()),
                    ("fragment_enabled".into(), if self.config.mux.fragment_enabled { "true".into() } else { "false".into() }),
                    ("fragment_packets".into(), self.config.mux.fragment_packets.clone().unwrap_or_default()),
                    ("fragment_length".into(), self.config.mux.fragment_length.clone().unwrap_or_default()),
                    ("fragment_interval".into(), self.config.mux.fragment_interval.clone().unwrap_or_default()),
                ]
            }
            Stats => {
                vec![
                    ("enabled".into(), if self.config.statistics.enabled { "true".into() } else { "false".into() }),
                ]
            }
            Routing => {
                vec![]
            }
            Updates => {
                vec![]
            }
        }
    }

    fn apply_settings_fields(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        use crate::SettingsSection::*;
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
                .unwrap_or("")
                .to_owned()
        };
        let get_opt = |key: &str| {
            let v = get(key);
            if v.is_empty() { None } else { Some(v) }
        };
        match section {
            Core => {
                self.config.core.xray_path = get_opt("xray_path");
                self.config.core.sing_box_path = get_opt("sing_box_path");
                let core_str = get("default_core");
                self.config.core.core_type = if core_str.is_empty() || core_str == "Auto" {
                    None
                } else {
                    core_str.parse::<xray_tui_core::CoreType>().ok()
                };
                if !get("log_level").is_empty() {
                    self.config.core.log_level = get("log_level");
                }
            }
            Gui => {
                self.config.gui.language = get("language");
                self.config.gui.theme = get_opt("theme");
                if let Ok(v) = get("refresh_interval").parse::<u64>() {
                    self.config.gui.refresh_interval_secs = v;
                }
            }
            Inbound => {
                if let Ok(v) = get("socks_port").parse::<u16>() {
                    self.config.inbound.socks_port = v;
                }
                self.config.inbound.http_port = get("http_port").parse::<u16>().ok();
                self.config.inbound.mixed_port = get("mixed_port").parse::<u16>().ok();
                if !get("listen").is_empty() {
                    self.config.inbound.listen = get("listen");
                }
                self.config.inbound.sniffing = get("sniffing") == "true";
            }
            SystemProxy => {
                self.config.system_proxy.enabled = get("enabled") == "true";
                self.config.system_proxy.http_port = get("http_port").parse::<u16>().ok();
                self.config.system_proxy.socks_port = get("socks_port").parse::<u16>().ok();
                self.config.system_proxy.bypass = get_opt("bypass");
            }
            Tun => {
                self.config.tun.enabled = get("enabled") == "true";
                self.config.tun.interface_name = get_opt("interface_name");
                self.config.tun.mtu = get("mtu").parse::<u16>().ok();
            }
            Mux => {
                self.config.mux.enabled = get("enabled") == "true";
                self.config.mux.concurrency = get("concurrency").parse::<u8>().ok();
                self.config.mux.fragment_enabled = get("fragment_enabled") == "true";
                self.config.mux.fragment_packets = get_opt("fragment_packets");
                self.config.mux.fragment_length = get_opt("fragment_length");
                self.config.mux.fragment_interval = get_opt("fragment_interval");
            }
            Stats => {
                self.config.statistics.enabled = get("enabled") == "true";
            }
            // Dns and Routing are handled separately (DB-backed)
            Dns | Routing | Updates => {}
        }
    }

    pub fn enter_settings_form(&mut self, section: SettingsSection) {
        let fields = self.build_settings_fields(section);
        let mode = match section {
            SettingsSection::Core => SettingsMode::CoreForm { fields, focus_index: 0 },
            SettingsSection::Gui => SettingsMode::GuiForm { fields, focus_index: 0 },
            SettingsSection::Inbound => SettingsMode::InboundForm { fields, focus_index: 0 },
            SettingsSection::Routing => SettingsMode::RoutingList { selected: 0 },
            SettingsSection::Dns => SettingsMode::DnsForm { fields, focus_index: 0 },
            SettingsSection::SystemProxy => SettingsMode::SystemProxyForm { fields, focus_index: 0 },
            SettingsSection::Tun => SettingsMode::TunForm { fields, focus_index: 0 },
            SettingsSection::Mux => SettingsMode::MuxForm { fields, focus_index: 0 },
            SettingsSection::Stats => SettingsMode::StatsForm { fields, focus_index: 0 },
            SettingsSection::Updates => {
                let status_xray = self.update_status.get(&CoreType::Xray).cloned().unwrap_or_default();
                let status_singbox = self.update_status.get(&CoreType::SingBox).cloned().unwrap_or_default();
                SettingsMode::UpdateForm { status_xray, status_singbox }
            }
        };
        self.mode = AppMode::Settings { mode };
    }

    pub fn save_settings_form(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        self.apply_settings_fields(section, fields);
        if let Err(e) = self.config.save() {
            self.add_log("error", &format!("Failed to save config: {e}"));
        } else {
            self.add_log("info", "Settings saved");
        }
        self.enter_settings();
    }

    pub fn save_routing_rule(&mut self, rule_id: Option<String>, fields: &[(String, String)]) {
        let id = rule_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
                .unwrap_or("")
                .to_owned()
        };
        let get_opt = |key: &str| {
            let v = get(key);
            if v.is_empty() { None } else { Some(v) }
        };
        let rule = RoutingRule {
            id,
            group_id: None,
            r#type: get("type").parse::<i32>().unwrap_or(0),
            domain_matcher: get_opt("domain_matcher"),
            domains: get_opt("domains"),
            ips: get_opt("ips"),
            inbound_tags: get_opt("inbound_tags"),
            port: get_opt("port"),
            source_ports: get_opt("source_ports"),
            network: get_opt("network"),
            protocols: get_opt("protocols"),
            domain_strategy: get_opt("domain_strategy"),
            outbound_tag: get_opt("outbound_tag"),
            balancer_tag: get_opt("balancer_tag"),
            rule_set_file: get_opt("rule_set_file"),
            rule_set_url: get_opt("rule_set_url"),
            sort_order: None,
        };
        let result = if rule_id.is_some() {
            self.db.update_routing_rule(&rule)
        } else {
            self.db.insert_routing_rule(&rule)
        };
        match result {
            Ok(()) => self.add_log("info", "Routing rule saved"),
            Err(e) => self.add_log("error", &format!("Failed to save routing rule: {e}")),
        }
    }

    pub fn save_dns_settings(&mut self, fields: &[(String, String)]) {
        let id = self
            .db
            .get_dns_settings()
            .ok()
            .flatten()
            .map(|d| d.id)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
                .unwrap_or("")
                .to_owned()
        };
        let get_opt = |key: &str| {
            let v = get(key);
            if v.is_empty() { None } else { Some(v) }
        };
        let dns = DnsSetting {
            id,
            name: None,
            servers: get_opt("servers"),
            hosts: get_opt("hosts"),
            query_strategy: get_opt("query_strategy"),
            disable_cache: Some(if get("disable_cache") == "true" { 1 } else { 0 }),
            disable_fallback: Some(if get("disable_fallback") == "true" { 1 } else { 0 }),
            client_ip: get_opt("client_ip"),
        };
        match self.db.upsert_dns_settings(&dns) {
            Ok(()) => self.add_log("info", "DNS settings saved"),
            Err(e) => self.add_log("error", &format!("Failed to save DNS settings: {e}")),
        }
    }

    pub fn delete_profile(&mut self, id: &str) {
        if let Err(e) = self.db.delete_profile(id) {
            self.add_log("error", &format!("Failed to delete profile: {e}"));
            return;
        }
        self.add_log("info", "Profile deleted");
        self.confirmation = None;
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
                let protocol =
                    Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
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

    pub fn start_batch_import(&mut self, urls: &[String]) {
        let results: Vec<BatchImportItem> = urls
            .iter()
            .map(|url| match xray_tui_config::import_export::parse_share_url(url) {
                Ok(profile) => BatchImportItem {
                    url: url.clone(),
                    profile: Some(profile),
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
        self.mode = AppMode::BatchImport { results, scroll: 0 };
    }

    pub fn confirm_batch_import(&mut self) {
        let items = match &self.mode {
            AppMode::BatchImport { results, .. } => results.clone(),
            _ => return,
        };
        let now = format_now();
        let mut imported = 0usize;
        let mut errors = 0usize;
        for item in &items {
            if let Some(mut profile) = item.profile.clone() {
                profile.sub_uid = Some(profile.compute_sub_uid() as i64);
                if profile.created_at.is_none() {
                    profile.created_at = Some(now.clone());
                }
                if profile.updated_at.is_none() {
                    profile.updated_at = Some(now.clone());
                }
                if self.db.insert_profile(&profile).is_ok() {
                    imported += 1;
                } else {
                    errors += 1;
                }
            }
        }
        self.add_log("info", &format!("Batch import: {imported} imported, {errors} errors"));
        self.mode = AppMode::List;
        self.reload_profiles();
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
        if let Err(e) = self
            .db
            .reorder_profiles(&[(id.clone(), b_order), (prev_id.clone(), a_order)])
        {
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
        if let Err(e) = self
            .db
            .reorder_profiles(&[(id.clone(), b_order), (next_id.clone(), a_order)])
        {
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
                self.add_log(
                    "error",
                    &format!("Unknown protocol: {}", profile.config_type),
                );
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
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
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
            v2ray_api_enabled: matches!(core_type, CoreType::Xray),
            clash_api_enabled: matches!(core_type, CoreType::SingBox),
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
        let bin_dir = config_dir.join("bin");
        let bin_configs_dir = config_dir.join("binConfigs");

        // Need to move params into the async block

        tokio::spawn(async move {
            // 1. Build config
            let backend_config =
                match ConfigBuilder::build(&profile, core_type, &params, &routing, &dns) {
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

            let profile_id = profile.id.clone();

            if core_type == CoreType::Xray {
                // === gRPC polling loop (xray-core) ===
                let provider = match grpc_client::create_stats_provider(CoreType::Xray).await {
                    Ok(p) => Some(p),
                    Err(e) => {
                        let _ = tx.send(CoreEvent::Error(format!("Stats API unavailable: {e}")));
                        None
                    }
                };
                let poll_interval = std::time::Duration::from_secs(3);
                let mut ticker = tokio::time::interval(poll_interval);
                ticker.tick().await;
                let mut sys_tick_counter = 0u8;
                loop {
                    tokio::select! {
                        _ = &mut stop_rx => break,
                        _ = ticker.tick() => {
                            if let Some(ref provider) = provider {
                                match provider.query_stats("outbound>>>*>>>traffic>>>*", true).await {
                                    Ok(stats) => {
                                        let mut today_up = 0i64;
                                        let mut today_down = 0i64;
                                        for stat in &stats {
                                            if stat.name.contains(">>>uplink") {
                                                today_up += stat.value;
                                            } else if stat.name.contains(">>>downlink") {
                                                today_down += stat.value;
                                            }
                                        }
                                        let _ = tx.send(CoreEvent::StatsUpdate {
                                            profile_id: profile_id.clone(),
                                            today_up,
                                            today_down,
                                            total_up: today_up,
                                            total_down: today_down,
                                        });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(CoreEvent::Error(format!("Stats query failed: {e}")));
                                    }
                                }
                                // sys stats every 3rd tick (~9s)
                                sys_tick_counter += 1;
                                if sys_tick_counter >= 3 {
                                    sys_tick_counter = 0;
                                    match provider.get_sys_stats().await {
                                        Ok(sys) => { let _ = tx.send(CoreEvent::SysStatsUpdate(sys)); }
                                        Err(e) => {
                                            let _ = tx.send(CoreEvent::Error(
                                                format!("Sys stats query failed: {e}"),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // === Sing-box Clash API /traffic streaming ===
                let url = format!("http://127.0.0.1:{}/traffic", CLASH_API_PORT);

                match reqwest::Client::new().get(&url).send().await {
                    Ok(resp) => {
                        let mut stream = Box::pin(resp.bytes_stream());
                        let mut buf = Vec::new();
                        let mut session_up: i64 = 0;
                        let mut session_down: i64 = 0;
                        loop {
                            tokio::select! {
                                _ = &mut stop_rx => break,
                                chunk = stream.next() => {
                                    match chunk {
                                        Some(Ok(bytes)) => {
                                            buf.extend_from_slice(&bytes);
                                            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                                                let line: Vec<u8> = buf.drain(..=pos).collect();
                                                let trimmed = line.as_slice().trim_ascii();
                                                if let Ok(t) = serde_json::from_slice::<ClashTraffic>(trimmed) {
                                                    session_up += t.up;
                                                    session_down += t.down;
                                                    let _ = tx.send(CoreEvent::StatsUpdate {
                                                        profile_id: profile_id.clone(),
                                                        today_up: session_up,
                                                        today_down: session_down,
                                                        total_up: session_up,
                                                        total_down: session_down,
                                                    });
                                                }
                                            }
                                        }
                                        Some(Err(e)) => {
                                            let _ = tx.send(CoreEvent::StatsError(
                                                format!("Clash API stream error: {e}")
                                            ));
                                            break;
                                        }
                                        None => break,
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(CoreEvent::StatsError(
                            format!("Clash API unavailable (is sing-box running?): {e}")
                        ));
                    }
                }
            }

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

    // ── Speed test methods ────────────────────────────────────────

    /// Start TCP ping on the given profile. Returns immediately; result arrives via CoreEvent.
    pub fn start_tcp_ping(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            self.add_log("warn", "Test already in progress for this profile");
            return;
        }

        // Find the profile and extract address:port
        let row = match self.profiles.iter().find(|r| r.profile.id == profile_id) {
            Some(r) => r,
            None => {
                self.add_log("error", "Profile not found for TCP ping");
                return;
            }
        };
        let addr = match &row.profile.address {
            Some(a) => a.clone(),
            None => {
                self.add_log("error", "Profile has no address");
                return;
            }
        };
        let port = match row.profile.port {
            Some(p) if p > 0 && p <= 65535 => p as u16,
            _ => {
                self.add_log("error", "Profile has invalid port");
                return;
            }
        };

        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => {
                self.add_log("error", "Core event channel not initialized");
                return;
            }
        };

        let pid = profile_id.to_string();
        self.testing_profiles.insert(pid.clone());
        let timeout_dur = std::time::Duration::from_secs(5);

        tokio::spawn(async move {
            let result = xray_tui_core::speed_test::tcp_ping(&addr, port, timeout_dur).await;
            let (latency_ms, error) = match result {
                Ok(dur) => (Some(dur.as_millis() as u64), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let _ = tx.send(CoreEvent::SpeedTestResult {
                profile_id: pid,
                test_type: TestType::TcpPing,
                latency_ms,
                speed_bps: None,
                error,
            });
        });
    }

    /// Start real ping (HTTP through proxy) on the given profile.
    pub fn start_real_ping(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }
        if self.connected_core.is_none() {
            self.add_log("warn", "Core not connected — proxy required for real ping");
            return;
        }
        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_profiles.insert(pid.clone());
        let proxy_addr = self.config.inbound.listen.clone();
        let proxy_port = self.config.inbound.socks_port;
        let test_url = "http://www.gstatic.com/generate_204".to_string();
        let timeout_dur = std::time::Duration::from_secs(5);

        tokio::spawn(async move {
            let result = xray_tui_core::speed_test::real_ping(
                &proxy_addr,
                proxy_port,
                &test_url,
                timeout_dur,
            )
            .await;
            let (latency_ms, error) = match result {
                Ok(dur) => (Some(dur.as_millis() as u64), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let _ = tx.send(CoreEvent::SpeedTestResult {
                profile_id: pid,
                test_type: TestType::RealPing,
                latency_ms,
                speed_bps: None,
                error,
            });
        });
    }

    /// Start speed test (download through proxy) on the given profile.
    pub fn start_speed_test(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }
        if self.connected_core.is_none() {
            self.add_log("warn", "Core not connected — proxy required for speed test");
            return;
        }
        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_profiles.insert(pid.clone());
        let proxy_addr = self.config.inbound.listen.clone();
        let proxy_port = self.config.inbound.socks_port;
        let test_url = "http://cachefly.cachefly.net/1mb.test".to_string();
        let min_dur = std::time::Duration::from_secs(3);
        let max_dur = std::time::Duration::from_secs(10);

        tokio::spawn(async move {
            let result = xray_tui_core::speed_test::speed_test(
                &proxy_addr,
                proxy_port,
                &test_url,
                min_dur,
                max_dur,
            )
            .await;
            let (speed_bps, error) = match result {
                Ok(bps) => (Some(bps), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let _ = tx.send(CoreEvent::SpeedTestResult {
                profile_id: pid,
                test_type: TestType::SpeedTest,
                latency_ms: None,
                speed_bps,
                error,
            });
        });
    }

    /// Start UDP test on the given profile.
    pub fn start_udp_test(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }
        if self.connected_core.is_none() {
            self.add_log("warn", "Core not connected — proxy required for UDP test");
            return;
        }
        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_profiles.insert(pid.clone());
        let proxy_addr = self.config.inbound.listen.clone();
        let proxy_port = self.config.inbound.socks_port;
        let timeout_dur = std::time::Duration::from_secs(5);

        tokio::spawn(async move {
            let result =
                xray_tui_core::speed_test::udp_test(&proxy_addr, proxy_port, timeout_dur).await;
            let (latency_ms, error) = match result {
                Ok(dur) => (Some(dur.as_millis() as u64), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let _ = tx.send(CoreEvent::SpeedTestResult {
                profile_id: pid,
                test_type: TestType::UdpTest,
                latency_ms,
                speed_bps: None,
                error,
            });
        });
    }

    /// Batch TCP ping all visible (filtered) profiles, deduplicating by address:port.
    pub fn start_batch_ping(&mut self) {
        let visible = self.filtered_profiles();
        if visible.is_empty() {
            self.add_log("info", "No profiles to ping");
            return;
        }

        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };

        // Deduplicate: collect unique (address, port) pairs and the profiles that share them
        let mut unique_targets: Vec<UniqueTarget> = Vec::new();
        for row in &visible {
            let addr = match &row.profile.address {
                Some(a) => a.clone(),
                None => continue,
            };
            let port = match row.profile.port {
                Some(p) if p > 0 && p <= 65535 => p as u16,
                _ => continue,
            };
            let key = (addr.clone(), port);
            // Find or create the target
            match unique_targets.iter_mut().find(|t| t.key == key) {
                Some(t) => t.profile_ids.push(row.profile.id.clone()),
                None => unique_targets.push(UniqueTarget {
                    key,
                    profile_ids: vec![row.profile.id.clone()],
                }),
            }
        }

        // All testing flags now set after the visible borrow is dropped
        for target in &unique_targets {
            for pid in &target.profile_ids {
                self.testing_profiles.insert(pid.clone());
            }
        }

        let total = unique_targets.iter().map(|t| t.profile_ids.len()).sum();
        self.test_progress = Some((0, total));
        let timeout_dur = std::time::Duration::from_secs(5);

        tokio::spawn(async move {
            for target in &unique_targets {
                let result =
                    xray_tui_core::speed_test::tcp_ping(&target.key.0, target.key.1, timeout_dur)
                        .await;
                let (latency_ms, error) = match result {
                    Ok(dur) => (Some(dur.as_millis() as u64), None),
                    Err(e) => (None, Some(e.to_string())),
                };

                for pid in &target.profile_ids {
                    let _ = tx.send(CoreEvent::SpeedTestResult {
                        profile_id: pid.clone(),
                        test_type: TestType::TcpPing,
                        latency_ms,
                        speed_bps: None,
                        error: error.clone(),
                    });
                }
            }
            // Batch progress is cleared in poll_core_events
        });
    }
    /// Remove profiles whose extension.delay == Some(-1) (failed TCP ping).
    pub fn remove_failed_servers(&mut self) {
        let to_remove: Vec<String> = self
            .profiles
            .iter()
            .filter(|r| {
                r.extension
                    .as_ref()
                    .map(|e| e.delay == Some(-1))
                    .unwrap_or(false)
            })
            .map(|r| r.profile.id.clone())
            .collect();
        let count = to_remove.len();
        for id in to_remove {
            self.delete_profile(&id);
        }
        self.multi_select.clear();
        self.add_log("info", &format!("Removed {count} failed server(s)"));
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
                CoreEvent::StatsError(msg) => {
                    self.connection_error = Some(msg.clone());
                    self.add_log("warning", &format!("Stats error: {msg}"));
                }
                CoreEvent::StatsUpdate {
                    profile_id,
                    today_up,
                    today_down,
                    total_up,
                    total_down,
                } => {
                    let stats = ServerStat {
                        profile_id: profile_id.clone(),
                        today_up: Some(today_up as i32),
                        today_down: Some(today_down as i32),
                        total_up: Some(total_up as i32),
                        total_down: Some(total_down as i32),
                        last_updated: Some(crate::format_now()),
                    };
                    if let Err(e) = self.db.upsert_server_stats(&stats) {
                        self.add_log("error", &format!("Failed to save stats: {e}"));
                    }
                    // Update in-memory ProfileRow to avoid full reload
                    if let Some(row) = self
                        .profiles
                        .iter_mut()
                        .find(|r| r.profile.id == profile_id)
                    {
                        row.stats = Some(stats);
                    }
                }
                CoreEvent::SysStatsUpdate(stats) => {
                    self.system_stats = Some(stats);
                }
                CoreEvent::SubscriptionsUpdated {
                    group_id,
                    count,
                    error,
                } => {
                    self.updating_groups.remove(&group_id);
                    if let Some(err) = error {
                        self.add_log("error", &format!("Subscription update failed: {err}"));
                    } else {
                        self.add_log("info", &format!("Subscription updated: {count} profiles"));
                    }
                    self.reload_profiles();
                    self.reload_groups();
                    self.subscriptions = self.db.get_all_subscriptions().unwrap_or_default();
                }
                CoreEvent::SpeedTestResult {
                    profile_id,
                    test_type,
                    latency_ms,
                    speed_bps,
                    error,
                } => {
                    self.testing_profiles.remove(&profile_id);

                    // Update profile extension and extract name in a scoped block
                    // to drop the mutable borrow before further self-method calls.
                    let name = {
                        let row = self
                            .profiles
                            .iter_mut()
                            .find(|r| r.profile.id == profile_id);
                        match row {
                            Some(row) => {
                                let ext = row.extension.get_or_insert_with(|| ProfileExtension {
                                    profile_id: profile_id.clone(),
                                    delay: None,
                                    speed: None,
                                    sort_order: None,
                                    ip_info: None,
                                });
                                match test_type {
                                    TestType::TcpPing | TestType::RealPing | TestType::UdpTest => {
                                        ext.delay = latency_ms.map(|v| v as i32);
                                    }
                                    TestType::SpeedTest => {
                                        ext.speed = speed_bps
                                            .map(|v| std::cmp::min(v, i32::MAX as u64) as i32);
                                    }
                                }
                                let _ = self.db.upsert_profile_extension(ext);
                                row.profile.remarks.clone().unwrap_or(profile_id.clone())
                            }
                            None => profile_id.clone(),
                        }
                    };

                    match error {
                        Some(ref err) => {
                            self.add_log(
                                "warn",
                                &format!("{test_type:?} failed for {name}: {err}"),
                            );
                        }
                        None => {
                            let latency_str =
                                latency_ms.map(|ms| format!("{ms}ms")).unwrap_or_default();
                            let speed_str =
                                speed_bps.map(|bps| format!("{bps}bps")).unwrap_or_default();
                            let detail = if !speed_str.is_empty() {
                                speed_str
                            } else if !latency_str.is_empty() {
                                latency_str
                            } else {
                                "success".to_string()
                            };
                            self.add_log("info", &format!("{test_type:?} {name}: {detail}"));
                        }
                    }

                    // Update batch progress — use take() to avoid borrowing self.test_progress
                    if let Some((done, total)) = self.test_progress.take() {
                        let new_done = done + 1;
                        if new_done >= total {
                            self.add_log(
                                "info",
                                &format!("Batch complete: {new_done}/{total} profiles tested"),
                            );
                        } else {
                            self.test_progress = Some((new_done, total));
                        }
                    }
                }
                CoreEvent::UpdateCheckResult { core_type, current_version, latest_version, error } => {
                    let status = self.update_status.entry(core_type).or_default();
                    status.current_version = current_version.clone();
                    status.latest_version = latest_version.clone();
                    status.update_available = match &current_version {
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
                    };
                    status.error = error;
                    if let Some(ref ver) = latest_version {
                        match core_type {
                            CoreType::Xray => self.config.updates.xray_latest_known = Some(ver.clone()),
                            CoreType::SingBox => self.config.updates.sing_box_latest_known = Some(ver.clone()),
                            CoreType::Auto => {}
                        }
                    }
                    // Refresh form snapshots if currently viewing the updates form
                    if let AppMode::Settings { mode: SettingsMode::UpdateForm { status_xray, status_singbox } } = &mut self.mode {
                        *status_xray = self.update_status.get(&CoreType::Xray).cloned().unwrap_or_default();
                        *status_singbox = self.update_status.get(&CoreType::SingBox).cloned().unwrap_or_default();
                    }
                }
                CoreEvent::UpdateCompleted { core_type, old_version, new_version, success, error } => {
                    let status = self.update_status.entry(core_type).or_default();
                    status.downloading = false;
                    status.download_progress = None;
                    if success {
                        status.current_version = Some(new_version.clone());
                        status.update_available = false;
                        self.add_log("info", &format!("{core_type} updated: {} → {}", old_version.as_deref().unwrap_or("none"), new_version));
                    } else {
                        status.error = error.clone();
                        self.add_log("error", &format!("{core_type} update failed: {:?}", error));
                    }
                    // Refresh form snapshots if currently viewing the updates form
                    if let AppMode::Settings { mode: SettingsMode::UpdateForm { status_xray, status_singbox } } = &mut self.mode {
                        *status_xray = self.update_status.get(&CoreType::Xray).cloned().unwrap_or_default();
                        *status_singbox = self.update_status.get(&CoreType::SingBox).cloned().unwrap_or_default();
                    }
                }
            }
        }
    }
    // ── Group management ─────────────────────────────────────────────

    pub fn start_add_group(&mut self) {
        let fields = vec![
            ("name".into(), String::new()),
            ("subscription_url".into(), String::new()),
            ("user_agent".into(), String::new()),
            ("update_interval".into(), "1440".into()),
            ("core_type".into(), "auto".into()),
        ];
        self.mode = AppMode::AddGroup {
            fields,
            focus_index: 0,
        };
    }

    pub fn start_edit_group(&mut self, group_id: &str) {
        let group = match self.groups.iter().find(|g| g.id == group_id) {
            Some(g) => g.clone(),
            None => {
                self.add_log("error", "Group not found");
                return;
            }
        };
        let fields = vec![
            ("name".into(), group.name.unwrap_or_default()),
            (
                "subscription_url".into(),
                group.subscription_url.unwrap_or_default(),
            ),
            ("user_agent".into(), group.user_agent.unwrap_or_default()),
            ("update_interval".into(), "1440".into()),
            (
                "core_type".into(),
                group.core_type.unwrap_or_else(|| "auto".into()),
            ),
        ];
        self.mode = AppMode::EditGroup {
            group_id: group_id.into(),
            fields,
            focus_index: 0,
        };
    }

    pub fn confirm_add_group(&mut self) {
        let fields = match &self.mode {
            AppMode::AddGroup { fields, .. } => fields.clone(),
            _ => return,
        };
        let group = Group {
            id: uuid::Uuid::new_v4().to_string(),
            name: get_field(&fields, "name"),
            subscription_url: get_field(&fields, "subscription_url"),
            subscription_enabled: Some(1),
            user_agent: get_field(&fields, "user_agent"),
            convert_target: None,
            core_type: get_field(&fields, "core_type"),
            sort_order: Some((self.groups.len() + 1) as i32),
            is_system: None,
        };
        if let Err(e) = self.db.insert_group(&group) {
            self.add_log("error", &format!("Failed to add group: {e}"));
            return;
        }
        // Create subscription tracking row
        let interval: i32 = get_field(&fields, "update_interval")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1440);
        let sub = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            group_id: Some(group.id.clone()),
            url: group.subscription_url.clone().unwrap_or_default(),
            last_updated: None,
            update_interval: Some(interval),
            user_agent: group.user_agent.clone(),
            status: Some("idle".into()),
            error_message: None,
        };
        let _ = self.db.upsert_subscription(&sub);
        self.add_log(
            "info",
            &format!(
                "Group '{}' added",
                group.name.as_deref().unwrap_or("unnamed")
            ),
        );
        self.mode = AppMode::List;
        self.reload_groups();
    }

    pub fn confirm_edit_group(&mut self) {
        let (group_id, fields) = match &self.mode {
            AppMode::EditGroup {
                group_id, fields, ..
            } => (group_id.clone(), fields.clone()),
            _ => return,
        };
        let mut group = match self.groups.iter().find(|g| g.id == group_id) {
            Some(g) => g.clone(),
            None => {
                self.add_log("error", "Group not found");
                return;
            }
        };
        group.name = get_field(&fields, "name");
        group.subscription_url = get_field(&fields, "subscription_url");
        group.user_agent = get_field(&fields, "user_agent");
        group.core_type = get_field(&fields, "core_type");
        if let Err(e) = self.db.update_group(&group) {
            self.add_log("error", &format!("Failed to update group: {e}"));
            return;
        }
        // Update subscription tracking row
        let interval: i32 = get_field(&fields, "update_interval")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1440);
        if let Ok(Some(mut sub)) = self.db.get_subscription_by_group(&group_id) {
            sub.url = group.subscription_url.clone().unwrap_or_default();
            sub.update_interval = Some(interval);
            sub.user_agent = group.user_agent.clone();
            let _ = self.db.upsert_subscription(&sub);
        }
        self.add_log("info", "Group updated");
        self.mode = AppMode::List;
        self.reload_groups();
    }

    pub fn delete_group(&mut self, group_id: &str) {
        if let Err(e) = self.db.delete_group(group_id) {
            self.add_log("error", &format!("Failed to delete group: {e}"));
            return;
        }
        let _ = self.db.delete_subscriptions_by_group(group_id);
        self.add_log("info", "Group deleted");
        self.selected_group_id = None;
        self.confirmation = None;
        self.reload_groups();
        self.reload_profiles();
    }

    pub fn clear_group(&mut self, group_id: &str) {
        match self.db.clear_group(group_id) {
            Ok(count) => {
                self.add_log("info", &format!("Cleared {count} profiles from group"));
            }
            Err(e) => {
                self.add_log("error", &format!("Failed to clear group: {e}"));
            }
        }
        self.confirmation = None;
        self.reload_profiles();
    }

    // ── Subscription update ──────────────────────────────────────────

    pub fn update_group_subscriptions(&mut self, group_id: &str) {
        if self.updating_groups.contains(group_id) {
            return;
        }
        let group = match self.groups.iter().find(|g| g.id == group_id) {
            Some(g) => g.clone(),
            None => {
                self.add_log("error", "Group not found");
                return;
            }
        };
        let url = match &group.subscription_url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => {
                self.add_log("warn", "Group has no subscription URL");
                return;
            }
        };

        self.updating_groups.insert(group_id.to_string());
        let gid = group_id.to_string();
        let tx = self.core_event_tx.clone();
        let user_agent = group
            .user_agent
            .clone()
            .unwrap_or_else(|| "xray-tui/0.1".into());
        let db_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui")
            .join("data.db");

        tokio::spawn(async move {
            let result = Self::do_update_subscription(url, user_agent, gid.clone(), db_path).await;
            if let Some(tx) = &tx {
                let _ = tx.send(CoreEvent::SubscriptionsUpdated {
                    group_id: result.0,
                    count: result.1,
                    error: result.2,
                });
            }
        });
    }

    async fn do_update_subscription(
        url: String,
        user_agent: String,
        group_id: String,
        db_path: std::path::PathBuf,
    ) -> (String, usize, Option<String>) {
        let client = match reqwest::Client::builder()
            .user_agent(&user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(c) => c,
            Err(e) => return (group_id, 0, Some(e.to_string())),
        };
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => return (group_id, 0, Some(format!("HTTP: {e}"))),
        };
        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => return (group_id, 0, Some(format!("Body: {e}"))),
        };

        let profiles = match xray_tui_config::subscription::parse_subscription_data(&bytes) {
            Ok(p) => p,
            Err(e) => return (group_id, 0, Some(e)),
        };

        let db = match Database::open(&db_path) {
            Ok(d) => d,
            Err(e) => return (group_id, 0, Some(format!("DB: {e}"))),
        };

        let now = format_now();
        let mut sub_uids: Vec<u64> = Vec::with_capacity(profiles.len());
        let enriched: Vec<Profile> = profiles
            .into_iter()
            .map(|mut p| {
                p.sub_uid = Some(p.compute_sub_uid() as i64);
                p.group_id = Some(group_id.clone());
                p.is_sub = Some(1);
                p.sub_id = Some(uuid::Uuid::new_v4().to_string());
                p.updated_at = Some(now.clone());
                if p.created_at.is_none() {
                    p.created_at = Some(now.clone());
                }
                sub_uids.push(p.sub_uid.unwrap_or(0) as u64);
                p
            })
            .collect();

        if let Err(e) = db.subscription_upsert_profiles(&group_id, &enriched) {
            return (group_id, 0, Some(format!("DB upsert: {e}")));
        }

        let _ = db.move_orphans_to_graveyard(&group_id, &sub_uids, GRAVEYARD_GROUP_ID);
        let _ = db.purge_graveyard(GRAVEYARD_GROUP_ID, 24);

        let now_str = format_now();
        let sub = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            group_id: Some(group_id.clone()),
            url: url.clone(),
            last_updated: Some(now_str),
            update_interval: Some(1440),
            user_agent: Some(user_agent),
            status: Some("ok".into()),
            error_message: None,
        };
        let _ = db.upsert_subscription(&sub);

        (group_id, enriched.len(), None)
    }

    pub fn update_all_subscriptions(&mut self) {
        let group_ids: Vec<String> = self
            .groups
            .iter()
            .filter(|g| {
                g.id != GRAVEYARD_GROUP_ID
                    && g.id != ALL_GROUP_ID
                    && g.subscription_url.as_deref().is_some_and(|u| !u.is_empty())
            })
            .map(|g| g.id.clone())
            .collect();
        for gid in group_ids {
            self.update_group_subscriptions(&gid);
        }
    }

    /// Spawn async task to check for backend updates on startup or manual trigger.
    pub fn spawn_update_check(&mut self) {
        let Some(tx) = self.core_event_tx.clone() else {
            return;
        };
        let bin_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui/bin");
        for &core_type in &[CoreType::Xray, CoreType::SingBox] {
            let tx = tx.clone();
            let bin_dir = bin_dir.clone();
            tokio::spawn(async move {
                let current = xray_tui_core::updater::get_current_version(core_type, &bin_dir).await;
                let latest = xray_tui_core::updater::get_latest_version(core_type).await;
                let error = if current.is_none() && latest.is_none() {
                    Some("binary not found and check failed".into())
                } else if latest.is_none() {
                    Some("failed to check latest version".into())
                } else {
                    None
                };
                let _ = tx.send(CoreEvent::UpdateCheckResult {
                    core_type,
                    current_version: current,
                    latest_version: latest,
                    error,
                });
            });
        }
    }

    /// Spawn async task to download and install an update for the given core.
    pub fn spawn_update_download(&mut self, core_type: CoreType) {
        // Guard: don't download if already downloading
        if self.update_status.get(&core_type).map(|s| s.downloading).unwrap_or(false) {
            return;
        }
        // Guard: don't download if core is currently running
        if self.connected_core == Some(core_type) {
            self.add_log("warn", &format!("Cannot update {core_type} while it's running. Disconnect first."));
            return;
        }

        let latest = match self.update_status.get(&core_type).and_then(|s| s.latest_version.clone()) {
            Some(v) => v,
            None => return,
        };
        let old_version = self.update_status.get(&core_type).and_then(|s| s.current_version.clone());
        let Some(tx) = self.core_event_tx.clone() else {
            return;
        };
        let bin_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui/bin");
        let client = reqwest::Client::new();
        let temp_dir = std::env::temp_dir().join(format!("xray-tui-update-{core_type}"));

        self.update_status.entry(core_type).or_default().downloading = true;

        tokio::spawn(async move {
            // Download
            let archive = match xray_tui_core::updater::download_release(&client, core_type, &latest, &temp_dir).await {
                Ok(path) => path,
                Err(e) => {
                    let _ = tx.send(CoreEvent::UpdateCompleted {
                        core_type, old_version: old_version.clone(), new_version: latest,
                        success: false, error: Some(e),
                    });
                    return;
                }
            };
            // Install
            let result = xray_tui_core::updater::install_binary(&archive, core_type, &bin_dir).await;
            let (success, error) = match result {
                Ok(_) => (true, None),
                Err(e) => (false, Some(e)),
            };
            // Clean up temp file
            let _ = std::fs::remove_file(&archive);
            let _ = std::fs::remove_dir_all(&temp_dir);

            let old_version = old_version.clone();
            let _ = tx.send(CoreEvent::UpdateCompleted {
                core_type, old_version, new_version: latest,
                success, error,
            });
        });
    }

    /// Start a background task to check and update subscriptions.
    pub fn spawn_auto_update(&mut self) {
        let Some(tx) = self.core_event_tx.clone() else {
            return;
        };
        let db_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui")
            .join("data.db");

        tokio::spawn(async move {
            use std::time::Duration;
            tokio::time::sleep(Duration::from_secs(10)).await;
            loop {
                let db = match Database::open(&db_path) {
                    Ok(d) => d,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };
                let due_groups = match db.get_groups_due_update() {
                    Ok(g) => g,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };
                for group in &due_groups {
                    let url = match &group.subscription_url {
                        Some(u) => u.clone(),
                        None => continue,
                    };
                    let ua = group
                        .user_agent
                        .clone()
                        .unwrap_or_else(|| "xray-tui/0.1".into());
                    let gid = group.id.clone();
                    let result = Self::do_update_subscription(url, ua, gid, db_path.clone()).await;
                    let _ = tx.send(CoreEvent::SubscriptionsUpdated {
                        group_id: result.0,
                        count: result.1,
                        error: result.2,
                    });
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
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
        && let Ok(obj) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ss)
    {
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
        && let Ok(obj) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(ps)
    {
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

/// Get a value from a form field list, returning None if empty or missing.
pub fn get_field(fields: &[(String, String)], key: &str) -> Option<String> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
}

fn format_now() -> String {
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
        if remaining < days_in_year {
            break;
        }
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
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
