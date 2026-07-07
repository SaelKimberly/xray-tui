use std::collections::HashSet;
use std::path::Path;
use toasty::stmt::IntoStatement;

use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::helpers::normalize_remark;
use crate::models_toasty::{
    DnsSetting, Group, PingResultUpdate, PingSession, Profile, ProfileExtension, RoutingRule,
    ServerStat, Subscription, ALL_GROUP_ID, GRAVEYARD_GROUP_ID,
};

// ── Database handle ─────────────────────────────────────────────────────

pub struct Database {
    db: toasty::Db,
}

// ── Helpers (private) ───────────────────────────────────────────────────

/// Check whether toasty's schema has already been applied (profiles table exists).
async fn schema_needed(conn: &mut dyn toasty::Executor) -> Result<bool> {
    let rows = toasty::sql::query(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='profiles'",
    )
    .exec(conn)
    .await?;
    Ok(rows.is_empty())
}

/// Create system groups (All, Graveyard) if they don't exist.
async fn ensure_system_groups(conn: &mut dyn toasty::Executor) -> Result<()> {
    let all_exists = Group::filter_by_id(ALL_GROUP_ID.to_string())
        .first()
        .exec(conn)
        .await?
        .is_some();
    if !all_exists {
        create_system_group(conn, ALL_GROUP_ID, "All", 0).await?;
    }

    let grave_exists = Group::filter_by_id(GRAVEYARD_GROUP_ID.to_string())
        .first()
        .exec(conn)
        .await?
        .is_some();
    if !grave_exists {
        create_system_group(conn, GRAVEYARD_GROUP_ID, "Graveyard", i32::MAX).await?;
    }
    Ok(())
}

