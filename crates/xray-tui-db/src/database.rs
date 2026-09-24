use std::collections::HashMap;
use std::path::Path;

use toasty::Executor;
use toasty::stmt::IntoStatement;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::models_toasty::{
    DnsSetting, Endpoint, EndpointGroup, EndpointId, EndpointIp, EndpointRank, EndpointRow, Group,
    ProfileStats, Protocol, ProtocolId, RouteProbes, RoutingRule, TrafficStats, now_epoch,
};
use crate::retry_on_busy;

// ── Database handle ─────────────────────────────────────────────────────

pub struct Database {
    db: toasty::Db,
}

/// Which mutable column groups a [`LinkPatch`] writes.
///
/// The groups exist so the two independent writers of a `profile_stats` row —
/// ping results and the scheduler gate — cannot clobber each other's columns
/// when their patches coalesce in the same flush.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinkGroups(u8);

impl LinkGroups {
    /// `latency` + `speed_bps` + `error` (ping results, error TTL sweeps).
    pub const RESULT: Self = Self(0b001);
    /// `purge_reason` (the real probe's verdict).
    ///
    /// Its own bit, not part of RESULT: [`link_patch_conflict_sql`] writes a
    /// FIXED column set from each patch's *snapshot*, so riding RESULT would
    /// let a phase-1 fast half — whose snapshot is the plan-time row — rewrite
    /// a verdict it never classified. That is the clobber shape these groups
    /// exist to prevent (ADR 0002), and the classifier is the only producer.
    pub const PURGE: Self = Self(0b010);
    /// `traffic_*` (the gRPC stats poller).
    pub const TRAFFIC: Self = Self(0b100);
    /// Every group.
    ///
    /// Every group can change a STORED ORDERING KEY — the rank columns derive
    /// from a link's `error`/`latency`/`last_seen_at`/`speed_bps`/`traffic`/
    /// `config_type`/`purge_reason` — so every patch refreshes its endpoint's
    /// keys. (The scheduler's task state used to be a third group; it is
    /// runtime-only now and never reaches this table.)
    pub const ALL: Self = Self(0b111);

    /// Whether `other`'s groups are all present in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any group is shared.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// OR of two groups.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// One row's pending write: the mutable state plus the groups that changed.
#[derive(Debug, Clone)]
pub struct LinkPatch {
    /// Row identity and the new values of the patched groups.
    pub link: ProfileStats,
    /// Groups to write; unset groups keep their persisted values.
    pub groups: LinkGroups,
}

// ── Constructors ────────────────────────────────────────────────────────

impl Database {
    /// Opens existing DB or creates fresh. Recovers from corruption by recreating.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "the Turso driver temporaries are consumed by try_open_db in the very next statement; clippy attributes their drop to the whole fn through the generic await"
    )]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        // Tag for databases created by this 7-table schema. toasty 0.9's
        // push_schema emits CREATE TABLE without IF NOT EXISTS, so it can
        // only run on a database that has no tables yet; the tag lets reopen
        // skip it. Any other tag is a pre-T8 9-table database (incompatible
        // with the typed models) and is recreated from scratch.
        //
        // 7 = per-kind binary identity (`Protocol.id = uid`, see
        // `xray-tui-proto::proto_spec::identity`). v6 rows carry uids hashed
        // from canonical JSON, which are unrelated to the new values, so
        // reusing a v6 file would re-key every protocol into a duplicate row
        // and orphan its links. The bump WIPES the file by design.
        //
        // 8 = `endpoint_rank` (ADR 0003) as a first-class table. This project
        // is pre-alpha, so a new table arrives by the same wipe rather than by
        // migration machinery: a v7 file is discarded and rebuilt with the
        // table present. The stored keys are derived state — a feed is
        // re-imported, and the keys rebuild from its links.
        //
        // 9 = the durable-facts pass: `profile_stats` loses `task_id` /
        // `task_queue` (the scheduler's state is runtime-only — an id is only
        // meaningful inside the process that allocated it), the timestamps
        // become epoch seconds, `protocols.cred_hash` and `endpoints.parent_id`
        // go (both derivable: the uid is recomputed from the config, a DNS
        // endpoint's resolutions live in `resolved_as`), and the two indexes
        // the purge / failed-sweep predicates need are added. Same contract as
        // every bump: a v8 file is WIPED, never migrated.
        //
        // 10 = resolved addresses become a table: the JSON-array
        // `endpoints.resolved_as` column is replaced by `endpoint_ip`, one row
        // per address with a sortable key. A v9 file is WIPED (the addresses it
        // holds cannot be moved: the old column is gone, and the enrichment
        // pipeline re-resolves on the next pass).
        //
        // 11 = `endpoint_ip` carries the address's ISO-3166 country, written
        // once by the mmdb lookup, so a later launch renders the flag without
        // the database. A v10 file is WIPED (a column cannot be added to a
        // pushed table here; the countries are re-derived on the next pass).
        //
        // 12 = `profile_stats.purge_reason` (spec 2026-09-17-purge-reason):
        // the typed, permanent verdict a real probe's evidence writes and only
        // a data-carrying success clears. A v11 file is WIPED (a column cannot
        // be added to a pushed table here; the verdicts are re-derived by the
        // next Real run, and the reference classification is kept beside the
        // dataset copy the design spec names).
        //
        // 13 = no column change at all. The ws `?ed=NNNN` path query is now
        // hoisted into the typed `max_early_data`, which is IN the identity
        // stream (`write_transport` → `TR_MAX_EARLY_DATA`), so every ws config
        // carrying the query re-keys. The bump is the user's call (2026-09-23)
        // to wipe rather than carry the transient duplicate set the re-key
        // would otherwise age out through Purgatory — the stored shapes are
        // unchanged, so a v12 file would open fine; it is dropped for the
        // clean re-import.
        const SCHEMA_VERSION: i64 = 13;

        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;

        // If file is empty (0 bytes), delete so toasty can create it fresh
        if Path::new(path_str).exists() && std::fs::metadata(path_str)?.len() == 0 {
            std::fs::remove_file(path_str)?;
        }

        let driver = toasty_driver_turso::Turso::file(path_str);
        let mut db = match Self::try_open_db(driver).await {
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

        let rows = toasty::sql::query("PRAGMA user_version")
            .exec(&mut conn)
            .await?;
        let current_version = first_i64(&rows).unwrap_or(0);
        if current_version != SCHEMA_VERSION {
            match db.push_schema().await {
                Ok(()) => {
                    toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                        .exec(&mut conn)
                        .await?;
                }
                Err(e) => {
                    // Existing tables (pre-T8 schema or half-created DB):
                    // drop the file and rebuild with the 7-table schema.
                    tracing::warn!(
                        version = current_version,
                        error = %e,
                        "incompatible DB schema, recreating from scratch"
                    );
                    drop(conn);
                    if Path::new(path_str).exists() {
                        std::fs::remove_file(path_str)?;
                    }
                    let driver = toasty_driver_turso::Turso::file(path_str);
                    db = Self::try_open_db(driver).await?;
                    let mut fresh = db.connection().await?;
                    db.push_schema().await?;
                    toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                        .exec(&mut fresh)
                        .await?;
                    conn = fresh;
                }
            }
        }

        let _ = toasty::sql::query("PRAGMA journal_mode=WAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        // WAL + `synchronous=NORMAL` is the standard durability/throughput
        // trade: commits are not fsynced individually (a power loss may lose
        // the most recent transactions, the database stays consistent). The
        // default FULL costs ~4.2 ms per commit on this engine — a ping batch
        // over 30k links issues ~90k commits, which froze the UI task that
        // awaits them (see the 2026-09-11 batch-ping investigation).
        toasty::sql::query("PRAGMA synchronous=NORMAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_groups(&mut conn).await?;
        // Materialized per-endpoint ordering keys (decision 21): additive
        // side table, created here and backfilled when empty. A database that
        // predates it pays the fill once, at open.
        if std::env::var("XRAY_TUI_SKIP_RANK_ENSURE").is_err()
            && let Err(e) = crate::endpoint_rank::ensure(&mut conn).await
        {
            tracing::warn!(target: "xray_tui_db", "endpoint_rank: {e}");
        }
        // `endpoint_ip`'s covering index — the address-ordered read path. Raw
        // DDL for the same reason the rank indexes are: toasty's `#[index]` is
        // single-column, and the sort wants `(ip_key, endpoint_id)` together.
        if let Err(e) = crate::endpoint_ip::ensure(&mut conn).await {
            tracing::warn!(target: "xray_tui_db", "endpoint_ip: {e}");
        }
        Ok(Self { db })
    }

    /// Open a toasty DB by constructing builder. Separate for recovery logic.
    async fn try_open_db(driver: toasty_driver_turso::Turso) -> Result<toasty::Db> {
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                Protocol,
                ProfileStats,
                EndpointGroup,
                Group,
                RoutingRule,
                DnsSetting,
                RouteProbes,
                EndpointRank,
                EndpointIp
            ))
            .build(driver)
            .await?;
        Ok(db)
    }

    /// Acquire a pooled connection with the `SQLite` busy-wait configured.
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
        // Per-connection like `busy_timeout`: the pragma set in `open()` never
        // reaches pool-created connections.
        toasty::sql::query("PRAGMA synchronous=NORMAL")
            .exec(&mut conn)
            .await?;
        Ok(conn)
    }

    /// Public pooled-connection accessor for callers building typed
    /// create/update statements (Task 10 writes; integration tests).
    pub async fn connection(&self) -> Result<toasty::Connection> {
        self.conn().await
    }

    pub async fn in_memory() -> Result<Self> {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                Protocol,
                ProfileStats,
                EndpointGroup,
                Group,
                RoutingRule,
                DnsSetting,
                RouteProbes,
                EndpointRank,
                EndpointIp
            ))
            .build(driver)
            .await?;

        let mut conn = db.connection().await?;
        db.push_schema().await?;

        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        // WAL + `synchronous=NORMAL` is the standard durability/throughput
        // trade: commits are not fsynced individually (a power loss may lose
        // the most recent transactions, the database stays consistent). The
        // default FULL costs ~4.2 ms per commit on this engine — a ping batch
        // over 30k links issues ~90k commits, which froze the UI task that
        // awaits them (see the 2026-09-11 batch-ping investigation).
        toasty::sql::query("PRAGMA synchronous=NORMAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_groups(&mut conn).await?;
        // Materialized per-endpoint ordering keys (decision 21): additive
        // side table, created here and backfilled when empty. A database that
        // predates it pays the fill once, at open.
        if std::env::var("XRAY_TUI_SKIP_RANK_ENSURE").is_err()
            && let Err(e) = crate::endpoint_rank::ensure(&mut conn).await
        {
            tracing::warn!(target: "xray_tui_db", "endpoint_rank: {e}");
        }
        // `endpoint_ip`'s covering index — the address-ordered read path. Raw
        // DDL for the same reason the rank indexes are: toasty's `#[index]` is
        // single-column, and the sort wants `(ip_key, endpoint_id)` together.
        if let Err(e) = crate::endpoint_ip::ensure(&mut conn).await {
            tracing::warn!(target: "xray_tui_db", "endpoint_ip: {e}");
        }
        Ok(Self { db })
    }

    async fn init_default_groups(conn: &mut impl toasty::Executor) -> Result<()> {
        let count = Group::all().count().exec(conn).await?;
        if count == 0 {
            Group::create()
                .id(uuid::Uuid::new_v4().to_string())
                .name(Some("Default".to_string()))
                .enabled(true)
                .sort_order(Some(0))
                .into_statement()
                .exec(conn)
                .await?;
        }
        Ok(())
    }
}

/// Rows per literal statement chunk in the link writers
/// ([`Database::apply_link_patches`], [`upsert_links_bulk`]) — a
/// statement-size bound, not a predicate-depth one: both writers emit one
/// multi-row upsert per chunk, whose width does not change its expression depth
/// (measured 2026-09-16: 400 / 1,000 / 2,000 rows per statement are equal
/// within noise — 11.1 / 10.6 / 10.6 ms for a 512-patch window), so this
/// number is free to move.
const LINK_STATEMENT_ROWS: usize = 400;

