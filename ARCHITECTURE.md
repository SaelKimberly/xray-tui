# xray-tui Architecture

## Crate Dependency Graph

```
xray-tui (bin)
  ├── xray-tui-core     (Protocol, CoreType, resolve_core, config_builder, process, log_heed)
  │     └── xray-tui-proto  (ProtocolConfig types, Proto identity container, URL parsing, Clash YAML)
  ├── xray-tui-db       (toasty ORM, Database query methods, Model definitions)
  ├── xray-tui-config   (AppConfig load/save, import_export, forms, permissive_json)
  │     └── xray-tui-proto  (ProtocolConfig/Proto types for import/export round-trip)
  ├── xray-tui-dns      (DNSCrypt-stamp DNS resolution — enrichment pipeline)
  ├── xray-tui-geoip    (GeoLite2-City country/city lookup — enrichment pipeline)
  └── xray-tui-host-features  (SNI/IP/CIDR whitelist membership — enrichment pipeline)
```

## Crate Responsibilities

### xray-tui (binary crate)

Entry point at `crates/xray-tui/src/main.rs`. Creates the tokio async runtime, initializes all subsystems, enters the ratatui event loop.

**Shared state** (`crates/xray-tui/src/state.rs`):

```rust
pub struct AppState {
    pub db: Arc<Database>,
    pub config: AppConfig,
    /// Currently selected theme name.
    pub theme_name: ratatui_themes::ThemeName,
    pub current_tab: Tab,
    pub endpoints: Vec<EndpointRow>,
    pub cached_filtered_indices: RefCell<Vec<usize>>,
    pub filter_cache_valid: Cell<bool>,
    pub endpoints_gen: u64,
    pub groups: Vec<Group>,
    pub purgatory_view: PurgatoryView,
    pub purgatory_ttl_secs: i64,
    pub purgatory_retention_secs: i64,
    pub selected_index: usize,
    pub selected_sub: Option<usize>,
    pub log_scroll: usize,
    pub log_select_anchor: Option<usize>,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub connected_core: Option<CoreType>,
    pub connecting: bool,
    pub system_stats: Option<grpc_client::SysStats>,
    pub log_cache: VecDeque<LogLine>,
    pub log_has_older: bool,
    pub log_seek_home: bool,
    pub connection_error: Option<String>,
    pub core_event_rx: Option<mpsc::Receiver<CoreEvent>>,
    pub core_event_tx: Option<mpsc::Sender<CoreEvent>>,
    pub disconnect_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub previous_mode: Option<Box<AppMode>>,
    pub multi_select: HashSet<i64>,
    pub clipboard: Option<String>,
    pub confirmation: Option<ConfirmAction>,
    pub updating_groups: HashSet<String>,
    pub testing_profiles: HashSet<i64>,
    pub testing_details: HashMap<i64, TestType>,
    pub update_status: HashMap<CoreType, BackendUpdateStatus>,
    pub actions_compact: bool,
    pub connected_protocol_id: Option<i64>,
    pub speed_test_stop: Arc<AtomicBool>,
    pub last_test_tcp: Option<u64>,
    pub batch_progress: Option<Arc<(AtomicU16, AtomicU16)>>,
    pub last_test_real: Option<u64>,
    pub last_test_speed: Option<u64>,
    pub current_traffic_up: i64,
    pub current_traffic_down: i64,
    pub current_memory: u64,
    pub term_height: Cell<u16>,
    pub routing_rules: Vec<RoutingRule>,
    pub geo_ip: Option<Arc<xray_tui_geoip::GeoIp>>,
    pub dns_resolver: Option<Arc<xray_tui_dns::DnsResolver>>,
    pub host_features: Option<Arc<xray_tui_host_features::HostFeaturesChecker>>,
    pub endpoint_info: HashMap<i64, EndpointInfo>, // enrichment cache, survives reload_profiles
    pub dns_cache_ttl_secs: i64,                   // TTL for DNS-resolution cache; default 300
    pub shutdown_token: Arc<AtomicBool>,
    pub core_task_handle: Option<JoinHandle<()>>,
    pub heed_storage: Option<Arc<HeedLogStorage>>,
    pub last_seen_log_ns: u64,
    pub known_targets: Vec<String>,
    pub selected_targets: Vec<String>,
    pub last_heed_poll: std::time::Instant,
    pub log_sender_tx: Option<std::sync::mpsc::Sender<xray_tui_core::log_heed::LogMessage>>,
    pub logs_loaded: bool,
}

### CoreEvent Channel

```rust
pub enum CoreEvent {
    Connected(CoreType),
    Disconnected,
    Error(String),
    StatsError(String),
    StatsUpdate {
        protocol_id: i64,
        today_up: i64,
        today_down: i64,
        total_up: i64,
        total_down: i64,
    },
    SysStatsUpdate(grpc_client::SysStats),
    LogLine {
        level: String,
        target: String,
        message: String,
        timestamp_nanos: i64,
    },
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
    SpeedTestResult {
        protocol_id: i64,
        test_type: TestType,
        latency_ms: Option<u64>,
        speed_bps: Option<u64>,
        ip_info: Option<String>,
        error: Option<String>,
    },
    TestTypeUpdate {
        protocol_id: i64,
        test_type: TestType,
    },
    UpdateCheckResult {
        core_type: CoreType,
        current_version: Option<String>,
        latest_version: Option<String>,
        error: Option<String>,
    },
    UpdateDownloadProgress {
        core_type: CoreType,
        downloaded: u64,
        total: u64,
    },
    UpdateCompleted {
        core_type: CoreType,
        old_version: Option<String>,
        new_version: Option<String>,
        success: bool,
        error: Option<String>,
    },
    HostFeaturesLoaded(Arc<xray_tui_host_features::HostFeaturesChecker>),
    EndpointInfoUpdated {
        endpoint_id: i64,
        info: EndpointInfo,
    },
}
spawned `CoreManager` task and the TUI event loop. `poll_core_events()` is called each frame,
draining pending events and updating `AppState` fields (`connected_core`, `connecting`, `connection_error`).
The `disconnect_tx` oneshot channel signals the running core task to stop gracefully.

