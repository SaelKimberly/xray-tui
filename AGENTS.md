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
- `crates/xray-tui/src/lib.rs` — crate root, module wiring, `try_send_or_warn` CoreEvent helper; helpers `profile_to_fields` (typed Protocol+Endpoint → form fields), `flatten_json_to_fields`, `format_ts(&jiff::Timestamp)`, `iso_to_flag`, `parse_core_log_line`. `AppState` (incl. `scheduler: Arc<scheduler::TaskScheduler>`) in `state.rs` with `persist_parsed`/`load_protocol_with_config`; `Tab`, `SortColumn`, `LogLine`, `SettingsMode` (Split variant), `SettingsSection` (incl. SpeedTest), `SplitFocus`, `SplitRightPane`, `CoreEvent` (incl. TestTypeUpdate, HostFeaturesLoaded, EndpointInfoUpdated, SpeedTestResult, BatchProgress), `EndpointInfo`, `AppMode`, `BatchImportItem`, `ConfirmAction` in `types.rs`
- `crates/xray-tui/src/ops/enrich.rs` — background enrichment engine: spawn_dns_resolve (TTL-gated via dns_cache_ttl_secs, `x` force; DNS failures call `scheduler.mark_dns_failure`), spawn_enrich_ip_hosts (startup seed from persisted resolved_as, no network), spawn_whitelist_pass (on HostFeaturesLoaded), spawn_outbound_enrich (real-ping exit IP), extract_sni (typed `Protocol.security.sni` column — covers tls+reality), should_resolve, fill_features, is_resolvable_hostname
- `crates/xray-tui/src/ops/scheduler.rs` — `TaskScheduler`: per-(protocol, endpoint) task gate (`task_id: Option<u16>` + FIFO `task_queue: Vec<u16>` persisted on ProfileStats), all transitions serialized by a tokio Mutex with fresh() re-reads inside the critical section, orphan sweep, DNS-deferral (`dns_failure_defer_secs`), queue limit (`task_queue_limit`, 0 = no queue), `set_limits` on settings save. `ScheduleOutcome` = Started/Queued/QueueFull/DnsDeferred
- `crates/xray-tui-core/src/lib.rs` — core logic facade
- `crates/xray-tui-db/src/lib.rs` — re-export hub; Database, DatabaseError, Result public
- `crates/xray-tui-db/src/error.rs` — DatabaseError, Result, ProfileWithDetails
- `crates/xray-tui-db/src/database.rs` — Database struct + all public query/write methods on the typed toasty models. ZERO raw SQL except PRAGMAs (journal_mode/busy_timeout/foreign_keys) + a minimal `PRAGMA user_version=5` schema tag — `push_schema` is not idempotent, the tag skips re-push on reopen (no migrations/ensure_column machinery; bump the tag on schema changes; any other tag = pre-T8 9-table DB, recreated from scratch). Typed reads with includes: `get_active_endpoints`/`get_stale_endpoints`/`get_active_endpoints_by_group` (endpoint.links() + batched ProfileStats include protocol+endpoint — no N+1), `get_stale_ids` (id-only path), `upsert_link`, `update_scheduler_state` (OCC via #[version] + retry), `update_endpoint_resolution`, `upsert_resolved_ip_children`, `purge_expired`, `delete_group`
- `crates/xray-tui-proto/src/` — Protocol config types (VMess, VLESS, Trojan, Shadowsocks, SOCKS, HTTP, WireGuard, Hysteria2, Hysteria1, TUIC, Naive, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR) with URL parsing/splitting infrastructure + Clash YAML conversion. Parse boundary: `ProtocolConfig::try_parse_proto` → `ParsedProto { endpoints: Vec<EndpointEssentials>, protocol: ProtocolEssentials { proto_kind, config_type, core_type, config: ProtocolConfig } }`. Host-free parse mandate: configs never carry the server host/port — explicit URL host/sni params are stored, builders inject the endpoint host at build time. Identity: `uid = sig ^ cred_hash` computed over protocol essentials ONLY (endpoints excluded); hashing is canonical (`serde_json::to_value` — BTreeMap-sorted); orphan protocols (empty endpoints) are legal. `ProtocolKind` (27 variants, `as_str()` = the old core `Protocol` Display strings) replaces core's `Protocol` enum. `InjectToCoreConf::inject_to(conf, core_type, endpoint, InjectOptions)` per config (xray + sing-box shapes); `core_mapping::resolve_core(kind, override, ss_method)` in `proto_spec/core_mapping.rs`. Adopted from sub-healer project.
- `crates/xray-tui-db/src/models_toasty.rs` — toasty Model definitions for all 7 tables: `Endpoint` (EndpointId key, host_type enum, ports Vec<u16>, resolved_as Vec<String>, jiff timestamps, has_many links+group_links), `Protocol` (ProtocolId = uid = sig ^ cred_hash sans host/port, transport/security embeds, `config: Deferred<Json<ProtocolConfig>>`), `ProfileStats` (composite key (protocol_id, endpoint_id), core_type/config_type/last_used_at/last_seen_at/task_id Option<u16>/task_queue Vec<u16>/latency Option<Latency>/speed_bps/error Option<ErrorInfo>/traffic TrafficStats/auto timestamps/#[version] OCC), `EndpointGroup` (composite key (endpoint_id, group_id)), `Group` (typed enums, bool, jiff — no is_system), `RoutingRule` (Vec<String>/Vec<u16> fields, opaque i32 type), `DnsSetting` (Vec<String>/bool/QueryStrategy); non-model types (EndpointRow)
- `crates/xray-tui-db/src/retry.rs` — `retry_on_busy`/`is_busy_error`: retry-with-backoff (20ms doubling, 1.28s cap) for SQLite write contention (`is_serialization_failure` or "database is locked"); wired into `update_endpoint_resolution` + `update_scheduler_state` so the enrichment herd and the scheduler never drop writes. `Database::conn()` (database.rs) additionally sets `PRAGMA busy_timeout=5000` on EVERY pooled connection acquisition — the pragma in `open()` is per-connection and never reaches pool-created conns (the real cause of the lock-failure dumps); all write paths go through `conn()`
- `crates/xray-tui-core/src/config_builder/mod.rs` — BackendConfig enum, BuildParams, BuildError, ConfigBuilder struct; `protocol_config` (borrows the typed `Deferred<Json<ProtocolConfig>>`, refusing unloaded rows), `shadowsocks_method`, `endpoint_essentials` (db Endpoint → proto EndpointEssentials). Outbound blocks are produced by `protocol.config.inject_to(...)` — per-config xray/sing-box shapes, ss cipher whitelists, reality/cipher build-time validation → `SupportError` → `BuildError::Support`
- `crates/xray-tui-config/src/lib.rs` — config management, module registration
- `crates/xray-tui-core/src/grpc_client.rs` — StatsProvider trait + GrpcStatsClient + MockStatsProvider (test double) + factory (unified from former XrayGrpcClient/SingBoxGrpcClient)
- `crates/xray-tui-core/src/updater.rs` — backend auto-update (version check, download, install) for xray-core and sing-box
- `crates/xray-tui-config/src/import_export.rs` — share URL parse/format (`parse_share_url` → `ParsedProfile { parsed: ParsedProto, validation }`, `format_share_url(parsed, endpoint)`) with per-profile required-field validation and `validate_host`. Parsing itself lives in the proto crate (`ProtocolConfig::try_parse_proto` — scheme dispatch + fallback chain); this module owns the URL-shape gate, validation settings, and host checks. `validate_host` hard-rejects unspecified IPs (`0.0.0.0`/`::`, also bracketed `[::]`) regardless of `allow_private_ips`; policy-gated checks (private/loopback/link-local) stay behind the gate
- `crates/xray-tui-config/src/base64_util.rs` — robust base64 decode with percent-decoding and annotation stripping
- `crates/xray-tui-dns/src/lib.rs` — DnsResolver: DNSCrypt stamp parsing (dns-stamp-parser) → hickory-resolver 0.26 config, cached resolver list, panic-free async OnceCell init
- `crates/xray-tui-geoip/src/lib.rs` — GeoIp: GeoLite2-City mmdb download + country/city lookup (maxminddb 0.30)
- `crates/xray-tui-host-features/src/lib.rs` — HostFeaturesChecker: SNI/exact-IP/CIDR whitelist membership checks (fastbloom fast-negative guard + exact HashSet/interval verification, IPv4-only), download-if-missing from hxehex/russia-mobile-internet-whitelist, `get_host_features(&ServerName)` main API
- `crates/xray-tui-config/src/permissive_json.rs` — lenient JSON parser for vmess:// subscriptions
- `crates/xray-tui-config/src/fast_perc.rs` — hand-rolled UTF-8 + percent-decoding character source
- `crates/xray-tui-config/src/subscription.rs` — chunked base64 streaming decoder with URL splitting
- `crates/xray-tui-core/src/speed_test.rs` — async speed test engine (tcp_ping, real_ping, speed_test, udp_test, udp_ping) using tokio + reqwest SOCKS5 proxy. RealPingResult includes latency + ISP info. Configurable via SpeedTestConfig. Batch entry points live in the TUI crate (`ops/ping.rs`: `start_batch_ping` / `start_batch_then_real_ping`, rebuilt on the TaskScheduler).
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
- `crates/xray-tui-tls/src/` — ring-based TLS 1.3 client engine with browser fingerprint mimicry + REALITY (workspace crypto standard: ring). Module layout: `spec/` (declarative `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec` model + exact RFC 6066/8446 wire encoding, GREASE), `profiles/` (12 browser fingerprints as spec data via the `define_profiles!` macro → `BrowserProfile` enum: Chrome130/119/133, ChromeAndroid130, Edge130, Brave167, Opera114, Firefox, Firefox128ESR, Safari17, SafariIOS17 + generic aliases), `client/` (unified engine entry: `TlsConfig { mode, server_name, alpn, rng }`, `TlsMode::{Plain, Reality}`, one `connect` — plain + REALITY share the `handshake::drive` driver), `hello/` (ClientHello builder + parser), `crypto/` (key schedule, AEAD, X25519 + JA3/JA4 codecs), `record/` (record layer + `TlsStream` AsyncRead/AsyncWrite), `handshake/` (TLS 1.3 client handshake, HRR detection, `ServerVerifier` seam), `verify/` (`WebPkiVerifier`: chain+SAN+`CertificateVerify` signature via rustls-webpki; `insecure`/`pin_sha256` trust modes), `reality/` (`HelloProvisioner` + `ProfileProvisioner(BrowserProfile)` + 9-step wire contract, Spider-X h2 fallback; client keypair on x25519-dalek — ring's `EphemeralPrivateKey` is single-use and can't do REALITY's dual agreement), `http2/` (minimal h2 layer: tls.peet.ws grader + Spider-X fallback GETs). Tier-2 verification: `examples/grader.rs` + ignored `tests/tls_peet_ws.rs`. Purpose, profiles, and verification tiers: `NATIVE_CORE.md`.
- `crates/xray-tui-native/src/` — in-process native proxy core (subprocess-free alternative to spawning xray-core/sing-box). Layering (Xray composition order): dial → security → transport upgrade → protocol → tunnel, folded by `connect_chain` — TLS is OUTERMOST, ws/grpc framing runs INSIDE the engine TLS session (matches xray/sing-box). Transports: `transport/tcp.rs` (dial), `transport/ws.rs` (tokio-tungstenite over the engine stream, v2ray Host/path/headers, write-through), `transport/grpc.rs` (h2 over the engine stream, gun mode, `Hunk` protobuf + 5-byte gRPC prefix, deferred response headers via spawned task, RST(NO_ERROR)=EOF), `transport/httpupgrade.rs` (hyper http1 + RFC 7230 101 upgrade over the engine stream, `Connection: Upgrade`/`Upgrade: websocket` echo validated, ALPN `http/1.1`), `transport/xhttp.rs` (splithttp v3, xray-only server: uuid session in path, GET-body download, raw POST uploads with seq + 30 ms pacing + `Referer` `x_padding`, ≤1 MB chunks; packet-up + stream-up; h1 when no TLS, h2 over TLS), `transport/v2rayhttp.rs` (h2 single full-duplex PUT stream, `:authority` = config host else `www.example.com`; sing-box only — xray removed the h2 transport in 26.x). HTTP framing is hyper 1.11 (`client`+`http1`+`http2`) + hyper-util 0.1.20 (`tokio`) + http-body-util 0.1.5 (`channel`) — we own the byte stream, the dial, and the timeouts; transport-implied ALPN is added by `context.rs::alpn_vec` (`h2` for grpc/xhttp/v2rayhttp, `http/1.1` for ws/httpupgrade; explicit user ALPN wins). Security phase (`security/mod.rs` `wrap()`) is engine-only: it builds an engine `TlsConfig` and runs `xray_tui_tls::client::connect` for both arms — `Tls` → `TlsMode::Plain` (profile from `security/fingerprint.rs`'s 5-id `fp` parser → `BrowserProfile`, `WebPkiVerifier` via `verifier_for`, default Chrome130), `Reality` → `TlsMode::Reality` (`security/reality.rs`: `HelloProvisionerChoice` default `FixedChrome133`, pbk/sid decoders, Spider-X fallback). The rustls client path and the `TlsProvider` plug are gone (rustls = server-side test double only). Protocols: `protocol/vless/` (plain + `xtls-rprx-vision` flow in `protocol/vless/vision.rs`: padded camouflage frames, inner-TLS 1.3 filter, Direct raw-splice state machine driven through the engine stream's per-direction direct mode; TCP only — vision+UDP rejected, no XUDP mux, though the plain UDP command path below exists; guarded to outer TLS/REALITY + raw TCP; inner TLS1.3 → Direct splice, non-1.3 → End, outer TLS continues) + the UDP command path (`protocol/vless/udp.rs` framing codec, `packet.rs` `PacketConn` datagram API + `packetaddr.rs` destination codec — cmd 0x02, `[2B BE len][payload]` framing both directions, packetaddr mode: header dest = magic fqdn `sp.packet-addr.v2fly.arpa`, per-packet frame header `atyp|addr|port`, no magic in the frame) + the v1.mux.cool multiplexer (`protocol/vless/mux.rs`: frame codec `[2B meta_len][metadata][2B data_len][payload]`, `MuxClient` demux/writer/keepalive tasks + `SessionStream`, cmd 0x03 with NO destination bytes in the header, eager New/Keep/End + tunnel KeepAlive, 8 KiB chunks, concurrent TCP sessions over one tunnel; vision+mux composition rides the vision-padded stream, sing-box server only — xray's vision+mux is the XUDP path) + `protocol/vmess/` (AEAD aes-128-gcm / chacha20-poly1305; legacy ciphers rejected); the other protocol modules are `NotImplemented`. E2e (feature `native-e2e`, real cores): vless 66 green + 4 documented ignored (incl. 8 vision rows × both cores + 5 UDP rows + 5 mux rows), vmess 48 green — 114 green + 4 ignored total. Config source of truth is `NativeConnectParams` (wraps xray-tui-proto typed models); e2e harness under feature `native-e2e` in `src/e2e/`. Purpose, protocol roadmap and verification tiers: `NATIVE_CORE.md`.

### TUI screens (crates/xray-tui/src/ui/)
- `mod.rs` — run(), render(), event loop, keyboard handler, tab routing, AppMode dispatch, speed test menu overlay
- `profiles.rs` — profile list DataGrid with connected indicator, Test column (colored delay + `[name]`/`[fast]`/`[real]` labels from the persisted failure marker), Feat flag column, multi-sort indicators (Test column sortable: ranks endpoints by best protocol's test priority), purgatory view filter (`p`: Active/Stale/All), tree markers (▶/▾) for expandable endpoints, column separators; sub-row protocol variants; reverse-highlighted selected sub-row; multi-select, delete confirmation, batch import overlay; expand/collapse (←/→), variant navigation (↑↓), Enter to activate variant
- `add_server.rs` — form rendering, protocol picker, field editing, import URL screen
- `settings.rs` — Settings panel with split-pane tree+form view. Left pane: collapsible tree (SETTINGS_TREE const, SettingsSection-navigated). Right pane: Form, UpdateForm, GroupList, GroupForm, Empty. Sections: Core, GUI, Inbound, Routing, DNS, System Proxy, TUN, Mux, Statistics, Protocol Core, Updates, Speed Test, Logging, Subscriptions (14 total). Tree navigation (arrows) + form focus switching (Ctrl+W). Full rewrite. Replaced per-section SettingsMode variants with unified Split { tree, focus, right }. Group (subscription) management lives here — `g` from Profiles jumps straight to the Subscriptions section (GroupList/GroupForm).
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
2. **Protocol-core auto-resolution**: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → sing-box. All others (VMess, VLESS, etc.) → xray-core by default. Shadowsocks/Shadowsocks-2022 is cipher-aware: AEAD + 2022-blake3 methods → xray-core, legacy methods (`aes-*-cfb`, `aes-*-ctr`, `rc4-md5`, `chacha20-ietf`, `xchacha20`, `none`) → sing-box (xray-core's `CipherType` enum has no legacy entries — `XRAY_SS_METHODS`/`SINGBOX_SS_METHODS` in `proto_spec/core_mapping.rs` are the whitelists). Both config builders reject ciphers neither core supports. User overrides per-profile; a forced core that can't build the cipher fails at build time with a clear error, never an invalid config. **Config validity is enforced at build**: the reality/cipher checks now live in `inject_to` (`xray-tui-proto`, `SupportError` → `BuildError::Support`) — the xray injector rejects `security: "reality"` configs whose reality settings lack `publicKey`/`serverName` (`validate_xray_reality`; xray-core dies at startup with `REALITY: Empty "realitySettings"` or `empty "password"` otherwise) and ciphers xray-core's `CipherType` enum has no entry for, and the sing-box reality block emits only `enabled`/`public_key`/`short_id` (sing-box has no `spider_x` outbound field — the URL's `spx` is dropped, never written into `short_id`).
3. **One core at a time**: Only one backend process runs per connection session. Switching profiles between backends stops current core and starts other. Matches v2rayN.
4. **SQLite via toasty ORM (async)** — Single DB file for all persistent data, 7 typed tables (`models_toasty.rs`). Schema pushed once by toasty's `db.push_schema()` under a `PRAGMA user_version=5` tag — `push_schema` is not idempotent (CREATE TABLE without IF NOT EXISTS), so the tag skips re-push on reopen; it is a tag, not migration machinery (any other tag = pre-T8 9-table DB, recreated from scratch; bump the tag on schema changes). No raw SQL outside PRAGMAs. All DB methods are `async fn` on `Database` struct backed by `toasty::Db`; `Database::open()` accepts impl AsRef<Path>. No system groups (All/Graveyard are view filters — `PurgatoryView` — not rows; `Group` has no `is_system`). Toasty v0.9 with `turso` driver for async SQLite.
5. **Config generation** — Two builders: xray.rs (ports v2rayN's CoreConfigContextBuilder) and singbox.rs (ports sing-box JSON format).
6. **gRPC stats abstraction**: `StatsProvider` trait with unified `GrpcStatsClient` (both backends share the same V2Ray Stats gRPC API).
7. **Sing-box config differs structurally** from xray-core: `type` vs `protocol`, `route` vs `routing`, `experimental.v2ray_api` vs `stats`+`api`+`policy`, different TLS/transport key names.
8. **Theme system**: `ThemeStyles` struct (in `theme.rs`) provides static methods returning `Style` from a `&Palette`. Palette is constructed from `ratatui_themes::ThemeName` -> `ratatui_themes::Theme` -> `palette_bridge::current_palette()`. AppState::current_palette() is canonical accessor. Every screen accepts `&Palette` and calls `ThemeStyles::*` methods instead of hardcoded colors.
9. **Sing-box V2Ray API is experimental**: May require build tag `with_v2ray_api`. If unavailable, stats/logs show "not supported by core".
10. **Typed data model** (the redesign): `endpoints`/`protocols`/`profile_stats`/`endpoint_groups`/`groups`/`routing_rules`/`dns_settings` replace the legacy 9-table schema. The protocol config is a typed `Deferred<Json<ProtocolConfig>>` column on `Protocol` — no raw JSON blob columns, no bridge traits (`ProfileLegacy`/`ProfileMut` gone). Group membership is the `EndpointGroup` (endpoint_id, group_id) many-to-many link; the old `Connection` table is gone. Everything is typed: newtype ids, embedded enums/structs, jiff timestamps, Vec columns. Every read goes through the typed query API.

11. **Identity/uid scheme**: `uid = sig ^ cred_hash`, computed over the serialized protocol essentials ONLY — endpoints (host/port) never influence identity, so one protocol pointed at different servers dedups to a single `Protocol` row (dedup-compatible, orphan protocols with zero endpoints are legal). Hashing is canonical: `ProtocolEssentials` is serialized via `serde_json::to_value` (BTreeMap-sorted maps) then rapidhashed. Form-created profiles and outbound-only kinds (Redirect/TProxy/Mixed, `PlaceholderConfig`) follow the same path; per-pair state lives in `ProfileStats`, so form profiles differ only in `ConfigType::Form`.
12. **Clash YAML conversion**: `ProtoSpec` trait has default `try_from_clash`/`to_clash` methods (both return Err); the parse-boundary forms are `try_from_clash_proto`/`to_clash_proto` (→ `ParsedProto` with the endpoint split out). Per-protocol overrides implement bidirectional conversion between Clash YAML structs and internal ProtocolConfig types. `common.rs` provides conversion helpers for TLS/security, transport, host/port. `dispatch!` macro in `mod.rs` routes by `ClashProxy` variant. Hysteria2 uses `port_spec_serde` (PortSpec); all other protocols use `port_serde` (u16).
13. **Background enrichment pipeline** (`ops/enrich.rs`): DNS resolution, mmdb country lookups, and whitelist checks run in spawned tokio tasks reporting via `CoreEvent::EndpointInfoUpdated`; the UI thread never blocks. `EndpointInfoUpdated` handling merges by field group (concurrent resolution/whitelist/outbound events must not clobber each other); failed DNS lookups (empty IPs) materialize TTL-gated attempt entries so auto-retriggers don't re-hang; `x` (force) always re-attempts. Hosts failing `is_resolvable_hostname` (plugin URLs like `host:port?plugin=...`, underscores from Telegram-channel names, non-ASCII) are skipped before lookup and recorded as failed attempts — hickory's "Label contains invalid characters" spam never fires. Failed lookups also call `scheduler.mark_dns_failure(endpoint)` so the batch pipeline DNS-defers the endpoint for `dns_failure_defer_secs`. DNS persistence is two places: the typed `Endpoint` columns `resolved_as: Vec<String>` + `resolved_at: Option<Timestamp>`, AND one child `Endpoint` per resolved IP (`upsert_resolved_ip_children`, `parent_id` = the DNS endpoint, deterministic `stable_hash(ip, 0)` id, pruned when the IP leaves the set) — country and whitelist features are never persisted (recomputed per launch). Hard deadlines: 10s DNSCrypt-list download + 8s overall lookup timeout (a blocked network must degrade, never hang).
14. **Profiles tab row model**: each endpoint = one variable-height DataTable row (collapsed 1 line; expanded `1 + panel_rows + 4 + 1` — endpoint line + rounded panel (border+IPs+separator+sub-rows+border) + 1 gap line so the panel bottom border never touches the next row). 17 fixed columns; scroll offset is computed in line units (`compute_scroll_offset`: ideal centering clamped to a height-aware `max_offset` and a visibility floor — row-index math strands the last profiles when rows expand). Expansion nav: expand lands on sub-row 0; `↑` at sub 0 → full row; `↓` at last sub-row → next profile; `↓` from full row of an expanded endpoint re-enters sub 0; collapsed endpoints move on one `↓`. Stale `selected_sub` is cleared on every endpoint-nav fallthrough.
15. **Batch ping pipeline** (`ops/ping.rs`, rebuilt on the `TaskScheduler`): Phase 1 schedules one `FastPing` task per link; fast probes dedup by (address, port) — one TCP ping per unique address, the result fanned out to every link sharing it (the old `fast_cache` semantics). **Result events (`SpeedTestResult`/`TestTypeUpdate`) are keyed by `(endpoint_id, protocol_id)` — never protocol alone: a `Protocol` row is shared across endpoints (identity dedup excludes host/port, decision 11), so a protocol-only key makes the handler resolve by first owner and write one endpoint's ping onto another's row (a batch probe that ran on endpoint B updated endpoint A's link, and the per-protocol dedupe guard dropped B's result as a duplicate). Single-ping entry points (`start_tcp_ping`/`start_real_ping`/`start_speed_test`/`start_udp_test`) take the `(endpoint_id, protocol_id)` pair from the menu selection; `state.testing_profiles`/`testing_details` are pair-keyed; `spawn_outbound_enrich` takes the endpoint id directly.** `ScheduleOutcome::DnsDeferred` links are re-scheduled after `dns_failure_defer_secs` in a spawned task; `QueueFull` links are skipped with a `tracing::warn!`. Phase 2 (after every phase-1 task settles) schedules one `RealPing` per link through the kept real-ping path (`CorePool::ping` with the protocol row reloaded WITH config). With `dedup_endpoints` (the `real_ping_test_all_protocols` negation), the first successful real ping on an endpoint retires the remaining links' real tasks via `scheduler.cancel_queued` + `complete` — cancelled tasks never write error markers. The scheduler is the single gate authority: probes run only for ids `schedule`/`complete` hand out, every completion re-reads the link (stale snapshots rejected), and `update_scheduler_state` replaces the whole queue vector with OCC (`#[version]` — Turso has no pop/remove). Batches are serialized (one at a time): the fire-handshake cannot race two batches on the same link, and the shared progress bar displays one batch. In-memory `BatchShared` progress (atomic counters) drives `CoreEvent::BatchProgress`. Sub-table rows are sorted by test priority (decision 16) via `EndpointRow::sort_links_by_test_priority` (db crate) — the single source both the panel and sub-row nav index. Enter on a sub-row pins `manual_protocol_override`; Enter on the endpoint row clears it. **In-memory sync rule**: `endpoints_gen` is write-only and `reload_profiles` runs only on subscription events, so any op mutating endpoint state (`set_protocol_default`/`set_active` in `ops/profiles.rs`) must patch `state.endpoints` rows directly or the UI shows stale data. Selected sub-row renders as an accent bar via `ThemeStyles::panel_row_selected` (fg on_highlight/bg highlight) — the endpoint row highlight (`table_row_selected`) paints the whole expanded panel, so an identical sub-row style would be invisible. Profiles grid has no zebra striping: unselected rows share one background (`table_row_normal`); `table_row_alt` was removed.
16. **Test-priority ordering**: sub-table protocols and the main-table Test sort share one comparator (`EndpointRow::sort_links_by_test_priority` / `best_test_priority_key` in the db crate; ascending key `(tier, latency, -last_seen_at, id)`). Tiers: 0 real-ok, 1 fast/udp-ok, 2 untested, 3 real-err, 4 fast-err, 5 DNS-unresolved; fresh failures dominate stored successes (user decision); latency orders only tiers 0-1 (tiers 2-5 by last_seen_at desc then id). Failure provenance is the persisted `link.error: Option<ErrorInfo>` (`ProfileErr` kind + text) written by the `SpeedTestResult` handler and the batch probe tasks through the scheduler gate — the Profiles Test column renders `[name]`/`[real]`/`[fast]` from the ACTIVE link's `error.kind` (not any link — a failed sibling doesn't paint the row red when the best protocol succeeded; the expanded panel shows per-link markers), with real over fast precedence (the real check is the deeper probe) and DNS-unresolved (`[name]`, tier 5) deepest, beating both — and the label shows only when the active link has NO measured delay (a measurement beats a stale marker). The single-row display preference is the best MEASURED link (`EndpointRow::select_best_measured_link`: real-ok lowest delay, else fast-ok lowest; error markers don't disqualify a measured link), re-selected after every fast/real result and at load — so the row carries the endpoint's best tested delay, exit IP (`Latency::Real.ip`, persisted) and country (mmdb cache per exit IP, seeded at load + on ping; survives reruns). Errors persist across restarts and are swept by the TTL (`error_ttl_hours`; `clear_expired_errors` runs on profile reload and batch finish; empty = never); `Cancelled` results never write markers. Live re-sort happens in the `SpeedTestResult` handler for TcpPing/RealPing results (success or failure) and on `EndpointInfoUpdated` DNS unresolved→resolved flips; `selected_sub` is remapped by protocol id (only when the result's endpoint is the selected one); `filter_cache_valid` invalidated after re-sorts. Main table: `SortColumn::Test` ranks endpoints by best protocol's `(tier, latency, -last_seen_at, id)`; load-time sorting reads the persisted `error`/`latency` columns.
17. **TLS fingerprint engine** (`crates/xray-tui-tls`, replaces the former "M2" forward reference): browser mimicry is declarative — a `ClientHelloSpec` describes the hello at the semantic level, `profiles/` supplies one spec per browser via the `define_profiles!` macro (12 profiles, Chrome130/FF128ESR verified live against tls.peet.ws), and `hello/` encodes it to the exact RFC 6066/8446 wire format. Certificate verification is real WebPKI (`verify::WebPkiVerifier` on rustls-webpki: chain + SAN + `CertificateVerify` signature over the RFC 8446 §4.4.3 transcript; `insecure`/`pin_sha256` trust modes only — never a blanket skip). The REALITY client reuses the same hello machinery (`reality::HelloProvisioner` injects keyshare/SNI/random/sealed SessionId) and builds its dual-agreement X25519 keypair on xray-tui-tls's `crypto::X25519KeyPair` (x25519-dalek runtime dep — ring's `EphemeralPrivateKey` is single-use and cannot do both the REALITY auth ECDH and the TLS keyshare ECDH). `xray-tui-native` security consumes it engine-only: `security::wrap` builds an engine `TlsConfig` and runs `xray_tui_tls::client::connect` (plain + REALITY, Spider-X fallback) — the rustls client path is gone (rustls = server-side test double only).
18. **Transport composition (xray order, hyper-framed upgrades)**: transports are upgrade steps INSIDE the (outermost) security session — ws (tokio-tungstenite), grpc (h2 crate), httpupgrade (RFC 7230 101), xhttp/splithttp and v2rayhttp (hyper) all frame over the engine TLS stream, matching xray/sing-box. HTTP/1.1 + HTTP/2 framing is hyper 1.11 on minimal features (`client`+`http1`+`http2`; default features are empty anyway) + hyper-util 0.1.20 `tokio` (`TokioIo`/`TokioExecutor` bridge — hyper 1.x has no tokio blanket impls; hand-rolling would own unsafe cursor code) + http-body-util 0.1.5 `channel` (upload pipes: stream-up POST, v2rayhttp PUT). Rationale: maintainability over hand-rolled HTTP — hyper owns requests/responses/chunked/101; the native layer keeps ownership of the byte stream, the dial, and the timeouts (every hyper read/write wrapped in `tokio::time::timeout`). grpc.rs deliberately stays on the h2 crate (working, Hunk-codec-verified — same h2 0.4 major as hyper's, no duplicate).

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
1. Add a variant to `ProtocolKind` in `xray-tui-proto/src/proto_spec/kinds.rs` — `as_str()` must return the old Display string (identity hashing serializes it)
2. Implement `try_parse_proto`/`reconstruct_proto` on the typed config in `xray-tui-proto/src/proto_spec/`: URL → `ParsedProto` with the endpoint essentials (host/port) split out — the config payload stays host-free; form fields in `crates/xray-tui-config/src/forms.rs` build the same `ParsedProto`
3. Implement `InjectToCoreConf::inject_to` for both cores (xray + sing-box shapes; host/SNI injected from the endpoint at build time) — replaces per-builder `p_settings`/`s_settings` parsing
4. Assign the core in `core_mapping.rs` via `resolve_core(kind, override, ss_method)` — no legacy blob/encode helpers
5. Reference: v2rayN's individual `*Fmt.cs` files for format specs; sing-box's `option/*.go` for JSON config structs

### Adding a new Clash YAML protocol conversion
1. Add Clash struct in `crates/xray-tui-proto/src/clash/mod.rs` (kebab-case serde field names)
2. Add `use crate::clash::{ClashProxy, ClashXxx};` import in the protocol's config file
3. Implement `try_from_clash(&proxy)` — match `ClashProxy::Xxx(c)` arm; convert fields using helpers from `common.rs` (`clash_server_to_host`, `clash_tls_to_security`, `transport_to_clash` for transport, `PortSpec::new_with(c.port)` or `c.port` for port)
4. Implement `to_clash(&self)` — construct `ClashProxy::Xxx(ClashXxx { fields })` using `host_spec_to_string`, `security_to_clash_tls`, `port_spec_first` or `self.port`, `transport_to_clash` for transport
5. Add test with `check_clash_roundtrip::<XxxConfig>()` in the protocol's test module
6. Add dispatch arm in `clash_match!` macro in `mod.rs`

### Adding subscription management features
1. `confirm_add_group()` / `confirm_edit_group()` in `crates/xray-tui/src/ops/subscriptions.rs` handle form submit (group list/form UI lives in `ui/settings.rs` — `SplitRightPane::GroupList`/`GroupForm`; `g` on Profiles jumps to the Subscriptions section)
2. `update_group_subscriptions()` / `do_update_subscription()` in `crates/xray-tui/src/ops/subscriptions.rs` handle HTTP fetch + parse + upsert via `persist_parsed` (typed `ParsedProto` → endpoint/protocol/profile_stats upserts)
3. Staleness replaces move-orphans-to-purgatory: every upsert refreshes the link's `last_seen_at`; the Stale view (`PurgatoryView`) shows links older than `purgatory_ttl_secs` (7d), and `purge_expired` reclaims them after `purgatory_retention_secs` (30d)
4. `spawn_auto_update()` runs a background check at 60s intervals
5. `shutdown_token: Arc<AtomicBool>` on `AppState` signals background loop to stop on quit — checked via `token.load()` in each iteration

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
4. Batch import mode handles keys: Up/Down scroll, Enter saves all successful imports via `persist_parsed` (`confirm_batch_import` in `ops/profiles.rs`), Esc cancels
5. Reference `crates/xray-tui/src/ops/profiles.rs` `start_batch_import()` method for pattern
6. Refer to `ui/settings.rs` (GroupList/GroupForm) + `ops/subscriptions.rs` for list/form overlay patterns (matching `add_server.rs` form conventions)

### Adding a DNS/enrichment feature (country flags, whitelist, outbound IP)
1. Background work goes in `crates/xray-tui/src/ops/enrich.rs` as a `spawn_*` fn: clone needed `Arc`s + `core_event_tx`, `tokio::spawn`, send `CoreEvent::EndpointInfoUpdated` on completion. Never block the UI thread.
2. Merge/persist logic lives in the `EndpointInfoUpdated` arm of `poll_core_events` (`ops/events.rs`) — merge by field group (`resolved_ips` / `sni_whitelisted` / `outbound_ip` / `resolved_at`), persist DNS resolutions via the typed `db.update_endpoint_resolution(endpoint_id, Vec<String>, Timestamp)` + `db.upsert_resolved_ip_children` only when `resolved_at` changed.
3. DNS failures call `scheduler.mark_dns_failure(endpoint_id)` so the batch pipeline defers the endpoint for `dns_failure_defer_secs` (`ops/enrich.rs` → `ops/scheduler.rs`).
4. Display reads `state.endpoint_info.get(&endpoint_id)` in `build_display_rows` (`ui/profiles.rs`); flags: unknown country `🏴`, whitelisted `🏳️`, DNS-unresolved `🏁`.
5. Every network path needs a hard deadline — `reqwest::Client::builder().timeout(...)` for downloads, `tokio::time::timeout(...)` around lookups. A hang is a bug.
6. DB changes: add typed columns to the model (`models_toasty.rs`) and bump the `PRAGMA user_version` schema tag in `Database::open` (the tag skips re-push; there is no migration machinery).

### Adding a new speed test type
1. Add variant to `TestType` enum in `crates/xray-tui-core/src/speed_test.rs`
2. Implement async function in `speed_test.rs` using tokio timeouts
3. Add `start_xxx_test()` method in `crates/xray-tui/src/ops/ping.rs` AppState that spawns tokio task calling function
4. Wire result into `CoreEvent::SpeedTestResult` handler in `poll_core_events()` (`ops/events.rs`): success writes the typed `ProfileStats` latency/speed via `db.upsert_link`, failures persist `link.error` (`ProfileErr` kind via `err_kind_for(test_type)`) — the scheduler gate owns task ids on the per-link ProfileStats rows (no separate session table)
5. Add menu item in `render_speed_test_menu()` in `crates/xray-tui/src/ui/mod.rs`
6. Add key handler entry in `handle_key()` menu navigation
7. Use `create_socks5_client(proxy, port, socks5h, timeout)` helper from `speed_test.rs` to build reqwest::Client with SOCKS5 proxy — reuses connection pool and avoids per-call construction overhead

### Adding stop-testing support to a batch operation
1. Add `speed_test_stop: Arc<AtomicBool>` to AppState, initialize `Arc::new(AtomicBool::new(false))`
2. Add `stop_speed_test()` method: `self.speed_test_stop.store(true, Ordering::Relaxed)`
3. Clone flag and pass into spawned task. Check at iteration boundaries (page, target, profile) with `stop_flag.load(Ordering::Relaxed)`. Break when true.
4. Track which profile IDs received results via `testing_profiles: HashSet<i64>` to prevent double-emission of "Cancelled" — only emit for unprocessed profiles.
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

### Fixing xray-core config build errors (reality/cipher validation)
1. Reality and cipher validation happens at build time inside `inject_to` (`xray-tui-proto`), surfaced as `SupportError` → `BuildError::Support` — a bad config is refused, never written to disk
2. A `security: "reality"` config without `publicKey`/`serverName` is rejected (`validate_xray_reality`) — xray-core dies at startup with `REALITY: Empty "realitySettings"` / `empty "password"` otherwise
3. Shadowsocks ciphers are whitelisted per core (`XRAY_SS_METHODS`/`SINGBOX_SS_METHODS` in `core_mapping.rs`); a forced core that can't build the cipher fails with a clear error
4. Routing-rule lesson kept: xray-core 26+ rejects rules with no match fields — `build_multi()` must not emit a bare catch-all

### Determining which core a protocol belongs to
- Single entry point: `core_mapping::resolve_core(kind, profile_override, ss_method)` in `xray-tui-proto/src/proto_spec/core_mapping.rs` — returns `CoreType::Xray`/`SingBox` (never Auto); per-profile overrides and the cipher-aware SS method argument are handled there
- Reference `thirdparty/sing-box/constant/proxy.go` for sing-box protocol type strings
- Reference `thirdparty/Xray-core/proxy/` directory listing for xray-core protocols
- Protocols present in both: prefer xray-core (user can override the per-pair core_type to force sing-box)
- See `docs/protocols.md` for protocol specifications and `docs/clash.md` for Clash YAML format docs

### Adding a new gRPC-based feature (stats, logs, routing API)
1. Add/update proto definition in `crates/xray-tui-core/proto/` and re-run build (auto-compiled via build.rs)
2. Add query method to `StatsProvider` trait in `grpc_client.rs`
3. Implement in `GrpcStatsClient` (unified — the former `XrayGrpcClient`/`SingBoxGrpcClient` are one client + factory)
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

## Native Core Reference

`NATIVE_CORE.md` documents the purpose, principles, and verification tiers of
`xray-tui-native` (in-process proxy core) and `xray-tui-tls` (TLS fingerprint +
REALITY engine), plus the per-protocol roadmap with mainstream-repo capability
comparison. Consult it before touching either crate's protocol/security/transport
modules or planning protocol work.
