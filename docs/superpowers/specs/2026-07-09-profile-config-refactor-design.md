# Profile Config Storage Architecture — Implementation Spec

**Date:** 2026-07-09
**Source:** `docs/refactor-model.md` (draft, refined through design review)

---

## Overview

Refactor the profile config storage layer from flat `Profile` + `Connection` tables to normalized `Endpoint` + `ProtocolRow` + `EndpointGroup` hierarchy. One endpoint per (host, port) — multiple protocol configs share one endpoint. Purgatory becomes `ProtocolRow.last_seen_at < NOW - TTL` filter, not data movement.

## Motivation

- Current schema: one Profile row per config, even when 10 configs share the same host:port. No dedup.
- Connection table adds complexity (per-group active profile) without real benefit.
- Subscription model is separate from Group; fields overlap.
- Purgatory requires explicit data moves (insert to graveyard, delete from live).
- No DNS resolution tracking — resolved IPs are ephemeral.

## Design Decisions

1. **Pre-alpha DB migration**: Delete old DB file on schema version mismatch. `push_schema()` creates fresh tables. No migration chain, no version tracking.
2. **Subscription refresh = UPDATE, never DELETE**: Refresh updates `ProtocolRow.last_seen_at` on active configs. Configs not in the refresh response keep old `last_seen_at` and age into Purgatory naturally. Endpoint staleness is derived from `MAX(protocol.last_seen_at)`.
3. **Purgatory = filter toggle** on Profiles tab: three-way toggle — Active / Stale / All. No separate Purgatory tab.
4. **Manual protocol override**: `Endpoint.manual_protocol_override` FK to `ProtocolRow.id`. Safe because protocol rows are never deleted — only age out.
5. **No system groups, no `is_system` flag**. Groups are pure source tracking.
6. **All config types have `ProtoSpec`** producing `sig` and `cred_hash` for uid derivation.
7. **Protocol auto-select** by `protocol_security_rank()` (hardcoded). Sort settings UI deferred to follow-up.
8. **DNS resolution** stored as child Endpoint rows with `parent_id`. No caching layer — `parent_id` IS the cache. Re-resolve on subscription refresh.

---

## Schema

### `Endpoint` — one row per (host, port)

```rust
#[derive(Debug, Clone, toasty::Model)]
pub struct Endpoint {
    #[key]
    pub id: i64,                       // stable_hash(host, port) for known types; stable_hash("undefined", config_uid) for exotic
    pub host: String,                  // canonical host string; empty for undefined
    pub host_type: String,             // "ipv4" | "ipv6" | "dns" | "undefined"
    pub port: i32,                     // primary port; 0 for undefined
    pub port_spec_str: Option<String>, // full PortSpec when multi-port
    pub parent_id: Option<i64>,        // resolved IP → DnsName parent
    pub last_source: Option<String>,   // hash of source subscription — plain string, not FK. Analytics only
    pub created_at: i64,
    pub manual_protocol_override: Option<i64>, // FK → protocols.id — user's manual protocol choice, NULL = auto-select best
}
```

### `ProtocolRow` — replaces `Profile`

```rust
#[derive(Debug, Clone, toasty::Model)]
pub struct ProtocolRow {
    #[key]
    pub id: i64,                  // = uid = sig ^ cred_hash (same as old Profile.id)
    pub endpoint_id: i64,         // FK → endpoints.id
    pub sig: i64,
    pub cred_hash: i64,
    pub proto_kind: String,
    pub spec_blob: Vec<u8>,
    pub config_type: i32,         // same semantics: 0 = share URL, 1 = form created
    pub core_type: String,
    pub transport: Option<String>,
    pub security: Option<String>,
    pub remarks: Option<String>,
    pub created_at: i64,
    pub last_seen_at: i64,        // NEW: per-config staleness tracking

    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,

    #[has_one]
    pub extension: Deferred<Option<ProfileExtension>>,
    #[has_one]
    pub server_stat: Deferred<Option<ServerStat>>,
}
```

### `EndpointGroup` — source provenance join

```rust
#[derive(Debug, Clone, toasty::Model)]
#[unique(endpoint_id, group_id)]
pub struct EndpointGroup {
    #[key]
    pub id: String,            // UUID
    pub endpoint_id: i64,      // → endpoints.id
    pub group_id: String,      // → groups.id
    pub last_seen_at: i64,     // per-source last confirmation
    pub sort_order: Option<i32>,

    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,
    #[belongs_to(key = group_id, references = id)]
    pub group: Deferred<Option<Group>>,
}
```