/// A SQL TEXT literal: single quotes doubled, so a stored error message can
/// never terminate the literal. The values come from our own rows, never from
/// user input at this layer.
fn sql_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push('\'');
        }
        out.push(ch);
    }
    out.push('\'');
    out
}

/// A SQL TEXT literal for a nullable column (`NULL` when absent).
fn sql_opt_lit(s: Option<&str>) -> String {
    s.map_or_else(|| "NULL".to_string(), sql_lit)
}

/// A SQL integer literal for a nullable column.
fn sql_opt_num(n: Option<i64>) -> String {
    n.map_or_else(|| "NULL".to_string(), |n| n.to_string())
}

const LINK_COMPARE_KEYS: &str = "protocol_id, endpoint_id";

/// The `INSERT … VALUES (…),(…)` prefix both link writers share; a caller
/// appends its tuples and then the `ON CONFLICT` action. The column order IS
/// the order [`link_values_sql`] emits.
const LINK_UPSERT_PREFIX: &str = "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, \
     config_type, last_used_at, last_seen_at, latency, latency_delay, latency_ip, speed_bps, \
     error, error_kind, error_text, purge_reason, traffic_today_up, traffic_today_down, \
     traffic_total_up, traffic_total_down, created_at, updated_at, version) VALUES ";

/// The whole-snapshot SQL tuple for one link.
///
/// SQL literals rather than binds: turso charges ~0.8 ms per bound parameter
/// (200 ids = 174 ms against 9.7 ms for the same statement with literals), and a
/// chunk carries 20 columns per row.
fn link_values_sql(link: &ProfileStats, now: i64) -> String {
    use std::fmt::Write as _;

    let mut sql = String::with_capacity(160);
    let _ = write!(
        sql,
        "({}, {}, '{}', '{}', NULL, {}, ",
        link.protocol_id.get(),
        link.endpoint_id.get(),
        core_type_str(link.core_type),
        config_type_str(link.config_type),
        link.last_seen_at
    );
    match &link.latency {
        Some(crate::models_toasty::Latency::Real { delay, ip }) => {
            let _ = write!(sql, "'real', {delay}, {}", sql_opt_lit(ip.as_deref()));
        }
        Some(crate::models_toasty::Latency::Fast { delay }) => {
            let _ = write!(sql, "'fast', {delay}, NULL");
        }
        None => sql.push_str("NULL, NULL, NULL"),
    }
    let _ = write!(sql, ", {}, ", sql_opt_num(link.speed_bps));
    match &link.error {
        Some(error) => {
            let _ = write!(
                sql,
                "1, {}, {}",
                sql_lit(error_kind_str(error.kind)),
                sql_lit(&error.text)
            );
        }
        None => sql.push_str("NULL, NULL, NULL"),
    }
    let _ = write!(
        sql,
        ", {}",
        sql_opt_lit(link.purge_reason.map(purge_reason_str))
    );
    let _ = write!(
        sql,
        ", {}, {}, {}, {}, {}, {}, 1)",
        link.traffic.today_up,
        link.traffic.today_down,
        link.traffic.total_up,
        link.traffic.total_down,
        now,
        now
    );
    sql
}

/// The `ON CONFLICT (protocol_id, endpoint_id) …` action for one patch's column
/// groups.
///
/// An existing row is written with exactly the patch's groups, so a column
/// another writer owns (`last_used_at`) or another group owns keeps its
/// persisted value — the disjointness [`LinkGroups`] exists for. A patch that
/// carries no group at all only creates a missing row (`DO NOTHING`).
fn link_patch_conflict_sql(has_result: bool, has_traffic: bool, has_purge: bool) -> String {
    let mut sets: Vec<&str> = Vec::with_capacity(12);
    if has_result {
        sets.extend_from_slice(&[
            "latency = excluded.latency",
            "latency_delay = excluded.latency_delay",
            "latency_ip = excluded.latency_ip",
            "speed_bps = excluded.speed_bps",
            "error = excluded.error",
            "error_kind = excluded.error_kind",
            "error_text = excluded.error_text",
        ]);
    }
    if has_purge {
        sets.push("purge_reason = excluded.purge_reason");
    }
    if has_traffic {
        sets.extend_from_slice(&[
            "traffic_today_up = excluded.traffic_today_up",
            "traffic_today_down = excluded.traffic_today_down",
            "traffic_total_up = excluded.traffic_total_up",
            "traffic_total_down = excluded.traffic_total_down",
        ]);
    }
    if sets.is_empty() {
        return format!(" ON CONFLICT({LINK_COMPARE_KEYS}) DO NOTHING");
    }
    sets.push("updated_at = excluded.updated_at");
    // `version` is the row's own optimistic-concurrency counter, so it only
    // ever moves forward — the values tuple's `1` applies to a fresh row.
    sets.push("version = version + 1");
    format!(
        " ON CONFLICT({LINK_COMPARE_KEYS}) DO UPDATE SET {}",
        sets.join(", ")
    )
}

/// The `ON CONFLICT` action of the import/refresh writer: SOURCE columns only.
///
/// The caller's snapshot comes from a fresh parse, so writing the RESULT columns
/// (`latency*`, `speed_bps`, `error*`) or the TRAFFIC counters on update
/// silently wipes the measurements and counters the ping pipeline and the stats
/// poller own — the whole-row shape this replaced did exactly that
/// (2026-09-15: an import refresh running beside a Fast+Real batch destroyed
/// every fast latency it had just written). `last_used_at` is likewise never
/// touched here (its owner is [`Database::update_last_used`]).
const LINK_SOURCE_CONFLICT_SQL: &str = " ON CONFLICT(protocol_id, endpoint_id) DO UPDATE SET \
     core_type = excluded.core_type, config_type = excluded.config_type, \
     last_seen_at = excluded.last_seen_at, updated_at = excluded.updated_at";

/// One multi-row upsert statement for a set of rows whose conflict action is
/// uniform.
async fn exec_link_upsert(
    tx: &mut impl Executor,
    rows: &[ProfileStats],
    conflict: &str,
    now: i64,
) -> Result<()> {
    for chunk in rows.chunks(LINK_STATEMENT_ROWS) {
        let mut sql = String::with_capacity(chunk.len() * 160 + LINK_UPSERT_PREFIX.len());
        sql.push_str(LINK_UPSERT_PREFIX);
        for (i, link) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push(',');
            }
            sql.push_str(&link_values_sql(link, now));
        }
        sql.push_str(conflict);
        toasty::sql::statement(sql).exec(tx).await?;
    }
    Ok(())
}

/// The `CHECK`-constrained storage text of a [`ProfileErr`] variant. An
/// exhaustive match, so a new variant cannot be forgotten; a wrong string is
/// rejected by the column's CHECK constraint rather than stored.
const fn error_kind_str(kind: crate::models_toasty::ProfileErr) -> &'static str {
    match kind {
        crate::models_toasty::ProfileErr::Real => "real",
        crate::models_toasty::ProfileErr::Fast => "fast",
        crate::models_toasty::ProfileErr::Name => "name",
    }
}

/// The `CHECK`-constrained storage text of a [`PurgeReason`] variant, spelled
/// the way toasty's derive renders it (`snake_case`, verified by the column-shape
/// probe of 2026-09-17). A wrong spelling is refused by the column CHECK, and
/// `apply_link_patches_isolates_column_groups` (tests/integration.rs) executes
/// this statement and proves the group it belongs to.
const fn purge_reason_str(reason: crate::models_toasty::PurgeReason) -> &'static str {
    use crate::models_toasty::PurgeReason;
    match reason {
        PurgeReason::RealityFallback => "reality_fallback",
        PurgeReason::CertificateMismatch => "certificate_mismatch",
        PurgeReason::CertificateExpired => "certificate_expired",
        PurgeReason::NotTls => "not_tls",
        PurgeReason::ConfigInvalid => "config_invalid",
        PurgeReason::TransportRejected => "transport_rejected",
        PurgeReason::OriginUnreachable => "origin_unreachable",
    }
}

/// The `CHECK`-constrained storage text of a [`CoreType`] column value.
const fn core_type_str(core: xray_tui_proto::proto_spec::CoreType) -> &'static str {
    match core {
        xray_tui_proto::proto_spec::CoreType::Xray => "xray",
        xray_tui_proto::proto_spec::CoreType::SingBox => "sing_box",
    }
}

/// The `CHECK`-constrained storage text of a [`ConfigType`] column value.
const fn config_type_str(config: crate::models_toasty::ConfigType) -> &'static str {
    match config {
        crate::models_toasty::ConfigType::ShareUrl => "share_url",
        crate::models_toasty::ConfigType::Form => "form",
    }
}

/// Extract the first INTEGER column of the first row (used for PRAGMA reads).
fn first_i64(rows: &[Value]) -> Option<i64> {
    rows.first().and_then(|v| {
        if let Value::Record(fields) = v {
            fields.first().and_then(|f| match f {
                Value::I64(n) => Some(*n),
                _ => None,
            })
        } else {
            None
        }
    })
}

// ── Read queries (public API) ───────────────────────────────────────────

