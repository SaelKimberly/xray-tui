use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16};

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use xray_tui_config::AppConfig;
use xray_tui_config::import_export::{Profile, ValidationSummary, encode_profile_spec};
use xray_tui_core::grpc_client;
use xray_tui_core::log_heed::HeedLogStorage;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{CoreType, resolve_core};
use xray_tui_db::models::{
    Endpoint, Group, PingResultUpdate, ProfileExtension, ProtocolRow, PurgatoryView, RoutingRule,
};
use xray_tui_db::{Database, stable_hash};

use crate::BackendUpdateStatus;
use crate::format_now;
use crate::ops::{connect, events, ping, profiles, settings, subscriptions, updates};
use crate::types::{
    AppMode, ConfirmAction, CoreEvent, EndpointRow, LogLine, SettingsSection, SortColumn,
    SplitRightPane, Tab,
};
use crate::ui::settings::PROTOCOL_CORE_DEFS;

pub struct AppState {
    pub db: Arc<Database>,
    pub config: AppConfig,
    /// Currently selected theme name from config or UI selection.
    pub theme_name: ratatui_themes::ThemeName,
    pub current_tab: Tab,
    pub endpoints: Vec<EndpointRow>,
    /// Cached filtered/sorted profile indices for performance.
    pub cached_filtered_indices: RefCell<Vec<usize>>,
    pub filter_cache_valid: Cell<bool>,
    /// Generation counter bumped on every profile mutation.
    /// Used to skip redundant reloads.
    pub endpoints_gen: u64,
    /// Cached groups for subscriptions/settings UI.
    pub groups: Vec<Group>,

