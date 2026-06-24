pub mod models;
pub mod schema;

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("turso error: {0}")]
    Turso(#[from] turso::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("{0}")]
    Generic(String),
}

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Result tuple returned by `get_all_profiles_with_details`.
pub type ProfileWithDetails = (Profile, Option<ProfileExtension>, Option<ServerStat>);

pub struct Database {
    conn: turso::Connection,
}

// ── Column enums ──────────────────────────────────────────────────────

/// Column indices for Profile JOIN query (gp + pc, 19 cols).
#[repr(usize)]
enum ProfileCol {
    Id = 0,
    SubUid,
    GroupId,
    Remarks,
    IsSub,
    SubId,
    SortOrder,
    IsActive,
    UpdatedAt,
    ConfigType,
    CoreType,
    Address,
    Port,
    UserId,
    Security,
    Network,
    StreamSettings,
    ProtocolSettings,
    CreatedAt,
}

/// Column indices for groups table (SELECT * order).
#[repr(usize)]
enum GroupCol {
    Id = 0,
    Name,
    SubscriptionUrl,
    SubscriptionEnabled,
    UserAgent,
    ConvertTarget,
    CoreType,
    SortOrder,
    IsSystem,
}

/// Column indices for subscriptions table (SELECT * order).
#[repr(usize)]
enum SubscriptionCol {
    Id = 0,
    GroupId,
    Url,
    LastUpdated,
    UpdateInterval,
    UserAgent,
    Status,
    ErrorMessage,
}

/// Column indices for profile_extensions table (SELECT * order).
#[repr(usize)]
enum ProfileExtCol {
    ProfileId = 0,
    Delay,
    Speed,
    SortOrder,
    IpInfo,
}

/// Column indices for server_stats table (SELECT * order).
#[repr(usize)]
enum ServerStatCol {
    ProfileId = 0,
    TodayUp,
    TodayDown,
    TotalUp,
    TotalDown,
    LastUpdated,
}

/// Column indices for routing_rules table (SELECT * order).
#[repr(usize)]
enum RoutingRuleCol {
    Id = 0,
    GroupId,
    Type,
    DomainMatcher,
    Domains,
    Ips,
    InboundTags,
    Port,
    SourcePorts,
    Network,
    Protocols,
    DomainStrategy,
    OutboundTag,
    BalancerTag,
    RuleSetFile,
    RuleSetUrl,
    SortOrder,
}

/// Column indices for dns_settings table (SELECT * order).
#[repr(usize)]
enum DnsSettingCol {
    Id = 0,
    Name,
    Servers,
    Hosts,
    QueryStrategy,
    DisableCache,
    DisableFallback,
    ClientIp,
}

/// Column indices for the 3-table LEFT JOIN query (30 cols).
/// Profile columns 0-18, extension columns 19-23, stats columns 24-29.
#[repr(usize)]
#[allow(dead_code)]
enum ProfileDetailsCol {
    // Profile columns (0-18) — same order as ProfileCol
    Id = 0,
    SubUid,
    GroupId,
    Remarks,
    IsSub,
    SubId,
    SortOrder,
    IsActive,
    UpdatedAt,
    ConfigType,
    CoreType,
    Address,
    Port,
    UserId,
    Security,
    Network,
    StreamSettings,
    ProtocolSettings,
    CreatedAt,
    // Extension columns (19-23)
    ExtProfileId,
    Delay,
    Speed,
    ExtSortOrder,
    IpInfo,
    // Stats columns (24-29)
    StatsProfileId,
    TodayUp,
    TodayDown,
    TotalUp,
    TotalDown,
    LastUpdated,
}

// ── Row conversion impls ──────────────────────────────────────────────

impl Profile {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(ProfileCol::Id as usize)?,
            config_type: row.get::<i32>(ProfileCol::ConfigType as usize)?,
            core_type: row.get::<String>(ProfileCol::CoreType as usize)?,
            remarks: row.get::<Option<String>>(ProfileCol::Remarks as usize)?,
            address: row.get::<Option<String>>(ProfileCol::Address as usize)?,
            port: row.get::<Option<i32>>(ProfileCol::Port as usize)?,
            user_id: row.get::<Option<String>>(ProfileCol::UserId as usize)?,
            security: row.get::<Option<String>>(ProfileCol::Security as usize)?,
            network: row.get::<Option<String>>(ProfileCol::Network as usize)?,
            stream_settings: row.get::<Option<String>>(ProfileCol::StreamSettings as usize)?,
            protocol_settings: row.get::<Option<String>>(ProfileCol::ProtocolSettings as usize)?,
            is_sub: row.get::<Option<i32>>(ProfileCol::IsSub as usize)?,
            sub_id: row.get::<Option<String>>(ProfileCol::SubId as usize)?,
            group_id: row.get::<Option<String>>(ProfileCol::GroupId as usize)?,
            sort_order: row.get::<Option<i32>>(ProfileCol::SortOrder as usize)?,
            is_active: row.get::<Option<i32>>(ProfileCol::IsActive as usize)?,
            created_at: row.get::<Option<String>>(ProfileCol::CreatedAt as usize)?,
            updated_at: row.get::<Option<String>>(ProfileCol::UpdatedAt as usize)?,
            sub_uid: row.get::<Option<i64>>(ProfileCol::SubUid as usize)?,
        })
    }
}

