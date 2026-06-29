use crate::columns::*;
use crate::error::*;
use crate::models::*;

// ── _inner helpers (read-only) ───────────────────────────────────────────

#[inline]
pub(crate) async fn get_all_profiles_inner(conn: &turso::Connection) -> Result<Vec<Profile>> {
    let mut stmt = conn
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

#[inline]
pub(crate) async fn get_profiles_by_group_inner(
    conn: &turso::Connection,
    group_id: &str,
) -> Result<Vec<Profile>> {
    let mut stmt = conn
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

#[inline]
pub(crate) async fn get_all_groups_inner(conn: &turso::Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM groups ORDER BY sort_order")
        .await?;
    let mut rows = stmt.query(()).await?;
    let mut groups = Vec::new();
    while let Some(row) = rows.next().await? {
        groups.push(Group::from_row(&row)?);
    }
    Ok(groups)
}

#[inline]
pub(crate) async fn get_profile_extension_inner(
    conn: &turso::Connection,
    profile_id: &str,
) -> Result<Option<ProfileExtension>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM profile_extensions WHERE profile_id = ?1")
        .await?;
    let mut rows = stmt.query(turso::params![profile_id]).await?;
    match rows.next().await? {
        Some(row) => Ok(Some(ProfileExtension::from_row(&row)?)),
        None => Ok(None),
    }
}

#[inline]
pub(crate) async fn get_server_stats_inner(
    conn: &turso::Connection,
    profile_id: &str,
) -> Result<Option<ServerStat>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM server_stats WHERE profile_id = ?1")
        .await?;
    let mut rows = stmt.query(turso::params![profile_id]).await?;
    match rows.next().await? {
        Some(row) => Ok(Some(ServerStat::from_row(&row)?)),
        None => Ok(None),
    }
}

#[inline]
pub(crate) async fn get_all_profiles_with_details_inner(
    conn: &turso::Connection,
) -> Result<Vec<ProfileWithDetails>> {
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
    let mut stmt = conn.prepare_cached(query).await?;
    let mut rows = stmt.query(()).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        let profile = Profile::from_row(&row)?;

        let ext_profile_id: Option<String> = row.get(ProfileDetailsCol::ExtProfileId as usize)?;
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
                last_updated: row.get::<Option<String>>(ProfileDetailsCol::LastUpdated as usize)?,
            })
        } else {
            None
        };

        results.push((profile, extension, stats));
    }
    Ok(results)
}

#[inline]
pub(crate) async fn get_profile_inner(
    conn: &turso::Connection,
    id: &str,
) -> Result<Option<Profile>> {
    let mut stmt = conn
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

#[inline]
pub(crate) async fn get_subscription_by_group_inner(
    conn: &turso::Connection,
    group_id: &str,
) -> Result<Option<Subscription>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM subscriptions WHERE group_id = ?1 LIMIT 1")
        .await?;
    let mut rows = stmt.query(turso::params![group_id]).await?;
    match rows.next().await? {
        Some(row) => Ok(Some(Subscription::from_row(&row)?)),
        None => Ok(None),
    }
}

#[inline]
pub(crate) async fn get_all_subscriptions_inner(
    conn: &turso::Connection,
) -> Result<Vec<Subscription>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM subscriptions ORDER BY group_id")
        .await?;
    let mut rows = stmt.query(()).await?;
    let mut subs = Vec::new();
    while let Some(row) = rows.next().await? {
        subs.push(Subscription::from_row(&row)?);
    }
    Ok(subs)
}

#[inline]
pub(crate) async fn get_groups_due_update_inner(conn: &turso::Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
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

#[inline]
pub(crate) async fn get_all_routing_rules_inner(
    conn: &turso::Connection,
) -> Result<Vec<RoutingRule>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM routing_rules ORDER BY sort_order")
        .await?;
    let mut rows = stmt.query(()).await?;
    let mut rules = Vec::new();
    while let Some(row) = rows.next().await? {
        rules.push(RoutingRule::from_row(&row)?);
    }
    Ok(rules)
}

#[inline]
pub(crate) async fn get_dns_settings_inner(conn: &turso::Connection) -> Result<Option<DnsSetting>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM dns_settings LIMIT 1")
        .await?;
    let mut rows = stmt.query(()).await?;
    match rows.next().await? {
        Some(row) => Ok(Some(DnsSetting::from_row(&row)?)),
        None => Ok(None),
    }
}

