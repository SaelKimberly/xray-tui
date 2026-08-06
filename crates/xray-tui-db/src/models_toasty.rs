use toasty::{Deferred, Json};

/// `ProfileExtension.delay_source` provenance values.
pub const DELAY_SOURCE_FAST: i32 = 0;
pub const DELAY_SOURCE_REAL: i32 = 1;
pub const DELAY_SOURCE_UDP: i32 = 2;

// ── Primary models (toasty ORM, mapped to DB tables) ────────────────────

/// Endpoint: a network endpoint identified by host+port.
#[derive(Debug, Clone, toasty::Model)]
pub struct Endpoint {
    #[key]
    pub id: i64, // stable_hash(host, port) for known types; stable_hash("undefined", config_uid) for exotic
    pub host: String,                  // canonical host string; empty for undefined
    pub host_type: String,             // "ipv4" | "ipv6" | "dns" | "undefined"
    pub port: i32,                     // primary port; 0 for undefined
    pub port_spec_str: Option<String>, // full PortSpec when multi-port
    pub parent_id: Option<i64>,        // resolved IP -> DnsName parent
    pub last_source: Option<String>,   // hash of source subscription
    pub created_at: i64,
    pub manual_protocol_override: Option<i64>, // FK -> protocols.id, NULL = auto-select best
    /// Cached DNS resolution of `host` for `host_type == "dns"`: comma-joined
    /// IP strings ("`1.2.3.4,2606:4700::1`"). NULL = not resolved yet (deferred)
    /// or host is an IP. Persisted so launches do not re-resolve.
    pub resolved_as: Option<String>,
    /// Unix secs of the `resolved_as` lookup. NULL = never / IP host.
    pub resolved_at: Option<i64>,
}

/// `ProtocolRow`: a protocol configuration. Replaces Profile.
/// PK = uid = sig ^ `cred_hash` (same as old Profile.id).
#[derive(Debug, Clone, toasty::Model)]
#[index(endpoint_id)]
pub struct ProtocolRow {
    #[key]
    pub id: i64, // = uid = sig ^ cred_hash (same as old Profile.id)
    pub endpoint_id: i64, // FK -> endpoints.id
    pub sig: i64,
    pub cred_hash: i64,
    pub proto_kind: String,
    pub spec_blob: Vec<u8>,
    pub config_type: i32, // same semantics: 0 = share URL, 1 = form created
    pub core_type: String,
    pub transport: Option<String>,
    pub security: Option<String>,
    pub last_used_at: Option<i64>, // unix secs of last activation; None = never used
    pub created_at: i64,
    pub last_seen_at: i64, // per-config staleness tracking

    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,

    #[has_one]
    pub extension: Deferred<Option<ProfileExtension>>,
    #[has_one]
    pub server_stat: Deferred<Option<ServerStat>>,
}

/// Many-to-many link between endpoints and groups.
/// Replaces Connection (was `profile_id/group_id`, now `endpoint_id/group_id`).
#[derive(Debug, Clone, toasty::Model)]
#[unique(endpoint_id, group_id)]
pub struct EndpointGroup {
    #[key]
    pub id: String, // UUID
    pub endpoint_id: i64,  // -> endpoints.id
    pub group_id: String,  // -> groups.id
    pub last_seen_at: i64, // per-source last confirmation
    pub sort_order: Option<i32>,

    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,
    #[belongs_to(key = group_id, references = id)]
    pub group: Deferred<Option<Group>>,
}

