# Typed Data Model Redesign (Toasty 0.9 + jiff)

Date: 2026-08-05
Status: Approved for planning

## 1. Context

The project stores protocol configs as opaque binary blobs (`spec_blob`) and
re-derives typed values by JSON-parsing them at every access site: config
builders (`parse_settings`/`to_settings`), import/export (`profile_config`),
enrichment (`extract_sni`), subscription filtering. Three intermediate
representations of the same entity coexist: `Proto` (typed parse result),
Clash YAML structs (external format), and settings-JSON blobs (legacy
round-trip). `database.rs` contains ~70 raw `toasty::sql` statements including
a 36-column JOIN with hand-rolled deserializers. Timestamps are unix-`i64` or
SQLite `datetime('now')` strings. Magic strings (`proto_kind`, `core_type`,
`host_type`, `status`, `ping_type`) carry semantics that should be enums.

The project is alpha: the database is dropped and recreated, migrations and
legacy-format compatibility are out of scope.

### Goals

- Fully typed data model across all crates; no raw data interaction except at
  the single parse boundary.
- Better normalization: protocol config stored once per distinct protocol, not
  per (endpoint, protocol) pair.
- Modern Toasty features: embedded structs/enums, `Deferred`, `Json<T>`,
  `Vec<scalar>`, composite keys, `#[version]` optimistic concurrency, auto
  timestamps.
- Replace magic `String`/`i32` fields on Toasty models with typed enums.
- Deny string-based timestamp representation; adopt `jiff::Timestamp` everywhere.
- Zero raw SQL except connection PRAGMAs, without performance degradation or
  memory bloat.

## 2. Parse Boundary Normalization (`ProtoSpec` rework)

`ProtoSpec` and every implementer are reworked so parsing produces objects
that embed directly into database rows. The parse result is endpoint-centric:

```rust
pub struct Proto {
    /// 0..N endpoints derivable from the input (usually exactly one).
    /// Empty when the whole config is encrypted and endpoint info cannot
    /// be extracted (e.g. future "SlipNet"-style protocols).
    pub endpoints: Vec<EndpointEssentials>,
    /// Always present. Contains exposed fields plus the exact, serializable
    /// protocol definition — with NO host/port information.
    pub protocol: ProtocolEssentials,
}

pub struct EndpointEssentials {
    pub host: String,
    pub host_type: HostType,          // ipv4 | ipv6 | dns | undefined
    pub port: u16,                    // primary port
    pub ports: Vec<u16>,              // full port spec when multi-port
}

pub struct ProtocolEssentials {
    pub proto_kind: ProtocolKind,
    pub config_type: ConfigType,       // ShareUrl | Form
    pub core_type: CoreType,           // Xray | SingBox (auto-resolved at parse)
    /// The exact, serializable protocol definition — the config struct sans
    /// host/port. This is the identity-hashed payload. Transport/security
    /// "exposed fields" (TransportEssentials/SecurityEssentials) are derived
    /// from it at DB write time for the cached columns — they are NOT stored
    /// here.
    pub config: ProtocolConfig,
}
```

- Protocol config structs (`VlessConfig`, `VmessConfig`, ...) drop their
  `host`/`port` fields; those move to `EndpointEssentials`. Config builders
  already receive `(endpoint, protocol)` — host comes from the Endpoint row.
- `uid = sig ^ cred_hash` is computed from the protocol essentials only;
  host and port specifications are excluded from `protocol_id` computation.
- Parse may yield: one endpoint + one protocol (normal), several endpoints +
  one protocol (multi-port / multi-endpoint inputs), or zero endpoints + one
  protocol (encrypted config). Zero-endpoint protocols produce an orphan
  `Protocol` row (no `ProfileStats` link) — schema must allow it. TUI
  presentation of orphan protocols is deferred until such a protocol exists.
- Natural keys prevent duplicate protocol configs: endpoint natural key is
  `(host, host_type, ports)`; protocol natural key is `uid`.