### `Group` — simplified, merges Subscription fields

```rust
#[derive(Debug, Clone, toasty::Model)]
pub struct Group {
    #[key]
    pub id: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<i32>,
    pub user_agent: Option<String>,
    pub convert_target: Option<i32>,
    pub core_type: Option<String>,
    pub sort_order: Option<i32>,
    pub last_refreshed: Option<String>,
    pub status: Option<String>,         // "ok" | "error" | "never"
    pub error_message: Option<String>,
    pub refresh_interval: Option<i32>,
}
```

### Unchanged models (FK renamed only)

| Model              | FK rename                    | Notes                                      |
| ------------------ | ---------------------------- | ------------------------------------------ |
| `ProfileExtension` | `profile_id` → `protocol_id` | Same i64 value                             |
| `ServerStat`       | `profile_id` → `protocol_id` | Same i64 value                             |
| `PingSession`      | `profile_id` → `protocol_id` | Same i64 value. All other fields unchanged |
| `PingResultUpdate` | `profile_id` → `protocol_id` | Same i64 value. All other fields unchanged |
| `RoutingRule`      | No change                    | Unrelated to profile refactor              |
| `DnsSetting`       | No change                    | Unrelated to profile refactor              |

### Dropped models

- `Profile` — replaced by `ProtocolRow` + `Endpoint`
- `Connection` — replaced by `EndpointGroup`
- `Subscription` — fields merged into `Group`

### New utility

```rust
// crates/xray-tui-db/src/hash.rs
pub fn stable_hash(left: impl Hash, right: impl Hash) -> i64
```

Uses `std::hash::DefaultHasher`. Exposed as `xray_tui_db::hash::stable_hash`. Imported in `xray-tui-config` for `parse_share_url` callers.

---

## Implementation Phases

### Phase 0 — New models and utility

**Files:** `crates/xray-tui-db/src/models_toasty.rs`, `crates/xray-tui-db/src/hash.rs`, `crates/xray-tui-db/src/lib.rs`
pub struct EndpointRow {
pub endpoint: Endpoint,
pub protocols: Vec<ProtocolRow>,
pub extensions: HashMap<i64, ProfileExtension>,
pub stats: HashMap<i64, ServerStat>,
pub resolved_ips: Vec<IpAddr>,
pub selected_protocol: usize,
pub expanded: bool,
}

pub enum PurgatoryView {
Active,
Stale,
All,
}

**Data flow:**

```rust
// Endpoint.id derivation
let endpoint_id = stable_hash(host, port);             // known host:port
let endpoint_id = stable_hash("undefined", uid_str);   // exotic config

// ProtocolRow.id = uid = sig ^ cred_hash (unchanged from Profile.id)
```

### Phase 1 — Schema initialization

**Files:** `crates/xray-tui-db/src/database.rs`

1. `Database::open()`:
   - Detect existing DB file. If exists → delete it (unlink the file).
   - Create new empty DB file via toasty `push_schema()`.
   - Call `init_default_groups()`.

2. `init_default_groups()`:
   - Insert one default group "Default" for backward compat.
   - Use UUID for group id (same pattern as current system group IDs).

3. Remove old init logic:
   - Delete `create_system_groups()` and related system group code.
   - Delete `PURGATORY_TTL_DAYS` constant or replace with config-driven value.
   - Drop all old `PRAGMA user_version` / schema version checks.

**Schema version detection detail:**

- On open, check if DB file exists at path.
- If exists → delete with `std::fs::remove_file()`.
- Call `Database { db: toasty::Db::open(path)? }`.
- `db.push_schema()` creates all tables from current toasty::models!().
- `init_default_groups()` inserts default row.

### Phase 2 — DB read/write methods

**Files:** `crates/xray-tui-db/src/database.rs`

Replace all CRUD methods. New method signatures:

