# Typed Data Model Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the blob-based data model (`spec_blob`, settings-JSON round-trips, raw SQL, string timestamps) with a fully typed Toasty 0.9 model (embedded types, `Deferred<Json<T>>`, jiff timestamps, composite keys, OCC), a single typed parse boundary (`Proto` = endpoints + protocol), a `ProfileStats` activity hub with task gate/queue, and `InjectToCoreConf`-based config builders.

**Architecture:** Bottom-up: (A) proto crate parse split (configs lose host/port; `Proto` gains `endpoints` + `protocol`), (B) db crate model rewrite (7 tables, typed queries, zero raw SQL except PRAGMAs), (C) config crate boundary cleanup, (D) core crate builder rewrite via `InjectToCoreConf`, (E) TUI scheduler + ping pipeline + labels, (F) verification.

**Tech Stack:** Rust 2024, Toasty 0.9 (`turso`, `serde`, `jiff` features), jiff, tokio, serde/serde_json.

**Spec:** `docs/superpowers/specs/2026-08-05-typed-data-model-design.md` (authoritative — read before implementing any task).

## Global Constraints

- Fresh database: no migrations, no `PRAGMA user_version`, no legacy-format compat. Pure `db.push_schema()`.
- Zero raw SQL except `PRAGMA journal_mode=WAL`, `PRAGMA busy_timeout=5000`, `PRAGMA foreign_keys=ON` (connection setup only).
- `uid = sig ^ cred_hash` computed from protocol essentials ONLY — never includes host/port.
- `task_id`/`task_queue` are `u16`; 0 is invalid by convention; DashMap keys `u16`.
- `#[auto]` on `created_at: jiff::Timestamp` / `updated_at: jiff::Timestamp` per Toasty shorthand (needs toasty `jiff` feature).
- `Deferred<Json<T>>` fields require `#[column(type = text)]` on Turso.
- `#[shared(delay)]` on `Latency` variant fields — NOT `#[column(variant)]`.
- Queue mutation on Turso is whole-vec read-modify-write (no `pop`/`remove` on `Vec<scalar>`); guarded by `#[version]` OCC.
- `retry_on_busy` + per-connection `busy_timeout` remain.
- Clash YAML structs stay as external-format boundary types — never delete.
- Workspace lints: `cargo clippy` (pedantic+nursery warn), `cargo fmt`, Rust 2024 edition.
- `xray-tui-db` gains `xray-tui-proto` dependency (acyclic: proto has no internal deps). `xray-tui-proto` gains `toasty` with `default-features = false` (for `Embed` derives only).
- Every task ends with a test cycle + commit. Do not run project-wide `cargo test`/`clippy` inside a task; run them once at the end (Task 30).

---

## File Structure

| File | Responsibility after redesign |
|---|---|
| `crates/xray-tui-proto/src/proto_spec/kinds.rs` (new) | `ProtocolKind`, `TransportType`, `SecurityType` enums (`toasty::Embed`); move `Protocol` here from `xray-tui-core/src/protocol.rs` |
| `crates/xray-tui-proto/src/proto_spec/endpoint.rs` (new) | `EndpointEssentials`, `ProtocolEssentials`, `ParsedProto` (the new parse boundary types) |
| `crates/xray-tui-proto/src/proto_spec/mod.rs` | `CoreType` gains `toasty::Embed`; `InjectToCoreConf` trait + `SupportError`; `dispatch!` extended; `ProtoSpec` split contract |
| `crates/xray-tui-proto/src/proto_spec/*.rs` (20 config files) | Config structs lose `host`/`port`; parse returns `ParsedProto`; `inject_to` impls |
| `crates/xray-tui-proto/src/clash/mod.rs` | unchanged (boundary), conversion signatures use endpoint essentials |
| `crates/xray-tui-db/src/models_toasty.rs` | full rewrite: embeds + 7 models |
| `crates/xray-tui-db/src/database.rs` | typed queries only; PRAGMAs raw; ~70 raw statements deleted |
| `crates/xray-tui-config/src/import_export.rs` | typed parse/format; `Profile` legacy struct + helpers deleted |
| `crates/xray-tui-config/src/forms.rs` | typed form → config constructors |
| `crates/xray-tui-config/src/subscription.rs` | typed upsert of endpoints/protocols/links |
| `crates/xray-tui-core/src/config_builder/{xray,singbox,mod}.rs` | skeleton builders + `inject_to` dispatch; `parse_settings`/`to_settings` deleted |
| `crates/xray-tui/src/ops/{ping,events,profiles,enrich,connect,settings,subscriptions}.rs` | scheduler, batch→tasks, labels, typed state |
| `crates/xray-tui/src/ui/{profiles,settings,statistics,mod}.rs` | typed rows, jiff display, new settings |
| `crates/xray-tui/src/state.rs` | typed `EndpointRow` state |
| `crates/xray-tui-core/src/protocol.rs` | deleted (moved to proto `kinds.rs`; re-export shim during transition, removed in Task 29) |

---

## Phase A — Proto crate parse split

### Task 1: Add deps + kind enums

**Files:**
- Modify: `crates/xray-tui-proto/Cargo.toml`
- Create: `crates/xray-tui-proto/src/proto_spec/kinds.rs`
- Modify: `crates/xray-tui-proto/src/lib.rs`

**Interfaces:**
- Produces: `ProtocolKind`, `TransportType`, `SecurityType` (each `#[derive(toasty::Embed)]` + serde `Serialize`/`Deserialize` + `FromStr` + `Display`), re-exported from crate root.

- [ ] **Step 1: Add deps**

`crates/xray-tui-proto/Cargo.toml`:
```toml
toasty = { version = "0.9", default-features = false }
```
(also `serde`/`serde_json` — already present.)

- [ ] **Step 2: Write the kinds module with tests**