- `ProtocolEssentials.config` is the identity-hashed payload; the DB's cached
  `transport`/`security` embed columns (type, sni, fp, insecure + JSON data)
  are derived from it at write time via config accessors
  (`config.transport()`, `config.security()`). No denormalized
  `TransportEssentials`/`SecurityEssentials` types in the boundary.
- HOST-FREE PARSE MANDATE: parsers never call `TransportConfig::with_host`
  and never copy the endpoint host into transport/security config fields
  (ws/http/grpc host, `GrpcConfig.authority`, `SecurityConfig.sni`). An
  explicit URL-level override (ws `host=` param, grpc authority, sni param)
  IS a protocol parameter and stays in the config; absent → field unset.
  Builders inject endpoint host at build time (`inject_to` receives
  `endpoint`). Keeps endpoint-derived bytes out of `uid`.
- The `Proto` container survives with the new content. `ProtoSpec` trait
  (`parse_share_url`, `format_share_url`, `try_from_clash`, `to_clash`,
  identity computation) is updated to the split. Clash YAML structs remain
  external-format boundary types, used only by the conversion methods.

## 3. Toasty Feature Enablement

`xray-tui-db` manifest changes:

```toml
toasty = { version = "0.9", features = ["turso", "serde", "jiff"] }
jiff = "0.2"        # (workspace dep; also used by xray-tui for display)
xray-tui-proto = { path = "../xray-tui-proto" }   # new dep: typed config types
```

The `jiff` feature enables `jiff::Timestamp` as a scalar. `#[auto]` on
`created_at: jiff::Timestamp` expands to `#[default(jiff::Timestamp::now())]`;
`#[auto]` on `updated_at` expands to `#[update(jiff::Timestamp::now())]`.

Dependency note: `xray-tui-db` is currently a leaf crate; `xray-tui-proto`
has no internal dependencies, so `db -> proto` is acyclic. The `Protocol`
enum moves from `xray-tui-core/src/protocol.rs` into `xray-tui-proto`
(renamed `ProtocolKind`, `#[derive(toasty::Embed)]`); `xray-tui-core`
re-exports it for its callers.

## 4. Models (`xray-tui-db/src/models_toasty.rs`, rewritten)

Tables: 9 → 7. `ProtocolRow`, `ProfileExtension`, `ServerStat`, `PingSession`
are deleted. `Protocol` and `ProfileStats` are born.

### Embedded types

```rust
// ── enums (all #[derive(toasty::Embed)]) ─────────────────────────────
enum ProtocolKind  { Vmess, Vless, Trojan, Ss, Ssr, Tuic, Hysteria1, Hysteria2,
                     Wireguard, Socks, Http, Naive, AnyTls, ShadowTls, Tor,
                     Ssh, Tailscale, Redirect, TProxy, Mixed }            // moved from xray-tui-core
enum CoreType      { Xray, SingBox }                                      // from proto_spec
enum HostType      { Ipv4, Ipv6, Dns, Undefined }
enum ConfigType    { ShareUrl, Form }
enum TransportType { Tcp, Ws, Grpc, Http, HttpUpgrade, XHttp, Kcp }       // mirrors TransportConfig::type_str()
enum SecurityType  { None, Tls, Reality }                                 // mirrors SecurityConfig::type_str()
enum TaskKind      { FastPing, RealPing, UdpPing, UdpTest, SpeedTest }    // DashMap value
enum ProfileErr    { Real, Fast, Name }                                   // TUI label marker
enum GroupStatus   { Ok, Error, Never }
enum GroupCoreType { Auto, Xray, SingBox }   // Group only: form allows "auto"; resolved CoreType has no Auto
enum QueryStrategy { UseIp, UseIpv4, UseIpv6 } // DnsSetting.query_strategy (settings form is a select)
enum ConvertTarget { Clash }                  // Group.convert_target; never written today, kept typed

// ── structs ────────────────────────────────────────────────────────────
struct Transport {
    r#type: TransportType,
    #[column(type = text)]
    data: Deferred<Json<TransportConfig>>,       // opaque JSON, deferred load
}
struct Security {
    r#type: SecurityType,
    sni: Option<String>,        // cached informational fields
    fp: Option<String>,
    insecure: Option<bool>,
    #[column(type = text)]
    data: Deferred<Json<SecurityConfig>>,
}
struct ErrorInfo  { kind: ProfileErr, text: String }
struct TrafficStats { today_up: i64, today_down: i64, total_up: i64, total_down: i64 }

enum Latency {
    Real { #[shared(delay)] delay: i32, ip: Option<String> },   // columns: latency_delay, latency_ip
    Fast { #[shared(delay)] delay: i32 },                        // shares latency_delay
}
```

