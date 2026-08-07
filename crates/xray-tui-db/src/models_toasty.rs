//! Toasty models for the typed data model (phase B).
//!
//! Seven tables replace the old nine: `endpoints`, `protocols`,
//! `profile_stats` (per endpoint-protocol pair state), `endpoint_groups`
//! (many-to-many link), `groups`, `routing_rules`, `dns_settings`.
//! All values are typed — newtype ids, embedded enums/structs, jiff
//! timestamps, deferred JSON config blobs — and every read in
//! [`crate::database`] goes through the typed query API.

// allow(clippy::used_underscore_binding): toasty-macros synthesizes `_0`-named
// parameters in the `Update` builder setters for unnamed tuple fields
// (`EndpointId`, `ProtocolId`) and uses them. The binding names are generated
// by the third-party derive with the field's span, so a struct-level allow
// cannot reach them (the generated impls are sibling items); the lint level
// must be set at this module. No handwritten code here uses underscore-
// prefixed bindings.
#![allow(clippy::used_underscore_binding)]

use std::collections::HashMap;

use jiff::Timestamp;
use toasty::{Deferred, Json};
use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{
    CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
};

// ── Typed embed types ───────────────────────────────────────────────────
//
// Toasty embeds flatten into columns of the owning table (no separate
// tables). Proto's typed configs are stored opaque as deferred JSON columns.

/// Endpoint id. Non-zero invariant — toasty has no `NonZero` column support,
/// so the constructor enforces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, toasty::Embed)]
pub struct EndpointId(i64);

impl EndpointId {
    /// Construct a new id. Panics (debug builds) when `v == 0`.
    #[must_use]
    pub const fn new(v: i64) -> Self {
        debug_assert!(v != 0, "EndpointId must be non-zero");
        Self(v)
    }

    /// The raw column value.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// Protocol row id. Non-zero invariant — see [`EndpointId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, toasty::Embed)]
pub struct ProtocolId(i64);

impl ProtocolId {
    /// Construct a new id. Panics (debug builds) when `v == 0`.
    #[must_use]
    pub const fn new(v: i64) -> Self {
        debug_assert!(v != 0, "ProtocolId must be non-zero");
        Self(v)
    }

