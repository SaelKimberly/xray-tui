use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use ratatui_cheese::tree::TreeState;
use xray_tui_config::import_export::{ParsedProfile, ValidationSummary};
use xray_tui_core::CoreType;
use xray_tui_core::grpc_client;
use xray_tui_core::speed_test::TestType;
use xray_tui_native::telemetry::{TraceEvent, TraceKind, TraceSecurity};

/// Re-export `EndpointRow` as `EndpointRow` for backward compatibility.
pub use xray_tui_db::models::EndpointRow;

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

/// One traced native-core connection leg, fed by `CoreEvent::NativeTrace`.
#[derive(Debug, Clone)]
pub struct NativeActivityEntry {
    pub conn_id: u64,
    pub opened_ms: i64,
    pub kind: TraceKind,
    pub dest: String,
    pub protocol: String,
    pub transport: String,
    pub security: TraceSecurity,
    pub up: u64,
    pub down: u64,
    pub closed: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Session-only trace log (not persisted): open legs plus recent finished
/// ones, with session totals. Capped — oldest entries fall off the front.
#[derive(Debug, Clone)]
pub struct NativeActivityLog {
    pub entries: VecDeque<NativeActivityEntry>,
    cap: usize,
    pub total_up: u64,
    pub total_down: u64,
    pub open_count: usize,
    pub fail_count: usize,
}

impl NativeActivityLog {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            cap: cap.max(1),
            total_up: 0,
            total_down: 0,
            open_count: 0,
            fail_count: 0,
        }
    }

    /// Record one trace event. `Opened` pushes a fresh row; `Closed` finds
    /// the matching row from the back (conn ids are near-monotonic).
    pub fn record(&mut self, event: &TraceEvent) {
        match event {
            TraceEvent::Opened(opened) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis().try_into().unwrap_or(i64::MAX));
                self.entries.push_back(NativeActivityEntry {
                    conn_id: opened.conn_id,
                    opened_ms: now_ms,
                    kind: opened.kind,
                    dest: opened.dest.clone(),
                    protocol: opened.protocol.clone(),
                    transport: opened.transport.clone(),
                    security: opened.security,
                    up: 0,
                    down: 0,
                    closed: false,
                    duration_ms: 0,
                    error: None,
                });
                self.open_count += 1;
            }
            TraceEvent::Closed(closed) => {
                let row = self
                    .entries
                    .iter_mut()
                    .rev()
                    .find(|entry| entry.conn_id == closed.conn_id && !entry.closed);
                match row {
                    Some(entry) => {
                        entry.up = closed.up_bytes;
                        entry.down = closed.down_bytes;
                        entry.duration_ms = closed.duration_ms;
                        entry.error.clone_from(&closed.error);
                        entry.closed = true;
                        self.open_count = self.open_count.saturating_sub(1);
                    }
                    None => {
                        // Close without a matching open (dropped open event):
                        // keep a ghost closed row so the counters stay exact.
                        self.entries.push_back(NativeActivityEntry {
                            conn_id: closed.conn_id,
                            opened_ms: 0,
                            kind: TraceKind::Tcp,
                            dest: String::new(),
                            protocol: String::new(),
                            transport: String::new(),
                            security: TraceSecurity::Plain,
                            up: closed.up_bytes,
                            down: closed.down_bytes,
                            closed: true,
                            duration_ms: closed.duration_ms,
                            error: closed.error.clone(),
                        });
                    }
                }
                self.total_up = self.total_up.saturating_add(closed.up_bytes);
                self.total_down = self.total_down.saturating_add(closed.down_bytes);
                if closed.error.is_some() {
                    self.fail_count += 1;
                }
            }
        }
        while self.entries.len() > self.cap {
            let dropped = self.entries.pop_front();
            if dropped.is_some_and(|entry| !entry.closed) {
                self.open_count = self.open_count.saturating_sub(1);
            }
        }
    }

    /// Start a fresh native session: drop every row, zero every counter,
    /// keep the configured cap.
    ///
    /// The native session task is aborted on disconnect, so its final
    /// `Disconnected` can be lost; without an explicit reset the next
    /// session would render the previous one's rows and session totals.
    pub fn reset(&mut self) {
        self.entries.clear();
        self.total_up = 0;
        self.total_down = 0;
        self.open_count = 0;
        self.fail_count = 0;
    }

    /// Close out every still-open row: the session ended.
    ///
    /// A native shutdown can drop the in-flight `Closed` traces (the session
    /// task is aborted mid-copy), which would leave those connections
    /// rendered live forever. Byte totals stay as last observed — the close
    /// event that would have carried them never arrived.
    pub fn close_out_open(&mut self) {
        if self.open_count == 0 {
            return;
        }
        for entry in self.entries.iter_mut().filter(|entry| !entry.closed) {
            entry.closed = true;
        }
        self.open_count = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Logs,
    Statistics,
    NativeActivity,
    Actions,
}