`#[shared(delay)]` coalesces the `delay` field of both variants into one
column (`latency_delay`); only the active variant's columns are non-null.

### Models

```rust
#[derive(toasty::Model)]
struct Endpoint {
    #[key] id: EndpointId,                  // newtype(i64), never zero; stable_hash(host, host_type, ports)
    host: String,
    host_type: HostType,
    port: u16,                              // primary port; 0 for undefined
    ports: Vec<u16>,                        // replaces dead port_spec_str (verified: never read)
    parent_id: Option<EndpointId>,          // resolved IP -> DnsName parent
    last_source: Option<String>,            // hash of source subscription
    manual_protocol_override: Option<ProtocolId>,   // NULL = auto-select best
    resolved_as: Vec<String>,               // replaces comma-joined String
    resolved_at: Option<jiff::Timestamp>,
    #[auto] created_at: jiff::Timestamp,
    #[has_many] links: Deferred<Vec<ProfileStats>>,      // per-pair state
    #[has_many] group_links: Deferred<Vec<EndpointGroup>>,
}

#[derive(toasty::Model)]
struct Protocol {
    #[key] id: ProtocolId,                  // = uid = sig ^ cred_hash (protocol essentials only; never zero)
    sig: i64,
    cred_hash: i64,
    proto_kind: ProtocolKind,
    transport: Transport,                   // embed — cached type/data derived from config at write
    security: Security,                     // embed — cached sni/fp/insecure/data derived from config at write
    /// The full exact definition (ProtocolConfig sans host/port), stored as
    /// one opaque JSON column. Deferred: not loaded by list queries; decoded
    /// only when the config is needed (config build, edit form). The cached
    /// transport/security embeds are derived from it on every write.
    #[column(type = text)]
    config: Deferred<Json<ProtocolConfig>>,
    #[auto] created_at: jiff::Timestamp,
    #[has_many] links: Deferred<Vec<ProfileStats>>,
}

#[derive(toasty::Model)]
#[key(protocol_id, endpoint_id)]
struct ProfileStats {                       // link + activity hub + task gate (single row per pair)
    protocol_id: ProtocolId,
    endpoint_id: EndpointId,
    core_type: CoreType,                    // per-pair override
    config_type: ConfigType,
    last_used_at: Option<jiff::Timestamp>,
    last_seen_at: jiff::Timestamp,          // per-link staleness tracking
    task_id: Option<u16>,                   // currently queued/running task; 0 never valid
    task_queue: Vec<u16>,                   // FIFO of queued task ids
    latency: Option<Latency>,               // embed enum, shared delay column
    speed_bps: Option<i64>,
    error: Option<ErrorInfo>,               // persisted failure marker (see §6.3)
    traffic: TrafficStats,                  // today/total up/down
    #[auto] created_at: jiff::Timestamp,
    #[auto] updated_at: jiff::Timestamp,    // doubles as error-TTL anchor
    #[version] version: u64,                // optimistic concurrency
    #[belongs_to(key = protocol_id, references = id)] protocol: Deferred<Option<Protocol>>,
    #[belongs_to(key = endpoint_id, references = id)] endpoint: Deferred<Option<Endpoint>>,
}

#[derive(toasty::Model)]
#[key(endpoint_id, group_id)]
struct EndpointGroup {                      // composite key replaces synthetic UUID id
    endpoint_id: EndpointId,
    group_id: String,
    last_seen_at: jiff::Timestamp,          // per-source last confirmation
    sort_order: Option<i32>,
}

#[derive(toasty::Model)]
struct Group {
    #[key] id: String,                      // UUID
    name: Option<String>,
    url: Option<String>,
    enabled: bool,                          // was Option<i32>
    user_agent: Option<String>,
    convert_target: Option<ConvertTarget>,  // never written today; typed for future use
    core_type: Option<GroupCoreType>,       // form allows "auto"
    sort_order: Option<i32>,
    last_refreshed: Option<jiff::Timestamp>,// was String
    status: Option<GroupStatus>,            // was String "ok"|"error"|"never"
    error_message: Option<String>,
    refresh_interval: Option<i64>,
}

#[derive(toasty::Model)]
struct RoutingRule {                        // fully typed this pass
    #[key] id: String,
    group_id: Option<String>,
    r#type: i32,                            // opaque free-form number, no consumer
                                            // (builders always emit "type":"field"); kept for form
    domain_matcher: Option<String>,
    domains: Vec<String>,                   // was comma-joined Option<String>
    ips: Vec<String>,
    inbound_tags: Vec<String>,
    ports: Vec<u16>,                        // was Option<String> "80,443"
    source_ports: Vec<u16>,
    network: Option<String>,                // pass-through, comma-able ("tcp,udp")
    protocols: Vec<String>,                 // IANA protocol names ("http","tls"), NOT ProtocolKind
    domain_strategy: Option<String>,        // pass-through ("AsIs"/"UseIP"/...)
    outbound_tag: Option<String>,
    balancer_tag: Option<String>,
    rule_set_file: Option<String>,
    rule_set_url: Option<String>,
    sort_order: Option<i32>,
}

#[derive(toasty::Model)]
struct DnsSetting {                         // fully typed this pass
    #[key] id: String,
    name: Option<String>,
    servers: Vec<String>,                   // was comma-joined Option<String>
    hosts: Vec<String>,
    query_strategy: Option<QueryStrategy>,  // settings form is a select: ,UseIP,UseIPv4,UseIPv6
    disable_cache: bool,                    // was Option<i32>
    disable_fallback: bool,
    client_ip: Option<String>,
    cache_ttl_secs: Option<i64>,            // TUI-side DNS cache TTL; None = default 300
}
```

