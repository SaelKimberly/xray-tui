# xray-tui Initial Baseline

Date: `2026-09-11`
Status: `initial dual-baseline snapshot`

## 1. Purpose

Bootstrap the project's product and architecture baselines so later
`Baseline Role Alignment` checks can distinguish a defective baseline from
implementation drift. This snapshot is evidence, not authority; authority lives
in `AGENTS.md`, `TUI_MANUAL.md`, `NATIVE_CORE.md`, and accepted ADRs.

## 2. Workspace Structure

- `crates/xray-tui` — TUI binary: event loop, screens, ops (connect, ping,
  enrich, scheduler, profiles, subscriptions).
- `crates/xray-tui-db` — typed persistence (toasty 0.10 over Turso), 7 tables.
- `crates/xray-tui-proto` — protocol configs, URL parsing, Clash conversion,
  per-kind identity writer, core injection.
- `crates/xray-tui-core` — core facade: config builders, gRPC stats, ping
  managers, process manager, updater, heed log storage.
- `crates/xray-tui-native` — in-process proxy core (subprocess-free arm).
- `crates/xray-tui-tls` — TLS fingerprint + REALITY client engine.
- `crates/xray-tui-route` — first-match routing engine with sniffers.
- `crates/xray-tui-config`, `-dns`, `-geoip`, `-host-features` — supporting
  libraries.
- `crates/xray-tui-hakari` — cargo-hakari feature unification.
- `thirdparty/` — read-only reference: Xray-core, sing-box, v2rayN, shoes,
  leaf, shadowsocks-rust.

## 3. Current Authority Surfaces

- `AGENTS.md` — project identity, critical design decisions 1-20, per-task
  recipes, verification commands.
- `TUI_MANUAL.md` — screens, keys, workflows.
- `NATIVE_CORE.md` — native core and TLS engine purpose, principles,
  verification tiers, capability roadmap.
- `docs/protocols.md`, `docs/clash.md`, `docs/native-core-integration.md`,
  `docs/routing-engine-decision.md`, `docs/tls-fingerprint-roster.md`.
- `docs/aegis/` — this workspace.
- Gaps: no ADR directory before this date; no product requirement document
  beyond `AGENTS.md` claims of v2rayN parity.

## 4. Product / Requirement Baseline

### 4.1 Current Truth

- Target: a Rust TUI client for Xray-core and Sing-box with feature parity to
  v2rayN for every protocol either backend supports.
- Users: a single operator managing large subscription feeds (7k+ endpoints)
  from a terminal.
- Requirement items in force: protocol coverage (in-scope list in `AGENTS.md`);
  connect/disconnect; fast and real ping; speed test; subscription refresh;
  routing rules; DNS settings; logs; statistics; native activity trace;
  settings surface (14 sections); group/subscription management.
- Acceptance expectations: `just quality-gate` passes; unit and e2e suites
  green (476 lib + 343 proto unit; 136 e2e with 6 ignored); manual runs against
  real xray-core and sing-box binaries.
- Phase focus already fixed: Phases 0-8 plus profiles redesign, log subsystem,
  theme system, test-priority sorting, native core, TLS/REALITY engine.

### 4.2 Non-negotiables

1. One core at a time; only one backend process per session.
2. No system groups; All/Graveyard are view filters.
3. Protocol identity excludes host/port; credentials never move `sig`.
4. Persisted `core_type` stays `{Xray, SingBox}`; native is runtime-only.
5. Every network path has a hard deadline.

### 4.3 Product Non-goals

- Any protocol requiring a third backend binary beyond xray-core or sing-box.
- Persisting the native core choice.
- Migrating an existing database across schema versions (a tag mismatch wipes).

## 5. Architecture / Runtime Boundary Baseline

### 5.1 Current Truth

- Canonical owners: `xray-tui-db` owns persistence and query shape;
  `xray-tui-proto` owns protocol config and identity; `xray-tui-core` owns
  config building and process lifetime; `xray-tui-native` owns the in-process
  data path; `xray-tui-tls` owns TLS/REALITY.
- Contract boundaries: `ProtocolConfig::try_parse_proto` → `ParsedProto`
  (parse boundary); `InjectToCoreConf::inject_to` (per-core config emission);
  `resolve_core` (core selection); `CoreEvent` (backend→TUI event contract).
- Dependency direction: proto → core/native/tls/route; db depends on proto;
  the TUI depends on all of them.
- Persistence: 7 typed tables, schema tag `PRAGMA user_version=7`, no
  migrations, raw SQL restricted to PRAGMAs.

### 5.2 Architecture Non-negotiables

1. `push_schema` runs only under the matching tag; a mismatch deletes the file.
2. Typed reads only on the profiles path; no toasty `.include()` there.
3. Result events are keyed by `(endpoint_id, protocol_id)`.
4. Capability gates fail closed; unsupported native rows fall back to a
   subprocess with a named warning.
5. Secret key material is wiped; crypto dependency features are re-checked
   after any dependency edit.

### 5.3 Architecture Non-goals

- No ORM-level migrations.
- No HTTP/2 or HTTP framing hand-rolled where hyper/h2 already serve.
- No blanket certificate-verification bypass.

## 6. Ownership / Contract Snapshot

| Surface | Owner |
| --- | --- |
| Profiles tab row model and key handling | `xray-tui/src/ui/profiles.rs` + `ops/profiles.rs` |
| Test-priority ordering law | `EndpointRow::link_test_tier` / `link_test_key` (`xray-tui-db`) |
| Scheduler gate | `xray-tui/src/ops/scheduler.rs` |
| Enrichment spawns | `xray-tui/src/ops/enrich.rs` |
| Identity/uid | `xray-tui-proto/src/proto_spec/identity.rs` |
| Core selection | `proto_spec/core_mapping.rs` + `ops/connect.rs` |
| Schema and tag | `xray-tui-db/src/database.rs` |

Missing seams: none recorded at this date.

## 7. Current State and Risks

- Stage: feature-complete for the target protocol set; ongoing hardening of the
  profiles data path and the native core.
- Risks: profiles tab loads the whole typed schema per reload and rebuilds
  every display row per frame (addressed by the 2026-09-11 spec); raw SQL
  absence constrains query shapes; timestamp columns are text and depend on
  fixed-width formatting.

## 8. Alignment Use

- Read the Product / Requirement Baseline before product behaviour, feature
  scope, or acceptance changes.
- Read the Architecture / Runtime Boundary Baseline before schema, owner,
  contract, or dependency changes.
- Report `scope: both` when a change alters behaviour and the boundary that
  carries it.

## 9. Compatibility Boundary

- Existing `data.db` files must reopen under tag 7 without deletion; any change
  that requires a tag bump is a wipe and needs explicit user approval.
- `CoreEvent` variants consumed by the TUI remain source-compatible through the
  current phase.
- Public crate APIs are workspace-internal; no external compatibility promise
  is made.
