use std::path::Path;
use turso::Builder;

use crate::error::{DatabaseError, Result};
use crate::helpers::normalize_remark;
use crate::inner::{
    delete_profile_inner, delete_routing_rule_inner, delete_subscriptions_by_group_inner,
    get_all_groups_inner, get_all_profiles_inner, get_all_profiles_with_details_inner,
    get_all_routing_rules_inner, get_all_subscriptions_inner, get_dns_settings_inner,
    get_groups_due_update_inner, get_profile_extension_inner, get_profile_inner,
    get_profiles_by_group_inner, get_server_stats_inner, get_subscription_by_group_inner,
    insert_group_inner, insert_profile_inner, insert_routing_rule_inner,
    move_orphans_to_graveyard_inner, purge_graveyard_inner, reorder_profiles_inner,
    reorder_routing_rules_inner, subscription_upsert_profiles_inner, update_group_inner,
    update_profile_active_inner, update_profile_inner, update_routing_rule_inner,
    upsert_dns_settings_inner, upsert_profile_extension_inner, upsert_server_stats_inner,
    upsert_subscription_inner,
};
use crate::models::{
    self, DnsSetting, Group, Profile, ProfileExtension, RoutingRule, ServerStat, Subscription,
};
use crate::schema;

pub struct Database {
    db: turso::Database,
}

impl Database {
    fn conn(&self) -> Result<turso::Connection> {
        let conn = self.db.connect().map_err(DatabaseError::Turso)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(DatabaseError::Turso)?;
        Ok(conn)
    }

    // ── Database ──────────────────────────────────────────────────────────

    pub async fn open(path: &Path) -> Result<Self> {
        let path_str = path
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;
        let pool = Builder::new_local(path_str)
            .build()
            .await
            .map_err(DatabaseError::Turso)?;
        let db_ = Self { db: pool };
        db_.initialize_schema().await?;
        Ok(db_)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let pool = Builder::new_local(":memory:")
            .build()
            .await
            .map_err(DatabaseError::Turso)?;
        let db_ = Self { db: pool };
        db_.initialize_schema().await?;
        Ok(db_)
    }

    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn()?;
        schema::create_tables(&conn).await?;

        // Ensure graveyard group exists
        let mut stmt = conn
            .prepare_cached("SELECT COUNT(*) FROM groups WHERE id = ?1")
            .await?;
        let count: i64 = stmt
            .query_row(turso::params![models::GRAVEYARD_GROUP_ID])
            .await
            .map_or(0, |row| row.get(0).unwrap_or(0));
        if count == 0 {
            conn.execute(
                "INSERT INTO groups (id, name, subscription_enabled, is_system) VALUES (?1, ?2, 0, 1)",
                turso::params![models::GRAVEYARD_GROUP_ID, "sub-graveyard"],
            )
            .await?;
        }