### Non-zero key invariants

Toasty 0.9.0 does not support `NonZeroI64` field types (verified in
`toasty-macros-0.9.0` and `toasty-core-0.9.0`). Non-zero guarantees are
enforced with newtype embeds whose constructors `debug_assert` and reject 0:

```rust
#[derive(toasty::Embed)] struct EndpointId(i64);   // stable_hash output; never 0
#[derive(toasty::Embed)] struct ProtocolId(i64);   // uid; sig never 0 => uid never 0
```

`task_id`/`task_queue` use plain `u16`; 0 is reserved-invalid by convention
(`task_id: Option<u16>` never contains 0; task ids are random in 1..=65535).

## 5. Config Storage (Protocol)

`Protocol.config` is a `Deferred<Json<ProtocolConfig>>` opaque JSON column
excluded from the default `SELECT`: list queries never decode configs. The
cached typed embeds (`transport`/`security`: `type`, `sni`, `fp`, `insecure`
+ their own `Deferred<Json<...>>` data columns) serve display and filtering.
The full config is decoded only when actually needed (config build, edit
form).

Write path: one constructor takes a `ProtocolConfig` (sans host/port) and
derives proto_kind + transport/security embeds from it in the same function —
cached columns cannot drift from the config. `spec_blob` column is deleted.

## 6. ProfileStats Task Scheduler

### 6.1 Scheduler state

In-process, in TUI `AppState`:

```rust
tasks: DashMap<u16, TaskKind>,        // live task registry; u16 keys, never 0
batches: HashMap<String, BatchState>, // in-memory batch progress (total/done)
```

`ProfileStats.task_id` + `task_queue` are the persisted image of the same
queue.

### 6.2 Scheduling rules