impl Database {
    /// Single endpoint by id with all links and protocols.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_endpoint(&self, id: EndpointId) -> Result<Option<EndpointRow>> {
        let mut conn = self.conn().await?;
        let endpoint = Endpoint::filter_by_id(id).first().exec(&mut conn).await?;
        let Some(endpoint) = endpoint else {
            return Ok(None);
        };
        let mut rows = self.load_endpoint_rows(vec![endpoint], &mut conn).await?;
        Ok(rows.pop())
    }

    /// Look up endpoint row by link protocol id (p.id not e.id).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_endpoint_by_protocol_id(
        &self,
        protocol_id: ProtocolId,
    ) -> Result<Option<EndpointRow>> {
        let mut conn = self.conn().await?;
        let link = ProfileStats::filter(ProfileStats::fields().protocol_id().eq(protocol_id))
            .first()
            .exec(&mut conn)
            .await?;
        let Some(link) = link else {
            return Ok(None);
        };
        let endpoint = Endpoint::filter_by_id(link.endpoint_id)
            .first()
            .exec(&mut conn)
            .await?;
        let Some(endpoint) = endpoint else {
            return Ok(None);
        };
        let mut rows = self.load_endpoint_rows(vec![endpoint], &mut conn).await?;
        Ok(rows.pop())
    }

    /// One `profile_stats` row by its composite key.
    ///
    /// The scheduler gate's `refresh` for a plain `Database` backend (the
    /// write-behind writer answers from its staged map instead).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn read_link_row(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
    ) -> Result<Option<ProfileStats>> {
        let mut conn = self.conn().await?;
        Ok(
            ProfileStats::filter_by_protocol_id_and_endpoint_id(protocol_id, endpoint_id)
                .first()
                .exec(&mut conn)
                .await?,
        )
    }

    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    /// Groups enabled with a non-empty url whose refresh window has elapsed:
    /// `last_refreshed IS NULL` or `last_refreshed + refresh_interval minutes
    /// < now` (`refresh_interval` defaults to 1440 minutes, matching the old
    /// subscription update interval). Ordered by `sort_order`.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
        let candidates: Vec<Group> = Group::filter(Group::fields().enabled().eq(true))
            .filter(Group::fields().url().is_some())
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        let now = now_epoch();
        let mut due = Vec::new();
        for group in candidates {
            if group.url.as_deref().is_none_or(str::is_empty) {
                continue;
            }
            match group.last_refreshed {
                None => due.push(group),
                Some(last) => {
                    let interval = group.refresh_interval.unwrap_or(1440) * 60;
                    if last.saturating_add(interval) < now {
                        due.push(group);
                    }
                }
            }
        }
        Ok(due)
    }

    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut conn = self.conn().await?;
        let rules: Vec<RoutingRule> = RoutingRule::all()
            .order_by(RoutingRule::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(rules)
    }

    /// The (single) DNS settings row, if any.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut conn = self.conn().await?;
        let settings: Vec<DnsSetting> = DnsSetting::all().exec(&mut conn).await?;
        Ok(settings.into_iter().next())
    }

    /// Assemble the page's rows in the page's order, typed.
    ///
    /// The reference implementation of the page's row shape: the ordering is
    /// the decision-16 law ([`EndpointRow::sort_links_by_test_priority`], the
    /// single source — SQL never re-derives it) and the page order is the
    /// caller's `ids`. [`Self::load_page_projection`] returns the same rows
    /// from one statement; the parity test in `tests/profiles_query.rs` pins
    /// the two together, and this path stays as their oracle.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn load_page_rows(
        &self,
        ids: &[EndpointId],
        include_purged: bool,
    ) -> Result<Vec<EndpointRow>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.conn().await?;
        let endpoints: Vec<Endpoint> =
            Endpoint::filter(toasty::stmt::in_list(Endpoint::fields().id(), ids.to_vec()))
                .exec(&mut conn)
                .await?;
        let mut rows = self.load_endpoint_rows(endpoints, &mut conn).await?;
        drop(conn);

        // The same Active-view filter the projection applies in SQL, so the two
        // page readers stay interchangeable (their parity test pins it).
        // `load_endpoint_rows` itself stays unfiltered: the single-endpoint
        // lookups must still see a purged link — that is how it gets re-proved.
        if !include_purged {
            for row in &mut rows {
                row.links.retain(|l| l.purge_reason.is_none());
                row.select_best_measured_link();
            }
        }

        // Page order, not id order: the caller's sequence IS the display order.
        let mut by_id: HashMap<EndpointId, EndpointRow> =
            rows.into_iter().map(|row| (row.endpoint.id, row)).collect();
        Ok(ids.iter().filter_map(|id| by_id.remove(id)).collect())
    }

    /// Assemble [`EndpointRow`]s for a small endpoint set (the single-endpoint
    /// lookups [`Self::get_endpoint`] / [`Self::get_endpoint_by_protocol_id`]
    /// — the tab loads go through [`Self::load_tab_rows`] instead).
    ///
    /// Loads the page's links with one `endpoint_id IN (...)` statement; the
    /// set is ≤1 endpoint here, so the row-value expansion cost that makes
    /// `in_list` untenable at 10k+ ids (see [`Self::load_tab_rows`]) never
    /// applies. Protocols are joined in memory (bounded by distinct configs).
    async fn load_endpoint_rows(
        &self,
        endpoints: Vec<Endpoint>,
        conn: &mut toasty::Connection,
    ) -> Result<Vec<EndpointRow>> {
        if endpoints.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<EndpointId> = endpoints.iter().map(|e| e.id).collect();
        // Newtype embed paths expose only `.eq()`; build the IN filter through
        // `stmt::in_list` (the field struct implements `IntoExpr<EndpointId>`).
        let links: Vec<ProfileStats> = ProfileStats::filter(toasty::stmt::in_list(
            ProfileStats::fields().endpoint_id(),
            ids.clone(),
        ))
        .exec(conn)
        .await?;
        // Only the protocols THIS page's links reference. The whole-table read
        // (6,111 rows here) decoded every `Protocol` model per page load —
        // ~0.47 s of the ~2.2 s a page-boundary move cost at 7.7k endpoints —
        // and the page only ever looks up the few ids its own links carry.
        let mut protocol_ids: Vec<ProtocolId> = links.iter().map(|l| l.protocol_id).collect();
        protocol_ids.sort_unstable();
        protocol_ids.dedup();
        let protocols: Vec<Protocol> = if protocol_ids.is_empty() {
            Vec::new()
        } else {
            Protocol::filter(toasty::stmt::in_list(Protocol::fields().id(), protocol_ids))
                .exec(conn)
                .await?
        };
        let protocol_by_id: HashMap<ProtocolId, Protocol> =
            protocols.into_iter().map(|p| (p.id, p)).collect();

        let mut by_endpoint: HashMap<EndpointId, Vec<ProfileStats>> = HashMap::new();
        for link in links {
            by_endpoint.entry(link.endpoint_id).or_default().push(link);
        }
        // The address set is a table of its own now: one id-inlined read for
        // the whole set, exactly like the links above.
        let mut resolved = crate::endpoint_ip::load(conn, &ids).await?;

        let mut rows = Vec::with_capacity(endpoints.len());
        for endpoint in endpoints {
            let links = by_endpoint.remove(&endpoint.id).unwrap_or_default();
            let protocols = links
                .iter()
                .filter_map(|l| {
                    protocol_by_id
                        .get(&l.protocol_id)
                        .map(|p| (p.id, p.clone()))
                })
                .collect();
            let resolved_ips = resolved.remove(&endpoint.id).unwrap_or_default();
            let dns_unresolved = crate::endpoint_rank::dns_unresolved_endpoint(
                endpoint.host_type,
                !resolved_ips.is_empty(),
            );
            let mut row = EndpointRow {
                endpoint,
                links,
                protocols,
                resolved_ips,
                selected_protocol: 0,
                expanded: false,
            };
            row.sort_links_by_test_priority(dns_unresolved);
            // Single-row display follows the best measured link (survives
            // restarts via the persisted latency columns); the sub-table
            // keeps test-priority order.
            row.select_best_measured_link();
            rows.push(row);
        }
        // Deterministic page order (the newtype id path cannot be ordered in SQL).
        rows.sort_by_key(|r| r.endpoint.id);
        Ok(rows)
    }
}

// ── Write methods (public API) ─────────────────────────────────────────

impl Database {
    // ── Upserts (idempotent, natural-key dedup) ──────────────────────────

