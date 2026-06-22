pub mod models;
pub mod schema;

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
}

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Result tuple returned by `get_all_profiles_with_details`.
pub type ProfileWithDetails = (Profile, Option<ProfileExtension>, Option<ServerStat>);

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        schema::create_tables(&self.conn)?;

        // Ensure graveyard group exists
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM groups WHERE id = ?1",
                rusqlite::params![models::GRAVEYARD_GROUP_ID],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if count == 0 {
            self.conn.execute(
                "INSERT INTO groups (id, name, subscription_enabled) VALUES (?1, ?2, 0)",
                rusqlite::params![models::GRAVEYARD_GROUP_ID, "sub-graveyard"],
            )?;
        }
        Ok(())
    }

    /// One-time backfill: normalize all existing profile remarks.
    /// Uses user_version pragma to run exactly once.
    pub fn normalize_all_remarks(&self) -> Result<()> {
        let version: i32 = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap_or(0);
        if version >= 1 {
            return Ok(());
        }

        let profiles = self.get_all_profiles()?;
        let mut count = 0u32;
        let tx = self.conn.unchecked_transaction()?;
        for p in &profiles {
            if let Some(ref r) = p.remarks {
                let normalized = normalize_remark(r);
                if &normalized != r {
                    tx.execute(
                        "UPDATE profiles SET remarks = ?1 WHERE id = ?2",
                        rusqlite::params![normalized, p.id],
                    )?;
                    count += 1;
                }
            }
        }
        tx.commit()?;

        if count > 0 {
            eprintln!("Backfilled {count} profile remarks");
        }
        self.conn
            .pragma_update(None, "user_version", 1)?;
        Ok(())
    }

}

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
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Normalize a remark string: percent-decode, trim, collapse inner whitespace.
fn normalize_remark(s: &str) -> String {
    let decoded = percent_decode(s);
    let mut out = String::with_capacity(decoded.len());
    let mut prev_was_space = false;
    for c in decoded.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(c);
            prev_was_space = false;
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() { String::new() } else { trimmed.to_string() }
}

use models::{Group, Profile, ProfileExtension, RoutingRule, DnsSetting, ServerStat, Subscription};
use rusqlite::Row;

// ── Row conversion impls ──────────────────────────────────────────────

impl TryFrom<&Row<'_>> for Profile {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            config_type: row.get("config_type")?,
            core_type: row.get("core_type")?,
            remarks: row.get("remarks")?,
            address: row.get("address")?,
            port: row.get("port")?,
            user_id: row.get("user_id")?,
            security: row.get("security")?,
            network: row.get("network")?,
            stream_settings: row.get("stream_settings")?,
            protocol_settings: row.get("protocol_settings")?,
            is_sub: row.get("is_sub")?,
            sub_id: row.get("sub_id")?,
            group_id: row.get("group_id")?,
            sort_order: row.get("sort_order")?,
            is_active: row.get("is_active")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            sub_uid: row.get("sub_uid")?,
        })
    }
}

impl TryFrom<&Row<'_>> for Subscription {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            url: row.get("url")?,
            last_updated: row.get("last_updated")?,
            update_interval: row.get("update_interval")?,
            user_agent: row.get("user_agent")?,
            status: row.get("status")?,
            error_message: row.get("error_message")?,
        })
    }
}

impl TryFrom<&Row<'_>> for Group {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            subscription_url: row.get("subscription_url")?,
            subscription_enabled: row.get("subscription_enabled")?,
            user_agent: row.get("user_agent")?,
            convert_target: row.get("convert_target")?,
            core_type: row.get("core_type")?,
            sort_order: row.get("sort_order")?,
        })
    }
}

impl TryFrom<&Row<'_>> for ProfileExtension {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            profile_id: row.get("profile_id")?,
            delay: row.get("delay")?,
            speed: row.get("speed")?,
            sort_order: row.get("sort_order")?,
            ip_info: row.get("ip_info")?,
        })
    }
}