```rust
// crates/xray-tui-proto/src/proto_spec/kinds.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
pub enum ProtocolKind {
    Vmess, Vless, Trojan, Ss, Ssr, Tuic, Hysteria1, Hysteria2, Wireguard,
    Socks, Http, Naive, AnyTls, ShadowTls, Tor, Ssh, Tailscale,
    Redirect, TProxy, Mixed,
}
impl ProtocolKind {
    /// Canonical protocol string used in URLs and proto_kind columns.
    /// NOTE: explicit mapping — serde snake_case would emit any_tls/shadow_tls/t_proxy.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vmess => "vmess", Self::Vless => "vless", Self::Trojan => "trojan",
            Self::Ss => "ss", Self::Ssr => "ssr", Self::Tuic => "tuic",
            Self::Hysteria1 => "hysteria1", Self::Hysteria2 => "hysteria2",
            Self::Wireguard => "wireguard", Self::Socks => "socks", Self::Http => "http",
            Self::Naive => "naive", Self::AnyTls => "anytls", Self::ShadowTls => "shadowtls",
            Self::Tor => "tor", Self::Ssh => "ssh", Self::Tailscale => "tailscale",
            Self::Redirect => "redirect", Self::TProxy => "tproxy", Self::Mixed => "mixed",
        }
    }
}
impl std::fmt::Display for ProtocolKind { /* as_str */ }
impl std::str::FromStr for ProtocolKind { /* parse as_str, case-insensitive */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
#[serde(rename_all = "snake_case")]
pub enum TransportType { Tcp, Ws, Grpc, Http, HttpUpgrade, XHttp, Kcp }
impl TransportType { pub const fn as_str(self) -> &'static str { /* "tcp","ws","grpc","http","httpupgrade","xhttp","kcp" */ } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
#[serde(rename_all = "snake_case")]
pub enum SecurityType { None, Tls, Reality }
impl SecurityType { pub const fn as_str(self) -> &'static str { /* "none","tls","reality" */ } }
```
(These `as_str` outputs MUST equal the current `proto_kind` column values and
`TransportConfig::type_str()`/`SecurityConfig::type_str()` strings — verify
against `crates/xray-tui-core/src/protocol.rs` and `common.rs` while writing.)

- [ ] **Step 3: Tests** — for each enum: `FromStr`/`Display` round-trip all variants; serde JSON round-trip; `TransportType`/`SecurityType` values match `TransportConfig::type_str()` / `SecurityConfig::type_str()` outputs for the variants that exist today.
- [ ] **Step 4: Run** `cargo test -p xray-tui-proto` — PASS.
- [ ] **Step 5: Commit** `feat(proto): add typed kind enums (ProtocolKind/TransportType/SecurityType)`

### Task 2: CoreType embed derive + SupportError

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/mod.rs`
- Create: `crates/xray-tui-proto/src/proto_spec/error.rs`

**Interfaces:**
- Produces: `CoreType` (existing enum + `toasty::Embed`); `SupportError` enum with `Display`/`Error`.

- [ ] **Step 1: Add `toasty::Embed` derive to `CoreType`** (keep serde-less plain enum semantics; embed derive only).
- [ ] **Step 2: Create `error.rs`**

```rust
#[derive(Debug, thiserror::Error)]
pub enum SupportError {
    #[error("protocol {0} is not supported by core {1}")]
    UnsupportedProtocol(String, CoreType),
    #[error("config error: {0}")]
    Config(String),
    #[error("missing required field {0} for {1}")]
    MissingField(&'static str, &'static str),
}
```

- [ ] **Step 3: Test** — construct each variant, assert `to_string()`.
- [ ] **Step 4: Run** `cargo test -p xray-tui-proto` — PASS.
- [ ] **Step 5: Commit** `feat(proto): CoreType embed derive + SupportError`

### Task 3: Parse boundary types (`EndpointEssentials`, `ProtocolEssentials`, `ParsedProto`)

**Files:**
- Create: `crates/xray-tui-proto/src/proto_spec/endpoint.rs`

**Interfaces:**
- Produces:
```rust
pub struct EndpointEssentials {
    pub host: String,
    pub host_type: HostKind,          // enum { Ipv4, Ipv6, Dns, Undefined } — plain, in this file
    pub port: u16,                    // primary port
    pub ports: Vec<u16>,              // full port spec; empty when single-port
}
impl EndpointEssentials { pub fn new(host: String, port: u16) -> Self; }