/// Group: merged with old Subscription fields. Removed `is_system`.
#[derive(Debug, Clone, toasty::Model)]
pub struct Group {
    #[key]
    pub id: String,
    pub name: Option<String>,
    pub url: Option<String>,  // was subscription_url
    pub enabled: Option<i32>, // was subscription_enabled
    pub user_agent: Option<String>,
    pub convert_target: Option<i32>,
    pub core_type: Option<String>,
    pub sort_order: Option<i32>,
    pub last_refreshed: Option<String>, // from Subscription.last_updated
    pub status: Option<String>,         // "ok" | "error" | "never" — from Subscription.status
    pub error_message: Option<String>,  // from Subscription.error_message
    pub refresh_interval: Option<i32>,  // from Subscription.update_interval
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProfileExtension {
    #[key]
    pub protocol_id: i64, // was profile_id

    pub delay: Option<i32>,
    pub speed: Option<i64>,
    pub sort_order: Option<i32>,
    pub ip_info: Option<String>,
    /// Provenance of `delay`: `DELAY_SOURCE_FAST` / `DELAY_SOURCE_REAL` /
    /// `DELAY_SOURCE_UDP`. `None` = no measurement recorded.
    pub delay_source: Option<i32>,

    #[belongs_to(key = protocol_id, references = id)]
    pub protocol_row: Deferred<Option<ProtocolRow>>, // was protocol
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ServerStat {
    #[key]
    pub protocol_id: i64, // was profile_id

    pub today_up: Option<i64>,
    pub today_down: Option<i64>,
    pub total_up: Option<i64>,
    pub total_down: Option<i64>,
    pub last_updated: Option<String>,

    #[belongs_to(key = protocol_id, references = id)]
    pub protocol_row: Deferred<Option<ProtocolRow>>, // was protocol
}

#[derive(Debug, Clone, toasty::Model)]
pub struct RoutingRule {
    #[key]
    pub id: String,

    pub group_id: Option<String>,
    pub r#type: i32,
    pub domain_matcher: Option<String>,
    pub domains: Option<String>,
    pub ips: Option<String>,
    pub inbound_tags: Option<String>,
    pub port: Option<String>,
    pub source_ports: Option<String>,
    pub network: Option<String>,
    pub protocols: Option<String>,
    pub domain_strategy: Option<String>,
    pub outbound_tag: Option<String>,
    pub balancer_tag: Option<String>,
    pub rule_set_file: Option<String>,
    pub rule_set_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct DnsSetting {
    #[key]
    pub id: String,

    pub name: Option<String>,
    pub servers: Option<String>,
    pub hosts: Option<String>,
    pub query_strategy: Option<String>,
    pub disable_cache: Option<i32>,
    pub disable_fallback: Option<i32>,
    pub client_ip: Option<String>,
    /// TTL (secs) for the TUI-side DNS resolution cache; None = default 300.
    pub cache_ttl_secs: Option<i64>,
}

#[derive(Debug, Clone, toasty::Model)]
#[index(batch_id, status, ping_type)]
pub struct PingSession {
    #[key]
    pub id: String,

    pub batch_id: String,
    pub protocol_id: i64, // was profile_id
    pub config_type: i32,
    pub core_type: String,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub triplet_rank: i32,
    pub ping_type: String,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub speed_bps: Option<i64>,
    pub ip_info: Option<String>,
    pub error: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// ── Non-model types (plain structs for data passing) ─────────────────────

/// Result update for a single ping session — used to batch-write to DB.
#[derive(Debug, Clone)]
pub struct PingResultUpdate {
    pub session_id: String,
    pub protocol_id: i64, // was profile_id
    pub status: String,
    pub ping_type: String,
    pub latency_ms: Option<i32>,
    pub speed_bps: Option<i64>,
    pub ip_info: Option<String>,
    pub error: Option<String>,
}

// ── Data-transfer types ──────────────────────────────────────────────────

use std::collections::{HashMap, HashSet};

/// An endpoint with all its protocols, extensions, and stats.
#[derive(Debug, Clone)]
pub struct EndpointRow {
    pub endpoint: Endpoint,
    pub protocols: Vec<ProtocolRow>,
    pub extensions: HashMap<i64, ProfileExtension>,
    pub stats: HashMap<i64, ServerStat>,
    pub selected_protocol: usize,
    pub expanded: bool,
}

impl EndpointRow {
    /// Returns the currently active protocol row, respecting manual override.
    /// Falls back to `selected_protocol` if override is unset or protocol not found.
    #[must_use]
    pub fn active_protocol(&self) -> &ProtocolRow {
        // Check manual override first
        if let Some(override_id) = self.endpoint.manual_protocol_override
            && let Some(p) = self.protocols.iter().find(|p| p.id == override_id)
        {
            return p;
        }
        // Fall back to selected_protocol index
        self.protocols
            .get(self.selected_protocol)
            .unwrap_or_else(|| &self.protocols[0])
    }

    /// Tier for one protocol under the test-priority model (lower = better):
    /// 0 real-ok, 1 fast/udp-ok, 2 untested, 3 real-err, 4 fast-err,
    /// 5 name/dns-unresolved. Fresh failures dominate stored successes.
    fn protocol_test_tier(
        delay: Option<i32>,
        delay_source: Option<i32>,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
        pid: i64,
    ) -> u8 {
        if dns_unresolved {
            5
        } else if let Some((fast_failed, _)) = rounds
            && fast_failed.contains(&pid)
        {
            4
        } else if let Some((_, real_failed)) = rounds
            && real_failed.contains(&pid)
        {
            3
        } else if delay_source == Some(DELAY_SOURCE_REAL) {
            0
        } else if delay.is_some() {
            1
        } else {
            2
        }
    }

    /// Ascending sort key: `(tier, latency, recency, id)`. `recency` is
    /// negated so newer `last_seen_at` sorts first on ties. Only success
    /// tiers (0/1) rank by latency; untested and error/dns tiers use
    /// `i32::MAX` so they order by recency then id (design spec: "stable"
    /// within-tier key).
    fn protocol_test_key(
        p: &ProtocolRow,
        ext: Option<&ProfileExtension>,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) -> (u8, i32, i64, i64) {
        let delay = ext.and_then(|e| e.delay);
        let tier = Self::protocol_test_tier(
            delay,
            ext.and_then(|e| e.delay_source),
            dns_unresolved,
            rounds,
            p.id,
        );
        let latency = if tier <= 1 {
            delay.unwrap_or(i32::MAX)
        } else {
            i32::MAX
        };
        (tier, latency, -p.last_seen_at, p.id)
    }

    /// Re-sort `protocols` by test priority: real-ping success first, then
    /// fast/TCP/UDP success (latency ascending), then untested (newest
    /// `last_seen_at` first), then failures (real below fast below untested),
    /// then DNS-unresolved endpoints at the bottom. Deterministic tiebreak by
    /// protocol id. `rounds` is `(fast_failed, real_failed)`; `None` when no
    /// session state exists.
    pub fn sort_protocols_by_test_priority(
        &mut self,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) {
        self.protocols.sort_by_key(|p| {
            Self::protocol_test_key(p, self.extensions.get(&p.id), dns_unresolved, rounds)
        });
    }

    /// The endpoint's representative sort key = its best (minimum) protocol
    /// key — used by the main-table Test column sort.
    #[must_use]
    pub fn best_test_priority_key(
        &self,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) -> (u8, i32, i64, i64) {
        self.protocols
            .iter()
            .map(|p| Self::protocol_test_key(p, self.extensions.get(&p.id), dns_unresolved, rounds))
            .min()
            .unwrap_or((2, i32::MAX, 0, 0))
    }
}

/// Three-way toggle for the Profiles tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PurgatoryView {
    #[default]
    Active,
    Stale,
    All,
}

// ── Typed embed types (Task 8's 7-table model rewrite) ──────────────────
//
// Toasty embeds flatten into columns of the owning table (no separate
// tables). Proto's typed configs are stored opaque as deferred JSON columns.
// The legacy string/i32 columns in the models above stay until Task 8 removes
// them; these types are added now and exercised by the scratch-model tests.

use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{SecurityConfig, SecurityType, TransportType};

/// Endpoint id. Non-zero invariant — toasty has no `NonZero` column support,
/// so the constructor enforces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub struct EndpointId(pub i64);

impl EndpointId {
    /// Construct a new id. Panics (debug builds) when `v == 0`.
    #[must_use]
    pub const fn new(v: i64) -> Self {
        debug_assert!(v != 0, "EndpointId must be non-zero");
        Self(v)
    }
}

/// Protocol row id. Non-zero invariant — see [`EndpointId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub struct ProtocolId(pub i64);

impl ProtocolId {
    /// Construct a new id. Panics (debug builds) when `v == 0`.
    #[must_use]
    pub const fn new(v: i64) -> Self {
        debug_assert!(v != 0, "ProtocolId must be non-zero");
        Self(v)
    }
}

/// Endpoint host kind (replaces the legacy `host_type` string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum HostType {
    Ipv4,
    Ipv6,
    Dns,
    Undefined,
}

/// How a protocol row was configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum ConfigType {
    ShareUrl,
    Form,
}

/// Kind of latency/ping task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum TaskKind {
    FastPing,
    RealPing,
    UdpPing,
    UdpTest,
    SpeedTest,
}

