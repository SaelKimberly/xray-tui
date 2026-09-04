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

- ✅ **Dual-backend CoreManager** — `CoreManager` is a Rust trait (start/stop/is_running/running_core_type/sighup_reload/rewrite_config); `RealCoreManager` spawns xray or sing-box subprocesses (requires `log_tx: Sender<String>`), `MockCoreManager` is a test double with `Option<String>` error fields
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
✅ WireGuard sing-box peer model — complete form fields for multi-peer arrays, pre-shared keys, allowed IPs
- ✅ Sub-healer profile model adoption — uid-based PK (sig ^ cred_hash), spec_blob (JSON-encoded ProtocolConfig, wrapped by the Proto identity container) replaces flat fields, Connection table for many-to-many Profile↔Group, bridge traits (ProfileLegacy/ProfileMut) for old parse/format compatibility, xray-tui-proto crate with protocol config types. All 401 workspace tests pass.
- ✅ Advanced form fields — multiplex, V2Ray transport (WS/gRPC/QUIC/HTTPUpgrade), TLS options (ECH, uTLS fingerprint, Fragment) all added to protocol forms
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
:- ✅ Heed-backed log storage — `logs` LMDB database with postcard-encoded LogMessage entries, `HeedLogStorage` in xray-tui-core::log_heed, non-blocking TuiLogLayer via std::sync::mpsc channel, background batched heed writer (spawn_blocking, batch up to 100), MapFull→auto-resize (1 GB default, doubles up to 8 GB, atomic fail counter) with backoff retry — batches retried after successful resize, never dropped, async heed read wrappers (spawn_blocking), lazy log loading on first Logs tab access
- ✅ Geo file auto-update — periodic download of geoip.dat/geosite.dat for both backends
- ✅ Log to file toggle — core log persistence with configurable path
- ✅ Certificate pinning UI — SHA-256 fingerprint or PEM upload per profile
- ✅ Default skip cert verify — global security toggle in Core settings
- ✅ Clear all stats — reset traffic counters per profile or globally
- ✅ Mux protocol selector — per-profile sing-box mux protocol choice

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
✅ Performance optimization — add_log owned strings (saves 3 allocs/log), get_str closure (saves 5-17 allocs/settings save), parse_core_log_line eq_ignore_ascii_case (removes to_lowercase alloc), BatchImport mutable scroll (avoids Vec clone per keypress), TCP ping spawn no longer clones Vec<Profile>, updater Cow<OsStr> over to_string_lossy

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

### Clash YAML

- ✅ Clash YAML proto_spec parsing — `clash/mod.rs` with 29 ClashProxy/ClashXxx serde structs, `try_from_clash`/`to_clash` on ProtoSpec trait, per-protocol implementations for all 17 protocols, dispatch macro, roundtrip tests passing
- ☐ Clash API proxies view — real-time proxy group selector TUI tab
- ☐ Clash API connections view — real-time active connection monitor TUI tab
- ☐ Clash API providers view — subscription provider management and health TUI tab
### Core Management

- ☐ Automatic server failover — health-check based fallback between profiles in a group
:- ✅ Batch test parameters — configurable concurrency (real_ping_concurrency via SpeedTestConfig), timeout, test URL per test type (timeouts + URLs done via SpeedTestConfig in Phase 7)
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

## Phase 10 — DNS & GeoIP

- ✅ xray-tui-dns + xray-tui-geoip crates adopted — DNSCrypt-stamp DNS resolution (DOH/DOT/DOQ via hickory-resolver 0.26) and GeoLite2-City IP → country/city lookup (maxminddb 0.30). Panic-free async init, per-instance cache dirs, workspace style clean (fmt/clippy/tests). Not yet wired into TUI.
- ✅ Resolve actual IP addresses of DNS-based endpoints in TUI — background enrichment pipeline (`ops/enrich.rs`): `spawn_dns_resolve` (TTL-gated, `x` key forces), results shown in the expanded panel's `IPs:` line; DNS hosts get `[?]` + `🏁` until resolved. Hard deadlines: 10s DNSCrypt-list download timeout + 8s overall lookup timeout — failed lookups never hang and materialize TTL-gated entries so auto-retriggers don't re-run.
- ✅ Location Info (country flags) — inbound country flag column (`🏴` unknown) from mmdb + outbound (egress) IP/country from real-ping `ip_info`, both in the single-line row; outbound also in the panel sub-table. DNS results persist across launches via `endpoints.resolved_as`/`resolved_at` columns.

## Phase 11 — Host Features

