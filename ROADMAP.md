# xray-tui — Roadmap

## Phase 0 — Foundation (scaffolding) ✅

- ✅ Cargo workspace with 4 crates: xray-tui (bin), xray-tui-core (lib), xray-tui-db (lib), xray-tui-config (lib)
- ✅ SQLite schema (tables: profiles, groups, subscriptions, routing_rules, dns_settings, profile_extensions, server_stats)
- ✅ `profiles` table includes `core_type TEXT NOT NULL DEFAULT 'xray'` — selects proxy backend (xray / sing-box / auto)
- ✅ `groups` table includes `core_type TEXT` — optional per-group override for subscription servers
- ✅ **CoreType enum**: `Xray` | `SingBox` | `Auto` (Auto resolves via protocol-core mapping table)
- ✅ Protocol enum covers both backends: xray-native protocols + sing-box-only protocols (TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect)
- ✅ Application config JSON file format (AppConfig struct with CoreConfig, GuiConfig, InboundConfig)
- ✅ Shared data models/enums (CoreType, Protocol, resolve_core, 7 model structs)
- ✅ Workspace builds with `cargo build`, tests pass with `cargo test`, clippy clean

## Phase 1 — TUI Shell ✅

- ✅ Ratatui main layout: tabbed panel structure (Profiles, Settings, Routing, DNS, Logs, Statistics)
- ✅ Profile list DataGrid with sortable columns (type, remarks, address, port, delay, speed, traffic, core)
- ✅ Group/subscription filter strip with live client-side search
- ✅ Status bar showing connection status + key hints (`[Tab] Next  [Ctrl+Q] Quit`)
- ✅ Tab navigation (Tab/Shift+Tab), profile list navigation (Up/Down/Home/End)
- ✅ Placeholder screens for Settings, Routing, DNS, Statistics
- ✅ DB query methods: get_all_profiles, get_profiles_by_group, get_all_groups, get_profile_extension, get_server_stats, get_all_profiles_with_details
- ✅ AppState with filtered/sorted profile list
- ✅ Keyboard shortcuts: q/Ctrl+C quit, / search, Esc clear filter, Tab cycle tabs

## Phase 2 — Profile CRUD ✅