/// Which error bucket a profile error belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum ProfileErr {
    Real,
    Fast,
    Name,
}

/// Group status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum GroupStatus {
    Ok,
    Error,
    Never,
}

/// Group core type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum GroupCoreType {
    Auto,
    Xray,
    SingBox,
}

/// IP selection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum QueryStrategy {
    UseIp,
    UseIpv4,
    UseIpv6,
}

/// Conversion target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum ConvertTarget {
    Clash,
}

/// Error info attached to a failed test.
#[derive(Debug, Clone, PartialEq, Eq, toasty::Embed)]
pub struct ErrorInfo {
    pub kind: ProfileErr,
    pub text: String,
}

/// Traffic accounting for an endpoint/group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub struct TrafficStats {
    pub today_up: i64,
    pub today_down: i64,
    pub total_up: i64,
    pub total_down: i64,
}

/// Transport-layer config. `data` is deferred + opaque JSON (not queryable).
#[derive(Debug, Clone, toasty::Embed)]
pub struct Transport {
    pub r#type: TransportType,
    #[column(type = text)]
    pub data: Deferred<Json<TransportConfig>>,
}

/// Security (TLS/Reality) config. `data` is deferred + opaque JSON.
#[derive(Debug, Clone, toasty::Embed)]
pub struct Security {
    pub r#type: SecurityType,
    pub sni: Option<String>,
    pub fp: Option<String>,
    pub insecure: Option<bool>,
    #[column(type = text)]
    pub data: Deferred<Json<SecurityConfig>>,
}