Methods previously in `lib.rs` extracted to `ops/` modules:
- `ops/connect.rs` — connect_to_profile, disconnect
- `ops/ping.rs` — start_batch_ping, start_batch_then_real_ping, stop_speed_test
- `ops/events.rs` — poll_core_events
- `ops/subscriptions.rs` — update_group_subscriptions, do_update_subscription
- `ops/updates.rs` — spawn_update_check, spawn_update_download
- `ops/settings.rs` — confirm_add_server, confirm_edit_server
- `ops/profiles.rs` — delete_profile, clone_profile, import_url, nav_protocol_up/down, toggle_expand
- `ops/enrich.rs` — background enrichment: spawn_dns_resolve (TTL-gated, `x` force), spawn_enrich_ip_hosts (startup seed), spawn_whitelist_pass (on HostFeaturesLoaded), spawn_outbound_enrich (real-ping exit IP), extract_sni, protocol_row_to_profile

**TUI Screens (modules under `crates/xray-tui/src/ui/`):**

- `profiles.rs` — 17-column endpoint rows (tree marker, indicator, #, Last Seen, Type, `[flag address:port][ip-flag sni-flag]=>{config}=>[outbound country]`) over a sortable DataTable; expanding an endpoint renders a rounded panel in the row's own height (IPs line + 10-col per-protocol sub-table: marker, hex id, last seen, last used, config type, delay, speed, traffic, outbound, country). Height-aware scroll offset (`compute_scroll_offset`) keeps expanded rows reachable; `panel_w` is viewport-capped so narrow terminals don't panic. Footer/overlays use `host:port` (remarks removed).
- `settings.rs` — Split-pane settings panel. Left: collapsible tree (SPLIT_SETTINGS_TREE) navigating by SettingsSection. Right: Form (SplitRightPane::Form), UpdateForm, or Empty. 14 sections: Core, GUI, Protocol Core, Inbound, Routing Rules, DNS, System Proxy, TUN, Mux/Fragment, Statistics, Updates, Speed Test, Logging, Subscriptions. Ctrl+W switches focus between tree and form panels. Routes/DNS persist to DB; all others to AppConfig JSON. Replaced per-section SettingsMode variants with unified Split { tree, focus, right } architecture.
- `groups.rs` — Subscription group overlay (list + add/edit forms) with update/delete actions. Accessed via `g` key from Profiles tab.
- `logs.rs` — Log viewer with source filtering (c/t toggles for core/TUI logs, v toggles validation/subscription logs)
- `actions_log.rs` — Live event log panel showing connection status, speed test results, core/TUI/app logs, traffic counters with color-coded levels. F1 toggles compact/full modes; auto-compacts on small terminals (<20 rows).
- `theme.rs` — ThemeStyles struct with static methods returning Style from a &Palette (container_border, container_title, hint, warning, error, success, tab_selected, etc.)
- `palette_bridge.rs` — Maps ratatui-themes ThemePalette (10 colors) to ratatui-cheese Palette (11 roles)
- `widgets/data_table.rs` — Reusable DataTable widget: sortable columns, multi-select, virtual-scrolled with themed scrollbar, DataTableRow trait (render takes `clip_bottom` so tall rows clip instead of overflowing)

**`speed_test.rs`** — Async speed test engine:
```rust
pub enum TestType { TcpPing, RealPing, SpeedTest, UdpTest }
pub struct RealPingResult { pub latency_ms: u64, pub ip_info: Option<String> }
pub enum SpeedTestError { Io, Timeout, Proxy, Http, InvalidAddress }
pub async fn tcp_ping(addr: &str, port: u16, test_timeout: Duration) -> Result<Duration, SpeedTestError>;
pub async fn real_ping(proxy: &str, port: u16, url: &str, retries: u32, test_timeout: Duration) -> Result<RealPingResult, SpeedTestError>;
pub async fn speed_test(proxy: &str, port: u16, url: &str, min_duration: Duration, max_duration: Duration) -> Result<u64, SpeedTestError>;
pub async fn udp_test(proxy: &str, port: u16, test_timeout: Duration) -> Result<Duration, SpeedTestError>;
```
tcp_ping connects directly to the target address. real_ping, speed_test, and udp_test route through the active SOCKS5 proxy.
`real_ping` sends up to `retries` HTTP GETs through SOCKS5, takes the fastest 2xx response, and optionally
fetches IP info (ISP/location) from a configurable `ip_api_url` through the same proxy. Returns `RealPingResult`
with both latency and IP metadata.
Results are sent via CoreEvent::SpeedTestResult (with optional `ip_info` field) and handled in poll_core_events(),
which updates the ProfileExtension (delay, ip_info) in memory and persists via upsert_profile_extension().

**Fast Ping (start_batch_ping)**: Uses `FastPingManager` to dispatch to the appropriate adapter (TCP, UDP, or QUIC) based on protocol. TcpPingAdapter supports all TCP-based protocols; UdpPingAdapter supports WireGuard and ShadowsocksR; QuicPingAdapter (optional feature) supports QUIC-enabled protocols. Falls through to RealPingManager for protocols without a matching adapter. Phase 1 drains fast-pingable profiles concurrently, bounded by `tcp_ping_concurrency` semaphore (default 20). Uses `ping_sessions` table for queue management with shared `batch_progress: Arc<(AtomicU16, AtomicU16)>` for status bar display. Supports cancellation via `speed_test_stop` flag.

**Batch-then-real-ping (start_batch_then_real_ping)**: Two-phase pipeline using `ping_sessions` table:
1. **Phase 1**: Creates ping_sessions records, runs Fast Ping on all visible profiles concurrently. Falls through to RealPingManager for protocols without adapter.
2. **Phase 2**: Wave-ordered real pings on ALL profiles from Phase 1. Wave ordering interleaves profiles from different host:port groups — one profile per unique target first (Wave 1), then remaining duplicates. Each profile gets its own **temp core + real ping** on a unique port allocated atomically from a shared allocator (`CorePool::port_allocator()`, one `Arc<AtomicU16>` exposed from the pool). Bounded by `real_ping_concurrency` semaphore. The same allocator serves both batch Phase 2 and individual real pings (no port collisions between flows); an RAII `BatchActiveGuard` disables warm-core pool reuse while a batch owns the allocator. The former `AppState.next_real_ping_port` field was removed — it duplicated the counter.

**Stop testing**: `speed_test_stop: Arc<AtomicBool>` on AppState. Set via menu ("Stop Testing" at index 10)
or hotkey `'s'`. Auto-resets when `testing_profiles` empties. `tested_pids`/`tcp_completed` sets prevent
double-emission of "Cancelled" for already-tested profiles. Status bar shows "■ Stopping..." in red while
stop flag is active.

**Configuration**: SpeedTestConfig in `AppConfig` includes `tcp_ping_concurrency` (default 20) and
`real_ping_concurrency` (default 5), both editable via Settings > Speed Test form.

### xray-tui-core (library crate)

`crates/xray-tui-core/src/lib.rs` — Facade + re-exports. Modules:

---

**`core_type.rs`** — Core type definitions
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreType {
    Xray,
    SingBox,
    Auto,
}
```

**`process.rs`** — Dual-backend subprocess lifecycle
```rust
pub struct CoreProcess {
    child: Option<Child>,
    config_path: PathBuf,
    pub core_type: CoreType,
}

/// Abstract interface for core process lifecycle.
#[async_trait]
pub trait CoreManager: Send + Sync {
    async fn start(
        &mut self,
        core_type: CoreType,
        config: &BackendConfig,
        binary_path: &Path,
        clash_mixin: Option<&serde_json::Value>,
    ) -> Result<(), ProcessError>;
    async fn stop(&mut self) -> Result<(), ProcessError>;
    fn is_running(&self) -> bool;
    fn running_core_type(&self) -> Option<CoreType>;
    fn sighup_reload(&self) -> Result<u32, ProcessError>;
    async fn rewrite_config(&self, config: &BackendConfig, clash_mixin: Option<&serde_json::Value>) -> Result<(), ProcessError>;
}

/// Real subprocess-backed implementation. `log_tx: Sender<String>` is required —
/// callers without a log consumer must pass a drained channel.
pub struct RealCoreManager {
    current: Option<CoreProcess>,
    config_dir: tempfile::TempDir,
    log_tx: Sender<String>,
}

/// Test double implementing the same trait (Option<String> error fields).
pub struct MockCoreManager { /* start_error, stop_error, is_running, ... */ }
```

`CoreManager::start` flow:
1. Resolve core binary path via `find_binary()`
2. Build config via `ConfigBuilder::build()`
3. Write JSON config to temp file in config_dir
4. Spawn `xray run -c <path>` or `sing-box run -c <path>`
8. Poll for readiness (check process hasn't exited early, max 10s)
9. Send `CoreEvent::Connected` on the event channel
10. Create `StatsProvider` via `grpc_client::create_stats_provider(core_type)`
11. Enter stats polling + wait loop: every 3s call `provider.query_stats()` for traffic deltas;
    every 3rd tick (~9s) also call `provider.get_sys_stats()`. Send `StatsUpdate`/`SysStatsUpdate`
    events. Loop exits when `stop_rx` receives the disconnect signal.
12. Kill child process + remove config file
13. Send `CoreEvent::Disconnected`


**`bin_manager.rs`** — Binary discovery and archive extraction
```rust
pub struct CoreBinInfo {
    pub name: &'static str,
    pub bin_names: &'static [&'static str],
    pub args_template: &'static str,
    pub archive_patterns: &'static [&'static str],
}