impl Tab {
    pub const ALL: &[Self] = &[
        Self::Profiles,
        Self::Settings,
        Self::Logs,
        Self::Statistics,
        Self::NativeActivity,
        Self::Actions,
    ];

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Profiles => Self::Settings,
            Self::Settings => Self::Logs,
            Self::Logs => Self::Statistics,
            Self::Statistics => Self::NativeActivity,
            Self::NativeActivity => Self::Actions,
            Self::Actions => Self::Profiles,
        }
    }
    #[must_use]
    pub const fn prev(&self) -> Self {
        match self {
            Self::Profiles => Self::Actions,
            Self::Settings => Self::Profiles,
            Self::Logs => Self::Settings,
            Self::Statistics => Self::Logs,
            Self::NativeActivity => Self::Statistics,
            Self::Actions => Self::NativeActivity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    ConfigType,
    Address,
    Port,
    Test,
    Speed,
    Traffic,
    Core,
    LastSeen,
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
    /// Group/subscription list shown in the settings Subscriptions section.
    GroupList {
        selected: usize,
        selected_mask: Vec<bool>,
    },
    /// Add/edit group form shown in the settings Subscriptions section.
    GroupForm {
        /// None = adding a new group; Some(gid) = editing an existing group.
        group_id: Option<String>,
        fields: Vec<(String, String)>,
        focus_index: usize,
        form_errors: HashMap<String, String>,
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
    Subscriptions,
    RouteProbes,
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
            Self::Subscriptions => write!(f, "Subscriptions"),
            Self::RouteProbes => write!(f, "Route Probes"),
        }
    }
}

/// Enrichment data for one endpoint (inbound host), computed lazily in the
/// background by `ops::enrich`. Survives profile reloads (`AppState.endpoint_info`).
#[derive(Debug, Clone, Default)]
pub struct EndpointInfo {
    /// DNS resolution of the inbound host (or the host itself when it is an IP).
    pub resolved_ips: Vec<std::net::IpAddr>,
    /// ISO-3166 alpha-2 country of the inbound IP from mmdb; None = unknown/not found.
    pub country: Option<String>,
    /// ip/cidr whitelist membership of the resolved IP (IPv4-only).
    pub host_features: xray_tui_host_features::HostFeatures,
    /// None = no SNI in config or checker not loaded.
    pub sni_whitelisted: Option<bool>,
    /// Exit IP parsed from real-ping `ip_info`.
    pub outbound_ip: Option<std::net::IpAddr>,
    /// ISO-3166 alpha-2 of `outbound_ip`.
    pub outbound_country: Option<String>,
    /// Unix secs of DNS resolution; None = IP host (static, never expires).
    pub resolved_at_secs: Option<i64>,
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
        /// Selected protocol kind (None while protocol picker shown)
        protocol: Option<xray_tui_proto::proto_spec::ProtocolKind>,
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
    pub profile: Option<ParsedProfile>,
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
    /// A per-connection trace event from the in-process native core.
    NativeTrace(TraceEvent),
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
    /// Result from a speed test operation. `endpoint_id` + `protocol_id`
    /// together address exactly one `ProfileStats` row: protocol rows are
    /// shared across endpoints (identity dedup excludes host/port), so a
    /// protocol-only key would write the result onto the first endpoint that
    /// happens to own the protocol.
    SpeedTestResult {
        endpoint_id: i64,
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
    /// Used by `batch_then_real_ping` to switch from TcpPing→RealPing emoji.
    /// Endpoint-scoped for the same reason as `SpeedTestResult`: the protocol
    /// row is shared, the (endpoint, protocol) pair is the unique link.
    TestTypeUpdate {
        endpoint_id: i64,
        protocol_id: i64,
        test_type: TestType,
    },
    /// Batch-level progress for real ping.
    BatchProgress {
        total: u16,
        completed: u16,
    },
    /// Background whitelist files loaded; carries the ready checker.
    HostFeaturesLoaded(Arc<xray_tui_host_features::HostFeaturesChecker>),
    /// Background enrichment (DNS resolve / geo lookup / whitelist / outbound)
    /// produced fresh data for one endpoint.
    EndpointInfoUpdated {
        endpoint_id: i64,
        info: EndpointInfo,
    },
    /// Native-core routing decision/probe event (surfaced in Actions Log).
    Route(xray_tui_route::events::RouteEvent),
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteProfile(i64),
    DeleteProfiles(Vec<i64>),
    DeleteGroup(String),
    ClearGroup(String),
    ClearLogs,
    PurgeLogsDatabase,
    ClearStats,
    Quit,
}
