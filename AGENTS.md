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
- `crates/xray-tui/src/main.rs` — binary entry, tokio::main, subsystem init
- `crates/xray-tui/src/lib.rs` — AppState, Tab, SortColumn, ProfileRow, LogLine, SettingsMode, SettingsSection
- `crates/xray-tui-core/src/lib.rs` — core logic facade
- `crates/xray-tui-db/src/lib.rs` — database layer + query methods
- `crates/xray-tui-config/src/lib.rs` — config management, module registration
- `crates/xray-tui-core/src/grpc_client.rs` — StatsProvider trait + XrayGrpcClient/SingBoxGrpcClient + factory
- `crates/xray-tui-core/src/updater.rs` — backend auto-update (version check, download, install) for xray-core and sing-box
- `crates/xray-tui-config/src/import_export.rs` — share URL parse/format (14 protocols + fallback chain)
- `crates/xray-tui-config/src/base64_util.rs` — robust base64 decode with percent-decoding and annotation stripping
- `crates/xray-tui-config/src/permissive_json.rs` — lenient JSON parser for vmess:// subscriptions
- `crates/xray-tui-config/src/fast_perc.rs` — hand-rolled UTF-8 + percent-decoding character source
- `crates/xray-tui-config/src/subscription.rs` — chunked base64 streaming decoder with URL splitting
- `crates/xray-tui-db/src/models.rs` — Profile (computed JOIN view), ProfileCore (deduplicated server config), Group, Subscription, GRAVEYARD_GROUP_ID, ALL_GROUP_ID
- `crates/xray-tui-core/src/speed_test.rs` — async speed test engine (TCP ping, real ping, speed test, UDP test, batch ping) using tokio + reqwest SOCKS5 proxy

### TUI screens (crates/xray-tui/src/ui/)
- `mod.rs` — run(), render(), event loop, keyboard handler, tab routing, AppMode dispatch, speed test menu overlay
- `profiles.rs` — profile list DataGrid, multi-select indicator, delete confirmation overlay
- `add_server.rs` — form rendering, protocol picker, field editing, import URL screen
- `settings.rs` — Settings panel with menu navigation, config forms (Core/GUI/Inbound/DNS/SystemProxy/TUN/Mux/Statistics), routing rules list+form, reorder. Full rewrite Phase 6.
- `status_bar.rs` — bottom connection indicator + key hints
- `groups.rs` — group management overlay with list/add/edit/clear/delete, system group handling
- `statistics.rs` — live traffic and system stats display
- `theme.rs` — central color palette and Style definitions (Theme struct)

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
5. **SQLite via rusqlite** — Single DB file for all persistent data.
6. **Multi-crate workspace** — Separation: bin crate (TUI) + 3 lib crates (core, db, config).
7. **Config generation** — Two builders: xray.rs (ports v2rayN's CoreConfigContextBuilder) and singbox.rs (ports sing-box JSON format).
8. **gRPC stats abstraction**: `StatsProvider` trait with `XrayGrpcClient` (xray-core native gRPC) and `SingBoxGrpcClient` (sing-box V2Ray API experimental).
9. **Sing-box config differs structurally** from xray-core: `type` vs `protocol`, `route` vs `routing`, `experimental.v2ray_api` vs `stats`+`api`+`policy`, different TLS/transport key names.
10. **Sing-box V2Ray API is experimental**: May require build tag `with_v2ray_api`. If unavailable, stats/logs show "not supported by core".

## Protocols: In Scope

### Xray-core native
VMess (AES/Chacha20/None/AEAD security), VLESS (with flow control: xtls-rprx-vision), Shadowsocks, Shadowsocks-2022, SOCKS, HTTP, Trojan, WireGuard, Hysteria v2, Dokodemo-door, Freedom, Blackhole, DNS, Loopback, Custom

### Sing-box only (extra beyond xray-core)
TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect, TProxy, Mixed (inbound)

## Protocols: Out of Scope

Anything requiring a third binary backend beyond xray-core or sing-box.

## Common Tasks

**Phase overview**: Phases 0-6 (Foundation through Settings) are fully implemented. Phase 7 (Advanced Features — logs, sing-box builder completeness, advanced forms) is in progress. Phase 8 (Polish & Release — i18n, themes, packaging) is planned. Phase 9 (v2rayN Parity — proxy chains, policy groups, Clash dashboard, subscription enhancements) captures the remaining feature gaps for full v2rayN feature parity. See `ROADMAP.md` for full details.

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

### Adding batch import for share URLs
1. Parse each URL with `parse_share_url()` from `xray_tui_config::import_export`
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

### Adding a backend auto-update feature
1. Create functions in `crates/xray-tui-core/src/updater.rs`: `get_current_version` (runs subprocess), `get_latest_version` (GitHub releases API), `download_release` (streaming download to temp dir), `install_binary` (extract to temp → verify → .bak → copy all → remove .bak on success/restore from .bak on failure)
2. Add `UpdateCheckResult` and `UpdateCompleted` variants to `CoreEvent` enum in `crates/xray-tui/src/lib.rs`; handle them in `poll_core_events()` to update `update_status: HashMap<CoreType, BackendUpdateStatus>`
3. Add `spawn_update_check()` and `spawn_update_download()` methods on `AppState` — each spawns a tokio task that calls updater functions and sends results back through the core event channel
4. Add `UpdateForm` to `SettingsMode` and `Updates` to `SettingsSection`; wire into `enter_settings_form()` with snapshot of current status
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
- Follow clippy with default settings
- Format with `rustfmt` (use `cargo fmt`)
- Tests go next to code in same file (unit) or `tests/` (integration)
- Use `thiserror` for error types
- Use `anyhow` for error propagation where appropriate
- Use `semver` for version parsing and comparison (backend updater)
- Use `serde` for JSON serialization
- Use `tokio` for async runtime
- gRPC via `tonic` crate
- HTTP via `reqwest` crate
- SQLite via `rusqlite` crate
- Use `reqwest` for HTTP client (subscription fetch)
- Use `escape8259` for JSON string unescaping
- Use `memchr` for vectorized byte search
- Use `rapidhash` for content-based profile deduplication (compute_sub_uid)
- Use `base64-simd` for SIMD-accelerated base64 decode/encode
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