- ✅ xray-tui-host-features crate adopted — whitelist feature extraction ported from sub-healer: SNI/exact-IP/CIDR membership checks (fastbloom fast-negative guard + exact HashSet/interval verification, zero false positives, IPv4-only), download-if-missing from hxehex/russia-mobile-internet-whitelist, `get_host_features(&ServerName)` main API. Standalone, not yet wired into TUI.
- ✅ Whitelist features wired into TUI — IP/CIDR + SNI membership flags (`🏳️`) in the Profiles single-line row, refreshed on every launch by the whitelist pass (`HostFeaturesLoaded` event); features never persisted (track current files).

## Phase 12 — Profiles UI Redesign ✅

- ✅ 17-column single-line endpoint rows — Type, inbound country flag, Address, Feat whitelist flags (IP/SNI merged), transport/security combo, Test delay, outbound IP+country; Remarks, Delay, Speed, Traffic, IP-info, top-level Last Seen dropped from the single line (Remarks wiped from TUI and DB; Last Seen lives only in the panel sub-table)
- ✅ Test column (2026-08) — active protocol's delay `[ 12 ]` colored by threshold (green <500ms, yellow ≥500ms, red ≥1000ms); red problem labels `[name]` (DNS unresolved), `[fast]`/`[real]` (every protocol of the endpoint unreachable for that test, per session-only per-endpoint ping rounds in `AppState.ping_status`, reset at batch start). Placed after the `}=>` arrow, before Outbound's `[` bracket
- ✅ Feat column — IP+SNI flag cells merged into one 4-wide column (`🏁` DNS unresolved, `🏳️` IP/CIDR + SNI whitelisted)
- ✅ Expandable rounded panel — `IPs:` line with `(x resolve)` hint + 10-column per-protocol sub-table (marker, hex id, last seen, last used, config type, delay, speed, traffic, outbound, country); panel keeps a 1-line gap below so the bottom border never touches the next row; height-aware scrolling so expanded rows never strand the last profiles
- ✅ Expansion nav semantics — expand lands on first sub-row; `↑`/`↓` walk variants; `↑` at sub 0 → full row; `↓` at last sub-row → next profile; `↓` from full row of expanded endpoint → re-enter sub 0; collapsed endpoint moves on one `↓`
- ✅ Sub-table newest-first sort — each endpoint's protocols sorted by `last_seen_at` desc in `deserialize_endpoint_rows` (stable, ties keep insertion order); superseded by the test-priority sort in Phase 15
- ✅ Protocol pin via Enter — Enter on a sub-row sets `manual_protocol_override` (`set_protocol_default`), Enter on the endpoint row clears it (`set_active`); both patch the in-memory row so the UI switches without a reload
- ✅ Sub-row selection feedback — selected sub-row renders as a REVERSE highlight across the full panel-inner width (`ThemeStyles::panel_row_selected`: fg foreground + `bg Color::Reset` + bold): the expanded panel sits on the endpoint row's `surface` highlight, the selected sub-row drops back to the common background as a "notch". Theme-safe — the former accent `bg on_highlight` was near-white in Charm/light palettes (invisible light-on-light). Explicit `Color::Reset` required because ratatui `Cell::set_style` merges and a bg-less style leaves `surface` in place
- ✅ Uniform row background — zebra striping removed from the Profiles grid; unselected rows share one background, selection/connected keep their highlights
- ✅ DNS persistence across launches — `endpoints.resolved_as` (comma-joined IPs) + `resolved_at` (unix secs); schema v1→v3 migration (SCHEMA_VERSION 3, `ensure_column` + explicit transaction + `protocol_rows.endpoint_id` index)
- ✅ Endpoint-scoped ping batches — Fast/Real Ping on a collapsed multi-protocol endpoint row pings all its protocols; on a sub-row pings the exact protocol (`get_batch_for_real_ping(batch_id, wave, limit, dedup_endpoints)`)
- ✅ Last Used column — `protocol_rows.last_used_at` set on connect, shown in the panel sub-table
- ✅ `x` key — force DNS resolution of the selected endpoint

## Phase 13 — Hardening & Identity Refactor ✅

