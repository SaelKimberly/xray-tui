use std::collections::HashSet;
use std::path::Path;
use toasty::stmt::IntoStatement;

use toasty_core::schema::db::Type as DbType;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::models_toasty::{
    Connection, DnsSetting, Group, PingResultUpdate, PingSession, Profile, ProfileExtension,
    RoutingRule, ServerStat, Subscription,
};

// ── Database handle ─────────────────────────────────────────────────────

pub struct Database {
    db: toasty::Db,
}

// ── Constructors ────────────────────────────────────────────────────────

impl Database {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;
        let driver = toasty_driver_turso::Turso::file(path_str);
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Profile,
                Group,
                Connection,
                ProfileExtension,
                ServerStat,
                Subscription,
                RoutingRule,
                DnsSetting,
                PingSession
            ))
            .build(driver)
            .await?;

        let mut conn = db.connection().await?;

        // Schema versioning: version 2 is the uid-based schema.
        // Old schema (v1) gets dropped entirely — pre-alpha, no migration compat.
        const SCHEMA_VERSION: i32 = 2;

        let version: i32 = {
            let rows = toasty::sql::query("PRAGMA user_version")
                .exec(&mut conn)
                .await?;
            rows.first()
                .and_then(|v| {
                    if let Value::Record(fields) = v {
                        fields.first().and_then(|f| match f {
                            Value::I64(n) => Some(*n as i32),
                            _ => None,
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        };

        if version < SCHEMA_VERSION {
            // Delete old DB and recreate fresh
            drop(conn);
            drop(db);
            let _ = std::fs::remove_file(path_str);
            let driver = toasty_driver_turso::Turso::file(path_str);
            let db = toasty::Db::builder()
                .models(toasty::models!(
                    Profile,
                    Group,
                    Connection,
                    ProfileExtension,
                    ServerStat,
                    Subscription,
                    RoutingRule,
                    DnsSetting,
                    PingSession
                ))
                .build(driver)
                .await?;
            let mut conn = db.connection().await?;
            db.push_schema().await?;
            toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                .exec(&mut conn)
                .await?;
            // Default group for fresh DB
            Self::init_default_group(&mut conn).await?;
            toasty::sql::query("PRAGMA journal_mode=WAL")
                .exec(&mut conn)
                .await?;
            toasty::sql::query("PRAGMA busy_timeout=5000")
                .exec(&mut conn)
                .await?;
            toasty::sql::query("PRAGMA foreign_keys=ON")
                .exec(&mut conn)
                .await?;
            return Ok(Self { db });
        }

        toasty::sql::query("PRAGMA journal_mode=WAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_group(&mut conn).await?;
        Ok(Self { db })
    }

    async fn init_default_group(conn: &mut impl toasty::Executor) -> Result<()> {
        let group_count = Group::all().count().exec(conn).await?;
        if group_count == 0 {
            let default_id = uuid::Uuid::new_v4().to_string();
            Group::create()
                .id(default_id)
                .name(Some("Default".to_string()))
                .sort_order(Some(0))
                .subscription_enabled(Some(0))
                .into_statement()
                .exec(conn)
                .await?;
        }
        Ok(())
    }

    pub async fn in_memory() -> Result<Self> {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Profile,
                Group,
                Connection,
                ProfileExtension,
                ServerStat,
                Subscription,
                RoutingRule,
                DnsSetting,
                PingSession
            ))
            .build(driver)
            .await?;

        let mut conn = db.connection().await?;
        db.push_schema().await?;

        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;
        Self::init_default_group(&mut conn).await?;
        Ok(Self { db })
    }
}

// ── Read queries (public API) ───────────────────────────────────────────

impl Database {
    /// All profiles that have at least one connection (DISTINCT).
    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT DISTINCT p.id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                    p.config_type, p.core_type, p.address, p.port, p.transport, p.security, p.created_at \
             FROM profiles p \
             INNER JOIN connections c ON c.profile_id = p.id \
             ORDER BY p.id",
        )
        .exec(&mut conn)
        .await?;
        deserialize_profiles(rows)
    }

    /// Profiles in a specific group, with their connection metadata.
    pub async fn get_profiles_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(Profile, Connection)>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT p.id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                    p.config_type, p.core_type, p.address, p.port, p.transport, p.security, p.created_at, \
                    c.id, c.profile_id, c.group_id, c.remarks, c.seen_at, c.is_sub, c.sort_order, c.is_active, c.updated_at \
             FROM profiles p \
             INNER JOIN connections c ON c.profile_id = p.id \
             WHERE c.group_id = ?1 \
             ORDER BY c.sort_order ASC, c.id ASC",
        )
        .bind(group_id)
        .exec(&mut conn)
        .await?;
        deserialize_profile_connections(rows)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    pub async fn get_profile_extension(&self, profile_id: i64) -> Result<Option<ProfileExtension>> {
        let mut conn = self.db.connection().await?;
        let ext = ProfileExtension::filter_by_profile_id(profile_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(ext)
    }

    pub async fn get_server_stats(&self, profile_id: i64) -> Result<Option<ServerStat>> {
        let mut conn = self.db.connection().await?;
        let stats = ServerStat::filter_by_profile_id(profile_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(stats)
    }

    pub async fn get_all_profiles_with_details(&self) -> Result<Vec<super::ProfileWithDetails>> {
        let mut conn = self.db.connection().await?;
        let profiles: Vec<Profile> = Profile::all()
            .include(Profile::fields().extension())
            .include(Profile::fields().server_stat())
            .exec(&mut conn)
            .await?;

        let mut result = Vec::with_capacity(profiles.len());
        for p in profiles {
            let ext = p.extension.get().clone();
            let stat = p.server_stat.get().clone();
            result.push((p, ext, stat));
        }
        Ok(result)
    }
    /// All profiles with their extension, stats, and connection group_id.
    /// Uses LEFT JOIN so profiles with 0 connections are still returned
    /// (group_id will be None for orphans).
    pub async fn get_all_profiles_with_connections(
        &self,
    ) -> Result<Vec<(Profile, Option<ProfileExtension>, Option<ServerStat>, Option<String>)>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT p.id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                    p.config_type, p.core_type, p.address, p.port, p.transport, p.security, p.created_at, \
                    c.group_id \
             FROM profiles p \
             LEFT JOIN connections c ON c.profile_id = p.id",
        )
        .exec(&mut conn)
        .await?;

        let mut result = Vec::with_capacity(rows.len());
        for value in rows {
            if let Value::Record(fields) = value {
                // Indices 0-11: profile fields
                let profile = Profile {
                    id: get_i64(&fields, 0)?,
                    sig: get_i64(&fields, 1)?,
                    cred_hash: get_i64(&fields, 2)?,
                    proto_kind: get_string(&fields, 3)?,
                    spec_blob: get_blob(&fields, 4),
                    config_type: get_i64(&fields, 5)? as i32,
                    core_type: get_string(&fields, 6)?,
                    address: get_string(&fields, 7)?,
                    port: get_i64(&fields, 8)? as i32,
                    transport: get_opt_string(&fields, 9),
                    security: get_opt_string(&fields, 10),
                    created_at: get_i64(&fields, 11)?,
                    extension: Default::default(),
                    server_stat: Default::default(),
                };
                // Index 12: group_id (optional, from LEFT JOIN)
                let group_id = get_opt_string(&fields, 12);
                // Load extension and stats via dedicated methods
                let ext = self.get_profile_extension(profile.id).await?;
                let stats = self.get_server_stats(profile.id).await?;
                result.push((profile, ext, stats, group_id));
            }
        }
        Ok(result)
    }

    pub async fn get_profile(&self, id: i64) -> Result<Option<Profile>> {
        let mut conn = self.db.connection().await?;
        let profile = Profile::filter_by_id(id).first().exec(&mut conn).await?;
        Ok(profile)
    }

    pub async fn get_subscription_by_group(&self, group_id: &str) -> Result<Option<Subscription>> {
        let mut conn = self.db.connection().await?;
        Ok(
            Subscription::filter(Subscription::fields().group_id().eq(group_id.to_string()))
                .first()
                .exec(&mut conn)
                .await?,
        )
    }

    pub async fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut conn = self.db.connection().await?;
        let subs: Vec<Subscription> = Subscription::all().exec(&mut conn).await?;
        Ok(subs)
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT g.id, g.name, g.subscription_url, g.subscription_enabled, \
                    g.user_agent, g.convert_target, g.core_type, g.sort_order, g.is_system \
             FROM groups g \
             LEFT JOIN subscriptions s ON s.group_id = g.id \
             WHERE g.subscription_enabled = 1 \
               AND g.subscription_url IS NOT NULL AND g.subscription_url != '' \
               AND (s.last_updated IS NULL OR \
                    datetime(s.last_updated, '+' || COALESCE(s.update_interval, 1440) || ' minutes') < datetime('now'))",
        )
        .exec(&mut conn)
        .await?;

        let mut groups = Vec::with_capacity(rows.len());
        for value in rows {
            if let Value::Record(fields) = value {
                groups.push(Group {
                    id: get_string(&fields, 0)?,
                    name: get_opt_string(&fields, 1),
                    subscription_url: get_opt_string(&fields, 2),
                    subscription_enabled: get_opt_i64(&fields, 3).map(|v| v as i32),
                    user_agent: get_opt_string(&fields, 4),
                    convert_target: get_opt_i64(&fields, 5).map(|v| v as i32),
                    core_type: get_opt_string(&fields, 6),
                    sort_order: get_opt_i64(&fields, 7).map(|v| v as i32),
                    is_system: get_opt_i64(&fields, 8).map(|v| v as i32),
                });
            }
        }
        Ok(groups)
    }

    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut conn = self.db.connection().await?;
        let rules: Vec<RoutingRule> = RoutingRule::all()
            .order_by(RoutingRule::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(rules)
    }

    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut conn = self.db.connection().await?;
        let settings: Vec<DnsSetting> = DnsSetting::all().exec(&mut conn).await?;
        Ok(settings.into_iter().next())
    }

    // ── New connection query helpers ───────────────────────────────────

    /// All connections for a given profile.
    pub async fn get_connections_for_profile(&self, profile_id: i64) -> Result<Vec<Connection>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT id, profile_id, group_id, remarks, seen_at, is_sub, sort_order, is_active, updated_at \
             FROM connections WHERE profile_id = ?1 ORDER BY sort_order ASC",
        )
        .bind(profile_id)
        .exec(&mut conn)
        .await?;
        deserialize_connections(rows)
    }

    /// All connections for a given group.
    pub async fn get_connections_for_group(&self, group_id: &str) -> Result<Vec<Connection>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT id, profile_id, group_id, remarks, seen_at, is_sub, sort_order, is_active, updated_at \
             FROM connections WHERE group_id = ?1 ORDER BY sort_order ASC",
        )
        .bind(group_id)
        .exec(&mut conn)
        .await?;
        deserialize_connections(rows)
    }

    /// The active profile in a group (is_active = 1), if any.
    pub async fn get_active_profile_for_group(&self, group_id: &str) -> Result<Option<Profile>> {
        let mut conn = self.db.connection().await?;
        let row = toasty::sql::query(
            "SELECT p.id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                    p.config_type, p.core_type, p.address, p.port, p.transport, p.security, p.created_at \
             FROM profiles p \
             INNER JOIN connections c ON c.profile_id = p.id \
             WHERE c.group_id = ?1 AND c.is_active = 1 \
             LIMIT 1",
        )
        .bind(group_id)
        .exec(&mut conn)
        .await?;
        let mut profiles = deserialize_profiles(row)?;
        Ok(profiles.pop())
    }
}