impl TryFrom<&Row<'_>> for ServerStat {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            profile_id: row.get("profile_id")?,
            today_up: row.get("today_up")?,
            today_down: row.get("today_down")?,
            total_up: row.get("total_up")?,
            total_down: row.get("total_down")?,
            last_updated: row.get("last_updated")?,
        })
    }
}

impl TryFrom<&Row<'_>> for RoutingRule {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            r#type: row.get("type")?,
            domain_matcher: row.get("domain_matcher")?,
            domains: row.get("domains")?,
            ips: row.get("ips")?,
            inbound_tags: row.get("inbound_tags")?,
            port: row.get("port")?,
            source_ports: row.get("source_ports")?,
            network: row.get("network")?,
            protocols: row.get("protocols")?,
            domain_strategy: row.get("domain_strategy")?,
            outbound_tag: row.get("outbound_tag")?,
            balancer_tag: row.get("balancer_tag")?,
            rule_set_file: row.get("rule_set_file")?,
            rule_set_url: row.get("rule_set_url")?,
            sort_order: row.get("sort_order")?,
        })
    }
}

impl TryFrom<&Row<'_>> for DnsSetting {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            servers: row.get("servers")?,
            hosts: row.get("hosts")?,
            query_strategy: row.get("query_strategy")?,
            disable_cache: row.get("disable_cache")?,
            disable_fallback: row.get("disable_fallback")?,
            client_ip: row.get("client_ip")?,
        })
    }
}

// ── Query methods ────────────────────────────────────────────────────