- ✅ Code review findings fixed (31/31, 5 critical) — single-codebase fixes: sing-box credential extraction, xray streamSettings synthesis, routing matcher validation, hysteria2 auth/pinSHA256, uid-0 collapse (PlaceholderConfig whole-body sig, never zero), typed sig/cred_hash retrofit (uid = sig ^ cred_hash; sig from semantic identity incl. security TYPE; cred_hash from credential values; no creds → uid == sig), transport host for ws/grpc/http, PortSpec overflow/coalesce, shared port allocator (CorePool::port_allocator) + batch active guard, pool mutex, udp_test timeouts, throughput fractional math, last_seen bump on connect, StatsUpdate protocol-id lookup, delete_group orphan cascade, edit view threshold, endpoint_id index (SCHEMA_VERSION 3), host-features merge guard, host-features/geoip/dns download deadlines + atomic writes + heal-on-corrupt, DataTable tall-row clipping, UI fixes (footer filtered row, Esc closes settings, copy URL, refresh_interval draw gate), log writer flush deadline, aarch64 xray asset (arm64-v8a), gRPC timeouts, heed resize backoff. 521 workspace tests pass.
- ✅ ProtoIdentity refactor — crate-private `ProtoIdentity` trait (compute_sig + compute_cred_hash) + `Proto` identity container (OnceLock<Identity> cache, Defer-less: empty lock IS the deferred state) replaces per-config sig_cache/cred_hash_cache fields and the impl_sig_cache! macro; `ProtoSpec: ProtoIdentity` sealed; `sig()/cred_hash()/uid()` inherent on `Proto`; spec_blob bytes unchanged (no DB migration).

## Phase 14 — Batch Ping Enhancement ✅

- ✅ Parallel Fast Ping — phase-1 pings a page concurrently via `run_page_pings` (one TCP ping per unique address:port, `buffer_unordered` capped by `fast_ping_concurrency` default 200, in-page owner/follower dedup + cross-page `fast_cache`); fast ping probes inbound only, so it is all-or-none per endpoint and the existing `[fast]`/`[real]` marker math stays valid
- ✅ Wave-scoped Real Ping dispatch — `get_batch_for_real_ping(batch_id, wave, limit, dedup_endpoints)` with occurrence ranks computed over ALL real sessions of the batch (status-independent, stable across dispatches); phase-2 consumer is a unified wake/drain pass loop (waves 1..N, `batch_page_size` chunks, 200ms coalescing, empty-wave termination); a failed protocol defers its endpoint's siblings to the next wave; `[real]` appears only after the endpoint's last candidate failed
- ✅ Skip/test-all after endpoint success — `real_ping_test_all_protocols` (default false = skip remaining protocols after one succeeds; true = test all); applies to all-visible batches; endpoint-scoped batches always test every protocol (`dedup=false`, and `cancel_stranded_real_pings` is gated on dedup so a success never cancels siblings)
- ✅ Group-failure halving retry + parallel readiness — stack-based page-halving on config-build/core-start/no-port-ready failures down to per-profile cores (missing binary not retryable); per-port SOCKS5 readiness waited in parallel (`join_all`), dead port = item-level failure
- ✅ Concurrency defaults — `real_ping_concurrency` 5 → 100, `fast_ping_concurrency` 200; `real_ping_window` (20) stays only as the phase-1 wake threshold
- ✅ Stopped tests don't fail rounds — `error == "Cancelled"` no longer inserts into the round's failed set, so stopping a batch never paints spurious `[fast]`/`[real]` labels
- ✅ Tests — 4 `run_page_pings` unit tests (parallel/cap/dedup/cached), DB wave-query test (stable ranks + dedup), integration dedup test updated

## Phase 15 — Test-Priority Sorting ✅

- ✅ Tier model + shared comparator — `EndpointRow::sort_protocols_by_test_priority` / `best_test_priority_key` in the db crate; ascending key `(tier, latency, -last_seen_at, id)`; tiers 0 real-ok, 1 fast/udp-ok, 2 untested, 3 real-err, 4 fast-err, 5 DNS-unresolved; latency only within tiers 0-1, tiers 2-5 by last_seen_at desc then id; fresh failure dominates stored success; main-table representative = best protocol (min key)
- ✅ `delay_source` provenance — `profile_extensions.delay_source` column (SCHEMA_VERSION 4, `ensure_column` migration): 0 fast, 1 real, 2 udp; written by the `SpeedTestResult` handler (from `test_type`) and `batch_upsert_buffer` (from session `ping_type`); real-ok survives restarts
- ✅ Live sub-table re-sort — `SpeedTestResult` handler re-sorts the owning endpoint's protocols on TcpPing/RealPing results (success or failure; UDP never re-sorts); `selected_sub` remapped by protocol id (only when the result's endpoint is selected); `EndpointInfoUpdated` unresolved→resolved DNS flip re-sorts; `filter_cache_valid` invalidated
- ✅ Error events mutate nothing — `if error.is_none()` gate wraps the whole ext mutation + upsert; a cancelled/failed test never ranks an untested protocol as real-ok and never writes a `delay = 0` row
- ✅ Main-table Test sort — `SortColumn::Delay` renamed `SortColumn::Test`; comparator ranks endpoints by best protocol's test-priority key; reachable via `o`/`O` cycle and speed-test menu item 7; dead duplicate `compute_filtered_indices` (state.rs) and dead `batch_upsert_buffer` (state.rs) deleted
- ✅ Tests — 18 new: comparator unit tests (tiers, fresh-failure dominance, latency ordering, dns sink, best-key), migration + SELECT round-trip tests, events tests (re-sort + remap, failure demote, dns sink, flip, cancelled-no-rank, UDP no-re-sort), main-table sort tests (best-protocol tier, failure/dns sinks). 553 workspace tests pass