pub struct ProtocolEssentials {
    pub proto_kind: ProtocolKind,
    pub config_type: ConfigKind,      // enum { ShareUrl, Form }
    pub core_type: CoreType,
    pub transport: TransportEssentials, // type + serialized config
    pub security: SecurityEssentials,
}
pub struct TransportEssentials { pub r#type: TransportType, pub config: TransportConfig }
pub struct SecurityEssentials { pub r#type: SecurityType, pub sni: Option<String>, pub fp: Option<String>, pub insecure: Option<bool>, pub config: SecurityConfig }

pub struct ParsedProto {
    pub endpoints: Vec<EndpointEssentials>,   // may be empty (encrypted config)
    pub protocol: ProtocolEssentials,
}
impl ParsedProto {
    pub fn uid(&self) -> i64;            // sig ^ cred_hash over protocol essentials only
    pub fn sig(&self) -> i64;
    pub fn cred_hash(&self) -> i64;
    pub fn first_endpoint(&self) -> Option<&EndpointEssentials>;
}
```
- Consumes: Task 1 kinds, Task 2 `CoreType`.

- [ ] **Step 1: Implement the module.** `uid()/sig()/cred_hash()` compute over the serde-JSON of `ProtocolEssentials` (protocol fields only — no host/port anywhere in it by construction). Use the existing identity hashing algorithm from `ProtoIdentity::compute_sig`/`compute_cred_hash` (move those fns here or call through `ProtoIdentity`).
- [ ] **Step 2: Tests** — (a) two `ParsedProto` with identical protocol but different `endpoints` produce equal `uid`; (b) different protocol → different `uid`; (c) `uid` never zero; (d) `first_endpoint` None when `endpoints` empty.
- [ ] **Step 3: Run** `cargo test -p xray-tui-proto` — PASS.
- [ ] **Step 4: Commit** `feat(proto): parse boundary types (endpoint/protocol essentials, uid sans host/port)`

### Task 4: Strip host/port from config structs (Vless + Vmess first, as pattern)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/vless.rs`, `vmess.rs`, `common.rs`

**Interfaces:**
- Changes: `VlessConfig`/`VmessConfig` lose `host: HostSpec` and `port: PortSpec` fields.
- Produces (pattern for Task 5):
```rust
impl VlessConfig {
    pub fn try_parse(raw: &str) -> Result<ParsedProto, ParseError>;   // signature change
    pub fn format_share_url(&self, endpoint: &EndpointEssentials) -> String;
    pub fn endpoint_essentials(&self, raw: &str) -> Result<EndpointEssentials, ParseError>;
}
```
- `SecurityConfig`/`TransportConfig` unchanged (they never held host/port).

- [ ] **Step 1: Read** `vless.rs` + `vmess.rs` fully. Identify every place `host`/`port` is read: parse (URL host:port → `EndpointEssentials`), format (re-inject from endpoint), clash conversion (`clash_server_to_host`/`port_spec_first`), `to_settings` (streamSettings `network` etc. — unaffected).
- [ ] **Step 2: Update `try_parse`** to return `ParsedProto { endpoints: vec![EndpointEssentials::new(host, port)], protocol: ... }` where the protocol part is the config WITHOUT host/port. URL userinfo/path params that encode host/port (e.g. `?sni=`, `&host=`) stay in the config.
- [ ] **Step 3: Update `format_share_url`** signature to take `&EndpointEssentials` and rebuild the URL.
- [ ] **Step 4: Fix clash conversion call sites** in `common.rs` (`clash_server_to_host` etc.) to take endpoint essentials.
- [ ] **Step 5: Update all existing tests** in both files to the new signatures; add test: parse → `ParsedProto`, assert endpoint host/port extracted, assert protocol JSON serialization contains no `"host"`/`"port"` key at top level.
- [ ] **Step 6: Run** `cargo test -p xray-tui-proto` — PASS.
- [ ] **Step 7: Commit** `refactor(proto): strip host/port from Vless/Vmess configs (pattern)`

### Task 5: Strip host/port from remaining 18 configs

**Files:** all of `crates/xray-tui-proto/src/proto_spec/{trojan,ss,ssr,tuic,hysteria1,hysteria2,wireguard,socks,http_client,naive,anytls,shadowtls,tor,ssh,tailscale}.rs` + `PlaceholderConfig` in `mod.rs`.

**Interfaces:** same pattern as Task 4, applied per protocol. `PlaceholderConfig` (Redirect/TProxy/Mixed) keeps its raw `settings_json` and produces `ParsedProto { endpoints: vec![], protocol: ... }` with `proto_kind` from the form selection.

- [ ] **Step 1:** Apply the Task 4 pattern to Trojan, Ss, Ssr, Tuic (4 protocols). Run `cargo test -p xray-tui-proto` — PASS. Commit `refactor(proto): strip host/port from trojan/ss/ssr/tuic`.
- [ ] **Step 2:** Apply to Hysteria1, Hysteria2, Wireguard, Socks, Http (5). Test + commit `refactor(proto): strip host/port from hy1/hy2/wg/socks/http`.
- [ ] **Step 3:** Apply to Naive, AnyTls, ShadowTls, Tor, Ssh, Tailscale (6). Test + commit `refactor(proto): strip host/port from naive/anytls/shadowtls/tor/ssh/tailscale`.
- [ ] **Step 4:** Update `PlaceholderConfig::try_parse` to empty-endpoints `ParsedProto`; `ProtocolConfig::try_parse` dispatch updated. Test + commit `refactor(proto): placeholder configs emit endpoint-less ParsedProto`.

### Task 6: `InjectToCoreConf` trait + dispatch

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/mod.rs`, `error.rs`

**Interfaces:**
- Produces:
```rust
pub trait InjectToCoreConf {
    fn inject_to(&self, core_conf: &mut serde_json::Value, core_type: CoreType)
        -> Result<(), SupportError>;
}
impl InjectToCoreConf for ProtocolConfig { /* dispatch! per variant */ }
```

- [ ] **Step 1: Write the trait + blanket dispatch** — `match self { ProtocolConfig::Vless(c) => c.inject_to(conf, ct), ... }` with `_ => Err(SupportError::UnsupportedProtocol(...))` for variants lacking impls yet.
- [ ] **Step 2: Test** — `ProtocolConfig::Vless(...).inject_to(&mut json!({}), CoreType::Xray)` returns `Err` for now (no impl yet) or `Ok` once Task 16 lands; keep the test asserting the dispatch routes to the right variant via a sentinel (implement `inject_to` for ONE protocol in Task 16; before that, assert `Err(UnsupportedProtocol)`).
- [ ] **Step 3: Run** `cargo test -p xray-tui-proto` — PASS.
- [ ] **Step 4: Commit** `feat(proto): InjectToCoreConf trait + ProtocolConfig dispatch`

---

## Phase B — db crate typed models

### Task 7: Deps + embed types

**Files:**
- Modify: `crates/xray-tui-db/Cargo.toml`, `crates/xray-tui-db/src/models_toasty.rs`
- Modify: `Cargo.toml` (workspace `[workspace.dependencies]` — add `jiff`)

**Interfaces:**
- Produces (all in `models_toasty.rs`, `pub`):
```rust
// newtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)] pub struct EndpointId(i64);
impl EndpointId { pub fn new(v: i64) -> Self { debug_assert!(v != 0); Self(v) } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)] pub struct ProtocolId(i64);
impl ProtocolId { pub fn new(v: i64) -> Self { debug_assert!(v != 0); Self(v) } }

// enums
pub enum HostType { Ipv4, Ipv6, Dns, Undefined }
pub enum ConfigType { ShareUrl, Form }
pub enum TaskKind { FastPing, RealPing, UdpPing, UdpTest, SpeedTest }
pub enum ProfileErr { Real, Fast, Name }
pub enum GroupStatus { Ok, Error, Never }
pub enum GroupCoreType { Auto, Xray, SingBox }
pub enum QueryStrategy { UseIp, UseIpv4, UseIpv6 }
pub enum ConvertTarget { Clash }

// structs
pub struct ErrorInfo { pub kind: ProfileErr, pub text: String }
pub struct TrafficStats { pub today_up: i64, pub today_down: i64, pub total_up: i64, pub total_down: i64 }
pub struct Transport { pub r#type: TransportType, pub data: Deferred<Json<TransportConfig>> }
pub struct Security { pub r#type: SecurityType, pub sni: Option<String>, pub fp: Option<String>, pub insecure: Option<bool>, pub data: Deferred<Json<SecurityConfig>> }
pub enum Latency { Real { #[shared(delay)] delay: i32, ip: Option<String> }, Fast { #[shared(delay)] delay: i32 } }
```
- Consumes: proto `ProtocolKind`/`TransportType`/`SecurityType`/`CoreType`/`TransportConfig`/`SecurityConfig`.

- [ ] **Step 1: Deps.** workspace root: `jiff = "0.2"` (or latest semver-compatible with toasty 0.9's jiff). `xray-tui-db/Cargo.toml`: `toasty = { version = "0.9", features = ["turso", "serde", "jiff"] }`, add `jiff = { workspace = true }`, `xray-tui-proto = { path = "../xray-tui-proto" }`.
- [ ] **Step 2: Write all embed types** above in `models_toasty.rs`, replacing the `DELAY_SOURCE_*` consts (deleted — provenance now lives in `Latency` variants).
- [ ] **Step 3: Tests** — `Latency` shared column: create a scratch model in tests with `latency: Option<Latency>`, insert `Real{delay: 42, ip}` then `Fast{delay: 99}`, read back both, assert column `latency_delay` holds both values and `latency_ip` is null for Fast. Transport/Security JSON round-trip through a scratch model.
- [ ] **Step 4: Run** `cargo test -p xray-tui-db` — PASS (old model code still compiles alongside new types; deletion happens in Task 8).
- [ ] **Step 5: Commit** `feat(db): toasty jiff feature + typed embed types`

### Task 8: Rewrite models (7 tables)

**Files:**
- Modify: `crates/xray-tui-db/src/models_toasty.rs` (delete old models, add new)

**Interfaces:**
- Produces (exact models from spec §4 — `Endpoint`, `Protocol`, `ProfileStats`, `EndpointGroup` composite-keyed, `Group`, `RoutingRule`, `DnsSetting`). `EndpointRow` DTO survives with new shape:
```rust
pub struct EndpointRow {
    pub endpoint: Endpoint,
    pub links: Vec<ProfileStats>,
    pub protocols: HashMap<ProtocolId, Protocol>,   // included via links
    pub selected_protocol: usize,
    pub expanded: bool,
}
impl EndpointRow {
    pub fn active_protocol(&self) -> Option<(&ProfileStats, &Protocol)>;
    pub fn sort_links_by_test_priority(&mut self, dns_unresolved: bool);
    pub fn best_test_priority_key(&self, dns_unresolved: bool) -> Option<(u8, i32, i64, i64)>;
}
```
- Deletes: `ProtocolRow`, `ProfileExtension`, `ServerStat`, `PingSession`, `PingResultUpdate`, `DELAY_SOURCE_*`.

- [ ] **Step 1: Write the 7 models** per spec §4 verbatim (column-for-column). Keep `#[table = "..."]` names: `endpoints`, `protocols`, `profile_stats`, `endpoint_groups`, `groups`, `routing_rules`, `dns_settings`.
- [ ] **Step 2: Rewrite `EndpointRow`** DTO + sort/tier logic. Tier key now reads `(latency, error.kind, resolved_as)`:
  - `tier 0` real-ok (`latency is Real`), `1` fast-ok (`latency is Fast`), `2` untested (no latency, no error), `3` real-err (`error.kind == Real`), `4` fast-err (`error.kind == Fast`), `5` dns-unresolved (empty `resolved_as`).
  - Sort key `(tier, delay, -last_seen_at_epoch, protocol_id)`; latency orders only tiers 0-1.
- [ ] **Step 3: Delete** old models + `PingResultUpdate`. Delete the old unit tests that reference them; keep the sort/tier tests, adapted to `ProfileStats` rows.
- [ ] **Step 4: Do NOT commit a broken state.** `database.rs` still references the deleted types until Task 9 lands. Apply Task 8 edits, then immediately apply Task 9 edits, then run `cargo test -p xray-tui-db` and commit once.
- [ ] **Step 5: Commit** (together with Task 9) `refactor(db): typed 7-table model set + EndpointRow DTO + typed reads`

### Task 9: database.rs typed rewrite (reads)

**Files:**
- Modify: `crates/xray-tui-db/src/database.rs`

**Interfaces:**
- Produces (public API — signatures the rest of the workspace calls):
```rust
pub async fn get_endpoints(&self, filter: &EndpointFilter, page: PageSpec) -> Result<Vec<EndpointRow>>;
pub async fn get_endpoint_by_id(&self, id: EndpointId) -> Result<Option<EndpointRow>>;
pub async fn get_children_endpoints(&self, parent: EndpointId) -> Result<Vec<Endpoint>>;
pub async fn get_groups(&self) -> Result<Vec<Group>>;
pub async fn get_group_endpoint_ids(&self, group_id: &str) -> Result<Vec<EndpointId>>;
pub async fn get_endpoint_count(&self, filter: &EndpointFilter) -> Result<usize>;
pub struct EndpointFilter { pub group_id: Option<String>, pub graveyard: PurgatoryView, pub search: Option<String> }
pub struct PageSpec { pub limit: usize, pub offset: usize }
```

- [ ] **Step 1: Delete** all `toasty::sql::query`/`statement` calls except PRAGMAs; delete `deserialize_endpoint_rows`, `deserialize_ping_sessions`, all `get_*` value extractors, `SCHEMA_VERSION` machinery, `ensure_column`.
- [ ] **Step 2: Implement typed reads:**
  - `get_endpoints`: `Endpoint::filter(/* group via links → group_links, graveyard via links last_seen, search via host */).include(endpoint.links()).include(endpoint.group_links()).sort(...).limit/offset`. Then for the page's endpoints, one batched `ProfileStats::filter(ProfileStats::fields().endpoint_id().in_list(ids)).include(profile_stats.protocol()).include(profile_stats.endpoint())` — assemble `EndpointRow` in memory. No N+1.
  - Group membership filter: fetch `group_links` for `group_id`, filter endpoint ids by the set (or use `Endpoint::fields().group_links().any(...)` if the relation path supports it — verify; fallback to set filtering).
  - Graveyard: `PurgatoryView::{Active,Stale,All}` → last_seen threshold per existing semantics.
- [ ] **Step 3: Port `Database::open`**: remove `schema_needed`/migrations; `db.push_schema()` + PRAGMAs (`journal_mode=WAL`, `busy_timeout=5000`, `foreign_keys=ON`) via raw `toasty::sql::query`; keep `conn()` helper with busy_timeout.
- [ ] **Step 4: Tests** — in-memory DB: create endpoint+protocol+link, `get_endpoints` returns assembled `EndpointRow` with `links`+`protocols`; filter by group; graveyard; pagination.
- [ ] **Step 5: Run** `cargo test -p xray-tui-db` — PASS (workspace crates that used old API now fail to compile; that is expected until Phase C/D/E — verify with `cargo test -p xray-tui-db` only).
- [ ] **Step 6: Commit** `refactor(db): typed read paths (includes, filters, pagination)`

### Task 10: database.rs typed writes + purge + upserts

**Files:**
- Modify: `crates/xray-tui-db/src/database.rs`

**Interfaces:**
- Produces:
```rust
pub async fn upsert_endpoint(&self, e: &Endpoint) -> Result<()>;
pub async fn upsert_protocol(&self, p: &Protocol) -> Result<()>;
pub async fn upsert_link(&self, s: &ProfileStats) -> Result<()>;          // (protocol_id, endpoint_id) key
pub async fn upsert_endpoint_group_link(&self, eg: &EndpointGroup) -> Result<()>;
pub async fn update_last_used(&self, protocol_id: ProtocolId, endpoint_id: EndpointId, ts: jiff::Timestamp) -> Result<()>;
pub async fn update_endpoint_resolution(&self, endpoint_id: EndpointId, ips: Vec<String>, at: jiff::Timestamp) -> Result<()>;
pub async fn set_manual_override(&self, endpoint_id: EndpointId, protocol_id: Option<ProtocolId>) -> Result<()>;
pub async fn purge_expired(&self, cutoff: jiff::Timestamp) -> Result<usize>;
pub async fn delete_endpoint(&self, endpoint_id: EndpointId) -> Result<()>;          // cascades links, group_links
pub async fn clear_all_stats(&self) -> Result<()>;
pub async fn clear_group_endpoints(&self, group_id: &str) -> Result<()>;
pub async fn delete_group(&self, group_id: &str) -> Result<()>;
pub async fn upsert_routing_rules(&self, rules: &[RoutingRule]) -> Result<()>;
pub async fn upsert_dns_setting(&self, d: &DnsSetting) -> Result<()>;
pub async fn update_scheduler_state(&self, protocol_id: ProtocolId, endpoint_id: EndpointId, task_id: Option<u16>, queue: &[u16]) -> Result<()>;
```

- [ ] **Step 1: Implement upserts** via `upsert_by_*` generated builders (composite-key upserts for `ProfileStats` = `upsert_by_protocol_id_and_endpoint_id`, `EndpointGroup` = `upsert_by_endpoint_id_and_group_id`).
- [ ] **Step 2: `purge_expired`** — `let expired: Vec<EndpointId> = Endpoint::filter(Endpoint::fields().links().all(ProfileStats::fields().last_seen_at().lt(cutoff))).exec(...)` (verify `.all()` relation filter compiles; fallback: fetch endpoints with no link newer than cutoff via two typed queries). Then in one transaction: delete links/group_links/routing for those ids, delete endpoints. Return count.
- [ ] **Step 3: `update_scheduler_state`** — read row (or use `upsert`), set `task_id` + replace `task_queue` whole-vec, `#[version]`-guarded instance update; on `Error::condition_failed`, reload + retry (reuse `retry_on_busy` helper).
- [ ] **Step 4: `clear_all_stats`** — query-based update zeroing `traffic` + clearing `latency`/`error`/`speed_bps` on all `ProfileStats`.
- [ ] **Step 5: Tests** — upsert idempotency (same key twice → one row); purge removes only expired endpoints; scheduler-state OCC conflict (stale handle update fails).
- [ ] **Step 6: Run** `cargo test -p xray-tui-db` — PASS.
- [ ] **Step 7: Commit** `refactor(db): typed writes, purge, OCC scheduler state`

---

## Phase C — config crate boundary

### Task 11: import_export typed parse (kill Profile/legacy)

**Files:**
- Modify: `crates/xray-tui-config/src/import_export.rs`, `crates/xray-tui-config/src/lib.rs`

**Interfaces:**
- Produces:
```rust
pub fn parse_share_url(url: &str, settings: &ValidationSettings)
    -> Result<ParsedProfile, ImportError>;
pub struct ParsedProfile {
    pub parsed: ParsedProto,          // proto crate type
    pub validation: Result<(), String>, // per-profile validation result
}
pub fn format_share_url(parsed: &ParsedProto, endpoint: &EndpointEssentials) -> Result<String, ImportError>;
```
- Deletes: `Profile`, `profile_config`, `encode_profile_spec`, `from_legacy_parse`, `convert_spec_blob`, `extract_spec_json`, legacy JSON-blob fallbacks, `ProfileInput`/`ProfileRow` bridge structs.

- [ ] **Step 1: Read** `import_export.rs` fully. Replace the `Profile`-returning `parse_share_url` with the `ParsedProto`-returning one. Move the 14-protocol fallback chain to `ProtocolConfig::try_parse` (proto crate now owns it).
- [ ] **Step 2: Delete** all legacy helpers listed above; `validate_required_fields`/`profile_user_id`/`flatten_json_to_fields` port to typed config accessors (read fields off the typed structs, not `to_settings`).
- [ ] **Step 3: Tests** — update all parse/format tests to the new API; add: parse → `ParsedProto`, endpoints extracted; `format_share_url(parsed, endpoint)` round-trips.
- [ ] **Step 4: Run** `cargo test -p xray-tui-config` — PASS.
- [ ] **Step 5: Commit** `refactor(config): typed parse boundary, Profile/legacy blob machinery deleted`

### Task 12: forms + subscription typed

**Files:**
- Modify: `crates/xray-tui-config/src/forms.rs`, `crates/xray-tui-config/src/subscription.rs`

**Interfaces:**
- Produces: `forms.rs`: `fn build_typed_config(fields: &[FormValue]) -> Result<ParsedProto, String>` (constructs the typed config directly from form fields — replaces `encode_profile_spec` path); `subscription.rs`: `upsert_subscription(endpoints: Vec<EndpointEssentials>, protocols: Vec<ProtocolEssentials>, group_id: &str)`.

- [ ] **Step 1: forms.rs** — port field→config construction to typed structs (map form `protocol_settings`/`stream_settings` JSON keys onto typed config fields; unknown keys → `SupportError::Config`). Remove `encode_profile_spec` usage.
- [ ] **Step 2: subscription.rs** — streaming decode + URL split unchanged; parse each line via new `parse_share_url`; dedup by `(endpoint natural key, uid)`; call db upserts. Remove `spec_blob` handling and the legacy `allow_insecure` JSON check (reads `security.insecure` now).
- [ ] **Step 3: Tests** — form→typed config for one representative protocol (VMess); subscription parse dedups identical protocol across two endpoints to one `Protocol` uid.
- [ ] **Step 4: Run** `cargo test -p xray-tui-config` — PASS.
- [ ] **Step 5: Commit** `refactor(config): typed form/subscription paths`

---

## Phase D — core crate builders

### Task 13: Delete parse_settings/to_settings; skeleton builders

**Files:**
- Modify: `crates/xray-tui-core/src/config_builder/mod.rs`, `xray.rs`, `singbox.rs`
- Modify: `crates/xray-tui-core/src/lib.rs` (connect_to_profile call sites)

**Interfaces:**
- Consumes: Task 6 `InjectToCoreConf`, Task 11 typed parse.
- Produces:
```rust
pub fn build(endpoint: &Endpoint, link: &ProfileStats, protocol: &Protocol,
             params: &BuildParams, routing: &[RoutingRule], dns: &DnsSetting)
    -> Result<BackendConfig, BuildError>;
```
- Deletes: `parse_settings`, `to_settings`, `p_settings`/`s_settings` reads, `set_protocol_settings_json`/`set_stream_settings_json` test helpers.

- [ ] **Step 1: mod.rs** — delete `parse_settings`; `BuildError` extended with `SupportError` passthrough (`#[error(transparent)] Support(#[from] SupportError)`).
- [ ] **Step 2: xray.rs** — keep skeleton construction (log, stats/api, inbounds, routing, dns, policy, mux/freedom/blackhole outbounds, tag naming); delete all `parse_settings`-driven field extraction from the outbound build; call `protocol.config().inject_to(&mut conf, CoreType::Xray)` for the outbound + streamSettings block.
- [ ] **Step 3: singbox.rs** — same for sing-box skeleton + `inject_to(..., CoreType::SingBox)`.
- [ ] **Step 4: Tests** — existing builder tests rewritten: build via typed protocol configs; the reality-without-publicKey rejection test now asserts `SupportError`; cipher whitelist tests assert `SupportError::Config`.
- [ ] **Step 5: Run** `cargo test -p xray-tui-core` — FAILS until Task 16 provides `inject_to` impls (expected). Land the skeleton with a temporary `ProtocolConfig::inject_to` returning `Err(UnsupportedProtocol)` for all — tests assert the error; commit.
- [ ] **Step 6: Commit** `refactor(core): builder skeletons consume InjectToCoreConf`

### Task 14: Per-protocol `inject_to` — xray core shape

**Files:**
- Modify: all protocol config files in `crates/xray-tui-proto/src/proto_spec/` (impl blocks)

**Interfaces:**
- Consumes: Task 6 trait; `common::to_xray_stream_settings(security, transport) -> Option<Value>` (exists).
- Produces: per-config `inject_to(&self, conf: &mut Value, CoreType::Xray)` writing:
  - `conf["outbounds"]` entry: `{ tag: "proxy", protocol: <xray protocol name>, settings: {...}, streamSettings: to_xray_stream_settings(...) }`
  - xray protocol name map: vmess/vless/trojan/ss/socks/http/wireguard/hysteria2 → xray names; sing-box-only protocols return `Err(UnsupportedProtocol)` for Xray.

- [ ] **Step 1: Vless + Vmess `inject_to` (xray shape)** — port the existing outbound construction from `xray.rs` into `vless.rs`/`vmess.rs` impls (settings: vnext/users, streamSettings via common helper). Test: build `json!({})` → inject → assert outbound JSON matches the old `xray.rs` output for the same config (golden test). Commit `feat(proto): xray inject_to for vmess/vless`.
- [ ] **Step 2: Trojan, Ss, Socks, Http, Wireguard, Hysteria2, Vless-xtls flow** (xray-native set) — same pattern; golden tests against old builder outputs. Commit `feat(proto): xray inject_to for trojan/ss/socks/http/wg/hy2`.
- [ ] **Step 3: Sing-box-only protocols** (TUIC, Hysteria1, Naive, AnyTls, ShadowTls, Tor, Ssh, Tailscale, Ssr, Redirect, TProxy, Mixed) — `inject_to` returns `Err(UnsupportedProtocol)` for Xray. Test each. Commit `feat(proto): xray inject_to rejects sing-box-only protocols`.

### Task 15: Per-protocol `inject_to` — sing-box core shape

**Files:** same as Task 14.

**Interfaces:**
- Produces: per-config `inject_to(&self, conf, CoreType::SingBox)` writing `conf["outbounds"]` entry `{ type: <singbox type>, tag: "proxy", ... }` with sing-box field names; xray-only protocols → `Err(UnsupportedProtocol)`.

- [ ] **Step 1: sing-box shape for the sing-box-native set** (TUIC, Hysteria1, Naive, AnyTls, ShadowTls, Tor, Ssh, Tailscale, Ssr, Redirect, TProxy, Mixed) — port from `singbox.rs`; golden tests. Commit `feat(proto): sing-box inject_to (native set)`.
- [ ] **Step 2: sing-box shape for shared protocols** (Vless, Vmess, Trojan, Ss, Socks, Http, Wireguard, Hysteria2) — port from `singbox.rs`; golden tests. Commit `feat(proto): sing-box inject_to (shared set)`.
- [ ] **Step 3: cipher whitelist enforcement** moves into `Ss::inject_to` (both cores): `XRAY_SS_METHODS`/`SINGBOX_SS_METHODS` checks → `Err(SupportError::Config(...))`. Commit `feat(proto): ss cipher whitelist enforced in inject_to`.

### Task 16: Wire builders fully; delete legacy

**Files:**
- Modify: `crates/xray-tui-core/src/config_builder/xray.rs`, `singbox.rs`, `mod.rs`

- [ ] **Step 1:** Remove the temporary `Err(UnsupportedProtocol)` fallback; make `ProtocolConfig::inject_to` the only outbound path; delete leftover `to_settings`-style extraction in both builders.
- [ ] **Step 2:** Run `cargo test -p xray-tui-core` — all builder tests pass with typed configs. `cargo test -p xray-tui-proto` — golden tests pass.
- [ ] **Step 3: Commit** `refactor(core): builders fully InjectToCoreConf-driven; legacy settings JSON removed`

---

## Phase E — TUI scheduler, ping pipeline, labels

### Task 17: Typed state (EndpointRow) + call-site sweep

**Files:**
- Modify: `crates/xray-tui/src/state.rs`, `crates/xray-tui/src/lib.rs`, `crates/xray-tui/src/ops/{profiles,connect,enrich,events}.rs`, `crates/xray-tui/src/ui/{profiles,statistics,mod}.rs`, `crates/xray-tui-core/src/ping/real/pool.rs`, `crates/xray-tui-db/src/tests/integration.rs`

**Interfaces:**
- Consumes: Task 9/10 db API, Task 11 parse API.
- `AppState.endpoints: Vec<EndpointRow>` (new DTO shape).

- [ ] **Step 1:** Update `AppState`/`reload_profiles` to the new `EndpointRow`; delete `Profile`-typed fields (spec_blob, core_type strings).
- [ ] **Step 2:** Sweep every `spec_blob`/`Profile` reference in TUI + `pool.rs` + `integration.rs`: profile construction sites now build `Protocol`/`ProfileStats` rows (config from typed parse, link fields set).
- [ ] **Step 3: Run** `cargo check --workspace` — clean.
- [ ] **Step 4: Commit** `refactor(tui): typed EndpointRow state, spec_blob references gone`

### Task 18: TaskScheduler core

**Files:**
- Create: `crates/xray-tui/src/ops/scheduler.rs`

**Interfaces:**
- Produces:
```rust
pub struct TaskScheduler {
    tasks: DashMap<u16, TaskKind>,
    next_id: AtomicU16,                       // wraps, skips 0
    queue_limit: u16,                          // from settings
    dns_defer_secs: i64,                       // from settings
    dns_failures: DashMap<EndpointId, jiff::Timestamp>,  // recent DNS failures
}
impl TaskScheduler {
    pub fn new(queue_limit: u16, dns_defer_secs: i64) -> Self;
    pub fn schedule(&self, link: &ProfileStats, kind: TaskKind, db: &Arc<Database>, tx: &mpsc::Sender<CoreEvent>) -> ScheduleOutcome;
    pub fn complete(&self, link: &ProfileStats, kind: TaskKind, result: PingOutcome, db: &Arc<Database>, tx: &mpsc::Sender<CoreEvent>);
    pub fn mark_dns_failure(&self, endpoint: EndpointId);
    pub fn sweep_orphans(&self, link: &ProfileStats, db: &Arc<Database>);
    fn alloc_id(&self) -> u16;                 // non-zero, not live
}
pub enum ScheduleOutcome { Started(u16), Queued(u16), QueueFull, DnsDeferred }
```

- [ ] **Step 1: Implement** the gate per spec §6.2: `task_id == None` → alloc + fire; live → queue (limit-checked, `QueueFull` → warn log); orphan → replace + wipe orphan queue ids; `complete` pops FIFO, fires next. `alloc_id` loops `AtomicU16` until non-zero and absent from `tasks`.
- [ ] **Step 2: DNS deferral** — `schedule` returns `DnsDeferred` when `mark_dns_failure` recorded the endpoint within `dns_defer_secs`; caller re-schedules after the delay via `tokio::time::sleep`.
- [ ] **Step 3: Tests** — pure-logic tests with a mock `db` seam (trait `SchedulerDb { fn read_link; fn write_task_state; }` implemented by `Database`): gate start/queue/full/orphan-sweep/complete-pop; id allocator never returns 0 or a live id; DNS deferral window.
- [ ] **Step 4: Run** `cargo test -p xray-tui` (scheduler tests only via `--lib scheduler`) — PASS.
- [ ] **Step 5: Commit** `feat(tui): TaskScheduler gate/queue/orphan-sweep/DNS-deferral`

### Task 19: Batch pipeline → tasks

**Files:**
- Modify: `crates/xray-tui/src/ops/ping.rs`

**Interfaces:**
- Consumes: Task 18 scheduler.
- Deletes: `create_ping_batch`/`get_batch_for_real_ping`/`cancel_stranded_real_pings` db calls, wave/occurrence loops, `PingSession` handling.

- [ ] **Step 1: Rework `start_batch_ping`** — phase 1: for each visible profile, `scheduler.schedule(link, FastPing, ...)`; `DnsDeferred` profiles collected and re-scheduled after the configured delay. Phase 2: after phase-1 completion (per-profile), `schedule(link, RealPing, ...)`.
- [ ] **Step 2: Probe dedup** — keep `fast_cache` (one TCP ping per address:port); keep per-core group dispatch (`run_page_pings` → the engine functions stay, they now key results by link id).
- [ ] **Step 3: Sibling cancel** — `dedup_endpoints=true`: when a `RealPing` completes successfully for one protocol of an endpoint, remove queued `RealPing` ids of sibling links (via `complete` + queue rewrite); keep the `real_ping_test_all_protocols` flag semantics.
- [ ] **Step 4: Progress** — in-memory `BatchState { total, done }` per batch id; emit `BatchProgress` events.
- [ ] **Step 5: Tests** — unit: batch of 3 profiles schedules 3 fast tasks then 3 real tasks; sibling-cancel on success with `dedup_endpoints`; all-fail → `error.kind = Real` set on all links.
- [ ] **Step 6: Run** `cargo test -p xray-tui --lib ops::ping` — PASS.
- [ ] **Step 7: Commit** `feat(tui): batch ping pipeline driven by TaskScheduler`

### Task 20: Labels + tiers from persisted rows; error TTL

**Files:**
- Modify: `crates/xray-tui/src/ui/profiles.rs`, `crates/xray-tui/src/ops/events.rs`, `crates/xray-tui/src/ops/ping.rs`

- [ ] **Step 1: Labels** — render `[real]` when all links of the endpoint have `error.kind == ProfileErr::Real`; `[fast]` analog; `[name]` when `resolved_as` empty. Remove `ping_status` round-map label logic.
- [ ] **Step 2: Re-sort triggers** — on `SpeedTestResult`, re-sort the endpoint's links by the new tier key (Task 8); invalidate `filter_cache_valid`; remap `selected_sub` by protocol id.
- [ ] **Step 3: Error TTL** — settings `error_ttl_hours`; background sweep (or on-load check) clears `error` where `updated_at` older than TTL. Default unset = never clear.
- [ ] **Step 4: Tests** — label logic pure fn: endpoint with 2 links both `error.kind=Real` → `[real]`; one ok one err → no label. TTL expiry clears.
- [ ] **Step 5: Commit** `feat(tui): persisted error labels + TTL`

### Task 21: New settings + jiff display

**Files:**
- Modify: `crates/xray-tui-config/src/app_config.rs`, `crates/xray-tui/src/ui/settings.rs`, `crates/xray-tui/src/types.rs`, `crates/xray-tui/src/ui/statistics.rs`, `crates/xray-tui/src/ops/events.rs`

- [ ] **Step 1: Settings** — Speed Test section gains: `task_queue_limit` (u16, default 3, 0 = no queueing), `error_ttl_hours` (Option<i64>, default None), `dns_failure_defer_secs` (i64, default 5). Wire into `SpeedTestConfig`/app config + settings form (number inputs) + scheduler construction.
- [ ] **Step 2: `format_ts`** — `i64` unix → `jiff::Timestamp`; port all call sites (`ui/profiles.rs`, `ui/statistics.rs`, logs) to jiff display.
- [ ] **Step 3: Traffic reset** — `ServerStat.last_updated` string-date logic → jiff day comparison on `ProfileStats.updated_at` (reset `today_up`/`today_down` when day changes).
- [ ] **Step 4: Commit** `feat(tui): task queue settings, jiff timestamps, daily traffic reset`

### Task 22: connect/disconnect + stats plumbing

**Files:**
- Modify: `crates/xray-tui/src/ops/connect.rs`, `crates/xray-tui-core/src/lib.rs` (connect_to_profile), `crates/xray-tui/src/ops/events.rs`, `crates/xray-tui/src/ui/statistics.rs`

- [ ] **Step 1:** `connect_to_profile` consumes `(&Endpoint, &ProfileStats, &Protocol)`; `last_used_at`/`last_seen_at` updates via typed db calls (Task 10).
- [ ] **Step 2:** traffic stats gRPC poller writes `ProfileStats.traffic` (today/total) + updates `updated_at`; statistics screen reads typed rows.
- [ ] **Step 3: Commit** `refactor(tui): connect/stats on typed rows`

---

## Phase F — cleanup and verification

### Task 23: Delete dead code + shims

**Files:**
- Modify: `crates/xray-tui-core/src/protocol.rs` (delete), `crates/xray-tui-core/src/lib.rs` (re-export removal), `crates/xray-tui-db/src/retry.rs` (keep), `crates/xray-tui/src/lib.rs`

- [ ] **Step 1:** Delete `protocol.rs`; remove re-export shims; `grep` for `spec_blob`, `PingSession`, `ProfileExtension`, `ServerStat`, `ProtocolRow`, `to_settings`, `parse_settings`, `DELAY_SOURCE` — zero hits in `crates/`.
- [ ] **Step 2: Commit** `chore: remove legacy model shims`

### Task 24: db integration tests rewritten typed

**Files:**
- Modify: `crates/xray-tui-db/src/tests/integration.rs`, `crates/xray-tui-db/src/database.rs` tests

- [ ] **Step 1:** Port every integration test (schema creation, index creation, endpoint/protocol/ext/stats inserts, purge, group ops) to the typed API and new tables.
- [ ] **Step 2: Commit** `test(db): typed integration tests`

### Task 25: Workspace verification

- [ ] **Step 1:** `cargo fmt --all` then `cargo clippy --workspace` — fix all warnings (pedantic/nursery at warn).
- [ ] **Step 2:** `cargo test --workspace` — all green.
- [ ] **Step 3:** `cargo build --release`.
- [ ] **Step 4: Commit** `chore: workspace verification`

### Task 26: Manual smoke test

- [ ] **Step 1:** Fresh DB (`rm -f ~/.local/share/xray-tui/*.db` per current data dir) → `cargo run`: import a subscription (both xray + sing-box protocols), verify profiles list renders typed fields (Type flag, address, test column, labels).
- [ ] **Step 2:** Connect VMess (xray-core) + TUIC (sing-box); verify stats (traffic up/down) and logs.
- [ ] **Step 3:** Fast ping + real ping batch on 5+ profiles: verify progress, `[fast]`/`[real]` labels on failures, queue-limit warning when limit=1, DNS-deferral for a bogus host.
- [ ] **Step 4:** Restart app: verify error labels persist; set `error_ttl_hours=1` and verify old errors cleared.
- [ ] **Step 5:** Commit nothing (manual). Record findings.

### Task 27: Docs

- [ ] **Step 1:** Update `AGENTS.md` (models list, decision entries for ProfileStats/task gate/InjectToCoreConf/zero-raw-SQL; remove PingSession/ProfileExtension/ServerStat/spec_blob references) and `TUI_MANUAL.md` (new settings).
- [ ] **Step 2: Commit** `docs: typed data model updates to AGENTS.md/TUI_MANUAL.md`

---

## Self-Review Notes

- Spec §2 (parse split) → Tasks 3-5. Spec §3 (deps) → Tasks 1, 7. Spec §4 (models) → Tasks 7-8. Spec §5 (config storage) → Tasks 7-8 (embeds). Spec §6 (scheduler) → Tasks 18-21. Spec §7 (builders) → Tasks 6, 13-16. Spec §8 (zero raw SQL) → Tasks 9-10. Spec §9 (boundary cleanup) → Tasks 11-12, 17, 23. Spec §10 (scope) → Task 27. Spec §11 (verification) → Tasks 24-26.
- Type consistency: `ProtocolId`/`EndpointId` newtypes used everywhere a key is passed (Tasks 8-10, 17-22); `ParsedProto` is the single parse return type (Tasks 3-5, 11-12); `ProfileStats` is the only per-pair state carrier (Tasks 8-10, 18-22). `schedule` returns `ScheduleOutcome` (Task 18) consumed by `ping.rs` (Task 19).
- Known compile breaks between phases are intentional (bottom-up order); each phase's final task restores `cargo test` for its crate. Workspace-wide green only at Task 25.
