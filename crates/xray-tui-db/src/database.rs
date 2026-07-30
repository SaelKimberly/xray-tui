use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::Path;
use toasty::stmt::IntoStatement;
use toasty_core::schema::db::Type as DbType;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::hash::stable_hash;
use crate::models_toasty::EndpointRow;
use crate::models_toasty::{
    DnsSetting, Endpoint, EndpointGroup, Group, PingResultUpdate, PingSession, ProfileExtension,
    ProtocolRow, RoutingRule, ServerStat,
};

// ── Database handle ─────────────────────────────────────────────────────

pub struct Database {
    db: toasty::Db,
}

// ── Constructors ────────────────────────────────────────────────────────

impl Database {
    /// Always deletes existing DB, creates fresh. Pre-alpha, no migration.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;

        // Delete existing DB file so we always start fresh
        if Path::new(path_str).exists() {
            std::fs::remove_file(path_str)?;
        }

        let driver = toasty_driver_turso::Turso::file(path_str);
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                ProtocolRow,
                EndpointGroup,
                Group,
                ProfileExtension,
                ServerStat,
                PingSession,
                RoutingRule,
                DnsSetting
            ))
            .build(driver)
            .await?;

        let mut conn = db.connection().await?;
        db.push_schema().await?;

        toasty::sql::query("PRAGMA journal_mode=WAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_groups(&mut conn).await?;
        Ok(Self { db })
    }

    pub async fn in_memory() -> Result<Self> {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                ProtocolRow,
                EndpointGroup,
                Group,
                ProfileExtension,
                ServerStat,
                PingSession,
                RoutingRule,
                DnsSetting
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

        Self::init_default_groups(&mut conn).await?;
        Ok(Self { db })
    }

    async fn init_default_groups(conn: &mut impl toasty::Executor) -> Result<()> {
        let count = Group::all().count().exec(conn).await?;
        if count == 0 {
            Group::create()
                .id(uuid::Uuid::new_v4().to_string())
                .name(Some("Default".to_string()))
                .sort_order(Some(0))
                .into_statement()
                .exec(conn)
                .await?;
        }
        Ok(())
    }
}

// ── Read queries (public API) ───────────────────────────────────────────