- **Schedule(protocol, kind):** read the pair's `ProfileStats`.
  - `task_id == None` → allocate random non-zero `u16`, insert into
    `tasks` DashMap, set `task_id`, fire the task.
  - `task_id == Some(id)` and `tasks` contains `id` (live) → push `id` of the
    new task onto `task_queue`. If queue length >= limit (setting, §6.4),
    do not enqueue; emit warning:
    `Cannot schedule <kind> on xray-tui://<protocol_id>: queue full` and skip.
  - `task_id == Some(id)` and `tasks` lacks `id` (orphan from a crashed
    session) → replace `task_id` with the new task's id, **wipe all orphan
    ids from `task_queue`** (any id not in `tasks`), then fire.
- **Completion(task_id, result):** write the result to `ProfileStats`
  (latency / speed_bps / error / traffic), then pop the first id from
  `task_queue` (FIFO), set it as `task_id`, fire it; empty queue → clear
  `task_id` to `None`. Remove the finished id from `tasks`.
- **Queue mutation on Turso:** `pop`/`remove`/`remove_at` on `Vec<scalar>`
  are unsupported on SQLite/Turso (verified: PostgreSQL only). FIFO pop and
  orphan wipe are read-modify-write of the whole `task_queue` vector, guarded
  by `#[version]` optimistic concurrency; on conflict, reload and retry
  (existing `retry_on_busy` machinery is reused).
- **DNS-failure deferral heuristic:** when an endpoint's DNS resolution
  failed within the last N seconds (new setting, default 5), tasks on that
  endpoint's profiles are not dispatched immediately; they are scheduled
  after the delay in a separate task. Uses the existing in-memory DNS-failure
  state (TTL-gated attempt entries, decision 13 of AGENTS.md) — no schema
  cost. Batch testing proceeds for all other endpoints immediately.

### 6.3 Batch pipeline rework

The `PingSession` multi-row ledger (batch × wave × occurrence-rank) is
replaced by per-pair tasks:

- `create_ping_batch` snapshots the visible profiles and enqueues one
  `FastPing` task per profile (probe dedup by address:port stays in the ping
  engine via `fast_cache`); phase 2 enqueues one `RealPing` task per profile.
- Per-pair serialization comes from the gate itself (one live task per pair).
  Wave/occurrence-rank machinery is deleted.
- `cancel_stranded_real_pings` becomes: when an endpoint's protocol succeeds
  under `dedup_endpoints=true`, remove queued `RealPing` task ids of the
  endpoint's sibling profiles (queue wipe, same OCC-guarded path).
- Batch progress (`BatchProgress` events) comes from the in-memory batch
  counter instead of session statuses.
- Group-level failures (config build / core start / no port ready) keep the
  existing retry-with-page-halving behavior, operating on the in-memory
  batch items.

### 6.4 Labels and persistence

- `[real]` / `[fast]` labels render when every protocol of the endpoint has
  `error.kind == Real` / `Fast` for that test. `[name]` renders when the
  endpoint is DNS-unresolved (`resolved_as` empty). Tier/sort logic reads
  persisted rows (`latency`, `error.kind`, `resolved_as`) instead of
  session-only round sets.
- **Behavior change (approved):** failure markers persist across restarts.
  New setting (Speed Test section) "Clear error after" — an optional TTL
  using `updated_at` as the anchor; when unset (default), errors survive
  until the next test overwrites them. On expiry, `error` is cleared.

### 6.5 Settings (Speed Test section)

- `task_queue_limit` — default 3; 0 disables queueing (tasks skipped when
  busy).
- `error_ttl_hours` — optional TTL for `ProfileStats.error`; unset = keep
  forever (default).
- `dns_failure_defer_secs` — default 5; the deferral window from §6.2.

## 7. Builder Redesign (`InjectToCoreConf`)

New sealed trait in `xray-tui-proto` (alongside `ProtoIdentity`):

```rust
pub trait InjectToCoreConf {
    fn inject_to(&self, core_conf: &mut serde_json::Value, core_type: CoreType)
        -> Result<(), SupportError>;
}
```

- Implemented by every `XxxConfig` type; `ProtocolConfig` dispatches via the
  existing `dispatch!` macro. `PlaceholderConfig` (Redirect/TProxy/Mixed —
  form-only, no URL format) injects its raw `settings_json`.
