#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "TUI display domain: ports validated to u16, timestamps fit i64 for billions of years, list indices < u16 for rendering, display precision loss acceptable"
)]
#![allow(
    clippy::future_not_send,
    clippy::manual_let_else,
    reason = "single-threaded TUI, futures never sent across threads; manual let-else where match is clearer"
)]
pub mod ui;
use crate::ui::settings::PROTOCOL_CORE_DEFS;

use futures_util::StreamExt;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinHandle;
use tracing::warn;
use xray_tui_config::{AppConfig, ValidationSettings, ValidationSummary};
use xray_tui_core::grpc_client;
use xray_tui_core::log_heed::HeedLogStorage;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{
    BuildParams, CLASH_API_PORT, ConfigBuilder, CoreManager, CoreType, find_binary,
    ping::PingError, resolve_core,
};
use xray_tui_db::Database;
use xray_tui_db::models::{
    ALL_GROUP_ID, DnsSetting, GRAVEYARD_GROUP_ID, Group, PingResultUpdate, Profile,
    ProfileExtension, RoutingRule, ServerStat, Subscription,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Logs,
    Statistics,
}

impl Tab {
    pub const ALL: &[Self] = &[Self::Profiles, Self::Settings, Self::Logs, Self::Statistics];
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
    pub target: String,
    pub message: String,
    pub timestamp_nanos: i64,
}

/// Right pane content in the Settings split view.
#[derive(Debug, Clone)]
pub enum SplitRightPane {
    Empty,
    Form {
        section: SettingsSection,
        fields: Vec<(String, String)>,
        focus_index: usize,
        form_errors: HashMap<String, String>,
    },
    RoutingList {
        selected: usize,
    },
    RoutingForm {
        rule_id: Option<String>,
        fields: Vec<(String, String)>,
        focus_index: usize,
        form_errors: HashMap<String, String>,
    },
    UpdateForm {
        status_xray: BackendUpdateStatus,
        status_singbox: BackendUpdateStatus,
    },
}

/// Which pane has focus in the Settings split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitFocus {
    Tree,
    Right,
}

use ratatui_cheese::tree::TreeState;

/// Sub-modes for the Settings panel.
#[derive(Debug)]
pub enum SettingsMode {
    Split {
        tree: RefCell<TreeState>,
        focus: SplitFocus,
        right: SplitRightPane,
    },
}

impl Clone for SettingsMode {
    fn clone(&self) -> Self {
        match self {
            Self::Split { tree, focus, right } => Self::Split {
                tree: RefCell::new(tree.borrow().clone()),
                focus: *focus,
                right: right.clone(),
            },
        }
    }
}
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
    ProtocolCore,
    Updates,
    SpeedTest,
    Logging,
}
/// Tracks what the UI is currently showing.
#[derive(Debug, Clone)]
pub enum AppMode {
    /// The main profile list
    List,
    /// Help overlay
    Help,
    Settings {
        mode: SettingsMode,
    },
    /// Adding a new server
    AddServer {
        /// Selected protocol (None while protocol picker shown)
        protocol: Option<Protocol>,
        /// Raw form field values: map of field key -> current input
        fields: Vec<(String, String)>,
        /// Index of the focused field
        focus_index: usize,
        /// Per-field validation errors
        form_errors: HashMap<String, String>,
    },
    /// Editing an existing server
    EditServer {
        profile_id: String,
        fields: Vec<(String, String)>,
        focus_index: usize,
        /// Per-field validation errors
        form_errors: HashMap<String, String>,
    },
    /// Import URL from paste
    ImportUrl {
        input: String,
        error: Option<String>,
    },
    /// Managing subscription groups
    ManageGroups {
        selected: usize,
    },
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
    SpeedTestMenu {
        selected: usize,
    },
    /// Batch import multiple share URLs
    BatchImport {
        /// Parsed/split profiles for each URL
        results: Vec<BatchImportItem>,
        /// Current scroll position
        scroll: usize,
    },
    /// Target filter picker overlay for the logs tab.
    TargetPicker {
        selected: usize,
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
    /// Non-fatal stats error — keeps `connected_core` intact
    StatsError(String),
    StatsUpdate {
        profile_id: String,
        today_up: i64,
        today_down: i64,
        total_up: i64,
        total_down: i64,
    },
    /// System stats update from gRPC.
    SysStatsUpdate(grpc_client::SysStats),
    /// A log line from the core process stderr.
    LogLine {
        level: String,
        target: String,
        message: String,
        timestamp_nanos: i64,
    },
    /// A log line from the TUI internals via tracing.
    TuiLog {
        target: String,
        level: String,
        message: String,
    },
    SubscriptionsUpdated {
        group_id: String,
        count: usize,
        error: Option<String>,
        summary: ValidationSummary,
    },
    /// Result from a speed test operation
    SpeedTestResult {
        profile_id: String,
        test_type: TestType,
        latency_ms: Option<u64>,
        speed_bps: Option<u64>,
        ip_info: Option<String>,
        error: Option<String>,
    },
    /// Result of a version check for a proxy backend.
    UpdateCheckResult {
        core_type: CoreType,
        current_version: Option<String>,
        latest_version: Option<String>,
        error: Option<String>,
    },
    /// Progress notification during update download.
    UpdateDownloadProgress {
        core_type: CoreType,
        downloaded: u64,
        total: u64,
    },
    /// Result of a download+install operation.
    UpdateCompleted {
        core_type: CoreType,
        old_version: Option<String>,
        new_version: String,
        success: bool,
        error: Option<String>,
    },
    /// Update the displayed test type for a profile without triggering cleanup.
    /// Used by `batch_then_real_ping` to switch from TcpPing→RealPing emoji
    /// after TCP completes but before real ping starts.
    TestTypeUpdate {
        profile_id: String,
        test_type: TestType,
    },
}

pub enum ConfirmAction {
    DeleteProfile(String),
    DeleteProfiles(Vec<String>),
    DeleteGroup(String),
    ClearGroup(String),
    ClearLogs,
    PurgeLogsDatabase,
    Quit,
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

#[allow(
    clippy::struct_excessive_bools,
    reason = "AppState aggregates many UI state flags"
)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: AppConfig,
    /// Currently selected theme name from config or UI selection.
    pub theme_name: ratatui_themes::ThemeName,
    pub current_tab: Tab,
    pub profiles: Vec<ProfileRow>,
    /// Cached filtered/sorted profile indices for performance.
    pub cached_filtered_indices: RefCell<Vec<usize>>,
    pub filter_cache_valid: Cell<bool>,
    pub groups: Vec<Group>,
    pub subscriptions: Vec<Subscription>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    /// Scroll offset from the bottom of the log buffer (0 = newest visible).
    pub log_scroll: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub connected_core: Option<CoreType>,
    pub connecting: bool,
    pub system_stats: Option<grpc_client::SysStats>,
    /// Cached log entries for the Logs tab, newest at end.
    pub log_cache: VecDeque<LogLine>,
    /// Whether there may be older entries in the DB to load.
    pub log_has_older: bool,
    /// Flag to trigger progressive loading of all logs to top (Home key).
    pub log_seek_home: bool,
    pub connection_error: Option<String>,
    pub core_event_rx: Option<mpsc::Receiver<CoreEvent>>,
    pub core_event_tx: Option<mpsc::Sender<CoreEvent>>,
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
    /// Which test type is currently running per profile (for display).
    pub testing_details: HashMap<String, TestType>,
    /// Cached update status for both backends.
    pub update_status: HashMap<CoreType, BackendUpdateStatus>,
    pub actions_compact: bool,
    pub connected_profile_id: Option<String>,
    /// Shared stop flag for batch speed tests.
    pub speed_test_stop: Arc<AtomicBool>,
    pub last_test_tcp: Option<u64>,
    /// Shared batch progress (total, completed) displayed in status bar.
    pub batch_progress: Option<Arc<(AtomicU16, AtomicU16)>>,
    pub last_test_real: Option<u64>,
    pub last_test_speed: Option<u64>,
    pub current_traffic_up: i64,
    pub current_traffic_down: i64,
    pub current_memory: u64,
    pub term_height: Cell<u16>,
    pub routing_rules: Vec<RoutingRule>,
    pub shutdown_token: Arc<AtomicBool>,
    /// Handle to the core process task, used for clean shutdown.
    pub core_task_handle: Option<JoinHandle<()>>,
    /// Heed-backed persistent log storage.
    pub heed_storage: Option<Arc<HeedLogStorage>>,
    /// Highest `timestamp_ns` we've seen (for polling new logs).
    pub last_seen_log_ns: u64,
    /// Known target names from the heed targets database.
    pub known_targets: Vec<String>,
    /// Selected targets for filtering (empty = show all).
    pub selected_targets: Vec<String>,
    /// Last time we polled heed for new log entries.
    pub last_heed_poll: std::time::Instant,
    /// Channel sender for non-blocking log persistence.
    /// `TuiLogLayer` and core log forwarder send `LogMessage` here; background writer batches and writes to heed.
    pub log_sender_tx: Option<std::sync::mpsc::Sender<xray_tui_core::log_heed::LogMessage>>,
    /// Whether initial logs have been loaded from heed into `log_cache` yet.
    pub logs_loaded: bool,
}
impl AppState {
    pub async fn new(db: Arc<Database>, config: AppConfig) -> Self {
        let theme_name = config.theme_name;
        let (core_tx, core_rx) = tokio::sync::mpsc::channel(65536);
        let mut state = Self {
            db,
            config,
            theme_name,
            current_tab: Tab::Profiles,
            update_status: HashMap::new(),
            profiles: Vec::new(),
            cached_filtered_indices: RefCell::new(Vec::new()),
            filter_cache_valid: Cell::new(true),
            groups: Vec::new(),
            subscriptions: Vec::new(),
            selected_group_id: None,
            selected_index: 0,
            log_scroll: 0,
            sort_column: SortColumn::Remarks,
            sort_ascending: true,
            search_query: String::new(),
            search_focused: false,
            log_cache: VecDeque::new(),
            log_has_older: false,
            log_seek_home: false,
            connected_core: None,
            connecting: false,
            connection_error: None,
            core_event_rx: Some(core_rx),
            core_event_tx: Some(core_tx),
            disconnect_tx: None,
            should_quit: false,
            mode: AppMode::List,
            previous_mode: None,
            multi_select: HashSet::new(),
            clipboard: None,
            confirmation: None,
            updating_groups: HashSet::new(),
            testing_profiles: HashSet::new(),
            testing_details: HashMap::new(),
            system_stats: None,
            actions_compact: false,
            connected_profile_id: None,
            last_test_tcp: None,
            last_test_real: None,
            last_test_speed: None,
            current_traffic_up: 0,
            current_traffic_down: 0,
            current_memory: 0,
            routing_rules: Vec::new(),
            core_task_handle: None,
            shutdown_token: Arc::new(AtomicBool::new(false)),
            speed_test_stop: Arc::new(AtomicBool::new(false)),
            batch_progress: None,
            term_height: Cell::new(80),
            heed_storage: None,
            last_seen_log_ns: 0,
            known_targets: Vec::new(),
            selected_targets: Vec::new(),
            last_heed_poll: std::time::Instant::now(),
            log_sender_tx: None,
            logs_loaded: false,
        };
        state.reload_profiles().await;
        state.reload_groups().await;
        state.subscriptions = state.db.get_all_subscriptions().await.unwrap_or_default();
        state.spawn_auto_update();
        state
    }
    /// Build a ratatui-cheese `Palette` from the currently selected theme.
    #[must_use]
    pub const fn current_palette(&self) -> ratatui_cheese::theme::Palette {
        let theme = ratatui_themes::Theme::new(self.theme_name);
        crate::ui::palette_bridge::current_palette(&theme)
    }

