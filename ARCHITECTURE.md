# xray-tui Architecture

## Crate Dependency Graph

```
xray-tui (bin)
  ├── xray-tui-core     (Protocol, CoreType, resolve_core)
  ├── xray-tui-db       (Database, query methods, models)
  └── xray-tui-config   (AppConfig load/save)
```

## Crate Responsibilities

### xray-tui (binary crate)

Entry point at `crates/xray-tui/src/main.rs`. Creates the tokio async runtime, initializes all subsystems, enters the ratatui event loop.

**Shared state** (`crates/xray-tui/src/lib.rs`):

```rust
pub enum Tab { Profiles, Settings, Logs, Statistics }
pub enum SortColumn { ConfigType, Remarks, Address, Port, Delay, Speed, Traffic, Core }
pub enum AppMode { List, Help, Settings{..}, AddServer{..}, EditServer{..}, ImportUrl{..}, BatchImport{results: Vec<BatchImportItem>, scroll: usize}, ManageGroups{..}, AddGroup{..}, EditGroup{..}, SpeedTestMenu{selected: usize} }
pub struct ProfileRow { profile: Profile, extension: Option<ProfileExtension>, stats: Option<ServerStat> }
pub struct LogLine { level: String, message: String, source: String }

pub struct AppState {
    pub db: Arc<Database>,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub mode: AppMode,
    pub profiles: Vec<ProfileRow>,
    pub groups: Vec<Group>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    pub multi_select: HashSet<String>,
    pub clipboard: Option<String>,
    pub confirmation: Option<ConfirmAction>,
    pub updating_groups: HashSet<String>,
    pub testing_profiles: HashSet<String>,
    pub test_progress: Option<(usize, usize)>,
    pub search_focused: bool,
    pub log_buffer: VecDeque<LogLine>,
    pub logs_show_core: bool,
    pub logs_show_tui: bool,
    pub connected_core: Option<CoreType>,
    pub connecting: bool,
    pub connection_error: Option<String>,
    pub disconnect_tx: Option<oneshot::Sender<()>>,
    pub system_stats: Option<SysStats>,
    pub update_status: HashMap<CoreType, BackendUpdateStatus>,
    pub previous_mode: Option<Box<AppMode>>,
    pub routing_rules: Vec<RoutingRule>,
    pub actions_compact: bool,
    pub connected_profile_id: Option<String>,
    pub last_core_log: Option<(String, String)>,
    pub last_tui_log: Option<(String, String, String)>,
    pub last_test_tcp: Option<u64>,
    pub testing_details: HashMap<uuid::Uuid, TestType>,
    pub logs_show_validation: bool,
    pub last_test_real: Option<u64>,
    /// terminal height (Atomics for interior mutability across render thread)
    pub term_height: AtomicU16,
    pub core_event_tx: Option<Sender<CoreEvent>>,
    pub log_worker_tx: Option<Sender<LogWorkerMessage>>,
    pub core_event_rx: Option<Receiver<CoreEvent>>,
}

### CoreEvent Channel

```rust
pub enum CoreEvent {
    Connected(CoreType),
    Disconnected,
    Error(String),
    StatsError(String),
    ClearGroup(String),
    StatsUpdate {
        profile_id: String,
        today_up: i64,
        today_down: i64,
        total_up: i64,
        total_down: i64,
    },
    SpeedTestResult { profile_id, test_type, latency_ms, speed_bps, ip_info, error },
    SubscriptionsUpdated { group_id, count, error, warnings },
    UpdateCheckResult { core_type, current_version, latest_version, error },
    UpdateCompleted { core_type, old_version, new_version, success, error },
    /// A log line from the core process stdout (xray-core) or stderr (sing-box).
    LogLine {
        level: String,
        message: String,
    },
    /// A log line from the TUI internals forwarded through tracing subscriber.
    TuiLog {
        target: String,
        level: String,
        message: String,
    },
    /// Update the displayed test type for a profile without triggering cleanup.
    /// Used by batch_then_real_ping to switch from TcpPing→RealPing emoji
    /// after TCP completes but before real ping starts.
    TestTypeUpdate {
        profile_id: String,
        test_type: TestType,
    },
}
```
spawned `CoreManager` task and the TUI event loop. `poll_core_events()` is called each frame,
draining pending events and updating `AppState` fields (`connected_core`, `connecting`, `connection_error`).
The `disconnect_tx` oneshot channel signals the running core task to stop gracefully.

- `import_url()` / `start_batch_import()` — parse share URL(s) and add profile(s)
- `filtered_profiles()` — group filter + search filter + sort by column
- `reload_profiles()` / `reload_groups()` — DB reload
- `add_log()` — capped circular log buffer (1000 entries) + forwards non-TUI entries to LogStorageWorker for DB persistence; takes level, message, and source
- `start_add_server()` / `start_edit_profile()` — enter form mode
- `confirm_add_server()` / `confirm_edit_server()` / `cancel_form()` — form lifecycle
- `delete_profile()` / `clone_profile()` — CRUD operations
- `toggle_multi_select()` / `move_profile_up()` / `move_profile_down()` — multi-select + reorder
- `set_active()` — set default server
- `import_url()` — parse share URL and add profile
- `connect_to_profile(&mut self, profile_id: &str)` — spawn async CoreManager task, send CoreEvents
- `cycle_group(dir: i8)` — cycle to next/previous group, skipping graveyard
- `resolved_core(row: &ProfileRow) -> CoreType` — 3-level core resolution: profile override → config per-protocol override → auto-detect
- `disconnect(&mut self)` — send stop signal via disconnect_tx oneshot
- `poll_core_events(&mut self)` — drain core event channel each frame, update state

**TUI Screens (modules under `crates/xray-tui/src/ui/`):**

::- `settings.rs` — Full settings panel (Phase 6). Menu listing 12 config sections: Core, GUI, Protocol Core, Inbound, Routing Rules (list/add/edit/delete/reorder), DNS, System Proxy, TUN, Mux/Fragment, Statistics, Updates, Speed Test. Each opens a form overlay. Routing/DNS forms persist to DB; all others persist to AppConfig JSON.
- `groups.rs` — Subscription group overlay (list + add/edit forms) with update/delete actions. Accessed via `g` key from Profiles tab.
::- `logs.rs` — Log viewer with source filtering (c/t toggles for core/TUI logs, v toggles validation/subscription logs)
- `actions_log.rs` — Live event log panel showing connection status, speed test results, core/TUI/app logs, traffic counters with color-coded levels. F1 toggles compact/full modes; auto-compacts on small terminals (<20 rows).
...
- `theme.rs` — Central color palette and Style definitions (Theme struct with 19 constants across 6 groups)

:**`speed_test.rs`** — Async speed test engine:
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
Batch ping mode deduplicates by (address, port) using the UniqueTarget helper struct.
`batch_then_real_ping` runs TCP ping on all profiles, selects the fastest N (batch_page_size), then runs real ping
on those with IP info fetch. Sends `TestTypeUpdate` events mid-flow to update displayed emoji.

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
    log_tx: UnboundedSender<String>,  // process stdout/stderr forwarded to TUI
}

pub enum RunningCore {
    Xray(CoreProcess),
    SingBox(CoreProcess),
}

pub struct CoreManager {
    current: Option<RunningCore>,
    config_dir: PathBuf,
}

impl CoreManager {
    pub async fn start(
        core_type: CoreType,
        profile: &Profile,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
        tx: UnboundedSender<CoreEvent>,
        stop_rx: oneshot::Receiver<()>,
    );
    pub async fn stop(&mut self);
    pub fn is_running(&self) -> bool;
    pub fn running_core_type(&self) -> Option<CoreType>;
}
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

`crates/xray-tui-db/src/lib.rs` — SQLite database layer.

**Schema initialization** (`schema.rs`):
```sql
CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    config_type INTEGER NOT NULL,
    core_type TEXT NOT NULL DEFAULT 'xray',  -- 'xray' | 'sing-box' | 'auto'
    remarks TEXT NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    port INTEGER NOT NULL DEFAULT 0,
    user_id TEXT,
    security TEXT,
    network TEXT,
    stream_settings TEXT,  -- JSON blob (transport + TLS + REALITY)
    protocol_settings TEXT, -- JSON blob (protocol-specific fields)
    is_sub INTEGER NOT NULL DEFAULT 0,
    sub_id TEXT,
    group_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subscription_url TEXT,
    subscription_enabled INTEGER NOT NULL DEFAULT 0,
    user_agent TEXT,
    convert_target INTEGER,
    core_type TEXT,          -- optional override for all servers in this group
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS routing_rules (
    id TEXT PRIMARY KEY,
    group_id TEXT,
    type INTEGER NOT NULL,
    domain_matcher TEXT,
    domains TEXT,
    ips TEXT,
    inbound_tags TEXT,
    port TEXT,
    source_ports TEXT,
    network TEXT,
    protocols TEXT,
    domain_strategy TEXT,
    outbound_tag TEXT,
    balancer_tag TEXT,
    rule_set_file TEXT,
    rule_set_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS dns_settings (
    id TEXT PRIMARY KEY,
    name TEXT,
    servers TEXT,
    hosts TEXT,
    query_strategy TEXT,
    disable_cache INTEGER,
    disable_fallback INTEGER,
    client_ip TEXT
);

CREATE TABLE IF NOT EXISTS profile_extensions (
    profile_id TEXT PRIMARY KEY,
    delay INTEGER NOT NULL DEFAULT -1,
    speed INTEGER NOT NULL DEFAULT -1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    ip_info TEXT
);

CREATE TABLE IF NOT EXISTS server_stats (
    profile_id TEXT PRIMARY KEY,
    today_up INTEGER NOT NULL DEFAULT 0,
    today_down INTEGER NOT NULL DEFAULT 0,
    total_up INTEGER NOT NULL DEFAULT 0,
    total_down INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS logs (
    id              INTEGER PRIMARY KEY,
    timestamp_nanos BIGINT NOT NULL,
    level           TEXT NOT NULL DEFAULT 'info',
    target          TEXT NOT NULL DEFAULT '',
    message         TEXT NOT NULL,
    metadata_json   TEXT,
    source          TEXT NOT NULL DEFAULT 'tui'
);
CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp_nanos);
CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_target ON logs(target);
CREATE INDEX IF NOT EXISTS idx_logs_source ON logs(source);
```

**Repository pattern** — Each table gets a typed repo (same as single-core design).


**Log storage**: `TuiLogLayer` (in `main.rs`) captures `tracing::Event` emissions and dual-sends to (a) `core_event_tx` for in-memory `log_buffer` display and (b) `LogWorkerMessage::Entry` via `log_worker_tx` for persistence. The `LogStorageWorker` (in `log_worker.rs`) runs a `tokio::select!` loop accepting entries and queries. Entries are batched (500 or 100ms) and flushed via `LogRepository::insert_batch()` on a **dedicated connection** using `BEGIN IMMEDIATE` + `busy_timeout(500ms)`. Filtered queries merge the pending batch with DB results (newest-first ordering). TTL cleanup runs hourly via a background tokio task (deletes logs older than `ttl_hours`, defaults 72h).
**Migration approach**: migratus-like version numbering in a `schema_version` table.

### xray-tui-config (library crate)

`crates/xray-tui-config/src/lib.rs` — App config management, protocol form fields, share URL parsing.

**Modules added in Phase 2:**
- `forms.rs` — `FormFieldType`, `FormField`, `FieldSection`, `form_fields_for()` (27 protocols)
- `import_export.rs` — `parse_share_url()`, `format_share_url()` (12 protocol formats)

**App config** (JSON at `~/.config/xray-tui/config.json`): mirrors v2rayN's `Config.cs` fields plus dual-core settings (xray binary path, sing-box binary path, default core type).

**Import/export format parsers**:
- `vmess://` — Base64 JSON decoding
- `vless://` — URI parsing (scheme://uuid@host:port? params)
- `ss://` — SIP002 URI format
- `trojan://` — URI with password
- `socks://`, `http://`, `hysteria2://`, `hysteria://`, `tuic://`, `naive://`, `anytls://`, `shadowtls://`, `wireguard://` — Various URI schemes
- Subscription format auto-detection (base64 list, plain list, v2rayN, sing-box)
- Export share URL generation (mirrors `Fmt/*.cs` in v2rayN)

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
    Xray    → XrayGrpcClient::connect(api_addr)
    SingBox → SingBoxGrpcClient::connect(api_addr)
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
- Log storage worker → DB: `LogStorageWorker` receives entries via `log_worker_tx`, batches every 100ms, flushes to `logs` table on a dedicated connection
- TUI event loop: polls all channels + terminal events + renders frame