- `xray.rs`/`singbox.rs` shrink to: build the common JSON skeleton (log,
  stats/api, inbounds, routing, DNS, policy, tag naming, mux/freedom/
  blackhole outbounds) → call `protocol.config().inject_to(&mut conf,
  core_type)`.
- `SupportError` is a new error type in `xray-tui-proto` for misconfiguration
  (e.g. reality without `publicKey`/`serverName`, cipher not supported by the
  chosen core). Build-time validation guarantee of today is preserved:
  a forced core that cannot build the config fails at build time with a
  clear error.
- `parse_settings`, `to_settings`, `from_legacy_parse`, `convert_spec_blob`,
  and all legacy raw-JSON parse paths are deleted. `inject_to` is the single
  place where a protocol's exact config-injection rules live.

## 8. Zero Raw SQL

Remaining raw SQL: `PRAGMA journal_mode=WAL`, `PRAGMA busy_timeout=5000`,
`PRAGMA foreign_keys=ON` on connection setup. Everything else typed:

- **Reads:** the 36-column JOIN queries become typed queries with relation
  preloads: `Endpoint::filter(...).include(endpoint.links())` with
  `include(link.protocol())`/`include(link.endpoint())`; per-page filters
  (group membership, graveyard, search, sort) are typed predicates.
- **Purge cascade (expired endpoints):**
  `Endpoint::filter(Endpoint::fields().links().all(ProfileStats::fields().last_seen_at().lt(cutoff)))`
  — `.all()` lowers to `NOT IN (SELECT ... WHERE NOT ...)`; matches the
  current `COALESCE(MAX(last_seen_at), 0) < cutoff` semantics including
  endpoints with no links. Deletes run in explicit transactions, typed
  filters per table.
- **Upserts:** `INSERT ... ON CONFLICT` statements become toasty upserts
  (`upsert_by_*`).
- **Migrations:** `SCHEMA_VERSION` / `PRAGMA user_version` machinery and all
  `ALTER TABLE`/index shims deleted; schema is pure `db.push_schema()` on a
  fresh database.
- `retry_on_busy` and per-connection `busy_timeout` stay (enrichment herd +
  OCC conflicts).

## 9. Boundary Cleanup

- Deleted: `spec_blob` column, `Profile` legacy struct (import_export),
  `profile_config`, `encode_profile_spec`, `from_legacy_parse`,
  `convert_spec_blob`, legacy raw-JSON fallbacks in `parse_settings`,
  `Proto` identity-container caching (identity computed once at parse;
  `sig`/`cred_hash` live on the `Protocol` row).
- `parse_share_url` returns `Proto` (§2); subscription import produces
  endpoint/protocol upserts directly; forms build typed configs (the
  `encode_profile_spec` settings-JSON path is replaced by typed
  constructors).
- TUI `EndpointRow` holds typed rows: endpoint + `ProfileStats` links +
  included `Protocol` entities. `format_ts` switches from unix-`i64` to
  `jiff` display; traffic `today_*` reset logic switches from string-date
  comparison to jiff day comparison.
- Clash YAML structs stay as external-format boundary types.

## 10. Scope

In scope: all 7 tables' models, ProtoSpec split, `InjectToCoreConf` +
builder rework, ping pipeline rework, scheduler, zero raw SQL, settings
additions, TUI state plumbing, db-crate tests rewritten typed.

Out of scope: UI redesign beyond label/status plumbing, Clash struct removal,
`xray-tui-dns` / `xray-tui-geoip` / `xray-tui-host-features` internals
(callers updated only), orphan-protocol UI treatment (deferred until such a
protocol exists), migration/back-compat of any kind.

## 11. Verification

- `cargo test` (workspace) — db tests rewritten against typed queries; new
  tests for scheduler gate semantics, shared `Latency` column round-trip,
  `Json<T>` deferred round-trip, purge predicate, composite-key upserts.
- `cargo clippy` (workspace lints) and `cargo build --release`.
- Manual: fresh DB, import subscription → typed rows; connect + stats for
  both backends; fast/real/udp/speed tests with queue-limit and
  DNS-deferral behaviors; error-label persistence across restart;
  `[real]`/`[fast]`/`[name]` rendering.