    /// The raw column value.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
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
#[derive(Debug, Clone, PartialEq, Eq, toasty::Embed)]
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

// ── Primary models (toasty ORM, mapped to DB tables) ────────────────────

/// Endpoint: a network endpoint identified by host+port.
#[derive(Debug, Clone, toasty::Model)]
#[table = "endpoints"]
pub struct Endpoint {
    #[key]
    pub id: EndpointId, // stable_hash(host, port) for known types; stable_hash("undefined", config_uid) for exotic
    pub host: String, // canonical host string; empty for undefined
    pub host_type: HostType,
    pub port: u16,                     // primary port; 0 for undefined
    pub ports: Vec<u16>,               // full port spec; empty when single-port
    pub parent_id: Option<EndpointId>, // resolved IP -> DnsName parent
    pub last_source: Option<String>,   // hash of source subscription
    /// Manual protocol override (FK -> protocols.id); NULL = auto-select best.
    pub manual_protocol_override: Option<ProtocolId>,
    /// Cached DNS resolution of `host` for `host_type == Dns`: the resolved
    /// IP strings. Empty = not resolved yet or host is an IP. Persisted so
    /// launches do not re-resolve.
    pub resolved_as: Vec<String>,
    /// Timestamp of the `resolved_as` lookup; NULL = never / IP host.
    pub resolved_at: Option<Timestamp>,
    #[auto]
    pub created_at: Timestamp,
    #[has_many]
    pub links: Deferred<Vec<ProfileStats>>,
    #[has_many]
    pub group_links: Deferred<Vec<EndpointGroup>>,
}

/// `Protocol`: a protocol configuration.
/// PK = uid = sig ^ `cred_hash`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "protocols"]
pub struct Protocol {
    #[key]
    pub id: ProtocolId, // = uid = sig ^ cred_hash (protocol essentials only)
    pub sig: i64,
    pub cred_hash: i64,
    pub proto_kind: ProtocolKind,
    pub transport: Transport, // embed (T7): type + Deferred<Json<TransportConfig>>
    pub security: Security,   // embed (T7): type/sni/fp/insecure + Deferred<Json<SecurityConfig>>
    /// Full exact definition, sans host/port.
    #[column(type = text)]
    pub config: Deferred<Json<ProtocolConfig>>,
    #[auto]
    pub created_at: Timestamp,
    #[has_many]
    pub links: Deferred<Vec<ProfileStats>>,
}

/// Per endpoint-protocol pair state (traffic, latency, errors).
#[derive(Debug, Clone, toasty::Model)]
#[table = "profile_stats"]
#[key(protocol_id, endpoint_id)]
pub struct ProfileStats {
    // Indexed so the `Endpoint::links` / `Protocol::links` has_many relations
    // (and the batched `endpoint_id IN (...)` read) can use them.
    #[index]
    pub protocol_id: ProtocolId,
    #[index]
    pub endpoint_id: EndpointId,
    pub core_type: CoreType, // per-pair override (resolved at parse, overridable)
    pub config_type: ConfigType,
    pub last_used_at: Option<Timestamp>,
    pub last_seen_at: Timestamp,  // per-link staleness tracking
    pub task_id: Option<u16>,     // current task slot; 0 never valid
    pub task_queue: Vec<u16>,     // FIFO of queued task ids
    pub latency: Option<Latency>, // embed enum, shared delay column
    pub speed_bps: Option<i64>,
    pub error: Option<ErrorInfo>, // persisted failure marker
    pub traffic: TrafficStats,    // today/total up/down
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,
    #[version]
    pub version: u64, // optimistic concurrency
    #[belongs_to(key = protocol_id, references = id)]
    pub protocol: Deferred<Option<Protocol>>,
    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,
}

/// Many-to-many link between endpoints and groups.
/// Replaces Connection (was `profile_id/group_id`, now `endpoint_id/group_id`).
#[derive(Debug, Clone, toasty::Model)]
#[table = "endpoint_groups"]
#[key(endpoint_id, group_id)]
pub struct EndpointGroup {
    // Indexed so `Endpoint::group_links` is queryable and the per-group
    // membership filter can use it.
    #[index]
    pub endpoint_id: EndpointId,
    #[index]
    pub group_id: String,
    pub last_seen_at: Timestamp, // per-source last confirmation
    pub sort_order: Option<i32>,
    #[belongs_to(key = endpoint_id, references = id)]
    pub endpoint: Deferred<Option<Endpoint>>,
    #[belongs_to(key = group_id, references = id)]
    pub group: Deferred<Option<Group>>,
}

/// Group: merged with old Subscription fields. Removed `is_system`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "groups"]
pub struct Group {
    #[key]
    pub id: String, // UUID
    pub name: Option<String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub user_agent: Option<String>,
    pub convert_target: Option<ConvertTarget>,
    pub core_type: Option<GroupCoreType>, // form allows "auto"
    pub sort_order: Option<i32>,
    pub last_refreshed: Option<Timestamp>,
    pub status: Option<GroupStatus>,
    pub error_message: Option<String>,
    pub refresh_interval: Option<i64>, // minutes; None = default 1440 (24h)
}

#[derive(Debug, Clone, toasty::Model)]
#[table = "routing_rules"]
pub struct RoutingRule {
    #[key]
    pub id: String,

    pub group_id: Option<String>,
    pub r#type: i32, // opaque free-form number, no consumer; kept for form
    pub domain_matcher: Option<String>,
    pub domains: Vec<String>,
    pub ips: Vec<String>,
    pub inbound_tags: Vec<String>,
    pub ports: Vec<u16>, // "80,443" -> [80, 443]
    pub source_ports: Vec<u16>,
    pub network: Option<String>,         // pass-through, comma-able
    pub protocols: Vec<String>,          // IANA protocol names, NOT ProtocolKind
    pub domain_strategy: Option<String>, // pass-through
    pub outbound_tag: Option<String>,
    pub balancer_tag: Option<String>,
    pub rule_set_file: Option<String>,
    pub rule_set_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, toasty::Model)]
#[table = "dns_settings"]
pub struct DnsSetting {
    #[key]
    pub id: String,