    /// Insert or update one endpoint by id.
    ///
    /// Replaces the endpoint's identity fields (`host`, `host_type`, `port`,
    /// `ports`, `last_source`). The resolution state (the `endpoint_ip`
    /// address set and `endpoints.resolved_at`) and the manual protocol
    /// override are owned by their dedicated writes
    /// ([`Self::update_endpoint_resolution`], [`Self::set_manual_override`])
    /// and are preserved on update — this matches the old subscription path,
    /// which never clobbered an existing endpoint's resolution state
    /// (INSERT OR IGNORE).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_endpoint(&self, e: &Endpoint) -> Result<()> {
        let mut conn = self.conn().await?;
        Endpoint::upsert_by_id(e.id)
            .host(e.host.clone())
            .host_type(e.host_type)
            .port(e.port)
            .ports(e.ports.clone())
            .last_source(e.last_source.clone())
            .on_create(|create| {
                // No `#[auto]` on an integer timestamp: the writer stamps it.
                create.created_at(now_epoch())
            })
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one protocol row by id.
    ///
    /// The `config`, `transport.data`, and `security.data` deferred JSON
    /// columns are read via `.get()`, which panics when unloaded — a default
    /// read path (including the crate's own `load_endpoint_rows`, which never
    /// includes deferred data) yields an unloaded `Protocol`. Callers must
    /// pass a freshly-built struct or one loaded with the deferred data
    /// included; an unloaded struct is rejected with an error instead of
    /// panicking.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_protocol(&self, p: &Protocol) -> Result<()> {
        if p.config.is_unloaded() || p.transport.data.is_unloaded() || p.security.data.is_unloaded()
        {
            return Err(DatabaseError::Generic(
                "upsert_protocol: deferred config not loaded (rebuild the Protocol or load it with config/transport/security data included)"
                    .into(),
            ));
        }
        let mut conn = self.conn().await?;
        Protocol::upsert_by_id(p.id)
            .sig(p.sig)
            .proto_kind(p.proto_kind)
            .transport(p.transport.clone())
            .security(p.security.clone())
            .config(p.config.get().0.clone())
            .on_create(|create| create.created_at(now_epoch()))
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one per-pair link row by its composite key
    /// `(protocol_id, endpoint_id)`.
    ///
    /// Replaces the link's source- and result-state fields (`core_type`,
    /// `config_type`, `last_seen_at`, `latency`, `speed_bps`, `error`,
    /// `traffic`). The activity timestamp (`last_used_at`) is owned by
    /// [`Self::update_last_used`] and is preserved on update.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_link(&self, s: &ProfileStats) -> Result<()> {
        let mut conn = self.conn().await?;
        let endpoint_id = s.endpoint_id;
        ProfileStats::upsert_by_protocol_id_and_endpoint_id(s.protocol_id, s.endpoint_id)
            .core_type(s.core_type)
            .config_type(s.config_type)
            .last_seen_at(s.last_seen_at)
            .latency(s.latency.clone())
            .speed_bps(s.speed_bps)
            .error(s.error.clone())
            .purge_reason(s.purge_reason)
            .traffic(s.traffic)
            .updated_at(now_epoch())
            .on_create(|create| create.created_at(now_epoch()))
            .exec(&mut conn)
            .await?;
        // The stored ordering key is derived state: refresh it for the write
        // that just invalidated it (the page reads the key, not the link).
        crate::endpoint_rank::refresh(&mut conn, &[endpoint_id]).await?;
        Ok(())
    }

    /// Apply one patch per row inside a single transaction.
    ///
    /// Each patch carries only the column *groups* that changed (see
    /// [`LinkGroups`]), and the groups decide the statement's `ON CONFLICT`
    /// action, so the two independent writers of a `profile_stats` row — ping
    /// results and the traffic poller — cannot clobber each other's columns:
    /// the groups make the writers disjoint instead of merely sequential.
    ///
    /// One multi-row upsert per (chunk, group shape) replaces one existence
    /// probe plus one `UPDATE` per row: turso has no
    /// `UPDATE … FROM (VALUES …)` (measured 2026-09-16: parse error), so the
    /// per-row form was one statement per link. Measured on a 512-patch window
    /// over the reference feed: 29.0 ms → 10.6 ms.
    ///
    /// One transaction per batch replaces one commit per row — the write-behind
    /// link writer calls this from its flush task, never from the UI task. A row
    /// deleted mid-batch is re-created by its own patch (an upsert inserts what
    /// it cannot update), which is the contract the callers rely on.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn apply_link_patches(&self, patches: &[LinkPatch]) -> Result<usize> {
        if patches.is_empty() {
            return Ok(0);
        }
        let mut conn = self.conn().await?;
        let now = now_epoch();
        let mut touched: Vec<EndpointId> = Vec::with_capacity(patches.len());

        let mut tx = conn.transaction().await?;
        // The `ON CONFLICT` action is per-STATEMENT, so the patches are
        // bucketed by the action they need (both groups / RESULT / TRAFFIC /
        // none) rather than by their exact bit pattern — `contains` is what
        // decides the columns written, exactly as the per-row form did.
        for (has_result, has_purge, has_traffic) in [
            (true, true, true),
            (true, true, false),
            (true, false, true),
            (true, false, false),
            (false, true, true),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            let shape: Vec<&LinkPatch> = patches
                .iter()
                .filter(|p| {
                    p.groups.contains(LinkGroups::RESULT) == has_result
                        && p.groups.contains(LinkGroups::PURGE) == has_purge
                        && p.groups.contains(LinkGroups::TRAFFIC) == has_traffic
                })
                .collect();
            if shape.is_empty() {
                continue;
            }
            let conflict = link_patch_conflict_sql(has_result, has_traffic, has_purge);
            for chunk in shape.chunks(LINK_STATEMENT_ROWS) {
                let mut sql = String::with_capacity(chunk.len() * 160 + LINK_UPSERT_PREFIX.len());
                sql.push_str(LINK_UPSERT_PREFIX);
                for (i, patch) in chunk.iter().enumerate() {
                    if i > 0 {
                        sql.push(',');
                    }
                    sql.push_str(&link_values_sql(&patch.link, now));
                }
                sql.push_str(&conflict);
                toasty::sql::statement(sql).exec(&mut tx).await?;
            }
            touched.extend(shape.iter().map(|p| p.link.endpoint_id));
        }
        tx.commit().await?;

        // Derived state: the patched endpoints' ordering keys follow their
        // links. Done after the commit (the page is read later, never here).
        touched.sort_unstable();
        touched.dedup();
        if let Err(e) = crate::endpoint_rank::refresh(&mut conn, &touched).await {
            tracing::warn!(target: "xray_tui_db", "endpoint_rank refresh: {e}");
        }
        Ok(patches.len())
    }

    /// Insert or update one endpoint↔group link by its composite key
    /// `(endpoint_id, group_id)`.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_endpoint_group_link(&self, eg: &EndpointGroup) -> Result<()> {
        let mut conn = self.conn().await?;
        EndpointGroup::upsert_by_endpoint_id_and_group_id(eg.endpoint_id, eg.group_id.clone())
            .last_seen_at(eg.last_seen_at)
            .sort_order(eg.sort_order)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one group by id (replaces `insert_group` +
    /// `update_group`).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.conn().await?;
        Group::upsert_by_id(g.id.clone())
            .name(g.name.clone())
            .url(g.url.clone())
            .enabled(g.enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type)
            .sort_order(g.sort_order)
            .last_refreshed(g.last_refreshed)
            .status(g.status)
            .error_message(g.error_message.clone())
            .refresh_interval(g.refresh_interval)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    // ── Activity ─────────────────────────────────────────────────────────

    /// Record active use of a link: sets `last_used_at` AND `last_seen_at`
    /// to `ts`, so active use keeps a profile out of Stale/purge (old
    /// `update_last_used` semantics).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn update_last_used(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
        at: i64,
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileStats::filter_by_protocol_id_and_endpoint_id(protocol_id, endpoint_id)
            .update()
            .last_used_at(Some(at))
            .last_seen_at(at)
            .exec(&mut conn)
            .await?;
        // `last_seen_at` is the order's recency tiebreak and the view windows'
        // input, so a "last used" stamp moves the stored key.
        crate::endpoint_rank::refresh(&mut conn, &[endpoint_id]).await?;
        Ok(())
    }

    /// Persist the DNS resolution of an endpoint host: its address set
    /// (`endpoint_ip`) and `resolved_at`. Survives launches so the TUI does not
    /// re-resolve DNS hosts on startup. Retried on write contention (the
    /// enrichment pipeline resolves many endpoints concurrently), as the old
    /// code did.
    ///
    /// The address set and the endpoint's rank keys move together: "has an
    /// address" IS part of the ordering law (decision 16 tier 5), so an
    /// endpoint leaves — or enters — the DNS-unresolved band here.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn update_endpoint_resolution(
        &self,
        endpoint_id: EndpointId,
        ips: Vec<std::net::IpAddr>,
        at: i64,
    ) -> Result<()> {
        let db = self;
        retry_on_busy(
            move || {
                let ips = ips.clone();
                async move {
                    let mut conn = db.conn().await?;
                    let mut tx = conn.transaction().await?;
                    Endpoint::filter_by_id(endpoint_id)
                        .update()
                        .resolved_at(Some(at))
                        .exec(&mut tx)
                        .await?;
                    crate::endpoint_ip::replace(&mut tx, endpoint_id, &ips).await?;
                    crate::endpoint_rank::refresh(&mut tx, &[endpoint_id]).await?;
                    tx.commit().await?;
                    Ok(())
                }
            },
            5,
        )
        .await
    }

    /// The persisted resolved addresses of `ids` with their stored countries.
    ///
    /// The enrichment seed's read: an address that already carries a country
    /// needs no mmdb walk.
    ///
    /// # Errors
    ///
    /// [`DatabaseError`] when the read fails.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn endpoint_resolutions(
        &self,
        ids: &[EndpointId],
    ) -> Result<HashMap<EndpointId, Vec<(std::net::IpAddr, Option<String>)>>> {
        let mut conn = self.conn().await?;
        crate::endpoint_ip::load_resolved(&mut conn, ids).await
    }

    /// Record the country of one resolved address (the mmdb lookup's write).
    ///
    /// # Errors
    ///
    /// [`DatabaseError`] when the write fails.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn set_endpoint_ip_country(
        &self,
        endpoint_id: EndpointId,
        ip: std::net::IpAddr,
        iso: &str,
    ) -> Result<()> {
        let db = self;
        let iso = iso.to_string();
        retry_on_busy(
            move || {
                let iso = iso.clone();
                async move {
                    let mut conn = db.conn().await?;
                    let mut tx = conn.transaction().await?;
                    crate::endpoint_ip::set_country(&mut tx, endpoint_id, ip, &iso).await?;
                    tx.commit().await?;
                    Ok(())
                }
            },
            5,
        )
        .await
    }

    /// Persist resolved-address countries for a whole page in ONE transaction.
    ///
    /// `set_endpoint_ip_country` takes its own connection and commit per call
    /// (~4.2 ms per FULL commit, `docs/database.md`), and the import's 500-URL
    /// chunk commits contend with it (`database is locked` → `retry_on_busy`).
    /// The page seed writes a page at a time instead.
    ///
    /// # Errors
    ///
    /// [`DatabaseError`] when the write fails.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn set_endpoint_ip_countries(
        &self,
        rows: &[(EndpointId, std::net::IpAddr, String)],
    ) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let db = self;
        retry_on_busy(
            move || async move {
                let mut conn = db.conn().await?;
                let mut tx = conn.transaction().await?;
                for (endpoint_id, ip, iso) in rows {
                    crate::endpoint_ip::set_country(&mut tx, *endpoint_id, *ip, iso).await?;
                }
                tx.commit().await?;
                Ok(())
            },
            5,
        )
        .await
    }

    /// Set or clear (`None`) the manual protocol override of an endpoint — the old `set_protocol_override` + `clear_protocol_override` merged.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn set_manual_override(
        &self,
        endpoint_id: EndpointId,
        protocol_id: Option<ProtocolId>,
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        Endpoint::filter_by_id(endpoint_id)
            .update()
            .manual_protocol_override(protocol_id)
            .exec(&mut conn)
            .await?;
        // The override decides the endpoint's DISPLAY link, whose columns the
        // non-Test sorts read.
        crate::endpoint_rank::refresh(&mut conn, &[endpoint_id]).await?;
        Ok(())
    }

    // ── Purge / delete ───────────────────────────────────────────────────

    /// Purge endpoints where EVERY link's `last_seen_at < cutoff` — the old
    /// `COALESCE(MAX(p.last_seen_at), 0) < cutoff` predicate, including
    /// linkless endpoints (`links().all(...)` is vacuously true for them).
    /// Returns the number of deleted endpoints.
    ///
    /// Cascade, in one transaction: the endpoints' `endpoint_groups` links,
    /// their `profile_stats` links, then the endpoints themselves, then
    /// orphan `protocol` rows (those left with zero links).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn purge_expired(&self, cutoff: i64) -> Result<usize> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        // All-links staleness: an endpoint whose EVERY link is older than the
        // cutoff (De-Morgan of the old `.all(last_seen < cutoff)`; vacuously
        // true for a linkless endpoint, so orphans are reclaimed too). This is
        // deliberately NOT the page's live-only `band`: a fresh-but-purged link
        // (a subscription re-listed it) must keep its endpoint alive, which a
        // live-only `rank_newest_seen` would not (ADR 0006 / view-band spec
        // §3.5). Raw because toasty's `.all()` quantifier compiled to a
        // whole-table projection (145 ms, dump-3.log); this is an indexed
        // NOT EXISTS on `profile_stats.last_seen_at`.
        let rows = toasty::sql::query(format!(
            "SELECT e.id FROM endpoints e WHERE NOT EXISTS \
             (SELECT 1 FROM profile_stats p WHERE p.endpoint_id = e.id \
             AND p.last_seen_at >= {cutoff})"
        ))
        .exec(&mut tx)
        .await?;
        let ids: Vec<EndpointId> = rows
            .iter()
            .filter_map(|row| match row {
                toasty_core::stmt::Value::Record(r) => r.fields.first().cloned(),
                _ => None,
            })
            .filter_map(|v| match v {
                toasty_core::stmt::Value::I64(id) => Some(EndpointId::new(id)),
                _ => None,
            })
            .collect();
        let count = ids.len();

        if count > 0 {
            EndpointGroup::filter(toasty::stmt::in_list(
                EndpointGroup::fields().endpoint_id(),
                ids.clone(),
            ))
            .delete()
            .exec(&mut tx)
            .await?;
            ProfileStats::filter(toasty::stmt::in_list(
                ProfileStats::fields().endpoint_id(),
                ids.clone(),
            ))
            .delete()
            .exec(&mut tx)
            .await?;
            Self::purge_orphan_protocols(&mut tx).await?;
            crate::endpoint_ip::delete_for(&mut tx, &ids).await?;
            Endpoint::filter(toasty::stmt::in_list(Endpoint::fields().id(), ids.clone()))
                .delete()
                .exec(&mut tx)
                .await?;
            // The page drives from `endpoint_rank`: drop the keys of the rows
            // whose links just went away, or the tab would list them again.
            crate::endpoint_rank::prune(&mut tx, &ids).await?;
        }

        tx.commit().await?;
        Ok(count)
    }

    /// Delete an endpoint and cascade: its `profile_stats` links, its
    /// `endpoint_groups` links, the endpoint row, then orphan `protocol`
    /// rows (those whose last link just died). One transaction.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn delete_endpoint(&self, endpoint_id: EndpointId) -> Result<()> {
        self.delete_endpoints(&[endpoint_id]).await.map(|_| ())
    }

    /// Delete several endpoints and cascade, in ONE transaction.
    ///
    /// The per-endpoint path ([`Self::delete_endpoint`]) opens a transaction,
    /// scans for orphan protocols and prunes rank keys per endpoint; a bulk
    /// delete ("remove failed servers" over a whole page) did all of that N
    /// times. Returns the number of endpoints deleted.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn delete_endpoints(&self, endpoint_ids: &[EndpointId]) -> Result<usize> {
        if endpoint_ids.is_empty() {
            return Ok(0);
        }
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        EndpointGroup::filter(toasty::stmt::in_list(
            EndpointGroup::fields().endpoint_id(),
            endpoint_ids.to_vec(),
        ))
        .delete()
        .exec(&mut tx)
        .await?;
        ProfileStats::filter(toasty::stmt::in_list(
            ProfileStats::fields().endpoint_id(),
            endpoint_ids.to_vec(),
        ))
        .delete()
        .exec(&mut tx)
        .await?;
        Self::purge_orphan_protocols(&mut tx).await?;
        // The address set is a child table, and nothing cascades on this
        // engine (toasty's model emits no `REFERENCES`), so the deletion
        // owners remove it here — in the same transaction as the endpoint.
        crate::endpoint_ip::delete_for(&mut tx, endpoint_ids).await?;
        Endpoint::filter(toasty::stmt::in_list(
            Endpoint::fields().id(),
            endpoint_ids.to_vec(),
        ))
        .delete()
        .exec(&mut tx)
        .await?;
        // The page drives from `endpoint_rank`: a deleted endpoint must not
        // leave its key behind.
        crate::endpoint_rank::prune(&mut tx, endpoint_ids).await?;

        tx.commit().await?;
        Ok(endpoint_ids.len())
    }

    /// Remove all endpoint↔group links for `group_id` (the old `clear_group`),
    /// returning the number of links removed. Endpoints and their links stay.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn clear_group_endpoints(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.conn().await?;
        let rows: Vec<EndpointGroup> =
            EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
                .exec(&mut conn)
                .await?;
        let count = rows.len();
        EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
            .delete()
            .exec(&mut conn)
            .await?;
        Ok(count)
    }

    /// Delete a group: its `endpoint_groups` links, then the group row.
    /// One transaction.
    ///
    /// Note: the old `delete_group` also purged endpoints of this group that
    /// no longer belonged to ANY group. The typed model keeps group-less
    /// endpoints (the All view shows them, and `clear_group_endpoints`
    /// unlinks without deleting) — endpoint cleanup is left to
    /// [`Self::purge_expired`] by staleness, so deleting a group never
    /// silently destroys endpoints.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn delete_group(&self, group_id: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
            .delete()
            .exec(&mut tx)
            .await?;
        Group::filter_by_id(group_id.to_string())
            .delete()
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // ── Stats / restore ─────────────────────────────────────────────────

    /// Zero traffic (`today_up/down`, `total_up/down`) and clear
    /// `latency`, `speed_bps`, and `error` on EVERY `profile_stats` row
    /// (query-based update, one statement; the old `clear_all_stats` wiped
    /// server stats + extensions' delay/speed).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn clear_all_stats(&self) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileStats::all()
            .update()
            .traffic(TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            })
            .latency(None)
            .error(None)
            .speed_bps(None)
            .exec(&mut conn)
            .await?;
        // Every column this clears feeds a stored key: a wholesale reset must
        // rebuild them all, not leave the old bands in place.
        crate::endpoint_rank::backfill_all(&mut conn).await?;
        Ok(())
    }

    /// Restore a stale endpoint by setting `last_seen_at = now` on all its
    /// links (old `restore_endpoint`).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn restore_endpoint(&self, endpoint_id: EndpointId) -> Result<()> {
        let now = now_epoch();
        let mut conn = self.conn().await?;
        ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(endpoint_id))
            .update()
            .last_seen_at(now)
            .exec(&mut conn)
            .await?;
        crate::endpoint_rank::refresh(&mut conn, &[endpoint_id]).await?;
        Ok(())
    }
}