    /// Load the most recent log entries from heed into [`log_cache`].
    /// Uses async heed methods that wrap LMDB reads in `spawn_blocking`.
    pub async fn load_initial_logs(&mut self) {
        let heed = match &self.heed_storage {
            Some(h) => h.clone(),
            None => return,
        };
        match heed.read_recent_async(500).await {
            Ok(entries) => {
                self.log_has_older = entries.len() >= 500;
                self.log_cache = entries
                    .into_iter()
                    .rev()
                    .map(|e| LogLine {
                        level: e.level,
                        target: e.target,
                        message: e.message,
                        timestamp_nanos: e.timestamp_nanos as i64,
                    })
                    .collect();
                // Update last_seen_log_ns from the newest entry
                if let Some(newest) = self.log_cache.back() {
                    self.last_seen_log_ns = newest.timestamp_nanos as u64;
                }
                // Load known targets
                if let Ok(targets) = heed.get_targets_async().await {
                    self.known_targets = targets;
                }
            }
            Err(e) => tracing::error!(target: "log_worker", "Failed to load initial logs: {e}"),
        }
    }

    pub async fn reload_profiles(&mut self) {
        match self.db.get_all_profiles_with_details().await {
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
                self.log_trace("error", "tui", &format!("Failed to load profiles: {e}"));
                self.profiles.clear();
            }
        }
        self.filter_cache_valid.set(false);
    }

    pub async fn reload_groups(&mut self) {
        match self.db.get_all_groups().await {
            Ok(groups) => self.groups = groups,
            Err(e) => {
                self.log_trace("error", "tui", &format!("Failed to load groups: {e}"));
                self.groups.clear();
            }
        }
    }
    pub async fn reload_routing_rules(&mut self) {
        self.routing_rules = self.db.get_all_routing_rules().await.unwrap_or_default();
    }

    pub fn filtered_profiles(&self) -> impl Iterator<Item = &ProfileRow> {
        if !self.filter_cache_valid.get() {
            let indices = self.compute_filtered_indices();
            *self.cached_filtered_indices.borrow_mut() = indices;
            self.filter_cache_valid.set(true);
        }
        // Clone indices (Vec<usize>) to drop the RefCell borrow, then map to ProfileRow refs
        let indices: Vec<usize> = self.cached_filtered_indices.borrow().clone();
        indices.into_iter().map(move |i| &self.profiles[i])
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
                // When viewing All, skip graveyard profiles and mirror rows
                if self.selected_group_id.is_none()
                    && (row.profile.group_id.as_deref() == Some(GRAVEYARD_GROUP_ID)
                        || row.profile.group_id.as_deref() == Some(ALL_GROUP_ID))
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
                SortColumn::ConfigType => a_row.profile.config_type.cmp(&b_row.profile.config_type),
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
                        .map_or(0, |s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0));
                    let tb = b_row
                        .stats
                        .as_ref()
                        .map_or(0, |s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0));
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

    pub fn cycle_group(&mut self, dir: i8) {
        if self.groups.is_empty() {
            return;
        }
        let current_idx = self
            .selected_group_id
            .as_ref()
            .and_then(|id| self.groups.iter().position(|g| g.id == *id));
        let len = self.groups.len();
        let skip_graveyard = |idx: usize| -> bool {
            self.groups
                .get(idx)
                .is_some_and(|g| g.id == *GRAVEYARD_GROUP_ID)
        };
        let new_idx = if let Some(idx) = current_idx {
            let mut next = (idx as isize + dir as isize).rem_euclid(len as isize) as usize;
            // Skip past graveyard
            for _ in 0..len {
                if !skip_graveyard(next) {
                    break;
                }
                next = (next as isize + dir as isize).rem_euclid(len as isize) as usize;
            }
            next
        } else {
            // No group selected → start at first non-graveyard group
            self.groups
                .iter()
                .position(|g| g.id != *GRAVEYARD_GROUP_ID)
                .unwrap_or(0)
        };
        self.selected_group_id = Some(self.groups[new_idx].id.clone());
        self.filter_cache_valid.set(false);
    }
    pub fn add_log(
        &mut self,
        level: String,
        target: String,
        message: String,
        timestamp_nanos: i64,
    ) {
        self.log_cache.push_back(LogLine {
            level,
            target,
            message,
            timestamp_nanos,
        });
        // Keep view stable when scrolled up: adjust scroll by 1 so offset
        // doesn't shift. When at bottom (scroll == 0), stay at bottom.
        if self.log_scroll != 0 {
            self.log_scroll = self.log_scroll.saturating_add(1);
        }
        if self.log_cache.len() > 10000 {
            let excess = self.log_cache.len() - 10000;
            self.log_cache.drain(0..excess);
            self.log_scroll = self.log_scroll.saturating_sub(excess);
        }
    }
    /// Log to the TUI log buffer AND emit a tracing event (target "tui").
    pub fn log_trace(&mut self, level: &str, _target: &str, message: &str) {
        match level {
            "info" => tracing::info!(target: "tui", "{message}"),
            "error" => tracing::error!(target: "tui", "{message}"),
            "warn" | "warning" => tracing::warn!(target: "tui", "{message}"),
            _ => tracing::info!(target: "tui", "{message}"),
        }
        self.add_log(
            level.to_owned(),
            "tui".to_owned(),
            message.to_owned(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64,
        );
    }
    /// Resolve which core a profile row should use, considering (in order):
    /// 1. Per-profile override (`row.profile.core_type`)
    /// 2. Per-protocol config override (`config.core.protocol_core_overrides`)
    /// 3. Hardcoded auto-detection (`core_for_protocol` via `resolve_core`)
    pub fn resolved_core(&self, row: &ProfileRow) -> CoreType {
        let protocol = Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
        let profile_override = row.profile.core_type.parse::<CoreType>().ok();
        let config_override = self
            .config
            .core
            .protocol_core_overrides
            .get(&protocol.to_string())
            .and_then(|s| s.parse::<CoreType>().ok());
        resolve_core(protocol, config_override.or(profile_override))
    }
    // ── CRUD operations ──────────────────────────────────────────────────

    pub fn start_add_server(&mut self) {
        let fields = common_field_defaults();
        self.mode = AppMode::AddServer {
            protocol: None,
            fields,
            focus_index: 0,
            form_errors: HashMap::new(),
        };
    }
    pub async fn start_edit_profile(&mut self, id: &str) {
        match self.db.get_profile(id).await {
            Ok(Some(profile)) => {
                let fields = profile_to_fields(&profile);
                self.mode = AppMode::EditServer {
                    profile_id: id.to_string(),
                    fields,
                    focus_index: 0,
                    form_errors: HashMap::new(),
                };
            }
            Ok(None) => self.log_trace("error", "tui", &format!("Profile {id} not found")),
            Err(e) => self.log_trace("error", "tui", &format!("Failed to load profile: {e}")),
        }
    }

    #[allow(dead_code)]
    fn selected_profile(&self) -> Option<&Profile> {
        self.filtered_profiles()
            .nth(self.selected_index)
            .map(|r| &r.profile)
    }

    fn selected_profile_id(&self) -> Option<String> {
        self.filtered_profiles()
            .nth(self.selected_index)
            .map(|r| r.profile.id.clone())
    }

    fn fields_to_profile(protocol: Protocol, fields: &[(String, String)]) -> Profile {
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
                "remarks" => {
                    profile.remarks = Some(xray_tui_config::import_export::normalize_remark(value));
                }
                "address" => profile.address = Some(value.clone()),
                "port" => profile.port = value.parse::<i32>().ok(),
                "core_type" => profile.core_type.clone_from(value),
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

    pub async fn confirm_add_server(&mut self) {
        // Phase 1: validate using immutable borrow (extract only needed values)
        let (protocol, address, port, user_id) = {
            let (p, fields) = if let AppMode::AddServer {
                protocol: Some(p),
                fields,
                ..
            } = &self.mode
            {
                (*p, fields)
            } else {
                self.log_trace("error", "tui", "Cannot confirm: no protocol selected");
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

        // Validate fields
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
            } = self.mode
            {
                *form_errors = errors;
            }
            return;
        }

        // Phase 2: take ownership of fields (only on success)
        let fields = match &mut self.mode {
            AppMode::AddServer { fields, .. } => std::mem::take(fields),
            _ => unreachable!(),
        };

        let mut profile = Self::fields_to_profile(protocol, &fields);
        // Assign to currently selected real group (not All or Graveyard)
        if let Some(gid) = &self.selected_group_id
            && gid != ALL_GROUP_ID
            && gid != GRAVEYARD_GROUP_ID
        {
            profile.group_id = Some(gid.clone());
        }
        match self.db.insert_profile(&profile).await {
            Ok(()) => {
                self.log_trace(
                    "info",
                    "tui",
                    &format!(
                        "Added server: {}",
                        profile.remarks.as_deref().unwrap_or("unnamed")
                    ),
                );
                self.mode = AppMode::List;
                self.reload_profiles().await;
            }
            Err(e) => {
                self.log_trace("error", "tui", &format!("Failed to add server: {e}"));
                // Restore fields so the form isn't empty
                if let AppMode::AddServer {
                    fields: ref mut f, ..
                } = self.mode
                {
                    *f = fields;
                }
            }
        }
    }

    pub async fn confirm_edit_server(&mut self) {
        // Phase 1: validate using immutable borrow
        let (profile_id, address, port, user_id) = {
            let (pid, fields) = match &self.mode {
                AppMode::EditServer {
                    profile_id, fields, ..
                } => (profile_id.clone(), fields),
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
            (pid, addr.to_owned(), prt.to_owned(), uid.to_owned())
        };

        // Validate fields
        let mut errors: HashMap<String, String> = HashMap::new();
        if address.is_empty() {
            errors.insert("address".into(), "Address is required".into());
        }
        if port.is_empty() || port.parse::<u16>().map_or(true, |p| p == 0) {
            errors.insert("port".into(), "Port must be 1-65535".into());
        }
        // Infer protocol from existing profile
        let protocol = Protocol::try_from_i32(
            self.db
                .get_profile(&profile_id)
                .await
                .ok()
                .flatten()
                .map_or(0, |p| p.config_type),
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
            } = self.mode
            {
                *form_errors = errors;
            }
            return;
        }

        // Phase 2: take ownership of fields (only on success)
        let fields = match &mut self.mode {
            AppMode::EditServer { fields, .. } => std::mem::take(fields),
            _ => unreachable!(),
        };

        let mut profile = if let Ok(Some(p)) = self.db.get_profile(&profile_id).await {
            p
        } else {
            self.log_trace("error", "tui", "Profile not found for edit");
            return;
        };
        // Rebuild from form fields
        let new_profile = Self::fields_to_profile(protocol, &fields);
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
        match self.db.update_profile(&profile).await {
            Ok(()) => {
                self.log_trace(
                    "info",
                    "tui",
                    &format!(
                        "Updated server: {}",
                        profile.remarks.as_deref().unwrap_or("unnamed")
                    ),
                );
                self.mode = AppMode::List;
                self.reload_profiles().await;
            }
            Err(e) => {
                self.log_trace("error", "tui", &format!("Failed to update server: {e}"));
                // Restore fields so the form isn't empty
                if let AppMode::EditServer {
                    fields: ref mut f, ..
                } = self.mode
                {
                    *f = fields;
                }
            }
        }
    }

    pub fn cancel_form(&mut self) {
        self.mode = AppMode::List;
    }

    // ── Settings helpers ──────────────────────────────────────────────────

    pub fn enter_settings(&mut self) {
        self.mode = AppMode::Settings {
            mode: SettingsMode::Split {
                tree: RefCell::new(TreeState::all_expanded(5)),
                focus: SplitFocus::Tree,
                right: SplitRightPane::Empty,
            },
        };
    }

    async fn build_settings_fields(&self, section: SettingsSection) -> Vec<(String, String)> {
        use crate::SettingsSection::{
            Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
            SystemProxy, Tun, Updates,
        };
        match section {
            Core => {
                vec![
                    (
                        "xray_path".into(),
                        self.config.core.xray_path.clone().unwrap_or_default(),
                    ),
                    (
                        "sing_box_path".into(),
                        self.config.core.sing_box_path.clone().unwrap_or_default(),
                    ),
                    (
                        "default_core".into(),
                        self.config
                            .core
                            .core_type
                            .as_ref()
                            .map_or_else(|| "Auto".into(), |ct| format!("{ct:?}")),
                    ),
                    ("log_level".into(), self.config.core.log_level.clone()),
                ]
            }
            Gui => {
                vec![
                    ("language".into(), self.config.gui.language.clone()),
                    (
                        "theme".into(),
                        self.config.gui.theme.clone().unwrap_or_default(),
                    ),
                    (
                        "refresh_interval".into(),
                        humantime::format_duration(*self.config.gui.refresh_interval_secs)
                            .to_string(),
                    ),
                ]
            }
            Inbound => {
                vec![
                    (
                        "socks_port".into(),
                        self.config.inbound.socks_port.to_string(),
                    ),
                    (
                        "http_port".into(),
                        self.config
                            .inbound
                            .http_port
                            .map(|p| p.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "mixed_port".into(),
                        self.config
                            .inbound
                            .mixed_port
                            .map(|p| p.to_string())
                            .unwrap_or_default(),
                    ),
                    ("listen".into(), self.config.inbound.listen.clone()),
                    (
                        "sniffing".into(),
                        if self.config.inbound.sniffing {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ),
                ]
            }
            Dns => {
                if let Ok(Some(dns)) = self.db.get_dns_settings().await {
                    vec![
                        ("servers".into(), dns.servers.unwrap_or_default()),
                        ("hosts".into(), dns.hosts.unwrap_or_default()),
                        (
                            "query_strategy".into(),
                            dns.query_strategy.unwrap_or_default(),
                        ),
                        (
                            "disable_cache".into(),
                            if dns.disable_cache.unwrap_or(0) != 0 {
                                "true".into()
                            } else {
                                "false".into()
                            },
                        ),
                        (
                            "disable_fallback".into(),
                            if dns.disable_fallback.unwrap_or(0) != 0 {
                                "true".into()
                            } else {
                                "false".into()
                            },
                        ),
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
                    (
                        "enabled".into(),
                        if self.config.system_proxy.enabled {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ),
                    (
                        "http_port".into(),
                        self.config
                            .system_proxy
                            .http_port
                            .map(|p| p.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "socks_port".into(),
                        self.config
                            .system_proxy
                            .socks_port
                            .map(|p| p.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "bypass".into(),
                        self.config.system_proxy.bypass.clone().unwrap_or_default(),
                    ),
                ]
            }
            Tun => {
                vec![
                    (
                        "enabled".into(),
                        if self.config.tun.enabled {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ),
                    (
                        "interface_name".into(),
                        self.config.tun.interface_name.clone().unwrap_or_default(),
                    ),
                    (
                        "mtu".into(),
                        self.config
                            .tun
                            .mtu
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                ]
            }
            Mux => {
                vec![
                    (
                        "enabled".into(),
                        if self.config.mux.enabled {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ),
                    (
                        "concurrency".into(),
                        self.config
                            .mux
                            .concurrency
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "fragment_enabled".into(),
                        if self.config.mux.fragment_enabled {
                            "true".into()
                        } else {
                            "false".into()
                        },
                    ),
                    (
                        "fragment_packets".into(),
                        self.config.mux.fragment_packets.clone().unwrap_or_default(),
                    ),
                    (
                        "fragment_length".into(),
                        self.config.mux.fragment_length.clone().unwrap_or_default(),
                    ),
                    (
                        "fragment_interval".into(),
                        self.config
                            .mux
                            .fragment_interval
                            .clone()
                            .unwrap_or_default(),
                    ),
                ]
            }
            Stats => {
                vec![(
                    "enabled".into(),
                    if self.config.statistics.enabled {
                        "true".into()
                    } else {
                        "false".into()
                    },
                )]
            }
            Routing | Updates => {
                vec![]
            }
            ProtocolCore => PROTOCOL_CORE_DEFS
                .iter()
                .map(|(key, _label, _)| {
                    let val = self
                        .config
                        .core
                        .protocol_core_overrides
                        .get(*key)
                        .cloned()
                        .unwrap_or_else(|| "Auto".to_string());
                    (key.to_string(), val)
                })
                .collect(),
            SpeedTest => {
                vec![
                    ("ping_url".into(), self.config.speed_test.ping_url.clone()),
                    (
                        "ip_api_url".into(),
                        self.config.speed_test.ip_api_url.clone(),
                    ),
                    (
                        "tcp_timeout_secs".into(),
                        humantime::format_duration(*self.config.speed_test.tcp_timeout_secs)
                            .to_string(),
                    ),
                    (
                        "real_ping_timeout_secs".into(),
                        humantime::format_duration(*self.config.speed_test.real_ping_timeout_secs)
                            .to_string(),
                    ),
                    (
                        "batch_page_size".into(),
                        self.config.speed_test.batch_page_size.to_string(),
                    ),
                    (
                        "real_ping_retries".into(),
                        self.config.speed_test.real_ping_retries.to_string(),
                    ),
                    (
                        "real_ping_concurrency".into(),
                        self.config.speed_test.real_ping_concurrency.to_string(),
                    ),
                    (
                        "tcp_ping_concurrency".into(),
                        self.config.speed_test.tcp_ping_concurrency.to_string(),
                    ),
                ]
            }
            Logging => {
                vec![(
                    "log_ttl_secs".into(),
                    humantime::format_duration(*self.config.logging.ttl_secs).to_string(),
                )]
            }
        }
    }

    fn apply_settings_fields(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        use crate::SettingsSection::{
            Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
            SystemProxy, Tun, Updates,
        };
        let get_str = |key: &str| -> &str {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or("", |(_, v)| v.as_str())
        };
        let get = |key: &str| get_str(key).to_owned();
        let get_opt = |key: &str| {
            let v = get_str(key);
            if v.is_empty() {
                None
            } else {
                Some(v.to_owned())
            }
        };
        match section {
            Core => {
                self.config.core.xray_path = get_opt("xray_path");
                self.config.core.sing_box_path = get_opt("sing_box_path");
                let core_str = get_str("default_core");
                self.config.core.core_type = if core_str.is_empty() || core_str == "Auto" {
                    None
                } else {
                    core_str.parse::<xray_tui_core::CoreType>().ok()
                };
                if !get_str("log_level").is_empty() {
                    self.config.core.log_level = get("log_level");
                }
            }
            Gui => {
                self.config.gui.language = get("language");
                self.config.gui.theme = get_opt("theme");
                if let Ok(d) = humantime::parse_duration(get_str("refresh_interval")) {
                    *self.config.gui.refresh_interval_secs = d;
                }
            }
            Inbound => {
                if let Ok(v) = get_str("socks_port").parse::<u16>() {
                    self.config.inbound.socks_port = v;
                }
                self.config.inbound.http_port = get_str("http_port").parse::<u16>().ok();
                self.config.inbound.mixed_port = get_str("mixed_port").parse::<u16>().ok();
                if !get_str("listen").is_empty() {
                    self.config.inbound.listen = get("listen");
                }
                self.config.inbound.sniffing = get_str("sniffing") == "true";
            }
            SystemProxy => {
                self.config.system_proxy.enabled = get_str("enabled") == "true";
                self.config.system_proxy.http_port = get_str("http_port").parse::<u16>().ok();
                self.config.system_proxy.socks_port = get_str("socks_port").parse::<u16>().ok();
                self.config.system_proxy.bypass = get_opt("bypass");
            }
            Tun => {
                self.config.tun.enabled = get_str("enabled") == "true";
                self.config.tun.interface_name = get_opt("interface_name");
                self.config.tun.mtu = get_str("mtu").parse::<u16>().ok();
            }
            Mux => {
                self.config.mux.enabled = get_str("enabled") == "true";
                self.config.mux.concurrency = get_str("concurrency").parse::<u8>().ok();
                self.config.mux.fragment_enabled = get_str("fragment_enabled") == "true";
                self.config.mux.fragment_packets = get_opt("fragment_packets");
                self.config.mux.fragment_length = get_opt("fragment_length");
                self.config.mux.fragment_interval = get_opt("fragment_interval");
            }
            Stats => {
                self.config.statistics.enabled = get_str("enabled") == "true";
            }
            ProtocolCore => {
                for (key, val) in fields {
                    if val == "Auto" {
                        self.config
                            .core
                            .protocol_core_overrides
                            .remove(key.as_str());
                    } else {
                        self.config
                            .core
                            .protocol_core_overrides
                            .insert(key.clone(), val.clone());
                    }
                }
            }
            SpeedTest => {
                if !get_str("ping_url").is_empty() {
                    self.config.speed_test.ping_url = get("ping_url");
                }
                if !get_str("ip_api_url").is_empty() {
                    self.config.speed_test.ip_api_url = get("ip_api_url");
                }
                if let Ok(d) = humantime::parse_duration(get_str("tcp_timeout_secs")) {
                    *self.config.speed_test.tcp_timeout_secs = d;
                }
                if let Ok(d) = humantime::parse_duration(get_str("real_ping_timeout_secs")) {
                    *self.config.speed_test.real_ping_timeout_secs = d;
                }
                if let Ok(v) = get_str("batch_page_size").parse::<usize>() {
                    self.config.speed_test.batch_page_size = v;
                }
                if let Ok(v) = get_str("real_ping_retries").parse::<u32>() {
                    self.config.speed_test.real_ping_retries = v;
                }
                if let Ok(v) = get_str("real_ping_concurrency").parse::<usize>() {
                    self.config.speed_test.real_ping_concurrency = v;
                }
                if let Ok(v) = get_str("tcp_ping_concurrency").parse::<usize>() {
                    self.config.speed_test.tcp_ping_concurrency = v.max(1);
                }
            }
            // Dns and Routing are handled separately (DB-backed)
            Dns | Routing | Updates => {}
            Logging => {
                if let Ok(d) = humantime::parse_duration(get_str("log_ttl_secs")) {
                    *self.config.logging.ttl_secs = d;
                }
            }
        }
    }

    pub async fn build_right_pane(&mut self, section: SettingsSection) -> SplitRightPane {
        let fields = self.build_settings_fields(section).await;
        match section {
            SettingsSection::Routing => {
                self.reload_routing_rules().await;
                SplitRightPane::RoutingList { selected: 0 }
            }
            SettingsSection::Updates => SplitRightPane::UpdateForm {
                status_xray: self
                    .update_status
                    .get(&CoreType::Xray)
                    .cloned()
                    .unwrap_or_default(),
                status_singbox: self
                    .update_status
                    .get(&CoreType::SingBox)
                    .cloned()
                    .unwrap_or_default(),
            },
            _ => SplitRightPane::Form {
                section,
                fields,
                focus_index: 0,
                form_errors: HashMap::new(),
            },
        }
    }

    pub fn save_settings_form(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        self.apply_settings_fields(section, fields);
        if let Err(e) = self.config.save() {
            self.log_trace("error", "tui", &format!("Failed to save config: {e}"));
        } else {
            self.log_trace("info", "tui", "Settings saved");
        }
    }

    pub async fn save_routing_rule(
        &mut self,
        rule_id: Option<String>,
        fields: &[(String, String)],
    ) {
        let id = rule_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let get_str = |key: &str| -> &str {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or("", |(_, v)| v.as_str())
        };
        let get_opt = |key: &str| {
            let v = get_str(key);
            if v.is_empty() {
                None
            } else {
                Some(v.to_owned())
            }
        };
        let rule = RoutingRule {
            id,
            group_id: None,
            r#type: get_str("type").parse::<i32>().unwrap_or(0),
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
            self.db.update_routing_rule(&rule).await
        } else {
            self.db.insert_routing_rule(&rule).await
        };
        match result {
            Ok(()) => self.log_trace("info", "tui", "Routing rule saved"),
            Err(e) => self.log_trace("error", "tui", &format!("Failed to save routing rule: {e}")),
        }
        self.reload_routing_rules().await;
    }

    pub async fn save_dns_settings(&mut self, fields: &[(String, String)]) {
        let id = self
            .db
            .get_dns_settings()
            .await
            .ok()
            .flatten()
            .map_or_else(|| uuid::Uuid::new_v4().to_string(), |d| d.id);
        let get_str = |key: &str| -> &str {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or("", |(_, v)| v.as_str())
        };
        let get_opt = |key: &str| {
            let v = get_str(key);
            if v.is_empty() {
                None
            } else {
                Some(v.to_owned())
            }
        };
        let dns = DnsSetting {
            id,
            name: None,
            servers: get_opt("servers"),
            hosts: get_opt("hosts"),
            query_strategy: get_opt("query_strategy"),
            disable_cache: Some(i32::from(get_str("disable_cache") == "true")),
            disable_fallback: Some(i32::from(get_str("disable_fallback") == "true")),
            client_ip: get_opt("client_ip"),
        };
        match self.db.upsert_dns_settings(&dns).await {
            Ok(()) => self.log_trace("info", "tui", "DNS settings saved"),
            Err(e) => self.log_trace("error", "tui", &format!("Failed to save DNS settings: {e}")),
        }
    }

    pub async fn delete_profile(&mut self, id: &str) {
        if let Err(e) = self.db.delete_profile(id).await {
            self.log_trace("error", "tui", &format!("Failed to delete profile: {e}"));
            return;
        }
        self.log_trace("info", "tui", "Profile deleted");
        self.confirmation = None;
        self.multi_select.remove(id);
        self.reload_profiles().await;
    }

    pub async fn clone_profile(&mut self, id: &str) {
        let new_id = uuid::Uuid::new_v4().to_string();
        if let Err(e) = self.db.clone_profile(id, &new_id).await {
            self.log_trace("error", "tui", &format!("Failed to clone profile: {e}"));
            return;
        }
        self.log_trace("info", "tui", "Profile cloned");
        self.reload_profiles().await;
    }

    pub fn toggle_multi_select(&mut self, id: &str) {
        if !self.multi_select.insert(id.to_string()) {
            self.multi_select.remove(id);
        }
    }

    pub fn import_url(&mut self, url: &str) {
        let settings = ValidationSettings::from(self.config.parsing.clone());
        match xray_tui_config::import_export::parse_share_url(url, &settings) {
            Ok(profile) => {
                let protocol =
                    Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
                let fields = profile_to_fields(&profile);
                self.mode = AppMode::AddServer {
                    protocol: Some(protocol),
                    fields,
                    focus_index: 0,
                    form_errors: HashMap::new(),
                };
                self.log_trace("info", "tui", "URL imported successfully");
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
        let settings = ValidationSettings::from(self.config.parsing.clone());
        let results: Vec<BatchImportItem> = urls
            .iter()
            .map(
                |url| match xray_tui_config::import_export::parse_share_url(url, &settings) {
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
                },
            )
            .collect();
        self.mode = AppMode::BatchImport { results, scroll: 0 };
    }

    pub async fn confirm_batch_import(&mut self) {
        let items = match &mut self.mode {
            AppMode::BatchImport { results, .. } => std::mem::take(results),
            _ => return,
        };
        let now = format_now();
        let mut imported = 0usize;
        let mut errors = 0usize;
        for item in items {
            if let Some(mut profile) = item.profile {
                // Assign to currently selected real group (not All or Graveyard)
                if let Some(gid) = &self.selected_group_id
                    && gid != ALL_GROUP_ID
                    && gid != GRAVEYARD_GROUP_ID
                {
                    profile.group_id = Some(gid.clone());
                }
                profile.sub_uid = Some(profile.compute_sub_uid() as i64);
                if profile.created_at.is_none() {
                    profile.created_at = Some(now.clone());
                }
                if profile.updated_at.is_none() {
                    profile.updated_at = Some(now.clone());
                }
                if self.db.insert_profile(&profile).await.is_ok() {
                    imported += 1;
                } else {
                    errors += 1;
                }
            }
        }
        self.log_trace(
            "info",
            "tui",
            &format!("Batch import: {imported} imported, {errors} errors"),
        );
        self.mode = AppMode::List;
        self.reload_profiles().await;
    }

    pub async fn move_profile_up(&mut self) {
        let id = match self.selected_profile_id() {
            Some(id) => id,
            None => return,
        };
        let filtered: Vec<&ProfileRow> = self.filtered_profiles().collect();
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
            .await
        {
            self.log_trace("error", "tui", &format!("Failed to reorder: {e}"));
        }
        self.reload_profiles().await;
    }

    pub async fn move_profile_down(&mut self) {
        let id = match self.selected_profile_id() {
            Some(id) => id,
            None => return,
        };
        let filtered: Vec<&ProfileRow> = self.filtered_profiles().collect();
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
            .await
        {
            self.log_trace("error", "tui", &format!("Failed to reorder: {e}"));
        }
        self.reload_profiles().await;
    }

    pub async fn set_active(&mut self, id: &str) {
        if let Err(e) = self.db.update_profile_active(id).await {
            self.log_trace("error", "tui", &format!("Failed to set active: {e}"));
            return;
        }
        self.reload_profiles().await;
    }

    // ── Core connection management ──────────────────────────────────────

    pub fn connect_to_profile(&mut self, profile_id: &str) {
        if self.connecting {
            return;
        }

        let profile = if let Some(r) = self.profiles.iter().find(|r| r.profile.id == profile_id) {
            r.profile.clone()
        } else {
            self.log_trace("error", "tui", "Profile not found for connection");
            return;
        };

        let protocol = if let Some(p) = Protocol::try_from_i32(profile.config_type) {
            p
        } else {
            self.log_trace(
                "error",
                "tui",
                &format!("Unknown protocol: {}", profile.config_type),
            );
            return;
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
        self.connected_profile_id = Some(profile_id.to_string());
        self.connection_error = None;

        let tx = if let Some(tx) = &self.core_event_tx {
            tx.clone()
        } else {
            self.connecting = false;
            self.log_trace("error", "tui", "Core event channel not initialized");
            return;
        };

        let params = BuildParams {
            v2ray_api_enabled: matches!(core_type, CoreType::Xray),
            clash_api_enabled: matches!(core_type, CoreType::SingBox),
            log_level: self.config.core.log_level.clone(),
            socks_port: self.config.inbound.socks_port,
            http_port: self.config.inbound.http_port,
            listen: self.config.inbound.listen.clone(),
            sniffing: self.config.inbound.sniffing,
            clash_api_port: self.config.clash_api_port,
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

        // Create log forwarding channel
        let (log_line_tx, mut log_line_rx) = mpsc::channel::<String>(512);
        let state_log_sender = self.log_sender_tx.clone();
        let handle = tokio::spawn(async move {
            // 1. Build config
            let backend_config =
                match ConfigBuilder::build(&profile, core_type, &params, &routing, &dns) {
                    Ok(c) => c,
                    Err(e) => {
                        try_send_or_warn(
                            &tx,
                            CoreEvent::Error(format!("Config build failed: {e}")),
                            "config_build_error",
                        );
                        return;
                    }
                };

            // 2. Find binary
            let bin_path = if let Some(p) = find_binary(core_type, &bin_dir) {
                p
            } else {
                try_send_or_warn(&tx, CoreEvent::Error(
                    "Core binary not found. Place it in ~/.config/xray-tui/bin/ or install in PATH."
                        .to_string(),
                ), "binary_not_found");
                return;
            };

            // 3. Start core
            let mut manager = CoreManager::with_log_channel(bin_configs_dir, log_line_tx);
            if let Err(e) = manager.start(core_type, &backend_config, &bin_path).await {
                try_send_or_warn(
                    &tx,
                    CoreEvent::Error(format!("Failed to start core: {e}")),
                    "core_start_error",
                );
                return;
            }

            // 3b. Readiness probe: wait for the core's proxy port to open
            let proxy_port = params.socks_port;
            if proxy_port > 0 {
                let probe_start = std::time::Instant::now();
                loop {
                    if probe_start.elapsed() > std::time::Duration::from_secs(10) {
                        break;
                    }
                    match tokio::net::TcpStream::connect(("127.0.0.1", proxy_port)).await {
                        Ok(_) => break,
                        Err(_) => tokio::time::sleep(std::time::Duration::from_millis(200)).await,
                    }
                }
            }

            // 4. Signal connected
            try_send_or_warn(&tx, CoreEvent::Connected(core_type), "connected");

            // Forward stderr log lines as CoreEvent::LogLine
            let log_tx = tx.clone();
            let log_sender = state_log_sender.clone();
            tokio::spawn(async move {
                while let Some(line) = log_line_rx.recv().await {
                    let (level, target, message, ts_nanos) = parse_core_log_line(&line, core_type);
                    let timestamp_nanos = ts_nanos.unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as i64
                    });
                    // Write to log storage via non-blocking channel (batched async writer)
                    if let Some(ref sender) = log_sender {
                        let _ = sender.send(xray_tui_core::log_heed::LogMessage {
                            level: level.clone(),
                            target: target.clone(),
                            message: message.clone(),
                            timestamp_nanos: timestamp_nanos as u64,
                        });
                    }
                    // Forward to TUI
                    try_send_or_warn(
                        &log_tx,
                        CoreEvent::LogLine {
                            level,
                            target,
                            message,
                            timestamp_nanos,
                        },
                        "log_line",
                    );
                }
            });

            let profile_id = profile.id.clone();

            if core_type == CoreType::Xray {
                // === gRPC polling loop (xray-core) ===
                let provider = match grpc_client::create_stats_provider(CoreType::Xray).await {
                    Ok(p) => Some(p),
                    Err(e) => {
                        try_send_or_warn(
                            &tx,
                            CoreEvent::StatsError(format!("Stats API unavailable: {e}")),
                            "stats_api_unavailable",
                        );
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
                                        try_send_or_warn(&tx, CoreEvent::StatsUpdate {
                                            profile_id: profile_id.clone(),
                                            today_up,
                                            today_down,
                                            total_up: today_up,
                                            total_down: today_down,
                                        }, "stats_update");
                                    }
                                    Err(e) => {
                                    try_send_or_warn(&tx, CoreEvent::StatsError(format!("Stats query failed: {e}")), "stats_query_err");
                                    }
                                }
                                // sys stats every 3rd tick (~9s)
                                sys_tick_counter += 1;
                                if sys_tick_counter >= 3 {
                                    sys_tick_counter = 0;
                                    match provider.get_sys_stats().await {
                                        Ok(sys) => { try_send_or_warn(&tx, CoreEvent::SysStatsUpdate(sys), "sys_stats_update"); }
                                        Err(e) => {
                                            try_send_or_warn(&tx, CoreEvent::StatsError(format!("Sys stats query failed: {e}")), "sys_stats_err");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // === Sing-box Clash API /traffic streaming ===
                let clash_port = params.clash_api_port.unwrap_or(CLASH_API_PORT);
                let url = format!("http://127.0.0.1:{clash_port}/traffic");

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
                                                    try_send_or_warn(&tx, CoreEvent::StatsUpdate {
                                                        profile_id: profile_id.clone(),
                                                        today_up: session_up,
                                                        today_down: session_down,
                                                        total_up: session_up,
                                                        total_down: session_down,
                                                    }, "clash_stats_update");
                                                }
                                            }
                                        }
                                        Some(Err(e)) => {
                                            try_send_or_warn(&tx, CoreEvent::StatsError(
                                                format!("Clash API stream error: {e}")
                                            ), "clash_stream_err");
                                            break;
                                        }
                                        None => break,
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        try_send_or_warn(
                            &tx,
                            CoreEvent::StatsError(format!(
                                "Clash API unavailable (is sing-box running?): {e}"
                            )),
                            "clash_unavailable",
                        );
                    }
                }
            }

            // 6. Stop core
            let _ = manager.stop().await;

            // 7. Signal disconnected
            try_send_or_warn(&tx, CoreEvent::Disconnected, "disconnected");
        });
        self.core_task_handle = Some(handle);
    }

    /// Disconnect the currently running core.
    pub fn disconnect(&mut self) {
        if let Some(tx) = self.disconnect_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.core_task_handle.take() {
            handle.abort();
        }
        self.connected_core = None;
        self.connected_profile_id = None;
        self.connecting = false;
        self.log_trace("info", "core", "Disconnected");
    }

    // ── Speed test methods ────────────────────────────────────────

    /// Start TCP ping on the given profile. Returns immediately; result arrives via `CoreEvent`.
    pub fn start_tcp_ping(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            self.log_trace("warn", "tui", "Test already in progress for this profile");
            return;
        }

        // Find the profile and extract address:port
        let row = if let Some(r) = self.profiles.iter().find(|r| r.profile.id == profile_id) {
            r
        } else {
            self.log_trace("error", "tui", "Profile not found for TCP ping");
            return;
        };
        let addr = if let Some(a) = &row.profile.address {
            a.clone()
        } else {
            self.log_trace("error", "tui", "Profile has no address");
            return;
        };
        let port = match row.profile.port {
            Some(p) if p > 0 && p <= 65535 => p as u16,
            _ => {
                self.log_trace("error", "tui", "Profile has invalid port");
                return;
            }
        };

        let tx = if let Some(tx) = &self.core_event_tx {
            tx.clone()
        } else {
            self.log_trace("error", "tui", "Core event channel not initialized");
            return;
        };

        let pid = profile_id.to_string();
        self.testing_details.insert(pid.clone(), TestType::TcpPing);
        self.testing_profiles.insert(pid.clone());
        let config_type = row.profile.config_type;
        let timeout_dur = *self.config.speed_test.tcp_timeout_secs;

        tokio::spawn(async move {
            let fmgr = xray_tui_core::FastPingManager::new(timeout_dur);
            let result = fmgr.ping(config_type, &addr, port).await;
            let (latency_ms, error) = match result {
                Ok(dur) => (Some(dur.as_millis() as u64), None),
                Err(e) => (None, Some(e.to_string())),
            };
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    profile_id: pid,
                    test_type: TestType::TcpPing,
                    latency_ms,
                    speed_bps: None,
                    ip_info: None,
                    error,
                },
                "tcp_ping_result",
            );
        });
    }

    /// Start real ping (HTTP through proxy) by starting a temporary core for the profile.
    pub fn start_real_ping(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }

        // Find profile row and resolve core
        let row = if let Some(r) = self.profiles.iter().find(|r| r.profile.id == profile_id) {
            r
        } else {
            self.log_trace("error", "tui", "Profile not found for real ping");
            return;
        };
        let profile = row.profile.clone();
        let protocol = Protocol::try_from_i32(profile.config_type).unwrap_or(Protocol::Custom);
        let core_override = self
            .config
            .core
            .protocol_core_overrides
            .get(protocol.to_string().as_str())
            .and_then(|s| s.parse::<CoreType>().ok());
        let core_type = resolve_core(protocol, core_override);

        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_details.insert(pid.clone(), TestType::RealPing);
        self.testing_profiles.insert(pid.clone());

        // Build params for the temp core
        let params = BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: 0, // overridden by temp allocation
            http_port: None,
            listen: self.config.inbound.listen.clone(),
            sniffing: false,
            clash_api_port: None,
        };

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

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join("xray-tui");
        let bin_dir = config_dir.join("bin");
        let bin_configs_dir = config_dir.join("binConfigs");
        let proxy_addr = self.config.inbound.listen.clone();
        let proxy_port = self.config.inbound.socks_port;
        let ping_url = self.config.speed_test.ping_url.clone();
        let ip_api_url = self.config.speed_test.ip_api_url.clone();
        let timeout_dur = *self.config.speed_test.real_ping_timeout_secs;
        let retries = self.config.speed_test.real_ping_retries;

        tokio::spawn(async move {
            // 1. Create temp config directory
            let temp_id = uuid::Uuid::new_v4().to_string();
            let temp_dir = bin_configs_dir.join(&temp_id);
            if let Err(e) = tokio::fs::create_dir_all(&temp_dir).await {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        profile_id: pid,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(format!("Failed to create temp dir: {e}")),
                    },
                    "real_ping_tempdir_err",
                );
                return;
            }

            // 2. Build config
            let backend_config = match ConfigBuilder::build(&profile, core_type, &params, &[], &dns)
            {
                Ok(c) => c,
                Err(e) => {
                    try_send_or_warn(
                        &tx,
                        CoreEvent::SpeedTestResult {
                            profile_id: pid,
                            test_type: TestType::RealPing,
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: Some(format!("Config build failed: {e}")),
                        },
                        "real_ping_config_err",
                    );
                    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                    return;
                }
            };

            // 3. Find binary
            let bin_path = if let Some(p) = find_binary(core_type, &bin_dir) {
                p
            } else {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        profile_id: pid,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some("Core binary not found".to_string()),
                    },
                    "real_ping_binary_err",
                );
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return;
            };

            // 4. Start core (discard log lines from temp core)
            let (log_line_tx, mut _log_line_rx) = mpsc::channel::<String>(512);
            let mut manager = CoreManager::with_log_channel(temp_dir.clone(), log_line_tx);
            if let Err(e) = manager.start(core_type, &backend_config, &bin_path).await {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        profile_id: pid,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(format!("Failed to start core: {e}")),
                    },
                    "real_ping_start_err",
                );
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return;
            }

            // 5. Wait for core readiness
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // 6. Run real ping through the temp core's proxy
            let result = xray_tui_core::speed_test::real_ping(
                &proxy_addr,
                proxy_port,
                &ping_url,
                &ip_api_url,
                timeout_dur,
                retries,
            )
            .await;

            let (latency_ms, ip_info, error) = match result {
                Ok(rp_result) => (Some(rp_result.latency_ms), rp_result.ip_info, None),
                Err(e) => (None, None, Some(e.to_string())),
            };

            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    profile_id: pid,
                    test_type: TestType::RealPing,
                    latency_ms,
                    speed_bps: None,
                    ip_info,
                    error,
                },
                "real_ping_result",
            );

            // 7. Stop core and clean up
            let _ = manager.stop().await;
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        });
    }

    /// Start speed test (download through proxy) on the given profile.
    pub fn start_speed_test(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }
        if self.connected_core.is_none() {
            self.log_trace(
                "warn",
                "tui",
                "Core not connected — proxy required for speed test",
            );
            return;
        }
        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_details.insert(pid.clone(), TestType::SpeedTest);
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
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    profile_id: pid,
                    test_type: TestType::SpeedTest,
                    latency_ms: None,
                    speed_bps,
                    ip_info: None,
                    error,
                },
                "speed_test_result",
            );
        });
    }

    /// Start UDP test on the given profile.
    pub fn start_udp_test(&mut self, profile_id: &str) {
        if self.testing_profiles.contains(profile_id) {
            return;
        }
        if self.connected_core.is_none() {
            self.log_trace(
                "warn",
                "tui",
                "Core not connected — proxy required for UDP test",
            );
            return;
        }
        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let pid = profile_id.to_string();
        self.testing_details.insert(pid.clone(), TestType::UdpTest);
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
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    profile_id: pid,
                    test_type: TestType::UdpTest,
                    latency_ms,
                    speed_bps: None,
                    ip_info: None,
                    error,
                },
                "udp_test_result",
            );
        });
    }

    /// Signal all running batch speed tests to stop.
    pub fn stop_speed_test(&mut self) {
        self.speed_test_stop.store(true, Ordering::Relaxed);
    }

    /// Batch TCP ping all visible (filtered) profiles — delegates to `start_batch_sieve`.
    pub fn start_batch_ping(&mut self) {
        self.start_batch_sieve(false);
    }

    /// Batch TCP ping all visible profiles, then real ping TCP-successful targets via temp core.
    /// Delegates to `start_batch_sieve`.
    pub fn start_batch_then_real_ping(&mut self) {
        self.start_batch_sieve(true);
    }
    /// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
    /// Uses DB-backed `ping_sessions` table for queue management.
    /// Phase 1 drains fast-pingable profiles quickly; Phase 2 handles remaining via temp core.
    #[allow(clippy::needless_collect)]
    pub fn start_batch_sieve(&mut self, real_ping_enabled: bool) {
        let visible: Vec<&ProfileRow> = self.filtered_profiles().collect();
        if visible.is_empty() {
            self.log_trace(
                "info",
                "tui",
                if real_ping_enabled {
                    "No profiles to test"
                } else {
                    "No profiles to ping"
                },
            );
            return;
        }

        let tx = match &self.core_event_tx {
            Some(tx) => tx.clone(),
            None => return,
        };
        let db = self.db.clone();
        let stop_flag = self.speed_test_stop.clone();
        let batch_id = uuid::Uuid::new_v4().to_string();
        let page_size = self.config.speed_test.batch_page_size.max(1);
        let group_id = self.selected_group_id.clone();

        let fast_timeout = *self.config.speed_test.tcp_timeout_secs;
        let real_ping_timeout = *self.config.speed_test.real_ping_timeout_secs;
        let retries = self.config.speed_test.real_ping_retries;
        let ping_url = self.config.speed_test.ping_url.clone();
        let ip_api_url = self.config.speed_test.ip_api_url.clone();
        let proxy_addr = self.config.inbound.listen.clone();
        let base_proxy_port = self.config.inbound.socks_port;

        let config_dir_path = dirs::config_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join("xray-tui");
        let bin_dir = config_dir_path.join("bin");
        let bin_configs_dir = config_dir_path.join("binConfigs");
        let real_ping_concurrency = self.config.speed_test.real_ping_concurrency.max(1);

        // Set up shared batch progress for status bar display
        let total_count = visible.len() as u16;
        let progress = Arc::new((AtomicU16::new(total_count), AtomicU16::new(0)));
        self.batch_progress = Some(progress.clone());

        tokio::spawn(async move {
            use std::sync::atomic::Ordering;

            // 1. Snapshot visible profiles into ping_sessions table
            let count = db.create_ping_batch(&batch_id, group_id.as_deref()).await;
            let count = match count {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(target: "tui", "create_ping_batch failed: {e}");
                    progress.0.store(0, Ordering::Relaxed);
                    return;
                }
            };
            if count == 0 {
                tracing::warn!(target: "tui", "create_ping_batch returned 0 — no matching profile_cores?");
                progress.0.store(0, Ordering::Relaxed);
                return;
            }

            let mut buffer: Vec<PingResultUpdate> = Vec::new();

            // 2. Initialize managers
            let fmgr = xray_tui_core::FastPingManager::new(fast_timeout);
            let rmgr = xray_tui_core::RealPingManager {
                ping_url,
                ip_api_url,
                timeout: real_ping_timeout,
                retries,
                proxy_addr,
                base_proxy_port,
                bin_dir,
                bin_configs_dir,
            };

            // ── Phase 1: Fast Ping ──
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                let sessions = db
                    .get_batch_page_ready_for_fast_ping(&batch_id, page_size)
                    .await;
                let Ok(sessions) = sessions else {
                    tracing::error!(target: "tui", "get_batch_page_ready_for_fast_ping failed");
                    progress.0.store(0, Ordering::Relaxed);
                    break;
                };
                if sessions.is_empty() {
                    break;
                }

                for session in &sessions {
                    let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                        profile_id: session.profile_id.clone(),
                        test_type: TestType::TcpPing,
                    });
                    let _ = db.update_session_status(&session.id, "fast_pinging").await;

                    let addr = match &session.address {
                        Some(a) => a.clone(),
                        None => continue,
                    };
                    let port = session.port.unwrap_or(0) as u16;
                    if port == 0 {
                        continue;
                    }

                    match fmgr.ping(session.config_type, &addr, port).await {
                        Ok(dur) => {
                            #[allow(clippy::cast_possible_truncation)]
                            let ms = dur.as_millis() as i32;
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                profile_id: session.profile_id.clone(),
                                status: "completed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: Some(ms),
                                speed_bps: None,
                                ip_info: None,
                                error: None,
                            });
                        }
                        Err(PingError::NotSupported) => {
                            if real_ping_enabled {
                                // Demote to real ping for Phase 2
                                let _ = db
                                    .update_session_ping_type(&session.id, "real", "queued")
                                    .await;
                            } else {
                                // No real ping phase — emit Cancelled immediately
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    profile_id: session.profile_id.clone(),
                                    status: "cancelled".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Not supported by fast ping".to_string()),
                                });
                                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                    profile_id: session.profile_id.clone(),
                                    test_type: TestType::TcpPing,
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Not supported by fast ping".to_string()),
                                });
                            }
                        }
                        Err(e) => {
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                profile_id: session.profile_id.clone(),
                                status: "failed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: None,
                                speed_bps: None,
                                ip_info: None,
                                error: Some(e.to_string()),
                            });
                        }
                    }
                    // Send SpeedTestResult to TUI for immediate feedback
                    let last = buffer.last().unwrap();
                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                        profile_id: session.profile_id.clone(),
                        test_type: TestType::TcpPing,
                        latency_ms: last.latency_ms.map(|v| v as u64),
                        speed_bps: None,
                        ip_info: None,
                        error: last.error.clone(),
                    });
                }
                // Flush buffer at page boundary
                let flushed = buffer.len() as u16;
                Self::batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
                progress.1.fetch_add(flushed, Ordering::Relaxed);
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            // ── Phase 2: Real Ping (only if enabled) ──
            if real_ping_enabled {
                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let sessions = db
                        .get_batch_page_ready_for_real_ping(&batch_id, page_size)
                        .await;
                    let Ok(sessions) = sessions else {
                        tracing::error!(target: "tui", "get_batch_page_ready_for_real_ping failed");
                        progress.0.store(0, Ordering::Relaxed);
                        break;
                    };
                    if sessions.is_empty() {
                        break;
                    }

                    let sem = Arc::new(Semaphore::new(real_ping_concurrency));
                    let mut handles = Vec::with_capacity(sessions.len());
                    for (session, profile) in &sessions {
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }
                        let permit = match Arc::clone(&sem).acquire_owned().await {
                            Ok(p) => p,
                            Err(_) => break,
                        };
                        let rmgr = rmgr.clone();
                        let tx = tx.clone();
                        let db = Arc::clone(&db);
                        let session = session.clone();
                        let profile = profile.clone();

                        handles.push(tokio::spawn(async move {
                            let _permit = permit;
                            let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                                profile_id: session.profile_id.clone(),
                                test_type: TestType::RealPing,
                            });
                            let _ = db.update_session_status(&session.id, "real_pinging").await;
                            let result = rmgr.real_ping(&profile, session.config_type).await;
                            let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                profile_id: session.profile_id.clone(),
                                test_type: TestType::RealPing,
                                latency_ms: result.latency_ms,
                                speed_bps: None,
                                ip_info: result.ip_info.clone(),
                                error: result.error.clone(),
                            });
                            PingResultUpdate {
                                session_id: session.id.clone(),
                                profile_id: session.profile_id.clone(),
                                status: if result.error.is_none() {
                                    "completed".to_string()
                                } else {
                                    "failed".to_string()
                                },
                                ping_type: "real".to_string(),
                                latency_ms: result.latency_ms.map(|v| v as i32),
                                speed_bps: None,
                                ip_info: result.ip_info.clone(),
                                error: result.error,
                            }
                        }));
                    }

                    // Collect results
                    for handle in handles {
                        if let Ok(update) = handle.await {
                            buffer.push(update);
                        }
                    }
                    let flushed = buffer.len() as u16;
                    Self::batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
                    progress.1.fetch_add(flushed, Ordering::Relaxed);
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }

            // Final flush + cancel remaining + cleanup
            if stop_flag.load(Ordering::Relaxed) {
                // Emit Cancelled events for any non-terminal sessions
                if let Ok(sessions) = db.get_ping_batch_page(&batch_id, 65536, 0).await {
                    for session in &sessions {
                        if session.status == "completed"
                            || session.status == "failed"
                            || session.status == "cancelled"
                        {
                            continue;
                        }
                        let test_type = if session.ping_type == "real" {
                            TestType::RealPing
                        } else {
                            TestType::TcpPing
                        };
                        let _ = tx.try_send(CoreEvent::SpeedTestResult {
                            profile_id: session.profile_id.clone(),
                            test_type,
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: Some("Cancelled".to_string()),
                        });
                    }
                }
                let _ = db.cancel_ping_batch(&batch_id).await;
            }
            let flushed = buffer.len() as u16;
            Self::batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);
            let _ = db.cleanup_ping_batch(&batch_id).await;
            progress.0.store(0, Ordering::Relaxed); // signal batch is done; enables stale cleanup
        });
    }

    /// Flush accumulated `PingResultUpdate`s to DB. Called at page boundaries and batch end.
    async fn batch_upsert_buffer(
        batch_id: &str,
        db: &Arc<Database>,
        buffer: &mut Vec<PingResultUpdate>,
    ) {
        if buffer.is_empty() {
            return;
        }
        let batch = std::mem::take(buffer);
        let extensions: Vec<ProfileExtension> = batch
            .iter()
            .filter_map(|r| {
                r.latency_ms.map(|ms| ProfileExtension {
                    profile_id: r.profile_id.clone(),
                    delay: Some(ms),
                    speed: None,
                    sort_order: None,
                    ip_info: r.ip_info.clone(),
                })
            })
            .collect();
        let _ = db
            .batch_flush_ping_buffer(batch_id, &batch, &extensions)
            .await;
    }
    /// Remove profiles whose extension.delay == Some(-1) (failed TCP ping).
    pub async fn remove_failed_servers(&mut self) {
        let to_remove: Vec<String> = self
            .profiles
            .iter()
            .filter(|r| r.extension.as_ref().is_some_and(|e| e.delay == Some(-1)))
            .map(|r| r.profile.id.clone())
            .collect();
        let count = to_remove.len();
        for id in to_remove {
            self.delete_profile(&id).await;
        }
        self.multi_select.clear();
        self.log_trace("info", "tui", &format!("Removed {count} failed server(s)"));
    }

    /// Poll core event channel and update state accordingly.
    pub async fn poll_core_events(&mut self) {
        // Clean up stale batch_progress when the task has finished silently
        // (e.g., create_ping_batch returned 0, no TestTypeUpdate events were ever sent).
        if self.batch_progress.is_some()
            && self.testing_profiles.is_empty()
            && self.batch_progress.as_ref().map(|p| p.0.load(Ordering::Relaxed)) == Some(0)
        {
            self.batch_progress = None;
        }
        while let Some(rx) = self.core_event_rx.as_mut() {
            let event = match rx.try_recv() {
                Ok(event) => event,
                Err(
                    tokio::sync::mpsc::error::TryRecvError::Empty
                    | tokio::sync::mpsc::error::TryRecvError::Disconnected,
                ) => break,
            };
            match event {
                CoreEvent::Connected(core_type) => {
                    self.connected_core = Some(core_type);
                    self.connecting = false;
                    self.connection_error = None;
                    self.log_trace("info", "core", &format!("Connected [{core_type}]"));
                }
                CoreEvent::Disconnected => {
                    // Ignore stale Disconnected if already reconnecting
                    if !self.connecting {
                        self.connected_core = None;
                        self.connected_profile_id = None;
                        self.log_trace("info", "core", "Core process stopped");
                    }
                }
                CoreEvent::Error(err) => {
                    self.connection_error = Some(err.clone());
                    self.connecting = false;
                    self.connected_core = None;
                    self.log_trace("error", "core", &format!("Connection error: {err}"));
                    self.connected_profile_id = None;
                }
                CoreEvent::StatsError(msg) => {
                    self.connection_error = Some(msg.clone());
                    self.log_trace("warning", "core", &format!("Stats error: {msg}"));
                }
                CoreEvent::StatsUpdate {
                    profile_id,
                    today_up,
                    today_down,
                    total_up,
                    total_down,
                } => {
                    self.connection_error = None;
                    let stats = ServerStat {
                        profile_id: profile_id.clone(),
                        today_up: Some(today_up as i32),
                        today_down: Some(today_down as i32),
                        total_up: Some(total_up as i32),
                        total_down: Some(total_down as i32),
                        last_updated: Some(crate::format_now()),
                    };
                    if let Err(e) = self.db.upsert_server_stats(&stats).await {
                        self.log_trace("error", "tui", &format!("Failed to save stats: {e}"));
                    }
                    // Update in-memory ProfileRow to avoid full reload
                    if let Some(row) = self
                        .profiles
                        .iter_mut()
                        .find(|r| r.profile.id == profile_id)
                    {
                        row.stats = Some(stats);
                    }
                    self.current_traffic_up = total_up;
                    self.current_traffic_down = total_down;
                }
                CoreEvent::SysStatsUpdate(stats) => {
                    self.current_memory = stats.alloc;
                    self.system_stats = Some(stats);
                }
                CoreEvent::LogLine {
                    level,
                    target,
                    message,
                    timestamp_nanos,
                } => {
                    self.add_log(level, target, message, timestamp_nanos);
                }
                CoreEvent::TuiLog {
                    target,
                    level,
                    message,
                } => {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as i64;
                    self.add_log(level, target, message, ts);
                }
                CoreEvent::SubscriptionsUpdated {
                    group_id,
                    count,
                    error,
                    summary,
                } => {
                    self.updating_groups.remove(&group_id);
                    if summary.total_errors > 0 || summary.security_warning_count > 0 {
                        let msg = format!(
                            "Subscription validation: {} errors (missing fields: {}, host validation: {}, security warnings: {}, other: {})",
                            summary.total_errors,
                            summary.missing_field_count,
                            summary.host_validation_count,
                            summary.security_warning_count,
                            summary.other_count,
                        );
                        self.log_trace("warn", "subscription", &msg);
                    }
                    if let Some(err) = error {
                        self.log_trace(
                            "error",
                            "subscription",
                            &format!("Subscription update failed: {err}"),
                        );
                    } else {
                        self.log_trace(
                            "info",
                            "subscription",
                            &format!("Subscription updated: {count} profiles"),
                        );
                    }
                    self.reload_profiles().await;
                    self.reload_groups().await;
                    self.subscriptions = self.db.get_all_subscriptions().await.unwrap_or_default();
                }
                CoreEvent::TestTypeUpdate {
                    profile_id,
                    test_type,
                } => {
                    self.testing_details.insert(profile_id.clone(), test_type);
                    self.testing_profiles.insert(profile_id);
                }
                CoreEvent::SpeedTestResult {
                    profile_id,
                    test_type,
                    latency_ms,
                    speed_bps,
                    ip_info,
                    error,
                } => {
                    self.testing_profiles.remove(&profile_id);
                    self.testing_details
                        .remove(&profile_id);

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
                                    TestType::RealPing => {
                                        ext.delay = latency_ms.map(|v| v as i32);
                                        ext.ip_info = ip_info;
                                    }
                                    TestType::TcpPing | TestType::UdpTest => {
                                        ext.delay = latency_ms.map(|v| v as i32);
                                    }
                                    TestType::SpeedTest => {
                                        ext.speed = speed_bps
                                            .map(|v| std::cmp::min(v, i32::MAX as u64) as i32);
                                    }
                                }
                                let _ = self.db.upsert_profile_extension(ext).await;
                                row.profile
                                    .remarks
                                    .clone()
                                    .unwrap_or_else(|| profile_id.clone())
                            }
                            None => profile_id.clone(),
                        }
                    };

                    if let Some(ref err) = error {
                        self.log_trace(
                            "warn",
                            "speedtest",
                            &format!("{test_type:?} failed for {name}: {err}"),
                        );
                    } else {
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
                        self.log_trace(
                            "info",
                            "speedtest",
                            &format!("{test_type:?} {name}: {detail}"),
                        );
                    }

                    // Update tracking fields for actions log
                    match test_type {
                        TestType::TcpPing => self.last_test_tcp = latency_ms,
                        TestType::RealPing => self.last_test_real = latency_ms,
                        TestType::SpeedTest => self.last_test_speed = speed_bps,
                        TestType::UdpTest => {} // no tracking for UDP
                    }

                    // Auto-reset stop flag when all tests complete
                    if self.testing_profiles.is_empty() {
                        self.speed_test_stop.store(false, Ordering::Relaxed);
                        self.batch_progress = None;
                    }
                }
                CoreEvent::UpdateCheckResult {
                    core_type,
                    current_version,
                    latest_version,
                    error,
                } => {
                    let status = self.update_status.entry(core_type).or_default();
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
                                        (Some(c), Some(l)) => {
                                            xray_tui_core::updater::is_newer(&c, &l)
                                        }
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
                                self.config.updates.xray_latest_known = Some(ver.clone());
                            }
                            CoreType::SingBox => {
                                self.config.updates.sing_box_latest_known = Some(ver.clone());
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
                    } = &mut self.mode
                    {
                        *status_xray = self
                            .update_status
                            .get(&CoreType::Xray)
                            .cloned()
                            .unwrap_or_default();
                        *status_singbox = self
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
                    if let Some(status) = self.update_status.get_mut(&core_type) {
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
                    let status = self.update_status.entry(core_type).or_default();
                    status.downloading = false;
                    status.download_progress = None;
                    if success {
                        status.current_version = Some(new_version.clone());
                        status.update_available = false;
                        self.log_trace(
                            "info",
                            "tui",
                            &format!(
                                "{core_type} updated: {} → {}",
                                old_version.as_deref().unwrap_or("none"),
                                new_version,
                            ),
                        );
                    } else {
                        status.error.clone_from(&error);
                        self.log_trace(
                            "error",
                            "tui",
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
                    } = &mut self.mode
                    {
                        *status_xray = self
                            .update_status
                            .get(&CoreType::Xray)
                            .cloned()
                            .unwrap_or_default();
                        *status_singbox = self
                            .update_status
                            .get(&CoreType::SingBox)
                            .cloned()
                            .unwrap_or_default();
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
            ("update_interval".into(), "1h".into()),
            ("core_type".into(), "auto".into()),
        ];
        self.mode = AppMode::AddGroup {
            fields,
            focus_index: 0,
        };
    }

    pub fn start_edit_group(&mut self, group_id: &str) {
        let group = if let Some(g) = self.groups.iter().find(|g| g.id == group_id) {
            g.clone()
        } else {
            self.log_trace("error", "tui", "Group not found");
            return;
        };
        let update_interval_value = self.subscriptions
            .iter()
            .find(|s| s.group_id.as_deref() == Some(group_id))
            .and_then(|s| s.update_interval).map_or_else(|| "1h".into(), |mins| humantime::format_duration(
                std::time::Duration::from_secs(mins as u64 * 60)
            ).to_string());
        let fields = vec![
            ("name".into(), group.name.unwrap_or_default()),
            (
                "subscription_url".into(),
                group.subscription_url.unwrap_or_default(),
            ),
            ("user_agent".into(), group.user_agent.unwrap_or_default()),
            ("update_interval".into(), update_interval_value),
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

    pub async fn confirm_add_group(&mut self) {
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
        if let Err(e) = self.db.insert_group(&group).await {
            self.log_trace("error", "tui", &format!("Failed to add group: {e}"));
            return;
        }
        // Create subscription tracking row
        let interval: i32 = get_field(&fields, "update_interval")
            .and_then(|v| humantime::parse_duration(&v).ok())
            .map_or(60, |d| (d.as_secs() / 60) as i32);
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
        let _ = self.db.upsert_subscription(&sub).await;
        self.log_trace(
            "info",
            "tui",
            &format!(
                "Group '{}' added",
                group.name.as_deref().unwrap_or("unnamed")
            ),
        );
        self.mode = AppMode::List;
        self.reload_groups().await;
    }

    pub async fn confirm_edit_group(&mut self) {
        let (group_id, fields) = match &self.mode {
            AppMode::EditGroup {
                group_id, fields, ..
            } => (group_id.clone(), fields.clone()),
            _ => return,
        };
        let mut group = if let Some(g) = self.groups.iter().find(|g| g.id == group_id) {
            g.clone()
        } else {
            self.log_trace("error", "tui", "Group not found");
            return;
        };
        group.name = get_field(&fields, "name");
        group.subscription_url = get_field(&fields, "subscription_url");
        group.user_agent = get_field(&fields, "user_agent");
        group.core_type = get_field(&fields, "core_type");
        if let Err(e) = self.db.update_group(&group).await {
            self.log_trace("error", "tui", &format!("Failed to update group: {e}"));
            return;
        }
        // Update subscription tracking row
        let interval: i32 = get_field(&fields, "update_interval")
            .and_then(|v| humantime::parse_duration(&v).ok())
            .map_or(60, |d| (d.as_secs() / 60) as i32);
        if let Ok(Some(mut sub)) = self.db.get_subscription_by_group(&group_id).await {
            sub.url = group.subscription_url.clone().unwrap_or_default();
            sub.update_interval = Some(interval);
            sub.user_agent = group.user_agent.clone();
            let _ = self.db.upsert_subscription(&sub).await;
        }
        self.log_trace("info", "tui", "Group updated");
        self.mode = AppMode::List;
        self.reload_groups().await;
    }

    pub async fn delete_group(&mut self, group_id: &str) {
        if let Err(e) = self.db.delete_group(group_id).await {
            self.log_trace("error", "tui", &format!("Failed to delete group: {e}"));
            return;
        }
        let _ = self.db.delete_subscriptions_by_group(group_id).await;
        self.log_trace("info", "tui", "Group deleted");
        self.selected_group_id = None;
        self.confirmation = None;
        self.reload_groups().await;
        self.reload_profiles().await;
    }

    pub async fn clear_group(&mut self, group_id: &str) {
        match self.db.clear_group(group_id).await {
            Ok(count) => {
                self.log_trace(
                    "info",
                    "tui",
                    &format!("Cleared {count} profiles from group"),
                );
            }
            Err(e) => {
                self.log_trace("error", "tui", &format!("Failed to clear group: {e}"));
            }
        }
        self.confirmation = None;
        self.reload_profiles().await;
    }

    pub fn clear_logs(&mut self) {
        self.log_cache.clear();
        self.log_scroll = 0;
        // Set cursor to now so poll_new_logs doesn't re-read old entries
        self.last_seen_log_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.confirmation = None;
        self.log_trace("info", "tui", "Logs cleared");
    }

    /// Clear entire log database (heed) in addition to the in-memory cache.
    /// Note: there may be a brief race with the background log writer;
    /// any messages already in the mpsc channel will be written after the clear
    /// and may reappear on next scroll load.
    pub fn purge_logs_database(&mut self) {
        self.log_cache.clear();
        self.log_scroll = 0;
        self.last_seen_log_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        // Clear the heed database in the background (best-effort)
        if let Some(ref heed) = self.heed_storage {
            let heed = heed.clone();
            tokio::spawn(async move {
                if let Err(e) = heed.clear_all_async().await {
                    tracing::error!(target: "tui", "Failed to clear log database: {e}");
                }
            });
        }
        self.confirmation = None;
        self.log_trace("info", "tui", "Log database cleared");
    }

    // ── Subscription update ──────────────────────────────────────────

    pub fn update_group_subscriptions(&mut self, group_id: &str) {
        if self.updating_groups.contains(group_id) {
            return;
        }
        let group = if let Some(g) = self.groups.iter().find(|g| g.id == group_id) {
            g.clone()
        } else {
            self.log_trace("error", "tui", "Group not found");
            return;
        };
        let url = match &group.subscription_url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => {
                self.log_trace("warn", "tui", "Group has no subscription URL");
                return;
            }
        };

        self.updating_groups.insert(group_id.to_string());
        let gid = group_id.to_string();
        let tx = self.core_event_tx.clone();
        let user_agent = group.user_agent.unwrap_or_else(|| "xray-tui/0.1".into());
        let db = self.db.clone();
        let validation: ValidationSettings = self.config.parsing.clone().into();
        tokio::spawn(async move {
            let result = tokio::time::timeout(
                std::time::Duration::from_mins(2),
                Self::do_update_subscription(url, user_agent, gid.clone(), db, validation),
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
        validation: xray_tui_config::import_export::ValidationSettings,
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
        tracing::info!(
            target: "tui",
            "Starting DB upsert for {} enriched profiles",
            enriched.len()
        );
        if let Err(e) = db.subscription_upsert_profiles(&group_id, &enriched).await {
            tracing::error!(target: "tui", "DB upsert failed: {e}");
            return (group_id, 0, summary, Some(format!("DB upsert: {e}")));
        }
        tracing::info!(target: "tui", "DB upsert succeeded");

        let _ = db
            .move_orphans_to_graveyard(&group_id, &sub_uids, GRAVEYARD_GROUP_ID)
            .await;
        let _ = db.purge_graveyard(GRAVEYARD_GROUP_ID, 24).await;

        let existing = db.get_subscription_by_group(&group_id).await.ok().flatten();
        let sub_id = existing.as_ref().map_or_else(|| uuid::Uuid::new_v4().to_string(), |s| s.id.clone());
        let stored_interval = existing.as_ref().and_then(|s| s.update_interval).unwrap_or(60);

        let sub = Subscription {
            id: sub_id,
            group_id: Some(group_id.clone()),
            url: url.clone(),
            last_updated: Some(format_now()),
            update_interval: Some(stored_interval),
            user_agent: Some(user_agent),
            status: Some("ok".into()),
            error_message: None,
        };
        let _ = db.upsert_subscription(&sub).await;

        (group_id, enriched.len(), summary, None)
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
                let current =
                    xray_tui_core::updater::get_current_version(core_type, &bin_dir).await;
                let latest = xray_tui_core::updater::get_latest_version(core_type).await;
                let error = if current.is_none() && latest.is_none() {
                    Some("binary not found and check failed".into())
                } else if latest.is_none() {
                    Some("failed to check latest version".into())
                } else {
                    None
                };
                try_send_or_warn(
                    &tx,
                    CoreEvent::UpdateCheckResult {
                        core_type,
                        current_version: current,
                        latest_version: latest,
                        error,
                    },
                    "update_check_result",
                );
            });
        }
    }

    /// Spawn async task to download and install an update for the given core.
    pub fn spawn_update_download(&mut self, core_type: CoreType) {
        // Guard: don't download if already downloading
        if self
            .update_status
            .get(&core_type)
            .is_some_and(|s| s.downloading)
        {
            return;
        }
        // Guard: don't download if core is currently running
        if self.connected_core == Some(core_type) {
            self.log_trace(
                "warn",
                "tui",
                &format!("Cannot update {core_type} while it's running. Disconnect first."),
            );
            return;
        }

        let latest = match self
            .update_status
            .get(&core_type)
            .and_then(|s| s.latest_version.clone())
        {
            Some(v) => v,
            None => return,
        };
        let old_version = self
            .update_status
            .get(&core_type)
            .and_then(|s| s.current_version.clone());
        let Some(tx) = self.core_event_tx.clone() else {
            return;
        };
        let bin_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui/bin");
        let client = reqwest::Client::new();
        let temp_dir = std::env::temp_dir().join(format!("xray-tui-update-{core_type}"));

        self.update_status.entry(core_type).or_default().downloading = true;

        let last_report = Arc::new(Mutex::new(std::time::Instant::now()));
        let core_type_progress = core_type;

        let progress_cb = {
            let tx_progress = tx.clone();
            move |downloaded: u64, total: u64| {
                let should_send = {
                    let mut last = last_report.lock().unwrap();
                    if last.elapsed() >= std::time::Duration::from_millis(100) {
                        *last = std::time::Instant::now();
                        true
                    } else {
                        false
                    }
                };
                if should_send {
                    let _ = tx_progress.try_send(CoreEvent::UpdateDownloadProgress {
                        core_type: core_type_progress,
                        downloaded,
                        total,
                    });
                }
            }
        };

        tokio::spawn(async move {
            // Download
            let archive = match xray_tui_core::updater::download_release(
                &client,
                core_type,
                &latest,
                &temp_dir,
                Some(progress_cb),
            )
            .await
            {
                Ok(path) => path,
                Err(e) => {
                    try_send_or_warn(
                        &tx,
                        CoreEvent::UpdateCompleted {
                            core_type,
                            old_version: old_version.clone(),
                            new_version: latest,
                            success: false,
                            error: Some(e.to_string()),
                        },
                        "update_completed_err",
                    );
                    return;
                }
            };
            // Install
            let result =
                xray_tui_core::updater::install_binary(&archive, core_type, &bin_dir).await;
            let (success, error) = match result {
                Ok(()) => (true, None),
                Err(e) => (false, Some(e.to_string())),
            };
            // Clean up temp file
            let _ = std::fs::remove_file(&archive);
            let _ = std::fs::remove_dir_all(&temp_dir);

            try_send_or_warn(
                &tx,
                CoreEvent::UpdateCompleted {
                    core_type,
                    old_version,
                    new_version: latest,
                    success,
                    error,
                },
                "update_completed",
            );
        });
    }

    /// Start a background task to check and update subscriptions.
    pub fn spawn_auto_update(&mut self) {
        let Some(tx) = self.core_event_tx.clone() else {
            return;
        };
        let db = self.db.clone();
        let validation: ValidationSettings = self.config.parsing.clone().into();
        let shutdown = self.shutdown_token.clone();
        tokio::spawn(async move {
            use std::time::Duration;
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
                    let url = match &group.subscription_url {
                        Some(u) => u.clone(),
                        None => continue,
                    };
                    let ua = group
                        .user_agent
                        .clone()
                        .unwrap_or_else(|| "xray-tui/0.1".into());
                    let gid = group.id.clone();
                    let result =
                        Self::do_update_subscription(url, ua, gid, db.clone(), validation.clone())
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
}

/// Helper to send a `CoreEvent` with a warning on channel full.
/// Prevents silent event loss.
fn try_send_or_warn(
    tx: &tokio::sync::mpsc::Sender<CoreEvent>,
    event: CoreEvent,
    label: &'static str,
) {
    if let Err(_e) = tx.try_send(event) {
        warn!(target: "log_worker", "try_send dropped {label}: channel full");
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
#[must_use]
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

const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn parse_core_log_line(line: &str, core_type: CoreType) -> (String, String, String, Option<i64>) {
    use regex::Regex;
    use std::sync::OnceLock;

    static XRAY_TS_RE: OnceLock<Regex> = OnceLock::new();
    static LEVEL_RE: OnceLock<Regex> = OnceLock::new();

    let trimmed = line.trim();
    let mut ts_nanos: Option<i64> = None;

    // Sing-box JSON format: {"level":"info","time":"...","msg":"..."}
    if trimmed.starts_with('{')
        && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed)
    {
        let raw_level = parsed
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let level = if raw_level == "warn" {
            "warning"
        } else {
            raw_level
        };
        let msg = parsed
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or(trimmed);

        // Parse timestamp
        if let Some(ts_str) = parsed.get("time").and_then(|v| v.as_str()) {
            ts_nanos = chrono::DateTime::parse_from_rfc3339(ts_str)
                .ok()
                .or_else(|| chrono::DateTime::parse_from_str(ts_str, "%+").ok())
                .and_then(|dt| dt.timestamp_nanos_opt());
        }

        // Extract target from msg if it has a "tag: message" pattern
        let target = msg.find(": ").map_or_else(
            || "sing".to_string(),
            |pos| format!("sing::{}", &msg[..pos]),
        );

        return (level.to_string(), target, msg.to_string(), ts_nanos);
    }

    // Xray-core format: "2026/06/25 15:18:54.387241 [Info] message"
    let xray_ts_re = XRAY_TS_RE
        .get_or_init(|| Regex::new(r"^(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}(?:\.\d+)?) ?").unwrap());
    let level_re = LEVEL_RE.get_or_init(|| Regex::new(r"\[(Debug|Info|Warning|Error)\]").unwrap());

    #[allow(
        clippy::option_if_let_else,
        reason = "side-effect on ts_nanos makes map_or awkward"
    )]
    let remaining = if let Some(caps) = xray_ts_re.captures(trimmed) {
        let ts_str = caps.get(1).unwrap().as_str();
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y/%m/%d %H:%M:%S%.f") {
            ts_nanos = Some(naive.and_utc().timestamp_nanos_opt().unwrap_or(0));
        }
        trimmed[caps.get(0).unwrap().len()..].trim()
    } else {
        trimmed
    };

    let (level_str, msg_after_level) = level_re.captures(remaining).map_or_else(
        || ("info", remaining),
        |caps| {
            let raw = caps.get(1).unwrap().as_str();
            let lvl = if raw.eq_ignore_ascii_case("debug") {
                "debug"
            } else if raw.eq_ignore_ascii_case("info") {
                "info"
            } else if raw.eq_ignore_ascii_case("warn") || raw.eq_ignore_ascii_case("warning") {
                "warning"
            } else if raw.eq_ignore_ascii_case("error") {
                "error"
            } else {
                raw
            };
            let after = remaining[caps.get(0).unwrap().len()..].trim();
            (lvl, after)
        },
    );

    // Extract target from message
    let core_prefix = if core_type == CoreType::SingBox {
        "sing"
    } else {
        "xray"
    };
    #[allow(
        clippy::option_if_let_else,
        reason = "nested if inside fn arm makes map_or_else less readable"
    )]
    let (target, message) = if let Some(pos) = msg_after_level.find(": ") {
        let tag = &msg_after_level[..pos];
        // For xray format: "infra/conf/serial: message" → replace / with ::
        let target = if core_type == CoreType::Xray {
            format!("xray::{}", tag.replace('/', "::"))
        } else {
            format!("sing::{tag}")
        };
        let rest = msg_after_level[pos + 2..].trim();
        (target, rest.to_string())
    } else {
        (core_prefix.to_string(), msg_after_level.to_string())
    };
    (level_str.to_string(), target, message, ts_nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xray_log() {
        let (level, target, msg, ts) = parse_core_log_line(
            "2026/06/25 15:18:54.387241 [Info] Server started",
            CoreType::Xray,
        );
        assert_eq!(level, "info");
        assert_eq!(target, "xray");
        assert!(msg.contains("Server started"));
        assert!(
            !msg.contains("2026/06/25"),
            "timestamp stripped from message"
        );
        assert!(ts.is_some(), "xray timestamp should parse");
    }
    #[test]
    fn parse_xray_log_with_path() {
        let (level, target, msg, ts) = parse_core_log_line(
            "2026/06/25 15:18:54.387241 [Info] infra/conf/serial: Reading config",
            CoreType::Xray,
        );
        assert_eq!(level, "info");
        assert_eq!(target, "xray::infra::conf::serial");
        assert_eq!(msg, "Reading config");
        assert!(ts.is_some());
    }
    #[test]
    fn parse_singbox_json() {
        let json = r#"{"level":"warn","time":"2024-01-01T12:00:00Z","msg":"timeout"}"#;
        let (level, target, msg, ts) = parse_core_log_line(json, CoreType::SingBox);
        assert_eq!(level, "warning");
        assert_eq!(target, "sing");
        assert_eq!(msg, "timeout");
        assert!(ts.is_some());
    }
    #[test]
    fn parse_singbox_json_with_tag() {
        let json = r#"{"level":"info","time":"2024-01-01T12:00:00Z","msg":"dns: resolved"}"#;
        let (level, target, msg, ts) = parse_core_log_line(json, CoreType::SingBox);
        assert_eq!(level, "info");
        assert_eq!(target, "sing::dns");
        assert_eq!(msg, "dns: resolved");
        assert!(ts.is_some());
    }
    #[test]
    fn parse_fallback() {
        let (level, target, msg, ts) = parse_core_log_line("Xray 26.3.27 started", CoreType::Xray);
        assert_eq!(level, "info");
        assert_eq!(target, "xray");
        assert_eq!(msg, "Xray 26.3.27 started");
        assert!(ts.is_none(), "no timestamp in header line");
    }
    #[test]
    fn parse_connection_log() {
        let (level, target, msg, ts) = parse_core_log_line(
            "2026/06/25 15:18:54.387241 from 127.0.0.1:60868 accepted //host:443 [socks -> proxy]",
            CoreType::Xray,
        );
        assert_eq!(level, "info", "connection logs default to info");
        assert_eq!(target, "xray");
        assert!(ts.is_some(), "connection logs have timestamps");
        assert!(!msg.contains("2026/06/25"), "timestamp stripped");
    }
}