impl Database {
    /// Active endpoints: `max(last_seen_at)` >= `active_threshold`
    pub async fn get_active_endpoints(&self, active_threshold: i64) -> Result<Vec<EndpointRow>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.remarks, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             LEFT JOIN profile_extensions ext ON ext.protocol_id = p.id \
             LEFT JOIN server_stats s ON s.protocol_id = p.id \
             WHERE ?1 <= (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id)",
        )
        .bind(active_threshold)
        .exec(&mut conn)
        .await?;
        deserialize_endpoint_rows(rows)
    }

    /// Active endpoints filtered by group membership.
    pub async fn get_active_endpoints_by_group(
        &self,
        group_id: &str,
        active_threshold: i64,
    ) -> Result<Vec<EndpointRow>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.remarks, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             INNER JOIN endpoint_groups eg ON eg.endpoint_id = e.id AND eg.group_id = ?2 \
             LEFT JOIN profile_extensions ext ON ext.protocol_id = p.id \
             LEFT JOIN server_stats s ON s.protocol_id = p.id \
             WHERE ?1 <= (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id)",
        )
        .bind(active_threshold)
        .bind(group_id)
        .exec(&mut conn)
        .await?;
        deserialize_endpoint_rows(rows)
    }

    /// Stale endpoints: `max(last_seen_at)` < `active_threshold` AND >= `stale_threshold`
    pub async fn get_stale_endpoints(
        &self,
        active_threshold: i64,
        stale_threshold: i64,
    ) -> Result<Vec<EndpointRow>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.remarks, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             LEFT JOIN profile_extensions ext ON ext.protocol_id = p.id \
             LEFT JOIN server_stats s ON s.protocol_id = p.id \
             WHERE (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id) < ?1 \
               AND ?2 <= (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id)",
        )
        .bind(active_threshold)
        .bind(stale_threshold)
        .exec(&mut conn)
        .await?;
        deserialize_endpoint_rows(rows)
    }

    /// Single endpoint by id with all protocols, extensions, stats.
    pub async fn get_endpoint(&self, id: i64) -> Result<Option<EndpointRow>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.remarks, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             LEFT JOIN profile_extensions ext ON ext.protocol_id = p.id \
             LEFT JOIN server_stats s ON s.protocol_id = p.id \
             WHERE e.id = ?1",
        )
        .bind(id)
        .exec(&mut conn)
        .await?;
        let mut all = deserialize_endpoint_rows(rows)?;
        Ok(all.pop())
    }

    pub async fn get_profile_extension(
        &self,
        protocol_id: i64,
    ) -> Result<Option<ProfileExtension>> {
        let mut conn = self.db.connection().await?;
        let ext = ProfileExtension::filter_by_protocol_id(protocol_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(ext)
    }

    pub async fn get_server_stats(&self, protocol_id: i64) -> Result<Option<ServerStat>> {
        let mut conn = self.db.connection().await?;
        let stats = ServerStat::filter_by_protocol_id(protocol_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(stats)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT g.id, g.name, g.url, g.enabled, g.user_agent, g.convert_target, g.core_type, g.sort_order, \
                    g.last_refreshed, g.status, g.error_message, g.refresh_interval \
             FROM groups g \
             WHERE g.enabled = 1 \
               AND g.url IS NOT NULL AND g.url != '' \
               AND (g.last_refreshed IS NULL OR \
                    datetime(g.last_refreshed, '+' || COALESCE(g.refresh_interval, 1440) || ' minutes') < datetime('now'))",
        )
        .exec(&mut conn)
        .await?;

        let mut groups = Vec::with_capacity(rows.len());
        for value in rows {
            if let Value::Record(fields) = value {
                groups.push(Group {
                    id: get_string(&fields, 0)?,
                    name: get_opt_string(&fields, 1),
                    url: get_opt_string(&fields, 2),
                    enabled: get_opt_i64(&fields, 3).map(|v| v as i32),
                    user_agent: get_opt_string(&fields, 4),
                    convert_target: get_opt_i64(&fields, 5).map(|v| v as i32),
                    core_type: get_opt_string(&fields, 6),
                    sort_order: get_opt_i64(&fields, 7).map(|v| v as i32),
                    last_refreshed: get_opt_string(&fields, 8),
                    status: get_opt_string(&fields, 9),
                    error_message: get_opt_string(&fields, 10),
                    refresh_interval: get_opt_i64(&fields, 11).map(|v| v as i32),
                });
            }
        }
        Ok(groups)
    }

    pub async fn get_stale_count(
        &self,
        active_threshold: i64,
        stale_threshold: i64,
    ) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT COUNT(DISTINCT e.id) \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             WHERE (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id) < ?1 \
               AND ?2 <= (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id)",
        )
        .bind(active_threshold)
        .bind(stale_threshold)
        .exec(&mut conn)
        .await?;

        let count = rows
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
        Ok(count)
    }
}

// ── Write methods (public API) ──────────────────────────────────────────

impl Database {
    /// Bulk upsert: endpoints, protocols, and endpoint-group links in one tx.
    /// Returns endpoint IDs that were upserted.
    pub async fn subscription_upsert(
        &self,
        group_id: &str,
        endpoint_protocols: &[(Endpoint, Vec<ProtocolRow>)],
    ) -> Result<Vec<i64>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        let mut eids = Vec::with_capacity(endpoint_protocols.len());

