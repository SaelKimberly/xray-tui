use toasty::Deferred;

// ── Primary models (toasty ORM, mapped to DB tables) ────────────────────

/// Profile with uid-based PK. uid = sig ^ cred_hash (computed by ProtoSpec).
#[derive(Debug, Clone, toasty::Model)]
pub struct Profile {
    #[key]
    pub id: i64,                   // = uid = sig ^ cred_hash

    pub sig: i64,                  // cached from ProtoSpec::sig()
    pub cred_hash: i64,            // cached from ProtoSpec::cred_hash()
    pub proto_kind: String,        // "vmess", "vless", "trojan", etc.
    pub spec_blob: Vec<u8>,        // postcard-encoded ProtocolConfig variant

    pub config_type: i32,          // kept for core routing
    pub core_type: String,         // "auto", "xray", "sing-box"
    pub address: String,           // cached from ProtoSpec::host()
    pub port: i32,                 // cached from ProtoSpec::port()
    pub transport: Option<String>, // cached from ProtoSpec::transport_type()
    pub security: Option<String>,  // cached from ProtoSpec::security_type()
    pub created_at: i64,           // UNIX timestamp

    // — Relations —
    #[has_one]
    pub extension: Deferred<Option<ProfileExtension>>,

    #[has_one]
    pub server_stat: Deferred<Option<ServerStat>>,
}

/// Many-to-many connection between profiles and groups.
/// Replaces the old Profile.group_id field.
#[derive(Debug, Clone, toasty::Model)]
#[unique(profile_id, group_id)]
pub struct Connection {
    #[key]
    pub id: String,                 // UUID — toasty requires PK
    pub profile_id: i64,            // -> profiles.id
    pub group_id: String,           // -> groups.id

    pub remarks: Option<String>,
    pub seen_at: Option<String>,    // subscription last-seen timestamp
    pub is_sub: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_active: Option<i32>,
    pub updated_at: i64,

    #[belongs_to(key = profile_id, references = id)]
    pub profile: Deferred<Option<Profile>>,

    #[belongs_to(key = group_id, references = id)]
    pub group: Deferred<Option<Group>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProfileExtension {
    #[key]
    pub profile_id: i64,

    pub delay: Option<i32>,
    pub speed: Option<i32>,
    pub sort_order: Option<i32>,
    pub ip_info: Option<String>,

    #[belongs_to(key = profile_id, references = id)]
    pub profile: Deferred<Option<Profile>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ServerStat {
    #[key]
    pub profile_id: i64,

    pub today_up: Option<i64>,
    pub today_down: Option<i64>,
    pub total_up: Option<i64>,
    pub total_down: Option<i64>,
    pub last_updated: Option<String>,

    #[belongs_to(key = profile_id, references = id)]
    pub profile: Deferred<Option<Profile>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Group {
    #[key]
    pub id: String,

    pub name: Option<String>,
    pub subscription_url: Option<String>,
    pub subscription_enabled: Option<i32>,
    pub user_agent: Option<String>,
    pub convert_target: Option<i32>,
    pub core_type: Option<String>,
    pub sort_order: Option<i32>,
    pub is_system: Option<i32>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Subscription {
    #[key]
    pub id: String,

    pub group_id: Option<String>,
    pub url: String,
    pub last_updated: Option<String>,
    pub update_interval: Option<i32>,
    pub user_agent: Option<String>,
    pub status: Option<String>,
    pub error_message: Option<String>,
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
    pub profile_id: i64,
    pub config_type: i32,
    pub core_type: String,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub triplet_rank: i32,
    pub ping_type: String,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub speed_bps: Option<i32>,
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
    pub profile_id: i64,
    pub status: String,
    pub ping_type: String,
    pub latency_ms: Option<i32>,
    pub speed_bps: Option<i32>,
    pub ip_info: Option<String>,
    pub error: Option<String>,
}
