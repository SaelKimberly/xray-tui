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
pub enum Tab { Profiles, Settings, Routing, Dns, Logs, Statistics }
pub enum SortColumn { ConfigType, Remarks, Address, Port, Delay, Speed, Traffic, Core }
pub enum AppMode { List, AddServer{..}, EditServer{..}, ImportUrl{..} }
pub struct ProfileRow { profile: Profile, extension: Option<ProfileExtension>, stats: Option<ServerStat> }
pub struct LogLine { level: String, message: String }

pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub mode: AppMode,
    pub profiles: Vec<ProfileRow>,
    pub groups: Vec<Group>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    pub multi_select: HashSet<String>,
    pub confirm_delete: Option<String>,
    pub clipboard: Option<String>,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub log_buffer: Vec<LogLine>,
    pub connected_core: Option<CoreType>,
    pub should_quit: bool,
}
```

AppState provides:
- `filtered_profiles()` — group filter + search filter + sort by column
- `reload_profiles()` / `reload_groups()` — DB reload
- `add_log()` — capped circular log buffer (1000 entries)
- `start_add_server()` / `start_edit_profile()` — enter form mode
- `confirm_add_server()` / `confirm_edit_server()` / `cancel_form()` — form lifecycle
- `delete_profile()` / `clone_profile()` — CRUD operations
- `toggle_multi_select()` / `move_profile_up()` / `move_profile_down()` — multi-select + reorder
- `set_active()` — set default server
- `import_url()` — parse share URL and add profile

**TUI Screens (modules under `crates/xray-tui/src/ui/`):**

- `mod.rs` — Main event loop, tab rendering, keyboard handler, AppMode dispatch, placeholder renderer
- `profiles.rs` — Profile list DataGrid with sortable columns, group filter, search bar, multi-select indicator, delete confirmation overlay
- `add_server.rs` — Form screen for add/edit, protocol picker, field editing, import URL screen
- `status_bar.rs` — Bottom strip: connection indicator + key hints
- `settings.rs` — **Placeholder** (Phase 1: "Coming Soon")
- `routing.rs` — **Placeholder** (Phase 1: "Coming Soon")
- `dns.rs` — **Placeholder** (Phase 1: "Coming Soon")
- `logs.rs` — **Placeholder** (Phase 1: "Coming Soon")
- `statistics.rs` — **Placeholder** (Phase 1: "Coming Soon")

Future screens (Phase 3+): `subscription.rs`, settings panels, routing editor, log viewer, statistics panels.

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
    core_type: CoreType,
}

pub enum RunningCore {
    Xray(CoreProcess),
    SingBox(SingBoxProcess),
}

pub struct CoreManager {
    current: Option<RunningCore>,
    config_dir: PathBuf,
}

impl CoreManager {
    pub async fn start(core_type: CoreType, config: &BackendConfig) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub async fn restart(&mut self, core_type: CoreType, config: &BackendConfig) -> Result<()>;
    pub fn is_running(&self) -> bool;
    pub fn running_core_type(&self) -> Option<CoreType>;
}
```

`CoreManager::start` flow:
1. If a core is running and of different type → stop it
2. Write the appropriate JSON config to temp file
3. Spawn `xray run -c <path>` or `sing-box run -c <path>`
4. Wait for process ready (poll every 500ms, max 10s)
5. Connect to the backend's gRPC API

---

**`config_builder/`** — Config builder module
- `mod.rs` — Dispatches to xray or sing-box builder based on core type
- `xray.rs` — Builds xray-core format JSON
- `singbox.rs` — Builds sing-box format JSON

```rust
pub enum BackendConfig {
    Xray(XrayConfig),
    SingBox(SingBoxConfig),
}

pub struct ConfigBuilder;
impl ConfigBuilder {
    pub fn build(
        profile: &Profile, settings: &Settings,
        routing: &[RoutingRule], dns: &DnsConfig
    ) -> Result<BackendConfig>;
}
```

**Config builder — xray.rs** ports v2rayN's `CoreConfigContextBuilder` (C#) to Rust. Produces the xray-core JSON format:
```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [{ "tag": "socks-in", "protocol": "socks", ... }],
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
  "api": { "services": ["HandlerService", "LoggerService", "StatsService"] },
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
      "listen": "127.0.0.1:8080",
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
}
```

Mapping: TUIC, Hysteria, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → `SingBox`. All others (VMess, VLESS, Shadowsocks, SOCKS, HTTP, Trojan, WireGuard, Hysteria2, Dokodemo-door, Freedom, Blackhole, DNS, Loopback, Custom) → `Xray`.

---

**`grpc_client.rs`** — gRPC stats abstraction
```rust
#[async_trait]
pub trait StatsProvider {
    async fn query_stats(&self, name: &str, reset: bool) -> Result<StatsResponse>;
    async fn subscribe_logs(&self) -> Result<mpsc::Receiver<LogEntry>>;
    async fn get_rules(&self) -> Result<Vec<RoutingRule>>;
}
```

Two implementations:
- `XrayGrpcClient` — connects to xray-core's gRPC API port, uses tonic with xray proto definitions from `thirdparty/Xray-core/app/`
- `SingBoxGrpcClient` — connects to sing-box's `experimental.v2ray_api` gRPC port, uses tonic with the same v2ray proto interface

Both produce the same response types consumed by the TUI.

---

**`subscription.rs`** — Subscription download and parsing
```rust
pub async fn update_subscription(url: &str, proxy: Option<&str>) -> Result<Vec<Profile>>;
```
Parses base64-encoded share URLs, plain URL lists, v2rayN subscription formats, and sing-box format.

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
```

**Repository pattern** — Each table gets a typed repo (same as single-core design).

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
User clicks "Update subscription" for a group
        │
        ▼
HTTP GET subscription_url → download response body
        │
        ▼
Auto-detect: base64? plain URL list? v2rayN format? sing-box format?
        │
        ▼
For each server in response:
  parse_share_url(server_url) → Profile struct
  resolve_core(profile.protocol, group.core_type) → assign core_type
  ProfileRepo::upsert(profile)  (match by address+port+userId as dedup key)
        │
        ▼
Refresh profile list in UI
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
- **Config errors**: Builder validates before writing. Core stderr parsed for config errors. Shown in log panel.
- **Core binary not found**: Detection at startup and profile connect time. Profiles for the missing core show "binary not found" in connect UI.
- **gRPC connection failure**: Retry with backoff. If core is running but API not responding, show degraded state (stats unavailable). Sing-box V2Ray API may be missing if built without `with_v2ray_api` tag.
- **Network errors** (subscription download, speed test): Show timeout/failure per-server. Don't block UI.
- **No running core**: Most screens (profiles, settings, routing, DNS) work fully offline.

## Threading Model

Single tokio async runtime on the main thread for I/O. Ratatui rendering in a synchronous `draw()` call on each frame tick. Communication between async tasks and TUI via `tokio::sync::mpsc` channels:

- Process monitor task → TUI: `process_event_tx` (Started, Stopped, Crashed(error))
- Stats poll task → TUI: `stats_update_tx` (IndexId, TodayUp, TodayDown, etc.)
- Log reader task → TUI: `log_line_tx` (timestamp, level, message)
- TUI event loop: polls all channels + terminal events + renders frame