```rust
// ── Read queries ──

/// Active endpoints (excludes purgatory).
/// Filter: MAX(p.last_seen_at) >= now - ttl_secs (inner join protocols)
pub async fn get_active_endpoints(&self, ttl_secs: i64) -> Result<Vec<EndpointRow>>;
pub async fn get_active_endpoints_by_group(&self, group_id: &str, ttl_secs: i64) -> Result<Vec<EndpointRow>>;

/// Stale endpoints (past TTL but not past retention).
/// Filter: MAX(p.last_seen_at) < now - ttl_secs AND MAX(p.last_seen_at) >= now - retention_secs
pub async fn get_stale_endpoints(&self, ttl_secs: i64, retention_secs: i64) -> Result<Vec<EndpointRow>>;

/// All endpoints (no TTL filter).
/// Single endpoint by ID, with all protocols.
pub async fn get_endpoint(&self, id: i64) -> Result<Option<EndpointRow>>;

// ── Write methods ──

/// Subscription upsert (transaction):
/// 1. INSERT OR IGNORE endpoint row (no last_seen_at — staleness derived from protocol rows)
/// 2. Upsert each protocol row (ON CONFLICT(id) UPDATE spec_blob, core_type, transport, security, remarks, last_seen_at)
/// 3. Upsert endpoint_groups row (ON CONFLICT(endpoint_id, group_id) UPDATE last_seen_at)
/// 4. No orphan deletion (protocols never deleted, age out naturally)
pub async fn subscription_upsert(&self, endpoint: Endpoint, protocols: &[ProtocolRow], group_id: &str) -> Result<()>;

/// Manual endpoint insert (transaction):
/// Inserts endpoint + protocols + EndpointGroup in one transaction.
pub async fn insert_manual_endpoint(&self, endpoint: &Endpoint, protocols: &[ProtocolRow], group_id: &str) -> Result<()>;

/// Hard-delete past retention (cascade):
/// DELETE FROM profile_extensions WHERE protocol_id IN (
///   SELECT p.id FROM protocols p
///   JOIN endpoints e ON e.id = p.endpoint_id
///   WHERE p.last_seen_at < now - retention_secs)
/// DELETE FROM server_stats WHERE protocol_id IN (same subquery)
/// DELETE FROM ping_sessions WHERE protocol_id IN (same subquery)
/// DELETE FROM protocols WHERE endpoint_id IN (
///   SELECT e.id FROM endpoints e
///   JOIN protocols p ON p.endpoint_id = e.id
///   WHERE p.last_seen_at < now - retention_secs)
/// DELETE FROM endpoint_groups WHERE endpoint_id IN (same subquery)
/// DELETE FROM endpoints WHERE id IN (same subquery)
/// Returns count of deleted endpoints.
pub async fn purge_expired(&self, retention_secs: i64) -> Result<u64>;

/// Restore endpoint from purgatory (resurrection).
/// Updates ProtocolRow.last_seen_at = NOW for all protocols under this endpoint.
pub async fn restore_endpoint(&self, endpoint_id: i64) -> Result<()>;

/// Delete endpoint + cascade.
pub async fn delete_endpoint(&self, endpoint_id: i64) -> Result<()>;

/// Set manual protocol override on endpoint.
pub async fn set_protocol_override(&self, endpoint_id: i64, protocol_id: Option<i64>) -> Result<()>;

/// Clear manual override, re-enable auto-select.
pub async fn clear_protocol_override(&self, endpoint_id: i64) -> Result<()>;

/// Get stale endpoint count (for status bar badge).
/// SELECT COUNT(DISTINCT e.id) FROM endpoints e
/// JOIN protocols p ON p.endpoint_id = e.id
/// WHERE p.last_seen_at < now - ttl_secs
///   AND p.last_seen_at >= now - retention_secs
pub async fn get_stale_count(&self, ttl_secs: i64, retention_secs: i64) -> Result<u64>;
```

**Return type:**

```rust
pub struct EndpointRow {
    pub endpoint: Endpoint,
    pub protocols: Vec<ProtocolRow>,
    pub extensions: HashMap<i64, ProfileExtension>,
    pub stats: HashMap<i64, ServerStat>,
    pub resolved_ips: Vec<IpAddr>,
    pub selected_protocol: usize,
    pub expanded: bool,
}
```

**Methods to DELETE (complete list):**

- `get_all_profiles`, `get_profiles_by_group`, `get_all_profiles_with_connections`, `get_all_profiles_with_details`
- `get_profile`, `get_connections_for_profile`, `get_connections_for_group`, `get_active_profile_for_group`
- `insert_profile`, `update_profile`, `delete_profile`, `reorder_profiles`
- `subscription_upsert_profiles`, `purge_orphans`, `clear_group`, `delete_group`, `update_profile_active`
- `move_orphans_to_graveyard`, `purge_graveyard`
- All associated deserialization helpers (`deserialize_profiles`, `deserialize_profile_connections`, `deserialize_connections`)

