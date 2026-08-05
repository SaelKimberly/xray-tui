# xray-tui — Agent Guide

## Project Identity

Rust TUI client for Xray-core and Sing-box. Target: feature parity with v2rayN (C# GUI) for all protocols supported by either backend.

## Quick Start

```bash
cargo build --release
cargo run
```

## Key Source Files

### Crate entry points
- `crates/xray-tui/src/main.rs` — binary entry, tokio::main, TuiLogLayer tracing subscriber + subsystem init
- `crates/xray-tui/src/lib.rs` — crate root, module wiring, `try_send_or_warn` CoreEvent helper. `AppState` (incl. profile_gen) in `state.rs`; `Tab`, `SortColumn`, `ProfileRow`, `LogLine`, `SettingsMode` (Split variant), `SettingsSection` (incl. SpeedTest), `SplitFocus`, `SplitRightPane`, `CoreEvent` (incl. TestTypeUpdate, HostFeaturesLoaded, EndpointInfoUpdated), `EndpointInfo`, `PingRound`/`EndpointPingStatus` (per-endpoint ping rounds), `testing_details` map, `endpoint_info` map, `ping_status` map, `format_ts`, `iso_to_flag` in `types.rs`
- `crates/xray-tui/src/ops/enrich.rs` — background enrichment engine: spawn_dns_resolve (TTL-gated via dns_cache_ttl_secs, `x` force), spawn_enrich_ip_hosts (startup seed from persisted resolved_as, no network), spawn_whitelist_pass (on HostFeaturesLoaded), spawn_outbound_enrich (real-ping exit IP), extract_sni (typed SecurityConfig::sni() — covers tls+reality), protocol_row_to_profile, should_resolve, fill_features
- `crates/xray-tui-core/src/lib.rs` — core logic facade
- `crates/xray-tui-db/src/lib.rs` — re-export hub; Database, DatabaseError, Result public
- `crates/xray-tui-db/src/error.rs` — DatabaseError, Result, ProfileWithDetails
- `crates/xray-tui-db/src/database.rs` — Database struct + all public query/write methods (toasty ORM, replaced raw turso SQLite)
- `crates/xray-tui-proto/src/` — Protocol config types (VMess, VLESS, Trojan, Shadowsocks, SOCKS, HTTP, WireGuard, Hysteria2, Hysteria1, TUIC, Naive, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR) with URL parsing/splitting infrastructure + Clash YAML conversion. Identity: crate-private `ProtoIdentity` trait + `Proto` container (OnceLock sig/cred_hash cache) in `proto_spec/mod.rs`. Adopted from sub-healer project. 137 unit + 3 doc tests.
- `crates/xray-tui-db/src/models_toasty.rs` — toasty Model definitions for all 9 tables (Endpoint, ProtocolRow, EndpointGroup, Group, ProfileExtension, ServerStat, RoutingRule, DnsSetting, PingSession); non-model types (ProfileWithDetails, PingResultUpdate); constants (GRAVEYARD_GROUP_ID, ALL_GROUP_ID, DELAY_SOURCE_FAST/REAL/UDP); ProfileExtension.delay_source (latency provenance: 0 fast, 1 real, 2 udp)
- `crates/xray-tui-db/src/retry.rs` — `retry_on_busy`/`is_busy_error`: retry-with-backoff (20ms doubling, 1.28s cap) for SQLite write contention (`is_serialization_failure` or "database is locked"); wired into `update_endpoint_resolution` + `batch_flush_ping_buffer` so the enrichment herd never drops writes. `Database::conn()` (database.rs) additionally sets `PRAGMA busy_timeout=5000` on EVERY pooled connection acquisition — the pragma in `open()` is per-connection and never reaches pool-created conns (the real cause of the lock-failure dumps); all write paths go through `conn()`
- `crates/xray-tui-core/src/config_builder/mod.rs` — BackendConfig enum, BuildParams, BuildError, ConfigBuilder struct, shared `parse_settings()` (extracts p_settings/s_settings from ProtocolRow spec_blob)
- `crates/xray-tui-config/src/lib.rs` — config management, module registration
- `crates/xray-tui-core/src/grpc_client.rs` — StatsProvider trait + GrpcStatsClient + MockStatsProvider (test double) + factory (unified from former XrayGrpcClient/SingBoxGrpcClient)
- `crates/xray-tui-core/src/updater.rs` — backend auto-update (version check, download, install) for xray-core and sing-box
- `crates/xray-tui-config/src/import_export.rs` — share URL parse/format (14 protocols + fallback chain) with per-profile validation via ValidationSettings
- `crates/xray-tui-config/src/base64_util.rs` — robust base64 decode with percent-decoding and annotation stripping
- `crates/xray-tui-dns/src/lib.rs` — DnsResolver: DNSCrypt stamp parsing (dns-stamp-parser) → hickory-resolver 0.26 config, cached resolver list, panic-free async OnceCell init
- `crates/xray-tui-geoip/src/lib.rs` — GeoIp: GeoLite2-City mmdb download + country/city lookup (maxminddb 0.30)
- `crates/xray-tui-host-features/src/lib.rs` — HostFeaturesChecker: SNI/exact-IP/CIDR whitelist membership checks (fastbloom fast-negative guard + exact HashSet/interval verification, IPv4-only), download-if-missing from hxehex/russia-mobile-internet-whitelist, `get_host_features(&ServerName)` main API
- `crates/xray-tui-config/src/permissive_json.rs` — lenient JSON parser for vmess:// subscriptions
- `crates/xray-tui-config/src/fast_perc.rs` — hand-rolled UTF-8 + percent-decoding character source
- `crates/xray-tui-config/src/subscription.rs` — chunked base64 streaming decoder with URL splitting
- `crates/xray-tui-core/src/speed_test.rs` — async speed test engine (tcp_ping, real_ping, speed_test, udp_test, udp_ping) using tokio + reqwest SOCKS5 proxy. RealPingResult includes latency + ISP info. Configurable via SpeedTestConfig. Batch versions (start_batch_ping, start_batch_then_real_ping) in `lib.rs` with FastPingManager dispatching TCP/UDP/QUIC adapters.
- `crates/xray-tui-core/src/ping/mod.rs` — FastPingAdapter trait + FastPingManager adapter registry, PingCapability enum (Tcp/Udp/Quic/None), PingError, PingResult, ProfileKey structs
- `crates/xray-tui-core/src/ping/adapters/mod.rs` — adapter trait registration, TcpPingAdapter, UdpPingAdapter (QuicPingAdapter gated by quic-ping feature)
- `crates/xray-tui-core/src/ping/adapters/tcp.rs` — TCP handshake ping adapter supporting VMess, VLESS, Shadowsocks, SOCKS, HTTP, Trojan, Naive, AnyTLS, ShadowTLS, Tor, SSH, Redirect, TProxy, Mixed
- `crates/xray-tui-core/src/ping/adapters/udp.rs` — direct UDP ping adapter for WireGuard and ShadowsocksR endpoints
- `crates/xray-tui-core/src/ping/adapters/quic.rs` — QUIC handshake ping adapter (quic-ping feature) for QUIC-enabled protocols
- `crates/xray-tui-core/src/ping/real/mod.rs` — RealPingManager: launches temp core binary to test profile via SOCKS5 HTTP requests with IP info fetch. Port pool via `next_ping_port: Arc<AtomicU16>` + `allocate_port()` shared across concurrent tasks.
- `crates/xray-tui-core/src/ping/real/pool.rs` — CorePool: single warm core process for single-ping reuse with SIGHUP reload (sing-box) or stop+restart (xray-core). POOL_TTL (30s). Atomic port allocation. Error propagation.
- `crates/xray-tui-core/src/log_heed.rs` — HeedLogStorage: LMDB-backed persistent log storage (postcard-encoded LogMessage entries, two databases for logs + targets)
- `crates/xray-tui-core/src/process.rs` — CoreManager trait (start/stop/is_running/running_core_type/sighup_reload/rewrite_config) + RealCoreManager (subprocess lifecycle, stdout/stderr capture via required log_tx: Sender<String>) + MockCoreManager (test double)
- `crates/xray-tui-proto/src/clash/mod.rs` — Clash YAML proxy structs (ClashProxy enum + 29 per-protocol Clash config structs with kebab-case serde) for bidirectional conversion between Clash YAML and internal ProtocolConfig types

### TUI screens (crates/xray-tui/src/ui/)
- `mod.rs` — run(), render(), event loop, keyboard handler, tab routing, AppMode dispatch, speed test menu overlay
- `profiles.rs` — profile list DataGrid with connected indicator, Test column (colored delay + `[name]`/`[fast]`/`[real]` labels), Feat flag column, multi-sort indicators (Test column sortable: ranks endpoints by best protocol's test priority), graveyard group filter, tree markers (▶/▾) for expandable endpoints, column separators; sub-row protocol variants; reverse-highlighted selected sub-row; multi-select, delete confirmation, batch import overlay; expand/collapse (←/→), variant navigation (↑↓), Enter to activate variant
- `add_server.rs` — form rendering, protocol picker, field editing, import URL screen
- `settings.rs` — Settings panel with split-pane tree+form view. Left pane: collapsible tree (SPLIT_SETTINGS_TREE const, SettingsSection-navigated). Right pane: Form, UpdateForm, GroupList, Empty. Sections: Core, GUI, Inbound, Routing, DNS, System Proxy, TUN, Mux, Statistics, Protocol Core, Updates, Speed Test, Logging, Subscriptions (14 total). Tree navigation (arrows) + form focus switching (Ctrl+W). Full rewrite. Replaced per-section SettingsMode variants with unified Split { tree, focus, right }.
- `groups.rs` — system-group-aware management (is_system guard, clear action)
- `logs.rs` — live core log viewer with cursor highlight, scrollable display, color-coded log levels (error/warning/info/debug), keyboard navigation (Up/Down/PgUp/PgDn/Home/End), source filtering (t/T), multi-line selection (Shift+Up/Down), copy-to-clipboard (y/Y), confirmation overlays for clear/purge
- `theme.rs` — central Palette-derived style methods (ThemeStyles struct) — static methods returning Style from a &Palette
- `statistics.rs` — statistics screen with 3 bordered sections (traffic, system stats, connection info)
- `actions_log.rs` — live state info panel: connection status, server info, test results, traffic/memory, recent logs
- `palette_bridge.rs` — maps ratatui-themes `ThemePalette` → ratatui-cheese `Palette` (10-to-11 color role mapping)
- `widgets/` — reusable widgets: `DataTable` (sortable, selectable, multi-select, virtual-scrolled with themed scrollbar, Column/ColumnWidth/SortDirection/DataTableRow trait)

### Reference repos (read-only — never edit)
- `thirdparty/Xray-core/` — protocol behavior, config schema, API
- `thirdparty/sing-box/` — sing-box protocol types, config format, API compatibility
- `thirdparty/v2rayN/` — feature reference, UI patterns

### Key Rust proxy references
- `thirdparty/shoes/` — Rust proxy protocol implementations (VMess, VLESS, Reality, etc.)
- `thirdparty/leaf/` — Embeddable Rust proxy framework pattern
- `thirdparty/shadowsocks-rust/` — Mature SS implementation, protocol dependency

### Sing-box reference files (critical for implementation)
- `thirdparty/sing-box/constant/proxy.go` — all protocol type strings (`TypeTUIC`, `TypeHysteria`, etc.)
- `thirdparty/sing-box/option/outbound.go` — per-protocol outbound options structs
- `thirdparty/sing-box/option/options.go` — top-level JSON config struct (`_Options`)
- `thirdparty/sing-box/option/rule.go` — routing rule options
- `thirdparty/sing-box/docs/configuration/outbound/` — human-readable config docs per protocol
- `thirdparty/sing-box/experimental/v2rayapi.go` — V2Ray API experimental feature
- `thirdparty/sing-box/docs/configuration/experimental/clash-api.md` — Clash API feature

## Critical Design Decisions

1. **Dual-backend architecture**: `CoreManager` abstracts over xray-core and sing-box subprocesses. TUI writes JSON configs and manages binary lifetime.
2. **Protocol-core auto-resolution**: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → sing-box. All others (VMess, VLESS, etc.) → xray-core by default. Shadowsocks/Shadowsocks-2022 is cipher-aware: AEAD + 2022-blake3 methods → xray-core, legacy methods (`aes-*-cfb`, `aes-*-ctr`, `rc4-md5`, `chacha20-ietf`, `xchacha20`, `none`) → sing-box (xray-core's `CipherType` enum has no legacy entries — `XRAY_SS_METHODS`/`SINGBOX_SS_METHODS` in `protocol_core_mapping.rs` are the whitelists). Both config builders reject ciphers neither core supports. User overrides per-profile; a forced core that can't build the cipher fails at build time with a clear error, never an invalid config.
3. **One core at a time**: Only one backend process runs per connection session. Switching profiles between backends stops current core and starts other. Matches v2rayN.
4. **SQLite via toasty ORM (async)** — Single DB file for all persistent data. Models defined via `#[derive(toasty::Model)]` in `models_toasty.rs`. Schema auto-managed by toasty's `db.push_schema()`. All DB methods are `async fn` on `Database` struct backed by `toasty::Db`. Database::open() accepts impl AsRef<Path>. System groups (All, Graveyard) created on first open. Toasty v0.9 with `turso` driver for async SQLite.
5. **Config generation** — Two builders: xray.rs (ports v2rayN's CoreConfigContextBuilder) and singbox.rs (ports sing-box JSON format).
6. **gRPC stats abstraction**: `StatsProvider` trait with unified `GrpcStatsClient` (both backends share the same V2Ray Stats gRPC API).
7. **Sing-box config differs structurally** from xray-core: `type` vs `protocol`, `route` vs `routing`, `experimental.v2ray_api` vs `stats`+`api`+`policy`, different TLS/transport key names.
8. **Theme system**: `ThemeStyles` struct (in `theme.rs`) provides static methods returning `Style` from a `&Palette`. Palette is constructed from `ratatui_themes::ThemeName` -> `ratatui_themes::Theme` -> `palette_bridge::current_palette()`. AppState::current_palette() is canonical accessor. Every screen accepts `&Palette` and calls `ThemeStyles::*` methods instead of hardcoded colors.
9. **Sing-box V2Ray API is experimental**: May require build tag `with_v2ray_api`. If unavailable, stats/logs show "not supported by core".
10. **spec_blob + bridge traits**: Profile data stored as hybrid — cached fields (address, port, transport, security) + `spec_blob` (JSON-encoded ProtocolConfig; the `Proto` identity container wraps it transparently, so serialized bytes are identical). Bridge traits `ProfileLegacy::leg()` and `ProfileMut::set_xxx()` in xray-tui-config enable old parse/format function code to read/write the new schema without full rewrite. `Connection` table replaces `Profile.group_id` for many-to-many group membership. `Profile.id` is i64 (uid = sig ^ cred_hash).

11. **Dual uid scheme**: Profiles parsed from share URLs get `uid = sig ^ cred_hash` from `ProtoSpec` (deterministic, dedup-compatible). Profiles created via TUI form get a random i64 PK because no URL was parsed — `sig` and `cred_hash` are set to `uid` and `0` respectively and are meaningless. Form profiles don't participate in URL-based dedup. Identity computation is normalized: crate-private `ProtoIdentity` trait (`compute_sig` + `compute_cred_hash`) feeds the `Proto` container's `OnceLock<Identity>` cache (`sig()`/`cred_hash()`/`uid()`); `ProtoSpec: ProtoIdentity` is sealed (`#[allow(private_bounds)]`). Per-config sig/cred_hash cache fields were removed.
12. **Clash YAML conversion**: `ProtoSpec` trait has default `try_from_clash`/`to_clash` methods (both return Err). Per-protocol overrides implement bidirectional conversion between Clash YAML structs and internal ProtocolConfig types. `common.rs` provides conversion helpers for TLS/security, transport, host/port. `dispatch!` macro in `mod.rs` routes `try_from_clash` by `ClashProxy` variant. Hysteria2 uses `port_spec_serde` (PortSpec); all other protocols use `port_serde` (u16).
13. **Background enrichment pipeline** (`ops/enrich.rs`): DNS resolution, mmdb country lookups, and whitelist checks run in spawned tokio tasks reporting via `CoreEvent::EndpointInfoUpdated`; the UI thread never blocks. `EndpointInfoUpdated` handling merges by field group (concurrent resolution/whitelist/outbound events must not clobber each other); failed DNS lookups (empty IPs) materialize TTL-gated attempt entries so auto-retriggers don't re-hang; `x` (force) always re-attempts. Hosts failing `is_resolvable_hostname` (plugin URLs like `host:port?plugin=...`, underscores from Telegram-channel names, non-ASCII) are skipped before lookup and recorded as failed attempts — hickory's "Label contains invalid characters" spam never fires. DNS persistence = two plain columns on `Endpoint` (`resolved_as` comma-joined IPs, `resolved_at` unix secs) — country and whitelist features are never persisted (recomputed per launch). Hard deadlines: 10s DNSCrypt-list download + 8s overall lookup timeout (a blocked network must degrade, never hang).
14. **Profiles tab row model**: each endpoint = one variable-height DataTable row (collapsed 1 line; expanded `1 + panel_rows + 4 + 1` — endpoint line + rounded panel (border+IPs+separator+sub-rows+border) + 1 gap line so the panel bottom border never touches the next row). 17 fixed columns; scroll offset is computed in line units (`compute_scroll_offset`: ideal centering clamped to a height-aware `max_offset` and a visibility floor — row-index math strands the last profiles when rows expand). Expansion nav: expand lands on sub-row 0; `↑` at sub 0 → full row; `↓` at last sub-row → next profile; `↓` from full row of an expanded endpoint re-enters sub 0; collapsed endpoints move on one `↓`. Stale `selected_sub` is cleared on every endpoint-nav fallthrough.
15. **Batch ping pipeline** (`ops/ping.rs`): Phase 1 fast-pings a page concurrently (`run_page_pings` — ONE TCP ping per unique address:port, `buffer_unordered` capped by `fast_ping_concurrency` default 200, in-page owner/follower dedup + cross-page `fast_cache`). Fast ping probes inbound reachability only (all protocols of an endpoint share its address:port), so results are all-or-none per endpoint — the existing `all_unreachable(protocol_count)` marker math is correct without candidate-count machinery. Phase 2 real pings dispatch wave-by-wave: `get_batch_for_real_ping(batch_id, wave, limit, dedup_endpoints)` computes occurrence ranks over ALL real sessions of the batch (status-independent — ranks stay stable after each dispatch) with `status='queued'` + `occurrence=?wave` filtered outer; the phase-2 consumer is one wake/drain pass loop (waves 1..N, `batch_page_size` chunks, 200ms coalescing sleep, empty-wave termination). A failed protocol defers its endpoint's siblings to the next wave; the `[real]` label fires only when every candidate was tested and none succeeded. After one protocol of an endpoint succeeds, siblings are skipped (`real_ping_test_all_protocols=false`, default) or still tested (true). `cancel_stranded_real_pings` is gated on `dedup_endpoints` — endpoint-scoped batches (dedup=false) must test every protocol, so a success must not cancel siblings mid-batch. Group-level failures (config build / core start / no port ready) retry with stack-based page-halving down to per-profile cores (missing binary = not retryable); per-port SOCKS5 readiness is waited in parallel (`join_all`), a dead port = item-level failure. `Cancelled` test results never count as round failures. Sub-table rows are sorted by test priority (decision 16) in `deserialize_endpoint_rows` (db crate) — the single source both the panel and sub-row nav index. Enter on a sub-row pins `manual_protocol_override`; Enter on the endpoint row clears it. **In-memory sync rule**: `endpoints_gen` is write-only and `reload_profiles` runs only on subscription events, so any op mutating endpoint state (`set_protocol_default`/`set_active` in `ops/profiles.rs`) must patch `state.endpoints` rows directly or the UI shows stale data. Selected sub-row renders as an accent bar via `ThemeStyles::panel_row_selected` (fg on_highlight/bg highlight) — the endpoint row highlight (`table_row_selected`) paints the whole expanded panel, so an identical sub-row style would be invisible. Profiles grid has no zebra striping: unselected rows share one background (`table_row_normal`); `table_row_alt` was removed.
16. **Test-priority ordering**: sub-table protocols and the main-table Test sort share one comparator (`EndpointRow::sort_protocols_by_test_priority` / `best_test_priority_key` in the db crate; ascending key `(tier, latency, -last_seen_at, id)`). Tiers: 0 real-ok, 1 fast/udp-ok, 2 untested, 3 real-err, 4 fast-err, 5 DNS-unresolved; fresh failures dominate stored successes (user decision); latency orders only tiers 0-1 (tiers 2-5 by last_seen_at desc then id). Provenance persisted via `profile_extensions.delay_source` (SCHEMA_VERSION 4; 0 fast, 1 real, 2 udp; None/-1 = no measurement; upserts write `unwrap_or(-1)`); writers are the `SpeedTestResult` handler (from `test_type`) and `batch_upsert_buffer` (from session `ping_type`). Live re-sort happens in the `SpeedTestResult` handler for TcpPing/RealPing results (success or failure) and on `EndpointInfoUpdated` DNS unresolved→resolved flips; `selected_sub` is remapped by protocol id (only when the result's endpoint is the selected one); `filter_cache_valid` invalidated after re-sorts. **Error events mutate nothing** (in-memory ext nor DB — the `if error.is_none()` gate wraps the whole ext mutation + upsert; a Cancelled RealPing must never rank an untested protocol as real-ok). Main table: `SortColumn::Test` (renamed from `Delay`) ranks endpoints by best protocol's `(tier, latency, -last_seen_at, id)`; load-time sort uses `resolved_as` for the DNS tier (rounds = None at load — failures are session-only).

## Protocols: In Scope

### Xray-core native
VMess (AES/Chacha20/None/AEAD security), VLESS (with flow control: xtls-rprx-vision), Shadowsocks, Shadowsocks-2022, SOCKS, HTTP, Trojan, WireGuard, Hysteria v2, Dokodemo-door, Freedom, Blackhole, DNS, Loopback, Custom

### Sing-box only (extra beyond xray-core)
TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect, TProxy, Mixed (inbound)

## Protocols: Out of Scope

Anything requiring a third binary backend beyond xray-core or sing-box.

## Common Tasks

**Phase overview**: Phases 0-6 (Foundation through Settings) fully implemented. Phase 6 includes split-pane settings refactor (unified SettingsMode::Split replacing per-section form variants). Phase 7 (Advanced Features) completed: logs tab, sing-box config builder for all 17 outbound protocols, normalized profile schema, advanced form fields (TLS/ECH/Fragment/uTLS/mux), speed test config with batch-then-real-ping, profiles table redesign (connected indicator, IP info, graveyard filter), Heed-backed log storage (HeedLogStorage, LMDB, Settings→Logging form), Clash Mixin, geo auto-update, log-to-file toggle, certificate pinning UI, default skip cert verify, clear all stats, mux protocol selector. Phase 8 (Polish) completed: confirmation overlay redesign, quit confirmation when connected, form validation with inline errors, empty-state guidance, search cursor, actions panel collapse, consistent form field display, scroll indicators, Home/End in group overlay, PgUp/PgDn in profiles, inverted log scroll, Ctrl+A select-all/deselect-all, connection indicator in tab bar, update indicator styling, statistics screen refactored into bordered sections. **Log subsystem overhaul** completed: non-blocking TuiLogLayer (std::sync::mpsc channel instead of direct heed writes), background batched heed writer (batch up to 100 messages per transaction, spawn_blocking), MapFull→resize_map with retry (1GB default, doubles up to 8GB, atomic counter instead of tracing events), async heed read wrappers (spawn_blocking for all reads), lazy log loading on first Logs tab access. **Theme system overhaul** completed: integrated ratatui-themes + ratatui-cheese crates, replaced hardcoded Theme Style constants with ThemeStyles + Palette pattern, added palette_bridge, extracted DataTable widget, added mouse support via tui-popup overlays. **Profiles UI redesign** (Phase 12) completed: 17-column single-line endpoint rows (Type, country flag, Address, Feat, config type, Test, Outbound, Country) + expandable rounded per-protocol panel, DNS resolution with cross-launch persistence (endpoints.resolved_as/resolved_at, schema v2 migration), endpoint-scoped ping batches, Last Used column, Remarks wiped from TUI and DB, background enrichment pipeline (ops/enrich.rs) using the geoip/dns/host-features crates. **Test column** (added 2026-08): active protocol's delay `[ 12 ]` colored by threshold (green <500ms, yellow ≥500ms, red ≥1000ms) + red problem labels `[name]` (DNS unresolved), `[fast]`/`[real]` (every protocol of endpoint unreachable for that test, per session-only `ping_status` rounds); placed after the `}=>` arrow, before Outbound's `[`. IP+SNI flag columns merged into `Feat`. Selected sub-row uses reverse highlight (`panel_row_selected`: fg foreground + `bg Color::Reset` + bold — a common-background notch in the surface-highlighted panel; theme-safe, replaces accent bg). Phase 15 (Test-Priority Sorting) completed: live sub-table re-sort on fast/real ping results (tier + latency), `delay_source` provenance column (schema v4), main-table Test column sort by best protocol (decision 16).

### Adding a new protocol form
1. Add config type enum variant and assign core type in `protocol_core_mapping.rs`
2. Create form fields in `crates/xray-tui-config/src/forms.rs` matching that protocol's parameters
3. Create `XxxFmt::parse_share_url` and `XxxFmt::format_share_url` in `crates/xray-tui-config/src/import_export.rs`
4. Add profile validation in appropriate config builder (`xray.rs` or `singbox.rs`)
5. Reference: v2rayN's individual `*Fmt.cs` files for format specs; sing-box's `option/*.go` for JSON config structs

### Adding a new Clash YAML protocol conversion
1. Add Clash struct in `crates/xray-tui-proto/src/clash/mod.rs` (kebab-case serde field names)
2. Add `use crate::clash::{ClashProxy, ClashXxx};` import in the protocol's config file
3. Implement `try_from_clash(&proxy)` — match `ClashProxy::Xxx(c)` arm; convert fields using helpers from `common.rs` (`clash_server_to_host`, `clash_tls_to_security`, `transport_to_clash` for transport, `PortSpec::new_with(c.port)` or `c.port` for port)
4. Implement `to_clash(&self)` — construct `ClashProxy::Xxx(ClashXxx { fields })` using `host_spec_to_string`, `security_to_clash_tls`, `port_spec_first` or `self.port`, `transport_to_clash` for transport
5. Add test with `check_clash_roundtrip::<XxxConfig>()` in the protocol's test module
6. Add dispatch arm in `clash_match!` macro in `mod.rs`

### Adding subscription management features
1. `confirm_add_group()` / `confirm_edit_group()` in `crates/xray-tui/src/ops/settings.rs` handle form submit
2. `update_group_subscriptions()` / `do_update_subscription()` in `crates/xray-tui/src/ops/subscriptions.rs` handle HTTP fetch + parse + DB upsert
3. `subscription_upsert_profiles()` in `crates/xray-tui-db/src/lib.rs` handles content-based dedup via `ON CONFLICT(group_id, sub_uid)`
4. `move_orphans_to_purgatory()` / `purge_purgatory()` handle stale profile cleanup
5. `spawn_auto_update()` runs background check at 60s intervals, comparing SQL datetime() arithmetic
6. `shutdown_token: Arc<AtomicBool>` on `AppState` signals background loop to stop on quit — checked via `token.load()` in each iteration

### Adding log storage features
1. `LogMessage` struct in `crates/xray-tui-core/src/log_heed.rs` — fields: `timestamp_nanos`, `level`, `target`, `message`
2. `HeedLogStorage` in same file — wraps two heed (LMDB) databases: `logs` (u64 BE timestamp → postcard-encoded `LogMessage`) and `targets` (set of seen target strings)
3. Methods: `write_log()` (single entry convenience), `write_log_batch()` (batched transaction with MapFull→resize+retry), `try_write_batch()` (internal single-txn batch with MapFull discrimination), `read_recent()`, `read_newer_than()`, `read_older_than()`, `get_targets()`, `delete_older_than()`, `clear_all()` (empties both logs+targets databases). Async variants: `read_recent_async()`, `read_newer_than_async()`, `read_older_than_async()`, `get_targets_async()`, `clear_all_async()` — each wraps sync call in `spawn_blocking`. All methods accept `self: &Arc<Self>` for async variants (Arc clone for spawn_blocking).
4. **TuiLogLayer** (`main.rs`) uses `std::sync::mpsc::Sender<LogMessage>` (unbounded channel, never blocks under tracing lock). `on_event` sends `LogMessage` via channel; background `spawn_blocking` task receives, batches up to 100 messages, calls `heed.write_log_batch()`. MapFull triggers `unsafe { env.resize(new_size * 2) }` with retry and atomic fail counter.
5. Core process log forwarding (`connect_to_profile` in `lib.rs`) sends to same `std::sync::mpsc::Sender` channel (clone stored in `AppState.log_sender_tx`), not directly to heed.
6. Log tab polls heed via async read wrappers (`read_newer_than_async`, `read_older_than_async`) from event loop (spawn_blocking, non-blocking to TUI render thread).
7. TTL cleanup runs in background tokio task, calls `delete_older_than()` via `spawn_blocking` (infrequent, every 10 min).
8. Initial log loading is lazy — deferred from startup to first `Tab::Logs` activation (`AppState.logs_loaded` flag).
9. DEFAULT_MAP_SIZE is 1 GB (was 256 MB) — see `DEFAULT_MAP_SIZE` constant. Runtime resizing doubles up to 8 GB on MapFull, with backoff retry (50ms*(attempt+1), max 5 attempts); the batch is retried after a successful resize — MapFull no longer drops batches.
10. Error handling: `HeedError::MapFull` variant for discrimination. `mapsize_full_count: AtomicU64` counter tracks MapFull events without emitting `tracing::error!()` (which would re-enter TuiLogLayer).

### Adding log copy and selection features
1. **Key dispatch whitelist in `mod.rs`**: Logs tab keys are whitelisted at line 402. Any new hotkey (`y`, `Y`, `Esc`, etc.) must be added to the `is_logs_key` pattern — otherwise it silently fall through to the main handler.
2. **Cursor highlight**: `log_scroll` (offset-from-bottom, 0=newest) doubles as cursor position. Set `DataTableState.selected` in render to highlight cursor line. No separate state needed.
3. **Selection mode**: Add `log_select_anchor: Option<usize>` to AppState (same offset-from-bottom convention). Shift+Up/Down sets anchor at cursor, then extends range. Selection range computed at render time between anchor and cursor. Viewport pins to show full range when it fits.
4. **Copy to clipboard**: Use `arboard::Clipboard::new()` (already in deps). `y` copies selection (if active) or cursor line. `Y` copies all filtered logs. Format: `{iso-ts} [{level}] [{target}] {message}`.
5. **Anchor stability during log mutations**: Adjust `log_select_anchor` by same delta as `log_scroll` when entries are prepended (`try_load_older`), appended (`poll_new_logs`), or evicted — keeps selection pointing at the same logical entries.
6. **Visual highlight**: Set `selection_style` on DataTable (use `ThemeStyles::table_row_selected`). Populate `DataTableState.multi_selected` with row indices in the selection range.

### Adding batch import for share URLs
1. Parse each URL with `parse_share_url(url, &config.validation)` from `xray_tui_config::import_export`
2. Collect results as `Vec<BatchImportItem>` and set `AppMode::BatchImport { results, scroll }`
3. Render scrollable success/failure list in `profiles.rs` (`render_batch_import()`)
4. Batch import mode handles keys: Up/Down scroll, Enter saves all successful imports via `db.add_profile()`, Esc cancels
5. Reference `crates/xray-tui/src/ops/profiles.rs` `start_batch_import()` method for pattern
6. Refer to `groups.rs` for group management overlay UI patterns (matching `add_server.rs` form conventions)

### Adding a DNS/enrichment feature (country flags, whitelist, outbound IP)
1. Background work goes in `crates/xray-tui/src/ops/enrich.rs` as a `spawn_*` fn: clone needed `Arc`s + `core_event_tx`, `tokio::spawn`, send `CoreEvent::EndpointInfoUpdated` on completion. Never block the UI thread.
2. Merge/persist logic lives in the `EndpointInfoUpdated` arm of `poll_core_events` (`ops/events.rs`) — merge by field group (`resolved_ips` / `sni_whitelisted` / `outbound_ip` / `resolved_at_secs`), persist DNS resolutions via `db.update_endpoint_resolution` only when `resolved_at` changed.
3. Display reads `state.endpoint_info.get(&endpoint_id)` in `build_display_rows` (`ui/profiles.rs`); flags: unknown country `🏴`, whitelisted `🏳️`, DNS-unresolved `🏁`.
4. Every network path needs a hard deadline — `reqwest::Client::builder().timeout(...)` for downloads, `tokio::time::timeout(...)` around lookups. A hang is a bug.
5. DB changes: new columns ride the existing endpoint SELECTs + `deserialize_endpoint_rows`; bump `SCHEMA_VERSION` + add an `ensure_column` call inside the migration transaction in `Database::open`.

### Adding a new speed test type
1. Add variant to `TestType` enum in `crates/xray-tui-core/src/speed_test.rs`
2. Implement async function in `speed_test.rs` using tokio timeouts
3. Add `start_xxx_test()` method in `crates/xray-tui/src/ops/ping.rs` AppState that spawns tokio task calling function
4. Wire result into `CoreEvent::SpeedTestResult` handler in `poll_core_events()` in `crates/xray-tui/src/ops/events.rs`
5. Add menu item in `render_speed_test_menu()` in `crates/xray-tui/src/ui/mod.rs`
6. Add key handler entry in `handle_key()` menu navigation
7. Use `create_socks5_client(proxy, port, socks5h, timeout)` helper from `speed_test.rs` to build reqwest::Client with SOCKS5 proxy — reuses connection pool and avoids per-call construction overhead

### Adding stop-testing support to a batch operation
1. Add `speed_test_stop: Arc<AtomicBool>` to AppState, initialize `Arc::new(AtomicBool::new(false))`
2. Add `stop_speed_test()` method: `self.speed_test_stop.store(true, Ordering::Relaxed)`
3. Clone flag and pass into spawned task. Check at iteration boundaries (page, target, profile) with `stop_flag.load(Ordering::Relaxed)`. Break when true.
4. Track which profile IDs received results via `HashSet<String>` to prevent double-emission of "Cancelled" — only emit for unprocessed profiles.
5. Auto-reset flag when `testing_profiles.is_empty()` in `poll_core_events` handler.
6. Add menu entry in `SPEED_TEST_MENU_ITEMS` with dispatch. Add hotkey `'s'` in key handler.
7. Update status bar to show "■ Stopping..." indicator when flag is active, overriding testing indicators.

### Adding a backend auto-update feature
1. Create functions in `crates/xray-tui-core/src/updater.rs`: `get_current_version` (runs subprocess), `get_latest_version` (GitHub releases API), `download_release` (streaming download to temp dir), `install_binary` (extract to temp → verify → .bak → copy all → remove .bak on success/restore from .bak on failure)
2. Add `UpdateCheckResult` and `UpdateCompleted` variants to `CoreEvent` enum in `crates/xray-tui/src/lib.rs`; handle them in `poll_core_events()` to update `update_status: HashMap<CoreType, BackendUpdateStatus>`
3. Add `spawn_update_check()` and `spawn_update_download()` methods on `AppState` — each spawns tokio task that calls updater functions and sends results back through core event channel
4. Add `UpdateForm` to `SplitRightPane` and `Updates` to `SettingsSection`; wire into `build_right_pane()` with snapshot of current status
5. Create `render_update_form()` and `handle_update_form_key()` in `settings.rs` — C triggers check, D triggers download for all available updates, Esc goes back
6. Add update-available indicator (colored `[Update: ...]`) to `status_bar.rs`
7. Add startup check in `ui::run()` gated by `config.updates.check_on_startup`

### Fixing xray-core config build errors ("this rule has no effective fields")
1. xray-core 26+ rejects routing rules with no match fields (no inboundTag/domain/ip/port/network)
2. `build_multi()` catch-all `{ "type": "field", "outboundTag": "direct" }` has no matchers
3. Fix: remove the catch-all — all traffic arrives through specific socks inbounds with explicit routing rules
4. Single-profile `build()` doesn't add catch-all and works fine

### Determining which core a protocol belongs to
- Reference `thirdparty/sing-box/constant/proxy.go` for sing-box protocol type strings
- Reference `thirdparty/Xray-core/proxy/` directory listing for xray-core protocols
- Update `protocol_core_mapping.rs` with new entry
- Protocols present in both: prefer xray-core (user can override profile core_type to force sing-box)
- See `docs/protocols.md` for protocol specifications and `docs/clash.md` for Clash YAML format docs

### Adding a new gRPC-based feature (stats, logs, routing API)
1. Add/update proto definition in `crates/xray-tui-core/proto/` and re-run build (auto-compiled via build.rs)
2. Add query method to `StatsProvider` trait in `grpc_client.rs`
3. Implement in both `XrayGrpcClient` and `SingBoxGrpcClient`
4. Add `CoreEvent` variant in `crates/xray-tui/src/lib.rs` and handle in `poll_core_events()`
5. For polling: add to select loop in `connect_to_profile()` with appropriate interval
6. For UI: update relevant screen module in `crates/xray-tui/src/ui/`

## Style Guide

- Rust 2024 edition
- Follow clippy with workspace-level lints (`pedantic` + `nursery` at `warn`, config in root `Cargo.toml` under `[workspace.lints.clippy]`, per-crate opt-in via `[lints] workspace = true`)
- Format with `rustfmt` (use `cargo fmt`)
- Tests go next to code in same file (unit) or `tests/` (integration)
- Use `thiserror` for error types
- Use `anyhow` for error propagation where appropriate
- Dependency organization: root `[workspace.dependencies]` holds only deps used by 2+ crates or architecture-driving crates; single-crate deps are declared in the crate's own manifest with a minor-version bound pinned to the latest release (transitive deps named directly: minor bound if upstream allows, else `*`)
- Use `semver` for version parsing and comparison (backend updater)
- Use `serde` for JSON serialization
- Use `tokio` for async runtime
- gRPC via `tonic` crate
- `reqwest` for HTTP client (subscription fetch)
- `toasty` ORM v0.9 with `toasty-driver-turso` for async SQLite
- `tracing` for diagnostic event system (subscriber in bin crate, macros in lib crates)
- `tracing-subscriber` for event filtering, formatting, and TuiLogLayer routing
- `escape8259` for JSON string unescaping
- Use `memchr` for vectorized byte search
- `tokio` for async runtime (also direct dep in xray-tui-db for retry backoff sleep)
- Use `urlencoding` for percent-decoding
- Use `uuid` for system group IDs (`ALL_GROUP_ID`)

## Verification

- `cargo test` — runs all tests
- `cargo clippy` — lint
- `cargo build --release` — release build
- Manual: run xray-tui against real xray-core and sing-box binaries, verify connect/speedtest/disconnect flow for both backends

### Tool Restrictions

- **DO NOT use `ast_edit`** — this tool is forbidden. Use `edit` for all code changes. `ast_edit` previews corrupt session state.
- Use `search` for content search, NOT `grep`/`rg`/`awk` in bash.
- Use `read` for file display, NOT `cat`/`head`/`tail`/`ls` in bash.

## UI Manual Reference

`TUI_MANUAL.md` documents all screens, keyboard shortcuts, and common workflows.
Consult it when implementing UI changes, writing automated TUI tests with `tui-test` tools,
or verifying expected screen content and key behavior.
