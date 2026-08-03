use toasty::Deferred;

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
}

/// `ProtocolRow`: a protocol configuration. Replaces Profile.
/// PK = uid = sig ^ `cred_hash` (same as old Profile.id).
#[derive(Debug, Clone, toasty::Model)]
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
    pub remarks: Option<String>,
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

use std::collections::HashMap;
use std::net::IpAddr;

/// An endpoint with all its protocols, extensions, stats, and resolution info.
#[derive(Debug, Clone)]
pub struct EndpointRow {
    pub endpoint: Endpoint,
    pub protocols: Vec<ProtocolRow>,
    pub extensions: HashMap<i64, ProfileExtension>,
    pub stats: HashMap<i64, ServerStat>,
    pub resolved_ips: Vec<IpAddr>,
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
}

/// Three-way toggle for the Profiles tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PurgatoryView {
    #[default]
    Active,
    Stale,
    All,
}