/// Latency of one probe, real or fast. Both variants share the `delay`
/// column (`latency_delay`); `ip` exists only on `Real` (`latency_ip`).
#[derive(Debug, Clone, PartialEq, toasty::Embed)]
pub enum Latency {
    Real {
        #[shared(delay)]
        delay: i32,
        ip: Option<String>,
    },
    Fast {
        #[shared(delay)]
        delay: i32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use toasty::Deferred;

    /// Endpoint with protocols `(id, last_seen_at, delay, delay_source)`.
    /// `delay: None` = untested protocol (no extension entry).
    fn row(protos: &[(i64, i64, Option<i32>, Option<i32>)]) -> EndpointRow {
        let mut row = EndpointRow {
            endpoint: Endpoint {
                id: 1,
                host: "h.example".to_string(),
                host_type: "ipv4".to_string(),
                port: 443,
                port_spec_str: None,
                parent_id: None,
                last_source: None,
                created_at: 0,
                manual_protocol_override: None,
                resolved_as: None,
                resolved_at: None,
            },
            protocols: Vec::new(),
            extensions: HashMap::new(),
            stats: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        };
        for (id, last_seen, delay, src) in protos {
            row.protocols.push(ProtocolRow {
                id: *id,
                endpoint_id: 1,
                sig: 0,
                cred_hash: 0,
                proto_kind: String::new(),
                spec_blob: Vec::new(),
                config_type: 1,
                core_type: "xray".to_string(),
                transport: None,
                security: None,
                last_used_at: None,
                created_at: 0,
                last_seen_at: *last_seen,
                endpoint: Deferred::from(None::<Endpoint>),
                extension: Deferred::from(None::<ProfileExtension>),
                server_stat: Deferred::from(None::<ServerStat>),
            });
            if let Some(d) = delay {
                row.extensions.insert(
                    *id,
                    ProfileExtension {
                        protocol_id: *id,
                        delay: Some(*d),
                        speed: None,
                        sort_order: None,
                        ip_info: None,
                        delay_source: *src,
                        protocol_row: Deferred::from(None::<ProtocolRow>),
                    },
                );
            }
        }
        row
    }

    fn ids(r: &EndpointRow) -> Vec<i64> {
        r.protocols.iter().map(|p| p.id).collect()
    }

    fn failed(ids: &[i64]) -> HashSet<i64> {
        ids.iter().copied().collect()
    }

    #[test]
    fn real_ok_above_fast_ok_above_untested() {
        // real-ok 200ms outranks fast-ok 10ms — tier beats latency.
        let mut r = row(&[
            (10, 1, Some(200), Some(DELAY_SOURCE_REAL)), // real-ok
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),  // fast-ok
            (30, 3, None, None),                         // untested
        ]);
        r.sort_protocols_by_test_priority(false, None);
        assert_eq!(ids(&r), vec![10, 20, 30]);
    }