        for (ep, protos) in endpoint_protocols {
            eids.push(ep.id);

            // INSERT OR IGNORE endpoint
            toasty::sql::statement(
                "INSERT OR IGNORE INTO endpoints \
                 (id, host, host_type, port, port_spec_str, parent_id, last_source, created_at, manual_protocol_override) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .bind(ep.id)
            .bind(&ep.host)
            .bind(&ep.host_type)
            .bind(ep.port)
            .bind(ep.port_spec_str.as_deref().unwrap_or(""))
            .bind(ep.parent_id.unwrap_or(0))
            .bind(ep.last_source.as_deref().unwrap_or(""))
            .bind(ep.created_at)
            .bind(ep.manual_protocol_override.unwrap_or(0))
            .exec(&mut tx)
            .await?;

            // Upsert each protocol
            for p in protos {
                ProtocolRow::upsert_by_id(p.id)
                    .endpoint_id(p.endpoint_id)
                    .sig(p.sig)
                    .cred_hash(p.cred_hash)
                    .proto_kind(&p.proto_kind)
                    .spec_blob(p.spec_blob.clone())
                    .config_type(p.config_type)
                    .core_type(&p.core_type)
                    .transport(p.transport.as_deref().unwrap_or(""))
                    .security(p.security.as_deref().unwrap_or(""))
                    .remarks(p.remarks.as_deref().unwrap_or(""))
                    .created_at(p.created_at)
                    .last_seen_at(p.last_seen_at)
                    .exec(&mut tx)
                    .await?;
            }

            // Upsert endpoint-group link: deterministic id = group_id:ep.id
            let eg_id = format!("{group_id}:{}", ep.id);
            toasty::sql::statement(
                "INSERT INTO endpoint_groups (id, endpoint_id, group_id, last_seen_at, sort_order) \
                 VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT(id) DO UPDATE SET \
                 last_seen_at=excluded.last_seen_at, sort_order=excluded.sort_order",
            )
            .bind(&eg_id)
            .bind(ep.id)
            .bind(group_id)
            .bind(now)
            .bind(0i32)
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(eids)
    }

    /// Insert a single manually-added endpoint + protocol + group link.
    pub async fn insert_manual_endpoint(
        &self,
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        group_id: &str,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        toasty::sql::statement(
            "INSERT INTO endpoints \
             (id, host, host_type, port, port_spec_str, parent_id, last_source, created_at, manual_protocol_override) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(endpoint.id)
        .bind(&endpoint.host)
        .bind(&endpoint.host_type)
        .bind(endpoint.port)
        .bind(endpoint.port_spec_str.as_deref().unwrap_or(""))
        .bind(endpoint.parent_id.unwrap_or(0))
        .bind(endpoint.last_source.as_deref().unwrap_or(""))
        .bind(endpoint.created_at)
        .bind(endpoint.manual_protocol_override.unwrap_or(0))
        .exec(&mut tx)
        .await?;

        toasty::sql::statement(
            "INSERT INTO protocol_rows \
             (id, endpoint_id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, transport, security, remarks, created_at, last_seen_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .bind(protocol.id)
        .bind(protocol.endpoint_id)
        .bind(protocol.sig)
        .bind(protocol.cred_hash)
        .bind(&protocol.proto_kind)
        .bind_typed(protocol.spec_blob.clone(), DbType::Blob)
        .bind(protocol.config_type)
        .bind(&protocol.core_type)
        .bind(protocol.transport.as_deref().unwrap_or(""))
        .bind(protocol.security.as_deref().unwrap_or(""))
        .bind(protocol.remarks.as_deref().unwrap_or(""))
        .bind(protocol.created_at)
        .bind(protocol.last_seen_at)
        .exec(&mut tx)
        .await?;

        toasty::sql::statement(
            "INSERT INTO endpoint_groups (id, endpoint_id, group_id, last_seen_at, sort_order) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(endpoint.id)
        .bind(group_id)
        .bind(now)
        .bind(0i32)
        .exec(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Purge endpoints where every protocol has `last_seen_at` < threshold.
    /// Returns count of deleted endpoints.
    pub async fn purge_expired(&self, expire_threshold: i64) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Delete endpoint_groups for expired endpoints
        toasty::sql::statement(
            "DELETE FROM endpoint_groups WHERE endpoint_id IN ( \
             SELECT e.id FROM endpoints e \
             WHERE (SELECT COALESCE(MAX(p.last_seen_at), 0) FROM protocol_rows p WHERE p.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await?;

        // Delete profile_extensions for protocols of expired endpoints
        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE protocol_id IN ( \
             SELECT p.id FROM protocol_rows p \
             INNER JOIN endpoints e ON e.id = p.endpoint_id \
             WHERE (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await?;

        // Delete server_stats for protocols of expired endpoints
        toasty::sql::statement(
            "DELETE FROM server_stats WHERE protocol_id IN ( \
             SELECT p.id FROM protocol_rows p \
             INNER JOIN endpoints e ON e.id = p.endpoint_id \
             WHERE (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await?;

        // Delete ping_sessions for protocols of expired endpoints
        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE protocol_id IN ( \
             SELECT p.id FROM protocol_rows p \
             INNER JOIN endpoints e ON e.id = p.endpoint_id \
             WHERE (SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2 WHERE p2.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await?;

        // Delete protocol_rows for expired endpoints
        toasty::sql::statement(
            "DELETE FROM protocol_rows WHERE endpoint_id IN ( \
             SELECT e.id FROM endpoints e \
             WHERE (SELECT COALESCE(MAX(p.last_seen_at), 0) FROM protocol_rows p WHERE p.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await?;

        let count = toasty::sql::statement(
            "DELETE FROM endpoints WHERE id IN ( \
             SELECT e.id FROM endpoints e \
             WHERE (SELECT COALESCE(MAX(p.last_seen_at), 0) FROM protocol_rows p WHERE p.endpoint_id = e.id) < ?1)",
        )
        .bind(expire_threshold)
        .exec(&mut tx)
        .await? as usize;

        tx.commit().await?;
        Ok(count)
    }

    /// Restore a stale endpoint by setting `last_seen_at` = now on all its protocols.
    pub async fn restore_endpoint(&self, endpoint_id: i64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("UPDATE protocol_rows SET last_seen_at = ?1 WHERE endpoint_id = ?2")
            .bind(now)
            .bind(endpoint_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Delete an endpoint and all related data.
    pub async fn delete_endpoint(&self, endpoint_id: i64) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        toasty::sql::statement("DELETE FROM endpoint_groups WHERE endpoint_id = ?1")
            .bind(endpoint_id)
            .exec(&mut tx)
            .await?;

        toasty::sql::statement(
            "DELETE FROM profile_extensions WHERE protocol_id IN (SELECT id FROM protocol_rows WHERE endpoint_id = ?1)",
        )
        .bind(endpoint_id)
        .exec(&mut tx)
        .await?;

        toasty::sql::statement(
            "DELETE FROM server_stats WHERE protocol_id IN (SELECT id FROM protocol_rows WHERE endpoint_id = ?1)",
        )
        .bind(endpoint_id)
        .exec(&mut tx)
        .await?;

        toasty::sql::statement(
            "DELETE FROM ping_sessions WHERE protocol_id IN (SELECT id FROM protocol_rows WHERE endpoint_id = ?1)",
        )
        .bind(endpoint_id)
        .exec(&mut tx)
        .await?;

        toasty::sql::statement("DELETE FROM protocol_rows WHERE endpoint_id = ?1")
            .bind(endpoint_id)
            .exec(&mut tx)
            .await?;

        toasty::sql::statement("DELETE FROM endpoints WHERE id = ?1")
            .bind(endpoint_id)
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_protocol_override(&self, endpoint_id: i64, protocol_id: i64) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("UPDATE endpoints SET manual_protocol_override = ?1 WHERE id = ?2")
            .bind(protocol_id)
            .bind(endpoint_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn clear_protocol_override(&self, endpoint_id: i64) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement(
            "UPDATE endpoints SET manual_protocol_override = NULL WHERE id = ?1",
        )
        .bind(endpoint_id)
        .exec(&mut conn)
        .await?;
        Ok(())
    }

    /// Upsert resolved IPs as child endpoints of a DNS endpoint.
    pub async fn upsert_resolved_ips(&self, dns_endpoint_id: i64, ips: &[IpAddr]) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let mut conn = self.db.connection().await?;

        for ip in ips {
            let eid = stable_hash(ip.to_string(), 0i64);
            let host_type = match ip {
                IpAddr::V4(_) => "ipv4",
                IpAddr::V6(_) => "ipv6",
            };
            toasty::sql::statement(
                "INSERT OR IGNORE INTO endpoints \
                 (id, host, host_type, port, parent_id, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(eid)
            .bind(ip.to_string())
            .bind(host_type)
            .bind(0i32)
            .bind(dns_endpoint_id)
            .bind(now)
            .exec(&mut conn)
            .await?;
        }
        Ok(())
    }

    /// Resolve a DNS endpoint's hostname, replace its child IPs, return the
    /// resolved addresses.
    pub async fn resolve_endpoint_dns(&self, endpoint_id: i64, host: &str) -> Result<Vec<IpAddr>> {
        use std::time::Duration;
        use tokio::time::timeout;

        // Resolve DNS with 5s timeout
        let lookup = tokio::net::lookup_host((host, 0));
        let addrs = match timeout(Duration::from_secs(5), lookup).await {
            Ok(Ok(addrs)) => addrs,
            Ok(Err(e)) => return Err(DatabaseError::Io(e)),
            Err(_) => {
                return Err(DatabaseError::Generic(format!(
                    "DNS resolution timed out for {host}"
                )));
            }
        };

        let mut ips: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
        ips.sort();
        ips.dedup();
        ips.sort_by_key(|ip| match ip {
            IpAddr::V4(_) => 0u8,
            IpAddr::V6(_) => 1,
        });

        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Remove old child endpoints
        toasty::sql::statement("DELETE FROM endpoints WHERE parent_id = ?1")
            .bind(endpoint_id)
            .exec(&mut tx)
            .await?;

        // Insert new children
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for ip in &ips {
            let eid = stable_hash(ip.to_string(), 0i64);
            let host_type = match ip {
                IpAddr::V4(_) => "ipv4",
                IpAddr::V6(_) => "ipv6",
            };
            toasty::sql::statement(
                "INSERT OR IGNORE INTO endpoints \
                 (id, host, host_type, port, parent_id, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(eid)
            .bind(ip.to_string())
            .bind(host_type)
            .bind(0i32)
            .bind(endpoint_id)
            .bind(now)
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(ips)
    }

    pub async fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        let mut conn = self.db.connection().await?;
        ProfileExtension::upsert_by_protocol_id(ext.protocol_id)
            .delay(ext.delay.unwrap_or(0))
            .speed(ext.speed.unwrap_or(0))
            .sort_order(ext.sort_order.unwrap_or(0))
            .ip_info(ext.ip_info.as_deref().unwrap_or(""))
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn clear_all_stats(&self) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("DELETE FROM server_stats")
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        let mut conn = self.db.connection().await?;
        ServerStat::upsert_by_protocol_id(stats.protocol_id)
            .today_up(stats.today_up.unwrap_or(0))
            .today_down(stats.today_down.unwrap_or(0))
            .total_up(stats.total_up.unwrap_or(0))
            .total_down(stats.total_down.unwrap_or(0))
            .last_updated(stats.last_updated.as_deref().unwrap_or(""))
            .exec(&mut conn)
            .await?;
        Ok(())
    }
}

// ── Group management ────────────────────────────────────────────────────

impl Database {
    pub async fn insert_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.db.connection().await?;
        Group::create()
            .id(g.id.clone())
            .name(g.name.clone())
            .url(g.url.clone())
            .enabled(g.enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type.clone())
            .sort_order(g.sort_order)
            .last_refreshed(g.last_refreshed.clone())
            .status(g.status.clone())
            .error_message(g.error_message.clone())
            .refresh_interval(g.refresh_interval)
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
            .url(g.url.clone())
            .enabled(g.enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type.clone())
            .sort_order(g.sort_order)
            .last_refreshed(g.last_refreshed.clone())
            .status(g.status.clone())
            .error_message(g.error_message.clone())
            .refresh_interval(g.refresh_interval)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Delete a group and its associated data.
    pub async fn delete_group(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;

        // Remove group-endpoint links
        toasty::sql::statement("DELETE FROM endpoint_groups WHERE group_id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        // Delete group
        toasty::sql::statement("DELETE FROM groups WHERE id = ?1")
            .bind(id)
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Remove all endpoints from a group, return count of affected links.
    pub async fn clear_group(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query("SELECT COUNT(*) FROM endpoint_groups WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut conn)
            .await?;

        let count = rows
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

        toasty::sql::statement("DELETE FROM endpoint_groups WHERE group_id = ?1")
            .bind(group_id)
            .exec(&mut conn)
            .await?;

        Ok(count)
    }
}

// ── Routing rules ───────────────────────────────────────────────────────

impl Database {
    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut conn = self.db.connection().await?;
        let rules: Vec<RoutingRule> = RoutingRule::all()
            .order_by(RoutingRule::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(rules)
    }

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
        RoutingRule::filter_by_id(id.to_string())
            .delete()
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        for (rule_id, order) in ids {
            RoutingRule::filter_by_id(rule_id.clone())
                .update()
                .sort_order(Some(*order))
                .exec(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

// ── DNS settings ────────────────────────────────────────────────────────

impl Database {
    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut conn = self.db.connection().await?;
        let settings: Vec<DnsSetting> = DnsSetting::all().exec(&mut conn).await?;
        Ok(settings.into_iter().next())
    }

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

        // Get protocols, optionally filtered by group
        let protocol_rows: Vec<(i64, i32, String)> = if let Some(gid) = group_id {
            let rows = toasty::sql::query(
                "SELECT p.id, p.config_type, p.core_type \
                 FROM protocol_rows p \
                 INNER JOIN endpoint_groups eg ON eg.endpoint_id = p.endpoint_id \
                 WHERE eg.group_id = ?1",
            )
            .bind(gid)
            .exec(&mut conn)
            .await?;

            let mut out = Vec::with_capacity(rows.len());
            for value in rows {
                if let Value::Record(fields) = value {
                    out.push((
                        get_i64(&fields, 0)?,
                        get_i64(&fields, 1)? as i32,
                        get_string(&fields, 2)?,
                    ));
                }
            }
            out
        } else {
            let rows = toasty::sql::query(
                "SELECT p.id, p.config_type, p.core_type \
                 FROM protocol_rows p",
            )
            .exec(&mut conn)
            .await?;

            let mut out = Vec::with_capacity(rows.len());
            for value in rows {
                if let Value::Record(fields) = value {
                    out.push((
                        get_i64(&fields, 0)?,
                        get_i64(&fields, 1)? as i32,
                        get_string(&fields, 2)?,
                    ));
                }
            }
            out
        };

        // Dedup by (config_type, core_type) triplet and assign rank
        let mut seen = HashSet::new();
        let mut rank = 0i32;
        let mut items = Vec::with_capacity(protocol_rows.len());
        for (pid, ct, ct_str) in &protocol_rows {
            let triplet = (*ct, ct_str.clone());
            if seen.insert(triplet) {
                rank += 1;
            }
            items.push((*pid, *ct, ct_str.clone(), rank));
        }

        let mut tx = conn.transaction().await?;
        let mut inserted = 0usize;
        for (pid, ct, ct_str, trank) in &items {
            toasty::sql::statement(
                "INSERT INTO ping_sessions \
                 (id, batch_id, protocol_id, config_type, core_type, ping_type, status, triplet_rank) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'fast', 'queued', ?6)",
            )
            .bind(format!("{batch_id}-{pid}"))
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

    pub async fn get_ping_sessions(&self, batch_id: &str) -> Result<Vec<PingSession>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT id, batch_id, protocol_id, config_type, core_type, address, port, triplet_rank, \
                    ping_type, status, latency_ms, speed_bps, ip_info, error, created_at, updated_at \
             FROM ping_sessions WHERE batch_id = ?1 \
             ORDER BY triplet_rank, id",
        )
        .bind(batch_id)
        .exec(&mut conn)
        .await?;
        deserialize_ping_sessions(rows)
    }

    pub async fn get_ping_sessions_by_batch(
        &self,
        batch_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<PingSession>> {
        let mut conn = self.db.connection().await?;
        let rows = toasty::sql::query(
            "SELECT id, batch_id, protocol_id, config_type, core_type, address, port, triplet_rank, \
                    ping_type, status, latency_ms, speed_bps, ip_info, error, created_at, updated_at \
             FROM ping_sessions WHERE batch_id = ?1 \
             ORDER BY triplet_rank, id LIMIT ?2 OFFSET ?3",
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
            .bind(r.speed_bps.unwrap_or(0))
            .bind(r.ip_info.as_deref().unwrap_or(""))
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
            "SELECT id, batch_id, protocol_id, config_type, core_type, address, port, triplet_rank, \
                    ping_type, status, latency_ms, speed_bps, ip_info, error, created_at, updated_at \
             FROM ping_sessions \
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
            "SELECT id, batch_id, protocol_id, config_type, core_type, address, port, triplet_rank, \
                    ping_type, status, latency_ms, speed_bps, ip_info, error, created_at, updated_at \
             FROM ping_sessions \
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
            ProfileExtension::upsert_by_protocol_id(ext.protocol_id)
                .delay(ext.delay.unwrap_or(0))
                .speed(ext.speed.unwrap_or(0))
                .sort_order(ext.sort_order.unwrap_or(0))
                .ip_info(ext.ip_info.as_deref().unwrap_or(""))
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
            .bind(r.speed_bps.unwrap_or(0))
            .bind(r.ip_info.as_deref().unwrap_or(""))
            .bind(r.error.as_deref())
            .bind(&r.session_id)
            .bind(batch_id)
            .exec(&mut tx)
            .await?;
        }

        for ext in extensions {
            ProfileExtension::upsert_by_protocol_id(ext.protocol_id)
                .delay(ext.delay.unwrap_or(0))
                .speed(ext.speed.unwrap_or(0))
                .sort_order(ext.sort_order.unwrap_or(0))
                .ip_info(ext.ip_info.as_deref().unwrap_or(""))
                .exec(&mut tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_ping_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.db.connection().await?;
        toasty::sql::statement("DELETE FROM ping_sessions WHERE id = ?1")
            .bind(session_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }
}

// ── Deserialization helpers ────────────────────────────────────────────

/// Deserialize JOIN query results into `EndpointRow` instances.
/// Column order (0-32):
///   0-8: Endpoint fields
///   9-21: `ProtocolRow` fields
///   22-26: `ProfileExtension` fields (nullable from LEFT JOIN)
///   27-32: `ServerStat` fields (nullable from LEFT JOIN)
fn deserialize_endpoint_rows(rows: Vec<Value>) -> Result<Vec<EndpointRow>> {
    let mut map: HashMap<i64, EndpointRow> = HashMap::new();
    // Preserve insertion order
    let mut order: Vec<i64> = Vec::with_capacity(rows.len());

    for value in rows {
        if let Value::Record(fields) = value {
            let eid = get_i64(&fields, 0)?;

            if let std::collections::hash_map::Entry::Vacant(e) = map.entry(eid) {
                order.push(eid);
                e.insert(EndpointRow {
                    endpoint: Endpoint {
                        id: eid,
                        host: get_string(&fields, 1)?,
                        host_type: get_string(&fields, 2)?,
                        port: get_i64(&fields, 3)? as i32,
                        port_spec_str: get_opt_string(&fields, 4),
                        parent_id: get_opt_i64(&fields, 5),
                        last_source: get_opt_string(&fields, 6),
                        created_at: get_i64(&fields, 7)?,
                        manual_protocol_override: get_opt_i64(&fields, 8),
                    },
                    protocols: Vec::new(),
                    extensions: HashMap::new(),
                    stats: HashMap::new(),
                    resolved_ips: Vec::new(),
                    selected_protocol: 0,
                    expanded: false,
                });
            }

            if let Some(entry) = map.get_mut(&eid) {
                // Add protocol (dedup by id)
                let pid = get_i64(&fields, 9)?;
                if !entry.protocols.iter().any(|p| p.id == pid) {
                    entry.protocols.push(ProtocolRow {
                        id: pid,
                        endpoint_id: eid,
                        sig: get_i64(&fields, 11)?,
                        cred_hash: get_i64(&fields, 12)?,
                        proto_kind: get_string(&fields, 13)?,
                        spec_blob: get_blob(&fields, 14),
                        config_type: get_i64(&fields, 15)? as i32,
                        core_type: get_string(&fields, 16)?,
                        transport: get_opt_string(&fields, 17),
                        security: get_opt_string(&fields, 18),
                        remarks: get_opt_string(&fields, 19),
                        created_at: get_i64(&fields, 20)?,
                        last_seen_at: get_i64(&fields, 21)?,
                        extension: Default::default(),
                        server_stat: Default::default(),
                        endpoint: Default::default(),
                    });
                }

                // Add extension if present (field 22 is protocol_id, non-null = exists)
                if let Some(ext_pid) = get_opt_i64(&fields, 22) {
                    entry
                        .extensions
                        .entry(ext_pid)
                        .or_insert_with(|| ProfileExtension {
                            protocol_id: ext_pid,
                            delay: get_opt_i64(&fields, 23).map(|v| v as i32),
                            speed: get_opt_i64(&fields, 24).map(|v| v as i32),
                            sort_order: get_opt_i64(&fields, 25).map(|v| v as i32),
                            ip_info: get_opt_string(&fields, 26),
                            protocol_row: Default::default(),
                        });
                }

                // Add stats if present (field 27 is protocol_id, non-null = exists)
                if let Some(stat_pid) = get_opt_i64(&fields, 27) {
                    entry.stats.entry(stat_pid).or_insert_with(|| ServerStat {
                        protocol_id: stat_pid,
                        today_up: get_opt_i64(&fields, 28),
                        today_down: get_opt_i64(&fields, 29),
                        total_up: get_opt_i64(&fields, 30),
                        total_down: get_opt_i64(&fields, 31),
                        last_updated: get_opt_string(&fields, 32),
                        protocol_row: Default::default(),
                    });
                }
            }
        }
    }

    // Return in insertion order
    Ok(order.into_iter().filter_map(|id| map.remove(&id)).collect())
}

/// Deserialize ping session rows.
fn deserialize_ping_sessions(rows: Vec<Value>) -> Result<Vec<PingSession>> {
    let mut sessions = Vec::with_capacity(rows.len());
    for value in rows {
        if let Value::Record(fields) = value {
            sessions.push(PingSession {
                id: get_string(&fields, 0)?,
                batch_id: get_string(&fields, 1)?,
                protocol_id: get_i64(&fields, 2)?,
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