async fn create_system_group(
    conn: &mut dyn toasty::Executor,
    id: &str,
    name: &str,
    sort_order: i32,
) -> Result<()> {
    Group::create()
        .id(id.to_string())
        .name(Some(name.to_string()))
        .sort_order(Some(sort_order))
        .is_system(Some(1))
        .subscription_enabled(Some(0))
        // into_statement() is required because the generic .exec(&mut impl Executor)
        // on the create builder doesn't accept &mut dyn Executor.
        .into_statement()
        .exec(conn)
        .await?;
    Ok(())
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

        // Only push schema on fresh DB files — bare CREATE TABLE would fail if tables exist.
        if schema_needed(&mut conn).await? {
            db.push_schema().await?;
        }

        toasty::sql::statement("PRAGMA journal_mode=WAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::statement("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::statement("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        ensure_system_groups(&mut conn).await?;
        Ok(Self { db })
    }

    pub async fn in_memory() -> Result<Self> {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Profile,
                Group,
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
        db.push_schema().await?; // fresh DB — always safe

        toasty::sql::statement("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::statement("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        ensure_system_groups(&mut conn).await?;
        Ok(Self { db })
    }
}

// ── Schema migration helpers ────────────────────────────────────────────

impl Database {
    /// One-time backfill: normalize all existing profile remarks.
    /// Uses `user_version` pragma to run exactly once.
    pub async fn normalize_all_remarks(&self) -> Result<()> {
        let mut conn = self.db.connection().await?;

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

        if version >= 1 {
            return Ok(());
        }

        let profiles: Vec<Profile> = Profile::all().exec(&mut conn).await?;
        let mut count = 0u32;

        let mut tx = conn.transaction().await?;
        for p in &profiles {
            if let Some(r) = &p.remarks {
                let normalized = normalize_remark(r);
                if normalized != *r {
                    toasty::sql::statement("UPDATE profiles SET remarks = ?1 WHERE id = ?2")
                        .bind(&normalized)
                        .bind(&p.id)
                        .exec(&mut tx)
                        .await?;
                    count += 1;
                }
            }
        }
        tx.commit().await?;

        if count > 0 {
            tracing::info!(target: "db", "Normalized {count} profile remarks");
        }

        toasty::sql::statement("PRAGMA user_version = 1")
            .exec(&mut conn)
            .await?;
        Ok(())
    }
}

// ── Query methods (public API) ──────────────────────────────────────────

impl Database {
    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let mut conn = self.db.connection().await?;
        let profiles: Vec<Profile> = Profile::all()
            .order_by(Profile::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(profiles)
    }

    pub async fn get_profiles_by_group(&self, group_id: &str) -> Result<Vec<Profile>> {
        let mut conn = self.db.connection().await?;
        let profiles: Vec<Profile> = Profile::filter(Profile::fields().group_id().eq(group_id))
            .order_by(Profile::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(profiles)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    pub async fn get_profile_extension(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileExtension>> {
        let mut conn = self.db.connection().await?;
        let ext = ProfileExtension::filter_by_profile_id(profile_id.to_string())
            .first()
            .exec(&mut conn)
            .await?;
        Ok(ext)
    }

    pub async fn get_server_stats(&self, profile_id: &str) -> Result<Option<ServerStat>> {
        let mut conn = self.db.connection().await?;
        let stats = ServerStat::filter_by_profile_id(profile_id.to_string())
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
            .order_by(Profile::fields().sort_order().asc())
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

    pub async fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        let mut conn = self.db.connection().await?;
        let profile = Profile::filter_by_id(id.to_string())
            .first()
            .exec(&mut conn)
            .await?;
        Ok(profile)
    }

    pub async fn get_subscription_by_group(
        &self,
        group_id: &str,
    ) -> Result<Option<Subscription>> {
        let mut conn = self.db.connection().await?;
        Ok(Subscription::filter(Subscription::fields().group_id().eq(group_id.to_string()))
            .first()
            .exec(&mut conn)
            .await?)
    }

    pub async fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut conn = self.db.connection().await?;
        let subs: Vec<Subscription> = Subscription::all().exec(&mut conn).await?;
        Ok(subs)
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;

        // Single query with LEFT JOIN: last_updated and update_interval live
        // on the subscriptions table, not groups. COALESCE default 1440 = 24h.
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
                // Column order matches SELECT (9 columns, indices 0-8)
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
}

// ── Write methods (public API) ──────────────────────────────────────────

impl Database {
    pub async fn insert_profile(&self, p: &Profile) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        insert_profile_inner(&mut tx, p).await?;

        // Mirror to All group unless already in All
        if p.group_id != ALL_GROUP_ID {
            let mirror = Profile {
                id: format!("{}-all", p.id),
                group_id: ALL_GROUP_ID.to_string(),
                ..p.clone()
            };
            insert_profile_inner(&mut tx, &mirror).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_profile(&self, p: &Profile) -> Result<()> {
        let mut conn = self.db.connection().await?;

        // Query-based update — avoids toasty's instance-update projection panic.
        Profile::filter_by_id(p.id.clone())
            .update()
            .remarks(p.remarks.clone())
            .address(p.address.clone())
            .port(p.port)
            .user_id(p.user_id.clone())
            .security(p.security.clone())
            .network(p.network.clone())
            .stream_settings(p.stream_settings.clone())
            .protocol_settings(p.protocol_settings.clone())
            .sort_order(p.sort_order)
            .is_active(p.is_active)
            .is_sub(p.is_sub)
            .sub_id(p.sub_id.clone())
            .updated_at(p.updated_at.clone())
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // No cascade — manually delete related rows
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

    pub async fn clone_profile(&self, id: &str, new_id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;

        let original = Profile::filter_by_id(id.to_string())
            .first()
            .exec(&mut conn)
            .await?
            .ok_or_else(|| DatabaseError::Generic(format!("profile not found: {id}")))?;

        let mut tx = conn.transaction().await?;

        let clone = Profile {
            id: new_id.to_string(),
            ..original.clone()
        };
        insert_profile_inner(&mut tx, &clone).await?;

        if let Ok(Some(ext)) = ProfileExtension::filter_by_profile_id(id.to_string())
            .first()
            .exec(&mut tx)
            .await
        {
            ProfileExtension::create()
                .profile_id(new_id.to_string())
                .delay(ext.delay)
                .speed(ext.speed)
                .sort_order(ext.sort_order)
                .ip_info(ext.ip_info.clone())
                .into_statement()
                .exec(&mut tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn reorder_profiles(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        for (id, order) in ids {
            toasty::sql::statement("UPDATE profiles SET sort_order = ?1 WHERE id = ?2")
                .bind(*order)
                .bind(id)
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
        .bind(&ext.profile_id)
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
        .bind(&stats.profile_id)
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

    pub async fn update_profile_active(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Deactivate all profiles
        toasty::sql::statement("UPDATE profiles SET is_active = 0")
            .exec(&mut tx)
            .await?;
        // Activate the target
        toasty::sql::statement("UPDATE profiles SET is_active = 1 WHERE id = ?1")
            .bind(id)
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

        // Delete all profiles in this group + their extensions/stats
        // Bulk delete related rows via subquery (no per-profile loop)
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(id)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(id)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(id)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement("DELETE FROM profiles WHERE group_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;
        toasty::sql::statement("DELETE FROM groups WHERE id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

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

        // Bulk delete related rows via subquery (no per-profile loop)
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(group_id)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(group_id)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id IN (SELECT id FROM profiles WHERE group_id = ?1)",
        )
        .bind(group_id)
        .exec(&mut tx)
        .await?;
        let count = toasty::sql::statement("DELETE FROM profiles WHERE group_id = ?1")
            .bind(group_id)
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
        .bind(sub.last_updated.as_deref())
        .bind(opt_to_value(sub.update_interval, |v| Value::I64(v as i64)))
        .bind(sub.user_agent.as_deref())
        .bind(sub.status.as_deref())
        .bind(sub.error_message.as_deref())
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

    /// Bulk-upsert profiles from a subscription fetch.
    /// Uses raw SQL with ON CONFLICT for atomic dedup.
    pub async fn subscription_upsert_profiles(
        &self,
        group_id: &str,
        profiles: &[Profile],
    ) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Phase 1: upsert into target group
        for p in profiles {
            upsert_profile_row(&mut tx, p, group_id).await?;
        }

        // Phase 2: mirror to "All" group (same sub_uid, different group_id)
        for p in profiles {
            let mut mirror = p.clone();
            mirror.id = format!("{}-all", p.id);
            mirror.group_id = ALL_GROUP_ID.to_string();
            upsert_profile_row(&mut tx, &mirror, ALL_GROUP_ID).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn move_orphans_to_graveyard(
        &self,
        group_id: &str,
        active_sub_uids: &[i64],
        graveyard_id: &str,
    ) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Load only id + sub_uid for orphan detection
        let rows = toasty::sql::query(
            "SELECT id, sub_uid FROM profiles WHERE group_id = ?1 AND is_sub = 1",
        )
        .bind(group_id)
        .exec(&mut tx)
        .await?;

        let active_set: HashSet<i64> = active_sub_uids.iter().copied().collect();
        let mut count = 0usize;

        for value in rows {
            if let Value::Record(fields) = value {
                let sub_uid = get_opt_i64(&fields, 1).unwrap_or(0);
                if !active_set.contains(&sub_uid) {
                    let id = get_string(&fields, 0)?;
                    toasty::sql::statement("UPDATE profiles SET group_id = ?1 WHERE id = ?2")
                        .bind(graveyard_id)
                        .bind(&id)
                        .exec(&mut tx)
                        .await?;
                    count += 1;
                }
            }
        }

        tx.commit().await?;
        Ok(count)
    }

    pub async fn purge_graveyard(&self, graveyard_id: &str, ttl_hours: i64) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Bulk delete via subquery — no per-ID loop
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE profile_id IN \
             (SELECT id FROM profiles WHERE group_id = ?1 \
              AND (updated_at IS NULL OR \
                   datetime(updated_at, '+' || ?2 || ' hours') <= datetime('now')))",
        )
        .bind(graveyard_id)
        .bind(ttl_hours)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE profile_id IN \
             (SELECT id FROM profiles WHERE group_id = ?1 \
              AND (updated_at IS NULL OR \
                   datetime(updated_at, '+' || ?2 || ' hours') <= datetime('now')))",
        )
        .bind(graveyard_id)
        .bind(ttl_hours)
        .exec(&mut tx)
        .await?;
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE profile_id IN \
             (SELECT id FROM profiles WHERE group_id = ?1 \
              AND (updated_at IS NULL OR \
                   datetime(updated_at, '+' || ?2 || ' hours') <= datetime('now')))",
        )
        .bind(graveyard_id)
        .bind(ttl_hours)
        .exec(&mut tx)
        .await?;
        let count = toasty::sql::statement(
            "DELETE FROM profiles WHERE group_id = ?1 \
             AND (updated_at IS NULL OR \
                  datetime(updated_at, '+' || ?2 || ' hours') <= datetime('now'))",
        )
        .bind(graveyard_id)
        .bind(ttl_hours)
        .exec(&mut tx)
        .await? as usize;

        tx.commit().await?;
        Ok(count)
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
    /// Create a ping batch with Rust-side triplet dedup (replaces window function).
    pub async fn create_ping_batch(
        &self,
        batch_id: &str,
        group_id: Option<&str>,
    ) -> Result<usize> {
        let mut conn = self.db.connection().await?;

        let mut query = Profile::all()
            .filter(Profile::fields().address().is_some())
            .filter(Profile::fields().port().gt(0))
            .filter(Profile::fields().port().le(65535));
        if let Some(gid) = group_id {
            query = query.filter(Profile::fields().group_id().eq(gid));
        }
        let profiles: Vec<Profile> = query
            .order_by((
                Profile::fields().config_type().asc(),
                Profile::fields().address().asc(),
                Profile::fields().port().asc(),
            ))
            .exec(&mut conn)
            .await?;

        // Rust-side dedup + dense ranking
        let mut seen = HashSet::new();
        let mut rank = 0i32;
        let mut out = Vec::with_capacity(profiles.len());
        for p in &profiles {
            let triplet = (p.config_type, p.address.clone(), p.port);
            if seen.insert(triplet) {
                rank += 1;
            }
            out.push((
                p.id.clone(),
                p.config_type,
                p.core_type.clone(),
                p.address.clone(),
                p.port,
                rank,
            ));
        }

        let mut tx = conn.transaction().await?;
        let mut inserted = 0usize;
        for (pid, ct, ct_str, addr, port, trank) in &out {
            toasty::sql::statement(
                "INSERT INTO ping_sessions \
                 (id, batch_id, profile_id, config_type, core_type, address, port, triplet_rank, ping_type, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'fast', 'queued')",
            )
            .bind(&format!("{batch_id}-{pid}"))
            .bind(batch_id)
            .bind(pid)
            .bind(*ct)
            .bind(ct_str)
            .bind(addr.as_deref())
            .bind(*port)
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
    ) -> Result<Vec<(PingSession, Profile)>> {
        let mut conn = self.db.connection().await?;
        // Load ping sessions + JOIN their profiles
        let rows = toasty::sql::query(
            "SELECT ps.id, ps.batch_id, ps.profile_id, ps.config_type, ps.core_type, \
                    ps.address, ps.port, ps.triplet_rank, ps.ping_type, ps.status, \
                    ps.latency_ms, ps.speed_bps, ps.ip_info, ps.error, \
                    ps.created_at, ps.updated_at, \
                    p.id, p.sub_uid, p.group_id, p.config_type, p.core_type, \
                    p.address, p.port, p.user_id, p.security, p.network, \
                    p.stream_settings, p.protocol_settings, p.remarks, p.is_sub, \
                    p.sub_id, p.sort_order, p.is_active, p.created_at, p.updated_at, p.version \
             FROM ping_sessions ps \
             JOIN profiles p ON p.id = ps.profile_id \
             WHERE ps.batch_id = ?1 AND ps.status = 'queued' AND ps.ping_type = 'real' \
             ORDER BY ps.triplet_rank, ps.id LIMIT ?2",
        )
        .bind(batch_id)
        .bind(limit as i64)
        .exec(&mut conn)
        .await?;

        deserialize_ping_session_pairs(rows)
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
            .bind(&ext.profile_id)
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
            .bind(&ext.profile_id)
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

// ── Private helpers ─────────────────────────────────────────────────────

/// Insert a single profile row using toasty's generated create (for use inside transactions).
async fn insert_profile_inner(
    executor: &mut dyn toasty::Executor,
    p: &Profile,
) -> Result<()> {
    Profile::create()
        .id(p.id.clone())
        .sub_uid(p.sub_uid)
        .group_id(p.group_id.clone())
        .config_type(p.config_type)
        .core_type(p.core_type.clone())
        .address(p.address.clone())
        .port(p.port)
        .user_id(p.user_id.clone())
        .security(p.security.clone())
        .network(p.network.clone())
        .stream_settings(p.stream_settings.clone())
        .protocol_settings(p.protocol_settings.clone())
        .remarks(p.remarks.clone())
        .is_sub(p.is_sub)
        .sub_id(p.sub_id.clone())
        .sort_order(p.sort_order)
        .is_active(p.is_active)
        .created_at(p.created_at.clone())
        .updated_at(p.updated_at.clone())
        .into_statement()
        .exec(executor)
        .await?;
    Ok(())
}

/// Upsert a profile row via raw SQL with concrete `Value` bindings for nullable cols.
/// Uses `Value::Null` directly (not `Option<T>` → `Into<Value>`) to avoid type-inference
/// failures in the Turso driver for null bindings.
async fn upsert_profile_row(
    executor: &mut dyn toasty::Executor,
    p: &Profile,
    group_id: &str,
) -> Result<()> {
    toasty::sql::statement(
        "INSERT INTO profiles \
         (id, sub_uid, group_id, config_type, core_type, address, port, user_id, \
          security, network, stream_settings, protocol_settings, \
          remarks, is_sub, sub_id, sort_order, is_active, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, datetime('now')) \
         ON CONFLICT(group_id, sub_uid) DO UPDATE SET \
         address=excluded.address, port=excluded.port, \
         user_id=excluded.user_id, security=excluded.security, \
         network=excluded.network, stream_settings=excluded.stream_settings, \
         protocol_settings=excluded.protocol_settings, \
         remarks=excluded.remarks, sub_id=excluded.sub_id, \
         is_sub=excluded.is_sub, \
         sort_order=excluded.sort_order, is_active=excluded.is_active, \
         updated_at=datetime('now')",
    )
    .bind(&p.id)
    .bind(p.sub_uid)
    .bind(group_id)
    .bind(p.config_type)
    .bind(&p.core_type)
    .bind(opt_to_value(p.address.clone(), Value::String))
    .bind(opt_to_value(p.port, |v| Value::I64(v as i64)))
    .bind(opt_to_value(p.user_id.clone(), Value::String))
    .bind(opt_to_value(p.security.clone(), Value::String))
    .bind(opt_to_value(p.network.clone(), Value::String))
    .bind(opt_to_value(p.stream_settings.clone(), Value::String))
    .bind(opt_to_value(p.protocol_settings.clone(), Value::String))
    .bind(opt_to_value(p.remarks.clone(), Value::String))
    .bind(opt_to_value(p.is_sub, |v| Value::I64(v as i64)))
    .bind(opt_to_value(p.sub_id.clone(), Value::String))
    .bind(opt_to_value(p.sort_order, |v| Value::I64(v as i64)))
    .bind(opt_to_value(p.is_active, |v| Value::I64(v as i64)))
    .bind(opt_to_value(p.created_at.clone(), Value::String))
    .exec(executor)
    .await?;
    Ok(())
}
/// Deserialize rows from a raw `SELECT * FROM ping_sessions` query.
fn deserialize_ping_sessions(rows: Vec<Value>) -> Result<Vec<PingSession>> {
    let mut sessions = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            sessions.push(PingSession {
                id: get_string(&fields, 0)?,
                batch_id: get_string(&fields, 1)?,
                profile_id: get_string(&fields, 2)?,
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

/// Deserialize rows from the real-ping JOIN query (ping_sessions JOIN profiles).
/// Column order: ps.* (16 cols) then profiles.* (20 cols).
fn deserialize_ping_session_pairs(
    rows: Vec<Value>,
) -> Result<Vec<(PingSession, Profile)>> {
    let mut pairs = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            let session = PingSession {
                id: get_string(&fields, 0)?,
                batch_id: get_string(&fields, 1)?,
                profile_id: get_string(&fields, 2)?,
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
            };

            // Profile columns start at index 16
            let p = Profile {
                id: get_string(&fields, 16)?,
                sub_uid: get_i64(&fields, 17)?,
                group_id: get_string(&fields, 18)?,
                config_type: get_i64(&fields, 19)? as i32,
                core_type: get_string(&fields, 20)?,
                address: get_opt_string(&fields, 21),
                port: get_opt_i64(&fields, 22).map(|v| v as i32),
                user_id: get_opt_string(&fields, 23),
                security: get_opt_string(&fields, 24),
                network: get_opt_string(&fields, 25),
                stream_settings: get_opt_string(&fields, 26),
                protocol_settings: get_opt_string(&fields, 27),
                remarks: get_opt_string(&fields, 28),
                is_sub: get_opt_i64(&fields, 29).map(|v| v as i32),
                sub_id: get_opt_string(&fields, 30),
                sort_order: get_opt_i64(&fields, 31).map(|v| v as i32),
                is_active: get_opt_i64(&fields, 32).map(|v| v as i32),
                created_at: get_opt_string(&fields, 33),
                updated_at: get_opt_string(&fields, 34),
                version: get_i64(&fields, 35)? as u64,
                group: Default::default(),
                extension: Default::default(),
                server_stat: Default::default(),
            };

            pairs.push((session, p));
        }
    }
    Ok(pairs)
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

/// Wrap an `Option<T>` into `Value`, mapping `None` to `Value::Null`.
/// Avoids type-inference failures in the Turso driver for `Option<T>` bindings.
fn opt_to_value<T, F>(val: Option<T>, to_val: F) -> Value
where
    F: FnOnce(T) -> Value,
{
    val.map(to_val).unwrap_or(Value::Null)
}
