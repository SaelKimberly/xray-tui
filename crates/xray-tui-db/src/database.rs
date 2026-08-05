use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::Path;
use toasty::stmt::IntoStatement;
use toasty_core::schema::db::Type as DbType;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::hash::stable_hash;
use crate::retry_on_busy;
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
    /// Opens existing DB or creates fresh. Recovers from corruption by recreating.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;

        // If file is empty (0 bytes), delete so toasty can create it fresh
        if Path::new(path_str).exists() && std::fs::metadata(path_str)?.len() == 0 {
            std::fs::remove_file(path_str)?;
        }

        let driver = toasty_driver_turso::Turso::file(path_str);
        let db = match Self::try_open_db(driver).await {
            Ok(db) => db,
            Err(e) => {
                // DB might be corrupted — log warning, delete, recreate
                tracing::warn!(error = %e, "DB open failed, attempting recovery by recreating");
                if Path::new(path_str).exists() {
                    std::fs::remove_file(path_str)?;
                }
                let driver = toasty_driver_turso::Turso::file(path_str);
                Self::try_open_db(driver).await?
            }
        };

        let mut conn = db.connection().await?;

        // Check schema version before running push_schema.
        // toasty 0.9 uses CREATE TABLE without IF NOT EXISTS, so we must
        // only run push_schema on fresh databases. Existing databases are
        // upgraded in place by an explicit transaction of idempotent
        // ALTER TABLE / CREATE INDEX statements.
        const SCHEMA_VERSION: i64 = 4;
        let rows = toasty::sql::query("PRAGMA user_version")
            .exec(&mut conn)
            .await?;
        let current_version: i64 = rows
            .first()
            .and_then(|v| {
                if let Value::Record(fields) = v {
                    fields.first().and_then(|f| match f {
                        Value::I64(n) => Some(*n),
                        _ => None,
                    })
                } else {
                    None
                }
            })
            .unwrap_or(0);

        if current_version < SCHEMA_VERSION {
            if current_version < 1 {
                db.push_schema().await?;
            }
            // Additive migration for databases created before these columns
            // existed. push_schema never runs on them, so ALTER in place.
            // Runs in one transaction so the whole upgrade is atomic and
            // committed before any other connection sees the schema.
            let mut tx = conn.transaction().await?;
            Self::ensure_column(
                &mut tx,
                "protocol_rows",
                "last_used_at",
                "ALTER TABLE protocol_rows ADD COLUMN last_used_at INTEGER",
            )
            .await?;
            Self::ensure_column(
                &mut tx,
                "endpoints",
                "resolved_as",
                "ALTER TABLE endpoints ADD COLUMN resolved_as TEXT",
            )
            .await?;
            Self::ensure_column(
                &mut tx,
                "endpoints",
                "resolved_at",
                "ALTER TABLE endpoints ADD COLUMN resolved_at INTEGER",
            )
            .await?;
            Self::ensure_column(
                &mut tx,
                "dns_settings",
                "cache_ttl_secs",
                "ALTER TABLE dns_settings ADD COLUMN cache_ttl_secs INTEGER",
            )
            .await?;
            Self::ensure_column(
                &mut tx,
                "profile_extensions",
                "delay_source",
                "ALTER TABLE profile_extensions ADD COLUMN delay_source INTEGER",
            )
            .await?;
            // get_active_endpoints' correlated MAX(last_seen_at) subquery
            // needs an index on protocol_rows(endpoint_id) or it full-scans
            // per endpoint. push_schema only runs on fresh databases, so
            // existing ones get the index here. The name matches toasty's
            // generated `index_protocol_rows_by_endpoint_id`, keeping the
            // migration idempotent with push_schema-created databases.
            toasty::sql::statement(
                "CREATE INDEX IF NOT EXISTS index_protocol_rows_by_endpoint_id \
                 ON protocol_rows (endpoint_id)",
            )
            .exec(&mut tx)
            .await?;
            toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                .exec(&mut tx)
                .await?;
            tx.commit().await?;
        }

        let _ = toasty::sql::query("PRAGMA journal_mode=WAL")
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

    /// Open a toasty DB by constructing builder. Separate for recovery logic.
    async fn try_open_db(driver: toasty_driver_turso::Turso) -> Result<toasty::Db> {
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
        Ok(db)
    }

    /// Idempotent additive migration: run `ddl` (an `ALTER TABLE ... ADD
    /// COLUMN`) only when `table.column` does not exist yet. Existing
    /// databases never get toasty's `push_schema`, so new columns must be
    /// added in place.
    async fn ensure_column(
        conn: &mut impl toasty::Executor,
        table: &str,
        column: &str,
        ddl: &str,
    ) -> Result<()> {
        let rows = toasty::sql::query(format!(
            "SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"
        ))
        .bind(column)
        .exec(conn)
        .await?;
        let count: i64 = rows
            .first()
            .and_then(|v| {
                if let Value::Record(fields) = v {
                    fields.first().and_then(|f| match f {
                        Value::I64(n) => Some(*n),
                        _ => None,
                    })
                } else {
                    None
                }
            })
            .unwrap_or(0);
        if count == 0 {
            toasty::sql::statement(ddl).exec(conn).await?;
        }
        Ok(())
    }

    /// Acquire a pooled connection with the SQLite busy-wait configured.
    /// `PRAGMA busy_timeout` is per-connection, so the pragma set in `open()`
    /// never reaches pool-created connections. Without it, concurrent
    /// writers (enrichment resolutions, ping-buffer flushes) fail instantly
    /// with "database is locked" instead of queuing behind the lock holder —
    /// this was the 850-line "database is locked" class in the dumps.
    async fn conn(&self) -> Result<toasty::Connection> {
        let mut conn = self.db.connection().await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        Ok(conn)
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
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.last_used_at, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated, \
                    e.resolved_as, e.resolved_at \
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
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.last_used_at, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated, \
                    e.resolved_as, e.resolved_at \
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
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.last_used_at, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated, \
                    e.resolved_as, e.resolved_at \
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
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, p.config_type, p.core_type, p.transport, p.security, p.last_used_at, p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, s.last_updated, \
                    e.resolved_as, e.resolved_at \
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

    /// Look up endpoint row by protocol row id (p.id not e.id).
    pub async fn get_endpoint_by_protocol_id(
        &self,
        protocol_id: i64,
    ) -> Result<Option<EndpointRow>> {
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT e.id, e.host, e.host_type, e.port, e.port_spec_str, e.parent_id, \
                    e.last_source, e.created_at, e.manual_protocol_override, \
                    p.id, p.endpoint_id, p.sig, p.cred_hash, p.proto_kind, p.spec_blob, \
                    p.config_type, p.core_type, p.transport, p.security, p.last_used_at, \
                    p.created_at, p.last_seen_at, \
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
                    s.protocol_id, s.today_up, s.today_down, s.total_up, s.total_down, \
                    s.last_updated, e.resolved_as, e.resolved_at \
             FROM endpoints e \
             INNER JOIN protocol_rows p ON p.endpoint_id = e.id \
             LEFT JOIN profile_extensions ext ON ext.protocol_id = p.id \
             LEFT JOIN server_stats s ON s.protocol_id = p.id \
             WHERE p.id = ?1",
        )
        .bind(protocol_id)
        .exec(&mut conn)
        .await?;
        let mut all = deserialize_endpoint_rows(rows)?;
        Ok(all.pop())
    }

    pub async fn get_profile_extension(
        &self,
        protocol_id: i64,
    ) -> Result<Option<ProfileExtension>> {
        let mut conn = self.conn().await?;
        let ext = ProfileExtension::filter_by_protocol_id(protocol_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(ext)
    }

    pub async fn get_server_stats(&self, protocol_id: i64) -> Result<Option<ServerStat>> {
        let mut conn = self.conn().await?;
        let stats = ServerStat::filter_by_protocol_id(protocol_id)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(stats)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
             (id, endpoint_id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, transport, security, created_at, last_seen_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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

    /// Record when a protocol was last activated; also refreshes `last_seen_at`
    /// so active use keeps a profile out of the Stale/purge lists. ts = unix
    /// seconds. Runs in an explicit transaction — raw statements on a pooled
    /// turso connection do not reliably commit in WAL mode.
    pub async fn update_last_used(&self, protocol_id: i64, ts: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;
        toasty::sql::statement(
            "UPDATE protocol_rows SET last_used_at = ?1, last_seen_at = ?1 WHERE id = ?2",
        )
        .bind(ts)
        .bind(protocol_id)
        .exec(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Persist the DNS resolution of an endpoint host (comma-joined IPs +
    /// unix secs). Both NULL clears the cache (failed lookup). Survives
    /// launches so the TUI does not re-resolve DNS hosts on startup.
    /// Explicit transaction — see `update_last_used`.
    pub async fn update_endpoint_resolution(
        &self,
        endpoint_id: i64,
        resolved_as: Option<&str>,
        resolved_at: Option<i64>,
    ) -> Result<()> {
        // Retry on write contention: the enrichment pipeline resolves many
        // endpoints concurrently, and concurrent writers surface SQLite
        // "database is locked" (previously dropped the resolution write).
        let db = self;
        retry_on_busy(
            move || async move {
                let mut conn = db.db.connection().await?;
                let mut tx = conn.transaction().await?;
                toasty::sql::statement(
                    "UPDATE endpoints SET resolved_as = ?1, resolved_at = ?2 WHERE id = ?3",
                )
                .bind_typed(resolved_as, DbType::Text)
                .bind_typed(resolved_at, DbType::Integer(8))
                .bind(endpoint_id)
                .exec(&mut tx)
                .await?;
                tx.commit().await?;
                Ok(())
            },
            5,
        )
        .await
    }

    /// Purge endpoints where every protocol has `last_seen_at` < threshold.
    /// Returns count of deleted endpoints.
    pub async fn purge_expired(&self, expire_threshold: i64) -> Result<usize> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        toasty::sql::statement("UPDATE protocol_rows SET last_seen_at = ?1 WHERE endpoint_id = ?2")
            .bind(now)
            .bind(endpoint_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Delete an endpoint and all related data.
    pub async fn delete_endpoint(&self, endpoint_id: i64) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        toasty::sql::statement("UPDATE endpoints SET manual_protocol_override = ?1 WHERE id = ?2")
            .bind(protocol_id)
            .bind(endpoint_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn clear_protocol_override(&self, endpoint_id: i64) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;

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

        let mut conn = self.conn().await?;
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

    /// Endpoints whose `parent_id` references `parent_id` (resolved-IP children
    /// of a `DnsName` endpoint). Used by tests; ordered by id.
    pub async fn endpoints_by_parent(&self, parent_id: i64) -> Result<Vec<Endpoint>> {
        let mut conn = self.conn().await?;
        let rows = toasty::sql::query(
            "SELECT id, host, host_type, port, port_spec_str, parent_id, last_source, \
                    created_at, manual_protocol_override, resolved_as, resolved_at \
             FROM endpoints WHERE parent_id = ?1 ORDER BY id",
        )
        .bind(parent_id)
        .exec(&mut conn)
        .await?;
        let mut out = Vec::new();
        for value in rows {
            if let toasty_core::stmt::Value::Record(fields) = value {
                out.push(Endpoint {
                    id: get_i64(&fields, 0)?,
                    host: get_string(&fields, 1)?,
                    host_type: get_string(&fields, 2)?,
                    port: get_i64(&fields, 3)? as i32,
                    port_spec_str: get_opt_string(&fields, 4),
                    parent_id: get_opt_i64(&fields, 5),
                    last_source: get_opt_string(&fields, 6),
                    created_at: get_i64(&fields, 7)?,
                    manual_protocol_override: get_opt_i64(&fields, 8),
                    resolved_as: get_opt_string(&fields, 9),
                    resolved_at: get_opt_i64(&fields, 10),
                });
            }
        }
        Ok(out)
    }

    pub async fn upsert_profile_extension(&self, ext: &ProfileExtension) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileExtension::upsert_by_protocol_id(ext.protocol_id)
            .delay(ext.delay.unwrap_or(0))
            .speed(ext.speed.unwrap_or(0))
            .sort_order(ext.sort_order.unwrap_or(0))
            .ip_info(ext.ip_info.as_deref().unwrap_or(""))
            .delay_source(ext.delay_source.unwrap_or(-1))
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn clear_all_stats(&self) -> Result<()> {
        let mut conn = self.conn().await?;
        toasty::sql::statement("DELETE FROM server_stats")
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn upsert_server_stats(&self, stats: &ServerStat) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        // Capture this group's endpoints BEFORE unlinking. The orphan purge
        // below must be scoped to exactly these ids: profiles preserved by
        // `clear_group` (which unlinks but keeps profiles in the All view)
        // must survive deleting an unrelated group, so a global zero-link
        // predicate is not safe.
        let rows =
            toasty::sql::query("SELECT endpoint_id FROM endpoint_groups WHERE group_id = ?1")
                .bind(id)
                .exec(&mut tx)
                .await?;
        let group_endpoints: Vec<i64> = rows
            .iter()
            .filter_map(|v| {
                if let Value::Record(fields) = v {
                    fields.first().and_then(|f| match f {
                        Value::I64(n) => Some(*n),
                        _ => None,
                    })
                } else {
                    None
                }
            })
            .collect();

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

        // Delete profiles of this group's endpoints that now belong to no
        // group (cascade order: extensions/stats/sessions reference
        // protocol_rows, protocol_rows reference endpoints). Endpoints still
        // linked to another group keep an endpoint_groups row and survive.
        Self::delete_orphaned_profiles(&mut tx, &group_endpoints).await?;

        tx.commit().await?;
        Ok(())
    }

    /// Delete all data of endpoints in `group_endpoints` that no longer
    /// belong to ANY group. Scoped to the caller's group so profiles kept by
    /// `clear_group` or shared with other groups are untouched. Runs inside
    /// the caller's transaction.
    async fn delete_orphaned_profiles(
        tx: &mut dyn toasty::Executor,
        group_endpoints: &[i64],
    ) -> Result<()> {
        if group_endpoints.is_empty() {
            return Ok(());
        }
        let in_list = (1..=group_endpoints.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let orphan_where = format!(
            "e.id IN ({in_list}) AND e.id NOT IN (SELECT DISTINCT endpoint_id FROM endpoint_groups)"
        );

        let statements = [
            format!(
                "DELETE FROM profile_extensions WHERE protocol_id IN ( \
                 SELECT p.id FROM protocol_rows p \
                 INNER JOIN endpoints e ON e.id = p.endpoint_id \
                 WHERE {orphan_where})"
            ),
            format!(
                "DELETE FROM server_stats WHERE protocol_id IN ( \
                 SELECT p.id FROM protocol_rows p \
                 INNER JOIN endpoints e ON e.id = p.endpoint_id \
                 WHERE {orphan_where})"
            ),
            format!(
                "DELETE FROM ping_sessions WHERE protocol_id IN ( \
                 SELECT p.id FROM protocol_rows p \
                 INNER JOIN endpoints e ON e.id = p.endpoint_id \
                 WHERE {orphan_where})"
            ),
            format!(
                "DELETE FROM protocol_rows WHERE endpoint_id IN ({in_list}) \
                 AND endpoint_id NOT IN (SELECT DISTINCT endpoint_id FROM endpoint_groups)"
            ),
            format!(
                "DELETE FROM endpoints WHERE id IN ({in_list}) \
                 AND id NOT IN (SELECT DISTINCT endpoint_id FROM endpoint_groups)"
            ),
        ];

        for sql in statements {
            let mut stmt = toasty::sql::statement(sql);
            for ep_id in group_endpoints {
                stmt = stmt.bind(*ep_id);
            }
            stmt.exec(tx).await?;
        }
        Ok(())
    }

    /// Remove all endpoints from a group, return count of affected links.
    pub async fn clear_group(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        let rules: Vec<RoutingRule> = RoutingRule::all()
            .order_by(RoutingRule::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(rules)
    }

    pub async fn insert_routing_rule(&self, r: &RoutingRule) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        RoutingRule::filter_by_id(id.to_string())
            .delete()
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn reorder_routing_rules(&self, ids: &[(String, i32)]) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        let settings: Vec<DnsSetting> = DnsSetting::all().exec(&mut conn).await?;
        Ok(settings.into_iter().next())
    }

    pub async fn upsert_dns_settings(&self, dns: &DnsSetting) -> Result<()> {
        let mut conn = self.conn().await?;
        toasty::sql::statement(
            "INSERT INTO dns_settings (id, name, servers, hosts, query_strategy, disable_cache, disable_fallback, client_ip, cache_ttl_secs) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
             ON CONFLICT(id) DO UPDATE SET \
             name=excluded.name, servers=excluded.servers, hosts=excluded.hosts, \
             query_strategy=excluded.query_strategy, disable_cache=excluded.disable_cache, \
             disable_fallback=excluded.disable_fallback, client_ip=excluded.client_ip, \
             cache_ttl_secs=excluded.cache_ttl_secs",
        )
        .bind(&dns.id)
        .bind(dns.name.as_deref())
        .bind(dns.servers.as_deref())
        .bind(dns.hosts.as_deref())
        .bind(dns.query_strategy.as_deref())
        .bind(dns.disable_cache)
        .bind(dns.disable_fallback)
        .bind(dns.client_ip.as_deref())
        .bind(dns.cache_ttl_secs)
        .exec(&mut conn)
        .await?;
        Ok(())
    }
}

// ── Ping batch management ───────────────────────────────────────────────

impl Database {
    /// Create a ping batch.
    ///
    /// `profiles` optionally provides `(protocol_id, sort_order)` pairs — when set,
    /// only those profiles get sessions and their `sort_order` replaces `triplet_rank`.
    /// `group_id` is ignored when `profiles` is `Some`.
    pub async fn create_ping_batch(
        &self,
        batch_id: &str,
        group_id: Option<&str>,
        profiles: Option<&[(i64, i32)]>,
    ) -> Result<usize> {
        let mut conn = self.conn().await?;

        // Helper to deserialize protocol rows with endpoint address/port
        let deserialize = |rows: Vec<Value>| -> Result<Vec<(i64, i32, String, String, i32)>> {
            let mut out = Vec::with_capacity(rows.len());
            for value in rows {
                if let Value::Record(fields) = value {
                    out.push((
                        get_i64(&fields, 0)?,
                        get_i64(&fields, 1)? as i32,
                        get_string(&fields, 2)?,
                        get_string(&fields, 3)?,
                        get_i64(&fields, 4)? as i32,
                    ));
                }
            }
            Ok(out)
        };

        // Get protocol rows with endpoint address/port
        let protocol_rows = if profiles.is_some() {
            // Profiles provided — ignore group_id, SELECT all with endpoint join
            let rows = toasty::sql::query(
                "SELECT p.id, p.config_type, p.core_type, e.host, e.port \
                 FROM protocol_rows p \
                 JOIN endpoints e ON e.id = p.endpoint_id",
            )
            .exec(&mut conn)
            .await?;
            deserialize(rows)?
        } else if let Some(gid) = group_id {
            // Group filter
            let rows = toasty::sql::query(
                "SELECT p.id, p.config_type, p.core_type, e.host, e.port \
                 FROM protocol_rows p \
                 INNER JOIN endpoint_groups eg ON eg.endpoint_id = p.endpoint_id \
                 JOIN endpoints e ON e.id = p.endpoint_id \
                 WHERE eg.group_id = ?1",
            )
            .bind(gid)
            .exec(&mut conn)
            .await?;
            deserialize(rows)?
        } else {
            // All rows
            let rows = toasty::sql::query(
                "SELECT p.id, p.config_type, p.core_type, e.host, e.port \
                 FROM protocol_rows p \
                 JOIN endpoints e ON e.id = p.endpoint_id",
            )
            .exec(&mut conn)
            .await?;
            deserialize(rows)?
        };

        // Build items with rank and filter
        let items: Vec<(i64, i32, String, i32, String, i32)>;
        if let Some(profile_list) = profiles {
            // Use sort_order from profiles, filter to listed protocol_ids
            let rank_map: HashMap<i64, i32> = profile_list.iter().copied().collect();
            items = protocol_rows
                .into_iter()
                .filter_map(|(pid, ct, ct_str, addr, port)| {
                    rank_map
                        .get(&pid)
                        .map(|&rank| (pid, ct, ct_str, rank, addr, port))
                })
                .collect();
        } else {
            // Existing triplet_rank dedup
            let mut seen = HashSet::new();
            let mut rank = 0i32;
            items = protocol_rows
                .into_iter()
                .map(|(pid, ct, ct_str, addr, port)| {
                    let triplet = (ct, ct_str.clone());
                    if seen.insert(triplet) {
                        rank += 1;
                    }
                    (pid, ct, ct_str, rank, addr, port)
                })
                .collect();
        }

        let mut tx = conn.transaction().await?;
        let inserted = items.len();
        for (pid, ct, ct_str, trank, addr, port) in &items {
            toasty::sql::statement(
                "INSERT INTO ping_sessions \
                 (id, batch_id, protocol_id, config_type, core_type, ping_type, status, \
                  triplet_rank, address, port) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'fast', 'queued', ?6, ?7, ?8)",
            )
            .bind(format!("{batch_id}-{pid}"))
            .bind(batch_id)
            .bind(*pid)
            .bind(*ct)
            .bind(ct_str)
            .bind(*trank)
            .bind(addr)
            .bind(*port)
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn get_ping_sessions(&self, batch_id: &str) -> Result<Vec<PingSession>> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        for r in results {
            toasty::sql::statement(
                "UPDATE ping_sessions SET status=?1, latency_ms=?2, speed_bps=?3, ip_info=?4, error=?5, updated_at=datetime('now') \
                 WHERE id=?6 AND batch_id=?7",
            )
            .bind(&r.status)
            .bind_typed(r.latency_ms, DbType::Integer(4))
            .bind(r.speed_bps.unwrap_or(0))
            .bind(r.ip_info.as_deref().unwrap_or(""))
            .bind_typed(r.error.as_deref(), DbType::Text)
            .bind(&r.session_id)
            .bind(batch_id)
            .exec(&mut tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Cancel sessions stranded by the race: demoted to real/queued AFTER the endpoint
    /// already has a completed real ping in Phase 2. These sessions are invisible to both
    /// the fast-ping pager (type != 'fast') and the real-ping query (excluded by NOT EXISTS).
    pub async fn cancel_stranded_real_pings(&self, batch_id: &str) -> Result<usize> {
        let mut conn = self.conn().await?;
        let updated = toasty::sql::statement(
            "UPDATE ping_sessions \
             SET status = 'cancelled', error = 'Endpoint already completed', \
                 updated_at = datetime('now') \
             WHERE batch_id = ?1 \
               AND status = 'queued' \
               AND ping_type = 'real' \
               AND EXISTS ( \
                   SELECT 1 FROM ping_sessions ps_s \
                   WHERE ps_s.batch_id = ping_sessions.batch_id \
                     AND ps_s.address = ping_sessions.address \
                     AND ps_s.port = ping_sessions.port \
                     AND ps_s.ping_type = 'real' \
                     AND ps_s.status = 'completed' \
               )",
        )
        .bind(batch_id)
        .exec(&mut conn)
        .await?;
        Ok(updated as usize)
    }

    pub async fn cancel_ping_batch(&self, batch_id: &str) -> Result<usize> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
        toasty::sql::statement("DELETE FROM ping_sessions WHERE batch_id = ?1")
            .bind(batch_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_session_status(&self, session_id: &str, status: &str) -> Result<()> {
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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
        let mut conn = self.conn().await?;
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

    /// Fetch queued real-ping sessions for `wave` (occurrence rank within each
    /// endpoint, ordered by `config_type` then `protocol_id`) of `batch_id`.
    ///
    /// Occurrence is computed over ALL `ping_type='real'` rows of the batch
    /// (status-independent), so wave ranks stay STABLE across dispatches: a
    /// rank-k row of an endpoint implies its rank-(k-1) row was created first,
    /// so an empty wave means no queued row at any higher rank either.
    /// `status = 'queued'` is applied in the outer query.
    ///
    /// When `dedup_endpoints` is true, skips endpoints (address+port) that
    /// already have a completed real ping — the all-visible batch behavior.
    /// Endpoint-scoped batches pass `false` so every protocol of one endpoint
    /// gets real-pinged (different credentials → different exit IPs).
    pub async fn get_batch_for_real_ping(
        &self,
        batch_id: &str,
        wave: i64,
        limit: usize,
        dedup_endpoints: bool,
    ) -> Result<Vec<PingSession>> {
        let mut conn = self.conn().await?;
        let dedup_sql = if dedup_endpoints {
            "AND NOT EXISTS ( \
                      SELECT 1 FROM ping_sessions ps_s \
                      WHERE ps_s.batch_id = ps.batch_id \
                        AND ps_s.address = ps.address \
                        AND ps_s.port = ps.port \
                        AND ps_s.ping_type = 'real' \
                        AND ps_s.status = 'completed' \
                  ) \
"
        } else {
            ""
        };
        let query = format!(
            "SELECT sub.id, sub.batch_id, sub.protocol_id, sub.config_type, sub.core_type, \
                    sub.address, sub.port, sub.triplet_rank, \
                    sub.ping_type, sub.status, sub.latency_ms, sub.speed_bps, \
                    sub.ip_info, sub.error, sub.created_at, sub.updated_at \
             FROM ( \
                SELECT ps.*, \
                       COUNT(*) OVER ( \
                           PARTITION BY ps.address, ps.port \
                           ORDER BY ps.config_type, ps.protocol_id \
                       ) AS occurrence \
                FROM ping_sessions ps \
                WHERE ps.batch_id = ?1 \
                  AND ps.ping_type = 'real' \
                  {dedup_sql}     ) sub \
             WHERE sub.occurrence = ?2 AND sub.status = 'queued' \
             ORDER BY sub.id \
             LIMIT ?3",
        );
        let rows = toasty::sql::query(&query)
            .bind(batch_id)
            .bind(wave)
            .bind(limit as i64)
            .exec(&mut conn)
            .await?;
        deserialize_ping_sessions(rows)
    }

    pub async fn batch_upsert_profile_extensions(
        &self,
        extensions: &[ProfileExtension],
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        for ext in extensions {
            ProfileExtension::upsert_by_protocol_id(ext.protocol_id)
                .delay(ext.delay.unwrap_or(0))
                .speed(ext.speed.unwrap_or(0))
                .sort_order(ext.sort_order.unwrap_or(0))
                .ip_info(ext.ip_info.as_deref().unwrap_or(""))
                .delay_source(ext.delay_source.unwrap_or(-1))
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
        // Retry on write contention — the flush collides with enrichment
        // writes during batch pings; previously the whole buffer was dropped.
        let db = self;
        retry_on_busy(
            move || async move {
                let mut conn = db.db.connection().await?;
                let mut tx = conn.transaction().await?;

                for r in results {
                    toasty::sql::statement(
                        "UPDATE ping_sessions SET status=?1, latency_ms=?2, speed_bps=?3, ip_info=?4, error=?5, updated_at=datetime('now') \
                         WHERE id=?6 AND batch_id=?7",
                    )
                    .bind(&r.status)
                    .bind_typed(r.latency_ms, DbType::Integer(4))
                    .bind(r.speed_bps.unwrap_or(0))
                    .bind(r.ip_info.as_deref().unwrap_or(""))
                    .bind_typed(r.error.as_deref(), DbType::Text)
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
                        .delay_source(ext.delay_source.unwrap_or(-1))
                        .exec(&mut tx)
                        .await?;
                }

                tx.commit().await?;
                Ok(())
            },
            5,
        )
        .await
    }

    pub async fn delete_ping_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        toasty::sql::statement("DELETE FROM ping_sessions WHERE id = ?1")
            .bind(session_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }
}

// ── Deserialization helpers ────────────────────────────────────────────

/// Deserialize JOIN query results into `EndpointRow` instances.
/// Column order (0-35):
///   0-8: Endpoint fields
///   9-21: `ProtocolRow` fields
///   22-27: `ProfileExtension` fields (nullable from LEFT JOIN)
///   28-33: `ServerStat` fields (nullable from LEFT JOIN)
///   34-35: `resolved_as`, `resolved_at`
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
                        resolved_as: get_opt_string(&fields, 34),
                        resolved_at: get_opt_i64(&fields, 35),
                    },
                    protocols: Vec::new(),
                    extensions: HashMap::new(),
                    stats: HashMap::new(),
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
                        last_used_at: get_opt_i64(&fields, 19),
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
                            speed: get_opt_i64(&fields, 24),
                            sort_order: get_opt_i64(&fields, 25).map(|v| v as i32),
                            ip_info: get_opt_string(&fields, 26),
                            delay_source: get_opt_i64(&fields, 27).map(|v| v as i32),
                            protocol_row: Default::default(),
                        });
                }

                // Add stats if present (field 28 is protocol_id, non-null = exists)
                if let Some(stat_pid) = get_opt_i64(&fields, 28) {
                    entry.stats.entry(stat_pid).or_insert_with(|| ServerStat {
                        protocol_id: stat_pid,
                        today_up: get_opt_i64(&fields, 29),
                        today_down: get_opt_i64(&fields, 30),
                        total_up: get_opt_i64(&fields, 31),
                        total_down: get_opt_i64(&fields, 32),
                        last_updated: get_opt_string(&fields, 33),
                        protocol_row: Default::default(),
                    });
                }
            }
        }
    }

    // Sort each endpoint's protocols by test priority: measured latency
    // (real ping first, then fast/UDP) ascending on top, untested protocols
    // by last_seen_at descending (newest variant first, today's default),
    // failures and DNS-unresolved hosts sink to the bottom. `rounds` is None
    // here — failures are session-only state, applied live in the TUI.
    // Persisted resolution counts as resolved: `resolved_as` empty means the
    // host is unresolved (name tier) until a live resolution event flips it.
    for row in map.values_mut() {
        let dns_unresolved = row.endpoint.host_type == "dns"
            && row.endpoint.resolved_as.as_deref().is_none_or(str::is_empty);
        row.sort_protocols_by_test_priority(dns_unresolved, None);
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
                speed_bps: get_opt_i64(&fields, 11),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Simulate a pre-migration database (schema v1: no last_used_at /
    /// resolved_as / resolved_at / cache_ttl_secs) and prove that
    /// `Database::open` migrates it in place.
    #[tokio::test]
    async fn test_open_migrates_old_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("old.db");

        // 1. Build the v2 schema, then strip the new columns and rewind
        //    user_version to 1 — the shape of a pre-migration database.
        {
            let driver = toasty_driver_turso::Turso::file(&path);
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
                .await
                .expect("build db");
            db.push_schema().await.expect("push schema");
            let mut conn = db.connection().await.expect("connection");
            toasty::sql::statement("ALTER TABLE protocol_rows DROP COLUMN last_used_at")
                .exec(&mut conn)
                .await
                .expect("drop last_used_at");
            toasty::sql::statement("ALTER TABLE endpoints DROP COLUMN resolved_as")
                .exec(&mut conn)
                .await
                .expect("drop resolved_as");
            toasty::sql::statement("ALTER TABLE endpoints DROP COLUMN resolved_at")
                .exec(&mut conn)
                .await
                .expect("drop resolved_at");
            toasty::sql::statement("ALTER TABLE dns_settings DROP COLUMN cache_ttl_secs")
                .exec(&mut conn)
                .await
                .expect("drop cache_ttl_secs");
            toasty::sql::query("PRAGMA user_version = 1")
                .exec(&mut conn)
                .await
                .expect("set version 1");
        }

        // 2. Reopen through the app's Database — the migration must re-add
        //    the columns (a SELECT touching last_used_at would otherwise fail).
        let db = Database::open(&path).await.expect("open migrates schema");
        let row = db
            .get_endpoint(0)
            .await
            .expect("endpoint query uses last_used_at");
        assert!(row.is_none(), "empty old db has no endpoints");

        // Reopening again must be a no-op (idempotent).
        let db2 = Database::open(&path).await.expect("reopen");
        assert!(db2.get_endpoint(0).await.is_ok());
    }

    /// Simulate a v3 database (no delay_source column) and prove `Database::open`
    /// re-adds it in place — push_schema never runs on existing databases.
    #[tokio::test]
    async fn test_open_adds_delay_source_column() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("v3.db");
        {
            let driver = toasty_driver_turso::Turso::file(&path);
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
                .await
                .expect("build db");
            db.push_schema().await.expect("push schema");
            let mut conn = db.connection().await.expect("connection");
            toasty::sql::statement("ALTER TABLE profile_extensions DROP COLUMN delay_source")
                .exec(&mut conn)
                .await
                .expect("drop delay_source");
            toasty::sql::query("PRAGMA user_version = 3")
                .exec(&mut conn)
                .await
                .expect("set version 3");
        }
        let db = Database::open(&path).await.expect("open migrates delay_source");
        let mut conn = db.db.connection().await.expect("connection");
        let rows = toasty::sql::query(
            "SELECT COUNT(*) FROM pragma_table_info('profile_extensions') WHERE name = 'delay_source'",
        )
        .exec(&mut conn)
        .await
        .expect("pragma");
        let count: i64 = rows
            .first()
            .and_then(|v| {
                if let Value::Record(fields) = v {
                    fields.first().and_then(|f| match f {
                        Value::I64(n) => Some(*n),
                        _ => None,
                    })
                } else {
                    None
                }
            })
            .unwrap_or(0);
        assert_eq!(count, 1, "open() did not add profile_extensions.delay_source");
    }

    /// `delay_source` plus the shifted stats/resolved column indices round-trip
    /// through `get_active_endpoints` (a misaligned SELECT breaks these reads).
    #[tokio::test]
    async fn delay_source_roundtrips_through_get_active_endpoints() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.db.connection().await.expect("connection");
        toasty::sql::statement(
            "INSERT INTO endpoints (id, host, host_type, port, created_at, resolved_as, resolved_at) \
             VALUES (1001, 'dns.example', 'dns', 443, 0, '1.2.3.4', 100)",
        )
        .exec(&mut conn)
        .await
        .expect("insert endpoint");
        toasty::sql::statement(
            "INSERT INTO protocol_rows \
               (id, endpoint_id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, created_at, last_seen_at) \
             VALUES (2001, 1001, 1, 0, 'vmess', X'', 0, 'xray', 0, 10)",
        )
        .exec(&mut conn)
        .await
        .expect("insert protocol");
        toasty::sql::statement(
            "INSERT INTO profile_extensions (protocol_id, delay, speed, sort_order, ip_info, delay_source) \
             VALUES (2001, 123, 0, 0, '1.2.3.4|US', 1)",
        )
        .exec(&mut conn)
        .await
        .expect("insert extension");
        toasty::sql::statement(
            "INSERT INTO server_stats (protocol_id, total_down, total_up) VALUES (2001, 5, 6)",
        )
        .exec(&mut conn)
        .await
        .expect("insert stats");

        let rows = db.get_active_endpoints(0).await.expect("load");
        assert_eq!(rows.len(), 1);
        let ext = &rows[0].extensions[&2001];
        assert_eq!(ext.delay, Some(123));
        assert_eq!(ext.delay_source, Some(1));
        assert_eq!(ext.ip_info.as_deref(), Some("1.2.3.4|US"));
        let st = &rows[0].stats[&2001];
        assert_eq!(st.total_down, Some(5));
        assert_eq!(st.total_up, Some(6));
        assert_eq!(rows[0].endpoint.resolved_as.as_deref(), Some("1.2.3.4"));
        assert_eq!(rows[0].endpoint.resolved_at, Some(100));
    }

    /// `get_active_endpoints` runs a correlated subquery
    /// `(SELECT COALESCE(MAX(p2.last_seen_at), 0) FROM protocol_rows p2
    /// WHERE p2.endpoint_id = e.id)` once per endpoint. Without an index on
    /// `protocol_rows(endpoint_id)` every subquery is a full table scan, so
    /// profile lists degrade quadratically. Prove the schema (created by
    /// toasty's `push_schema` from the model) has an index covering it.
    #[tokio::test]
    async fn protocol_rows_are_indexed_by_endpoint_id() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.db.connection().await.expect("connection");

        // sqlite_master records every CREATE INDEX; the autoindex behind the
        // PRIMARY KEY has NULL sql and never matches. Assert a real index on
        // protocol_rows references the endpoint_id column.
        let rows = toasty::sql::query(
            "SELECT name FROM sqlite_master \
             WHERE type = 'index' AND tbl_name = 'protocol_rows' \
               AND sql LIKE '%endpoint_id%'",
        )
        .exec(&mut conn)
        .await
        .expect("query sqlite_master for protocol_rows indexes");
        assert!(
            !rows.is_empty(),
            "protocol_rows has no index on endpoint_id; \
             get_active_endpoints' correlated MAX(last_seen_at) subquery \
             full-scans the table once per endpoint"
        );
    }

    /// Simulate a database created before the endpoint_id index existed
    /// (schema version 2, index dropped) and prove that `Database::open`
    /// recreates the index in place — push_schema never runs on existing
    /// databases, so the migration transaction must.
    #[tokio::test]
    async fn test_open_migrates_adds_endpoint_id_index() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("indexless.db");

        // 1. Build the current schema, then drop the endpoint_id index and
        //    rewind user_version to 2 — the shape of a pre-index database.
        {
            let driver = toasty_driver_turso::Turso::file(&path);
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
                .await
                .expect("build db");
            db.push_schema().await.expect("push schema");
            let mut conn = db.connection().await.expect("connection");
            toasty::sql::statement("DROP INDEX IF EXISTS index_protocol_rows_by_endpoint_id")
                .exec(&mut conn)
                .await
                .expect("drop endpoint_id index");
            toasty::sql::query("PRAGMA user_version = 2")
                .exec(&mut conn)
                .await
                .expect("set version 2");
        }

        // 2. Reopen through the app's Database — the migration must recreate
        //    the index (a query touching endpoint_id would otherwise scan).
        let db = Database::open(&path).await.expect("open migrates index");
        let mut conn = db.db.connection().await.expect("connection");
        let rows = toasty::sql::query(
            "SELECT name FROM sqlite_master \
             WHERE type = 'index' AND tbl_name = 'protocol_rows' \
               AND sql LIKE '%endpoint_id%'",
        )
        .exec(&mut conn)
        .await
        .expect("query sqlite_master for protocol_rows indexes");
        assert!(
            !rows.is_empty(),
            "open() migration did not recreate the protocol_rows(endpoint_id) index"
        );
    }

    /// `get_batch_for_real_ping(batch_id, wave, ...)` must return one queued
    /// session per endpoint at STABLE occurrence ranks: dispatching wave 1
    /// (even failing it) must NOT re-rank the remaining protocols to 1, and
    /// the `dedup_endpoints` NOT EXISTS must only skip endpoints with a
    /// completed real ping.
    #[tokio::test]
    async fn test_wave_query_real_ping_stable_ranks() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.db.connection().await.expect("connection");
        toasty::sql::statement(
            "INSERT INTO endpoints (id, host, host_type, port, created_at) VALUES \
             (1001, '1.1.1.1', 'ipv4', 443, 0), \
             (1002, '2.2.2.2', 'ipv4', 8443, 0)",
        )
        .exec(&mut conn)
        .await
        .expect("insert endpoints");
        toasty::sql::statement(
            "INSERT INTO protocol_rows \
               (id, endpoint_id, sig, cred_hash, proto_kind, spec_blob, config_type, \
                core_type, created_at, last_seen_at) \
             VALUES \
               (2001, 1001, 1, 0, 'vmess', X'', 0, 'xray', 0, 0), \
               (2002, 1001, 2, 0, 'trojan', X'', 1, 'xray', 0, 0), \
               (2003, 1002, 3, 0, 'vmess', X'', 0, 'xray', 0, 0), \
               (2004, 1002, 4, 0, 'socks', X'', 2, 'xray', 0, 0)",
        )
        .exec(&mut conn)
        .await
        .expect("insert protocol_rows");

        let batch_id = "wave-test-batch";
        let count = db
            .create_ping_batch(
                batch_id,
                None,
                Some(&[(2001, 0), (2002, 1), (2003, 0), (2004, 1)]),
            )
            .await
            .expect("create batch");
        assert_eq!(count, 4);

        // Demote all four to real candidates.
        let sessions = db
            .get_ping_sessions_by_batch(batch_id, 100, 0)
            .await
            .expect("sessions");
        assert_eq!(sessions.len(), 4);
        for s in &sessions {
            db.update_session_ping_type(&s.id, "real", "queued")
                .await
                .expect("demote");
        }

        // Wave 1: exactly one session per endpoint (lowest config_type).
        let wave1 = db
            .get_batch_for_real_ping(batch_id, 1, 100, false)
            .await
            .expect("wave1");
        assert_eq!(wave1.len(), 2);
        let wave1_ids: Vec<i64> = wave1.iter().map(|s| s.protocol_id).collect();
        assert!(wave1_ids.contains(&2001));
        assert!(wave1_ids.contains(&2003));

        // Wave 1 fails -> wave 2 still returns the second protocol of each
        // endpoint at its STABLE rank 2 (never re-ranked to 1).
        for s in &wave1 {
            db.update_session_status(&s.id, "failed")
                .await
                .expect("fail");
        }
        let wave2 = db
            .get_batch_for_real_ping(batch_id, 2, 100, false)
            .await
            .expect("wave2");
        assert_eq!(wave2.len(), 2);
        let wave2_ids: Vec<i64> = wave2.iter().map(|s| s.protocol_id).collect();
        assert!(wave2_ids.contains(&2002));
        assert!(wave2_ids.contains(&2004));

        // One endpoint completes -> dedup_endpoints=true skips its remaining
        // protocol; the other endpoint's second protocol still appears.
        let s2001 = wave1.iter().find(|s| s.protocol_id == 2001).unwrap();
        db.update_session_status(&s2001.id, "completed")
            .await
            .expect("complete");
        let wave2_dedup = db
            .get_batch_for_real_ping(batch_id, 2, 100, true)
            .await
            .expect("wave2 dedup");
        let dedup_ids: Vec<i64> = wave2_dedup.iter().map(|s| s.protocol_id).collect();
        assert_eq!(dedup_ids, vec![2004]);

        // Wave 3 is empty — terminates the pass loop.
        let wave3 = db
            .get_batch_for_real_ping(batch_id, 3, 100, false)
            .await
            .expect("wave3");
        assert!(wave3.is_empty());
    }

    /// 50 concurrent writers must all succeed. Pooled turso connections are
    /// created WITHOUT the busy_timeout pragma that `open()` sets on its own
    /// connection (busy_timeout is per-connection), so the enrichment herd /
    /// ping flush used to fail writes instantly with `database is locked`
    /// instead of waiting their turn. Must be multi-threaded — a
    /// current_thread runtime serializes the spawns and never contends.
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_concurrent_writes_no_database_locked() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = std::sync::Arc::new(
            Database::open(dir.path().join("busy.db"))
                .await
                .expect("open db"),
        );
        let handles: Vec<_> = (0..50i64)
            .map(|i| {
                let db = db.clone();
                tokio::spawn(async move { db.update_last_used(i, 1_752_000_000).await })
            })
            .collect();
        for handle in handles {
            handle
                .await
                .expect("task panicked")
                .unwrap_or_else(|e| panic!("concurrent write failed: {e}"));
        }
    }
}