## Phase 16 — Log-Dump Hardening ✅

- ✅ Cipher-aware Shadowsocks core routing — `XRAY_SS_METHODS` (AEAD + 2022-blake3 + `aead_*` aliases, from `proxy/shadowsocks/config.proto` + `cipherFromString`) and `SINGBOX_SS_METHODS` (18 methods from sing-box docs) in `protocol_core_mapping.rs`; `resolve_core` takes `ss_method`; legacy ciphers (`aes-*-cfb`, `aes-*-ctr`, `rc4-md5`, `chacha20-ietf`, `xchacha20`, `none`) auto-route to sing-box (xray-core's `CipherType` enum lacks them). Fixes the 6 "unknown cipher method: aes-256-cfb" core-start failures in dump-1.log
- ✅ Builder cipher validation + Shadowsocks2022 arms — xray.rs and singbox.rs both reject ciphers neither core supports via `BuildError::InvalidProfile` (invalid config never written; a forced core fails at build with a clear error, never a core crash on startup); `Protocol::Shadowsocks2022` now builds under protocol/type "shadowsocks" (was: "not supported" error)
- ✅ DB write-contention retry — `retry_on_busy`/`is_busy_error` in `xray-tui-db/src/retry.rs` (backoff 20ms doubling, 1.28s cap), wired into `update_endpoint_resolution` + `batch_flush_ping_buffer`; the 850 "database is locked" lines (enrichment herd, 20 ms-bursts) no longer drop writes. `Database::conn()` sets `PRAGMA busy_timeout=5000` on every pooled acquisition (the `open()` pragma is per-connection and never reaches pool conns — verified against dump-2, where 462 lock failures still fired in 5 second-bursts); all 49 write call sites route through it, so writers queue instead of failing
- ✅ Enrich hostname validation — `is_resolvable_hostname` (ASCII labels, no `?`/`=`/`:`/`_`/spaces) skips plugin URLs (`host:port?plugin=...`) and Telegram-channel garbage before hickory lookup; invalid names recorded as TTL-gated failed attempts
- ✅ WS vhost deprecation — xray wsSettings emit top-level `host` instead of deprecated `headers.Host` (typed path in `xray-tui-proto/src/proto_spec/common.rs` + legacy dotted path in `config_builder/xray.rs`); kills the 152 "host in headers is deprecated" core warnings
- ✅ Log hygiene — `TuiLogLayer` filter `trace,hickory_net::h2=error` (391 DoH h2 noise lines suppressed, everything else stays trace+); enrich "no records found" DNS warnings downgraded to debug
- ✅ Forms — Shadowsocks2022 method select: removed invalid `none`, added missing `2022-blake3-chacha20-poly1305`
- ✅ REALITY config validation (dump-2) — the xray builder rejects `security: "reality"` without `realitySettings.publicKey`/`serverName` (6× `REALITY: Empty "realitySettings"` core-start failures from legacy VLESS blobs with `security=reality` and no key material); sing-box reality block fixed to emit only `enabled`/`public_key`/`short_id` (the URL's `spx` was being written into `short_id`, corrupting it — sing-box has no `spider_x` outbound field)
- ✅ Tests — 24 new (routing 12, builder validation + 2022 4, retry 4, hostname 2, ws-host assertions 2, reality validation 2, singbox short_id 1). 575 workspace tests pass
- ✅ Host validation — `validate_host` rejects unspecified IP addresses (`0.0.0.0`/`::`) as a hard rule (not gated by `allow_private_ips`); bracketed-IPv6 forms (`[::]`, `[::1]`) now parse for all host checks (previously slipped past everything). Subscription summary counts them as host validation. 5 new tests — 580 workspace tests pass
## Phase 17 — Quality Gate ✅

- ✅ Gate target selector — `just quality-gate code|deps|all` (and `quality-gate-ci`): `code` = fmt-check/clippy/nextest (source checks), `deps` = hakari-check/deny/machete/outdated/audit (dependency checks), `all` default. Run source checks without dep audits and vice versa.
- ✅ Tooling configs — rustfmt.toml (edition 2024, max_width 100, Unix newlines), criterion.toml (`.benchmarks/` home; `cargo criterion` is the only bench launcher), .config/nextest.toml (`ci` profile), .cargo/audit.toml (vuln scan; 5 allowed informational warnings), deny.toml (licenses/bans/sources).
- ✅ cargo-hakari unified deps — `crates/xray-tui-hakari/` (renamed from workspace-hack to match the `xray-tui-*` crate convention); `.config/hakari.toml` targets the two toolchain triples; regenerate after any Cargo.lock change.
- ✅ cargo-machete (unused deps, hard-fail) — 16 genuinely unused deps removed; OUT_DIR/build.rs-generated deps (prost, tonic-prost, tonic-prost-build) and the documented tokio-rustls override ignored via `[package.metadata.cargo-machete]`; hakari crate's deps ignored by design (regeneration command in the manifest comment + AGENTS.md).
- ✅ cargo-outdated (version freshness, informational `--exit-code 0`) — 12 patch-level updates applied; 4 semver-major tracks applied (toasty 0.10, base64 0.23, brotli 8, sha2 0.11) with breakage verified — zero code changes required; residual outdated entries are graph-inherent dual-major pins in the generated hakari crate.
- ✅ CI hygiene — `.cargo/audit.toml` at `.cargo/` (cargo-audit has no `--config` flag and does not expand `~` in config paths); 1381 tests pass, clippy `-D warnings` clean, fmt clean.
- ✅ Clippy-allow audit (2026-08-27) — every `#[allow]/#![allow(clippy::...)]` in the workspace reviewed on Rust 1.98: 38 stale/fixable attributes removed and the source fixed instead (14 `manual_let_else` rewrites, 70 `must_use`/const fixes via `clippy --fix`, `option_if_let_else` → `map_or_else`, iterator loops replacing index loops, heap-allocated large arrays, vmess `Cipher` variants boxed for `large_enum_variant`, XUDP `open_udp_session` de-asynced with all call sites migrated, cast sites converted to checked arithmetic); 16 defensible keeps rewritten as two-line attrs with explicit `reason = "..."`. Clippy pedantic+nursery zero-warning workspace-wide; quality-gate code green.

## Phase 18 — Native SOCKS5 (client + UDP inbound) ✅

- ✅ SOCKS5 client handshake — `protocol/socks` replaces the placeholder: greeting → method selection → RFC 1929 sub-negotiation → CONNECT → reply, each step under `timeouts::PROTOCOL`, wired into `protocol::connect`. Credentials come from unvalidated share URLs, so an over-long one is `Socks5Error::CredentialTooLong`, never a panic; an empty username counts as absent (no `ULEN = 0` frame on the wire); `REP` is checked before `RSV` and a non-zero reply `RSV` is logged, not fatal (neither fast-socks5 nor xray validates it). The client's UDP side stays `NotImplemented`: SOCKS5 datagrams need a raw UDP socket, not the (possibly TLS-wrapped) stream this layer sits on.
- ✅ UDP ASSOCIATE in the inbound — `Socks5InboundConfig::udp` (default on) replaces the `0x07` refusal. One relay task per association routes EVERY datagram through the engine (`NetworkMask::UDP`), so an association may reach direct and proxy outbounds at once; `frag != 0` is discarded; reply headers reverse v4-mapped v6 addresses. The association pins to the control connection's peer address (an unpinned local process could otherwise hijack it), expires via `UDP_PIN_DEADLINE` when no datagram ever arrives, and ends on control-TCP EOF — the control read half is polled from the first iteration, so an associate-then-disconnect client cannot park the task and leak its two sockets.
- ✅ Splittable datagram carriers — `PacketTunnel::split() -> (PacketReader, PacketWriter)` with per-carrier halves (`vless::{PacketReader,PacketWriter}`, `vmess::udp::…`, `trojan::…`, `hysteria2::udp::{UdpReader,UdpWriter}`); each carrier now separates per-direction framing state from its transport (`VmessClientStream` = transport + `VmessRead`/`VmessWrite`; hysteria2 shares its h3 keepalive through an `Arc` so the session outlives either half), and vless/trojan frame readers resume a cancelled read from struct-held progress. `PacketTunnel::recv` spans several awaits for the stream carriers, so racing it in a `select!` desynchronised the tunnel; the proxy leg (`run_proxy_leg`) now owns the split tunnel with the reader in its own task. VLESS XUDP refuses to split (shared mux tunnel, no halves).
- ✅ Per-datagram failure isolation — a malformed header, unroutable destination, failed bind/send, refused per-packet destination or full proxy queue drops that datagram only; the leg's exit clears itself so a later datagram re-opens the tunnel after `PROXY_REOPEN_BACKOFF` (no permanent blackhole latch). Domain destinations resolve off the relay loop (spawned task for direct, inside the leg for proxy) so one DNS miss cannot stall the other flows or control-EOF detection; both upstream family sockets are bound together, so an `AAAA`-only name is reachable.
- ✅ Routing fidelity — the sniffer receives the WHOLE datagram, gated on `Engine::needs_sniff()`: a QUIC Initial is padded to ≥1200 bytes and the QUIC arm rejects a truncated packet, so the previous 512-byte prefix made `protocol: quic` rules unmatchable.
- ✅ Micro-optimizations — `TCP_NODELAY` on accepted inbound sockets, transport dials and direct outbound dials (Go cores set it by default); the greeting is read in two bulk reads instead of one per method byte; `reply_for` maps `io::ErrorKind` to RFC 1928 §6 codes (an unresolvable domain answers `0x04`, a refusal `0x05`, a local-firewall `EACCES` `0x02`) while dial errors keep both the kind and the target address.
- ✅ Tests — 494 native lib tests (from 442): UDP echo round-trip, fragment drop, block-drop with association survival, source pinning, multi-destination fan-out, teardown-without-a-datagram and control-EOF both proven by reply-port release, `unmap_v6`, every mapped `reply_for` kind, UDP header codec round-trip + malformed inputs, `Command` discriminants, greeting edge cases, exact client-handshake wire bytes (domain + IPv6 targets, RFC 1929 frame, `0xFF`/GSSAPI/over-long-credential refusals), and per-carrier split round-trip, concurrent write-during-pending-read, cancel-safety and EOF. Quality gate green (1698 nextest).

## Phase 19 — Secret Wiping (`zeroize`) ✅

- ✅ Crypto dependency features — the load-bearing half, invisible in source: `zeroize` is NOT a default feature of `aes` 0.9 / `aes-gcm` 0.11 / `chacha20poly1305` 0.11 / `ctr` 0.10 / `hmac` 0.13 / `sha2` 0.11 / `md-5` 0.11 / `blake2` 0.11, so every VMess/VLESS cipher instance was leaving its expanded round keys, ChaCha state and HMAC blocks in freed memory; and `zeroize` IS in `x25519-dalek`'s defaults, which `default-features = false` silently stripped, so the REALITY dual-agreement scalar and every `SharedSecret` were dropped unwiped. Both manifests now list the feature explicitly (verified via `cargo tree -e features`); zero code churn.
- ✅ TLS engine key material — `Zeroizing` on the whole TLS 1.3 key schedule (`hkdf_extract`, `hkdf_expand_label`, `derive_secret`, `handshake_secret`, `master_secret`, `finished_key`, the new `crypto::TrafficSecrets` pair alias), the TLS 1.2 PRF chain including every `A(i)` intermediate (`p_hash`/`prf`/`master_secret*`/`key_block`/`finished_verify_data`), `X25519KeyPair::agree`, `reality::auth::derive_auth_key`, and the hybrid `pq ‖ classical` IKM buffer. Wire-visible outputs (`transcript_hash`, `finished_mac`) stay plain.
- ✅ ML-KEM secrets — `mlkem::SecretKey`/`SharedSecret` are `#[derive(ZeroizeOnDrop)]` with hand-written length-only `Debug` (the derive printed a 2400-byte decapsulation key into any log line). liboqs never wipes its own `newtype_buffer!` `Vec<u8>`, so `encapsulate`/`decapsulate` take ownership via `into_vec()` into a `Zeroizing` instead of copying out of `as_ref()`.
- ✅ Native protocol key material — VMess (`cmd_key`, `chacha20_key_32`, the AEAD-KDF's heap `msg`/`pad_key` buffers, and `Session` via `#[derive(ZeroizeOnDrop)]` covering all five key/IV fields with no call-site churn), VLESS `mlkem768x25519plus` (`nfs_key`, `united`), the trojan hex-SHA224 auth token (a replayable bearer credential), `Salamander.psk` (+ redacted `Debug`), and the SOCKS5 RFC 1929 credential frame.
- ✅ Documented non-goals — ring's `LessSafeKey`/`Prk`/`hmac::Tag` (opaque, no wipe hook — the derivation buffers on either side are the wipeable surface), AEAD write IVs (useless without the ring-held key), wire-visible values (nonces, salts, MACs, transcript hashes, public keys, `pin_sha256`, REALITY `short_id`), QUIC Initial keys in `xray-tui-route::sniff` (derived from a public salt + DCID — any observer derives them), and the `xray-tui-proto` config structs + `Socks5InboundConfig.auth` (the same bytes sit plaintext in SQLite and the generated core config JSON and live until process exit).
- ✅ Secret-exposure audit (non-zeroize) — the generated core config JSON is written into a `tempfile::TempDir`, which is created 0o700, so the 0o644 file inside is not reachable by other local users; secrets reach the core via that file path only, never argv; no `tracing` macro in the core/config/native/proto/tui crates interpolates a credential. The two real leak vectors found were the `Debug` derives on the ML-KEM secret types and on `Salamander`, both fixed.
- ✅ Verification — 1698 workspace tests pass unchanged (RFC 8448 key-schedule vectors, RFC 7748 X25519 known-answers, REALITY seal round-trip, hybrid-PQ handshake, Go-verified VMess vectors, trojan wire order), proving byte-identical crypto output; clippy pedantic+nursery clean (`type_complexity` on the traffic-secret pairs resolved with the `TrafficSecrets` alias, not an allow), fmt clean, hakari regenerated.

## Phase 20 — Native Core as Runtime Backend

- ✅ Runtime-only core selection — the in-process core (`xray-tui-native`) is now a first-class backend, chosen AT CONNECT TIME and never persisted. Only the runtime enum moved: `xray_tui_core::CoreType` gains `Native` (Display/`as_str`/`FromStr` `"native"`); the persisted `xray_tui_proto::proto_spec::CoreType` stays `{Xray, SingBox}`, `resolve_core` still returns concrete values, and no native value ever reaches the DB or a generated core config — **no schema or persisted-enum change shipped**. Both reasons are load-bearing: `core_type` is a field of the identity-hashed `ProtocolEssentials`, so re-stamping it re-keys `ProtocolId` (= `ParsedProto::uid()`) and duplicates rows; and toasty renders the column as `TEXT ... CHECK ("core_type" IN ('xray','sing_box'))`, frozen in every already-created DB (the `PRAGMA user_version` tag is 6 and a mismatch makes `Database::open` DELETE the file — never a migration).
- ✅ Connect-time gate — `resolve_runtime_core(link_core, kind, forced, config, proxy_all_blocked)` (private in `ops/connect.rs`) owns the decision: native when the link's concrete core is `Xray`, the kind is vless/vmess/trojan/hysteria2 (`capability::kind_supported`), no `protocol_core_overrides` entry forces xray/sing-box, and `capability::supported(kind, config)` passes on the LOADED config. A `native`/`auto` override asks for native (`asks_native`) and still faces the same gate — never silently native. Every refusal warns with host, kind, chosen core and rule/DNS counts (`REFUSED_KIND` / `REFUSED_PROXY_ALL` / `REFUSED_UNLOADED` / `REFUSED_CAPABILITY`). No new toggle shipped: `PROTOCOL_CORE_DEFS` (Settings → Protocol Core) and the per-profile core field still offer `Auto,Xray,SingBox`, and Auto resolution reaches native without one — spelling `native` in a `protocol_core_overrides` value is a config-file opt-in.
- ✅ Capability gates — `crates/xray-tui-native/src/capability.rs` (`NATIVE_KINDS`, `kind_supported`, `supported`) defers every row native would serve worse than the subprocess: VLESS account encryption (incl. `mlkem768x25519plus`, where native diverges from real-xray interop), non-vision VLESS flows, legacy VMess payload ciphers, non-zero `alter_id`, TLS fingerprint ids outside the five `security::fingerprint::parse_fingerprint_id` accepts (the same parser the dial uses, so the gate cannot drift from it), and mKCP `seed` / `header_type` including the share-link `path` seed carrier. Transports are matched POSITIVELY, so a new `TransportConfig` variant fails closed at compile time.
- ✅ `NativeCoreServer` — `xray-tui-native/src/server/mod.rs`: `ServerConfig::new(socks, http, proxy, telemetry)` binds a SOCKS5 inbound (UDP ASSOCIATE on) plus an optional HTTP CONNECT inbound over a proxy-all engine. `stop()` is the watch-channel signal, `shutdown()` awaits the accept loops so the ports are released before the next `start` (no EADDRINUSE), `Drop` signals first and then aborts them, a failing HTTP bind unwinds the already-serving SOCKS listener, and a non-loopback bind warns that neither inbound authenticates its clients.
- ✅ HTTP CONNECT inbound — `inbound/http.rs`: bounded head scan (`MAX_HEAD_BYTES` 16 KiB, checked mid-line) that replays bytes the client pipelined behind the head into the tunnel (dropping them truncated the stream and stalled it), framed `400`/`403`/`407`/`431`/`501`/`502` refusals (`Connection: close` + `Content-Length` + a one-line reason), optional `Proxy-Authorization: Basic` compared by DECODING the client token (one indistinguishable 407 for missing/malformed/wrong), open-relay warning at bind, transient accept errors absorbed.
- ✅ Telemetry feed — `telemetry.rs`: `Telemetry` / `NativeEvents` with SEPARATE log and trace queues (traces drain first — the open/close pair matters more than the line), `TraceEvent::Opened`/`Closed` rows, `TraceGuard` emitting `Closed` even when the relay future is dropped mid-leg (inbound shutdown, task abort), `Counted<S>` wrapping each relayed stream to feed the shared byte atomics live, and per-window drop counts folded into one summary instead of one line per loss.
- ✅ Native session arm — `crates/xray-tui/src/ops/native_connect.rs::run_native_session` replaces the subprocess arm for native sessions: binds the listeners, emits `CoreEvent::Connected(CoreType::Native)`, forwards telemetry on the EXISTING `StatsUpdate` / `SysStatsUpdate` / log shapes (so persistence and the Statistics screen behave identically; RSS from `/proc/self/status` `VmRSS`, which needs no page-size assumption) plus the new `CoreEvent::NativeTrace`, then `server.shutdown().await` + `Disconnected`.
- ✅ Native Activity tab — `Tab::NativeActivity` + `crates/xray-tui/src/ui/native_activity.rs`: live per-connection trace rows from `AppState.native_activity` (`NativeActivityLog`, 2000-row session ring in `types.rs`, reset on each native `Connected`, still-open rows closed out on `Disconnected` so the counters stay exact), a `NATIVE_TRACE_BUDGET` of 512 traces per frame in `ops/events.rs`, window-then-format render and clamped scrolling.
- ✅ One-core-at-a-time now covers the in-process server (extends Phase 3) — `connect_to_profile` awaits the previous session task (`PREV_SESSION_TEARDOWN` 5 s, then abort) before binding its own listeners; `disconnect()` leaves a bounded waiter that gives the session `DISCONNECT_TEARDOWN` 3 s to run `server.shutdown()` + `Disconnected` instead of aborting it outright; `ui::run`'s exit path awaits that waiter before leaving the alternate screen.
- ✅ Two exit hangs fixed — the crossterm reader (`event::read()`) and the heed log batch writer (`std::sync::mpsc::recv()`) were `spawn_blocking` tasks that never returned, and `Runtime::drop` waits for in-flight blocking work, so quit parked forever. Both now poll in 200 ms slices (`IDLE_POLL`) against the canonical `AppState::shutdown_token`, which `main` stores unconditionally after `ui::run` returns and also from the panic hook (a panic unwinds past the normal store).
- ✅ Wiring — `xray-tui` takes an unconditional path dependency on `xray-tui-native`, whose tokio gains the `sync` feature; `cargo hakari verify` passes. Design brief: `docs/native-core-integration.md`; per-protocol matrix: `NATIVE_CORE.md`; keys and tab: `TUI_MANUAL.md`.
- ☐ Native real ping — real ping and speed test still need an xray or sing-box binary: the JSON config builders and the temp ping cores dispatch on the link's concrete `core_type`, so there is no native real-ping path and a native-only install fails real pings with `Binary not found`. Fast (TCP) ping is unaffected — it never spawned a core.
- ☐ Native routing + DNS — native is proxy-all, so any routing rule or non-default DNS setting demotes the connect to xray-core BEFORE `BuildParams` is built (the api flags follow the real core). Lifting this needs the routing engine and a DNS path wired into the native session.
- ☐ HTTP inbound surface — CONNECT-only (absolute-form/plain proxying answers `501`) and no user-facing auth: `HttpInboundConfig::auth` exists in the crate, but the profile model has no credential source, so the native session leaves it `None` and relies on the loopback bind.
- ☐ SOCKS5 UDP through a SOCKS5 outbound — `protocol/socks` has no `PacketTunnel` (`NotImplemented`: "socks5 udp (needs a raw UDP socket shape, not the stream tunnel)"), so a socks:// upstream carries TCP only. Inbound UDP ASSOCIATE over the four native proxy outbounds landed in Phase 18.
- ☐ Status-bar hints clipped (~25 columns, every tab) — pre-existing right-alignment miscount, not introduced here: `ui/status_bar.rs` sizes its filler from `str::len()` (bytes, not display columns) and inserts the update/download indicator spans AFTER that math, so the composed line overruns `area.width` and `Paragraph` truncates the tail of every documented hint row.
