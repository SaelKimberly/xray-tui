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
:- `crates/xray-tui/src/lib.rs` — AppState, Tab, SortColumn, ProfileRow, LogLine, SettingsMode (Split variant), SettingsSection (incl. SpeedTest), SplitFocus, SplitRightPane, CoreEvent (incl. TestTypeUpdate), testing_details map
- `crates/xray-tui-core/src/lib.rs` — core logic facade
:- `crates/xray-tui-db/src/lib.rs` — re-export hub; Database, DatabaseError, Result public
:- `crates/xray-tui-db/src/error.rs` — DatabaseError, Result, ProfileWithDetails
:- `crates/xray-tui-db/src/database.rs` — Database struct + all public query/write methods
:- `crates/xray-tui-db/src/inner.rs` — private _inner helpers (31 functions, read + write)
:- `crates/xray-tui-db/src/columns.rs` — private column-index enums for compile-time safe row extraction
:- `crates/xray-tui-db/src/convert.rs` — private from_row impls for all model types
:- `crates/xray-tui-db/src/helpers.rs` — private percent_decode / normalize_remark utilities
:- `crates/xray-tui-db/src/models.rs` — Profile (computed JOIN view), ProfileCore (deduplicated server config), Group, Subscription, GRAVEYARD_GROUP_ID, ALL_GROUP_ID
- `crates/xray-tui-config/src/lib.rs` — config management, module registration
- `crates/xray-tui-core/src/grpc_client.rs` — StatsProvider trait + XrayGrpcClient/SingBoxGrpcClient + factory
- `crates/xray-tui-core/src/updater.rs` — backend auto-update (version check, download, install) for xray-core and sing-box
    - `crates/xray-tui-config/src/import_export.rs` — share URL parse/format (14 protocols + fallback chain) with per-profile validation via ValidationSettings
- `crates/xray-tui-config/src/base64_util.rs` — robust base64 decode with percent-decoding and annotation stripping
- `crates/xray-tui-config/src/permissive_json.rs` — lenient JSON parser for vmess:// subscriptions
- `crates/xray-tui-config/src/fast_perc.rs` — hand-rolled UTF-8 + percent-decoding character source
- `crates/xray-tui-config/src/subscription.rs` — chunked base64 streaming decoder with URL splitting
- `crates/xray-tui-db/src/models.rs` — Profile (computed JOIN view), ProfileCore (deduplicated server config), Group, Subscription, GRAVEYARD_GROUP_ID, ALL_GROUP_ID
:- `crates/xray-tui-core/src/speed_test.rs` — async speed test engine (TCP ping, real ping with IP info, speed test, UDP test) using tokio + reqwest SOCKS5 proxy. RealPingResult includes latency + ISP info. Configurable via SpeedTestConfig. Batch versions (start_batch_ping, start_batch_then_real_ping) in `lib.rs` with concurrent TCP (`tcp_ping_concurrency`, default 20 via Semaphore), wave-ordered real pings, and stop-testing support (`speed_test_stop: Arc<AtomicBool>`).
:- `crates/xray-tui-core/src/log_heed.rs` — HeedLogStorage: LMDB-backed persistent log storage (postcard-encoded LogMessage entries, two databases for logs + targets)

- `crates/xray-tui-core/src/process.rs` — CoreManager subprocess lifecycle, stdout/stderr capture via log channel
### TUI screens (crates/xray-tui/src/ui/)
- `mod.rs` — run(), render(), event loop, keyboard handler, tab routing, AppMode dispatch, speed test menu overlay
:- `profiles.rs` — profile list DataGrid with connected indicator, IP info column, multi-sort indicators, graveyard group filter; multi-select, delete confirmation, batch import overlay
- `add_server.rs` — form rendering, protocol picker, field editing, import URL screen
:- `settings.rs` — Settings panel with split-pane tree+form view. Left pane: collapsible tree (SPLIT_SETTINGS_TREE const, SettingsSection-navigated). Right pane: Form, UpdateForm, Empty. Sections: Core, GUI, Inbound, System Proxy, TUN, Mux, Statistics, Protocol Core, SpeedTest, Logging, Routing Rules (list+form, reorder). Tree navigation (arrows) + form focus switching (Ctrl+W). Full rewrite. Replaced per-section SettingsMode variants with unified Split { tree, focus, right }.
- `groups.rs` — system-group-aware management (is_system guard, clear action)
- `logs.rs` — live core log viewer with scrollable display, color-coded log levels (error/warning/info/debug), keyboard navigation (Up/Down/PgUp/PgDn/Home/End)
- `theme.rs` — central Palette-derived style methods (ThemeStyles struct) — static methods returning Style from a &Palette
- `actions_log.rs` — live state info panel: connection status, server info, test results, traffic/memory, recent logs
- `palette_bridge.rs` — maps ratatui-themes `ThemePalette` → ratatui-cheese `Palette` (10-to-11 color role mapping)
:- `widgets/` — reusable widgets: `DataTable` (sortable, selectable, multi-select, virtual-scrolled with themed scrollbar, Column/ColumnWidth/SortDirection/DataTableRow trait)

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