impl Subscription {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(SubscriptionCol::Id as usize)?,
            group_id: row.get::<Option<String>>(SubscriptionCol::GroupId as usize)?,
            url: row.get::<String>(SubscriptionCol::Url as usize)?,
            last_updated: row.get::<Option<String>>(SubscriptionCol::LastUpdated as usize)?,
            update_interval: row.get::<Option<i32>>(SubscriptionCol::UpdateInterval as usize)?,
            user_agent: row.get::<Option<String>>(SubscriptionCol::UserAgent as usize)?,
            status: row.get::<Option<String>>(SubscriptionCol::Status as usize)?,
            error_message: row.get::<Option<String>>(SubscriptionCol::ErrorMessage as usize)?,
        })
    }
}

impl Group {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(GroupCol::Id as usize)?,
            name: row.get::<Option<String>>(GroupCol::Name as usize)?,
            subscription_url: row.get::<Option<String>>(GroupCol::SubscriptionUrl as usize)?,
            subscription_enabled: row.get::<Option<i32>>(GroupCol::SubscriptionEnabled as usize)?,
            user_agent: row.get::<Option<String>>(GroupCol::UserAgent as usize)?,
            convert_target: row.get::<Option<i32>>(GroupCol::ConvertTarget as usize)?,
            core_type: row.get::<Option<String>>(GroupCol::CoreType as usize)?,
            sort_order: row.get::<Option<i32>>(GroupCol::SortOrder as usize)?,
            is_system: row.get::<Option<i32>>(GroupCol::IsSystem as usize)?,
        })
    }
}

impl ProfileExtension {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            profile_id: row.get::<String>(ProfileExtCol::ProfileId as usize)?,
            delay: row.get::<Option<i32>>(ProfileExtCol::Delay as usize)?,
            speed: row.get::<Option<i32>>(ProfileExtCol::Speed as usize)?,
            sort_order: row.get::<Option<i32>>(ProfileExtCol::SortOrder as usize)?,
            ip_info: row.get::<Option<String>>(ProfileExtCol::IpInfo as usize)?,
        })
    }
}

impl ServerStat {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            profile_id: row.get::<String>(ServerStatCol::ProfileId as usize)?,
            today_up: row.get::<Option<i32>>(ServerStatCol::TodayUp as usize)?,
            today_down: row.get::<Option<i32>>(ServerStatCol::TodayDown as usize)?,
            total_up: row.get::<Option<i32>>(ServerStatCol::TotalUp as usize)?,
            total_down: row.get::<Option<i32>>(ServerStatCol::TotalDown as usize)?,
            last_updated: row.get::<Option<String>>(ServerStatCol::LastUpdated as usize)?,
        })
    }
}

impl RoutingRule {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(RoutingRuleCol::Id as usize)?,
            group_id: row.get::<Option<String>>(RoutingRuleCol::GroupId as usize)?,
            r#type: row.get::<i32>(RoutingRuleCol::Type as usize)?,
            domain_matcher: row.get::<Option<String>>(RoutingRuleCol::DomainMatcher as usize)?,
            domains: row.get::<Option<String>>(RoutingRuleCol::Domains as usize)?,
            ips: row.get::<Option<String>>(RoutingRuleCol::Ips as usize)?,
            inbound_tags: row.get::<Option<String>>(RoutingRuleCol::InboundTags as usize)?,
            port: row.get::<Option<String>>(RoutingRuleCol::Port as usize)?,
            source_ports: row.get::<Option<String>>(RoutingRuleCol::SourcePorts as usize)?,
            network: row.get::<Option<String>>(RoutingRuleCol::Network as usize)?,
            protocols: row.get::<Option<String>>(RoutingRuleCol::Protocols as usize)?,
            domain_strategy: row.get::<Option<String>>(RoutingRuleCol::DomainStrategy as usize)?,
            outbound_tag: row.get::<Option<String>>(RoutingRuleCol::OutboundTag as usize)?,
            balancer_tag: row.get::<Option<String>>(RoutingRuleCol::BalancerTag as usize)?,
            rule_set_file: row.get::<Option<String>>(RoutingRuleCol::RuleSetFile as usize)?,
            rule_set_url: row.get::<Option<String>>(RoutingRuleCol::RuleSetUrl as usize)?,
            sort_order: row.get::<Option<i32>>(RoutingRuleCol::SortOrder as usize)?,
        })
    }
}

impl DnsSetting {
    fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(DnsSettingCol::Id as usize)?,
            name: row.get::<Option<String>>(DnsSettingCol::Name as usize)?,
            servers: row.get::<Option<String>>(DnsSettingCol::Servers as usize)?,
            hosts: row.get::<Option<String>>(DnsSettingCol::Hosts as usize)?,
            query_strategy: row.get::<Option<String>>(DnsSettingCol::QueryStrategy as usize)?,
            disable_cache: row.get::<Option<i32>>(DnsSettingCol::DisableCache as usize)?,
            disable_fallback: row.get::<Option<i32>>(DnsSettingCol::DisableFallback as usize)?,
            client_ip: row.get::<Option<String>>(DnsSettingCol::ClientIp as usize)?,
        })
    }
}