// ── Write methods (public API) ──────────────────────────────────────────

impl Database {
    /// Upsert a profile and its connection to a group.
    /// Uses ON CONFLICT for atomic dedup.
    pub async fn insert_profile(&self, p: &Profile, group_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let conn_id = uuid::Uuid::new_v4().to_string();

        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Upsert profile
        toasty::sql::statement(
            "INSERT OR REPLACE INTO profiles (id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, address, port, transport, security, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(p.id)
        .bind(p.sig)
        .bind(p.cred_hash)
        .bind(&p.proto_kind)
        .bind_typed(p.spec_blob.clone(), DbType::Blob)
        .bind(p.config_type)
        .bind(&p.core_type)
        .bind(&p.address)
        .bind(p.port)
        .bind(p.transport.as_deref())
        .bind(p.security.as_deref())
        .bind(p.created_at)
        .exec(&mut tx)
        .await?;

        // Upsert connection
        toasty::sql::statement(
            "INSERT OR REPLACE INTO connections (id, profile_id, group_id, is_active, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(&conn_id)
        .bind(p.id)
        .bind(group_id)
        .bind(1i32)
        .bind(now)
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
    pub async fn update_profile(&self, p: &Profile) -> Result<()> {
        let mut conn = self.db.connection().await?;
        Profile::filter_by_id(p.id)
            .update()
            .sig(p.sig)
            .cred_hash(p.cred_hash)
            .proto_kind(p.proto_kind.clone())
            .spec_blob(p.spec_blob.clone())
            .config_type(p.config_type)
            .core_type(p.core_type.clone())
            .address(p.address.clone())
            .port(p.port)
            .transport(p.transport.clone())
            .security(p.security.clone())
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Delete a profile and all related rows (cascade manually).
    pub async fn delete_profile(&self, id: i64) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Delete connections
        toasty::sql::statement("DELETE FROM connections WHERE profile_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;
        // Related extensions, stats, ping sessions
        toasty::sql::statement("DELETE FROM profile_extensions WHERE profile_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;
        toasty::sql::statement("DELETE FROM server_stats WHERE profile_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;
        toasty::sql::statement("DELETE FROM ping_sessions WHERE profile_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;
        toasty::sql::statement("DELETE FROM profiles WHERE id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Reorder connections in a group.
    pub async fn reorder_profiles(&self, group_id: &str, ids: &[(i64, i32)]) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        for (profile_id, order) in ids {
            toasty::sql::statement(
                "UPDATE connections SET sort_order = ?1 WHERE profile_id = ?2 AND group_id = ?3",
            )
            .bind(*order)
            .bind(*profile_id)
            .bind(group_id)
            .exec(&mut tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "INSERT INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(profile_id) DO UPDATE SET \
             delay=excluded.delay, speed=excluded.speed, \
             sort_order=excluded.sort_order, ip_info=excluded.ip_info",
        )
        .bind(ext.profile_id)
        .bind(ext.delay)
        .bind(ext.speed)
        .bind(ext.sort_order)
        .bind(ext.ip_info.as_deref())
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "INSERT INTO server_stats (profile_id, today_up, today_down, total_up, total_down, last_updated) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(profile_id) DO UPDATE SET \
             today_up=excluded.today_up, today_down=excluded.today_down, \
             total_up=excluded.total_up, total_down=excluded.total_down, \
             last_updated=excluded.last_updated",
        )
        .bind(stats.profile_id)
        .bind(stats.today_up)
        .bind(stats.today_down)
        .bind(stats.total_up)
        .bind(stats.total_down)
        .bind(stats.last_updated.as_deref())
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn insert_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.db.connection().await?;
        Group::create()
            .id(g.id.clone())
            .name(g.name.clone())
            .subscription_url(g.subscription_url.clone())
            .subscription_enabled(g.subscription_enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type.clone())
            .sort_order(g.sort_order)
            .is_system(g.is_system)
            .into_statement()
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.db.connection().await?;
        Group::filter_by_id(g.id.clone())
            .update()
            .name(g.name.clone())
            .subscription_url(g.subscription_url.clone())
            .subscription_enabled(g.subscription_enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type.clone())
            .sort_order(g.sort_order)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Update which profile is active in a group.
    pub async fn update_profile_active(&self, profile_id: i64, group_id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Deactivate all connections in this group
        toasty::sql::statement("UPDATE connections SET is_active = 0 WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut tx)
            .await?;
        // Activate target
        toasty::sql::statement(
            "UPDATE connections SET is_active = 1 WHERE profile_id = ?1 AND group_id = ?2",
        )
        .bind(profile_id)
        .bind(group_id)
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

// ── Group management ────────────────────────────────────────────────────

impl Database {
    pub async fn delete_group(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;

        // Prevent deletion of system groups
        if let Ok(Some(g)) = Group::filter_by_id(id.to_string())
            .first()
            .exec(&mut conn)
            .await
        {
            if g.is_system == Some(1) {
                return Err(DatabaseError::Generic(
                    "cannot delete system group".to_string(),
                ));
            }
        }

        let mut tx = conn.transaction().await?;

        // Delete connections for this group and clean up orphaned profiles
        // Delete connections for this group
        toasty::sql::statement("DELETE FROM connections WHERE group_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        // Delete related extension/stat/ping data for profiles that now have no connections
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;

        // Delete orphaned profiles
        toasty::sql::statement(
            "DELETE FROM profiles WHERE id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;

        toasty::sql::statement("DELETE FROM groups WHERE id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Remove all connections for a group, then purge orphaned profiles.
    pub async fn clear_group(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.db.connection().await?;

        // Prevent clearing system groups
        if let Ok(Some(g)) = Group::filter_by_id(group_id.to_string())
            .first()
            .exec(&mut conn)
            .await
        {
            if g.is_system == Some(1) {
                return Err(DatabaseError::Generic(
                    "cannot clear system group".to_string(),
                ));
            }
        }

        let mut tx = conn.transaction().await?;

        // Count connections first
        let count_row = toasty::sql::query("SELECT COUNT(*) FROM connections WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut tx)
            .await?;
        let count = count_row
            .first()
            .and_then(|v| {
                if let Value::Record(fields) = v {
                    fields.first().and_then(|f| match f {
                        Value::I64(n) => Some(*n as usize),
                        _ => None,
                    })
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Delete connections
        toasty::sql::statement("DELETE FROM connections WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut tx)
            .await?;

        // Purge orphaned profiles (no remaining connections)
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM profiles WHERE id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(count)
    }

    /// Purge profiles with zero connections that were created before a TTL threshold.
    pub async fn purge_orphans(&self, ttl_seconds: i64) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let threshold = now - ttl_seconds;

        // Clean related tables for orphaned profiles
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections) \
             AND profile_id IN (SELECT id FROM profiles WHERE created_at < ?1)",
        )
        .bind(threshold)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections) \
             AND profile_id IN (SELECT id FROM profiles WHERE created_at < ?1)",
        )
        .bind(threshold)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections) \
             AND profile_id IN (SELECT id FROM profiles WHERE created_at < ?1)",
        )
        .bind(threshold)
        .exec(&mut tx)
        .await?;

        let count = toasty::sql::statement(
            "DELETE FROM profiles WHERE id NOT IN (SELECT DISTINCT profile_id FROM connections) \
             AND created_at < ?1",
        )
        .bind(threshold)
        .exec(&mut tx)
        .await? as usize;

        tx.commit().await?;
        Ok(count)
    }

    pub async fn upsert_subscription(&self, sub: &Subscription) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "INSERT INTO subscriptions (id, group_id, url, last_updated, update_interval, user_agent, status, error_message) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
             ON CONFLICT(id) DO UPDATE SET \
             group_id=excluded.group_id, url=excluded.url, \
             last_updated=excluded.last_updated, update_interval=excluded.update_interval, \
             user_agent=excluded.user_agent, status=excluded.status, error_message=excluded.error_message",
        )
        .bind(&sub.id)
        .bind(sub.group_id.as_deref())
        .bind(&sub.url)
        .bind_typed(sub.last_updated.clone(), DbType::Text)
        .bind_typed(sub.update_interval, DbType::Integer(4))
        .bind_typed(sub.user_agent.clone(), DbType::Text)
        .bind_typed(sub.status.clone(), DbType::Text)
        .bind_typed(sub.error_message.clone(), DbType::Text)
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn delete_subscriptions_by_group(&self, group_id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("DELETE FROM subscriptions WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Upsert profiles from a subscription. Uses uid-based dedup.
    /// profiles is a Vec<(Profile, Connection)> — the caller has parsed URLs via ProtoSpec.
    pub async fn subscription_upsert_profiles(
        &self,
        group_id: &str,
        profiles: &[(Profile, Connection)],
    ) -> Result<Vec<i64>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        let mut uids = Vec::with_capacity(profiles.len());

        // Phase 1: upsert profiles only (ON CONFLICT DO UPDATE), collect uids
        for (p, _) in profiles {
            uids.push(p.id);
            toasty::sql::statement(
                "INSERT INTO profiles (id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, address, port, transport, security, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) \
                 ON CONFLICT(id) DO UPDATE SET \
                 sig=excluded.sig, cred_hash=excluded.cred_hash, \
                 spec_blob=excluded.spec_blob, config_type=excluded.config_type, \
                 core_type=excluded.core_type, address=excluded.address, port=excluded.port, \
                 transport=excluded.transport, security=excluded.security",
            )
            .bind(p.id)
            .bind(p.sig)
            .bind(p.cred_hash)
            .bind(&p.proto_kind)
            .bind_typed(p.spec_blob.clone(), DbType::Blob)
        .bind(p.config_type)
        .bind(&p.core_type)
        .bind(&p.address)
        .bind(p.port)
        .bind(p.transport.as_deref())
        .bind(p.security.as_deref())
        .bind(p.created_at)
            .exec(&mut tx)
            .await?;
        }

        // Phase 2: delete connections for profiles no longer in the subscription
        if !uids.is_empty() {
            let ph: Vec<String> = uids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect();
            let sql = format!(
                "DELETE FROM connections WHERE group_id = ?1 AND profile_id NOT IN ({})",
                ph.join(", "),
            );
            let mut stmt = toasty::sql::statement(&sql).bind(group_id);
            for uid in &uids {
                stmt = stmt.bind(*uid);
            }
            stmt.exec(&mut tx).await?;
        } else {
            toasty::sql::statement("DELETE FROM connections WHERE group_id = ?1")
                .bind(group_id)
                .exec(&mut tx)
                .await?;
        }

        // Phase 3: upsert connections (DO NOT overwrite is_active — preserve existing)
        for (p, c) in profiles {
            let conn_id = c.id.clone();
            toasty::sql::statement(
                "INSERT INTO connections (id, profile_id, group_id, remarks, seen_at, is_sub, sort_order, is_active, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
                 ON CONFLICT(profile_id, group_id) DO UPDATE SET \
                 remarks=excluded.remarks, seen_at=excluded.seen_at, \
                 is_sub=excluded.is_sub, sort_order=excluded.sort_order, \
                 updated_at=excluded.updated_at",
            )
            .bind(&conn_id)
            .bind(p.id)
            .bind(group_id)
            .bind_typed(c.remarks.clone(), DbType::Text)
            .bind_typed(c.seen_at.clone(), DbType::Text)
            .bind_typed(c.is_sub, DbType::Integer(4))
            .bind_typed(c.sort_order, DbType::Integer(4))
            .bind_typed(c.is_active, DbType::Integer(4))
            .bind(now)
            .exec(&mut tx)
            .await?;
        }

        // Phase 4: purge orphaned profiles (zero connections)
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM profiles WHERE id NOT IN (SELECT DISTINCT profile_id FROM connections)",
        )
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(uids)
    }
}

// ── Routing rules ───────────────────────────────────────────────────────

impl Database {
    pub async fn insert_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        let mut conn = self.db.connection().await?;
        RoutingRule::create()
            .id(r.id.clone())
            .group_id(r.group_id.clone())
            .r#type(r.r#type)
            .domain_matcher(r.domain_matcher.clone())
            .domains(r.domains.clone())
            .ips(r.ips.clone())
            .inbound_tags(r.inbound_tags.clone())
            .port(r.port.clone())
            .source_ports(r.source_ports.clone())
            .network(r.network.clone())
            .protocols(r.protocols.clone())
            .domain_strategy(r.domain_strategy.clone())
            .outbound_tag(r.outbound_tag.clone())
            .balancer_tag(r.balancer_tag.clone())
            .rule_set_file(r.rule_set_file.clone())
            .rule_set_url(r.rule_set_url.clone())
            .sort_order(r.sort_order)
            .into_statement()
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        let mut conn = self.db.connection().await?;
        RoutingRule::filter_by_id(r.id.clone())
            .update()
            .group_id(r.group_id.clone())
            .r#type(r.r#type)
            .domain_matcher(r.domain_matcher.clone())
            .domains(r.domains.clone())
            .ips(r.ips.clone())
            .inbound_tags(r.inbound_tags.clone())
            .port(r.port.clone())
            .source_ports(r.source_ports.clone())
            .network(r.network.clone())
            .protocols(r.protocols.clone())
            .domain_strategy(r.domain_strategy.clone())
            .outbound_tag(r.outbound_tag.clone())
            .balancer_tag(r.balancer_tag.clone())
            .rule_set_file(r.rule_set_file.clone())
            .rule_set_url(r.rule_set_url.clone())
            .sort_order(r.sort_order)
            .exec(&mut conn)
            .await?;
        Ok(())
    }
    pub async fn delete_routing_rule(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("DELETE FROM routing_rules WHERE id = ?1")
            .bind(id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }
    pub async fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        for (id, order) in ids {
            toasty::sql::statement("UPDATE routing_rules SET sort_order = ?1 WHERE id = ?2")
                .bind(*order)
                .bind(id)
                .exec(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

// ── DNS settings ────────────────────────────────────────────────────────

impl Database {
    pub async fn upsert_dns_settings(&self, dns: &DnsSetting) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "INSERT INTO dns_settings (id, name, servers, hosts, query_strategy, disable_cache, disable_fallback, client_ip) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
             ON CONFLICT(id) DO UPDATE SET \
             name=excluded.name, servers=excluded.servers, hosts=excluded.hosts, \
             query_strategy=excluded.query_strategy, disable_cache=excluded.disable_cache, \
             disable_fallback=excluded.disable_fallback, client_ip=excluded.client_ip",
        )
        .bind(&dns.id)
        .bind(dns.name.as_deref())
        .bind(dns.servers.as_deref())
        .bind(dns.hosts.as_deref())
        .bind(dns.query_strategy.as_deref())
        .bind(dns.disable_cache)
        .bind(dns.disable_fallback)
        .bind(dns.client_ip.as_deref())
        .exec(&mut conn)
        .await?;
        Ok(())
    }
}

// ── Ping batch management ───────────────────────────────────────────────

impl Database {
    /// Create a ping batch with Rust-side triplet dedup.
    pub async fn create_ping_batch(&self, batch_id: &str, group_id: Option<&str>) -> Result<usize> {
        let mut conn = self.db.connection().await?;

        let profiles_and_connections: Vec<(Profile, Connection)> = if let Some(gid) = group_id {
            self.get_profiles_by_group(gid).await?
        } else {
            let rows = toasty::sql::query(
                "SELECT p.id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                        p.config_type, p.core_type, p.address, p.port, p.transport, p.security, p.created_at \
                 FROM profiles p \
                 INNER JOIN connections c ON c.profile_id = p.id \
                 WHERE c.is_active = 1",
            )
            .exec(&mut conn)
            .await?;
            let profiles = deserialize_profiles(rows)?;
            profiles
                .into_iter()
                .map(|p| {
                    (
                        p,
                        Connection {
                            id: String::new(),
                            profile_id: 0,
                            group_id: String::new(),
                            remarks: None,
                            seen_at: None,
                            is_sub: None,
                            sort_order: None,
                            is_active: None,
                            updated_at: 0,
                            profile: Default::default(),
                            group: Default::default(),
                        },
                    )
                })
                .collect()
        };

        let mut seen = HashSet::new();
        let mut rank = 0i32;
        let mut out = Vec::with_capacity(profiles_and_connections.len());
        for (p, _) in &profiles_and_connections {
            // Deserialize address/port from spec_blob
            let addr = String::new(); // Placeholder — will need ProtoSpec deserialization
            let port: Option<i32> = None; // Placeholder
            let triplet = (p.config_type, addr.clone(), port);
            if seen.insert(triplet) {
                rank += 1;
            }
            out.push((
                p.id,
                p.config_type,
                p.core_type.clone(),
                addr.clone(),
                port,
                rank,
            ));
        }

        let mut tx = conn.transaction().await?;
        let mut inserted = 0usize;
        for (pid, ct, ct_str, _addr, _port, trank) in &out {
            toasty::sql::statement(
                "INSERT INTO ping_sessions \
                 (id, batch_id, profile_id, config_type, core_type, ping_type, status, triplet_rank) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'fast', 'queued', ?6)",
            )
            .bind(&format!("{batch_id}-{pid}"))
            .bind(batch_id)
            .bind(*pid)
            .bind(*ct)
            .bind(ct_str)
            .bind(*trank)
            .exec(&mut tx)
            .await?;
            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn get_ping_batch_page(
        &self,
        batch_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<PingSession>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT * FROM ping_sessions WHERE batch_id = ?1 ORDER BY triplet_rank, id LIMIT ?2 OFFSET ?3",
        )
        .bind(batch_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .exec(&mut conn)
        .await?;

        deserialize_ping_sessions(rows)
    }

    pub async fn batch_update_ping_results(
        &self,
        batch_id: &str,
        results: &[PingResultUpdate],
    ) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        for r in results {
            toasty::sql::statement(
                "UPDATE ping_sessions SET status=?1, latency_ms=?2, speed_bps=?3, ip_info=?4, error=?5, updated_at=datetime('now') \
                 WHERE id=?6 AND batch_id=?7",
            )
            .bind(&r.status)
            .bind(r.latency_ms)
            .bind(r.speed_bps)
            .bind(r.ip_info.as_deref())
            .bind(r.error.as_deref())
            .bind(&r.session_id)
            .bind(batch_id)
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn cancel_ping_batch(&self, batch_id: &str) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let updated = toasty::sql::statement(
            "UPDATE ping_sessions SET status='cancelled', updated_at=datetime('now') \
             WHERE batch_id=?1 AND status='queued'",
        )
        .bind(batch_id)
        .exec(&mut conn)
        .await?;
        Ok(updated as usize)
    }

    pub async fn cleanup_ping_batch(&self, batch_id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("DELETE FROM ping_sessions WHERE batch_id = ?1")
            .bind(batch_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_session_status(&self, session_id: &str, status: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "UPDATE ping_sessions SET status=?1, updated_at=datetime('now') WHERE id=?2",
        )
        .bind(status)
        .bind(session_id)
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn update_session_ping_type(
        &self,
        session_id: &str,
        ping_type: &str,
        new_status: &str,
    ) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "UPDATE ping_sessions SET ping_type=?1, status=?2, updated_at=datetime('now') WHERE id=?3",
        )
        .bind(ping_type)
        .bind(new_status)
        .bind(session_id)
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn get_batch_page_ready_for_fast_ping(
        &self,
        batch_id: &str,
        limit: usize,
    ) -> Result<Vec<PingSession>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT * FROM ping_sessions \
             WHERE batch_id = ?1 AND status = 'queued' AND ping_type = 'fast' \
             ORDER BY triplet_rank, id LIMIT ?2",
        )
        .bind(batch_id)
        .bind(limit as i64)
        .exec(&mut conn)
        .await?;

        deserialize_ping_sessions(rows)
    }

    pub async fn get_batch_page_ready_for_real_ping(
        &self,
        batch_id: &str,
        limit: usize,
    ) -> Result<Vec<PingSession>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT * FROM ping_sessions \
             WHERE batch_id = ?1 AND status = 'queued' AND ping_type = 'real' \
             ORDER BY triplet_rank, id LIMIT ?2",
        )
        .bind(batch_id)
        .bind(limit as i64)
        .exec(&mut conn)
        .await?;

        deserialize_ping_sessions(rows)
    }

    pub async fn batch_upsert_profile_extensions(
        &self,
        extensions: &[ProfileExtension],
    ) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        for ext in extensions {
            toasty::sql::statement(
                "INSERT INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) \
                 VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT(profile_id) DO UPDATE SET \
                 delay=excluded.delay, speed=excluded.speed, \
                 sort_order=excluded.sort_order, ip_info=excluded.ip_info",
            )
            .bind(ext.profile_id)
            .bind(ext.delay)
            .bind(ext.speed)
            .bind(ext.sort_order)
            .bind(ext.ip_info.as_deref())
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Flush both ping results and profile extensions in a single transaction.
    pub async fn batch_flush_ping_buffer(
        &self,
        batch_id: &str,
        results: &[PingResultUpdate],
        extensions: &[ProfileExtension],
    ) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        for r in results {
            toasty::sql::statement(
                "UPDATE ping_sessions SET status=?1, latency_ms=?2, speed_bps=?3, ip_info=?4, error=?5, updated_at=datetime('now') \
                 WHERE id=?6 AND batch_id=?7",
            )
            .bind(&r.status)
            .bind(r.latency_ms)
            .bind(r.speed_bps)
            .bind(r.ip_info.as_deref())
            .bind(r.error.as_deref())
            .bind(&r.session_id)
            .bind(batch_id)
            .exec(&mut tx)
            .await?;
        }

        for ext in extensions {
            toasty::sql::statement(
                "INSERT INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) \
                 VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT(profile_id) DO UPDATE SET \
                 delay=excluded.delay, speed=excluded.speed, \
                 sort_order=excluded.sort_order, ip_info=excluded.ip_info",
            )
            .bind(ext.profile_id)
            .bind(ext.delay)
            .bind(ext.speed)
            .bind(ext.sort_order)
            .bind(ext.ip_info.as_deref())
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

// ── Deserialization helpers ────────────────────────────────────────────

/// Deserialize Profile rows from raw query results.
fn deserialize_profiles(rows: Vec<Value>) -> Result<Vec<Profile>> {
    let mut profiles = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            profiles.push(Profile {
                id: get_i64(&fields, 0)?,
                sig: get_i64(&fields, 1)?,
                cred_hash: get_i64(&fields, 2)?,
                proto_kind: get_string(&fields, 3)?,
                spec_blob: get_blob(&fields, 4),
                config_type: get_i64(&fields, 5)? as i32,
                core_type: get_string(&fields, 6)?,
                address: get_string(&fields, 7)?,
                port: get_i64(&fields, 8)? as i32,
                transport: get_opt_string(&fields, 9),
                security: get_opt_string(&fields, 10),
                created_at: get_i64(&fields, 11)?,
                extension: Default::default(),
                server_stat: Default::default(),
            });
        }
    }
    Ok(profiles)
}

/// Deserialize (Profile, Connection) pairs from a JOIN query.
/// Column order: 12 profile cols, 9 connection cols.
fn deserialize_profile_connections(rows: Vec<Value>) -> Result<Vec<(Profile, Connection)>> {
    let mut pairs = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            let p = Profile {
                id: get_i64(&fields, 0)?,
                sig: get_i64(&fields, 1)?,
                cred_hash: get_i64(&fields, 2)?,
                proto_kind: get_string(&fields, 3)?,
                spec_blob: get_blob(&fields, 4),
                config_type: get_i64(&fields, 5)? as i32,
                core_type: get_string(&fields, 6)?,
                address: get_string(&fields, 7)?,
                port: get_i64(&fields, 8)? as i32,
                transport: get_opt_string(&fields, 9),
                security: get_opt_string(&fields, 10),
                created_at: get_i64(&fields, 11)?,
                extension: Default::default(),
                server_stat: Default::default(),
            };
            let c = Connection {
                id: get_string(&fields, 12)?,
                profile_id: get_i64(&fields, 13)?,
                group_id: get_string(&fields, 14)?,
                remarks: get_opt_string(&fields, 15),
                seen_at: get_opt_string(&fields, 16),
                is_sub: get_opt_i64(&fields, 17).map(|v| v as i32),
                sort_order: get_opt_i64(&fields, 18).map(|v| v as i32),
                is_active: get_opt_i64(&fields, 19).map(|v| v as i32),
                updated_at: get_i64(&fields, 20)?,
                profile: Default::default(),
                group: Default::default(),
            };
            pairs.push((p, c));
        }
    }
    Ok(pairs)
}

/// Deserialize Connection rows from raw query results.
fn deserialize_connections(rows: Vec<Value>) -> Result<Vec<Connection>> {
    let mut connections = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            connections.push(Connection {
                id: get_string(&fields, 0)?,
                profile_id: get_i64(&fields, 1)?,
                group_id: get_string(&fields, 2)?,
                remarks: get_opt_string(&fields, 3),
                seen_at: get_opt_string(&fields, 4),
                is_sub: get_opt_i64(&fields, 5).map(|v| v as i32),
                sort_order: get_opt_i64(&fields, 6).map(|v| v as i32),
                is_active: get_opt_i64(&fields, 7).map(|v| v as i32),
                updated_at: get_i64(&fields, 8)?,
                profile: Default::default(),
                group: Default::default(),
            });
        }
    }
    Ok(connections)
}

/// Deserialize ping session rows.
fn deserialize_ping_sessions(rows: Vec<Value>) -> Result<Vec<PingSession>> {
    let mut sessions = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            sessions.push(PingSession {
                id: get_string(&fields, 0)?,
                batch_id: get_string(&fields, 1)?,
                profile_id: get_i64(&fields, 2)?,
                config_type: get_i64(&fields, 3)? as i32,
                core_type: get_string(&fields, 4)?,
                address: get_opt_string(&fields, 5),
                port: get_opt_i64(&fields, 6).map(|v| v as i32),
                triplet_rank: get_i64(&fields, 7)? as i32,
                ping_type: get_string(&fields, 8)?,
                status: get_string(&fields, 9)?,
                latency_ms: get_opt_i64(&fields, 10).map(|v| v as i32),
                speed_bps: get_opt_i64(&fields, 11).map(|v| v as i32),
                ip_info: get_opt_string(&fields, 12),
                error: get_opt_string(&fields, 13),
                created_at: get_opt_string(&fields, 14),
                updated_at: get_opt_string(&fields, 15),
            });
        }
    }
    Ok(sessions)
}

// ── Value extraction helpers ───────────────────────────────────────────

fn get_string(fields: &[Value], idx: usize) -> Result<String> {
    fields
        .get(idx)
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| DatabaseError::Generic(format!("expected string column at index {idx}")))
}

fn get_opt_string(fields: &[Value], idx: usize) -> Option<String> {
    fields.get(idx).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        _ => None,
    })
}

fn get_i64(fields: &[Value], idx: usize) -> Result<i64> {
    fields
        .get(idx)
        .and_then(|v| {
            if let Value::I64(n) = v {
                Some(*n)
            } else {
                None
            }
        })
        .ok_or_else(|| DatabaseError::Generic(format!("expected i64 column at index {idx}")))
}

fn get_opt_i64(fields: &[Value], idx: usize) -> Option<i64> {
    fields.get(idx).and_then(|v| match v {
        Value::I64(n) => Some(*n),
        Value::Null => None,
        _ => None,
    })
}

fn get_blob(fields: &[Value], idx: usize) -> Vec<u8> {
    fields
        .get(idx)
        .and_then(|v| {
            if let Value::Bytes(b) = v {
                Some(b.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}
