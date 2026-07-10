use std::cell::RefCell;
use std::collections::HashMap;

use ratatui_cheese::tree::TreeState;
use xray_tui_config::import_export::{Profile, ValidationSummary};
use xray_tui_core::grpc_client;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::CoreType;
use xray_tui_db::models::EndpointRow;

/// Re-export EndpointRow as ProfileRow for backward compatibility.
pub use xray_tui_db::models::EndpointRow as ProfileRow;

/// Clash API /traffic response struct.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ClashTraffic {
    pub up: i64,
    pub down: i64,
}

/// Cached update status for a proxy backend.
#[derive(Debug, Clone, Default)]
pub struct BackendUpdateStatus {
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub downloading: bool,
    pub download_progress: Option<(u64, u64)>,
    pub error: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Logs,
    Statistics,
    Actions,
}

impl Tab {
    pub const ALL: &[Self] = &[Self::Profiles, Self::Settings, Self::Logs, Self::Statistics, Self::Actions];

    pub fn next(&self) -> Self {
        match self {
            Self::Profiles => Self::Settings,
            Self::Settings => Self::Logs,
            Self::Logs => Self::Statistics,
            Self::Statistics => Self::Actions,
            Self::Actions => Self::Profiles,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            Self::Profiles => Self::Actions,
            Self::Settings => Self::Profiles,
            Self::Logs => Self::Settings,
            Self::Statistics => Self::Logs,
            Self::Actions => Self::Statistics,
        }
    }
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

#[derive(Debug, Clone)]
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

impl std::fmt::Display for SettingsSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "Core"),
            Self::Gui => write!(f, "GUI"),
            Self::Inbound => write!(f, "Inbound"),
            Self::Routing => write!(f, "Routing"),
            Self::Dns => write!(f, "DNS"),
            Self::SystemProxy => write!(f, "System Proxy"),
            Self::Tun => write!(f, "TUN"),
            Self::Mux => write!(f, "Mux"),
            Self::Stats => write!(f, "Statistics"),
            Self::ProtocolCore => write!(f, "Protocol Core"),
            Self::Updates => write!(f, "Updates"),
            Self::SpeedTest => write!(f, "Speed Test"),
            Self::Logging => write!(f, "Logging"),
        }
    }
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
        protocol: Option<xray_tui_core::protocol::Protocol>,
        /// Raw form field values: map of field key -> current input
        fields: Vec<(String, String)>,
        /// Index of the focused field
        focus_index: usize,
        /// Per-field validation errors
        form_errors: HashMap<String, String>,
    },
    EditServer {
        protocol_id: i64,
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

/// Events from the core process manager to the UI event loop.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    Connected(CoreType),
    Disconnected,
    Error(String),
    /// Non-fatal stats error — keeps `connected_core` intact
    StatsError(String),
    StatsUpdate {
        protocol_id: i64,
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
        protocol_id: i64,
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
    TestTypeUpdate {
        protocol_id: i64,
        test_type: TestType,
    },
    /// Batch-level progress for real ping.
    BatchProgress {
        total: u16,
        completed: u16,
    },
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteProfile(i64),
    DeleteProfiles(Vec<i64>),
    DeleteGroup(String),
    ClearGroup(String),
    ClearLogs,
    PurgeLogsDatabase,
    Quit,
}