// ── Database ──────────────────────────────────────────────────────────

impl Database {
    pub async fn open(path: &Path) -> Result<Self> {
        let path_str = path
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;
        use turso::Builder;
        let db = Builder::new_local(path_str)
            .build()
            .await
            .map_err(DatabaseError::Turso)?;
        let conn = db.connect().map_err(DatabaseError::Turso)?;
        let db_ = Self { conn };
        db_.initialize_schema().await?;
        Ok(db_)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:")
            .build()
            .await
            .map_err(DatabaseError::Turso)?;
        let conn = db.connect().map_err(DatabaseError::Turso)?;
        let db_ = Self { conn };
        db_.initialize_schema().await?;
        Ok(db_)
    }

    async fn initialize_schema(&self) -> Result<()> {
        schema::create_tables(&self.conn).await?;

        // Ensure graveyard group exists
        let mut stmt = self
            .conn
            .prepare_cached("SELECT COUNT(*) FROM groups WHERE id = ?1")
            .await?;
        let count: i64 = match stmt
            .query_row(turso::params![models::GRAVEYARD_GROUP_ID])
            .await
        {
            Ok(row) => row.get(0).unwrap_or(0),
            Err(_) => 0,
        };
        if count == 0 {
            self.conn
                .execute(
                    "INSERT INTO groups (id, name, subscription_enabled, is_system) VALUES (?1, ?2, 0, 1)",
                    turso::params![models::GRAVEYARD_GROUP_ID, "sub-graveyard"],
                )
                .await?;
        }

        // Ensure "All" system group exists
        let _ = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO groups (id, name, subscription_enabled, is_system) VALUES (?1, ?2, 0, 1)",
                turso::params![models::ALL_GROUP_ID, "All"],
            )
            .await;
        Ok(())
    }

    /// One-time backfill: normalize all existing profile remarks.
    /// Uses user_version pragma to run exactly once.
    pub async fn normalize_all_remarks(&self) -> Result<()> {
        // Query user_version via PRAGMA
        let mut stmt = self.conn.prepare_cached("PRAGMA user_version").await?;
        let version: i32 = stmt
            .query_row(())
            .await
            .ok()
            .and_then(|row| row.get::<i32>(0).ok())
            .unwrap_or(0);

        if version >= 1 {
            return Ok(());
        }

        let profiles = self.get_all_profiles().await?;
        let mut count = 0u32;

        // Use Transaction::new_unchecked since unchecked_transaction may not exist
        let tx = self.conn.unchecked_transaction().await?;

        for p in &profiles {
            if let Some(ref r) = p.remarks {
                let normalized = normalize_remark(r);
                if &normalized != r {
                    tx.execute(
                        "UPDATE group_profiles SET remarks = ?1 WHERE id = ?2",
                        turso::params![normalized, p.id.as_str()],
                    )
                    .await?;
                    count += 1;
                }
            }
        }
        tx.commit().await.map_err(DatabaseError::Turso)?;

        if count > 0 {
            eprintln!("Backfilled {count} profile remarks");
        }

        // Set user_version via PRAGMA
        let _ = self.conn.pragma_update("user_version", 1).await?;
        Ok(())
    }
}

// ── Helper functions ──────────────────────────────────────────────────

/// Percent-decode a string. Fallback to original on failure.
fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val(bytes[i + 1]);
            let lo = hex_val(bytes[i + 2]);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Normalize a remark string: percent-decode, trim, collapse inner whitespace.