1. **Dual-backend architecture**: `CoreManager` abstracts over xray-core and sing-box subprocesses. The TUI writes JSON configs and manages the binary lifetime.
2. **Protocol-core auto-resolution**: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → sing-box. All others (VMess, VLESS, Shadowsocks, etc.) → xray-core by default. User overrides per-profile.
3. **One core at a time**: Only one backend process runs per connection session. Switching profiles between backends stops the current core and starts the other. Matches v2rayN.
4. **Ratatui + Crossterm** — TUI framework. Async via tokio. Single render thread with async tasks sending updates via mpsc channels.
5. **SQLite via turso (async)** — Single DB file for all persistent data. Async pure-Rust SQLite engine. All DB methods are `async fn` with `#[repr(usize)]` column enums for compile-time safe row extraction. Transaction via `unchecked_transaction().await`. Connection wrapped in `Arc<Database>` for shared access across tasks.
6. **Multi-crate workspace** — Separation: bin crate (TUI) + 3 lib crates (core, db, config).
7. **Config generation** — Two builders: xray.rs (ports v2rayN's CoreConfigContextBuilder) and singbox.rs (ports sing-box JSON format).
8. **gRPC stats abstraction**: `StatsProvider` trait with `XrayGrpcClient` (xray-core native gRPC) and `SingBoxGrpcClient` (sing-box V2Ray API experimental).
9. **Sing-box config differs structurally** from xray-core: `type` vs `protocol`, `route` vs `routing`, `experimental.v2ray_api` vs `stats`+`api`+`policy`, different TLS/transport key names.
11. **Theme system**: `ThemeStyles` struct (in `theme.rs`) provides static methods returning `Style` from a `ratatui_cheese::theme::Palette`. `Palette` is constructed from `ratatui_themes::ThemeName` (stored in `AppConfig.theme_name`) → `ratatui_themes::Theme` → `palette_bridge::current_palette()`. `AppState::current_palette()` is the canonical accessor. Every screen accepts `&Palette` and calls `ThemeStyles::*` methods instead of hardcoded colors.
12. **Sing-box V2Ray API is experimental**: May require build tag `with_v2ray_api`. If unavailable, stats/logs show "not supported by core".
## Protocols: In Scope

### Xray-core native
VMess (AES/Chacha20/None/AEAD security), VLESS (with flow control: xtls-rprx-vision), Shadowsocks, Shadowsocks-2022, SOCKS, HTTP, Trojan, WireGuard, Hysteria v2, Dokodemo-door, Freedom, Blackhole, DNS, Loopback, Custom

### Sing-box only (extra beyond xray-core)
TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect, TProxy, Mixed (inbound)

## Protocols: Out of Scope

Anything requiring a third binary backend beyond xray-core or sing-box.

## Common Tasks
:**Phase overview**: Phases 0-6 (Foundation through Settings) are fully implemented. Phase 6 includes a split-pane settings refactor (unified SettingsMode::Split replacing per-section form variants). Phase 7 (Advanced Features) has completed: logs tab, sing-box config builder for all 17 outbound protocols, normalized profile schema, speed test config with batch-then-real-ping, profiles table redesign (connected indicator, IP info, graveyard filter), **Heed-backed log storage** (HeedLogStorage, LMDB, Settings→Logging form). Phase 8 (Polish) has completed: confirmation overlay redesign, quit confirmation when connected, form validation with inline errors, empty-state guidance, search cursor, actions panel collapse, consistent form field display, scroll indicators, Home/End in group overlay, PgUp/PgDn in profiles, inverted log scroll, Ctrl+A select-all/deselect-all, connection indicator in tab bar, update indicator styling, statistics screen refactored into bordered sections. **Log subsystem overhaul** completed: non-blocking TuiLogLayer (std::sync::mpsc channel instead of direct heed writes), background batched heed writer (batch up to 100 messages per transaction, spawn_blocking), MapFull→resize_map with retry (1GB default, doubles up to 8GB, atomic counter instead of tracing events), async heed read wrappers (spawn_blocking for all reads), lazy log loading on first Logs tab access. **Theme system overhaul** completed: integrated ratatui-themes + ratatui-cheese crates, replaced hardcoded Theme Style constants with ThemeStyles + Palette pattern, added palette_bridge, extracted DataTable widget, added mouse support via tui-popup overlays.
### Adding a new protocol form
1. Add config type enum variant and assign core type in `protocol_core_mapping.rs`
2. Create form fields in `crates/xray-tui-config/src/forms.rs` matching that protocol's parameters
3. Create `XxxFmt::parse_share_url` and `XxxFmt::format_share_url` in `crates/xray-tui-config/src/import_export.rs`
4. Add profile validation in the appropriate config builder (`xray.rs` or `singbox.rs`)
5. Reference: v2rayN's individual `*Fmt.cs` files for format specs; sing-box's `option/*.go` for JSON config structs
### Adding subscription management features
1. `confirm_add_group()` / `confirm_edit_group()` in `crates/xray-tui/src/lib.rs` handle form submit
2. `update_group_subscriptions()` / `do_update_subscription()` in same file handle HTTP fetch + parse + DB upsert
3. `subscription_upsert_profiles()` in `crates/xray-tui-db/src/lib.rs` handles content-based dedup via `ON CONFLICT(group_id, sub_uid)`
4. `move_orphans_to_graveyard()` / `purge_graveyard()` handle stale profile cleanup
5. `spawn_auto_update()` runs background check at 60s intervals, comparing SQL datetime() arithmetic
6. `shutdown_token: Arc<AtomicBool>` on `AppState` signals the background loop to stop on quit — checked via `token.load()` in each iteration

### Adding log storage features
1. `LogMessage` struct in `crates/xray-tui-core/src/log_heed.rs` — fields: `timestamp_nanos`, `level`, `target`, `message`
2. `HeedLogStorage` in same file — wraps two heed (LMDB) databases: `logs` (u64 BE timestamp → postcard-encoded `LogMessage`) and `targets` (set of seen target strings)
3. Methods: `write_log()` (single entry convenience), `write_log_batch()` (batched transaction with MapFull→resize+retry), `try_write_batch()` (internal single-txn batch with MapFull discrimination), `read_recent()`, `read_newer_than()`, `read_older_than()`, `get_targets()`, `delete_older_than()`, `clear_all()` (empties both logs+targets databases). Async variants: `read_recent_async()`, `read_newer_than_async()`, `read_older_than_async()`, `get_targets_async()`, `clear_all_async()` — each wraps the sync call in `spawn_blocking`. All methods accept `self: &Arc<Self>` for the async variants (Arc clone for spawn_blocking).
4. **TuiLogLayer** (`main.rs`) uses `std::sync::mpsc::Sender<LogMessage>` (unbounded channel, never blocks under the tracing lock). `on_event` sends `LogMessage` via the channel; a background `spawn_blocking` task receives, batches up to 100 messages, and calls `heed.write_log_batch()`. MapFull triggers `unsafe { env.resize(new_size * 2) }` with retry and atomic fail counter.
5. Core process log forwarding (`connect_to_profile` in `lib.rs`) sends to the same `std::sync::mpsc::Sender` channel (clone stored in `AppState.log_sender_tx`), not directly to heed.
6. Log tab polls heed via async read wrappers (`read_newer_than_async`, `read_older_than_async`) from the event loop (spawn_blocking, non-blocking to the TUI render thread).
7. TTL cleanup runs in a background tokio task, calls `delete_older_than()` via `spawn_blocking` (infrequent, every 10 min).
8. Initial log loading is lazy — deferred from startup to first `Tab::Logs` activation (`AppState.logs_loaded` flag).
9. DEFAULT_MAP_SIZE is 1 GB (was 256 MB) — see `DEFAULT_MAP_SIZE` constant. Runtime resizing doubles up to 8 GB on MapFull.
10. Error handling: `HeedError::MapFull` variant for discrimination. `mapsize_full_count: AtomicU64` counter tracks MapFull events without emitting `tracing::error!()` (which would re-enter TuiLogLayer).

### Adding batch import for share URLs
1. Parse each URL with `parse_share_url(url, &config.validation)` from `xray_tui_config::import_export`
2. Collect results as `Vec<BatchImportItem>` and set `AppMode::BatchImport { results, scroll }`
3. Render scrollable success/failure list in `profiles.rs` (`render_batch_import()`)
4. Batch import mode handles keys: Up/Down scroll, Enter saves all successful imports via `db.add_profile()`, Esc cancels
5. Reference `crates/xray-tui/src/lib.rs` `start_batch_import()` method for the pattern
6. Refer to `groups.rs` for group management overlay UI patterns (matching `add_server.rs` form conventions)

### Adding a new speed test type
1. Add variant to `TestType` enum in `crates/xray-tui-core/src/speed_test.rs`
2. Implement the async function in `speed_test.rs` using tokio timeouts
3. Add a `start_xxx_test()` method in `crates/xray-tui/src/lib.rs` AppState that spawns a tokio task calling the function
4. Wire the result into `CoreEvent::SpeedTestResult` handler in `poll_core_events()`
5. Add menu item in `render_speed_test_menu()` in `crates/xray-tui/src/ui/mod.rs`
6. Add key handler entry in `handle_key()` menu navigation
7. Use `create_socks5_client(proxy, port, socks5h, timeout)` helper from `speed_test.rs` to build the reqwest::Client with SOCKS5 proxy — reuses connection pool and avoids per-call construction overhead

### Adding stop-testing support to a batch operation
1. Add `speed_test_stop: Arc<AtomicBool>` to AppState, initialize `Arc::new(AtomicBool::new(false))`
2. Add `stop_speed_test()` method: `self.speed_test_stop.store(true, Ordering::Relaxed)`
3. Clone the flag and pass into the spawned task. Check at iteration boundaries (page, target, profile) with `stop_flag.load(Ordering::Relaxed)`. Break when true.
4. Track which profile IDs received results via a `HashSet<String>` to prevent double-emission of "Cancelled" — only emit for unprocessed profiles.
5. Auto-reset flag when `testing_profiles.is_empty()` in `poll_core_events` handler.
6. Add menu entry in `SPEED_TEST_MENU_ITEMS` with dispatch. Add hotkey `'s'` in key handler.
7. Update status bar to show "■ Stopping..." indicator when flag is active, overriding testing indicators.

### Adding a backend auto-update feature
1. Create functions in `crates/xray-tui-core/src/updater.rs`: `get_current_version` (runs subprocess), `get_latest_version` (GitHub releases API), `download_release` (streaming download to temp dir), `install_binary` (extract to temp → verify → .bak → copy all → remove .bak on success/restore from .bak on failure)
2. Add `UpdateCheckResult` and `UpdateCompleted` variants to `CoreEvent` enum in `crates/xray-tui/src/lib.rs`; handle them in `poll_core_events()` to update `update_status: HashMap<CoreType, BackendUpdateStatus>`
3. Add `spawn_update_check()` and `spawn_update_download()` methods on `AppState` — each spawns a tokio task that calls updater functions and sends results back through the core event channel
4. Add `UpdateForm` to `SplitRightPane` and `Updates` to `SettingsSection`; wire into `build_right_pane()` with snapshot of current status
5. Create `render_update_form()` and `handle_update_form_key()` in `settings.rs` — C triggers check, D triggers download for all available updates, Esc goes back
6. Add update-available indicator (colored `[Update: ...]`) to `status_bar.rs`
7. Add startup check in `ui::run()` gated by `config.updates.check_on_startup`

### Determining which core a protocol belongs to

- Reference `thirdparty/sing-box/constant/proxy.go` for sing-box protocol type strings
- Reference `thirdparty/Xray-core/proxy/` directory listing for xray-core protocols
- Update `protocol_core_mapping.rs` with the new entry
- Protocols present in both: prefer xray-core (user can override profile core_type to force sing-box)

### Adding a new gRPC-based feature (stats, logs, routing API)
1. Add/update proto definition in `crates/xray-tui-core/proto/` and re-run build (auto-compiled via build.rs)
2. Add query method to `StatsProvider` trait in `grpc_client.rs`
3. Implement in both `XrayGrpcClient` and `SingBoxGrpcClient`
4. Add a `CoreEvent` variant in `crates/xray-tui/src/lib.rs` and handle in `poll_core_events()`
5. For polling: add to the select loop in `connect_to_profile()` with appropriate interval
6. For UI: update the relevant screen module in `crates/xray-tui/src/ui/`

## Style Guide

- Rust 2024 edition
- Follow clippy with workspace-level lints (`pedantic` + `nursery` at `warn`, config in root `Cargo.toml` under `[workspace.lints.clippy]`, per-crate opt-in via `[lints] workspace = true`)
- Format with `rustfmt` (use `cargo fmt`)
- Tests go next to code in same file (unit) or `tests/` (integration)
- Use `thiserror` for error types
- Use `anyhow` for error propagation where appropriate
- Use `semver` for version parsing and comparison (backend updater)
- Use `serde` for JSON serialization
- Use `tokio` for async runtime
- gRPC via `tonic` crate
:- `reqwest` for HTTP client (subscription fetch)
- SQLite via `turso` crate (async)
:- `tracing` for diagnostic event system (subscriber in bin crate, macros in lib crates)
:- `tracing-subscriber` for event filtering, formatting, and TuiLogLayer routing
:- `escape8259` for JSON string unescaping
- Use `memchr` for vectorized byte search
:- `tokio` for async runtime (also direct dep in xray-tui-db for retry backoff sleep)
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