    #[test]
    fn latency_orders_within_success_tiers() {
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_FAST)),
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),
            (30, 3, Some(120), Some(DELAY_SOURCE_REAL)),
            (40, 4, Some(90), Some(DELAY_SOURCE_REAL)),
        ]);
        r.sort_protocols_by_test_priority(false, None);
        // real tier first (30:120, 40:90 by latency), then fast tier (20:10, 10:50)
        assert_eq!(ids(&r), vec![40, 30, 20, 10]);
    }

    #[test]
    fn fresh_failure_dominates_stored_success() {
        // 10 has a stored real-ok delay but failed real this round -> sinks
        // below the untested 30; 20 failed fast -> below 10 (fast worse than real).
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_REAL)),
            (20, 2, Some(80), Some(DELAY_SOURCE_FAST)),
            (30, 3, None, None),
        ]);
        r.sort_protocols_by_test_priority(false, Some((&failed(&[20]), &failed(&[10]))));
        assert_eq!(ids(&r), vec![30, 10, 20]);
    }

    #[test]
    fn both_failed_uses_fast_tier() {
        let mut r = row(&[(10, 1, None, None), (20, 2, None, None)]);
        r.sort_protocols_by_test_priority(false, Some((&failed(&[10]), &failed(&[10]))));
        assert_eq!(ids(&r), vec![20, 10]);
    }

    #[test]
    fn dns_unresolved_sinks_all_protocols() {
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_REAL)),
            (20, 2, None, None),
        ]);
        r.sort_protocols_by_test_priority(true, None);
        assert_eq!(ids(&r), vec![20, 10]); // untested first; dns tier wins for both
    }

    #[test]
    fn untested_keeps_last_seen_recency_order() {
        let mut r = row(&[
            (10, 5, None, None),
            (20, 9, None, None),
            (30, 1, None, None),
        ]);
        r.sort_protocols_by_test_priority(false, None);
        assert_eq!(ids(&r), vec![20, 10, 30]); // newest first
    }

    #[test]
    fn best_key_returns_min_over_protocols() {
        let r = row(&[
            (10, 1, Some(200), Some(DELAY_SOURCE_REAL)),
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),
            (30, 3, None, None),
        ]);
        // Best = real-ok (tier 0), latency 200
        assert_eq!(r.best_test_priority_key(false, None), (0, 200, -1, 10));
    }

    // ── Scratch-model probes: typed embeds end-to-end in SQLite ──────────
    //
    // Task 8 rewires the real models onto the typed embed types above; these
    // tests pin the embed behavior (shared columns, deferred JSON,
    // enum/struct round-trips) in an in-memory DB now, before the rewrite.

    use xray_tui_proto::proto_spec::common::WebSocketConfig;
    use xray_tui_proto::proto_spec::{TlsConfig, TlsOpts};

    #[derive(Debug, toasty::Model)]
    struct ScratchEmbedProbe {
        #[key]
        #[auto]
        id: i64,
        latency: Option<Latency>,
        transport: Transport,
        security: Security,
        traffic: TrafficStats,
        kind: TaskKind,
        err: Option<ErrorInfo>,
    }

    /// Fresh in-memory DB holding only the scratch probe model.
    async fn probe_db() -> toasty::Db {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(ScratchEmbedProbe))
            .build(driver)
            .await
            .expect("build db");
        db.push_schema().await.expect("push schema");
        db
    }

    fn tcp_transport() -> Transport {
        Transport {
            r#type: TransportType::Tcp,
            data: Deferred::from(Json(TransportConfig::Tcp)),
        }
    }

    fn no_security() -> Security {
        Security {
            r#type: SecurityType::None,
            sni: None,
            fp: None,
            insecure: None,
            data: Deferred::from(Json(SecurityConfig::default())),
        }
    }

    /// Zeroed traffic stats — the flattened columns default to 0 anyway, but
    /// the create! validation requires the non-Option embed field.
    fn zero_traffic() -> TrafficStats {
        TrafficStats {
            today_up: 0,
            today_down: 0,
            total_up: 0,
            total_down: 0,
        }
    }

    #[tokio::test]
    async fn latency_shared_delay_column_roundtrip() {
        let mut db = probe_db().await;

        let mut created = toasty::create!(ScratchEmbedProbe {
            latency: Some(Latency::Real {
                delay: 42,
                ip: Some("1.2.3.4".to_string()),
            }),
            transport: tcp_transport(),
            security: no_security(),
            traffic: zero_traffic(),
            kind: TaskKind::RealPing,
        })
        .exec(&mut db)
        .await
        .expect("create");

        assert_eq!(
            created.latency,
            Some(Latency::Real {
                delay: 42,
                ip: Some("1.2.3.4".to_string()),
            })
        );

        // Round-trips as Real (shared delay column + variant ip column).
        let read = ScratchEmbedProbe::filter_by_id(created.id)
            .get(&mut db)
            .await
            .expect("read back");
        assert_eq!(
            read.latency,
            Some(Latency::Real {
                delay: 42,
                ip: Some("1.2.3.4".to_string()),
            })
        );

        // Variant switch Real -> Fast: the shared `latency_delay` column
        // carries 99 into the new variant; `latency_ip` clears to NULL.
        toasty::update!(created {
            latency: Some(Latency::Fast { delay: 99 }),
        })
        .exec(&mut db)
        .await
        .expect("update");

        let read = ScratchEmbedProbe::filter_by_id(created.id)
            .get(&mut db)
            .await
            .expect("read back");
        assert_eq!(read.latency, Some(Latency::Fast { delay: 99 }));
    }

    #[tokio::test]
    async fn transport_security_json_roundtrip() {
        let mut db = probe_db().await;

        let ws = TransportConfig::Ws(WebSocketConfig {
            host: Some("x".into()),
            path: Some("/p".into()),
            headers: None,
            ..Default::default()
        });
        let tls = SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                sni: Some("example.com".into()),
                alpn: None,
                fp: Some("chrome".into()),
                insecure: Some(true),
                ..Default::default()
            })),
            enc: None,
        };

        let created = toasty::create!(ScratchEmbedProbe {
            transport: Transport {
                r#type: TransportType::Ws,
                data: Deferred::from(Json(ws.clone())),
            },
            security: Security {
                r#type: SecurityType::Tls,
                sni: Some("example.com".to_string()),
                fp: Some("chrome".to_string()),
                insecure: Some(true),
                data: Deferred::from(Json(tls.clone())),
            },
            traffic: zero_traffic(),
            kind: TaskKind::SpeedTest,
        })
        .exec(&mut db)
        .await
        .expect("create");

        // INSERT ... RETURNING echoes the supplied values — deferred JSON
        // arrives loaded.
        assert!(!created.transport.data.is_unloaded());
        assert_eq!(&ws, &created.transport.data.get().0);
        assert!(!created.security.data.is_unloaded());
        assert_eq!(&tls, &created.security.data.get().0);

        // A default read leaves the deferred JSON unloaded; `.include()`
        // loads the same query.
        let read = ScratchEmbedProbe::filter_by_id(created.id)
            .include(ScratchEmbedProbe::fields().transport().data())
            .include(ScratchEmbedProbe::fields().security().data())
            .get(&mut db)
            .await
            .expect("read back");
        assert!(!read.transport.data.is_unloaded());
        assert_eq!(&ws, &read.transport.data.get().0);
        assert_eq!(read.transport.r#type, TransportType::Ws);
        assert!(!read.security.data.is_unloaded());
        assert_eq!(&tls, &read.security.data.get().0);
        assert_eq!(read.security.r#type, SecurityType::Tls);
        assert_eq!(read.security.sni.as_deref(), Some("example.com"));
        assert_eq!(read.security.fp.as_deref(), Some("chrome"));
        assert_eq!(read.security.insecure, Some(true));
    }

    #[tokio::test]
    async fn task_kind_and_error_roundtrip() {
        let mut db = probe_db().await;

        let mut created = toasty::create!(ScratchEmbedProbe {
            transport: tcp_transport(),
            security: no_security(),
            traffic: TrafficStats {
                today_up: 1,
                today_down: 2,
                total_up: 3,
                total_down: 4,
            },
            kind: TaskKind::UdpTest,
            err: Some(ErrorInfo {
                kind: ProfileErr::Real,
                text: "timeout".to_string(),
            }),
        })
        .exec(&mut db)
        .await
        .expect("create");

        let read = ScratchEmbedProbe::filter_by_id(created.id)
            .get(&mut db)
            .await
            .expect("read back");
        assert_eq!(read.kind, TaskKind::UdpTest);
        assert_eq!(
            read.err,
            Some(ErrorInfo {
                kind: ProfileErr::Real,
                text: "timeout".to_string(),
            })
        );
        assert_eq!(
            read.traffic,
            TrafficStats {
                today_up: 1,
                today_down: 2,
                total_up: 3,
                total_down: 4,
            }
        );

        // Every TaskKind variant round-trips through the discriminant column.
        for kind in [
            TaskKind::FastPing,
            TaskKind::RealPing,
            TaskKind::UdpPing,
            TaskKind::UdpTest,
            TaskKind::SpeedTest,
        ] {
            toasty::update!(created { kind: kind })
                .exec(&mut db)
                .await
                .expect("update kind");
            let read = ScratchEmbedProbe::filter_by_id(created.id)
                .get(&mut db)
                .await
                .expect("read back");
            assert_eq!(read.kind, kind);
        }
    }

    #[test]
    fn endpoint_id_accepts_nonzero() {
        assert_eq!(EndpointId::new(42).0, 42);
        assert_eq!(ProtocolId::new(7).0, 7);
    }

    #[test]
    #[should_panic(expected = "EndpointId must be non-zero")]
    fn endpoint_id_rejects_zero() {
        let _ = EndpointId::new(0);
    }

    #[test]
    #[should_panic(expected = "ProtocolId must be non-zero")]
    fn protocol_id_rejects_zero() {
        let _ = ProtocolId::new(0);
    }
}