impl Database {
    pub fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM profiles ORDER BY sort_order")?;
        let rows = stmt.query_map([], |row| Profile::try_from(row))?;
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row?);
        }
        Ok(profiles)
    }

    pub fn get_profiles_by_group(&self, group_id: &str) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM profiles WHERE group_id = ?1 ORDER BY sort_order")?;
        let rows = stmt.query_map([group_id], |row| Profile::try_from(row))?;
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row?);
        }
        Ok(profiles)
    }

    pub fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM groups ORDER BY sort_order")?;
        let rows = stmt.query_map([], |row| Group::try_from(row))?;
        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }

    pub fn get_profile_extension(&self, profile_id: &str) -> Result<Option<ProfileExtension>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM profile_extensions WHERE profile_id = ?1")?;
        let mut rows = stmt.query_map([profile_id], |row| ProfileExtension::try_from(row))?;
        match rows.next() {
            Some(Ok(ext)) => Ok(Some(ext)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_server_stats(&self, profile_id: &str) -> Result<Option<ServerStat>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM server_stats WHERE profile_id = ?1")?;
        let mut rows = stmt.query_map([profile_id], |row| ServerStat::try_from(row))?;
        match rows.next() {
            Some(Ok(stats)) => Ok(Some(stats)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_profiles_with_details(&self) -> Result<Vec<ProfileWithDetails>> {
        let query = "
            SELECT
                p.*,
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
            FROM profiles p
            LEFT JOIN profile_extensions pe ON pe.profile_id = p.id
            LEFT JOIN server_stats ss ON ss.profile_id = p.id
            ORDER BY p.sort_order
        ";
        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let profile = Profile::try_from(row)?;

            let ext_profile_id: Option<String> = row.get("ext_profile_id")?;
            let extension = if ext_profile_id.is_some() {
                Some(ProfileExtension {
                    profile_id: ext_profile_id.unwrap_or_default(),
                    delay: row.get("delay").ok().flatten(),
                    speed: row.get("speed").ok().flatten(),
                    sort_order: row.get("ext_sort_order").ok().flatten(),
                    ip_info: row.get("ip_info").ok().flatten(),
                })
            } else {
                None
            };

            let stats_profile_id: Option<String> = row.get("stats_profile_id")?;
            let stats = if stats_profile_id.is_some() {
                Some(ServerStat {
                    profile_id: stats_profile_id.unwrap_or_default(),
                    today_up: row.get("today_up").ok().flatten(),
                    today_down: row.get("today_down").ok().flatten(),
                    total_up: row.get("total_up").ok().flatten(),
                    total_down: row.get("total_down").ok().flatten(),
                    last_updated: row.get("last_updated").ok().flatten(),
                })
            } else {
                None
            };

            Ok((profile, extension, stats))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
    // ── Write methods ─────────────────────────────────────────────────────

    pub fn insert_profile(&self, p: &Profile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO profiles (id, config_type, core_type, remarks, address, port, user_id, security, network, stream_settings, protocol_settings, is_sub, sub_id, group_id, sub_uid, sort_order, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            rusqlite::params![
                p.id, p.config_type, p.core_type, p.remarks, p.address, p.port,
                p.user_id, p.security, p.network, p.stream_settings, p.protocol_settings,
                p.is_sub, p.sub_id, p.group_id, p.sub_uid, p.sort_order, p.is_active,
                p.created_at, p.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_profile(&self, p: &Profile) -> Result<()> {
        self.conn.execute(
            "UPDATE profiles SET config_type=?1, core_type=?2, remarks=?3, address=?4, port=?5, user_id=?6, security=?7, network=?8, stream_settings=?9, protocol_settings=?10, is_sub=?11, sub_id=?12, group_id=?13, sub_uid=?14, sort_order=?15, is_active=?16, updated_at=?17 WHERE id=?18",
            rusqlite::params![
                p.config_type, p.core_type, p.remarks, p.address, p.port,
                p.user_id, p.security, p.network, p.stream_settings, p.protocol_settings,
                p.is_sub, p.sub_id, p.group_id, p.sub_uid, p.sort_order, p.is_active,
                p.updated_at, p.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_profile(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM profile_extensions WHERE profile_id = ?1",
            rusqlite::params![id],
        )?;
        self.conn.execute(
            "DELETE FROM server_stats WHERE profile_id = ?1",
            rusqlite::params![id],
        )?;
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        let mut stmt = self.conn.prepare("SELECT * FROM profiles WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| Profile::try_from(row))?;
        match rows.next() {
            Some(Ok(profile)) => Ok(Some(profile)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn clone_profile(&self, id: &str, new_id: &str) -> Result<()> {
        let original = self.get_profile(id)?.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName(format!("profile not found: {id}"))
        })?;
        let mut clone = original.clone();
        clone.id = new_id.to_string();
        self.insert_profile(&clone)?;
        // Also copy profile_extension if exists
        if let Some(ext) = self.get_profile_extension(id)? {
            let mut new_ext = ext.clone();
            new_ext.profile_id = new_id.to_string();
            self.upsert_profile_extension(&new_ext)?;
        }
        Ok(())
    }

    pub fn reorder_profiles(&self, ids: &[(String, i32)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE profiles SET sort_order = ?1 WHERE id = ?2")?;
            for (id, order) in ids {
                stmt.execute(rusqlite::params![order, id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![ext.profile_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info],
        )?;
        Ok(())
    }

    pub fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO server_stats (profile_id, today_up, today_down, total_up, total_down, last_updated) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![stats.profile_id, stats.today_up, stats.today_down, stats.total_up, stats.total_down, stats.last_updated],
        )?;
        Ok(())
    }

    pub fn insert_group(&self, g: &Group) -> Result<()> {
        self.conn.execute(
            "INSERT INTO groups (id, name, subscription_url, subscription_enabled, user_agent, convert_target, core_type, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![g.id, g.name, g.subscription_url, g.subscription_enabled, g.user_agent, g.convert_target, g.core_type, g.sort_order],
        )?;
        Ok(())
    }

    pub fn update_group(&self, g: &Group) -> Result<()> {
        self.conn.execute(
            "UPDATE groups SET name=?1, subscription_url=?2, subscription_enabled=?3, user_agent=?4, convert_target=?5, core_type=?6, sort_order=?7 WHERE id=?8",
            rusqlite::params![g.name, g.subscription_url, g.subscription_enabled, g.user_agent, g.convert_target, g.core_type, g.sort_order, g.id],
        )?;
        Ok(())
    }

    pub fn update_profile_active(&self, id: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            tx.execute("UPDATE profiles SET is_active = 0 WHERE is_active = 1", [])?;
            tx.execute(
                "UPDATE profiles SET is_active = 1 WHERE id = ?1",
                rusqlite::params![id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn delete_group(&self, id: &str) -> Result<()> {
        // Delete all profiles in this group first
        let profiles = self.get_profiles_by_group(id)?;
        for p in &profiles {
            self.delete_profile(&p.id)?;
        }
        self.conn
            .execute("DELETE FROM groups WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
    pub fn get_subscription_by_group(&self, group_id: &str) -> Result<Option<Subscription>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM subscriptions WHERE group_id = ?1 LIMIT 1")?;
        let mut rows = stmt.query_map([group_id], |row| Subscription::try_from(row))?;
        match rows.next() {
            Some(Ok(sub)) => Ok(Some(sub)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut stmt = self.conn.prepare("SELECT * FROM subscriptions ORDER BY group_id")?;
        let rows = stmt.query_map([], |row| Subscription::try_from(row))?;
        let mut subs = Vec::new();
        for row in rows {
            subs.push(row?);
        }
        Ok(subs)
    }

    pub fn upsert_subscription(&self, sub: &Subscription) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO subscriptions (id, group_id, url, last_updated, update_interval, user_agent, status, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![sub.id, sub.group_id, sub.url, sub.last_updated, sub.update_interval, sub.user_agent, sub.status, sub.error_message],
        )?;
        Ok(())
    }

    pub fn delete_subscriptions_by_group(&self, group_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM subscriptions WHERE group_id = ?1",
            rusqlite::params![group_id],
        )?;
        Ok(())
    }

    pub fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut stmt = self.conn.prepare(
            "SELECT g.* FROM groups g
             LEFT JOIN subscriptions s ON s.group_id = g.id
             WHERE g.subscription_enabled = 1
               AND g.subscription_url IS NOT NULL
               AND g.subscription_url != ''
               AND (s.last_updated IS NULL
                    OR datetime(s.last_updated, '+' || COALESCE(s.update_interval, 1440) || ' minutes') < datetime('now'))"
        )?;
        let rows = stmt.query_map([], |row| Group::try_from(row))?;
        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }

    pub fn subscription_upsert_profiles(
        &self,
        _group_id: &str,
        profiles: &[Profile],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut upsert = tx.prepare(
                "INSERT INTO profiles (id, config_type, core_type, remarks, address, port, user_id, security, network, stream_settings, protocol_settings, is_sub, sub_id, group_id, sub_uid, sort_order, is_active, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)
                 ON CONFLICT(group_id, sub_uid) WHERE sub_uid != 0 DO UPDATE SET
                 remarks=excluded.remarks, address=excluded.address, port=excluded.port,
                 user_id=excluded.user_id, security=excluded.security, network=excluded.network,
                 stream_settings=excluded.stream_settings, protocol_settings=excluded.protocol_settings,
                 updated_at=excluded.updated_at"
            )?;
            for p in profiles {
                upsert.execute(rusqlite::params![
                    p.id,
                    p.config_type,
                    p.core_type,
                    p.remarks,
                    p.address,
                    p.port,
                    p.user_id,
                    p.security,
                    p.network,
                    p.stream_settings,
                    p.protocol_settings,
                    p.is_sub,
                    p.sub_id,
                    p.group_id,
                    p.sub_uid,
                    p.sort_order,
                    p.is_active,
                    p.created_at,
                    p.updated_at,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn move_orphans_to_graveyard(
        &self,
        group_id: &str,
        active_sub_uids: &[u64],
        graveyard_id: &str,
    ) -> Result<usize> {
        if active_sub_uids.is_empty() {
            return Ok(self.conn.execute(
                "UPDATE profiles SET group_id = ?1, updated_at = datetime('now') WHERE group_id = ?2 AND is_sub = 1 AND sub_uid != 0",
                rusqlite::params![graveyard_id, group_id],
            )?);
        }
        let profiles_in_group: Vec<Profile> = {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM profiles WHERE group_id = ?1 AND is_sub = 1 AND sub_uid != 0",
            )?;
            let rows = stmt.query_map([group_id], |row| Profile::try_from(row))?;
            let mut v = Vec::new();
            for row in rows {
                v.push(row?);
            }
            v
        };
        let mut moved = 0;
        for p in &profiles_in_group {
            if !active_sub_uids.contains(&(p.sub_uid.unwrap_or(0) as u64)) {
                self.conn.execute(
                    "UPDATE profiles SET group_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![graveyard_id, p.id],
                )?;
                moved += 1;
            }
        }
        Ok(moved)
    }

    pub fn purge_graveyard(&self, graveyard_id: &str, ttl_hours: i64) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM profiles WHERE group_id = ?1 AND updated_at < datetime('now', ?2)",
            rusqlite::params![graveyard_id, format!("-{} hours", ttl_hours)],
        )?;
        Ok(count)
    }

    // ── Routing rules ──────────────────────────────────────────────────

    pub fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM routing_rules ORDER BY sort_order")?;
        let rows = stmt.query_map([], |row| RoutingRule::try_from(row))?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    pub fn insert_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        self.conn.execute(
            "INSERT INTO routing_rules (id, group_id, type, domain_matcher, domains, ips, inbound_tags, port, source_ports, network, protocols, domain_strategy, outbound_tag, balancer_tag, rule_set_file, rule_set_url, sort_order) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
            rusqlite::params![
                r.id, r.group_id, r.r#type, r.domain_matcher, r.domains, r.ips,
                r.inbound_tags, r.port, r.source_ports, r.network, r.protocols,
                r.domain_strategy, r.outbound_tag, r.balancer_tag, r.rule_set_file,
                r.rule_set_url, r.sort_order,
            ],
        )?;
        Ok(())
    }

    pub fn update_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        self.conn.execute(
            "UPDATE routing_rules SET group_id=?1, type=?2, domain_matcher=?3, domains=?4, ips=?5, inbound_tags=?6, port=?7, source_ports=?8, network=?9, protocols=?10, domain_strategy=?11, outbound_tag=?12, balancer_tag=?13, rule_set_file=?14, rule_set_url=?15, sort_order=?16 WHERE id=?17",
            rusqlite::params![
                r.group_id, r.r#type, r.domain_matcher, r.domains, r.ips,
                r.inbound_tags, r.port, r.source_ports, r.network, r.protocols,
                r.domain_strategy, r.outbound_tag, r.balancer_tag, r.rule_set_file,
                r.rule_set_url, r.sort_order, r.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_routing_rule(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM routing_rules WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt =
                tx.prepare("UPDATE routing_rules SET sort_order = ?1 WHERE id = ?2")?;
            for (id, order) in ids {
                stmt.execute(rusqlite::params![order, id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    // ── DNS settings ───────────────────────────────────────────────────

    pub fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM dns_settings LIMIT 1")?;
        let mut rows = stmt.query_map([], |row| DnsSetting::try_from(row))?;
        match rows.next() {
            Some(Ok(dns)) => Ok(Some(dns)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn upsert_dns_settings(&self, dns: &DnsSetting) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO dns_settings (id, name, servers, hosts, query_strategy, disable_cache, disable_fallback, client_ip) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            rusqlite::params![dns.id, dns.name, dns.servers, dns.hosts, dns.query_strategy, dns.disable_cache, dns.disable_fallback, dns.client_ip],
        )?;
        Ok(())
    }
}
