use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16};

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use toasty::{Deferred, Json};
use xray_tui_config::AppConfig;
use xray_tui_core::grpc_client;
use xray_tui_core::log_heed::HeedLogStorage;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{CorePool, CoreType};
use xray_tui_db::Database;
use xray_tui_db::hash::stable_hash;
use xray_tui_db::models::{
    ConfigType, Endpoint, EndpointGroup, EndpointId, Group, HostType, ProfileStats, Protocol,
    ProtocolId, PurgatoryView, RoutingRule, Security, TrafficStats, Transport,
};
use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{
    ConfigKind, EndpointEssentials, HostKind, ParsedProto, ProtoSpec, ProtocolConfig,
};

use crate::BackendUpdateStatus;
use crate::ops::{connect, events, ping, profiles, settings, subscriptions, updates};
use crate::types::{
    AppMode, ConfirmAction, CoreEvent, EndpointInfo, EndpointRow, LogLine, SettingsSection,
    SortColumn, SplitRightPane, Tab,
};

/// Global UI/connection state. The many `bool` fields are orthogonal flags
/// (one per independent UI behavior); a state machine would obscure them.
#[allow(clippy::struct_excessive_bools)]
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
    /// Anchor for multi-line selection in logs tab (offset from bottom).
    /// `None` = no active selection.
    pub log_select_anchor: Option<usize>,
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
    /// Pooled core for single-ping reuse (lazy — created on first single real ping).
    pub core_pool: Option<Arc<CorePool>>,
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
    /// Channel sender for core subprocess log lines (String lines from stdout/stderr).
    pub core_log_tx: Option<tokio::sync::mpsc::Sender<String>>,
    /// Whether initial logs have been loaded from heed into `log_cache` yet.
    pub logs_loaded: bool,

    /// Geo IP mmdb lookup (None until constructed; construction is cheap, no I/O).
    pub geo_ip: Option<Arc<xray_tui_geoip::GeoIp>>,
    /// Cached DNS resolver (construction is cheap, no I/O).
    pub dns_resolver: Option<Arc<xray_tui_dns::DnsResolver>>,
    /// Whitelist checker (loaded in background; None until `HostFeaturesLoaded`).
    pub host_features: Option<Arc<xray_tui_host_features::HostFeaturesChecker>>,
    /// Per-endpoint enrichment data; survives profile reloads.
    pub endpoint_info: HashMap<i64, EndpointInfo>,
    /// TTL (secs) for the DNS-resolution cache; default 300.
    pub dns_cache_ttl_secs: i64,
}

/// Derive the typed [`Transport`] embed from a protocol config: the transport
/// kind plus the exact [`TransportConfig`] payload for the deferred JSON
/// column. Only vless/vmess/trojan carry a transport field; the quic-family
/// protocols (hy/hy2/tuic) are [`TransportConfig::Quic`]; everything else is
/// TCP — mirroring the `ProtoSpec::transport_type` accessor.
fn transport_embed(config: &ProtocolConfig) -> Transport {
    let transport = match config {
        ProtocolConfig::Vless(c) => c.transport.clone(),
        ProtocolConfig::Vmess(c) => c.transport.clone(),
        ProtocolConfig::Trojan(c) => c.transport.clone(),
        ProtocolConfig::Hysteria2(_) | ProtocolConfig::Tuic(_) | ProtocolConfig::Hysteria1(_) => {
            TransportConfig::Quic
        }
        _ => TransportConfig::Tcp,
    };
    let r#type = transport
        .type_str()
        .parse::<xray_tui_proto::proto_spec::TransportType>()
        .unwrap_or(xray_tui_proto::proto_spec::TransportType::Tcp);
    Transport {
        r#type,
        data: Deferred::from(Json(transport)),
    }
}