impl Database {
    /// Delete `Protocol` rows with no remaining `profile_stats` links — the
    /// shared protocol table's orphan cleanup (the old per-row model had no
    /// shared table to clean). Runs inside the caller's transaction.
    ///
    /// `links().all(FALSE)` is vacuously true exactly when the links
    /// collection is empty, lowering to `id NOT IN (SELECT protocol_id FROM
    /// profile_stats)`. Returns the number of protocols deleted.
    async fn purge_orphan_protocols(tx: &mut dyn toasty::Executor) -> Result<usize> {
        let never: toasty::stmt::Expr<bool> =
            toasty::stmt::Expr::from_untyped(toasty_core::stmt::Expr::FALSE);
        let orphans: Vec<Protocol> = Protocol::filter(Protocol::fields().links().all(never))
            .exec(tx)
            .await?;
        let count = orphans.len();
        for protocol in orphans {
            protocol.delete().exec(tx).await?;
        }
        Ok(count)
    }
}
// ── Route probes (singleton settings row) ───────────────────────────────

impl Database {
    /// The global probe-hostname list; an absent row yields an empty vec.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn get_route_probes(&self) -> Result<Vec<String>> {
        let mut conn = self.conn().await?;
        let rows: Vec<RouteProbes> = RouteProbes::all().exec(&mut conn).await?;
        Ok(rows.into_iter().next().map(|r| r.hosts).unwrap_or_default())
    }

    /// Replace the global probe-hostname list (singleton `id == "global"`).
    /// Deduped case-insensitively, first spelling kept — same policy as
    /// merge's probe union.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn upsert_route_probes(&self, hosts: Vec<String>) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        let deduped: Vec<String> = hosts
            .into_iter()
            .filter(|h| seen.insert(h.to_lowercase()))
            .collect();
        let mut conn = self.conn().await?;
        RouteProbes::upsert_by_id("global".to_string())
            .hosts(deduped)
            .exec(&mut conn)
            .await?;
        Ok(())
    }
}
// ── Bulk upserts (subscription import) ───────────────────────────────
//
// Executor-taking free functions rather than `Database` methods: the caller
// owns the transaction, so a whole batch (endpoints + protocols + links +
// group links) commits as ONE transaction — one fsync per batch instead of
// four, and a crash mid-batch never leaves half a batch stored. Field lists
// are copied verbatim from the single-row methods above — the bulk path must
// never drift from them.

/// Insert-or-update many endpoints on the caller's executor (usually a
/// `&mut Transaction`). Empty slice is a no-op.
#[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
pub async fn upsert_endpoints_bulk(tx: &mut impl Executor, eps: &[Endpoint]) -> Result<()> {
    for e in eps {
        Endpoint::upsert_by_id(e.id)
            .host(e.host.clone())
            .host_type(e.host_type)
            .port(e.port)
            .ports(e.ports.clone())
            .last_source(e.last_source.clone())
            .on_create(|create| {
                // No `#[auto]` on an integer timestamp: the writer stamps it.
                create.created_at(now_epoch())
            })
            .exec(tx)
            .await?;
    }
    Ok(())
}

/// Insert-or-update many protocols on the caller's executor. Empty slice is a
/// no-op. Same deferred-unloaded guard as `Database::upsert_protocol`.
#[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
pub async fn upsert_protocols_bulk(tx: &mut impl Executor, ps: &[Protocol]) -> Result<()> {
    for p in ps {
        if p.config.is_unloaded() || p.transport.data.is_unloaded() || p.security.data.is_unloaded()
        {
            return Err(DatabaseError::Generic(
                "upsert_protocols_bulk: deferred config not loaded (rebuild the Protocol or load it with config/transport/security data included)"
                    .into(),
            ));
        }
    }
    for p in ps {
        Protocol::upsert_by_id(p.id)
            .sig(p.sig)
            .proto_kind(p.proto_kind)
            .transport(p.transport.clone())
            .security(p.security.clone())
            .config(p.config.get().0.clone())
            .on_create(|create| create.created_at(now_epoch()))
            .exec(tx)
            .await?;
    }
    Ok(())
}

/// Insert-or-update many per-pair link rows on the caller's executor — the
/// subscription/import path, and the one writer restricted to the SOURCE
/// columns.
///
/// Empty slice is a no-op. An **update** writes identity/config provenance only
/// (`core_type`, `config_type`, `last_seen_at`, `updated_at`); the RESULT
/// columns (`latency*`, `speed_bps`, `error*`) and the TRAFFIC counters are
/// written **on create only** (they ride the `VALUES` tuple, which a conflicting
/// row's `DO UPDATE` never reads). The caller's snapshot comes from a fresh
/// parse, so writing those columns on update silently wipes the measurements and
/// counters the ping pipeline and the stats poller own — the whole-row shape
/// this replaced did exactly that (2026-09-15: an import refresh running beside
/// a Fast+Real batch destroyed every fast latency it had just written).
/// `last_used_at` is likewise never touched here (its owner is
/// [`Database::update_last_used`]).
#[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
pub async fn upsert_links_bulk(tx: &mut impl Executor, links: &[ProfileStats]) -> Result<()> {
    // One multi-row upsert per chunk instead of one typed upsert per row
    // (measured 2026-09-16 over the reference feed: 2,000 links 360 ms → 48 ms).
    exec_link_upsert(tx, links, LINK_SOURCE_CONFLICT_SQL, now_epoch()).await?;
    // Derived state: the stored ordering keys follow the links, so every
    // link writer refreshes the endpoints it touched — here, inside the
    // caller's transaction, which makes the key update atomic with the write.
    let mut touched: Vec<EndpointId> = links.iter().map(|l| l.endpoint_id).collect();
    touched.sort_unstable_by_key(|id| id.get());
    touched.dedup();
    if touched.is_empty() {
        return Ok(());
    }
    crate::endpoint_rank::refresh(tx, &touched).await?;
    Ok(())
}

