# xray-tui

TUI client for Xray-core and Sing-box, written in Rust.
Feature parity with v2rayN (C# desktop GUI) for all protocols supported by either core.

## Project structure

```
crates/
├── xray-tui/          # Binary: ratatui event loop + all screens
│   └── src/            # state.rs (AppState), types.rs, ops/ (7 modules), ui/
├── xray-tui-core/     # Library: process mgmt, gRPC, config builders, ping, updater, log_heed
├── xray-tui-db/       # Library: toasty ORM persistence layer + Model definitions
├── xray-tui-proto/    # Library: protocol config types, URL parse/format, Clash YAML (20 protocols)
├── xray-tui-config/   # Library: import/export parsers, form fields, AppConfig management
├── xray-tui-dns/      # Library: secure DNS resolution via DNSCrypt stamps (DOH/DOT/DOQ)
├── xray-tui-geoip/    # Library: country/city lookup by IP (GeoLite2-City mmdb)
├── xray-tui-host-features/ # Library: SNI/exact-IP/CIDR whitelist membership checks (fastbloom)
	├── xray-tui-native/   # Library: in-process proxy core (subprocess-free tunnels; VLESS+VMess, TLS engine-backed)
	├── xray-tui-tls/      # Library: ring-based TLS client (1.3 + 1.2 fallback), browser fingerprint mimicry, REALITY client
	├── xray-tui-route/    # Library: first-match routing engine + TLS/HTTP/QUIC sniffer (native inbound)
	├── xray-tui-hakari/   # Generated feature-unification crate (cargo-hakari); every member depends on it
	thirdparty/
├── sing-box/          # Source of truth for sing-box protocols, config format, and API
├── v2rayN/            # Source of truth for UI/UX feature set
├── shoes/             # Rust proxy - possible protocol reference
├── leaf/              # Rust proxy framework
└── shadowsocks-rust/  # Shadowsocks protocol in Rust
ROADMAP.md
CONTEXT.md
ARCHITECTURE.md
NATIVE_CORE.md
TUI_MANUAL.md
AGENTS.md
```

:- **LogMessage** | Persisted tracing event — stored in heed (LMDB) with fields `timestamp_nanos`, `level`, `target`, `message`. Written by `TuiLogLayer` via non-blocking `std::sync::mpsc::Sender` channel, batched (up to 100) by background `spawn_blocking` writer, stored via `HeedLogStorage` (two LMDB databases: `logs` + `targets`). Async read wrappers (`read_recent_async`, `read_newer_than_async`, etc.) wrap LMDB reads in `spawn_blocking` for use from async context. No SQLite table, no separate DB connection. |

## Key reference repositories

- **Xray-core** (`thirdparty/Xray-core/`): Go primary proxy core. Run as subprocess, controlled via JSON config files and gRPC API. Handles VMess, VLESS, Shadowsocks, SOCKS, HTTP, Trojan, WireGuard, Hysteria2, and other native protocols.
- **sing-box** (`thirdparty/sing-box/`): Go universal proxy platform, secondary backend. Covers protocols Xray-core does not natively support: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, Redirect, and more. Config format differs from Xray-core. Provides experimental V2Ray API and Clash API compatibility.
- **v2rayN** (`thirdparty/v2rayN/`): C# desktop client. Feature reference for all UI/UX decisions. Protocol support, settings panels, and workflow patterns derive from this project.
- **shoes** (`thirdparty/shoes/`): Rust proxy server with VLESS/Vision/VMess/Shadowsocks/Trojan/Hysteria2/TUIC v5/AnyTLS/NaiveProxy support. Reference for protocol implementation patterns.
- **leaf** (`thirdparty/leaf/`): Rust proxy framework library. Embeddable proxy client implementation reference.
- **shadowsocks-rust** (`thirdparty/shadowsocks-rust/`): Mature Shadowsocks implementation in Rust. Protocol dependency candidate.

## Domain glossary

| Term | Meaning |
|---|---|
| **Inbound** | Local listening port/protocol that accepts traffic (SOCKS, HTTP, etc.) |
| **Outbound** | Remote proxy server connection (VMess, VLESS, Shadowsocks, etc.) |
| **Subscription group** | Named group of proxy profiles fetched from a subscription URL |
| **ALL_GROUP_ID** | Fixed UUID (`00000000-0000-0000-0000-000000000000`) identifying system "All" group for showing every profile across groups |
| **sub_uid** | Content-based hash (rapidhash) of profile identity fields — used for dedup during subscription update. URL-imported: `sig ^ cred_hash`, computed by the `Proto` identity container (proto crate) from the protocol's `ProtoIdentity` implementation. Form-created: random i64. |
| **Purgatory** | Destination group for orphaned subscription profiles (was "Graveyard"). Default TTL 7 days, retention 30 days. |
| **PurgatoryView** | Enum controlling which set of profiles to display: None (active only), Graveyard, Purgatory. |
| **endpoints_gen** | Generation counter bumped on endpoint mutations — write-only, no reader. In-memory `state.endpoints` is rebuilt only by `reload_profiles()` (subscription events); ops that mutate endpoint state (e.g. protocol override) must patch the in-memory row explicitly or the UI shows stale data. |
| **batch_progress** | `Arc<(AtomicU16, AtomicU16)>` shared between ping tasks and status bar — tracks (completed, total) for batch speed tests. |
| **testing_details** | `HashMap<uuid::Uuid, TestType>` tracking active test type per profile — enables TestTypeUpdate event to switch displayed emoji mid-flow (TcpPing→RealPing) during batch-then-real-ping |
| **ping_status** | `HashMap<endpoint_id, EndpointPingStatus>` — session-only per-endpoint ping rounds (`PingRound { seen, failed }` protocol-id sets per test type: fast/real). Updated in the `SpeedTestResult` handler, reset at batch start. Drives the Test column's red `[fast]`/`[real]` labels — fires only when every protocol of the endpoint failed that type in the current round — and the failure tiers of the test-priority sort. |
| **delay_source** | `profile_extensions` column persisting a latency's provenance: 0 = fast/TCP, 1 = real, 2 = udp; NULL/-1 = no measurement (legacy). Written by the `SpeedTestResult` handler (from `test_type`) and `batch_upsert_buffer` (from session `ping_type`); lets the test-priority sort distinguish real-ok (tier 0) from fast-ok (tier 1) across restarts. Only `Some(1)` counts as real; a stored success with any other source ranks fast. |
| **Fast Ping** | Transport-level latency test using TCP handshake (TcpPingAdapter), UDP datagram (UdpPingAdapter), or QUIC handshake (QuicPingAdapter). Dispatched by FastPingManager based on protocol support. |
| **DnsResolver** | Per-instance resolver (xray-tui-dns): resolves hostnames through the DNSCrypt public resolver list (DOH/DOT/DOQ stamps), with on-disk resolver cache in a caller-supplied directory. IP literals short-circuit. |
| **Location** | GeoIP lookup result (xray-tui-geoip): country ISO code + optional English city name, from a GeoLite2-City mmdb downloaded on first use. |
| **HostFeatures** | Whitelist-membership result (xray-tui-host-features): `sni_whitelisted` / `ip_whitelisted` / `cidr_whitelisted` flags for a `ServerName` (DnsName → SNI check, IPv4 → exact-IP + CIDR checks, IPv6 → empty). Whitelists are IPv4-only, sourced from hxehex/russia-mobile-internet-whitelist. |
| **PingSession** | SQLite record tracking a single ping attempt: batch_id, profile_id, adapter_type (TCP/UDP/QUIC), latency_ms, ip_info, error. Enables persistent history and async progress tracking via AtomicU16 counters. |
| **EndpointInfo** | In-memory enrichment cache (`AppState.endpoint_info`, keyed by endpoint id): resolved IPs, inbound country (mmdb), ip/cidr + SNI whitelist flags, outbound (egress) IP + country, `resolved_at_secs` (None = IP host, never re-resolves). Updated by background tasks via `CoreEvent::EndpointInfoUpdated`; survives `reload_profiles`. |
| **LinkContext** | Per-link policy decision surface in the native core (`xray-tui-native/src/context.rs`): one `LinkContext` per link in a chain carries the typed `NativeConnectParams` + target and answers the connection-policy questions every phase consumes — `is_tls()` (the single TLS/REALITY predicate; collapses the historical `sec.is_empty()` + two byte-identical `has_tls()` copies), `server_name()` (unified SNI: plain `TlsOpts.sni` or the REALITY steal target `RealityOpts.sni`, else endpoint host — collapses the historical divergent `ctx.sni()`/`sec.sni()` paths), `alpn_tokens()`/`alpn_vec()` (canonical ALPN split via `split_alpn`, shared with the xhttp transport's `http_version` decision), `curve_ids()`. Phases read their policy from the context, never re-derive it. |
| **Transport** | Network layer used for outbound connections (TCP, WebSocket, gRPC, QUIC, etc.) |
| **Stream Security** | TLS/REALITY/None wrapper around transport |
| **Config Type** | Proxy protocol (VMess, VLESS, Shadowsocks, Trojan, TUIC, etc.) |
| **Routing** | Rule-based traffic distribution: domain/IP match → specific outbound or balancer |
| **REALITY** | XTLS's TLS resemblance mechanism — makes proxy traffic look like normal web traffic |
| **uTLS fingerprint** | Browser TLS fingerprint emulation (Chrome, Firefox, Safari, etc.) |

## Key design decisions

- **Normalized schema**: Profiles stored as `Endpoint` (server config, dedup key `sub_uid`) + `ProtocolRow` (protocol fields) + `Connection` (M:N Endpoint↔Group). Query pattern JOINs all three tables. `ALL_GROUP_ID` system group shows all endpoints across groups. Purgatory replaces old Graveyard with configurable TTL/retention.
- **Profile identity**: `sub_uid = sig ^ cred_hash` for URL-imported (deterministic dedup via rapidhash), random i64 for manual form entries. Form profiles set `sig = uid`, `cred_hash = 0` (meaningless, no URL-dedup). Identity computation is normalized in the proto crate: crate-private `ProtoIdentity` trait (`compute_sig` + `compute_cred_hash`) feeds the `Proto` container's `OnceLock<Identity>` cache; `ProtoSpec: ProtoIdentity` is sealed.
- **Dual-backend architecture**: `CoreManager` abstracts over xray-core and sing-box subprocesses. Each profile tagged with core type (auto, xray, sing-box). Auto mode resolves based on protocol.
- **Protocol-core auto-resolution**: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → sing-box. All others (VMess, VLESS, etc.) → xray-core by default. Shadowsocks/Shadowsocks-2022 is cipher-aware (`XRAY_SS_METHODS`/`SINGBOX_SS_METHODS` in `protocol_core_mapping.rs`): AEAD + 2022-blake3 methods → xray-core; legacy methods (`aes-*-cfb`, `aes-*-ctr`, `rc4-md5`, `chacha20-ietf`, `xchacha20`, `none`) → sing-box, because xray-core's `CipherType` enum has no legacy entries. Both config builders reject ciphers neither core supports (clear `BuildError` instead of a core that dies on startup with "unknown cipher method"). User can override per-profile; a forced core that can't build the cipher fails at build time.
- **Protocol sub-table ordering (test priority)**: Each endpoint's `protocols` are sorted by test priority in `deserialize_endpoint_rows` via `EndpointRow::sort_protocols_by_test_priority` (db crate; rounds = None at load): real-ok (0) > fast/udp-ok (1) > untested (2) > real-err (3) > fast-err (4) > DNS-unresolved (5); latency orders only tiers 0-1, tiers 2-5 by `last_seen_at` desc then id. Fresh failures dominate stored successes; failure sets come from session-only `ping_status` rounds, so after a reload the tier falls back to persisted `delay` + `delay_source`. The same comparator re-sorts live in the `SpeedTestResult` handler (TcpPing/RealPing results, success or failure; `selected_sub` remapped by protocol id) and on `EndpointInfoUpdated` DNS flips. `active_protocol()` = `manual_protocol_override` if set, else `protocols[0]` — unpinned default is therefore the best-ranked variant. Pin/unpin via Enter on a sub-row / endpoint row (`set_protocol_default`/`set_active` in `ops/profiles.rs`), which write the override to DB AND patch the in-memory row (see `endpoints_gen` glossary entry — no reload happens otherwise).
- **Background enrichment pipeline** (`ops/enrich.rs`): DNS resolution, mmdb country lookups, and whitelist checks run in spawned tokio tasks that report via `CoreEvent::EndpointInfoUpdated` — the UI thread never blocks. `EndpointInfoUpdated` merges by field group (concurrent resolution/whitelist/outbound events must not clobber); failed DNS lookups materialize TTL-gated entries so auto-retriggers don't re-hang; DNS resolutions persist to `endpoints.resolved_as`/`resolved_at`. DNS + mmdb downloads have hard deadlines (10s/8s).
- **One core runs at a time**: Switching profiles between backends stops current core process and starts other. Matches v2rayN behavior and avoids port conflicts.
- **Xray-core runs as subprocess**; TUI writes JSON config files and communicates via gRPC API.
- **Sing-box runs as subprocess**; TUI writes JSON config files and communicates via sing-box's experimental V2Ray API (gRPC compatible).
- **TUI framework**: **Ratatui + Crossterm** (async via tokio).
- **Storage**: **LMDB** via `heed` for log persistence; **SQLite** via `toasty` ORM v0.10 (turso driver) for profiles, subscriptions, routing, DNS, stats.
- **Fast Ping architecture**: Transport-level latency tests via FastPingManager with adapter pattern. TcpPingAdapter (handshake), UdpPingAdapter (datagram), QuicPingAdapter (handshake). Protocols map to adapters automatically; unsupported protocols fall through to RealPingManager. PingSession table persists results with batch_id correlation.
- **Batch ping pipeline** (`ops/ping.rs`): Phase 1 pings a page concurrently (`run_page_pings`: one TCP ping per unique address:port, `buffer_unordered` capped by `fast_ping_concurrency` default 200, owner/follower dedup + cross-page `fast_cache`). Fast ping probes inbound reachability only, so results are all-or-none per endpoint. Phase 2 real pings dispatch wave-by-wave: wave N = Nth protocol per endpoint (stable occurrence ranks over ALL real sessions via `get_batch_for_real_ping(batch_id, wave, limit, dedup_endpoints)`), one core per core-type group with multi-inbound config, stack-based halving retry on group-level failures, parallel per-port SOCKS5 readiness (dead port = item failure). After one protocol of an endpoint succeeds, siblings are skipped (`real_ping_test_all_protocols=false`, default) or still tested (true); `cancel_stranded_real_pings` is gated on dedup so endpoint-scoped batches (dedup=false) always test every protocol. The `[fast]`/`[real]` red labels fire only when every protocol of the endpoint failed that round; `Cancelled` errors are not failures.
- **Log storage**: `TuiLogLayer` sends tracing events to log channel (non-blocking, `std::sync::mpsc::Sender`), consumed by background `spawn_blocking` batched writer. Uses `heed` (embedded LMDB) in `xray-tui-core::log_heed` with two databases: `logs` (u64 BE → postcard-encoded `LogMessage`) and `targets` (seen target string set). MapFull triggers automatic resize (1 GB default, doubles up to 8 GB) with backoff retry (50ms*(attempt+1), max 5) — the batch is retried after a successful resize, never dropped. Async wrappers (`read_recent_async`, `read_newer_than_async`, `read_older_than_async`, `get_targets_async`) wrap LMDB reads in `spawn_blocking` for non-blocking async calls from TUI event loop. Initial log loading is lazy (deferred to first Logs tab access).
- **Theme system**: `ThemeStyles` (in `theme.rs`) replaces hardcoded `Style` constants with static methods taking `&Palette`. `Palette` comes from `ratatui_themes::ThemeName` → `Theme` → `palette_bridge::current_palette()`. Every screen uses `state.current_palette()` and `ThemeStyles::*` instead of bare `Color` values. New dependencies: `ratatui-cheese` (form widgets, `Palette`), `ratatui-themes` (theme definitions), `tui-popup` (overlays), `tui-scrollbar`. New modules: `palette_bridge`, `widgets/` (reusable `DataTable`).
- **Protocols in scope**: Everything supported natively by either Xray-core or Sing-box. No third binary backends.
- **Native in-process core** (`xray-tui-native`): client-side protocol implementations that remove the subprocess dependency for the tunnel itself. Xray composition order (`dial → transport → security → protocol → tunnel`); config source of truth is `xray-tui-proto` typed models (`NativeConnectParams`). VLESS, VMess, Trojan + Hysteria2 implemented; every other kind has a dispatch arm returning `NativeError::NotImplemented` — no silent fallback. Feature `native-e2e` runs 14 real-core cases (trojan {tcp,ws,grpc} x {tls-standard,chrome-fp} x both cores + hysteria2 {tls-plain,obfs-salamander} x sing-box) against real xray-core 26.3.27 + sing-box 1.13.16 servers (version-pinned, hard-fail on mismatch). See `NATIVE_CORE.md`.
- **TLS fingerprint engine** (`xray-tui-tls`): ring-only TLS client (TLS 1.3, plus a TLS 1.2 ECDHE+AEAD path on a 1.2 ServerHello; no aws-lc-rs/rand/unsafe) with browser ClientHello mimicry (two-tier profile roster, GREASE pairing, 512-byte record padding) + a REALITY client (x25519-dalek for the dual agreement — the one documented crypto exception; REALITY stays 1.3-only). JA3/JA4 encoders; `WebPkiVerifier` with `insecure`/`pin_sha256` trust modes (pin replaces chain walk but never the CertificateVerify signature). Engine-only client path — every TLS/REALITY connect runs through `xray_tui_tls::client::connect` (the `TlsProvider` plug is gone; the `fp` profile selects the fingerprint, `reality` config selects `TlsMode::Reality`).
- **Verification tiers**: tier 1 offline unit/integration (RFC 8448 vectors, JA3/JA4 goldens, Go VMess vectors, rustls-server interop, multi-record reassembly); tier 2 live tls.peet.ws grader (`examples/grader.rs` + `#[ignore]`d test); tier 3 real-core e2e (feature + `XRAY_TUI_CORE_BIN_DIR`). No implicit network in the usual test run.

## Key source files

- `crates/xray-tui/src/state.rs` — AppState struct (60+ fields)
- `crates/xray-tui/src/types.rs` — Tab, SortColumn, SettingsSection (14 variants), AppMode, CoreEvent (15 variants), EndpointInfo, PingRound/EndpointPingStatus, LogLine, BackendUpdateStatus, ClashTraffic
- `crates/xray-tui/src/ops/` — extracted AppState methods: connect.rs, ping.rs, events.rs, subscriptions.rs, updates.rs, settings.rs, profiles.rs, enrich.rs
- `crates/xray-tui/src/ops/enrich.rs` — background enrichment: spawn_dns_resolve (TTL-gated), spawn_enrich_ip_hosts, spawn_whitelist_pass, spawn_outbound_enrich, extract_sni, protocol_row_to_profile
- `crates/xray-tui-db/src/models_toasty.rs` — toasty Model definitions for all 9 tables (Endpoint with resolved_as/resolved_at, ProtocolRow with last_used_at/last_seen_at, EndpointGroup, Group, ProfileExtension, ServerStat, RoutingRule, DnsSetting with cache_ttl_secs, PingSession), EndpointRow, PurgatoryView constants
- `crates/xray-tui-db/src/retry.rs` — `retry_on_busy`/`is_busy_error`: SQLite write-contention retry (backoff 20ms doubling, 1.28s cap), wired into `update_endpoint_resolution` + `batch_flush_ping_buffer`
- `crates/xray-tui-proto/src/proto_spec/mod.rs` — ProtocolConfig enum, `Proto` identity container (OnceLock<Identity> cache), sealed `ProtoIdentity`/`ProtoSpec` traits, dispatch macro
- `crates/xray-tui/src/ui/groups.rs` — system-group-aware management (is_system guard, clear action)
- `crates/xray-tui/src/ui/actions_log.rs` — live event log panel
- `crates/xray-tui-core/src/log_heed.rs` — HeedLogStorage (LMDB), LogMessage struct
- `crates/xray-tui-core/src/ping/mod.rs` — FastPingAdapter trait + FastPingManager
- `crates/xray-tui-core/src/ping/real/mod.rs` — RealPingManager
- `crates/xray-tui-core/src/ping/real/pool.rs` — CorePool: single warm core for single-ping reuse, atomic port allocation, SIGHUP/stop-restart reload
- `crates/xray-tui-core/src/config_builder/clash_mixin.rs` — Clash YAML overlay
- `crates/xray-tui-core/src/bin_manager.rs` — binary discovery and archive extraction
- `crates/xray-tui-dns/src/lib.rs` — DnsResolver: DNSCrypt stamp parsing → hickory-resolver 0.26 config, cached resolver list, panic-free async init
- `crates/xray-tui-geoip/src/lib.rs` — GeoIp: GeoLite2-City mmdb download + country/city lookup
- `crates/xray-tui-host-features/src/lib.rs` — HostFeaturesChecker: SNI/exact-IP/CIDR whitelist checks, download-if-missing from hxehex/russia-mobile-internet-whitelist
- `crates/xray-tui-native/src/` — in-process proxy core: `chain.rs` (connect_chain), `context.rs` (LinkContext/NativeConnectParams), `security/` (wrap → `xray_tui_tls::client::connect`; fingerprint.rs, reality.rs), `inbound/` (SOCKS5 listener → `xray-tui-route` `Engine` → Direct/Block/Proxy outbound), `protocol/` (vless/vmess/trojan/hysteria2 implemented; 16 kinds NotImplemented), `e2e/` (feature-gated real-core harness). Roadmap: `NATIVE_CORE.md`
- `crates/xray-tui-tls/src/` — ring TLS client (1.3 + 1.2 fallback): `spec/` + `profiles/` (two-tier roster), `hello/` (builder + parser), `crypto/` (key schedule + TLS 1.2 key block + JA3/JA4), `record/` (TlsStream + 1.2 record protection), `handshake/` + `verify/` (WebPkiVerifier), `reality/` (HelloProvisioner + 9-step wire contract), `http2/` (grader-minimal)

## Key Differences from v2rayN

### Storage & Entity Model

| Aspect | v2rayN | xray-tui |
|--------|--------|----------|
| Profile storage | Flat `profile` table with all fields inline | Normalized: Endpoint (server config) + ProtocolRow (protocol fields) + Connection (M:N to groups) |
| Profile identity | Auto-increment PK | `sub_uid = sig ^ cred_hash` (URL-imported, deterministic dedup); random i64 (form entries, no URL-dedup) |
| Group membership | `Profile.group_id` FK column | `Connection` table: M:N Endpoint↔Group |
| System groups | "All" + "Graveyard" hardcoded | ALL_GROUP_ID UUID + purgatory system with configurable TTL/retention |
| Subscription dedup | By URL + name heuristic | By `ON CONFLICT(group_id, sub_uid)` — content-hash based |
| Log storage | In-memory (recent) + optional file log | LMDB persistent via heed, postcard encoding, auto-resizing, async wrappers |
| Statistics | SQLite `profile_statistic` table (today/total up/down) | SQLite `server_stats` table — same pattern with i64 PK |
| Ping history | Transient (not persisted) | PingSession table: batch_id, endpoint_id, adapter_type, latency, ip_info, error |
| Settings persistence | XML AppConfig file | JSON AppConfig file + SQLite for Routing/DNS |
| Config generation | C# objects to JSON serialization | config_builder/xray.rs + singbox.rs producing serde_json::Value to file |
| Binary discovery | Checked process compatibility | bin_manager.rs: searches bin_dir, PATH, extracts archives |

### Display & UX

| Aspect | v2rayN | xray-tui |
|--------|--------|----------|
| Display | Desktop GUI (WinForms), multi-window | Terminal TUI (ratatui), single-window tabbed |
| Profile view | Flat list with sortable columns | 17-column endpoint rows (Type, country flag, Address, Feat flags, config type, Test delay, Outbound) with expandable rounded panel containing the per-protocol sub-table (sorted by test priority — fastest successful protocol on top; Enter on a sub-row pins that protocol as the endpoint default) |
| Group view | Dropdown filter | Modal overlay (g key) + Settings section |
| Settings | Menu-driven dialog boxes | Split-pane: collapsible tree + inline form/routing list |
| Search | Search box | `/` key focus to inline filter with cursor |
| Multi-select | Checkbox column + Ctrl+click | Space toggle + Ctrl+A select-all |
| Connection status | Icon in system tray | Status bar + connected column + tab bar indicator |
| Speed test | Popup progress dialog | Overlay menu + status bar batch progress + live column updates |
| Sorting | Click column header | `o` key cycles through 8 sort columns; Test column ranks endpoints by best protocol's test priority (tier + latency) |
| Routing rules | Separate dialog with up/down | Inline sortable list within Settings split-pane |
| Logs | Separate window, per-core file viewer | In-TUI tab with source filtering (core/TUI toggle), LMDB persistence |
| Help | Menu bar About dialog | `?` overlay modal with context-sensitive shortcuts |
| Drag-drop reorder | Drag column handle | Ctrl+up/down keyboard reorder |

### Feature Parity

**Implemented**: All 20 protocol config types, URL parsing for 14 protocols, Clash YAML conversion for 17 protocols, dual-backend (xray + sing-box), gRPC stats, subscription lifecycle, speed test system (TCP/real/speed/UDP/batch), routing rules CRUD, DNS settings, system proxy (HTTP_PROXY env vars), TUN toggle, theme system, inline form validation, backend auto-updates, geo file updates, Mux config, TLS/REALITY/ECH/Fragment form fields, Clash Mixin. Profiles redesign: 17-column single-line rows + expandable per-protocol panel, DNS endpoint IP resolution with cross-launch persistence, inbound/outbound country flags (geoip), Feat whitelist flag column (IP/SNI, host-features), Test column with colored delay + `[name]`/`[fast]`/`[real]` labels, test-priority sorting (live sub-table re-sort + Test column sort by best protocol), endpoint-scoped ping batches, Last Used column, reverse sub-row highlight, `x` resolve key.

**Not implemented**: proxy chain/policy groups, process-based routing, multi-URL subscriptions, SIP008 format, Clash API proxy/connections TUI tabs, PAC mode, WebDAV backup/restore, full TUN config (stack, MTU, route exclude), global hotkeys, i18n system, QR code display, config template editor, auto startup, sudo/polkit integration, drag-drop sort, tray integration.