// ── _inner helpers (write) ─────────────────────────────────────────────

#[inline]
pub(crate) async fn insert_profile_inner(conn: &turso::Connection, p: &Profile) -> Result<()> {
    let sub_uid = p.sub_uid.unwrap_or(0);
    if sub_uid == 0 {
        return Err(DatabaseError::Generic(
            "Cannot insert profile with sub_uid=0".into(),
        ));
    }

    // 1. Insert or ignore core data
    conn.execute(
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
    let group_id = p.group_id.as_deref().unwrap_or(GRAVEYARD_GROUP_ID);
    conn.execute(
        "INSERT OR REPLACE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        turso::params![
            p.id.as_str(), sub_uid, group_id, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(),
            p.sort_order, p.is_active, p.updated_at.as_deref(), p.created_at.as_deref()
        ],
    )
    .await?;

    // 3. Mirror to "All" group (same core, different group id)
    if group_id != ALL_GROUP_ID && group_id != GRAVEYARD_GROUP_ID {
        let all_id = format!("{}-all", p.id);
        conn.execute(
            "INSERT OR IGNORE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            turso::params![
                all_id.as_str(), sub_uid, ALL_GROUP_ID, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(),
                p.sort_order, p.is_active, p.updated_at.as_deref(), p.created_at.as_deref()
            ],
        )
        .await?;
    }
    Ok(())
}