pub fn find_binary(core_type: CoreType, bin_dir: &Path) -> Option<PathBuf>;
pub fn get_core_info(core_type: CoreType) -> CoreBinInfo;
pub fn find_and_extract_archives(core_type: CoreType, bin_dir: &Path) -> Result<(), BinError>;
```
`find_binary` checks `bin_dir`/`core_type` first (managed install), then falls back to `which`.
`get_core_info` returns binary names (xray: `["xray"]`; sing-box: `["sing-box-client", "sing-box"]`)
and archive patterns for automatic extraction in dev environments.

---

**`config_builder/`** — Config builder module
- `mod.rs` — Dispatches to xray or sing-box builder based on core type
- `xray.rs` — Builds xray-core format JSON
- `singbox.rs` — Builds sing-box format JSON


pub enum BackendConfig {
    Xray(XrayConfig),
    SingBox(SingBoxConfig),
}

pub struct BuildParams {
    log_level: String,
    socks_port: u16,
    http_port: Option<u16>,
    listen: String,
    sniffing: bool,
}

pub struct ConfigBuilder;
impl ConfigBuilder {
    pub fn build(
        profile: &Profile,
        core_type: CoreType,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError>;
}
  "inbounds": [
    { "tag": "socks-in", "protocol": "socks", "listen": "127.0.0.1", "port": 10808 },
    { "tag": "api", "protocol": "dokodemo-door", "listen": "127.0.0.1", "port": 62789 }
  ],
  "outbounds": [
    { "tag": "proxy", "protocol": "vmess", "settings": { ... },
      "streamSettings": { ... }, "mux": { ... } },
    { "tag": "direct", "protocol": "freedom" },
    { "tag": "block", "protocol": "blackhole" }
  ],
  "routing": {
    "domainStrategy": "AsIs",
    "rules": [...],
    "balancers": [...]
  },
  "dns": { "servers": [...], "hosts": {...} },
  "stats": {},
  "api": { "tag": "api", "services": ["HandlerService", "LoggerService", "StatsService"] },
  "policy": {
    "levels": { "0": { "statsUserUplink": true, "statsUserDownlink": true } },
    "system": { "statsInboundUplink": true, "statsOutboundUplink": true }
  }
}
```