    pub name: Option<String>,
    pub servers: Vec<String>,
    pub hosts: Vec<String>,
    pub query_strategy: Option<QueryStrategy>,
    pub disable_cache: bool,
    pub disable_fallback: bool,
    pub client_ip: Option<String>,
    /// TTL (secs) for the TUI-side DNS resolution cache; None = default 300.
    pub cache_ttl_secs: Option<i64>,
}

// ── Data-transfer types ──────────────────────────────────────────────────

/// An endpoint with its per-pair links and their protocols, as loaded by the
/// typed read paths in [`crate::database`].
#[derive(Debug, Clone)]
pub struct EndpointRow {
    pub endpoint: Endpoint,
    pub links: Vec<ProfileStats>, // per-pair state, sorted by test priority
    pub protocols: HashMap<ProtocolId, Protocol>, // included via links
    pub selected_protocol: usize, // index into links
    pub expanded: bool,
}

impl EndpointRow {
    /// Returns the currently active link, respecting manual override.
    /// Falls back to `selected_protocol` if the override is unset or the
    /// override protocol is not found among the links.
    #[must_use]
    pub fn active_link(&self) -> Option<&ProfileStats> {
        if let Some(pid) = self.endpoint.manual_protocol_override
            && let Some(link) = self.links.iter().find(|l| l.protocol_id == pid)
        {
            return Some(link);
        }
        self.links.get(self.selected_protocol)
    }

    /// The active link together with its included [`Protocol`] row.
    #[must_use]
    pub fn active_protocol(&self) -> Option<(&ProfileStats, &Protocol)> {
        let link = self.active_link()?;
        let protocol = self.protocols.get(&link.protocol_id)?;
        Some((link, protocol))
    }

    /// Tier for one link under the test-priority model (lower = better):
    /// 0 real-ok, 1 fast-ok, 2 untested, 3 real-err, 4 fast-err,
    /// 5 dns-unresolved. `dns_unresolved` is endpoint-level: one flag for
    /// all links (`host_type == Dns` and no cached `resolved_as`).
    const fn link_test_tier(link: &ProfileStats, dns_unresolved: bool) -> u8 {
        if dns_unresolved {
            5
        } else if let Some(err) = &link.error {
            match err.kind {
                // A name-resolution failure surfaces on a real attempt, so it
                // shares the real-err bucket.
                ProfileErr::Real | ProfileErr::Name => 3,
                ProfileErr::Fast => 4,
            }
        } else {
            match link.latency {
                Some(Latency::Real { .. }) => 0,
                Some(Latency::Fast { .. }) => 1,
                None => 2,
            }
        }
    }

    /// Ascending sort key: `(tier, latency, recency, protocol_id)`. `recency`
    /// is the negated `last_seen_at` epoch so newer links sort first on ties.
    /// Only success tiers (0/1) rank by latency; untested and error/dns tiers
    /// use `i32::MAX` so they order by recency then protocol id.
    fn link_test_key(link: &ProfileStats, dns_unresolved: bool) -> (u8, i32, i64, i64) {
        let tier = Self::link_test_tier(link, dns_unresolved);
        let delay = match link.latency {
            Some(Latency::Real { delay, .. } | Latency::Fast { delay }) => delay,
            None => i32::MAX,
        };
        let latency = if tier <= 1 { delay } else { i32::MAX };
        (
            tier,
            latency,
            -link.last_seen_at.as_second(),
            link.protocol_id.get(),
        )
    }

    /// Re-sort `links` by test priority: real-ping success first, then fast
    /// success (latency ascending), then untested (newest `last_seen_at`
    /// first), then persisted failures (real below fast), then DNS-unresolved
    /// endpoints at the bottom. Deterministic tiebreak by protocol id.
    pub fn sort_links_by_test_priority(&mut self, dns_unresolved: bool) {
        self.links
            .sort_by_key(|l| Self::link_test_key(l, dns_unresolved));
    }