#[inline]
pub(crate) async fn update_profile_inner(conn: &turso::Connection, p: &Profile) -> Result<()> {
    let sub_uid = p.sub_uid.unwrap_or(0);
    if sub_uid == 0 {
        return Err(DatabaseError::Generic(
            "Cannot update profile with sub_uid=0".into(),
        ));
    }
    conn.execute(
        "INSERT OR REPLACE INTO profile_cores (sub_uid, config_type, core_type, address, port, user_id, security, network, stream_settings, protocol_settings, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        turso::params![
            sub_uid, p.config_type, p.core_type.as_str(), p.address.as_deref(), p.port,
            p.user_id.as_deref(), p.security.as_deref(), p.network.as_deref(), p.stream_settings.as_deref(), p.protocol_settings.as_deref(),
            p.created_at.as_deref()
        ],
    )
    .await?;
    let group_id = p.group_id.as_deref().unwrap_or(GRAVEYARD_GROUP_ID);
    conn.execute(
        "UPDATE group_profiles SET sub_uid=?1, group_id=?2, remarks=?3, is_sub=?4, sub_id=?5, sort_order=?6, is_active=?7, updated_at=?8 WHERE id=?9",
        turso::params![
            sub_uid, group_id, p.remarks.as_deref(), p.is_sub, p.sub_id.as_deref(), p.sort_order, p.is_active,
            p.updated_at.as_deref(), p.id.as_str()
        ],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn delete_profile_inner(conn: &turso::Connection, id: &str) -> Result<()> {
    let sub_uid: Option<i64> = {
        let mut stmt = conn
            .prepare_cached("SELECT sub_uid FROM group_profiles WHERE id = ?1")
            .await?;
        stmt.query_row(turso::params![id])
            .await
            .map_or(None, |row| row.get::<i64>(0).ok())
    };
    conn.execute(
        "DELETE FROM profile_extensions WHERE profile_id = ?1",
        turso::params![id],
    )
    .await?;
    conn.execute(
        "DELETE FROM server_stats WHERE profile_id = ?1",
        turso::params![id],
    )
    .await?;
    conn.execute(
        "DELETE FROM group_profiles WHERE id = ?1",
        turso::params![id],
    )
    .await?;
    // Also delete the ALL-group mirror entry if it exists
    let mirror_id = format!("{id}-all");
    conn.execute(
        "DELETE FROM group_profiles WHERE id = ?1",
        turso::params![mirror_id.as_str()],
    )
    .await?;
    if let Some(su) = sub_uid {
        let remaining: i64 = {
            let mut stmt = conn
                .prepare_cached("SELECT COUNT(*) FROM group_profiles WHERE sub_uid = ?1")
                .await?;
            stmt.query_row(turso::params![su])
                .await
                .map_or(0, |row| row.get::<i64>(0).unwrap_or(0))
        };
        if remaining == 0 {
            conn.execute(
                "DELETE FROM profile_cores WHERE sub_uid = ?1",
                turso::params![su],
            )
            .await?;
        }
    }
    Ok(())
}

#[inline]
pub(crate) async fn upsert_profile_extension_inner(
    conn: &turso::Connection,
    ext: &ProfileExtension,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO profile_extensions (profile_id, delay, speed, sort_order, ip_info) VALUES (?1, ?2, ?3, ?4, ?5)",
        turso::params![ext.profile_id.as_str(), ext.delay, ext.speed, ext.sort_order, ext.ip_info.as_deref()],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn upsert_server_stats_inner(
    conn: &turso::Connection,
    stats: &ServerStat,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO server_stats (profile_id, today_up, today_down, total_up, total_down, last_updated) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        turso::params![
            stats.profile_id.as_str(), stats.today_up, stats.today_down, stats.total_up,
            stats.total_down, stats.last_updated.as_deref()
        ],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn insert_group_inner(conn: &turso::Connection, g: &Group) -> Result<()> {
    conn.execute(
        "INSERT INTO groups (id, name, subscription_url, subscription_enabled, user_agent, convert_target, core_type, sort_order, is_system) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        turso::params![
            g.id.as_str(), g.name.as_deref(), g.subscription_url.as_deref(), g.subscription_enabled, g.user_agent.as_deref(),
            g.convert_target, g.core_type.as_deref(), g.sort_order, g.is_system
        ],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn update_group_inner(conn: &turso::Connection, g: &Group) -> Result<()> {
    conn.execute(
        "UPDATE groups SET name=?1, subscription_url=?2, subscription_enabled=?3, user_agent=?4, convert_target=?5, core_type=?6, sort_order=?7, is_system=?8 WHERE id=?9",
        turso::params![
            g.name.as_deref(), g.subscription_url.as_deref(), g.subscription_enabled, g.user_agent.as_deref(),
            g.convert_target, g.core_type.as_deref(), g.sort_order, g.is_system, g.id.as_str()
        ],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn update_profile_active_inner(conn: &turso::Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE group_profiles SET is_active = 0 WHERE is_active = 1",
        (),
    )
    .await?;
    conn.execute(
        "UPDATE group_profiles SET is_active = 1 WHERE sub_uid = (SELECT COALESCE(sub_uid, id) FROM group_profiles WHERE id = ?1)",
        turso::params![id],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn reorder_profiles_inner(
    conn: &turso::Connection,
    ids: &[(String, i32)],
) -> Result<()> {
    let mut stmt = conn
        .prepare("UPDATE group_profiles SET sort_order = ?1 WHERE id = ?2")
        .await?;
    for (id, order) in ids {
        stmt.execute(turso::params![order, id.as_str()]).await?;
    }
    Ok(())
}

#[inline]
pub(crate) async fn upsert_subscription_inner(
    conn: &turso::Connection,
    sub: &Subscription,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO subscriptions (id, group_id, url, last_updated, update_interval, user_agent, status, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        turso::params![
            sub.id.as_str(), sub.group_id.as_deref(), sub.url.as_str(), sub.last_updated.as_deref(), sub.update_interval,
            sub.user_agent.as_deref(), sub.status.as_deref(), sub.error_message.as_deref()
        ],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn delete_subscriptions_by_group_inner(
    conn: &turso::Connection,
    group_id: &str,
) -> Result<()> {
    conn.execute(
        "DELETE FROM subscriptions WHERE group_id = ?1",
        turso::params![group_id],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn subscription_upsert_profiles_inner(
    conn: &turso::Connection,
    group_id: &str,
    profiles: &[Profile],
) -> Result<()> {
    // 1. Upsert cores
    {
        let mut stmt = conn
            .prepare(
                "INSERT OR REPLACE INTO profile_cores (sub_uid, config_type, core_type, address, port, user_id, security, network, stream_settings, protocol_settings, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .await?;
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
            .await?;
        }
    }

    // 2. Upsert group profiles (target group) with dedup by sub_uid
    {
        let mut stmt = conn
            .prepare(
                "INSERT INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(group_id, sub_uid) DO UPDATE SET
                 remarks=excluded.remarks, is_sub=excluded.is_sub, sub_id=excluded.sub_id,
                 sort_order=excluded.sort_order, is_active=excluded.is_active, updated_at=excluded.updated_at",
            )
            .await?;
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
            .await?;
        }
    }

    // 3. Upsert All group entries (same cores, different group)
    if group_id != ALL_GROUP_ID {
        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO group_profiles (id, sub_uid, group_id, remarks, is_sub, sub_id, sort_order, is_active, updated_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )
            .await?;
        for p in profiles {
            let su = p.sub_uid.unwrap_or(0);
            if su == 0 {
                continue;
            }
            let all_id = format!("{}-all", p.id);
            stmt.execute(turso::params![
                all_id.as_str(),
                su,
                ALL_GROUP_ID,
                p.remarks.as_deref(),
                p.is_sub,
                p.sub_id.as_deref(),
                p.sort_order,
                p.is_active,
                p.updated_at.as_deref(),
                p.created_at.as_deref(),
            ])
            .await?;
        }
    }

    // 4. Promote graveyard orphans: remove graveyard rows for sub_uids now in this group
    if group_id != ALL_GROUP_ID && group_id != GRAVEYARD_GROUP_ID {
        let _removed = conn
            .execute(
                "DELETE FROM group_profiles
                 WHERE group_id = ?1 AND sub_uid IN (
                     SELECT sub_uid FROM group_profiles WHERE group_id = ?2 AND sub_uid > 0
                 )",
                turso::params![GRAVEYARD_GROUP_ID, group_id],
            )
            .await
            .unwrap_or(0);
    }
    Ok(())
}

#[inline]
pub(crate) async fn move_orphans_to_graveyard_inner(
    conn: &turso::Connection,
    group_id: &str,
    active_sub_uids: &[u64],
    graveyard_id: &str,
) -> Result<usize> {
    if active_sub_uids.is_empty() {
        #[allow(clippy::cast_possible_truncation, reason = "row count fits in usize on all targets")]
        return Ok(conn
            .execute(
                "UPDATE group_profiles SET group_id = ?1, updated_at = datetime('now') WHERE group_id = ?2 AND is_sub = 1",
                turso::params![graveyard_id, group_id],
            )
            .await? as usize);
    }
    let profiles_in_group: Vec<Profile> = {
        let mut stmt = conn
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
        #[allow(
            clippy::cast_sign_loss,
            reason = "sub_uid is always non-negative (bit pattern from compute_sub_uid)"
        )]
        if !active_sub_uids.contains(&(p.sub_uid.unwrap_or(0) as u64)) {
            conn.execute(
                "UPDATE group_profiles SET group_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                turso::params![graveyard_id, p.id.as_str()],
            )
            .await?;
            moved += 1;
        }
    }
    Ok(moved)
}

#[inline]
pub(crate) async fn purge_graveyard_inner(
    conn: &turso::Connection,
    graveyard_id: &str,
    ttl_hours: i64,
) -> Result<usize> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "row count fits in usize on all targets"
    )]
    let count = conn
        .execute(
            "DELETE FROM group_profiles WHERE group_id = ?1 AND updated_at < datetime('now', ?2)",
            turso::params![graveyard_id, format!("-{ttl_hours} hours").as_str()],
        )
        .await? as usize;
    Ok(count)
}

#[inline]
pub(crate) async fn insert_routing_rule_inner(
    conn: &turso::Connection,
    r: &RoutingRule,
) -> Result<()> {
    conn.execute(
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

#[inline]
pub(crate) async fn update_routing_rule_inner(
    conn: &turso::Connection,
    r: &RoutingRule,
) -> Result<()> {
    conn.execute(
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

#[inline]
pub(crate) async fn delete_routing_rule_inner(conn: &turso::Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM routing_rules WHERE id = ?1",
        turso::params![id],
    )
    .await?;
    Ok(())
}

#[inline]
pub(crate) async fn reorder_routing_rules_inner(
    conn: &turso::Connection,
    ids: &[(String, i32)],
) -> Result<()> {
    let mut stmt = conn
        .prepare("UPDATE routing_rules SET sort_order = ?1 WHERE id = ?2")
        .await?;
    for (id, order) in ids {
        stmt.execute(turso::params![order, id.as_str()]).await?;
    }
    Ok(())
}

#[inline]
pub(crate) async fn upsert_dns_settings_inner(
    conn: &turso::Connection,
    dns: &DnsSetting,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dns_settings (id, name, servers, hosts, query_strategy, disable_cache, disable_fallback, client_ip) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        turso::params![
            dns.id.as_str(), dns.name.as_deref(), dns.servers.as_deref(), dns.hosts.as_deref(), dns.query_strategy.as_deref(),
            dns.disable_cache, dns.disable_fallback, dns.client_ip.as_deref()
        ],
    )
    .await?;
    Ok(())
}