/// Insert-or-update many endpoint↔group links on the caller's executor.
/// Empty slice is a no-op.
#[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
pub async fn upsert_endpoint_group_links_bulk(
    tx: &mut impl Executor,
    egs: &[EndpointGroup],
) -> Result<()> {
    for eg in egs {
        EndpointGroup::upsert_by_endpoint_id_and_group_id(eg.endpoint_id, eg.group_id.clone())
            .last_seen_at(eg.last_seen_at)
            .sort_order(eg.sort_order)
            .exec(tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models_toasty::HostType;
    use crate::models_toasty::{ConfigType, Latency, Security, TrafficStats, Transport};
    use toasty::{Deferred, Json};
    use xray_tui_proto::proto_spec::common::TransportConfig;
    use xray_tui_proto::proto_spec::{
        CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
        VlessConfig,
    };

    fn ts(secs: i64) -> i64 {
        secs
    }

    fn tcp_transport() -> Transport {
        Transport {
            r#type: TransportType::Tcp,
            data: Deferred::from(Json(TransportConfig::Tcp)),
        }
    }

    fn no_security() -> Security {
        Security {
            r#type: SecurityType::None,
            sni: None,
            fp: None,
            insecure: None,
            data: Deferred::from(Json(SecurityConfig::default())),
        }
    }

    fn vless_config() -> ProtocolConfig {
        ProtocolConfig::Vless(VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            uuid_origin: None,
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        })
    }

    fn zero_traffic() -> TrafficStats {
        TrafficStats {
            today_up: 0,
            today_down: 0,
            total_up: 0,
            total_down: 0,
        }
    }

    /// Insert one endpoint with one protocol and one link at `last_seen`.
    use crate::models_toasty::PurgatoryView;

    /// A one-shot page request for the view predicates.
    fn req(view: PurgatoryView, active: i64) -> crate::profiles_query::PageRequest {
        crate::profiles_query::PageRequest {
            view,
            active_threshold: active,
            scope: crate::profiles_query::PlanScope::All,
            search: None,
            group_id: None,
            sort: crate::profiles_query::PageSort::Test,
            ascending: true,
            offset: 0,
            limit: 10_000,
        }
    }

    /// Endpoint ids in page order.
    async fn ids(db: &Database, req: &crate::profiles_query::PageRequest) -> Vec<i64> {
        db.profiles_page(req)
            .await
            .expect("page")
            .ids
            .iter()
            .map(|id| id.get())
            .collect()
    }

    /// The page's assembled rows.
    async fn rows_of(
        db: &Database,
        req: &crate::profiles_query::PageRequest,
    ) -> Vec<crate::models_toasty::EndpointRow> {
        let meta = db.profiles_page(req).await.expect("page");
        db.load_page_rows(&meta.ids, true).await.expect("rows")
    }

    async fn seed_endpoint(
        conn: &mut toasty::Connection,
        endpoint_id: i64,
        protocol_id: i64,
        host: &str,
        host_type: HostType,
        port: u16,
        last_seen: i64,
    ) {
        toasty::create!(Endpoint {
            created_at: 0,
            id: EndpointId::new(endpoint_id),
            host: host.to_string(),
            host_type,
            port,
            ports: Vec::<u16>::new(),
        })
        .exec(conn)
        .await
        .expect("create endpoint");
        seed_link(conn, endpoint_id, protocol_id, last_seen).await;
    }

    /// Insert one additional protocol + link for an existing endpoint.
    async fn seed_link(
        conn: &mut toasty::Connection,
        endpoint_id: i64,
        protocol_id: i64,
        last_seen: i64,
    ) {
        toasty::create!(Protocol {
            created_at: 0,
            id: ProtocolId::new(protocol_id),
            sig: protocol_id,
            proto_kind: ProtocolKind::Vless,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(vless_config())),
        })
        .exec(conn)
        .await
        .expect("create protocol");

        toasty::create!(ProfileStats {
            created_at: 0,
            updated_at: 0,
            protocol_id: ProtocolId::new(protocol_id),
            endpoint_id: EndpointId::new(endpoint_id),
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_seen_at: ts(last_seen),
            traffic: zero_traffic(),
        })
        .exec(conn)
        .await
        .expect("create link");
    }

    #[tokio::test]
    async fn active_and_stale_windows() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        // Band membership is materialized against `now − ttl` (default 7d) at
        // refresh time, so the fixture uses real-clock-relative timestamps: one
        // fresh endpoint and one older than the ttl.
        let now = crate::models_toasty::now_epoch();
        seed_endpoint(&mut conn, 1, 1001, "5.6.7.8", HostType::Ipv4, 443, now).await;
        seed_endpoint(
            &mut conn,
            2,
            2001,
            "9.10.11.12",
            HostType::Ipv4,
            80,
            now - 8 * 86_400,
        )
        .await;

        // The fixture seeded with raw writes: make the stored keys follow.
        db.repair_endpoint_ranks().await.expect("ranks");

        // Active: only the fresh endpoint (the `active` arg is now advisory —
        // membership is the stored `band`).
        let active = req(PurgatoryView::Active, now - 7 * 86_400);
        assert_eq!(ids(&db, &active).await, vec![1]);

        // Stale: only the older-than-ttl endpoint.
        let stale = req(PurgatoryView::Purgatory, now - 7 * 86_400);
        assert_eq!(ids(&db, &stale).await, vec![2]);

        // Count matches the stale view.
        assert_eq!(db.profiles_page(&stale).await.expect("count").total, 1);
    }

    #[tokio::test]
    async fn rows_are_assembled_with_links_and_protocols() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        // The fixture seeded with raw writes: make the stored keys follow.
        db.repair_endpoint_ranks().await.expect("ranks");

        let rows = rows_of(&db, &req(PurgatoryView::All, ts(0))).await;
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.endpoint.id, EndpointId::new(1));
        assert_eq!(row.links.len(), 2, "both links present");
        assert_eq!(row.protocols.len(), 2, "protocols map built from links");
        assert!(
            row.protocols.contains_key(&ProtocolId::new(1001)),
            "protocol 1001 included"
        );
        assert!(
            row.protocols.contains_key(&ProtocolId::new(1002)),
            "protocol 1002 included"
        );
        // Newest link first (untested tier, recency order).
        assert_eq!(row.links[0].protocol_id, ProtocolId::new(1002));
        assert_eq!(row.links[1].protocol_id, ProtocolId::new(1001));

        // active_protocol resolves through the map.
        let (link, proto) = row.active_protocol().expect("active protocol");
        assert_eq!(link.protocol_id, ProtocolId::new(1002));
        assert_eq!(proto.proto_kind, ProtocolKind::Vless);
    }

    #[tokio::test]
    async fn group_filter_and_links() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "192.168.1.1", HostType::Ipv4, 8080, 100).await;

        // Link endpoint 1 to two groups.
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "source-a".to_string(),
            last_seen_at: ts(100),
        })
        .exec(&mut conn)
        .await
        .expect("link group a");
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "source-b".to_string(),
            last_seen_at: ts(100),
        })
        .exec(&mut conn)
        .await
        .expect("link group b");

        let group = |id: &str| crate::profiles_query::PageRequest {
            group_id: Some(id.to_string()),
            ..req(PurgatoryView::All, ts(0))
        };
        // The fixture seeded with raw writes: make the stored keys follow.
        db.repair_endpoint_ranks().await.expect("ranks");
        assert_eq!(ids(&db, &group("source-a")).await, vec![1]);
        assert_eq!(ids(&db, &group("source-b")).await, vec![1]);
        assert!(
            ids(&db, &group("source-c")).await.is_empty(),
            "unlinked group matches nothing"
        );
    }

    #[tokio::test]
    async fn get_endpoint_by_id_and_protocol() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 7, 3001, "10.0.0.1", HostType::Ipv4, 53, 100).await;

        let row = db.get_endpoint(EndpointId::new(7)).await.expect("get");
        assert_eq!(row.as_ref().expect("row").endpoint.host, "10.0.0.1");
        assert_eq!(row.unwrap().links.len(), 1);

        let by_proto = db
            .get_endpoint_by_protocol_id(ProtocolId::new(3001))
            .await
            .expect("by protocol");
        assert_eq!(by_proto.expect("row").endpoint.id, EndpointId::new(7));

        assert!(
            db.get_endpoint(EndpointId::new(999))
                .await
                .expect("missing")
                .is_none()
        );
        assert!(
            db.get_endpoint_by_protocol_id(ProtocolId::new(9999))
                .await
                .expect("missing")
                .is_none()
        );
    }

    #[tokio::test]
    async fn dns_unresolved_sinks_links_to_bottom() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");

        toasty::create!(Endpoint {
            created_at: 0,
            id: EndpointId::new(1),
            host: "unresolved.example".to_string(),
            host_type: HostType::Dns,
            port: 443,
            ports: Vec::<u16>::new(),
            // no address rows -> unresolved (the child table owns that fact)
        })
        .exec(&mut conn)
        .await
        .expect("endpoint");

        for (pid, last_seen) in [(1001, 1), (1002, 2)] {
            toasty::create!(Protocol {
                created_at: 0,
                id: ProtocolId::new(pid),
                sig: pid,
                proto_kind: ProtocolKind::Vless,
                transport: tcp_transport(),
                security: no_security(),
                config: Deferred::from(Json(vless_config())),
            })
            .exec(&mut conn)
            .await
            .expect("protocol");
            toasty::create!(ProfileStats {
                created_at: 0,
                updated_at: 0,
                protocol_id: ProtocolId::new(pid),
                endpoint_id: EndpointId::new(1),
                core_type: CoreType::Xray,
                config_type: ConfigType::ShareUrl,
                last_seen_at: ts(last_seen),
                latency: Some(Latency::Real {
                    delay: 10,
                    ip: None
                }),
                traffic: zero_traffic(),
            })
            .exec(&mut conn)
            .await
            .expect("link");
        }

        // The fixture seeded with raw writes: make the stored keys follow.
        db.repair_endpoint_ranks().await.expect("ranks");

        let rows = rows_of(&db, &req(PurgatoryView::All, ts(0))).await;
        let row = &rows[0];
        // Both links sink to tier 5; recency decides.
        assert_eq!(row.links[0].protocol_id, ProtocolId::new(1002));
        assert_eq!(row.links[1].protocol_id, ProtocolId::new(1001));
        assert_eq!(row.best_test_priority_key(true).unwrap().0, 5);
    }

    #[tokio::test]
    async fn open_reopen_preserves_data() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("reopen.db");

        let db = Database::open(&path).await.expect("open");
        let mut conn = db.connection().await.expect("connection");
        toasty::create!(Endpoint {
            created_at: 0,
            id: EndpointId::new(77),
            host: "1.1.1.1".to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::<u16>::new(),
        })
        .exec(&mut conn)
        .await
        .expect("create endpoint");
        drop(conn);
        drop(db);

        // Reopen must NOT wipe: the schema tag skips push_schema.
        let db2 = Database::open(&path).await.expect("reopen");
        let mut conn = db2.connection().await.expect("connection");
        let endpoint = Endpoint::filter_by_id(EndpointId::new(77))
            .first()
            .exec(&mut conn)
            .await
            .expect("read");
        assert_eq!(endpoint.expect("endpoint").host, "1.1.1.1");
    }

    #[tokio::test]
    async fn open_recreates_incompatible_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("pre8.db");

        // Build a database with a pre-T8 `endpoints` table (old 9-table
        // shape) — push_schema cannot run on it.
        {
            let driver = toasty_driver_turso::Turso::file(&path);
            let db = toasty::Db::builder()
                .models(toasty::models!(ScratchOnly))
                .build(driver)
                .await
                .expect("build db");
            db.push_schema().await.expect("push schema");
            let mut conn = db.connection().await.expect("connection");
            toasty::sql::statement("CREATE TABLE endpoints (id INTEGER PRIMARY KEY, host TEXT)")
                .exec(&mut conn)
                .await
                .expect("old endpoints table");
        }

        let db = Database::open(&path).await.expect("open recreates schema");
        let mut conn = db.connection().await.expect("connection");
        let rows = toasty::sql::query(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'profile_stats'",
        )
        .exec(&mut conn)
        .await
        .expect("check new table");
        assert_eq!(
            first_i64(&rows).expect("count"),
            1,
            "open() must recreate with the 7-table schema"
        );
    }

    #[derive(Debug, toasty::Model)]
    struct ScratchOnly {
        #[key]
        id: i64,
    }

    #[tokio::test]
    async fn groups_due_update_respects_refresh_interval() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        let now = jiff::Timestamp::now();
        let hour_ago = now
            .checked_sub(jiff::Span::new().hours(1))
            .expect("subtract");

        // Due: never refreshed.
        toasty::create!(Group {
            id: "g-never".to_string(),
            name: Some("never".to_string()),
            url: Some("https://example.com/sub".to_string()),
            enabled: true,
        })
        .exec(&mut conn)
        .await
        .expect("group");

        // Due: refreshed 1h ago with a 30-minute interval (default is 1440).
        toasty::create!(Group {
            id: "g-due".to_string(),
            name: Some("due".to_string()),
            url: Some("https://example.com/sub2".to_string()),
            enabled: true,
            refresh_interval: Some(30),
            last_refreshed: Some(hour_ago.as_second()),
        })
        .exec(&mut conn)
        .await
        .expect("group");

        // Not due: refreshed now.
        toasty::create!(Group {
            id: "g-fresh".to_string(),
            name: Some("fresh".to_string()),
            url: Some("https://example.com/sub3".to_string()),
            enabled: true,
            last_refreshed: Some(now.as_second()),
        })
        .exec(&mut conn)
        .await
        .expect("group");

        // Not due: disabled.
        toasty::create!(Group {
            id: "g-off".to_string(),
            name: Some("off".to_string()),
            url: Some("https://example.com/sub4".to_string()),
            enabled: false,
        })
        .exec(&mut conn)
        .await
        .expect("group");

        // Not due: no url.
        toasty::create!(Group {
            id: "g-nourl".to_string(),
            name: Some("nourl".to_string()),
            url: None,
            enabled: true,
        })
        .exec(&mut conn)
        .await
        .expect("group");

        let due = db.get_groups_due_update().await.expect("due");
        let mut due_ids: Vec<&str> = due.iter().map(|g| g.id.as_str()).collect();
        due_ids.sort_unstable();
        assert_eq!(due_ids, vec!["g-due", "g-never"]);
    }

    #[tokio::test]
    async fn dns_settings_and_routing_rules_roundtrip() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");

        assert!(db.get_dns_settings().await.expect("empty").is_none());

        toasty::create!(DnsSetting {
            id: "dns-1".to_string(),
            name: Some("main".to_string()),
            servers: ["1.1.1.1".to_string()],
            hosts: Vec::<String>::new(),
            disable_cache: true,
            disable_fallback: false,
        })
        .exec(&mut conn)
        .await
        .expect("dns setting");

        let dns = db.get_dns_settings().await.expect("dns").expect("row");
        assert_eq!(dns.servers, vec!["1.1.1.1".to_string()]);
        assert!(dns.disable_cache);

        toasty::create!(RoutingRule {
            id: "rule-1".to_string(),
            r#type: 0,
            domains: ["example.com".to_string()],
            ips: Vec::<String>::new(),
            inbound_tags: Vec::<String>::new(),
            ports: [443],
            source_ports: Vec::<u16>::new(),
            protocols: Vec::<String>::new(),
            sort_order: Some(2),
        })
        .exec(&mut conn)
        .await
        .expect("routing rule");

        let rules = db.get_all_routing_rules().await.expect("rules");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].domains, vec!["example.com".to_string()]);
        assert_eq!(rules[0].ports, vec![443]);
    }

    #[tokio::test]
    async fn route_probes_roundtrip_and_dedupe() {
        let db = Database::in_memory().await.expect("in-memory db");

        // Absent row ⇒ empty vec, not an error.
        assert!(db.get_route_probes().await.expect("absent").is_empty());

        // Create path: case-insensitive dedupe keeps the first spelling.
        db.upsert_route_probes(vec![
            "A.example.com".to_string(),
            "b.Example.com".to_string(),
            "a.EXAMPLE.com".to_string(),
        ])
        .await
        .expect("upsert create");
        assert_eq!(
            db.get_route_probes().await.expect("probes"),
            vec!["A.example.com".to_string(), "b.Example.com".to_string(),],
            "deduped case-insensitively, first spelling kept"
        );

        // Update path: wholesale replacement of the singleton row.
        db.upsert_route_probes(vec!["c.example.com".to_string()])
            .await
            .expect("upsert update");
        assert_eq!(
            db.get_route_probes().await.expect("probes"),
            vec!["c.example.com".to_string()],
        );

        // Singleton: both upserts hit the same row.
        let mut conn = db.connection().await.expect("connection");
        let count = RouteProbes::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn route_probes_survive_reopen_via_schema_tag() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("probes.db");

        let db = Database::open(&path).await.expect("open");
        db.upsert_route_probes(vec!["probe.example.com".to_string()])
            .await
            .expect("upsert");
        drop(db);

        // Reopen must NOT wipe (schema tag skips push_schema).
        let db2 = Database::open(&path).await.expect("reopen");
        assert_eq!(
            db2.get_route_probes().await.expect("probes"),
            vec!["probe.example.com".to_string()],
        );
    }

    /// `EndpointRow.active_link` respects a manual protocol override.
    #[tokio::test]
    async fn manual_override_shapes_active_link() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "10.10.10.10", HostType::Ipv4, 53, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        let row = db.get_endpoint(EndpointId::new(1)).await.expect("row");
        let mut row = row.expect("row");
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1002)
        );

        row.endpoint.manual_protocol_override = Some(ProtocolId::new(1001));
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1001)
        );

        // Override pointing at a missing protocol falls back to selection.
        row.endpoint.manual_protocol_override = Some(ProtocolId::new(999));
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1002)
        );
    }

    // ── Typed writes (Task 10) ───────────────────────────────────────────

    /// An address literal, for the tests' write arguments.
    fn ip(text: &str) -> std::net::IpAddr {
        text.parse().expect("test address")
    }

    /// An endpoint's stored addresses, in the table's key order.
    async fn stored_ips(conn: &mut toasty::Connection, id: i64) -> Vec<std::net::IpAddr> {
        let mut rows = crate::endpoint_ip::load(conn, &[EndpointId::new(id)])
            .await
            .expect("read addresses");
        rows.remove(&EndpointId::new(id)).unwrap_or_default()
    }

    /// A full `Endpoint` struct for the typed write methods.
    fn endpoint_struct(id: i64, host: &str, host_type: HostType, port: u16) -> Endpoint {
        Endpoint {
            id: EndpointId::new(id),
            host: host.to_string(),
            host_type,
            port,
            ports: Vec::new(),
            last_source: None,
            manual_protocol_override: None,
            resolved_at: None,
            created_at: ts(0),
            links: Deferred::default(),
            group_links: Deferred::default(),
        }
    }

    /// A full `ProfileStats` struct for the typed write methods.
    fn link_struct(protocol_id: i64, endpoint_id: i64, last_seen: i64) -> ProfileStats {
        ProfileStats {
            protocol_id: ProtocolId::new(protocol_id),
            endpoint_id: EndpointId::new(endpoint_id),
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_used_at: None,
            last_seen_at: ts(last_seen),
            latency: None,
            speed_bps: None,
            error: None,
            purge_reason: None,
            traffic: zero_traffic(),
            created_at: ts(0),
            updated_at: ts(0),
            version: 1,
            protocol: Deferred::default(),
            endpoint: Deferred::default(),
        }
    }

    /// A full, loaded `Protocol` struct (deferred JSON included) for
    /// `upsert_protocol`.
    fn protocol_struct(id: i64) -> Protocol {
        Protocol {
            id: ProtocolId::new(id),
            sig: id,
            proto_kind: ProtocolKind::Vless,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(vless_config())),
            created_at: ts(0),
            links: Deferred::default(),
        }
    }

    #[tokio::test]
    async fn upsert_endpoint_is_idempotent() {
        let db = Database::in_memory().await.expect("in-memory db");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("upsert");
        db.upsert_endpoint(&endpoint_struct(1, "9.9.9.9", HostType::Ipv4, 8443))
            .await
            .expect("upsert again");

        let mut conn = db.connection().await.expect("connection");
        let count = Endpoint::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 1, "second upsert must not duplicate the row");

        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.host, "9.9.9.9", "identity fields refresh on re-upsert");
        assert_eq!(ep.port, 8443);

        // Owned state (resolution cache, manual override) survives re-upserts.
        db.update_endpoint_resolution(EndpointId::new(1), vec![ip("1.1.1.1")], ts(77))
            .await
            .expect("resolve");
        db.set_manual_override(EndpointId::new(1), Some(ProtocolId::new(99)))
            .await
            .expect("override");
        db.upsert_endpoint(&endpoint_struct(1, "9.9.9.9", HostType::Ipv4, 8443))
            .await
            .expect("upsert after resolution");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(
            stored_ips(&mut conn, 1).await,
            vec![ip("1.1.1.1")],
            "the address set survives a re-upsert of the endpoint row"
        );
        assert_eq!(ep.resolved_at, Some(ts(77)));
        assert_eq!(
            ep.manual_protocol_override,
            Some(ProtocolId::new(99)),
            "manual override preserved"
        );
    }

    #[tokio::test]
    async fn import_refresh_preserves_result_and_traffic() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        let mut probed = link_struct(1001, 1, 100);
        probed.latency = Some(Latency::Fast { delay: 42 });
        probed.error = Some(crate::models_toasty::ErrorInfo {
            kind: crate::models_toasty::ProfileErr::Real,
            text: "timeout on tcp dial".to_string(),
        });
        probed.traffic = TrafficStats {
            today_up: 7,
            today_down: 9,
            total_up: 11,
            total_down: 13,
        };
        db.upsert_link(&probed).await.expect("probed link");

        // A subscription refresh re-persists the link from a fresh parse: its
        // snapshot carries no measurement and zeroed counters, and one SOURCE
        // column (the per-pair core override) genuinely changed.
        let mut fresh = link_struct(1001, 1, 200);
        fresh.core_type = CoreType::SingBox;
        let mut tx = conn.transaction().await.expect("tx");
        upsert_links_bulk(&mut tx, std::slice::from_ref(&fresh))
            .await
            .expect("bulk upsert");
        tx.commit().await.expect("commit");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(
            stored.latency,
            Some(Latency::Fast { delay: 42 }),
            "a measurement survives an import refresh"
        );
        assert!(
            stored.error.is_some(),
            "a failure marker survives an import refresh"
        );
        assert_eq!(
            (
                stored.traffic.today_up,
                stored.traffic.today_down,
                stored.traffic.total_up,
                stored.traffic.total_down
            ),
            (7, 9, 11, 13),
            "traffic counters survive an import refresh"
        );
        assert_eq!(stored.last_seen_at, ts(200), "source column updates");
        assert_eq!(
            stored.core_type,
            CoreType::SingBox,
            "the per-pair core override still updates"
        );
    }

    #[tokio::test]
    async fn import_insert_writes_the_whole_snapshot() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        let mut snapshot = link_struct(1001, 1, 300);
        snapshot.traffic = TrafficStats {
            today_up: 1,
            today_down: 2,
            total_up: 3,
            total_down: 4,
        };
        let mut tx = conn.transaction().await.expect("tx");
        upsert_links_bulk(&mut tx, std::slice::from_ref(&snapshot))
            .await
            .expect("bulk upsert");
        tx.commit().await.expect("commit");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(
            stored.traffic.total_down, 4,
            "a first insert stores the caller's snapshot"
        );
    }

    #[tokio::test]
    async fn upsert_link_is_idempotent_and_updates_fields() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        let mut link = link_struct(1001, 1, 100);
        link.traffic = TrafficStats {
            today_up: 5,
            today_down: 6,
            total_up: 7,
            total_down: 8,
        };
        db.upsert_link(&link).await.expect("upsert link");
        db.upsert_link(&link).await.expect("upsert link again");

        let count = ProfileStats::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 1, "composite-key upsert must not duplicate");

        // Second upsert replaces the result-state fields (e.g. latency).
        link.latency = Some(Latency::Fast { delay: 42 });
        link.last_seen_at = ts(200);
        db.upsert_link(&link)
            .await
            .expect("upsert link with latency");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(stored.latency, Some(Latency::Fast { delay: 42 }));
        assert_eq!(stored.last_seen_at, ts(200));
        assert_eq!(stored.traffic, link.traffic);
    }

    #[tokio::test]
    async fn upsert_link_preserves_activity_state() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        db.upsert_link(&link_struct(1001, 1, 100))
            .await
            .expect("upsert link");
        db.update_last_used(ProtocolId::new(1001), EndpointId::new(1), ts(300))
            .await
            .expect("last used");

        // A re-upsert must not clobber the activity state.
        db.upsert_link(&link_struct(1001, 1, 150))
            .await
            .expect("re-upsert");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(
            stored.last_used_at,
            Some(ts(300)),
            "activity timestamp survives upsert"
        );
        assert_eq!(
            stored.last_seen_at,
            ts(150),
            "link last_seen replaces on upsert"
        );
    }

    #[tokio::test]
    async fn upsert_protocol_rejects_unloaded_config() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        toasty::create!(Protocol {
            created_at: 0,
            id: ProtocolId::new(1001),
            sig: 1001,
            proto_kind: ProtocolKind::Vless,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(vless_config())),
        })
        .exec(&mut conn)
        .await
        .expect("protocol");

        // A default read leaves the deferred JSON unloaded (the crate's own
        // read paths never include it) — upserting must fail with an error
        // instead of panicking on `.get()`.
        let loaded = Protocol::filter_by_id(ProtocolId::new(1001))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert!(loaded.config.is_unloaded(), "config deferred by default");

        let err = db
            .upsert_protocol(&loaded)
            .await
            .expect_err("unloaded config must be rejected, not panic");
        assert!(
            err.to_string().contains("not loaded"),
            "error explains the unloaded state: {err}"
        );

        // A freshly-built struct with loaded config still upserts fine.
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("loaded protocol upserts");
    }

    #[tokio::test]
    async fn update_last_used_refreshes_both_columns() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;

        db.update_last_used(ProtocolId::new(1001), EndpointId::new(1), ts(500))
            .await
            .expect("update");

        let link = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(link.last_used_at, Some(ts(500)));
        assert_eq!(
            link.last_seen_at,
            ts(500),
            "active use keeps the link out of Stale"
        );
    }

    #[tokio::test]
    async fn update_endpoint_resolution_sets_cache() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;

        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec![ip("1.1.1.1"), ip("2.2.2.2")],
            ts(100),
        )
        .await
        .expect("resolve");

        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(
            stored_ips(&mut conn, 1).await,
            vec![ip("1.1.1.1"), ip("2.2.2.2")],
            "key order == address order, not the order the resolver printed"
        );
        assert_eq!(ep.resolved_at, Some(ts(100)));
    }

    #[tokio::test]
    async fn stored_country_survives_re_resolution() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;

        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec![ip("1.1.1.1"), ip("2.2.2.2")],
            ts(100),
        )
        .await
        .expect("resolve");
        db.set_endpoint_ip_country(EndpointId::new(1), ip("1.1.1.1"), "US")
            .await
            .expect("country");

        // A re-resolution keeps the country of an address that is still
        // present — the whole point of storing it — and drops the address
        // that is gone.
        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec![ip("1.1.1.1"), ip("3.3.3.3")],
            ts(200),
        )
        .await
        .expect("re-resolve");

        let resolved = db
            .endpoint_resolutions(&[EndpointId::new(1)])
            .await
            .expect("read");
        assert_eq!(
            resolved
                .get(&EndpointId::new(1))
                .cloned()
                .unwrap_or_default(),
            vec![
                (ip("1.1.1.1"), Some("US".to_string())),
                (ip("3.3.3.3"), None)
            ],
        );
    }

    #[tokio::test]
    async fn country_writes_before_the_address_row_exists() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;

        // The geo step can finish before the resolution event's write lands
        // (they are separate tasks): the country must create the row rather
        // than be dropped.
        db.set_endpoint_ip_country(EndpointId::new(1), ip("9.9.9.9"), "DE")
            .await
            .expect("country first");
        db.update_endpoint_resolution(EndpointId::new(1), vec![ip("9.9.9.9")], ts(100))
            .await
            .expect("resolve second");

        let resolved = db
            .endpoint_resolutions(&[EndpointId::new(1)])
            .await
            .expect("read");
        assert_eq!(
            resolved
                .get(&EndpointId::new(1))
                .cloned()
                .unwrap_or_default(),
            vec![(ip("9.9.9.9"), Some("DE".to_string()))],
        );
    }

    #[tokio::test]
    async fn country_batch_writes_every_row() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;
        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec![ip("1.1.1.1"), ip("2.2.2.2"), ip("3.3.3.3")],
            ts(100),
        )
        .await
        .expect("resolve");

        db.set_endpoint_ip_countries(&[
            (EndpointId::new(1), ip("1.1.1.1"), "US".to_string()),
            (EndpointId::new(1), ip("2.2.2.2"), "DE".to_string()),
            (EndpointId::new(1), ip("3.3.3.3"), "JP".to_string()),
        ])
        .await
        .expect("batch");

        let resolved = db
            .endpoint_resolutions(&[EndpointId::new(1)])
            .await
            .expect("read");
        assert_eq!(
            resolved
                .get(&EndpointId::new(1))
                .cloned()
                .unwrap_or_default(),
            vec![
                (ip("1.1.1.1"), Some("US".to_string())),
                (ip("2.2.2.2"), Some("DE".to_string())),
                (ip("3.3.3.3"), Some("JP".to_string())),
            ],
            "every row of the batch landed, and none overwrote another"
        );
    }

    #[tokio::test]
    async fn country_batch_empty_changes_nothing() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;
        db.set_endpoint_ip_country(EndpointId::new(1), ip("1.1.1.1"), "US")
            .await
            .expect("country");

        db.set_endpoint_ip_countries(&[])
            .await
            .expect("empty batch");

        let resolved = db
            .endpoint_resolutions(&[EndpointId::new(1)])
            .await
            .expect("read");
        assert_eq!(
            resolved
                .get(&EndpointId::new(1))
                .cloned()
                .unwrap_or_default(),
            vec![(ip("1.1.1.1"), Some("US".to_string()))],
        );
    }

    #[tokio::test]
    async fn country_batch_creates_missing_address_rows() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;

        // The geo step can finish before the resolution event's write lands
        // (they are separate tasks): a country in the batch must create its
        // row, and the later resolution write must keep it.
        db.set_endpoint_ip_countries(&[
            (EndpointId::new(1), ip("9.9.9.9"), "DE".to_string()),
            (EndpointId::new(1), ip("1.1.1.1"), "US".to_string()),
        ])
        .await
        .expect("batch first");
        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec![ip("9.9.9.9"), ip("1.1.1.1")],
            ts(100),
        )
        .await
        .expect("resolve second");

        let resolved = db
            .endpoint_resolutions(&[EndpointId::new(1)])
            .await
            .expect("read");
        assert_eq!(
            resolved
                .get(&EndpointId::new(1))
                .cloned()
                .unwrap_or_default(),
            vec![
                (ip("1.1.1.1"), Some("US".to_string())),
                (ip("9.9.9.9"), Some("DE".to_string())),
            ],
        );
    }

    #[tokio::test]
    async fn set_manual_override_sets_and_clears() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "10.10.10.10", HostType::Ipv4, 53, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        db.set_manual_override(EndpointId::new(1), Some(ProtocolId::new(1001)))
            .await
            .expect("set");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.manual_protocol_override, Some(ProtocolId::new(1001)));

        db.set_manual_override(EndpointId::new(1), None)
            .await
            .expect("clear");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.manual_protocol_override, None);
    }

    #[tokio::test]
    async fn restore_endpoint_bumps_last_seen() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        db.restore_endpoint(EndpointId::new(1))
            .await
            .expect("restore");

        let now = crate::models_toasty::now_epoch();
        let links: Vec<ProfileStats> =
            ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(EndpointId::new(1)))
                .exec(&mut conn)
                .await
                .expect("links");
        assert_eq!(links.len(), 2);
        for link in links {
            assert!(
                link.last_seen_at >= now - 60,
                "every link of the endpoint is refreshed"
            );
        }
    }

    #[tokio::test]
    async fn upsert_group_replaces_row() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        let mut g = Group {
            id: "g1".to_string(),
            name: Some("A".to_string()),
            url: Some("https://example.com/sub".to_string()),
            enabled: true,
            user_agent: None,
            convert_target: None,
            core_type: None,
            sort_order: Some(0),
            last_refreshed: None,
            status: None,
            error_message: None,
            refresh_interval: Some(30),
        };
        db.upsert_group(&g).await.expect("insert");

        g.name = Some("B".to_string());
        g.enabled = false;
        db.upsert_group(&g).await.expect("update");

        let rows: Vec<Group> = Group::filter(Group::fields().id().eq("g1".to_string()))
            .exec(&mut conn)
            .await
            .expect("rows");
        assert_eq!(rows.len(), 1, "upsert replaces insert_group/update_group");
        let stored = Group::filter_by_id("g1".to_string())
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(stored.name.as_deref(), Some("B"));
        assert!(!stored.enabled);
    }

    #[tokio::test]
    async fn clear_group_endpoints_and_delete_group() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "g-a".to_string(),
            last_seen_at: ts(1),
        })
        .exec(&mut conn)
        .await
        .expect("link a");
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "g-b".to_string(),
            last_seen_at: ts(1),
        })
        .exec(&mut conn)
        .await
        .expect("link b");

        let n = db.clear_group_endpoints("g-a").await.expect("clear");
        assert_eq!(n, 1, "clear_group returns the link count");
        let remaining: Vec<EndpointGroup> =
            EndpointGroup::all().exec(&mut conn).await.expect("rows");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].group_id, "g-b");

        // delete_group removes the remaining links + the group, keeps endpoints.
        db.upsert_group(&Group {
            id: "g-b".to_string(),
            name: None,
            url: None,
            enabled: true,
            user_agent: None,
            convert_target: None,
            core_type: None,
            sort_order: None,
            last_refreshed: None,
            status: None,
            error_message: None,
            refresh_interval: None,
        })
        .await
        .expect("group b");
        db.delete_group("g-b").await.expect("delete group");

        let count = EndpointGroup::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 0, "group links deleted");
        assert!(
            Group::filter_by_id("g-b".to_string())
                .first()
                .exec(&mut conn)
                .await
                .expect("read")
                .is_none(),
            "group deleted"
        );
        assert!(
            Endpoint::filter_by_id(EndpointId::new(1))
                .first()
                .exec(&mut conn)
                .await
                .expect("read")
                .is_some(),
            "endpoint + links survive group deletion"
        );
    }

    /// Aggregating a group's addresses into ONE blob, and decoding it without
    /// hex parsing. Measured 2026-09-16, and **not** adopted for the page:
    /// `docs/aegis/specs/2026-09-16-db-claim-verification.md` §2 row 6c.
    ///
    /// The idea (this test): `unhex(string_agg(hex(x), ''))` concatenates every
    /// address of a group into a single BLOB, and the decoder slices it by a
    /// fixed width — no comma split, no per-token hex parse, where the shipped
    /// carrier is `group_concat(hex(ip_key), ',')` and decodes token by token.
    ///
    /// What the measurement says (200-endpoint resolved page, 277 addresses,
    /// 7,656-endpoint feed, release build):
    ///
    /// * **The SQL side is not gated** — `string_agg`, `hex`, `unhex` and
    ///   `FILTER` all work with the custom-types flag OFF (the flag this test
    ///   passes is only what makes the `STRICT` DDL below legal; see the note in
    ///   `docs/database-manual-sql.md`).
    /// * **Equal in cost, identical in output**: statement 747–836 µs shipped
    ///   against 758–776 µs as one blob (the spread is run-to-run noise), decode
    ///   4.7 µs against 4.6–7.4 µs, and the decoded address sets are EQUAL
    ///   (277 == 277). Parsing the shipped text form for a whole page costs
    ///   8–14 µs, which is ≤0.4 % of a page.
    /// * **The split variant is wrong, not just slower**: on a `(family, addr)`
    ///   table the same `chunks::<4>()` decode returned **367** addresses where
    ///   the packed key has 277 — a single concatenated blob cannot be sliced by
    ///   one width when IPv4 and IPv6 are mixed, so it silently invents
    ///   addresses. Two `FILTER`ed aggregates would be needed to fix that.
    ///
    /// So the shipped `group_concat(hex(ip_key))` stays: same result, same cost,
    /// one authoritative spelling (the packed key's byte order IS address
    /// order), and no mixed-family trap. If the blob form is ever adopted, the
    /// correct decoder is a WALK over the packed key — the same bytes, family
    /// byte then 4/16 octets — which is byte-identical to the shipped output and
    /// needs no schema change.
    #[tokio::test]
    async fn test_array_agg_result_deserialize_to_ips() {
        use toasty::stmt::Value;

        #[derive(Debug, PartialEq)]
        struct TestIPs {
            epid: i64,
            ipv4: Box<[std::net::Ipv4Addr]>,
            ipv6: Box<[std::net::Ipv6Addr]>,
        }

        fn blob_to_ipv4(full_blob: &[u8]) -> Option<Box<[std::net::Ipv4Addr]>> {
            let (ipv4_blobs, &[]) = full_blob.as_chunks::<4>() else {
                return None;
            };
            Some(
                ipv4_blobs
                    .iter()
                    .copied()
                    .map(std::net::Ipv4Addr::from_octets)
                    .collect(),
            )
        }
        fn blob_to_ipv6(full_blob: &[u8]) -> Option<Box<[std::net::Ipv6Addr]>> {
            let (ipv4_blobs, &[]) = full_blob.as_chunks::<16>() else {
                return None;
            };
            Some(
                ipv4_blobs
                    .iter()
                    .copied()
                    .map(std::net::Ipv6Addr::from_octets)
                    .collect(),
            )
        }

        let db = Database::in_memory().await.expect("should open");
        // let db = toasty::Db::builder()
        //     .build(toasty_driver_turso::Turso::in_memory().experimental_custom_types(true))
        //     .await
        //     .expect("should open");
        let mut conn = db.connection().await.expect("should connect");

        toasty::sql::statement(
            "CREATE TABLE IF NOT EXISTS test_ips (epid INTEGER, ipv4 BLOB, ipv6 BLOB);",
        )
        .exec(&mut conn)
        .await
        .expect("should create");

        toasty::sql::statement("INSERT INTO test_ips VALUES (?, ?, ?), (?, ?, ?);")
            .bind(1_i32)
            .bind((0_u8..4).collect::<Vec<_>>())
            .bind((0_u8..16).collect::<Vec<_>>())
            .bind(1_i32)
            .bind((8_u8..12).collect::<Vec<_>>())
            .bind((16_u8..32).collect::<Vec<_>>())
            .exec(&mut conn)
            .await
            .expect("should insert");

        let mut rows = toasty::sql::query(
            "select epid, unhex(string_agg(hex(ipv4), '')) as ipv4, unhex(string_agg(hex(ipv6), '')) as ipv6 from test_ips group by epid",
        )
        .exec(&mut conn)
        .await
        .expect("should select")
        .into_iter()
        .map(Value::into_record);

        let test_ips = {
            let row = rows.next().expect("at least one row");
            let [Value::I64(epid), Value::Bytes(ipv4), Value::Bytes(ipv6)] = row.as_ref() else {
                panic!("expected three columns (bigint, blob, blob), found: {row:#?}");
            };
            TestIPs {
                epid: *epid,
                ipv4: blob_to_ipv4(ipv4).expect("valid blob"),
                ipv6: blob_to_ipv6(ipv6).expect("valid blob"),
            }
        };
        assert_eq!(
            test_ips,
            TestIPs {
                epid: 1,
                ipv4: Box::from(&[
                    std::net::Ipv4Addr::from_octets([0, 1, 2, 3]),
                    std::net::Ipv4Addr::from_octets([8, 9, 10, 11])
                ] as &[_]),
                ipv6: Box::from(&[
                    std::net::Ipv6Addr::from_octets([
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
                    ]),
                    std::net::Ipv6Addr::from_octets([
                        16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
                    ])
                ] as &[_])
            }
        );
        assert!(rows.next().is_none());
    }
}