    /// Set `selected_protocol` (the single-row display preference) to the
    /// endpoint's best MEASURED link: a real success with the lowest delay,
    /// else a fast success with the lowest delay. Error markers do NOT
    /// disqualify a measured link — a link that has both a successful
    /// measurement and a later failure marker still carries the result the
    /// row should show (decision: display preference follows measurements;
    /// the sub-table order keeps full test-priority tiers, where fresh
    /// failures dominate). Untested links rank last; when nothing is
    /// measured the current selection is kept. A pinned manual override is
    /// unaffected — `active_link()` checks it first.
    pub fn select_best_measured_link(&mut self) {
        let Some((best, _)) = self
            .links
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                let (rank, delay) = match l.latency {
                    Some(Latency::Real { delay, .. }) => (0u8, delay),
                    Some(Latency::Fast { delay }) => (1u8, delay),
                    None => return None,
                };
                Some((i, (rank, delay)))
            })
            .min_by_key(|&(_, (rank, delay))| (rank, delay))
        else {
            return;
        };
        self.selected_protocol = best;
    }

    /// The endpoint's representative sort key = its best (minimum) link key —
    /// used by the main-table Test column sort. `None` when the endpoint has
    /// no links.
    #[must_use]
    pub fn best_test_priority_key(&self, dns_unresolved: bool) -> Option<(u8, i32, i64, i64)> {
        self.links
            .iter()
            .map(|l| Self::link_test_key(l, dns_unresolved))
            .min()
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

#[cfg(test)]
mod tests {
    use super::*;
    use toasty::Deferred;

    /// Endpoint with links `(protocol_id, last_seen_at_secs, latency, error)`.
    fn row(links: &[(i64, i64, Option<Latency>, Option<ErrorInfo>)]) -> EndpointRow {
        let mut row = EndpointRow {
            endpoint: Endpoint {
                id: EndpointId::new(1),
                host: "h.example".to_string(),
                host_type: HostType::Ipv4,
                port: 443,
                ports: Vec::new(),
                parent_id: None,
                last_source: None,
                manual_protocol_override: None,
                resolved_as: Vec::new(),
                resolved_at: None,
                created_at: Timestamp::from_second(0).expect("ts"),
                links: Deferred::default(),
                group_links: Deferred::default(),
            },
            links: Vec::new(),
            protocols: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        };
        for (pid, last_seen, latency, error) in links {
            row.links.push(ProfileStats {
                protocol_id: ProtocolId::new(*pid),
                endpoint_id: EndpointId::new(1),
                core_type: CoreType::Xray,
                config_type: ConfigType::ShareUrl,
                last_used_at: None,
                last_seen_at: Timestamp::from_second(*last_seen).expect("ts"),
                task_id: None,
                task_queue: Vec::new(),
                latency: latency.clone(),
                speed_bps: None,
                error: error.clone(),
                traffic: TrafficStats {
                    today_up: 0,
                    today_down: 0,
                    total_up: 0,
                    total_down: 0,
                },
                created_at: Timestamp::from_second(0).expect("ts"),
                updated_at: Timestamp::from_second(0).expect("ts"),
                version: 1,
                protocol: Deferred::default(),
                endpoint: Deferred::default(),
            });
        }
        row
    }

    fn ids(r: &EndpointRow) -> Vec<i64> {
        r.links.iter().map(|l| l.protocol_id.get()).collect()
    }

    fn real(delay: i32) -> Latency {
        Latency::Real { delay, ip: None }
    }

    fn fast(delay: i32) -> Latency {
        Latency::Fast { delay }
    }

    fn err(kind: ProfileErr) -> ErrorInfo {
        ErrorInfo {
            kind,
            text: "boom".to_string(),
        }
    }

    #[test]
    fn select_best_measured_link_prefers_real_ok_lowest_delay() {
        // fast-ok 44 + error marker, real-ok 50, untested — the measured
        // real-ok wins despite the error-carrying fast sibling (sub-table
        // tiers still rank the error low; the display preference follows
        // measurements).
        let mut r = row(&[
            (10, 1, Some(fast(44)), Some(err(ProfileErr::Fast))),
            (11, 2, Some(real(50)), None),
            (12, 3, None, None),
        ]);
        r.select_best_measured_link();
        assert_eq!(r.selected_protocol, 1, "real-ok lowest delay wins");
        assert_eq!(r.active_link().map(|l| l.protocol_id.get()), Some(11));
    }

    #[test]
    fn select_best_measured_link_fast_ok_when_no_real() {
        let mut r = row(&[
            (10, 1, Some(fast(90)), None),
            (11, 2, Some(fast(44)), None),
        ]);
        r.select_best_measured_link();
        assert_eq!(r.selected_protocol, 1, "fast-ok lowest delay wins");
    }

    #[test]
    fn select_best_measured_link_keeps_selection_when_unmeasured() {
        let mut r = row(&[(10, 1, None, None), (11, 2, None, Some(err(ProfileErr::Real)))]);
        r.select_best_measured_link();
        assert_eq!(r.selected_protocol, 0, "no measurement -> keep selection");
    }

    #[test]
    fn select_best_measured_link_error_with_latency_still_measured() {
        // A link that carries both a success and a later failure marker is
        // still the best measured link (user-facing: the row shows 44ms).
        let mut r = row(&[
            (10, 1, Some(fast(44)), Some(err(ProfileErr::Fast))),
            (11, 2, None, None),
        ]);
        r.select_best_measured_link();
        assert_eq!(r.selected_protocol, 0);
    }

    #[test]
    fn real_ok_above_fast_ok_above_untested() {
        // real-ok 200ms outranks fast-ok 10ms — tier beats latency.
        let mut r = row(&[
            (10, 1, Some(real(200)), None), // real-ok
            (20, 2, Some(fast(10)), None),  // fast-ok
            (30, 3, None, None),            // untested
        ]);
        r.sort_links_by_test_priority(false);
        assert_eq!(ids(&r), vec![10, 20, 30]);
    }

    #[test]
    fn latency_orders_within_success_tiers() {
        let mut r = row(&[
            (10, 1, Some(fast(50)), None),
            (20, 2, Some(fast(10)), None),
            (30, 3, Some(real(120)), None),
            (40, 4, Some(real(90)), None),
        ]);
        r.sort_links_by_test_priority(false);
        // real tier first (30:120, 40:90 by latency), then fast tier (20:10, 10:50)
        assert_eq!(ids(&r), vec![40, 30, 20, 10]);
    }

    #[test]
    fn fresh_failure_dominates_stored_success() {
        // 10 has a stored real-ok delay but a persisted real error -> sinks
        // below the untested 30; 20 has a fast error -> below 10 (fast worse
        // than real).
        let mut r = row(&[
            (10, 1, Some(real(50)), Some(err(ProfileErr::Real))),
            (20, 2, Some(fast(80)), Some(err(ProfileErr::Fast))),
            (30, 3, None, None),
        ]);
        r.sort_links_by_test_priority(false);
        assert_eq!(ids(&r), vec![30, 10, 20]);
    }

    #[test]
    fn both_failed_uses_fast_tier() {
        // Both links carry a fast error: tie on tier, order by recency.
        let mut r = row(&[
            (10, 1, None, Some(err(ProfileErr::Fast))),
            (20, 2, None, Some(err(ProfileErr::Fast))),
        ]);
        r.sort_links_by_test_priority(false);
        assert_eq!(ids(&r), vec![20, 10]);
    }

    #[test]
    fn dns_unresolved_sinks_all_protocols() {
        let mut r = row(&[(10, 1, Some(real(50)), None), (20, 2, None, None)]);
        r.sort_links_by_test_priority(true);
        assert_eq!(ids(&r), vec![20, 10]); // untested first; dns tier wins for both
    }

    #[test]
    fn untested_keeps_last_seen_recency_order() {
        let mut r = row(&[
            (10, 5, None, None),
            (20, 9, None, None),
            (30, 1, None, None),
        ]);
        r.sort_links_by_test_priority(false);
        assert_eq!(ids(&r), vec![20, 10, 30]); // newest first
    }

    #[test]
    fn best_key_returns_min_over_links() {
        let r = row(&[
            (10, 1, Some(real(200)), None),
            (20, 2, Some(fast(10)), None),
            (30, 3, None, None),
        ]);
        // Best = real-ok (tier 0), latency 200
        assert_eq!(r.best_test_priority_key(false), Some((0, 200, -1, 10)));
        // Empty links -> None
        let empty = row(&[]);
        assert_eq!(empty.best_test_priority_key(false), None);
    }

    #[test]
    fn active_link_respects_override() {
        let mut r = row(&[(10, 1, Some(real(50)), None), (20, 2, Some(fast(10)), None)]);
        r.endpoint.manual_protocol_override = Some(ProtocolId::new(20));
        assert_eq!(r.active_link().unwrap().protocol_id, ProtocolId::new(20));
        // Override to a protocol with no link -> fall back to selection.
        r.endpoint.manual_protocol_override = Some(ProtocolId::new(99));
        assert_eq!(r.active_link().unwrap().protocol_id, ProtocolId::new(10));
        // No links -> None.
        let empty = row(&[]);
        assert!(empty.active_link().is_none());
    }

    #[test]
    fn active_protocol_pairs_link_with_protocol() {
        let mut r = row(&[(10, 1, Some(real(50)), None)]);
        let protocol = Protocol {
            id: ProtocolId::new(10),
            sig: 10,
            cred_hash: 0,
            proto_kind: ProtocolKind::Vless,
            transport: Transport {
                r#type: TransportType::Tcp,
                data: Deferred::from(Json(TransportConfig::Tcp)),
            },
            security: Security {
                r#type: SecurityType::None,
                sni: None,
                fp: None,
                insecure: None,
                data: Deferred::from(Json(SecurityConfig::default())),
            },
            config: Deferred::from(Json(vless_config())),
            created_at: Timestamp::from_second(0).expect("ts"),
            links: Deferred::default(),
        };
        r.protocols.insert(protocol.id, protocol);
        let (link, proto) = r.active_protocol().expect("active protocol");
        assert_eq!(link.protocol_id, ProtocolId::new(10));
        assert_eq!(proto.proto_kind, ProtocolKind::Vless);
    }

    fn vless_config() -> ProtocolConfig {
        ProtocolConfig::Vless(xray_tui_proto::proto_spec::VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            uuid_origin: None,
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        })
    }

    // ── Scratch-model probes: typed embeds end-to-end in SQLite ──────────
    //
    // These tests pin the embed behavior (shared columns, deferred JSON,
    // enum/struct round-trips, newtype key columns) in an in-memory DB.

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

    /// Endpoint's newtype key column round-trips through the real model.
    #[tokio::test]
    async fn endpoint_id_newtype_column_roundtrip() {
        let driver = toasty_driver_turso::Turso::in_memory();
        let mut db = toasty::Db::builder()
            .models(toasty::models!(Endpoint))
            .build(driver)
            .await
            .expect("build db");
        db.push_schema().await.expect("push schema");

        let created = toasty::create!(Endpoint {
            id: EndpointId::new(42),
            host: "1.2.3.4".to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::<u16>::new(),
            resolved_as: Vec::<String>::new(),
        })
        .exec(&mut db)
        .await
        .expect("create");

        assert_eq!(created.id, EndpointId::new(42));

        let read = Endpoint::filter_by_id(EndpointId::new(42))
            .get(&mut db)
            .await
            .expect("read back");
        assert_eq!(read.id.get(), 42);
        assert_eq!(read.port, 443);
    }

    #[test]
    fn endpoint_id_accepts_nonzero() {
        assert_eq!(EndpointId::new(42).get(), 42);
        assert_eq!(ProtocolId::new(7).get(), 7);
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