### Phase 3 — DNS resolver

**Files:** `crates/xray-tui-core/src/dns.rs` (new), `crates/xray-tui-core/src/lib.rs` (trigger points), `crates/xray-tui-db/src/database.rs` (IP upsert)

```rust
pub async fn resolve_dns_name(host: &str) -> Result<Vec<IpAddr>, std::io::Error>
```

Implementation:

- Use `tokio::net::lookup_host((host, 443))`.
- Sort results: IPv4 first, IPv6 second.
- Dedup.
- 5s timeout per resolve call.

**Lazy resolution — not during subscription import.** DNS resolved only on demand at three trigger points:

1. **Fast Ping / Real Ping** — before pinging a DnsName endpoint, resolve DNS. Ping each resolved IP. Store results as child Endpoint rows.
2. **User connects to profile** — before starting core, resolve DNS. Insert/update child Endpoint rows with `parent_id = DnsName endpoint.id`.
3. **User presses expander on profile tree** ("list IPs" section) — resolve DNS on the spot. Show results immediately, persist as child Endpoint rows.

**DB method:**

```rust
pub async fn upsert_resolved_ips(
    &self,
    dns_endpoint_id: i64,
    ips: &[IpAddr],
) -> Result<Vec<i64>>
// Returns the endpoint IDs of the inserted/found IP endpoints.
// INSERT OR IGNORE: if IP endpoint already exists (stable_hash), just
// update its parent_id and last_seen_at. Do not create duplicates.
```

**DB method:**

```rust
// Consolidated: resolve DNS + persist children. Atomically replaces old IP
// children for this DnsName endpoint with new set.
pub async fn resolve_endpoint_dns(
    &self,
    dns_endpoint_id: i64,
    host: &str,
) -> Result<Vec<IpAddr>>
// 1. Resolve DNS via tokio::net::lookup_host
// 2. DELETE FROM endpoints WHERE parent_id = dns_endpoint_id
// 3. INSERT new child rows for each resolved IP
// 4. Return resolved IPs
```

**EndpointRow.resolved_ips lazy-fetch:**

- NOT populated on endpoint load (no JOIN). Empty until first DNS trigger.
- First trigger calls `resolve_endpoint_dns()` → populates `resolved_ips` in AppState.
- Cached in-memory with 1h TTL configurable from AppConfig.DnsConfig.refresh_hours.
- On cache expiry or explicit re-trigger: re-resolve, overwrite children, update cache.

**Overwrite semantics:**

Each resolution replaces ALL child IP endpoints for that DnsName parent. This prevents stale IP accumulation when DNS records change. Child IPs from previous resolution are hard-deleted (not aged into purgatory — IP endpoints have no meaningful last_seen_at for a TTL filter).

### Phase 4 — Purgatory TTL logic

**Files:** `crates/xray-tui-config/src/lib.rs` (AppConfig), `crates/xray-tui-core/src/process.rs` or core lib (background sweep)

**Config:**

```rust
pub struct PurgatoryConfig {
    pub ttl_days: u64,       // default 7
    pub retention_days: u64, // default 30 (hard delete after this)
    pub enabled: bool,       // default true
}
```

Wire into `AppConfig` struct in `xray-tui-config`.

**Background sweep** (in existing `spawn_auto_update()` loop):

1. Every 60s, check if purgatory sweep is due (config-driven interval).
2. Call `db.purge_expired(retention_secs)`.
3. Log count of deleted endpoints.

**Manual resurrection:**

- Key `r` in profile list (when viewing Stale/All): calls `db.restore_endpoint(endpoint_id)`.
- Key `d` in profile list (when viewing Stale): calls `db.delete_endpoint(endpoint_id)` with confirmation overlay.

### Phase 5 — Protocol security ranking

**Files:** `crates/xray-tui-proto/src/proto_spec/security_rank.rs` (new)

```rust
pub fn protocol_security_rank(proto_kind: &str) -> u8
```

Ranking (lower = more secure = shown first):
| Protocol | Rank |
|----------|------|
| wireguard | 0 |
| vmess | 1 |
| vless | 2 |
| trojan | 3 |
| ss-2022 | 4 |
| ss | 5 |
| hysteria2 | 6 |
| tuic | 7 |
| hysteria | 8 |
| anytls | 9 |
| shadowtls | 10 |
| naive | 11 |
| socks | 12 |
| http | 13 |
| ssr | 14 |
| _unknown_ | 255 |