fn normalize_remark(s: &str) -> String {
    let decoded = percent_decode(s);
    let trimmed = decoded.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

use models::{DnsSetting, Group, Profile, ProfileExtension, RoutingRule, ServerStat, Subscription};

// ── Query methods ────────────────────────────────────────────────────

impl Database {
    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT gp.id, gp.sub_uid, gp.group_id, gp.remarks, gp.is_sub, gp.sub_id,
                    gp.sort_order, gp.is_active, gp.updated_at,
                    pc.config_type, pc.core_type, pc.address, pc.port, pc.user_id,
                    pc.security, pc.network, pc.stream_settings, pc.protocol_settings,
                    COALESCE(gp.created_at, pc.created_at) AS created_at
                 FROM group_profiles gp
                 JOIN profile_cores pc ON pc.sub_uid = gp.sub_uid
                 ORDER BY gp.sort_order",
            )
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut profiles = Vec::new();
        while let Some(row) = rows.next().await? {
            profiles.push(Profile::from_row(&row)?);
        }
        Ok(profiles)
    }

    pub async fn get_profiles_by_group(&self, group_id: &str) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT gp.id, gp.sub_uid, gp.group_id, gp.remarks, gp.is_sub, gp.sub_id,
                    gp.sort_order, gp.is_active, gp.updated_at,
                    pc.config_type, pc.core_type, pc.address, pc.port, pc.user_id,
                    pc.security, pc.network, pc.stream_settings, pc.protocol_settings,
                    COALESCE(gp.created_at, pc.created_at) AS created_at
                 FROM group_profiles gp
                 JOIN profile_cores pc ON pc.sub_uid = gp.sub_uid
                 WHERE gp.group_id = ?1
                 ORDER BY gp.sort_order",
            )
            .await?;
        let mut rows = stmt.query(turso::params![group_id]).await?;
        let mut profiles = Vec::new();
        while let Some(row) = rows.next().await? {
            profiles.push(Profile::from_row(&row)?);
        }
        Ok(profiles)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM groups ORDER BY sort_order")
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut groups = Vec::new();
        while let Some(row) = rows.next().await? {
            groups.push(Group::from_row(&row)?);
        }
        Ok(groups)
    }

    pub async fn get_profile_extension(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileExtension>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM profile_extensions WHERE profile_id = ?1")
            .await?;
        let mut rows = stmt.query(turso::params![profile_id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(ProfileExtension::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_server_stats(&self, profile_id: &str) -> Result<Option<ServerStat>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM server_stats WHERE profile_id = ?1")
            .await?;
        let mut rows = stmt.query(turso::params![profile_id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(ServerStat::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_profiles_with_details(&self) -> Result<Vec<ProfileWithDetails>> {
        let query = "
            SELECT
                gp.id, gp.sub_uid, gp.group_id, gp.remarks, gp.is_sub, gp.sub_id,
                gp.sort_order, gp.is_active, gp.updated_at,
                pc.config_type, pc.core_type, pc.address, pc.port, pc.user_id,
                pc.security, pc.network, pc.stream_settings, pc.protocol_settings,
                COALESCE(gp.created_at, pc.created_at) AS created_at,
                pe.profile_id AS ext_profile_id,
                pe.delay,
                pe.speed,
                pe.sort_order AS ext_sort_order,
                pe.ip_info,
                ss.profile_id AS stats_profile_id,
                ss.today_up,
                ss.today_down,
                ss.total_up,
                ss.total_down,
                ss.last_updated
            FROM group_profiles gp
            JOIN profile_cores pc ON pc.sub_uid = gp.sub_uid
            LEFT JOIN profile_extensions pe ON pe.profile_id = gp.id
            LEFT JOIN server_stats ss ON ss.profile_id = gp.id
            ORDER BY gp.sort_order
        ";
        let mut stmt = self.conn.prepare_cached(query).await?;
        let mut rows = stmt.query(()).await?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let profile = Profile::from_row(&row)?;

            let ext_profile_id: Option<String> =
                row.get(ProfileDetailsCol::ExtProfileId as usize)?;
            let extension = if ext_profile_id.is_some() {
                Some(ProfileExtension {
                    profile_id: ext_profile_id.unwrap_or_default(),
                    delay: row.get::<Option<i32>>(ProfileDetailsCol::Delay as usize)?,
                    speed: row.get::<Option<i32>>(ProfileDetailsCol::Speed as usize)?,
                    sort_order: row.get::<Option<i32>>(ProfileDetailsCol::ExtSortOrder as usize)?,
                    ip_info: row.get::<Option<String>>(ProfileDetailsCol::IpInfo as usize)?,
                })
            } else {
                None
            };

            let stats_profile_id: Option<String> =
                row.get(ProfileDetailsCol::StatsProfileId as usize)?;
            let stats = if stats_profile_id.is_some() {
                Some(ServerStat {
                    profile_id: stats_profile_id.unwrap_or_default(),
                    today_up: row.get::<Option<i32>>(ProfileDetailsCol::TodayUp as usize)?,
                    today_down: row.get::<Option<i32>>(ProfileDetailsCol::TodayDown as usize)?,
                    total_up: row.get::<Option<i32>>(ProfileDetailsCol::TotalUp as usize)?,
                    total_down: row.get::<Option<i32>>(ProfileDetailsCol::TotalDown as usize)?,
                    last_updated: row
                        .get::<Option<String>>(ProfileDetailsCol::LastUpdated as usize)?,
                })
            } else {
                None
            };

            results.push((profile, extension, stats));
        }
        Ok(results)
    }

    // ── Write methods ─────────────────────────────────────────────────

    pub async fn insert_profile(&self, p: &Profile) -> Result<()> {
        let sub_uid = p.sub_uid.unwrap_or(0);
        if sub_uid == 0 {
            return Err(DatabaseError::Generic(
                "Cannot insert profile with sub_uid=0".into(),
            ));
        }

        // 1. Insert or ignore core data
        self.conn
            .execute(
                "INSERT OR IGNORE INTO profile_cores (sub_uid, config_type, core_type, address, port, user_id, security, network, stream_settings, protocol_settings, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                turso::params![
                    sub_uid, p.config_type, p.core_type.as_str(), p.address.as_deref(), p.port,
                    p.user_id.as_deref(), p.security.as_deref(), p.network.as_deref(), p.stream_settings.as_deref(), p.protocol_settings.as_deref(),
                    p.created_at.as_deref()
                ],
            )
            .await?;

        // 2. Insert group profile (target group)
        let group_id = p.group_id.as_deref().unwrap_or(models::GRAVEYARD_GROUP_ID);
        self.conn
            .execute(
                "INSERT OR REPLACE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                turso::params![
                    p.id.as_str(), sub_uid, group_id, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(),
                    p.sort_order, p.is_active, p.updated_at.as_deref(), p.created_at.as_deref()
                ],
            )
            .await?;

        // 3. Mirror to "All" group (same core, different group id)
        if group_id != models::ALL_GROUP_ID && group_id != models::GRAVEYARD_GROUP_ID {
            let all_id = format!("{}-all", p.id);
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    turso::params![
                        all_id.as_str(), sub_uid, models::ALL_GROUP_ID, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(),
                        p.sort_order, p.is_active, p.updated_at.as_deref(), p.created_at.as_deref()
                    ],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn update_profile(&self, p: &Profile) -> Result<()> {
        let sub_uid = p.sub_uid.unwrap_or(0);
        if sub_uid == 0 {
            return Err(DatabaseError::Generic(
                "Cannot update profile with sub_uid=0".into(),
            ));
        }
        self.conn
            .execute(
                "INSERT OR REPLACE INTO profile_cores (sub_uid, config_type, core_type, address, port, user_id, security, network, stream_settings, protocol_settings, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                turso::params![
                    sub_uid, p.config_type, p.core_type.as_str(), p.address.as_deref(), p.port,
                    p.user_id.as_deref(), p.security.as_deref(), p.network.as_deref(), p.stream_settings.as_deref(), p.protocol_settings.as_deref(),
                    p.created_at.as_deref()
                ],
            )
            .await?;
        let group_id = p.group_id.as_deref().unwrap_or(models::GRAVEYARD_GROUP_ID);
        self.conn
            .execute(
                "UPDATE group_profiles SET sub_uid=?1, group_id=?2, remarks=?3, is_sub=?4, sub_id=?5, sort_order=?6, is_active=?7, updated_at=?8 WHERE id=?9",
                turso::params![
                    sub_uid, group_id, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(), p.sort_order, p.is_active,
                    p.updated_at.as_deref(), p.id.as_str()
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let sub_uid: Option<i64> = {
            let mut stmt = self
                .conn
                .prepare_cached("SELECT sub_uid FROM group_profiles WHERE id = ?1")
                .await?;
            match stmt.query_row(turso::params![id]).await {
                Ok(row) => row.get::<i64>(0).ok(),
                Err(_) => None,
            }
        };
        self.conn
            .execute(
                "DELETE FROM profile_extensions WHERE profile_id = ?1",
                turso::params![id],
            )
            .await?;
        self.conn
            .execute(
                "DELETE FROM server_stats WHERE profile_id = ?1",
                turso::params![id],
            )
            .await?;
        self.conn
            .execute(
                "DELETE FROM group_profiles WHERE id = ?1",
                turso::params![id],
            )
            .await?;
        // Also delete the ALL-group mirror entry if it exists
        let mirror_id = format!("{id}-all");
        self.conn
            .execute(
                "DELETE FROM group_profiles WHERE id = ?1",
                turso::params![mirror_id.as_str()],
            )
            .await?;
        if let Some(su) = sub_uid {
            let remaining: i64 = {
                let mut stmt = match self
                    .conn
                    .prepare_cached("SELECT COUNT(*) FROM group_profiles WHERE sub_uid = ?1")
                    .await
                {
                    Ok(stmt) => stmt,
                    Err(_) => return Ok(()),
                };
                match stmt.query_row(turso::params![su]).await {
                    Ok(row) => row.get::<i64>(0).unwrap_or(0),
                    Err(_) => 0,
                }
            };
            if remaining == 0 {
                self.conn
                    .execute(
                        "DELETE FROM profile_cores WHERE sub_uid = ?1",
                        turso::params![su],
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT gp.id, gp.sub_uid, gp.group_id, gp.remarks, gp.is_sub, gp.sub_id,
                    gp.sort_order, gp.is_active, gp.updated_at,
                    pc.config_type, pc.core_type, pc.address, pc.port, pc.user_id,
                    pc.security, pc.network, pc.stream_settings, pc.protocol_settings,
                    COALESCE(gp.created_at, pc.created_at) AS created_at
                 FROM group_profiles gp
                 JOIN profile_cores pc ON pc.sub_uid = gp.sub_uid
                 WHERE gp.id = ?1",
            )
            .await?;
        let mut rows = stmt.query(turso::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(Profile::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn clone_profile(&self, id: &str, new_id: &str) -> Result<()> {
        let original = self
            .get_profile(id)
            .await?
            .ok_or_else(|| DatabaseError::Generic(format!("profile not found: {id}")))?;
        let mut clone = original.clone();
        clone.id = new_id.to_string();
        self.insert_profile(&clone).await?;
        // Also copy profile_extension if exists
        if let Some(ext) = self.get_profile_extension(id).await? {
            let mut new_ext = ext.clone();
            new_ext.profile_id = new_id.to_string();
            self.upsert_profile_extension(&new_ext).await?;
        }
        Ok(())
    }

    pub async fn reorder_profiles(&self, ids: &[(String, i32)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction().await?;
        let mut stmt = tx
            .prepare("UPDATE group_profiles SET sort_order = ?1 WHERE id = ?2")
            .await
            .map_err(DatabaseError::Turso)?;
        for (id, order) in ids {
            stmt.execute(turso::params![order, id.as_str()])
                .await
                .map_err(DatabaseError::Turso)?;
        }
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) VALUES (?1, ?2, ?3, ?4, ?5)",
                turso::params![ext.profile_id.as_str(), ext.delay, ext.speed, ext.sort_order, ext.ip_info.as_deref()],
            )
            .await?;
        Ok(())
    }

    pub async fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO server_stats (profile_id, today_up, today_down, total_up, total_down, last_updated) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                turso::params![
                    stats.profile_id.as_str(), stats.today_up, stats.today_down, stats.total_up,
                    stats.total_down, stats.last_updated.as_deref()
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn insert_group(&self, g: &Group) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO groups (id, name, subscription_url, subscription_enabled, user_agent, convert_target, core_type, sort_order, is_system) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                turso::params![
                    g.id.as_str(), g.name.as_deref(), g.subscription_url.as_deref(), g.subscription_enabled, g.user_agent.as_deref(),
                    g.convert_target, g.core_type.as_deref(), g.sort_order, g.is_system
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn update_group(&self, g: &Group) -> Result<()> {
        self.conn
            .execute(
                "UPDATE groups SET name=?1, subscription_url=?2, subscription_enabled=?3, user_agent=?4, convert_target=?5, core_type=?6, sort_order=?7, is_system=?8 WHERE id=?9",
                turso::params![
                    g.name.as_deref(), g.subscription_url.as_deref(), g.subscription_enabled, g.user_agent.as_deref(),
                    g.convert_target, g.core_type.as_deref(), g.sort_order, g.is_system, g.id.as_str()
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn update_profile_active(&self, id: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction().await?;
        tx.execute(
            "UPDATE group_profiles SET is_active = 0 WHERE is_active = 1",
            (),
        )
        .await
        .map_err(DatabaseError::Turso)?;
        tx.execute(
            "UPDATE group_profiles SET is_active = 1 WHERE id = ?1",
            turso::params![id],
        )
        .await
        .map_err(DatabaseError::Turso)?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }
    pub async fn delete_group(&self, id: &str) -> Result<()> {
        // Prevent deletion of system groups
        let is_system: Option<i32> = {
            let mut stmt = match self
                .conn
                .prepare_cached("SELECT is_system FROM groups WHERE id = ?1")
                .await
            {
                Ok(stmt) => stmt,
                Err(_) => return Ok(()),
            };
            match stmt.query_row(turso::params![id]).await {
                Ok(row) => row.get::<i32>(0).ok(),
                Err(_) => None,
            }
        };
        if is_system == Some(1) {
            return Err(DatabaseError::Generic(
                "cannot delete system group".to_string(),
            ));
        }
        // Delete all profiles in this group first
        let profiles = self.get_profiles_by_group(id).await?;
        for p in &profiles {
            self.delete_profile(&p.id).await?;
        }
        self.conn
            .execute("DELETE FROM groups WHERE id = ?1", turso::params![id])
            .await?;
        Ok(())
    }

    /// Delete all profiles in a group but keep the group itself.
    /// Subscriptions linked to the group are preserved.
    pub async fn clear_group(&self, group_id: &str) -> Result<usize> {
        // Prevent clearing system groups
        let is_system: Option<i32> = {
            let mut stmt = match self
                .conn
                .prepare_cached("SELECT is_system FROM groups WHERE id = ?1")
                .await
            {
                Ok(stmt) => stmt,
                Err(_) => return Ok(0),
            };
            match stmt.query_row(turso::params![group_id]).await {
                Ok(row) => row.get::<i32>(0).ok(),
                Err(_) => None,
            }
        };
        if is_system == Some(1) {
            return Err(DatabaseError::Generic(
                "cannot clear system group".to_string(),
            ));
        }
        // Delete all profiles in this group (cascade: extensions + stats)
        let profiles = self.get_profiles_by_group(group_id).await?;
        for p in &profiles {
            self.delete_profile(&p.id).await?;
        }
        Ok(profiles.len())
    }

    pub async fn get_subscription_by_group(&self, group_id: &str) -> Result<Option<Subscription>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM subscriptions WHERE group_id = ?1 LIMIT 1")
            .await?;
        let mut rows = stmt.query(turso::params![group_id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(Subscription::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM subscriptions ORDER BY group_id")
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut subs = Vec::new();
        while let Some(row) = rows.next().await? {
            subs.push(Subscription::from_row(&row)?);
        }
        Ok(subs)
    }

    pub async fn upsert_subscription(&self, sub: &Subscription) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO subscriptions (id, group_id, url, last_updated, update_interval, user_agent, status, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                turso::params![
                    sub.id.as_str(), sub.group_id.as_deref(), sub.url.as_str(), sub.last_updated.as_deref(), sub.update_interval,
                    sub.user_agent.as_deref(), sub.status.as_deref(), sub.error_message.as_deref()
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn delete_subscriptions_by_group(&self, group_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM subscriptions WHERE group_id = ?1",
                turso::params![group_id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT g.* FROM groups g
                 LEFT JOIN subscriptions s ON s.group_id = g.id
                 WHERE g.subscription_enabled = 1
                   AND g.subscription_url IS NOT NULL
                   AND g.subscription_url != ''
                   AND (s.last_updated IS NULL
                        OR datetime(s.last_updated, '+' || COALESCE(s.update_interval, 1440) || ' minutes') < datetime('now'))",
            )
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut groups = Vec::new();
        while let Some(row) = rows.next().await? {
            groups.push(Group::from_row(&row)?);
        }
        Ok(groups)
    }

    pub async fn subscription_upsert_profiles(
        &self,
        group_id: &str,
        profiles: &[Profile],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction().await?;

        // 1. Upsert cores
        {
            let mut stmt = tx
                .prepare(
                    "INSERT OR REPLACE INTO profile_cores (sub_uid, config_type, core_type, address, port, user_id, security, network, stream_settings, protocol_settings, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )
                .await
                .map_err(DatabaseError::Turso)?;
            for p in profiles {
                let su = p.sub_uid.unwrap_or(0);
                if su == 0 {
                    continue;
                }
                stmt.execute(turso::params![
                    su,
                    p.config_type,
                    p.core_type.as_str(),
                    p.address.as_deref(),
                    p.port,
                    p.user_id.as_deref(),
                    p.security.as_deref(),
                    p.network.as_deref(),
                    p.stream_settings.as_deref(),
                    p.protocol_settings.as_deref(),
                    p.created_at.as_deref(),
                ])
                .await
                .map_err(DatabaseError::Turso)?;
            }
        }

        // 2. Upsert group profiles (target group) with dedup by sub_uid
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     ON CONFLICT(group_id, sub_uid) DO UPDATE SET
                     remarks=excluded.remarks, is_sub=excluded.is_sub, sub_id=excluded.sub_id,
                     sort_order=excluded.sort_order, is_active=excluded.is_active, updated_at=excluded.updated_at",
                )
                .await
                .map_err(DatabaseError::Turso)?;
            for p in profiles {
                let su = p.sub_uid.unwrap_or(0);
                if su == 0 {
                    continue;
                }
                stmt.execute(turso::params![
                    p.id.as_str(),
                    su,
                    group_id,
                    p.remarks.as_deref(),
                    p.is_sub,
                    p.sub_id.as_deref(),
                    p.sort_order,
                    p.is_active,
                    p.updated_at.as_deref(),
                    p.created_at.as_deref(),
                ])
                .await
                .map_err(DatabaseError::Turso)?;
            }
        }

        // 3. Upsert All group entries (same cores, different group)
        if group_id != models::ALL_GROUP_ID {
            let mut stmt = tx
                .prepare(
                    "INSERT OR IGNORE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                )
                .await
                .map_err(DatabaseError::Turso)?;
            for p in profiles {
                let su = p.sub_uid.unwrap_or(0);
                if su == 0 {
                    continue;
                }
                let all_id = format!("{}-all", p.id);
                stmt.execute(turso::params![
                    all_id.as_str(),
                    su,
                    models::ALL_GROUP_ID,
                    p.remarks.as_deref(),
                    p.is_sub,
                    p.sub_id.as_deref(),
                    p.sort_order,
                    p.is_active,
                    p.updated_at.as_deref(),
                    p.created_at.as_deref(),
                ])
                .await
                .map_err(DatabaseError::Turso)?;
            }
        }

        // 4. Promote graveyard orphans: remove graveyard rows for sub_uids now in this group
        if group_id != models::ALL_GROUP_ID && group_id != models::GRAVEYARD_GROUP_ID {
            let _removed = tx
                .execute(
                    "DELETE FROM group_profiles
                     WHERE group_id = ?1 AND sub_uid IN (
                         SELECT sub_uid FROM group_profiles WHERE group_id = ?2 AND sub_uid > 0
                     )",
                    turso::params![models::GRAVEYARD_GROUP_ID, group_id],
                )
                .await
                .unwrap_or(0);
        }

        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn move_orphans_to_graveyard(
        &self,
        group_id: &str,
        active_sub_uids: &[u64],
        graveyard_id: &str,
    ) -> Result<usize> {
        if active_sub_uids.is_empty() {
            return Ok(self
                .conn
                .execute(
                    "UPDATE group_profiles SET group_id = ?1, updated_at = datetime('now') WHERE group_id = ?2 AND is_sub = 1",
                    turso::params![graveyard_id, group_id],
                )
                .await
                .map_err(DatabaseError::Turso)? as usize);
        }
        let profiles_in_group: Vec<Profile> = {
            let mut stmt = self
                .conn
                .prepare_cached(
                    "SELECT gp.id, gp.sub_uid, gp.group_id, gp.remarks, gp.is_sub, gp.sub_id,
                        gp.sort_order, gp.is_active, gp.updated_at,
                        pc.config_type, pc.core_type, pc.address, pc.port, pc.user_id,
                        pc.security, pc.network, pc.stream_settings, pc.protocol_settings,
                        COALESCE(gp.created_at, pc.created_at) AS created_at
                     FROM group_profiles gp
                     JOIN profile_cores pc ON pc.sub_uid = gp.sub_uid
                     WHERE gp.group_id = ?1 AND gp.is_sub = 1",
                )
                .await?;
            let mut rows = stmt.query(turso::params![group_id]).await?;
            let mut v = Vec::new();
            while let Some(row) = rows.next().await? {
                v.push(Profile::from_row(&row)?);
            }
            v
        };
        let mut moved = 0;
        for p in &profiles_in_group {
            if !active_sub_uids.contains(&(p.sub_uid.unwrap_or(0) as u64)) {
                self.conn
                    .execute(
                        "UPDATE group_profiles SET group_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                        turso::params![graveyard_id, p.id.as_str()],
                    )
                    .await?;
                moved += 1;
            }
        }
        Ok(moved)
    }

    pub async fn purge_graveyard(&self, graveyard_id: &str, ttl_hours: i64) -> Result<usize> {
        let count = self
            .conn
            .execute(
                "DELETE FROM group_profiles WHERE group_id = ?1 AND updated_at < datetime('now', ?2)",
                turso::params![graveyard_id, format!("-{} hours", ttl_hours).as_str()],
            )
            .await
            .map_err(DatabaseError::Turso)? as usize;
        Ok(count)
    }

    // ── Routing rules ────────────────────────────────────────────────

    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM routing_rules ORDER BY sort_order")
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut rules = Vec::new();
        while let Some(row) = rows.next().await? {
            rules.push(RoutingRule::from_row(&row)?);
        }
        Ok(rules)
    }

    pub async fn insert_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO routing_rules (id, group_id, type, domain_matcher, domains, ips, inbound_tags, port, source_ports, network, protocols, domain_strategy, outbound_tag, balancer_tag, rule_set_file, rule_set_url, sort_order) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
                turso::params![
                    r.id.as_str(), r.group_id.as_deref(), r.r#type, r.domain_matcher.as_deref(), r.domains.as_deref(), r.ips.as_deref(),
                    r.inbound_tags.as_deref(), r.port.as_deref(), r.source_ports.as_deref(), r.network.as_deref(), r.protocols.as_deref(),
                    r.domain_strategy.as_deref(), r.outbound_tag.as_deref(), r.balancer_tag.as_deref(), r.rule_set_file.as_deref(),
                    r.rule_set_url.as_deref(), r.sort_order,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn update_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        self.conn
            .execute(
                "UPDATE routing_rules SET group_id=?1, type=?2, domain_matcher=?3, domains=?4, ips=?5, inbound_tags=?6, port=?7, source_ports=?8, network=?9, protocols=?10, domain_strategy=?11, outbound_tag=?12, balancer_tag=?13, rule_set_file=?14, rule_set_url=?15, sort_order=?16 WHERE id=?17",
                turso::params![
                    r.group_id.as_deref(), r.r#type, r.domain_matcher.as_deref(), r.domains.as_deref(), r.ips.as_deref(),
                    r.inbound_tags.as_deref(), r.port.as_deref(), r.source_ports.as_deref(), r.network.as_deref(), r.protocols.as_deref(),
                    r.domain_strategy.as_deref(), r.outbound_tag.as_deref(), r.balancer_tag.as_deref(), r.rule_set_file.as_deref(),
                    r.rule_set_url.as_deref(), r.sort_order, r.id.as_str(),
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn delete_routing_rule(&self, id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM routing_rules WHERE id = ?1",
                turso::params![id],
            )
            .await?;
        Ok(())
    }

    pub async fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction().await?;
        let mut stmt = tx
            .prepare("UPDATE routing_rules SET sort_order = ?1 WHERE id = ?2")
            .await
            .map_err(DatabaseError::Turso)?;
        for (id, order) in ids {
            stmt.execute(turso::params![order, id.as_str()])
                .await
                .map_err(DatabaseError::Turso)?;
        }
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    // ── DNS settings ─────────────────────────────────────────────────

    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM dns_settings LIMIT 1")
            .await?;
        let mut rows = stmt.query(()).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(DnsSetting::from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn upsert_dns_settings(&self, dns: &DnsSetting) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO dns_settings (id, name, servers, hosts, query_strategy, disable_cache, disable_fallback, client_ip) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                turso::params![
                    dns.id.as_str(), dns.name.as_deref(), dns.servers.as_deref(), dns.hosts.as_deref(), dns.query_strategy.as_deref(),
                    dns.disable_cache, dns.disable_fallback, dns.client_ip.as_deref()
                ],
            )
            .await?;
        Ok(())
    }
}
