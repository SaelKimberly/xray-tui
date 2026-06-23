# xray-tui

A terminal user interface (TUI) client for Xray-core and Sing-box, written in Rust.
Feature target: feature parity with v2rayN (C# desktop GUI) for all protocols supported by either Xray-core or Sing-box.

## Project structure

```
xray-tui/
├── xray-tui/          # Binary: ratatui event loop + all screens
│   └── src/ui/         # TUI screen modules (profiles, add_server, status_bar, settings, groups, logs, statistics, actions_log, theme)
├── xray-tui-core/     # Library: business logic, dual-core process mgmt, gRPC client, config builders
├── xray-tui-db/       # Library: SQLite persistence layer + query methods
├── xray-tui-config/   # Library: import/export format parsers, protocol form fields, JSON config management
├── thirdparty/
│   ├── Xray-core/         # Source of truth for protocols and behavior
│   ├── sing-box/          # Source of truth for sing-box protocols, config format, and API
│   ├── v2rayN/            # Source of truth for UI/UX feature set
│   ├── shoes/             # Rust proxy - possible protocol reference
│   ├── leaf/              # Rust proxy framework
│   └── shadowsocks-rust/  # Shadowsocks protocol in Rust
├── ROADMAP.md
├── CONTEXT.md
├── ARCHITECTURE.md
├── TUI_MANUAL.md
└── AGENTS.md

## Key reference repositories

- **Xray-core** (`thirdparty/Xray-core/`): Go implementation of the primary proxy core. Run as a subprocess, controlled via JSON config files and gRPC API. Handles VMess, VLESS, Shadowsocks, SOCKS, HTTP, Trojan, WireGuard, Hysteria2, and other native protocols.
- **sing-box** (`thirdparty/sing-box/`): Go universal proxy platform serving as the secondary backend. Covers protocols Xray-core does not natively support: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, Redirect, and more. Config format differs from Xray-core. Provides experimental V2Ray API and Clash API compatibility.
- **v2rayN** (`thirdparty/v2rayN/`): C# desktop client. The feature reference for all UI/UX decisions. Protocol support, settings panels, and workflow patterns derive from this project.
- **shoes** (`thirdparty/shoes/`): Rust proxy server with VLESS/Vision/VMess/Shadowsocks/Trojan/Hysteria2/TUIC v5/AnyTLS/NaiveProxy support. Reference for protocol implementation patterns.
- **leaf** (`thirdparty/leaf/`): Rust proxy framework library. Embeddable proxy client implementation reference.
- **shadowsocks-rust** (`thirdparty/shadowsocks-rust/`): Mature Shadowsocks implementation in Rust. Protocol dependency candidate.

## Domain glossary

| Term | Meaning |
|---|---|
| **Inbound** | Local listening port/protocol that accepts traffic (SOCKS, HTTP, etc.) |
| **Outbound** | Remote proxy server connection (VMess, VLESS, Shadowsocks, etc.) |
| **Subscription group** | A named group of proxy profiles fetched from a subscription URL |
| **ALL_GROUP_ID** | Fixed UUID (`00000000-0000-0000-0000-000000000000`) identifying the system "All" group for showing every profile across groups |
| **sub_uid** | Content-based hash (rapidhash) of profile identity fields — used for dedup during subscription update |
| **Graveyard** | Destination group (`sub-graveyard`) for orphaned subscription profiles; purged after 24h |
|- **testing_details** | `HashMap<String, TestType>` tracking active test type per profile — enables TestTypeUpdate event to switch displayed emoji mid-flow (TcpPing→RealPing) during batch-then-real-ping |
| **Transport** | The network layer used for outbound connections (TCP, WebSocket, gRPC, QUIC, etc.) |
| **Stream Security** | TLS/REALITY/None wrapper around the transport |
| **Config Type** | The proxy protocol (VMess, VLESS, Shadowsocks, Trojan, TUIC, etc.) |
| **Routing** | Rule-based traffic distribution: domain/IP match → specific outbound or balancer |
| **REALITY** | XTLS's TLS resemblance mechanism — makes proxy traffic look like normal web traffic |
| **uTLS fingerprint** | Browser TLS fingerprint emulation (Chrome, Firefox, Safari, etc.) |

## Key design decisions

- **Normalized schema**: Profiles are split across `profile_cores` (deduplicated server configs keyed by `sub_uid` hash) and `group_profiles` (per-group profile membership with per-group remarks). Query pattern always JOINs both tables. `ALL_GROUP_ID` system group shows all profiles across groups.
- **Dual-backend architecture**: `CoreManager` abstracts over xray-core and sing-box subprocesses. Each profile is tagged with a core type (auto, xray, sing-box). Auto mode resolves based on protocol.
- **Protocol-core auto-resolution**: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH, Tailscale, ShadowsocksR, Redirect → sing-box. All others (VMess, VLESS, Shadowsocks, etc.) → xray-core by default. User can override per-profile.
- **One core runs at a time**: Switching profiles between backends stops the current core process and starts the other. Matches v2rayN behavior and avoids port conflicts.
- **Xray-core runs as a subprocess**; the TUI writes JSON config files and communicates via gRPC API.
- **Sing-box runs as a subprocess**; the TUI writes JSON config files and communicates via sing-box's experimental V2Ray API (gRPC compatible).
- **TUI framework**: **Ratatui + Crossterm** (async via tokio).
- **Storage**: **SQLite** via `rusqlite` (single DB file) for profiles, subscriptions, routing, DNS, stats.
- **Multi-crate workspace**: `xray-tui` (bin), `xray-tui-core` (lib), `xray-tui-db` (lib), `xray-tui-config` (lib).
- **Protocols in scope**: Everything supported natively by either Xray-core or Sing-box. No third binary backends.

## Key source files

- `crates/xray-tui-db/src/models.rs` — ProfileCore struct for deduplicated server configs, Profile as JOIN view, ALL_GROUP_ID constant
- `crates/xray-tui/src/ui/groups.rs` — system-group-aware management (is_system guard, clear action)
- `crates/xray-tui/src/ui/actions_log.rs` — live event log panel: connection status, speed test results, core/TUI logs, traffic/memory
:- `crates/xray-tui/src/lib.rs` — BatchImport mode, ClearGroup event, start_batch_import method, batch-then-real-ping flow, testing_details HashMap, logs_show_validation toggle, SpeedTestConfig