Used as default sort for protocols within EndpointRow. Ties broken by proto_kind string order.

### Phase 6 — EndpointRow data structure and AppState changes

**Files:** `crates/xray-tui/src/lib.rs`

**New types:**

pub struct EndpointRow {
pub endpoint: Endpoint,
pub protocols: Vec<ProtocolRow>,
pub extensions: HashMap<i64, ProfileExtension>,
pub stats: HashMap<i64, ServerStat>,
pub resolved_ips: Vec<IpAddr>,
pub selected_protocol: usize,
pub expanded: bool,

````

```rust
pub enum PurgatoryView {
    Active,
    Stale,
    All,
}
````

**AppState changes:**

| Current                             | New                                                                               |
| ----------------------------------- | --------------------------------------------------------------------------------- |
| `profiles: Vec<ProfileRow>`         | `endpoints: Vec<EndpointRow>` (one filtered list, view depends on purgatory_view) |
| `selected_group_id`                 | REMOVE                                                                            |
| `profiles_gen: u64`                 | `endpoints_gen: u64`                                                              |
| `connected_profile_id: Option<i64>` | `connected_protocol_id: Option<i64>`                                              |
| `multi_select: HashSet<i64>`        | same, but stores endpoint IDs                                                     |
| —                                   | `purgatory_view: PurgatoryView` (three-way toggle, default Active)                |
| —                                   | `purgatory_ttl: u64` (from AppConfig.PurgatoryConfig.ttl_days)                    |
| —                                   | `purgatory_retention: u64` (from AppConfig.PurgatoryConfig.retention_days)        |
| —                                   | `ip_preference: IpPreference` (Ipv4/Ipv6/Auto)                                    |

**Pre-select best protocol:**

```rust
fn select_best_protocol(
    protocols: &[ProtocolRow],
    extensions: &HashMap<i64, ProfileExtension>,
    override_id: Option<i64>,
) -> usize
```

Called in `reload_endpoints()` after DB fetch. Stores result in `EndpointRow.selected_protocol`.

### Phase 7 — TUI: 3-way profile list with endpoint tree

**Files:** `crates/xray-tui/src/ui/profiles.rs`, `crates/xray-tui/src/ui/mod.rs`

**Tab bar:** `[Profiles] [Settings] [Logs] [Statistics]` no separate Purgatory tab.

**Three-way toggle:**

- Default: Active view (WHERE last_seen_at >= NOW - TTL)
- Key `P`: cycle Active → Stale → All → Active
- Stale view: endpoints where last_seen_at < TTL but >= retention. Show with "stale" coloring and TTL-left badge.
- All view: all endpoints. Stale ones get indicator.
- View mode shown in status bar or tab header: `[Profiles (Active)]`

**Endpoint summary row** (collapsed state):

| Col       | Content                                          |
| --------- | ------------------------------------------------ |
| ""        | connected/testing indicator + stale indicator    |
| "#"       | index / multi-select `*`                         |
| "Type"    | best protocol's proto_kind                       |
| "Remarks" | best protocol's remarks (or blank for undefined) |
| "Addr"    | host:port (empty for undefined)                  |
| "Confs"   | protocol count badge + expand marker `▶`         |
| "Tran"    | best protocol's transport                        |
| "Sec"     | best protocol's security                         |
| "Delay"   | from best protocol's extension                   |
| "Speed"   | from best protocol's extension                   |
| "IP"      | IP info or resolved IP count                     |
| "Traffic" | total across all protocols                       |
| (Stale)   | "last seen X days ago"                           |

**Expandable tree** (toggled by `Right`/`Left` arrow on selected row):

Section A — **IP Resolution** (for DnsName endpoints):

- On expand: trigger lazy DNS resolution if no cached IP endpoints exist or they're stale (last_seen_at > 1h old).
- While resolving: show spinner "Resolving...".
- After resolve: display IP list, store as child Endpoint rows.
- Resolution also triggers automatically on connect and fast/real ping (see Phase 3).
- If DNS fails: No cached IPs → expand shows "DNS unavailable, using hostname". Cached IPs → show cached list with stale indicator.

Example display:

```
└─ Addresses:
     192.168.1.1     ● ← active
     10.0.0.1        ○
     2606:4700::6810:8e44  ○
```

└─ Configurations:
│ VMess-TCP-TLS ● ← active (auto)
│ VMess-WS-TLS ○
│ VLESS-TCP-Reality ○

```

Sub-row operations:
- `Up`/`Down` navigate sub-rows
- `Enter` on non-active protocol → sets manual override + reconnects
- `Backspace` or `x` on manually-overridden protocol → clears override, reverts to auto
- Active indicator `●` vs `○`
- Manual override shows `(user)` label, auto-select shows `(auto)`

**Keyboard routing updates in `mod.rs`:**

- `P` toggles `purgatory_view` in AppState
- `Right` expands selected endpoint
- `Left` collapses selected endpoint
- `Up`/`Down` navigate profile list or sub-row tree
- `Enter` connects (uses selected_protocol)
- `r` in Stale view: restore endpoint
- `d` in Stale view: delete endpoint with confirmation

**Reducing existing profiles.rs code:**
- Remove `compute_filtered_indices` group filter path
- Remove `render_group_overlay` or delegate to settings
- Remove any `selected_group_id` logic
- Remove per-group active profile concept
- Remove graveyard group filter

**Purgatory badge:** Show stale count in status bar: `[Stale: N]` when any endpoints are in purgatory.

### Phase 8 — Load-bearing changes

**Files:** `crates/xray-tui-config/src/import_export.rs`, `crates/xray-tui-core/src/config_builder/xray.rs`, `crates/xray-tui-core/src/config_builder/singbox.rs`, `crates/xray-tui/src/lib.rs` (connect/delete/clone actions)

**parse_share_url changes:**
- Return type adds `(host: String, port: u16, host_type: String)` alongside profile data.
- For exotic configs: host = "", port = 0, host_type = "undefined".
- Caller computes `endpoint_id` via `stable_hash()`.

**confirm_add_server changes:**
- Accept endpoint metadata from form fields.
- Build `Endpoint` + `ProtocolRow` structs.
- Call `db.insert_manual_endpoint()`.

**Config builder changes:**
- Parameter type: `&Profile` → `(&Endpoint, &ProtocolRow)`.
- `address`/`port` from `Endpoint.host`/`Endpoint.port`.
- `transport`/`security` from `ProtocolRow.transport`/`ProtocolRow.security`.
- For `undefined` endpoints: produce minimal no-op config or skip.

**AppState actions:**

| Action | New behavior |
|--------|-------------|
| `connect_to_protocol(protocol_id)` | Starts core with that specific protocol's config. Uses Endpoint.host + ProtocolRow.spec_blob |
| `connect_to_endpoint(endpoint_id, protocol_idx)` | Resolves protocol from endpoint row by index, delegates to connect_to_protocol |
| `delete_endpoint(id)` | `db.delete_endpoint()` — cascade deletes endpoint + all protocols + extensions + stats + groups |
| `clone_endpoint(id)` | Clone endpoint + all protocol rows with new random uids (like current form-created profiles) |
| `set_protocol_override(endpoint_id, protocol_id)` | Writes manual_protocol_override to DB, reloads endpoint row, reconnects if connected |
| `clear_protocol_override(endpoint_id)` | Writes NULL to manual_protocol_override, reloads, reconnects with auto-select |

**Removed:**
- `set_active(id)` — no per-group active profile concept
- `reorder_profiles()` — removed
- `cycle_group()` — profile list always global

**Subscription import flow** (`update_group_subscriptions`):
1. Parse URLs from subscription.
2. For each parsed config:
   a. Derive endpoint_id from host/port (or undefined hash).
   b. Build ProtocolRow.
   c. If DNS name: resolve concurrently (Phase 3).
3. Call `db.subscription_upsert()` in batch.

**Subscription import — changes from current:**
- Remove `subscription_upsert_profiles()` call → `db.subscription_upsert()`.
- Remove `move_orphans_to_graveyard()` — no longer needed (protocols age out naturally).
- No ON CONFLICT dedup by `group_id + sub_uid` — dedup is by endpoint_id + protocol_id.

### Phase 9 — Tests

**Files:** New or updated test modules

1. **DB integration tests** (`crates/xray-tui-db/tests/integration.rs` or new file):
   - Insert endpoint with 2 protocols. Read: one EndpointRow with 2 ProtocolRows.
   - Subscription upsert: same endpoint+protocol → last_seen_at updated, no new rows.
   - Active vs Stale query: endpoint with old last_seen_at only in stale view. Update last_seen_at → disappears from stale.
   - Undefined endpoint: no host:port → host_type="undefined", host="", port=0.
   - Two sources same endpoint: one Endpoint row, two EndpointGroup rows.
   - Manual override: set manual_protocol_override → select_best_protocol() returns overridden.
   - Hard-delete: purge_expired removes rows past retention. Verify cascade.
   - DNS endpoint with IP children: insert, verify resolved_ips populated.

2. **Proto roundtrip tests**: Unchanged. Existing 96+ tests still pass.

3. **Build verification**: `cargo build`, `cargo clippy`.

4. **Manual smoke test**: `cargo run` → profiles displayed grouped by endpoint, expandable tree works, 3-way toggle works, connect/disconnect works.

---

## File Change Summary

| File | Change | Lines |
|------|--------|-------|
| `crates/xray-tui-db/src/hash.rs` | **NEW** — stable_hash utility | ~15 |
| `crates/xray-tui-db/src/models_toasty.rs` | Add Endpoint, ProtocolRow, EndpointGroup; remove Profile, Connection, Subscription; update Group; rename FK fields on PingSession, ProfileExtension, ServerStat | Heavy rewrite |
| `crates/xray-tui-db/src/lib.rs` | Add `pub mod hash;` | +1 |
| `crates/xray-tui-db/src/database.rs` | Replace all CRUD; add new read/write methods; delete old methods + deserializers | Heavy rewrite |
| `crates/xray-tui-db/src/error.rs` | Possibly remove ProfileWithDetails if unused | Minor |
| `crates/xray-tui-core/src/dns.rs` | **NEW** — DNS resolver | ~25 |
| `crates/xray-tui-core/src/lib.rs` | Wire DNS into subscription import; background sweep update | Moderate |
| `crates/xray-tui-core/src/speed_test.rs` | Update PingSession references if needed | Minor |
| `crates/xray-tui-proto/src/proto_spec/security_rank.rs` | **NEW** — protocol_security_rank() | ~30 |
| `crates/xray-tui-proto/src/proto_spec/mod.rs` | Export security_rank module | +1 |
| `crates/xray-tui-config/src/lib.rs` | Add PurgatoryConfig to AppConfig | Minor |
| `crates/xray-tui-config/src/import_export.rs` | Return endpoint metadata from parse_share_url | Moderate |
| `crates/xray-tui/src/lib.rs` | AppState changes (EndpointRow, remove selected_group_id, purgatory_view) | Moderate |
| `crates/xray-tui/src/ui/profiles.rs` | Rewrite profile list with two-row tree, 3-way toggle, expand/collapse | Heavy |
| `crates/xray-tui/src/ui/mod.rs` | Key routing changes (P toggle, arrow expand, no purgatory tab) | Moderate |
| `crates/xray-tui/src/ui/settings.rs` | Group mgmt → Subscriptions section; remove groups.rs call | Minor |
| `crates/xray-tui/src/ui/groups.rs` | DELETE — replaced by Settings::Subscriptions | Full deletion |
| `crates/xray-tui-core/src/config_builder/xray.rs` | (&Profile) → (&Endpoint, &ProtocolRow) | Moderate |
| `crates/xray-tui-core/src/config_builder/singbox.rs` | Same parameter change | Moderate |

## Verification

1. `cargo test` — all existing tests pass.
2. `cargo clippy` — workspace lints clean.
3. Fresh DB init: delete old DB, run app. New `endpoints`, `protocols`, `endpoint_groups`, `groups` tables created.
4. Subscription import: same subscription imported twice → one endpoint row per unique host:port, not duplicated.
5. Purgatory filter: endpoint with `ProtocolRow.last_seen_at = 0` appears in Stale view, not Active.
6. Resurrection: press `r` → protocol `last_seen_at = NOW` → endpoint back in Active view.
7. Protocol tree: endpoint with 3 protocols shows expandable tree, sorted by security rank.
8. Manual override: select non-default protocol, reconnects, shows (user) label.
9. Undefined endpoint: exotic config shows in profile list with no address, can connect.
10. DNS resolution: DnsName endpoint shows resolved IPs in expanded view.
11. Config builder: generates correct config from `(Endpoint, ProtocolRow)` pair.
12. `cargo run` — TUI boots, profile list renders, tab navigation works, connect works.
```