/// Derive the typed [`Security`] embed from a protocol config: the security
/// kind plus the queryable `sni`/`fp`/`insecure` columns and the exact
/// [`xray_tui_proto::proto_spec::SecurityConfig`] payload for the deferred
/// JSON column. The columns are populated from the same `config.security()`
/// accessors the UI reads, so they are queryable without loading the
/// deferred `config` JSON.
fn security_embed(config: &ProtocolConfig) -> Security {
    let security = config.security().cloned().unwrap_or_default();
    let r#type = match security.type_str() {
        Some("tls") => xray_tui_proto::proto_spec::SecurityType::Tls,
        Some("reality") => xray_tui_proto::proto_spec::SecurityType::Reality,
        _ => xray_tui_proto::proto_spec::SecurityType::None,
    };
    Security {
        r#type,
        sni: security.sni().map(str::to_string),
        fp: security.fp().map(str::to_string),
        insecure: security.insecure(),
        data: Deferred::from(Json(security)),
    }
}

/// Build a typed `Endpoint` row from parse-boundary endpoint essentials:
/// id = `stable_hash(host, port)`, host kind from the address family.
pub fn endpoint_from_essentials(ep: &EndpointEssentials) -> Endpoint {
    Endpoint {
        id: EndpointId::new(stable_hash(&ep.host, i64::from(ep.port))),
        host: ep.host.clone(),
        host_type: match ep.host_type {
            HostKind::Ipv4 => HostType::Ipv4,
            HostKind::Ipv6 => HostType::Ipv6,
            HostKind::Dns => HostType::Dns,
            HostKind::Undefined => HostType::Undefined,
        },
        port: ep.port,
        ports: ep.ports.clone(),
        parent_id: None,
        last_source: None,
        manual_protocol_override: None,
        resolved_as: Vec::new(),
        resolved_at: None,
        created_at: jiff::Timestamp::now(),
        links: Deferred::default(),
        group_links: Deferred::default(),
    }
}

/// Build a typed `Protocol` row from a parse result: id = `uid()`, with the
/// `config`/`transport.data`/`security.data` deferred JSON loaded so it is
/// ready for `Database::upsert_protocol`.
pub fn protocol_from_parsed(parsed: &ParsedProto) -> Protocol {
    Protocol {
        id: ProtocolId::new(parsed.uid()),
        sig: parsed.sig(),
        cred_hash: parsed.cred_hash(),
        proto_kind: parsed.protocol.proto_kind,
        transport: transport_embed(&parsed.protocol.config),
        security: security_embed(&parsed.protocol.config),
        config: Deferred::from(Json(parsed.protocol.config.clone())),
        created_at: jiff::Timestamp::now(),
        links: Deferred::default(),
    }
}

/// The per-pair `ProfileStats` link for one parsed endpoint.
fn link_from_parsed(parsed: &ParsedProto, ep: &EndpointEssentials) -> ProfileStats {
    let now = jiff::Timestamp::now();
    ProfileStats {
        protocol_id: ProtocolId::new(parsed.uid()),
        endpoint_id: EndpointId::new(stable_hash(&ep.host, i64::from(ep.port))),
        core_type: parsed.protocol.core_type,
        config_type: match parsed.protocol.config_type {
            ConfigKind::ShareUrl => ConfigType::ShareUrl,
            ConfigKind::Form => ConfigType::Form,
        },
        last_used_at: None,
        last_seen_at: now,
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
        created_at: now,
        updated_at: now,
        version: 1,
        protocol: Deferred::default(),
        endpoint: Deferred::default(),
    }
}

/// Convert a typed parse result into db rows: one `(Endpoint, Protocol,
/// ProfileStats)` triple per parsed endpoint. Encrypted configs that carry no
/// endpoint produce an empty vec (nothing to store).
pub fn parsed_to_rows(parsed: &ParsedProto) -> Vec<(Endpoint, Protocol, ProfileStats)> {
    parsed
        .endpoints
        .iter()
        .map(|ep| {
            let endpoint = endpoint_from_essentials(ep);
            (
                endpoint.clone(),
                protocol_from_parsed(parsed),
                link_from_parsed(parsed, ep),
            )
        })
        .collect()
}