- ✅ Add server forms per protocol (VMess, VLESS, Shadowsocks, SOCKS, HTTP, Trojan, WireGuard, Hysteria2, Hysteria v1, TUIC, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, Dokodemo-door, Custom)
- ✅ Core type selector per profile: auto-detect (default), force xray, force sing-box
- ✅ Edit/delete/clone/copy servers
- ✅ Share URL import (vmess://, vless://, ss://, trojan://, socks://, hysteria://, hysteria2://, tuic://, naive://, anytls://, shadowtls://, wireguard://, ssr://, http://) — all 14 protocols with fallback chain
- ✅ Multi-server selection operations
- ✅ Set default server, move/reorder servers
## Phase 3 — Core Integration

- ✅ **Dual-backend CoreManager** — common trait with xray and sing-box implementations
- ✅ `CoreProcess` for xray-core (spawn `xray run -c <path>`)
- ✅ `SingBoxProcess` for sing-box (spawn `sing-box run -c <path>`)
- ✅ Config builder split: `config_builder_xray.rs` (xray-core JSON) + `config_builder_singbox.rs` (sing-box JSON)
- ✅ Protocol → Core auto-resolution: TUIC/Hysteria/Naïve/AnyTLS/ShadowTLS/Tor/SSH → sing-box; all others → xray-core
- ✅ gRPC API abstraction: `StatsProvider` trait for stats; xray uses native gRPC, sing-box uses experimental v2ray_api. Stats polling loop (3s interval) in connect_to_profile. Live Statistics tab with traffic and system stats.
- ✅ One-core-at-a-time: switching profiles between backends stops current core, starts the other
- ✅ Connect/disconnect flow
- ✅ Auto-detect xray and sing-box binary paths (PATH, configured paths)
- ✅ Process spawn/kill/restart lifecycle with health polling

## Phase 4 — Subscription Management ✅
-
- ✅ Add/edit/delete subscription groups
- ✅ Update all/group subscriptions (HTTP download + parse)
- ☐ Update via proxy
- ✅ Subscription format detection (base64, plain, v2rayN, sing-box) — streaming decoder with auto-encoding detection
- ✅ Scheduled auto-update

## Phase 5 — Speed Testing ✅

- ✅ TCP ping (connect timeout test)
- ✅ Real ping (HTTP request through proxy)
- ✅ Speed test (download test through proxy)
- ✅ UDP test
- ✅ Fast batch ping (all servers)
- ✅ Sort by results, remove invalid servers
- ✅ Test result caching in SQLite

## Phase 6 — Settings Panel ✅
-
- ✅ Core type/settings, log level
- ✅ Inbound config (SOCKS, HTTP, mixed ports, sniffing)
- ✅ DNS settings (simple + advanced)
- ✅ Routing rules management (add/edit/delete/reorder)
- ✅ GUI settings (language (en/zh), theme, refresh interval)
- ✅ System proxy integration (set/unset HTTP_PROXY)
- ✅ TUN mode config
- ✅ Mux/fragment config
- ✅ Statistics toggle
:- ✅ Split-pane navigation (collapsible tree + form panel) — unified SettingsMode::Split replacing 10+ per-section form variants; Ctrl+W tree/form focus switching, SPLIT_SETTINGS_TREE const for easy section addition

## Phase 7 — Advanced Features

- ✅ Logs tab — wire core stdout/stderr pipeline to TUI display with live scrolling, color-coded log levels, and keyboard navigation (Up/Down/PgUp/PgDn/Home/End)
- ✅ Sing-box config builder completeness — config generation for all 17 outbound protocols (ShadowsocksR, Hysteria v1, Naive, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, VMess, VLESS, Trojan, WireGuard) via 12 new protocol arms + build_tls helper + protocol code mapping
- ☐ WireGuard sing-box peer model — complete form fields for multi-peer arrays, pre-shared keys, allowed IPs
- ✅ Sub-healer profile model adoption — uid-based PK (sig ^ cred_hash), spec_blob (postcard-encoded ProtocolConfig) replaces flat fields, Connection table for many-to-many Profile↔Group, bridge traits (ProfileLegacy/ProfileMut) for old parse/format compatibility, xray-tui-proto crate with protocol config types. All 253 workspace tests pass.
- ☐ Advanced form fields — add multiplex, V2Ray transport (WS/gRPC/QUIC/HTTPUpgrade), TLS options (ECH, uTLS fingerprint, Fragment) to protocol forms
- ✅ ProfileCore dedup — normalized schema (profile_cores + group_profiles) eliminates redundant storage for shared configs
- ✅ Version update check — GitHub releases API, download, install with .bak rollback, Updates settings form with per-core status
- ✅ Batch import — bulk import multiple share URLs at once via AppMode::BatchImport with scrollable results list
- ✅ Log source filtering — per-source toggle (core/TUI) via c/t keys in Logs tab, filter bar indicator
- ✅ Per-protocol core overrides — Settings → Protocol Core section with per-protocol Select:Auto/Xray/SingBox, 3-level resolution (profile→config→auto) in resolved_core() helper
- ✅ Group cycling shortcuts — [ / ] keys for next/previous group on Profiles tab, wraps with rem_euclid, skips graveyard
- ✅ Ctrl+G fallback — alternative connect key for terminals that can't distinguish Ctrl+Enter
- ✅ Group overlay fixes — sorted visual selection, Clear Group advertised, ALL-group mirror cleanup on delete
- ✅ Conditional Group column — shows profile origin group on "All" tab, hidden for specific groups, hidden at <107 col width
:- ✅ Speed test config — configurable ping URL, IP API URL, timeouts, batch page size via Settings > Speed Test form with persistent SpeedTestConfig
- ✅ Batch-then-real-ping — automatic real ping (with IP info) on fastest profiles after TCP batch, TestTypeUpdate event for emoji switching
- ✅ Profiles table redesign — connected indicator column (▶/space), IP info column, graveyard group filter, connected row highlight (TABLE_ROW_CONNECTED), selection preserved across sort column changes
- ✅ Log source validation toggle — V key on Logs tab toggles validation/subscription source logs visibility
- ✅ Graveyard orphan promotion — subscription_upsert_profiles promotes re-imported profiles from graveyard
- ✅ Keybinding harmonization — Ctrl+D disconnect, Ctrl+Shift+S copy share URL, TUI_MANUAL.md updated
:- ✅ Heed-backed log storage — `logs` LMDB database with postcard-encoded LogMessage entries, `HeedLogStorage` in xray-tui-core::log_heed, non-blocking TuiLogLayer via std::sync::mpsc channel, background batched heed writer (spawn_blocking, batch up to 100), MapFull→auto-resize (1 GB default, doubles up to 8 GB, atomic fail counter), async heed read wrappers (spawn_blocking), lazy log loading on first Logs tab access

## Phase 8 — Polish & Release
-
- ✅ UI/UX fixes — 25+ items across 9 files: centered confirmation overlay with quit safety; inline form validation with error display; empty-state guidance when no profiles; blinking terminal cursor in search mode; actions panel collapses in non-profile tabs; consistent boolean `[X]`/`[ ]` and select `< value >` display; Home/End in group overlay; PgUp/PgDn in profiles list; inverted log scroll (Up=newer); Ctrl+A select-all/deselect-all; connection status indicator in tab bar; update indicator with yellow background; statistics screen refactored into 3 bordered sections; extract_form_fields helpers for form mode dedup

- ☐ i18n (en, zh, ru, fa)
- ✅ Theme system (dark/light/custom) — integrated ratatui-themes + ratatui-cheese crates, ThemeStyles + Palette pattern, palette_bridge bridge module, DataTable widget extraction, mouse support, tui-popup overlays
:: ✅ Scrollbar support — themed scrollbar via tui-scrollbar 0.2 integrated into DataTable widget: Layout::horizontal split, GlyphSet::Unicode arrows, thumb/track styles via ThemeStyles::scrollbar_thumb/scrollbar_track, wired into logs and profiles screens
- ☐ Responsive layout for terminal resize
- ☐ Error handling & recovery flows
- ☐ man page / --help
- ☐ CI/CD, packaging
- ✅ Performance optimization — add_log owned strings (saves 3 allocs/log), get_str closure (saves 5-17 allocs/settings save), parse_core_log_line eq_ignore_ascii_case (removes to_lowercase alloc), BatchImport mutable scroll (avoids Vec clone per keypress), TCP ping spawn no longer clones Vec&lt;Profile&gt;, updater Cow&lt;OsStr&gt; over to_string_lossy
- ☐ Geo file auto-update — periodic download of geoip.dat/geosite.dat for both backends
- ☐ Log to file toggle — core log persistence with configurable path
- ☐ Certificate pinning UI — SHA-256 fingerprint or PEM upload per profile
- ☐ Default skip cert verify — global security toggle in Core settings
- ☐ Clear all stats — reset traffic counters per profile or globally
- ☐ Mux protocol selector — per-profile sing-box mux protocol choice

## Phase 9 — v2rayN Parity

### Routing

- ☐ Proxy chain — front proxy (入口代理) + back proxy (落地代理) per subscription group, with tag-based outbound chaining
- ☐ Policy groups/balancers — Xray 4 types (random/leastPing/leastLoad/leastConn) + sing-box 1 type, alias-based node selection with nested groups
- ☐ Routing mode switcher — quick Rule/Global/Direct/Unchanged toggle in status bar
- ☐ Rule set sources — custom geosite/geoip URLs with auto-update per routing rule
- ☐ Routing rules import/export — JSON array clipboard operations
- ☐ Process-based routing — per-process split tunneling (v2rayN v7.23.0 feature)
- ☐ End fragment support — outbound fragment for CDN evasion

### Subscription

- ☐ Multi-URL subscriptions — optional extra URLs merged into one group
- ☐ Regex filtering — filter imported profiles by alias during subscription update
- ☐ Subscription conversion target — convert between core types on import (convert_target field exists, always None)
- ☐ SIP008 format — Shadowsocks SIP008 subscription format support
- ☐ SOCKS4/5 share link import — socks:// scheme with SOCKS4 auth fields

### Clash

- ☐ Clash Mixin — YAML overlay injected into sing-box config for Clash-compatible features
- ☐ Clash API proxies view — real-time proxy group selector TUI tab
- ☐ Clash API connections view — real-time active connection monitor TUI tab

### Core Management

- ☐ Automatic server failover — health-check based fallback between profiles in a group
:- ✅ Batch test parameters — configurable concurrency (tcp_ping_concurrency, real_ping_concurrency via SpeedTestConfig), timeout, test URL per test type (timeouts + URLs done via SpeedTestConfig in Phase 7)
- ☐ PAC system proxy mode — PAC file generation with GFWList integration (HTTP proxy only currently)
- ☐ Multi-core simultaneous support — run xray-core and sing-box concurrently for different groups

### System Integration

- ☐ Auto startup — systemd user service / autostart desktop file / launchd plist
- ☐ Sudo password input — polkit/privileged operation support for TUN mode on Linux/macOS
- ☐ Global hotkey bindings — system-wide key combinations for common actions
- ☐ DBUS integration — system proxy set via networkmanager/gsetting D-Bus instead of env vars

### Config & Advanced

- ☐ Full config template editor — raw JSON editor for both xray-core and sing-box formats
- ☐ Config backup/restore — WebDAV cloud backup with scheduled auto-backup
- ☐ JSON config import/export — export full profile as JSON, import auto-detect format
- ☐ QR code display — render share URL as ANSI QR code in terminal