**Config builder — singbox.rs** produces the sing-box JSON format:
```json
{
  "log": { "level": "warn" },
  "dns": { ... },
  "inbounds": [{ "tag": "socks-in", "type": "socks", ... }],
  "outbounds": [
    { "tag": "proxy", "type": "tuic", "server": "...", "server_port": ...,
      "tls": { "enabled": true, ... } },
    { "tag": "direct", "type": "direct" },
    { "tag": "block", "type": "block" }
  ],
  "route": {
    "rules": [...],
    "final": "proxy"
  },
  "experimental": {
    "v2ray_api": {
      "listen": "127.0.0.1:62789",
      "stats": { "enabled": true, "outbounds": ["proxy", "direct"] }
    }
  }
}
```

Key differences from xray-core JSON:
- Protocol `type` field instead of `protocol` in xray-core outbound/inbound entries
- `route` key instead of `routing`
- `experimental.v2ray_api` block for stats (vs xray-core's `stats` + `api` + `policy`)
- Transport/TLS config differs: sing-box uses per-protocol `tls` sub-key vs xray-core's `streamSettings.security`
- No separate `policy` section (stats config lives under `experimental.v2ray_api`)

**`config_builder/clash_mixin.rs`** — Clash YAML overlay injection
```rust
pub fn parse_clash_mixin(path: &str) -> Result<serde_json::Value, MixinError>;
pub fn merge_mixin(config: &mut serde_json::Value, mixin: &serde_json::Value);
```
Reads Clash-compatible YAML, parses to JSON, and merges into sing-box config before writing. Supports JSON and YAML input formats (auto-detected by extension). 5 unit tests.

---

**`protocol_core_mapping.rs`** — Protocol → Core auto-resolution
```rust
fn resolve_core(protocol: ProtocolType, profile_override: Option<CoreType>) -> CoreType {
    match profile_override {
        Some(CoreType::Auto) | None => core_for_protocol(protocol),
        Some(core_type) => core_type,
    }
**`grpc_client.rs`** — gRPC StatsService abstraction

Proto definition in `crates/xray-tui-core/proto/stats.proto` (vendored sing-box stats proto), compiled
via `build.rs` using `tonic_build`. Package `experimental.v2rayapi`, 3 RPCs: `GetStats`, `QueryStats`,
`GetSysStats`.

```rust
pub const API_ENDPOINT: &str = "http://127.0.0.1:62789";

#[async_trait]
pub trait StatsProvider: Send + Sync {
    async fn query_stats(&self, pattern: &str, reset: bool) -> Result<Vec<Stat>, GrpcError>;
    async fn get_sys_stats(&self) -> Result<SysStats, GrpcError>;
    fn api_endpoint(&self) -> &str;
}
```

Two implementations — `XrayGrpcClient` and `SingBoxGrpcClient` — both connect to the same endpoint
(`127.0.0.1:62789`) and use the same `StatsServiceClient<Channel>`. They are separate types for
type-level distinction only; both backends expose the same V2Ray-compatible gRPC API.

Factory function:
```rust
pub async fn create_stats_provider(core_type: CoreType) -> Result<Box<dyn StatsProvider>, GrpcError>
```

Helpers: `format_bytes(i64) -> String`, `format_uptime(u32) -> String`.

---

**`subscription.rs`** — Subscription download and parsing
```rust
pub async fn update_subscription(url: &str, proxy: Option<&str>) -> Result<Vec<Profile>>;
```
Parses base64-encoded share URLs, plain URL lists, v2rayN subscription formats, and sing-box format.

**`updater.rs`** — Backend binary auto-update
Functions: `get_current_version()` (run `{core} version`, parse output), `get_latest_version()` (GitHub releases API), `download_release()` (streaming download via reqwest, progress tracking), `install_binary()` (extract archive → verify binary runs → .bak existing → copy all files → remove .bak on success, restore from .bak on failure).

**`speedtest.rs`** — Speed testing logic (same as single-core design)

**`import_export.rs`** — Share URL parsing and config export
```rust
pub fn parse_share_url(url: &str) -> Result<Profile>;
pub fn format_share_url(profile: &Profile) -> String;
pub fn export_client_config(profile: &Profile) -> Result<String>;
```
Ports format parsing from v2rayN's `Handler/Fmt/*.cs` files plus sing-box URI formats.

---

### xray-tui-db (library crate)

`crates/xray-tui-db/src/lib.rs` — toasty ORM database layer.

**Models** (defined via `#[derive(toasty::Model)]` in `models_toasty.rs`):
- `Endpoint` — server config; dedup key `sub_uid` (uid = sig ^ cred_hash); `resolved_as`/`resolved_at` DNS persistence
- `ProtocolRow` — per-protocol variant rows (many per endpoint); `last_used_at`/`last_seen_at`, `endpoint_id` index
- `EndpointGroup` — many-to-many Endpoint↔Group membership
- `Group` — subscription group with name, URL, sort order, is_system flag
- `ProfileExtension` — per-protocol test results (delay, speed, ip_info)
- `ServerStat` — traffic counters (today/total up/down as i64)
- `RoutingRule` — domain/IP/port matchers with outbound tag; sort-ordered
- `DnsSetting` — DNS resolver config
- `PingSession` — ping batch tracking (batch_id, status, ping_type, latency)

**Schema management**: toasty's `db.push_schema()` creates tables on first open — only when `PRAGMA user_version < 1` (toasty 0.9 uses `CREATE TABLE` without `IF NOT EXISTS`). Existing DBs migrate in place: `SCHEMA_VERSION = 3`, `ensure_column()` (pragma_table_info check + idempotent `ALTER TABLE ADD COLUMN`) for `protocol_rows.last_used_at`, `endpoints.resolved_as`/`resolved_at`, `dns_settings.cache_ttl_secs`; v3 adds the `protocol_rows.endpoint_id` index (`CREATE INDEX IF NOT EXISTS`, idempotent). All inside one explicit `conn.transaction()` + `tx.commit()` (bare multi-statement DDL on the pooled turso connection silently rolls back at drop). System groups created by `init_default_groups()`. Known quirk: toasty's `push_schema` leaves a cross-process SQLITE_BUSY write lock on the db file for the life of the process (external sqlite3 access blocked while the app runs; app's own single-pooled-connection ops unaffected).
**Log storage**: `TuiLogLayer` (in `main.rs`) captures `tracing::Event` emissions and sends to (a) `core_event_tx` for in-memory `log_cache` display and (b) `HeedLogStorage` via a non-blocking `std::sync::mpsc` channel. The `HeedLogStorage` (in `xray-tui-core::log_heed`) stores entries in an LMDB `logs` database keyed by big-endian u64 timestamp with postcard-encoded `LogMessage` values. A separate `targets` database tracks seen target strings. Batched writer (up to 100 msgs) runs in `spawn_blocking`; async read wrappers wrap LMDB reads in `spawn_blocking`. MapFull triggers auto-resize (1 GB default, doubles up to 8 GB) with backoff retry (50ms*(attempt+1), max 5) — the batch is retried after a successful resize, never dropped. Initial log loading is lazy (deferred to first Logs tab access).

### xray-tui-config (library crate)

`crates/xray-tui-config/src/lib.rs` — App config management, protocol form fields, share URL parsing.

**Modules added in Phase 2:**
- `forms.rs` — `FormFieldType`, `FormField`, `FieldSection`, `form_fields_for()` (27 protocols)
- `import_export.rs` — `parse_share_url()`, `format_share_url()` (14 protocol formats)

**App config** (JSON at `~/.config/xray-tui/config.json`): mirrors v2rayN's `Config.cs` fields plus dual-core settings (xray binary path, sing-box binary path, default core type).

**Import/export format parsers**:
- `vmess://` — Base64 JSON decoding
- `vless://` — URI parsing (scheme://uuid@host:port? params)
- `ss://` — SIP002 URI format
- `trojan://` — URI with password
- `socks://`, `http://`, `hysteria2://`, `hysteria://`, `tuic://`, `naive://`, `anytls://`, `shadowtls://`, `wireguard://` — Various URI schemes
- Subscription format auto-detection (base64 list, plain list, v2rayN, sing-box)
- Export share URL generation (mirrors `Fmt/*.cs` in v2rayN)

### xray-tui-dns (library crate)

`crates/xray-tui-dns/src/lib.rs` — secure DNS resolution through the DNSCrypt public resolver list.

```rust
pub struct DnsResolver { cache_dir: PathBuf, resolver: tokio::sync::OnceCell<TokioResolver> }
impl DnsResolver {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self;
    pub async fn lookup_ip(&self, hostname: &str, allow_ipv6: bool) -> anyhow::Result<Vec<IpAddr>>;
}
```

`lookup_ip` short-circuits IP literals (no DNS). Otherwise it lazily builds a `hickory_resolver` 0.26 `TokioResolver` configured from the DNSCrypt public-resolver list: `sdns://` stamps parsed by `dns-stamp-parser` (DnsPlain→UDP, DoH→HTTPS, DoT→TLS, DoQ→QUIC), filtered to NO_LOGS+NO_FILTER servers, with the resolver list cached as `dsncrypt.resolvers.txt` under the caller-supplied `cache_dir`. 500ms per-attempt timeout + RoundRobin ordering (ResolverOpts); the first-run DNSCrypt list download runs on a 10s reqwest deadline (a blocked network must not hang lookups forever — callers additionally wrap `lookup_ip` in an overall timeout). TLS cert roots come from hickory's `webpki-roots` feature — no direct rustls dependency. Init uses the OnceCell get-then-set pattern so a failed (offline) first init retries on the next call.

### xray-tui-geoip (library crate)

`crates/xray-tui-geoip/src/lib.rs` — IP → country/city lookup from a GeoLite2-City mmdb.

```rust
pub struct Location { pub country: String, pub city_en: Option<String> }
pub struct GeoIp { db_path: PathBuf, reader: tokio::sync::OnceCell<Arc<Reader<Vec<u8>>>> }
impl GeoIp {
    pub fn new(db_path: impl Into<PathBuf>) -> Self;
    pub async fn location_by_ip(&self, ip: IpAddr) -> anyhow::Result<Option<Location>>;
}
```

On first use downloads the mmdb (P3TERX GeoLite mirror) to `db_path` via reqwest — atomic write (tmp file + rename), partial-file cleanup on failure, hard download deadline; a corrupt/unreadable file at open triggers re-download (heal-on-corrupt); init is serialized via the OnceCell — then opens it with `maxminddb::Reader::open_readfile` (whole DB in RAM), and runs lookups + `decode_path` inside `spawn_blocking` (LookupResult borrows the Reader, so `Arc<Reader>` moves into the closure). Country is required; missing city yields `city_en: None`.

### xray-tui-host-features (library crate)

`crates/xray-tui-host-features/src/lib.rs` — whitelist membership feature extraction, ported from sub-healer's `WhitelistChecker`.

```rust
pub struct HostFeatures { pub sni_whitelisted: bool, pub ip_whitelisted: bool, pub cidr_whitelisted: bool }
pub struct HostFeaturesChecker { /* fastbloom BloomFilter + exact HashSet/interval backing */ }
impl HostFeaturesChecker {
    pub fn new(sni_path: &Path, ip_path: &Path, cidr_path: &Path) -> anyhow::Result<Self>;
    pub async fn load(sni_path: &Path, ip_path: &Path, cidr_path: &Path) -> anyhow::Result<Self>;
    pub async fn ensure_downloaded(sni_path: &Path, ip_path: &Path, cidr_path: &Path) -> anyhow::Result<()>;
    pub fn get_host_features(&self, server_name: &ServerName<'_>) -> HostFeatures;
}
```

`new` reads the three whitelist files (SNI hostnames, exact IPv4s, IPv4 CIDR ranges). `load` first
downloads any file missing from the hxehex/russia-mobile-internet-whitelist upstream (presence check
only, never re-downloads; hard download deadline, atomic tmp+rename write), mirroring the
geoip/dns download-if-missing convention. Lookups are
bloom-filter fast-negatives with exact verification (zero false positives): `DnsName` ServerNames →
SNI check, IPv4 → exact-IP + CIDR checks, IPv6/unknown → empty feature set. `fastbloom` is the only
crate-local dep (serde feature dropped — filters rebuilt from disk on `new()`).

## Data Flow: Connect to Proxy

```
User selects profile → hits Enter
        │
        ▼
resolve_core(profile.protocol, profile.core_type) → core_type
        │
        ▼
[Profile + Groups + Routing + DNS]
        │
        ▼
match core_type {
    Xray    → ConfigBuilderXray::build(profile, settings, routing, dns)
    SingBox → ConfigBuilderSingBox::build(profile, settings, routing, dns)
}
        │
        ▼
CoreManager::start(core_type, config)
  → if running_core_type != core_type: stop current
  → write JSON to temp file
  → spawn `xray run -c <path>` or `sing-box run -c <path>`
  → poll Child.status() every 500ms until running or timeout
        │
        ▼
match core_type {
    Xray | SingBox → GrpcStatsClient::connect(api_addr)
}
        │
        ▼
UI updates: status_bar shows "Connected [xray|sing-box]",
log panel shows core output, stats panel receives traffic updates
```

## Data Flow: Subscription Update
```
User clicks "Update subscription" for a group (or auto-update timer fires)
        │
        ▼
tokio::spawn(async move { do_update_subscription(...) })
  → 120s timeout wrapper (prevents silent hang)
  → tracing::info! checkpoints at each step for status bar visibility
        │
        ▼
HTTP GET subscription_url → download response body
        │
        ▼
Streaming base64 decoder → URL list
  → parse_share_url() for each URL → Vec<Profile>
  → validate_security() emits tracing::warn!(target: "validation", ...)
        │
        ▼
Enrich each Profile: compute sub_uid (rapidhash), set group_id, is_sub, sub_id
        │
        ▼
subscription_upsert_profiles() — single BEGIN DEFERRED transaction:
  1. INSERT OR REPLACE INTO profile_cores (dedup by sub_uid)
  2. INSERT INTO group_profiles ON CONFLICT(group_id, sub_uid) DO UPDATE
  3. INSERT OR IGNORE INTO group_profiles (ALL_GROUP_ID mirror)
  4. DELETE graveyard orphans promoted back to this group
        │
        ▼
move_orphans_to_graveyard() + purge_graveyard()
upsert_subscription() — update subscription metadata
        │
        ▼
Send CoreEvent::SubscriptionsUpdated via core_event_tx
        │
        ▼
Handler: add_log warnings → log_trace success/error → reload_profiles → reload_groups → load_subscriptions
```

## Data Flow: Endpoint Enrichment (DNS / GeoIP / Whitelist)

```
AppState::new / reload_profiles
        │
        ▼
spawn_enrich_ip_hosts: seed endpoint_info for IP hosts (host = own address)
  and DNS hosts with persisted endpoints.resolved_as (no network)
        │
        ▼
x key (force) / connect_to_profile / SpeedTestResult (DNS hosts)
        │
        ▼
spawn_dns_resolve: TTL-gated (dns_cache_ttl_secs, default 300; IP hosts never)
  → dns_resolver.lookup_ip (8s overall timeout; DNSCrypt list download 10s)
  → fill_features: mmdb country + host_features ip/cidr + sni
  → CoreEvent::EndpointInfoUpdated { endpoint_id, info }
        │
        ▼
HostFeaturesLoaded (whitelist files loaded, once per launch)
  → spawn_whitelist_pass: refresh ip/cidr + SNI flags for every endpoint
        │
        ▼
SpeedTestResult with ip_info (real ping)
  → spawn_outbound_enrich: parse exit IP → mmdb country → outbound fields
        │
        ▼
poll_core_events EndpointInfoUpdated handler: merge by field group into
  state.endpoint_info (concurrent events must not clobber); persist DNS
  resolutions via update_endpoint_resolution (only when resolved_at changed);
  failed lookups (empty IPs) materialize TTL-gated attempt entries
```

## gRPC API Services

| Service | Proto Path | Backend | Usage |
|---|---|---|---|
| `StatsService` | `app/stats/command/command.proto` | Xray-core | Query inbound/outbound/user traffic statistics |
| `LoggerService` | `app/log/command/config.proto` | Xray-core | Subscribe to real-time logs |
| `HandlerService` | `app/proxyman/command/config.proto` | Xray-core | Add/remove inbounds and outbounds at runtime |
| `RoutingService` | `app/router/command/config.proto` | Xray-core | Get/add/remove routing rules at runtime |
| `V2RayAPI` | v2ray gRPC (compatible subset) | Sing-box | Stats + Logger services via experimental.v2ray_api |
| `ReflectionService` | gRPC server reflection | Both | Discover available services |

## Error Handling Strategy

- **Process crashes**: Detected via `Child::try_wait()`. Auto-restart with backoff (1s, 2s, 4s, max 30s). UI shows error state with core type.
- **Config errors**: Builder validates before writing. Core output parsed for config errors. Shown in log panel.
- **Core binary not found**: Detection at startup and profile connect time. Profiles for the missing core show "binary not found" in connect UI.
- **gRPC connection failure**: Retry with backoff. If core is running but API not responding, show degraded state (stats unavailable). Sing-box V2Ray API may be missing if built without `with_v2ray_api` tag.
- **Network errors** (subscription download, speed test): Show timeout/failure per-server. Don't block UI.
- **No running core**: Most screens (profiles, settings, routing, DNS) work fully offline.

## Threading Model

Single tokio async runtime on the main thread for I/O. Ratatui rendering in a synchronous `draw()` call on each frame tick. Communication between async tasks and TUI via `tokio::sync::mpsc` channels:

- Process monitor task → TUI: `process_event_tx` (Started, Stopped, Crashed(error))
- Stats poll task → TUI: `stats_update_tx` (IndexId, TodayUp, TodayDown, etc.)
- Log reader task → TUI: `log_line_tx` (timestamp, level, message)
- Log persistence → heed (LMDB): `HeedLogStorage` stores entries synchronously via heed; no background worker or dedicated connection
- TUI event loop: polls all channels + terminal events + renders frame