/// Persist a parsed protocol as typed rows: one endpoint per parsed endpoint,
/// one shared protocol row, one per-pair link, plus the endpoint-group link
/// when `group_id` is `Some`. Returns the number of endpoints persisted.
///
/// Dedup is natural: endpoint ids (`stable_hash(host, port)`) and protocol
/// ids (`uid()`) are deterministic, so re-imports update the existing rows
/// instead of duplicating — profiles missing from a later fetch keep their
/// old `last_seen_at` and age into the Stale view (old orphan-to-purgatory
/// semantics via the typed staleness clock).
///
/// # Errors
///
/// Propagates the first failed upsert.
pub async fn persist_parsed(
    db: &Database,
    parsed: &ParsedProto,
    group_id: Option<&str>,
    core_override: Option<xray_tui_proto::proto_spec::CoreType>,
) -> Result<usize, xray_tui_db::DatabaseError> {
    let rows = parsed_to_rows(parsed);
    let mut count = 0usize;
    for (endpoint, protocol, mut link) in rows {
        if let Some(core) = core_override {
            link.core_type = core;
        }
        db.upsert_endpoint(&endpoint).await?;
        db.upsert_protocol(&protocol).await?;
        db.upsert_link(&link).await?;
        if let Some(gid) = group_id {
            db.upsert_endpoint_group_link(&EndpointGroup {
                endpoint_id: endpoint.id,
                group_id: gid.to_string(),
                last_seen_at: link.last_seen_at,
                sort_order: None,
                endpoint: Deferred::default(),
                group: Deferred::default(),
            })
            .await?;
        }
        count += 1;
    }
    Ok(count)
}

/// A link carries a persisted failure marker (`error.kind` — the old
/// in-memory round maps were replaced by persisted errors in T17). Drives the
/// "Remove Bad Servers" flow.
#[must_use]
pub const fn link_is_failed(link: &ProfileStats) -> bool {
    link.error.is_some()
}

