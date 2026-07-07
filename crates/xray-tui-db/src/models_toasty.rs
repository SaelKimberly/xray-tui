use toasty::Deferred;

// ── Primary models (toasty ORM, mapped to DB tables) ────────────────────

#[derive(Debug, Clone, toasty::Model)]
#[unique(group_id, sub_uid)]
pub struct Profile {
    #[key]
    pub id: String,

    /// Hash-based dedup key; unique per group.
    pub sub_uid: i64,
    pub group_id: String,

    // — Protocol config (formerly profile_cores table) —
    pub config_type: i32,
    pub core_type: String,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub user_id: Option<String>,
    pub security: Option<String>,
    pub network: Option<String>,
    pub stream_settings: Option<String>,
    pub protocol_settings: Option<String>,

    // — Profile metadata (formerly group_profiles table) —
    pub remarks: Option<String>,
    pub is_sub: Option<i32>,
    pub sub_id: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<i32>,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,

    /// Optimistic concurrency version — auto-managed by toasty.
    #[version]
    pub version: u64,

    // — Relations (query-only; toasty manages FK at ORM layer) —
    #[belongs_to(key = group_id, references = id)]
    pub group: Deferred<Option<Group>>,

    #[has_one]
    pub extension: Deferred<Option<ProfileExtension>>,

    #[has_one]
    pub server_stat: Deferred<Option<ServerStat>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProfileExtension {
    #[key]
    pub profile_id: String,

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
    pub profile_id: String,

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
    pub profile_id: String,
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
    pub profile_id: String,
    pub status: String,
    pub ping_type: String,
    pub latency_ms: Option<i32>,
    pub speed_bps: Option<i32>,
    pub ip_info: Option<String>,
    pub error: Option<String>,
}


impl Profile {
    /// Hash the semantic profile fields to produce a dedup key.
    /// Uses rapidhash v3 for stable output across crate versions.
    #[must_use]
    #[allow(
        clippy::cast_possible_wrap,
        reason = "u64 bit pattern stored in i64, not arithmetic"
    )]
    pub fn compute_sub_uid(&self) -> i64 {
        use rapidhash::v3::{DEFAULT_RAPID_SECRETS, RapidStreamHasherV3};
        let mut h = RapidStreamHasherV3::new(&DEFAULT_RAPID_SECRETS);
        h.write(&self.config_type.to_le_bytes());
        h.write(b":");
        h.write(self.address.as_deref().unwrap_or("").as_bytes());
        h.write(b":");
        h.write(&self.port.unwrap_or(0).to_le_bytes());
        h.write(b":");
        h.write(self.user_id.as_deref().unwrap_or("").as_bytes());
        h.write(b":");
        h.write(self.security.as_deref().unwrap_or("").as_bytes());
        h.write(b":");
        h.write(self.network.as_deref().unwrap_or("").as_bytes());
        h.write(b":");
        h.write(self.stream_settings.as_deref().unwrap_or("").as_bytes());
        h.write(b":");
        h.write(self.protocol_settings.as_deref().unwrap_or("").as_bytes());
        h.finish() as i64
    }
}
// ──── Constants ──────────────────────────────────────────────────────────

pub const GRAVEYARD_GROUP_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const GRAVEYARD_GROUP_TTL_HOURS: i64 = 24;
pub const ALL_GROUP_ID: &str = "00000000-0000-0000-0000-000000000000";