    pub purgatory_view: PurgatoryView,
    /// Purgatory TTL in seconds (default 7 days).
    pub purgatory_ttl_secs: i64,
    /// Purgatory retention in seconds (default 30 days).
    pub purgatory_retention_secs: i64,
    pub selected_index: usize,
    /// Optional protocol sub-row index for expanded endpoint tree navigation.
    pub selected_sub: Option<usize>,
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
    pub multi_select: HashSet<i64>,
    pub clipboard: Option<String>,
    pub confirmation: Option<ConfirmAction>,
    pub updating_groups: HashSet<String>,
    /// Profile IDs currently being tested
    pub testing_profiles: HashSet<i64>,
    /// Which test type is currently running per profile (for display).
    pub testing_details: HashMap<i64, TestType>,
    /// Cached update status for both backends.
    pub update_status: HashMap<CoreType, BackendUpdateStatus>,
    pub actions_compact: bool,
    pub connected_protocol_id: Option<i64>,
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

/// Convert a `Profile` to `Endpoint` + `ProtocolRow` (thin wrapper over ParsedProtocol conversion).
pub fn profile_to_endpoint_protocol(profile: &Profile) -> (Endpoint, ProtocolRow) {
    let parsed = xray_tui_config::import_export::ParsedProtocol {
        host: profile.address.clone(),
        port: profile.port as u16,
        host_type: if profile.address.parse::<std::net::IpAddr>().is_ok() {
            if profile.address.contains(':') {
                "ipv6".into()
            } else {
                "ipv4".into()
            }
        } else {
            "dns".into()
        },
        config_type: profile.config_type,
        proto_kind: profile.proto_kind.clone(),
        sig: profile.sig,
        cred_hash: profile.cred_hash,
        spec_blob: profile.spec_blob.clone(),
        core_type: profile.core_type.clone(),
        transport: profile.transport.clone(),
        security: profile.security.clone(),
        remarks: profile.remarks.clone(),
        created_at: profile.created_at,
    };
    parsed_to_endpoint_protocol(&parsed)
}

/// Convert a `ParsedProtocol` directly to `Endpoint` + `ProtocolRow` (no Profile intermediary).
pub fn parsed_to_endpoint_protocol(
    parsed: &xray_tui_config::import_export::ParsedProtocol,
) -> (Endpoint, ProtocolRow) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let host_type = if parsed.host_type.is_empty() {
        if parsed.host.parse::<std::net::IpAddr>().is_ok() {
            if parsed.host.contains(':') {
                "ipv6"
            } else {
                "ipv4"
            }
        } else {
            "dns"
        }
    } else {
        &parsed.host_type
    }
    .to_string();
    let endpoint = Endpoint {
        id: xray_tui_db::hash::stable_hash(&parsed.host, i64::from(parsed.port)),
        host: parsed.host.clone(),
        host_type,
        port: i32::from(parsed.port),
        port_spec_str: None,
        parent_id: None,
        last_source: None,
        created_at: now,
        manual_protocol_override: None,
    };
    let protocol = ProtocolRow {
        id: parsed.sig ^ parsed.cred_hash,
        endpoint_id: endpoint.id,
        sig: parsed.sig,
        cred_hash: parsed.cred_hash,
        proto_kind: parsed.proto_kind.clone(),
        spec_blob: parsed.spec_blob.clone(),
        config_type: parsed.config_type,
        core_type: parsed.core_type.clone(),
        transport: parsed.transport.clone(),
        security: parsed.security.clone(),
        remarks: parsed.remarks.clone(),
        created_at: now,
        last_seen_at: now,
        endpoint: Default::default(),
        extension: Default::default(),
        server_stat: Default::default(),
    };
    (endpoint, protocol)
}
impl AppState {
    pub async fn new(db: Arc<Database>, config: AppConfig) -> Self {
        let theme_name = config.theme_name;
        let (core_tx, core_rx) = tokio::sync::mpsc::channel(65536);
        let purgatory_ttl_secs = (config.purgatory.ttl_days * 86400) as i64;
        let purgatory_retention_secs = (config.purgatory.retention_days * 86400) as i64;
        let mut state = Self {
            db,
            config,
            theme_name,
            current_tab: Tab::Profiles,
            update_status: HashMap::new(),
            endpoints: Vec::new(),
            cached_filtered_indices: RefCell::new(Vec::new()),
            filter_cache_valid: Cell::new(true),
            endpoints_gen: 0,
            groups: Vec::new(),
            purgatory_view: PurgatoryView::Active,
            purgatory_ttl_secs,
            purgatory_retention_secs,

            selected_sub: None,
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
            connected_protocol_id: None,
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
        profiles::reload_profiles(self).await;
    }

    pub async fn reload_groups(&mut self) {
        profiles::reload_groups(self).await;
    }
    pub async fn reload_routing_rules(&mut self) {
        profiles::reload_routing_rules(self).await;
    }

    pub fn filtered_profiles(&self) -> impl Iterator<Item = &EndpointRow> {
        profiles::filtered_profiles(self)
    }

    fn compute_filtered_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    let address = row.endpoint.host.clone();
                    let port = row.endpoint.port.to_string();
                    if !address.to_lowercase().contains(&q) && !port.contains(&q) {
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

        let asc = self.sort_ascending;
        indices.sort_by(|&a, &b| {
            let a_row = &self.endpoints[a];
            let b_row = &self.endpoints[b];
            let cmp = match self.sort_column {
                SortColumn::ConfigType => a_row
                    .active_protocol()
                    .config_type
                    .cmp(&b_row.active_protocol().config_type),
                SortColumn::Remarks => {
                    let a_rem = a_row.active_protocol().remarks.clone().unwrap_or_default();
                    let b_rem = b_row.active_protocol().remarks.clone().unwrap_or_default();
                    a_rem.cmp(&b_rem)
                }
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

    pub fn filtered_len(&self) -> usize {
        profiles::filtered_len(self)
    }

    pub fn cycle_purgatory_view(&mut self) {
        profiles::cycle_purgatory_view(self);
    }
    /// Get the first group ID for new profile assignments (no active group filter).
    pub fn first_group_id(&self) -> String {
        self.groups
            .first()
            .map(|g| g.id.clone())
            .unwrap_or_default()
    }

    /// Log to the TUI log buffer AND the actions panel.
    ///
    /// Sends directly to the heed storage channel and the TUI event channel,
    /// bypassing the tracing subscriber (tracing's `target:` requires a
    /// `&'static str` literal at compile time, so a runtime target parameter
    /// cannot set event metadata target).
    pub fn log_trace(&mut self, level: &str, target: &str, message: &str) {
        use std::time::SystemTime;

        #[allow(
            clippy::cast_possible_truncation,
            reason = "nanos since epoch fits u64 (584yr range)"
        )]
        let timestamp_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let log_msg = xray_tui_core::log_heed::LogMessage {
            level: level.to_string(),
            target: target.to_string(),
            message: message.to_string(),
            timestamp_nanos,
        };

        // Send to heed storage (via background batched writer)
        if let Some(sender) = &self.log_sender_tx {
            let _ = sender.send(log_msg);
        }

        // Send to actions panel
        if let Some(tx) = &self.core_event_tx {
            let _ = tx.try_send(crate::CoreEvent::TuiLog {
                target: target.to_string(),
                level: level.to_string(),
                message: message.to_string(),
            });
        }
    }
    /// Resolve which core a profile row should use, considering (in order):
    /// 1. Per-profile override (`row.active_protocol().core_type`)
    /// 2. Per-protocol config override (`config.core.protocol_core_overrides`)
    /// 3. Hardcoded auto-detection (`core_for_protocol` via `resolve_core`)
    /// Resolve which core a profile row should use, considering (in order):
    /// 1. Per-profile override (`row.active_protocol().core_type`)
    /// 2. Per-protocol config override (`config.core.protocol_core_overrides`)
    /// 3. Hardcoded auto-detection (`core_for_protocol` via `resolve_core`)
    pub fn resolved_core(&self, row: &EndpointRow) -> CoreType {
        profiles::resolved_core(self, row)
    }
    // ── CRUD operations ──────────────────────────────────────────────────

    pub fn start_add_server(&mut self) {
        profiles::start_add_server(self);
    }
    pub async fn start_edit_profile(&mut self, id: &str) {
        profiles::start_edit_profile(self, id).await;
    }

    pub fn selected_profile_id(&self) -> Option<i64> {
        profiles::selected_profile_id(self)
    }

    pub fn toggle_expand(&mut self) {
        profiles::toggle_expand(self);
    }
    pub fn collapse_expand(&mut self) {
        profiles::collapse_expand(self);
    }
    pub fn nav_protocol_down(&mut self) -> bool {
        profiles::nav_protocol_down(self)
    }
    pub fn nav_protocol_up(&mut self) -> bool {
        profiles::nav_protocol_up(self)
    }
    pub fn is_on_sub_row(&self) -> bool {
        profiles::is_on_sub_row(self)
    }
    /// Get the protocol ID for the currently selected sub-row.
    pub fn selected_sub_protocol_id(&self) -> Option<i64> {
        profiles::selected_sub_protocol_id(self)
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
        // Form-created profiles cannot compute uid via ProtoSpec (no URL was parsed).
        // Use random i64 PK instead of deterministic sig ^ cred_hash.
        // This breaks the uid = sig ^ cred_hash convention for form profiles,
        // but is safe because form profiles never participate in URL-based
        // dedup (they have no proto_kind matchable to a ProtoSpec variant).
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let rand_bits = RandomState::new().build_hasher().finish();
        let uid: i64 = (now ^ rand_bits) as i64;
        // sig and cred_hash are meaningless for form profiles but kept non-zero
        // to avoid any sentinel-value confusion in DB queries.
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
            transport: network.clone(),
            security,
            created_at: now as i64,
            remarks: remarks.clone(),
        };
        let mut extra = serde_json::Map::new();
        if let Some(v) = &remarks {
            extra.insert("remarks".into(), serde_json::Value::String(v.clone()));
        }
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

    pub async fn confirm_add_server(&mut self) {
        profiles::confirm_add_server(self).await;
    }

    pub async fn confirm_edit_server(&mut self) {
        profiles::confirm_edit_server(self).await;
    }

    pub fn cancel_form(&mut self) {
        settings::cancel_form(self);
    }

    // ── Settings helpers ──────────────────────────────────────────────────

    pub fn enter_settings(&mut self) {
        settings::enter_settings(self);
    }

    async fn build_settings_fields(&self, section: SettingsSection) -> Vec<(String, String)> {
        use crate::SettingsSection::{
            Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
            Subscriptions, SystemProxy, Tun, Updates,
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
                    (
                        "skip_cert_verify".into(),
                        self.config.core.skip_cert_verify.to_string(),
                    ),
                    (
                        "clash_mixin".into(),
                        self.config.clash_mixin.clone().unwrap_or_default(),
                    ),
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
                    ("protocol".into(), self.config.mux.protocol.clone()),
                    (
                        "max_connections".into(),
                        self.config
                            .mux
                            .max_connections
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "min_streams".into(),
                        self.config
                            .mux
                            .min_streams
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "max_streams".into(),
                        self.config
                            .mux
                            .max_streams
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    ),
                    (
                        "padding".into(),
                        if self.config.mux.padding {
                            "true".into()
                        } else {
                            "false".into()
                        },
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
                    ("geoip_url".into(), self.config.geo.geoip_url.clone()),
                    ("geosite_url".into(), self.config.geo.geosite_url.clone()),
                    (
                        "geo_auto_update".into(),
                        self.config.geo.auto_update.to_string(),
                    ),
                    (
                        "geo_update_interval".into(),
                        self.config.geo.update_interval_hours.to_string(),
                    ),
                ]
            }
            Logging => {
                vec![
                    (
                        "log_ttl_secs".into(),
                        humantime::format_duration(*self.config.logging.ttl_secs).to_string(),
                    ),
                    (
                        "log_to_file".into(),
                        self.config.logging.log_to_file.to_string(),
                    ),
                    (
                        "log_file_path".into(),
                        self.config.logging.log_file_path.clone(),
                    ),
                ]
            }
            Subscriptions => vec![],
        }
    }

    fn apply_settings_fields(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        use crate::SettingsSection::{
            Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
            Subscriptions, SystemProxy, Tun, Updates,
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
                self.config.core.skip_cert_verify = get_str("skip_cert_verify") == "true";
                self.config.clash_mixin = {
                    let v = get_str("clash_mixin");
                    if v.is_empty() {
                        None
                    } else {
                        Some(v.to_owned())
                    }
                };
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
                if !get_str("protocol").is_empty() {
                    self.config.mux.protocol = get("protocol");
                }
                self.config.mux.max_connections = get_str("max_connections").parse::<u8>().ok();
                self.config.mux.min_streams = get_str("min_streams").parse::<u8>().ok();
                self.config.mux.max_streams = get_str("max_streams").parse::<u16>().ok();
                self.config.mux.padding = get_str("padding") == "true";
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
                if let Ok(v) = get_str("tcp_ping_concurrency").parse::<usize>() {
                    self.config.speed_test.tcp_ping_concurrency = v.max(1);
                }
                if !get_str("geoip_url").is_empty() {
                    self.config.geo.geoip_url = get("geoip_url");
                }
                if !get_str("geosite_url").is_empty() {
                    self.config.geo.geosite_url = get("geosite_url");
                }
                self.config.geo.auto_update = get_str("geo_auto_update") == "true";
                if let Ok(v) = get_str("geo_update_interval").parse::<u64>() {
                    self.config.geo.update_interval_hours = v;
                }
            }
            // Dns and Routing are handled separately (DB-backed)
            Dns | Routing | Updates | Subscriptions => {}
            Logging => {
                if let Ok(d) = humantime::parse_duration(get_str("log_ttl_secs")) {
                    *self.config.logging.ttl_secs = d;
                }
                self.config.logging.log_to_file = get_str("log_to_file") == "true";
                if !get_str("log_file_path").is_empty() {
                    self.config.logging.log_file_path = get("log_file_path");
                }
            }
        }
    }

    pub async fn build_right_pane(&mut self, section: SettingsSection) -> SplitRightPane {
        settings::build_right_pane(self, section).await
    }
    pub fn save_settings_form(&mut self, section: SettingsSection, fields: &[(String, String)]) {
        settings::save_settings_form(self, section, fields);
    }
    pub async fn save_routing_rule(
        &mut self,
        rule_id: Option<String>,
        fields: &[(String, String)],
    ) {
        settings::save_routing_rule(self, rule_id, fields).await;
    }
    pub async fn save_dns_settings(&mut self, fields: &[(String, String)]) {
        settings::save_dns_settings(self, fields).await;
    }
    pub async fn delete_profile(&mut self, id: i64) {
        profiles::delete_profile(self, id).await;
    }
    pub async fn clone_profile(&mut self, id: i64) {
        profiles::clone_profile(self, id).await;
    }
    pub fn toggle_multi_select(&mut self, id: i64) {
        profiles::toggle_multi_select(self, id);
    }
    pub fn import_url(&mut self, url: &str) {
        profiles::import_url(self, url);
    }
    pub fn start_batch_import(&mut self, urls: &[String]) {
        profiles::start_batch_import(self, urls);
    }
    pub async fn confirm_batch_import(&mut self) {
        profiles::confirm_batch_import(self).await;
    }
    pub async fn move_profile_up(&mut self) {
        profiles::move_profile_up(self).await;
    }
    pub async fn move_profile_down(&mut self) {
        profiles::move_profile_down(self).await;
    }
    pub async fn set_active(&mut self, id: &str) {
        profiles::set_active(self, id).await;
    }
    pub fn connect_to_profile(&mut self, protocol_id: i64) {
        connect::connect_to_profile(self, protocol_id);
    }
    pub fn disconnect(&mut self) {
        connect::disconnect(self);
    }
    pub fn start_tcp_ping(&mut self, protocol_id: i64) {
        ping::start_tcp_ping(self, protocol_id);
    }
    pub fn start_real_ping(&mut self, protocol_id: i64) {
        ping::start_real_ping(self, protocol_id);
    }
    pub fn start_speed_test(&mut self, protocol_id: i64) {
        ping::start_speed_test(self, protocol_id);
    }
    pub fn start_udp_test(&mut self, protocol_id: i64) {
        ping::start_udp_test(self, protocol_id);
    }
    pub fn stop_speed_test(&mut self) {
        ping::stop_speed_test(self);
    }
    pub fn start_batch_ping(&mut self) {
        ping::start_batch_ping(self);
    }
    pub fn start_batch_then_real_ping(&mut self) {
        ping::start_batch_then_real_ping(self);
    }
    /// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
    /// Uses DB-backed `ping_sessions` table for queue management.
    /// Phase 1 drains fast-pingable profiles quickly; Phase 2 handles remaining via temp core.
    #[allow(clippy::needless_collect)]
    pub fn start_batch_sieve(&mut self, real_ping_enabled: bool) {
        ping::start_batch_sieve(self, real_ping_enabled);
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
                    protocol_id: r.protocol_id,
                    delay: Some(ms),
                    speed: None,
                    sort_order: None,
                    ip_info: r.ip_info.clone(),
                    protocol_row: Default::default(),
                })
            })
            .collect();
        let _ = db
            .batch_flush_ping_buffer(batch_id, &batch, &extensions)
            .await;
    }
    /// Remove profiles whose extension.delay == Some(-1) (failed TCP ping).
    pub async fn remove_failed_servers(&mut self) {
        ping::remove_failed_servers(self).await;
    }

    /// Poll core event channel and update state accordingly.
    pub async fn poll_core_events(&mut self) {
        events::poll_core_events(self).await;
    }
    // ── Group management ─────────────────────────────────────────────

    pub fn start_add_group(&mut self) {
        subscriptions::start_add_group(self);
    }

    pub fn start_edit_group(&mut self, group_id: &str) {
        subscriptions::start_edit_group(self, group_id);
    }

    pub async fn confirm_add_group(&mut self) {
        subscriptions::confirm_add_group(self).await;
    }

    pub async fn confirm_edit_group(&mut self) {
        subscriptions::confirm_edit_group(self).await;
    }

    pub async fn delete_group(&mut self, group_id: &str) {
        subscriptions::delete_group(self, group_id).await;
    }

    pub async fn clear_group(&mut self, group_id: &str) {
        subscriptions::clear_group(self, group_id).await;
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

    /// Clear all server stats (traffic counters).
    pub async fn clear_all_stats(&mut self) {
        if let Err(e) = self.db.clear_all_stats().await {
            self.log_trace("error", "tui", &format!("Failed to clear stats: {e}"));
            return;
        }
        self.endpoints_gen = self.endpoints_gen.wrapping_add(1);
        self.filter_cache_valid.set(false);
        self.confirmation = None;
        self.log_trace("info", "tui", "All stats cleared");
    }

    // ── Subscription update ──────────────────────────────────────────

    pub fn update_group_subscriptions(&mut self, group_id: &str) {
        subscriptions::update_group_subscriptions(self, group_id);
    }

    pub fn update_all_subscriptions(&mut self) {
        subscriptions::update_all_subscriptions(self);
    }

    /// Spawn async task to check for backend updates on startup or manual trigger.
    pub fn spawn_update_check(&mut self) {
        updates::spawn_update_check(self);
    }

    /// Spawn async task to download and install an update for the given core.
    pub fn spawn_update_download(&mut self, core_type: CoreType) {
        updates::spawn_update_download(self, core_type);
    }

    /// Start a background task to check and update subscriptions.
    pub fn spawn_auto_update(&mut self) {
        subscriptions::spawn_auto_update(self);
    }
}