/// Load a `Protocol` row with its deferred `config`/`transport.data`/
/// `security.data` JSON included. Default read paths exclude deferred columns
/// (the `EndpointRow` list ships unloaded `Protocol`s); `ConfigBuilder::build`
/// and `Database::upsert_protocol` require them loaded. Returns `Ok(None)`
/// when no such protocol row exists.
pub async fn load_protocol_with_config(
    db: &Database,
    id: ProtocolId,
) -> Result<Option<Protocol>, xray_tui_db::DatabaseError> {
    let mut conn = db.connection().await?;
    let protocol = Protocol::filter_by_id(id)
        .include(Protocol::fields().config())
        .include(Protocol::fields().transport().data())
        .include(Protocol::fields().security().data())
        .first()
        .exec(&mut conn)
        .await?;
    Ok(protocol)
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
            log_select_anchor: None,
            sort_column: SortColumn::Address,
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
            core_pool: None, // lazily created on first single real ping
            batch_progress: None,
            term_height: Cell::new(80),
            heed_storage: None,
            last_seen_log_ns: 0,
            known_targets: Vec::new(),
            selected_targets: Vec::new(),
            last_heed_poll: std::time::Instant::now(),
            log_sender_tx: None,
            core_log_tx: None,
            logs_loaded: false,
            geo_ip: None,
            dns_resolver: None,
            host_features: None,
            endpoint_info: HashMap::new(),
            dns_cache_ttl_secs: 300,
        };
        // Cheap constructors — no I/O until first lookup.
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui");
        state.geo_ip = Some(Arc::new(xray_tui_geoip::GeoIp::new(
            config_dir.join("GeoLite2-City.mmdb"),
        )));
        state.dns_resolver = Some(Arc::new(xray_tui_dns::DnsResolver::new(
            config_dir.join("dns-cache"),
        )));
        state.dns_cache_ttl_secs = state
            .db
            .get_dns_settings()
            .await
            .ok()
            .flatten()
            .and_then(|d| d.cache_ttl_secs)
            .unwrap_or(300);
        state.reload_profiles().await;
        state.reload_groups().await;
        state.spawn_auto_update();
        // Background whitelist load (downloads missing files). Result arrives
        // via CoreEvent::HostFeaturesLoaded → state.host_features.
        let tx = state.core_event_tx.clone();
        tokio::spawn(async move {
            let w = config_dir.join("whitelist");
            match xray_tui_host_features::HostFeaturesChecker::load(
                &w.join("whitelist.txt"),
                &w.join("ipwhitelist.txt"),
                &w.join("cidrwhitelist.txt"),
            )
            .await
            {
                Ok(c) => {
                    if let Some(t) = tx {
                        let _ = t.try_send(CoreEvent::HostFeaturesLoaded(Arc::new(c)));
                    }
                }
                Err(e) => tracing::warn!(target: "tui::state", "whitelist load failed: {e}"),
            }
        });
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
            Err(e) => {
                tracing::error!(target: "tui::state::log_init", "Failed to load initial logs: {e}");
            }
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

    pub fn filtered_len(&self) -> usize {
        profiles::filtered_len(self)
    }

    pub const fn cycle_purgatory_view(&mut self) {
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
    ///   1. Per-pair override (`link.core_type` — resolved at parse time)
    ///   2. Per-protocol config override (`config.core.protocol_core_overrides`)
    ///   3. Hardcoded auto-detection (`core_for_protocol` via `resolve_core`)
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
    pub const fn is_on_sub_row(&self) -> bool {
        profiles::is_on_sub_row(self)
    }
    /// Get the protocol ID for the currently selected sub-row.
    pub fn selected_sub_protocol_id(&self) -> Option<i64> {
        profiles::selected_sub_protocol_id(self)
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
    pub fn clone_profile(&mut self, id: i64) {
        profiles::clone_profile(self, id);
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
    pub fn move_profile_up(&mut self) {
        profiles::move_profile_up(self);
    }
    pub fn move_profile_down(&mut self) {
        profiles::move_profile_down(self);
    }
    pub async fn set_active(&mut self, id: &str) {
        profiles::set_active(self, id).await;
    }
    pub async fn set_protocol_default(&mut self, endpoint_id: i64, protocol_id: i64) {
        profiles::set_protocol_default(self, endpoint_id, protocol_id).await;
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
    /// Remove profiles whose links carry a persisted failure marker
    /// (`error.kind` — the round-map labels were replaced by persisted errors
    /// in T17; batch machinery is removed until T19 rebuilds it).
    pub async fn remove_failed_servers(&mut self) {
        ping::remove_failed_servers(self).await;
    }

    /// Poll core event channel and update state accordingly.
    ///
    /// Returns `true` when an event was handled — callers should redraw
    /// immediately rather than waiting for the idle refresh cadence.
    #[must_use]
    pub async fn poll_core_events(&mut self) -> bool {
        events::poll_core_events(self).await
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
        self.log_select_anchor = None;
        // Set cursor to now so poll_new_logs doesn't re-read old entries
        self.last_seen_log_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.confirmation = None;
        self.log_trace("info", "tui::state", "Logs cleared");
    }

    /// Clear entire log database (heed) in addition to the in-memory cache.
    /// Note: there may be a brief race with the background log writer;
    /// any messages already in the mpsc channel will be written after the clear
    /// and may reappear on next scroll load.
    pub fn purge_logs_database(&mut self) {
        self.log_cache.clear();
        self.log_scroll = 0;
        self.log_select_anchor = None;
        self.last_seen_log_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        // Clear the heed database in the background (best-effort)
        if let Some(ref heed) = self.heed_storage {
            let heed = heed.clone();
            tokio::spawn(async move {
                if let Err(e) = heed.clear_all_async().await {
                    tracing::error!(target: "tui::state", "Failed to clear log database: {e}");
                }
            });
        }
        self.confirmation = None;
        self.log_trace("info", "tui::state", "Log database cleared");
    }

    /// Clear all server stats (traffic counters).
    pub async fn clear_all_stats(&mut self) {
        if let Err(e) = self.db.clear_all_stats().await {
            self.log_trace(
                "error",
                "tui::state",
                &format!("Failed to clear stats: {e}"),
            );
            return;
        }
        self.endpoints_gen = self.endpoints_gen.wrapping_add(1);
        self.filter_cache_valid.set(false);
        self.confirmation = None;
        self.log_trace("info", "tui::state", "All stats cleared");
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