        // Ensure "All" system group exists
        let _ = conn
            .execute(
                "INSERT OR IGNORE INTO groups (id, name, subscription_enabled, is_system) VALUES (?1, ?2, 0, 1)",
                turso::params![models::ALL_GROUP_ID, "All"],
            )
            .await;
        Ok(())
    }

    /// One-time backfill: normalize all existing profile remarks.
    /// Uses `user_version` pragma to run exactly once.
    pub async fn normalize_all_remarks(&self) -> Result<()> {
        let mut conn = self.conn()?;
        // Query user_version via PRAGMA
        let version: i32 = {
            let mut stmt = conn.prepare_cached("PRAGMA user_version").await?;
            stmt.query_row(())
                .await
                .ok()
                .and_then(|row| row.get::<i32>(0).ok())
                .unwrap_or(0)
        };

        if version >= 1 {
            return Ok(());
        }

        let profiles = get_all_profiles_inner(&conn).await?;
        let mut count = 0u32;

        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;

        for p in &profiles {
            if let Some(r) = &p.remarks {
                let normalized = normalize_remark(r);
                if normalized != *r {
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
            tracing::info!(target: "db", "Backfilled {count} profile remarks");
        }

        // Set user_version via PRAGMA
        let _ = conn.pragma_update("user_version", 1).await?;
        Ok(())
    }

    // ── Query methods (public API) ──────────────────────────────────────

    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let conn = self.conn()?;
        get_all_profiles_inner(&conn).await
    }

    pub async fn get_profiles_by_group(&self, group_id: &str) -> Result<Vec<Profile>> {
        let conn = self.conn()?;
        get_profiles_by_group_inner(&conn, group_id).await
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let conn = self.conn()?;
        get_all_groups_inner(&conn).await
    }

    pub async fn get_profile_extension(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileExtension>> {
        let conn = self.conn()?;
        get_profile_extension_inner(&conn, profile_id).await
    }

    pub async fn get_server_stats(&self, profile_id: &str) -> Result<Option<ServerStat>> {
        let conn = self.conn()?;
        get_server_stats_inner(&conn, profile_id).await
    }

    pub async fn get_all_profiles_with_details(&self) -> Result<Vec<super::ProfileWithDetails>> {
        let conn = self.conn()?;
        get_all_profiles_with_details_inner(&conn).await
    }

    pub async fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        let conn = self.conn()?;
        get_profile_inner(&conn, id).await
    }

    pub async fn get_subscription_by_group(&self, group_id: &str) -> Result<Option<Subscription>> {
        let conn = self.conn()?;
        get_subscription_by_group_inner(&conn, group_id).await
    }

    pub async fn get_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let conn = self.conn()?;
        get_all_subscriptions_inner(&conn).await
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let conn = self.conn()?;
        get_groups_due_update_inner(&conn).await
    }

    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let conn = self.conn()?;
        get_all_routing_rules_inner(&conn).await
    }

    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let conn = self.conn()?;
        get_dns_settings_inner(&conn).await
    }

    // ── Write methods (public API) ──────────────────────────────────────

    pub async fn insert_profile(&self, p: &Profile) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        insert_profile_inner(&tx, p).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn update_profile(&self, p: &Profile) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        update_profile_inner(&tx, p).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        delete_profile_inner(&tx, id).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn clone_profile(&self, id: &str, new_id: &str) -> Result<()> {
        let conn = self.conn()?;
        let original = get_profile_inner(&conn, id)
            .await?
            .ok_or_else(|| DatabaseError::Generic(format!("profile not found: {id}")))?;
        let mut clone = original.clone();
        clone.id = new_id.to_string();
        let mut conn2 = self.conn()?;
        let tx = conn2.transaction().await.map_err(DatabaseError::Turso)?;
        insert_profile_inner(&tx, &clone).await?;
        // Also copy profile_extension if exists
        if let Some(ext) = get_profile_extension_inner(&tx, id).await? {
            let mut new_ext = ext.clone();
            new_ext.profile_id = new_id.to_string();
            upsert_profile_extension_inner(&tx, &new_ext).await?;
        }
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn reorder_profiles(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        reorder_profiles_inner(&tx, ids).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        upsert_profile_extension_inner(&tx, ext).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        upsert_server_stats_inner(&tx, stats).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn insert_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        insert_group_inner(&tx, g).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn update_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        update_group_inner(&tx, g).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn update_profile_active(&self, id: &str) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        update_profile_active_inner(&tx, id).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    // ── Group management ────────────────────────────────────────────────

    pub async fn delete_group(&self, id: &str) -> Result<()> {
        let mut conn = self.conn()?;
        // Prevent deletion of system groups — propagate error on query failure
        let is_system: Option<i32> = {
            let mut stmt = conn
                .prepare_cached("SELECT is_system FROM groups WHERE id = ?1")
                .await?;
            stmt.query_row(turso::params![id])
                .await
                .map_or(None, |row| row.get::<i32>(0).ok())
        };
        if is_system == Some(1) {
            return Err(DatabaseError::Generic(
                "cannot delete system group".to_string(),
            ));
        }
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        // Delete all profiles in this group first
        let profiles = get_profiles_by_group_inner(&tx, id).await?;
        for p in &profiles {
            delete_profile_inner(&tx, &p.id).await?;
        }
        tx.execute("DELETE FROM groups WHERE id = ?1", turso::params![id])
            .await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn clear_group(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.conn()?;
        // Prevent clearing system groups — propagate error on query failure
        let is_system: Option<i32> = {
            let mut stmt = conn
                .prepare_cached("SELECT is_system FROM groups WHERE id = ?1")
                .await?;
            stmt.query_row(turso::params![group_id])
                .await
                .map_or(None, |row| row.get::<i32>(0).ok())
        };
        if is_system == Some(1) {
            return Err(DatabaseError::Generic(
                "cannot clear system group".to_string(),
            ));
        }
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        // Delete all profiles in this group (cascade: extensions + stats)
        let profiles = get_profiles_by_group_inner(&tx, group_id).await?;
        for p in &profiles {
            delete_profile_inner(&tx, &p.id).await?;
        }
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(profiles.len())
    }

    pub async fn upsert_subscription(&self, sub: &Subscription) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        upsert_subscription_inner(&tx, sub).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn delete_subscriptions_by_group(&self, group_id: &str) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        delete_subscriptions_by_group_inner(&tx, group_id).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn subscription_upsert_profiles(
        &self,
        group_id: &str,
        profiles: &[Profile],
    ) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        subscription_upsert_profiles_inner(&tx, group_id, profiles).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn move_orphans_to_graveyard(
        &self,
        group_id: &str,
        active_sub_uids: &[u64],
        graveyard_id: &str,
    ) -> Result<usize> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        let result =
            move_orphans_to_graveyard_inner(&tx, group_id, active_sub_uids, graveyard_id).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(result)
    }

    pub async fn purge_graveyard(&self, graveyard_id: &str, ttl_hours: i64) -> Result<usize> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        let result = purge_graveyard_inner(&tx, graveyard_id, ttl_hours).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(result)
    }

    // ── Routing rules ───────────────────────────────────────────────────

    pub async fn insert_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        insert_routing_rule_inner(&tx, r).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn update_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        update_routing_rule_inner(&tx, r).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn delete_routing_rule(&self, id: &str) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        delete_routing_rule_inner(&tx, id).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    pub async fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        reorder_routing_rules_inner(&tx, ids).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }

    // ── DNS settings ────────────────────────────────────────────────────

    pub async fn upsert_dns_settings(&self, dns: &DnsSetting) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction().await.map_err(DatabaseError::Turso)?;
        upsert_dns_settings_inner(&tx, dns).await?;
        tx.commit().await.map_err(DatabaseError::Turso)?;
        Ok(())
    }
}
