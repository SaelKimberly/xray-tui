//! Materialized per-endpoint ordering keys.
//!
//! The Profiles page used to derive each endpoint's position from a
//! `ROW_NUMBER() OVER (PARTITION BY endpoint_id …)` window over every link
//! plus correlated subqueries for the display-link sorts. On the reference
//! feed (7,672 endpoints / 9,048 links) that cost ~975 ms per page fetch: the
//! window re-evaluates the ordering law for the whole table on every page.
//! Storing the keys and reading them through an index makes the same page
//! ~1 ms (measured; the plan is `SCAN … USING COVERING INDEX`).
//!
//! The keys are DERIVED STATE of the decision-16 law. The law is implemented
//! ONCE, here, over a minimal [`RankLink`] view — `EndpointRow::link_test_key`
//! delegates to it, so the panel order, the stored keys, and the parity golden
//! cannot drift apart. SQL only stores the numbers and reads them back; it
//! never re-derives the law.

use std::collections::HashMap;

use toasty_core::stmt::Value;

use crate::models_toasty::{
    ConfigType, Endpoint, EndpointId, EndpointRank, EndpointRow, HostType, Latency, ProfileErr,
    ProfileStats, ProtocolId,
};

/// Sentinel for "no display link": sorts before any real timestamp, matching
/// the retired SQL `COALESCE(<seen>, '')` (empty text sorts first ascending).
pub const NO_SEEN: i64 = i64::MIN;
/// Sentinel for "no measured display link", matching the retired
/// `COALESCE(<speed>, -1)`.
pub const NO_SPEED: i64 = -1;
/// Sentinel for "unknown config type", matching the retired `COALESCE(…, 2)`.
pub const CONFIG_OTHER: i64 = 2;

/// The link facts the ordering law reads. Built from a typed `ProfileStats`
/// when one is in hand, or straight from the stored columns when the refresh
/// path would otherwise pay ~0.8 ms per bound id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankLink {
    pub protocol_id: i64,
    /// `Some(real)` / `Some(fast)` for a measured link (the flag is "is it a
    /// real ping"), `None` when the link carries no latency.
    pub measured: Option<bool>,
    pub delay: i32,
    pub error_kind: Option<ProfileErr>,
    pub seen_nanos: i64,
    pub speed: Option<i64>,
    pub traffic: i64,
    pub config: ConfigType,
}

impl RankLink {
    /// Ascending key `(tier, latency, -seen, protocol_id)` — the decision-16
    /// law, mirroring the retired SQL bands:
    /// 0 real-ok, 1 fast-ok, 2 untested, 3 real/name-err, 4 fast-err,
    /// 5 dns-unresolved. Only success tiers carry a delay.
    #[must_use]
    pub const fn key(&self, dns_unresolved: bool) -> (u8, i32, i64, i64) {
        let tier = if dns_unresolved {
            5
        } else if let Some(kind) = &self.error_kind {
            match kind {
                // A name-resolution failure surfaces on a real attempt, so it
                // shares the real-err bucket.
                ProfileErr::Real | ProfileErr::Name => 3,
                ProfileErr::Fast => 4,
            }
        } else {
            match self.measured {
                Some(true) => 0,
                Some(false) => 1,
                None => 2,
            }
        };
        let latency = if tier <= 1 { self.delay } else { i32::MAX };
        (tier, latency, -self.seen_nanos, self.protocol_id)
    }

    /// "Measured" rank of the display preference: real (0) before fast (1).
    const fn display_rank(&self) -> Option<u8> {
        match self.measured {
            Some(true) => Some(0),
            Some(false) => Some(1),
            None => None,
        }
    }
}

impl From<&ProfileStats> for RankLink {
    fn from(link: &ProfileStats) -> Self {
        let (measured, delay) = match link.latency {
            Some(Latency::Real { delay, .. }) => (Some(true), delay),
            Some(Latency::Fast { delay }) => (Some(false), delay),
            None => (None, i32::MAX),
        };
        Self {
            protocol_id: link.protocol_id.get(),
            measured,
            delay,
            error_kind: link.error.as_ref().map(|e| e.kind),
            seen_nanos: nanos(link.last_seen_at),
            speed: link.speed_bps,
            traffic: link
                .traffic
                .total_up
                .saturating_add(link.traffic.total_down),
            config: link.config_type,
        }
    }
}

/// Epoch nanoseconds of a `jiff` timestamp.
fn nanos(ts: jiff::Timestamp) -> i64 {
    i64::try_from(ts.as_nanosecond()).unwrap_or(i64::MAX)
}

/// True when the endpoint is a DNS host whose resolution has not landed —
/// the flag that collapses its links into one band (decision 16, tier 5).
#[must_use]
pub fn dns_unresolved_endpoint(host_type: HostType, resolved_as: &[String]) -> bool {
    host_type == HostType::Dns && resolved_as.is_empty()
}

/// True for an [`EndpointRow`].
#[must_use]
pub fn dns_unresolved(row: &EndpointRow) -> bool {
    dns_unresolved_endpoint(row.endpoint.host_type, &row.endpoint.resolved_as)
}

/// Index of the endpoint's display link.
///
/// The rule the retired SQL used: a manual override naming an existing link,
/// else the best measured link (real before fast, lowest delay, then protocol
/// id), else nothing. Deliberately not `active_link()`: that one falls back to
/// the selected link (the first link when nothing was ever measured), while
/// the ordering law treats "no measurement" as a sentinel that sorts first
/// ascending.
#[must_use]
pub fn display_link_index(links: &[RankLink], override_protocol: Option<i64>) -> Option<usize> {
    if let Some(pid) = override_protocol
        && let Some(index) = links.iter().position(|l| l.protocol_id == pid)
    {
        return Some(index);
    }
    links
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            l.display_rank()
                .map(|rank| (i, rank, l.delay, l.protocol_id))
        })
        .min_by_key(|&(_, rank, delay, protocol)| (rank, delay, protocol))
        .map(|(i, _, _, _)| i)
}

const fn config_rank(config: ConfigType) -> i64 {
    match config {
        ConfigType::Form => 0,
        ConfigType::ShareUrl => 1,
    }
}

/// Compute the stored keys for one endpoint. `None` when it has no links: the
/// page only ever lists endpoints that have at least one.
#[must_use]
pub fn compute_rank(
    endpoint_id: EndpointId,
    dns_unresolved: bool,
    override_protocol: Option<i64>,
    links: &[RankLink],
) -> Option<EndpointRank> {
    let (tier, latency, neg_seen, protocol) = links.iter().map(|l| l.key(dns_unresolved)).min()?;
    let display = display_link_index(links, override_protocol).map(|i| links[i]);
    let newest_seen = links.iter().map(|l| l.seen_nanos).max().unwrap_or(0);
    Some(EndpointRank {
        endpoint_id,
        dns: i64::from(dns_unresolved),
        tier: i64::from(tier),
        latency: i64::from(latency),
        seen: -neg_seen,
        protocol,
        display_seen: display.map_or(NO_SEEN, |l| l.seen_nanos),
        speed: display.map_or(NO_SPEED, |l| l.speed.unwrap_or(NO_SPEED)),
        traffic: display.map_or(0, |l| l.traffic),
        config: display.map_or(CONFIG_OTHER, |l| config_rank(l.config)),
        newest_seen,
    })
}

/// Keys for a whole row (the backfill/typed entry point).
#[must_use]
pub fn rank_of_row(row: &EndpointRow) -> Option<EndpointRank> {
    let links: Vec<RankLink> = row.links.iter().map(RankLink::from).collect();
    compute_rank(
        row.endpoint.id,
        dns_unresolved(row),
        row.endpoint.manual_protocol_override.map(ProtocolId::get),
        &links,
    )
}

// ── Persistence ─────────────────────────────────────────────────────────
//
// A side table, not a `toasty` model: adding a model would need `push_schema`
// (skipped at the current tag) and therefore a tag bump, which decision 4
// defines as a full wipe. `CREATE TABLE IF NOT EXISTS` is additive and
// idempotent, so an existing database keeps its rows, and dropping the table
// restores the previous behaviour.
//
// Reads and writes inline their integer ids instead of binding them: the
// engine parses ~0.8 ms per bound parameter (200 ids = 174 ms; the same
// statement with literals = 9.7 ms), which would dominate every refresh.

/// The Test order (decision 16): the default sort, and the one the tab
/// scrolls under.
///
/// It stays a raw statement because toasty's `#[index]` is single-column and
/// cannot express a composite whose last-but-one term is DESCENDING — and this
/// index is what makes a page an index scan (~1 ms) instead of a sort over
/// every endpoint (~240 ms). Additive (`IF NOT EXISTS`), no data of its own.
const COVERING_INDEX: &str = "CREATE INDEX IF NOT EXISTS endpoint_rank_test ON endpoint_rank(\
     rank_dns, rank_tier, rank_latency, rank_seen DESC, rank_protocol, endpoint_id)";

/// Rows per bulk statement.
const RANK_CHUNK: usize = 400;

/// Column order the bulk insert writes (explicit so a schema reorder cannot
/// silently mis-map values).
const RANK_COLUMNS: &str = "endpoint_id, rank_dns, rank_tier, rank_latency, rank_seen, \
     rank_protocol, rank_display_seen, rank_speed, rank_traffic, rank_config, \
     rank_newest_seen";

/// The view windows read this column.
const WINDOW_INDEX: &str =
    "CREATE INDEX IF NOT EXISTS endpoint_rank_window ON endpoint_rank(rank_newest_seen)";

/// Create the rank table and, on a database that has none yet, fill it from
/// the current link state. Runs once per database: an upgraded one pays the
/// backfill here, at open, instead of on its first page.
pub(crate) async fn ensure(conn: &mut toasty::Connection) -> crate::Result<()> {
    // The table itself comes from the schema (tag 8) — this only creates the
    // indexes and, on a database whose keys are not materialized yet, fills
    // them. One transaction: the indexes and the fill land together, and no
    // implicit write lock outlives the call (leaving one behind made the next
    // writer on the pool time out with "database is locked").
    let mut tx = conn.transaction().await?;
    let result = ensure_in(&mut tx).await;
    match result {
        Ok(()) => tx.commit().await?,
        Err(e) => return Err(e),
    }
    Ok(())
}

async fn ensure_in(conn: &mut impl toasty::Executor) -> crate::Result<()> {
    for ddl in [COVERING_INDEX, WINDOW_INDEX] {
        toasty::sql::query(ddl).exec(conn).await?;
    }
    if scalar_i64(conn, "SELECT COUNT(*) FROM endpoint_rank").await? > 0 {
        // A database whose keys are absent or stale (written before a refresh
        // path existed, or by a path that bypassed one) heals here rather than
        // hiding rows from the page.
        repair_missing(conn).await?;
        return Ok(());
    }
    let written = backfill_all(conn).await?;
    tracing::info!(target: "xray_tui_db", "endpoint_rank: backfilled {written} rows");
    Ok(())
}

/// One integer from a single-value query.
async fn scalar_i64(conn: &mut impl toasty::Executor, sql: &str) -> crate::Result<i64> {
    let rows = toasty::sql::query(sql).exec(conn).await?;
    Ok(rows
        .first()
        .and_then(|row| match row {
            Value::Record(record) => record.fields.first().cloned(),
            _ => None,
        })
        .and_then(|v| match v {
            Value::I64(n) => Some(n),
            _ => None,
        })
        .unwrap_or(0))
}

/// Upsert the rank rows on the caller's executor, one multi-row statement per
/// chunk.
///
/// Per-row `upsert_by_endpoint_id` costs ~1.2 ms per statement on this engine,
/// so a flush window's 400 endpoints took ~470 ms and a 7.7k-endpoint import
/// ~8 s of key writes; this form does the same rows in one statement per chunk
/// (measured ~10× cheaper). The values are inlined for the same reason the
/// reads are: they are integers the database produced, never user text, and
/// the engine charges ~0.8 ms per bound parameter.
pub(crate) async fn write(
    conn: &mut impl toasty::Executor,
    ranks: &[EndpointRank],
) -> crate::Result<usize> {
    for chunk in ranks.chunks(RANK_CHUNK) {
        let values = chunk
            .iter()
            .map(|r| {
                format!(
                    "({},{},{},{},{},{},{},{},{},{},{})",
                    r.endpoint_id.get(),
                    r.dns,
                    r.tier,
                    r.latency,
                    r.seen,
                    r.protocol,
                    r.display_seen,
                    r.speed,
                    r.traffic,
                    r.config,
                    r.newest_seen
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        toasty::sql::query(format!(
            "INSERT OR REPLACE INTO endpoint_rank ({RANK_COLUMNS}) VALUES {values}"
        ))
        .exec(conn)
        .await?;
    }
    Ok(ranks.len())
}

/// The raw facts the rank law needs, straight from the stored columns.
struct RawEndpoint {
    dns_unresolved: bool,
    override_protocol: Option<i64>,
}

impl crate::Database {
    /// Create the keys of any endpoint that has links but no rank row.
    ///
    /// The page drives from `endpoint_rank`, so a write that bypassed the
    /// refresh would hide its endpoint until this ran. It runs at `open` and
    /// is available to callers that write links outside the normal paths
    /// (fixtures, maintenance scripts).
    pub async fn repair_endpoint_ranks(&self) -> crate::Result<usize> {
        let mut conn = self.connection().await?;
        repair_missing(&mut conn).await
    }

    /// Recompute the stored ordering keys for `endpoint_ids`.
    ///
    /// Called by every write path that can change a link (single insert,
    /// patch flush, bulk import, error sweep), so a stored key is never older
    /// than the write that invalidated it.
    pub async fn refresh_endpoint_ranks(
        &self,
        endpoint_ids: &[EndpointId],
    ) -> crate::Result<usize> {
        let mut conn = self.connection().await?;
        refresh(&mut conn, endpoint_ids).await
    }
}

/// Delete rank rows whose endpoint no longer has any link.
///
/// The page drives from this table, so a lingering row would list a linkless
/// endpoint. Called by the deletion owners (`purge_expired`) rather than on a
/// timer: they are the only writers that remove links.
pub(crate) async fn prune(
    conn: &mut impl toasty::Executor,
    endpoint_ids: &[EndpointId],
) -> crate::Result<usize> {
    for id in endpoint_ids {
        EndpointRank::filter_by_endpoint_id(*id)
            .delete()
            .exec(conn)
            .await?;
    }
    Ok(endpoint_ids.len())
}

/// Recompute EVERY endpoint's stored keys from its current links.
///
/// For the wholesale resets (`clear_all_stats`) where nothing narrower is
/// correct: every link just lost the columns the keys are made of.
pub(crate) async fn backfill_all(conn: &mut impl toasty::Executor) -> crate::Result<usize> {
    let endpoints: Vec<Endpoint> = Endpoint::all().exec(conn).await?;
    let links: Vec<ProfileStats> = ProfileStats::all().exec(conn).await?;
    let mut by_endpoint: HashMap<EndpointId, Vec<RankLink>> = HashMap::new();
    for link in links {
        by_endpoint
            .entry(link.endpoint_id)
            .or_default()
            .push(RankLink::from(&link));
    }
    let ranks: Vec<EndpointRank> = endpoints
        .into_iter()
        .filter_map(|endpoint| {
            let links = by_endpoint.remove(&endpoint.id)?;
            compute_rank(
                endpoint.id,
                dns_unresolved_endpoint(endpoint.host_type, &endpoint.resolved_as),
                endpoint.manual_protocol_override.map(ProtocolId::get),
                &links,
            )
        })
        .collect();
    write(conn, &ranks).await
}

/// Backfill rank rows for endpoints that have links but no row yet.
pub(crate) async fn repair_missing(conn: &mut impl toasty::Executor) -> crate::Result<usize> {
    let rows = toasty::sql::query(
        "SELECT e.id FROM endpoints e WHERE \
         EXISTS (SELECT 1 FROM profile_stats p WHERE p.endpoint_id = e.id) \
         AND NOT EXISTS (SELECT 1 FROM endpoint_rank k WHERE k.endpoint_id = e.id)",
    )
    .exec(conn)
    .await?;
    let ids: Vec<EndpointId> = rows
        .iter()
        .filter_map(|row| match row {
            Value::Record(record) => record.fields.first().cloned(),
            _ => None,
        })
        .filter_map(|v| match v {
            Value::I64(id) => Some(EndpointId::new(id)),
            _ => None,
        })
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    refresh(conn, &ids).await
}

/// Recompute the stored keys for `endpoint_ids` from their current links.
///
/// The single refresh entry point: every write path that can change a link
/// (patch flush, bulk upsert, error sweep) calls it for the endpoints it
/// touched, so a stored key is never older than the write that invalidated it.
pub(crate) async fn refresh(
    conn: &mut impl toasty::Executor,
    endpoint_ids: &[EndpointId],
) -> crate::Result<usize> {
    let mut ids: Vec<i64> = endpoint_ids.iter().map(|id| id.get()).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(0);
    }
    let id_list = ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let endpoints = load_raw_endpoints(conn, &id_list).await?;
    let mut links: HashMap<i64, Vec<RankLink>> = HashMap::new();
    let rows = toasty::sql::query(format!(
        "SELECT endpoint_id, protocol_id, error_kind, latency, latency_delay, \
         last_seen_at, speed_bps, traffic_total_up, traffic_total_down, config_type \
         FROM profile_stats WHERE endpoint_id IN ({id_list})"
    ))
    .exec(conn)
    .await?;
    for row in &rows {
        let Value::Record(record) = row else { continue };
        let field = |i: usize| record.fields.get(i);
        let Some(endpoint_id) = field(0).and_then(as_i64) else {
            continue;
        };
        links.entry(endpoint_id).or_default().push(RankLink {
            protocol_id: field(1).and_then(as_i64).unwrap_or(0),
            error_kind: field(2)
                .and_then(as_text)
                .and_then(|k| parse_error_kind(&k)),
            measured: field(3).and_then(as_text).and_then(|k| match k.as_str() {
                "real" => Some(true),
                "fast" => Some(false),
                _ => None,
            }),
            delay: field(4)
                .and_then(as_i64)
                .and_then(|d| i32::try_from(d).ok())
                .unwrap_or(0),
            seen_nanos: field(5)
                .and_then(as_text)
                .and_then(|t| parse_nanos(&t))
                .unwrap_or(0),
            speed: field(6).and_then(as_i64),
            traffic: field(7).and_then(as_i64).unwrap_or(0)
                + field(8).and_then(as_i64).unwrap_or(0),
            config: match field(9).and_then(as_text).as_deref() {
                Some("form") => ConfigType::Form,
                _ => ConfigType::ShareUrl,
            },
        });
    }

    let ranks: Vec<EndpointRank> = ids
        .iter()
        .filter_map(|id| {
            let endpoint = endpoints.get(id)?;
            let empty: Vec<RankLink> = Vec::new();
            let endpoint_links = links.get(id).unwrap_or(&empty);
            compute_rank(
                EndpointId::new(*id),
                endpoint.dns_unresolved,
                endpoint.override_protocol,
                endpoint_links,
            )
        })
        .collect();
    write(conn, &ranks).await
}

async fn load_raw_endpoints(
    conn: &mut impl toasty::Executor,
    id_list: &str,
) -> crate::Result<HashMap<i64, RawEndpoint>> {
    let rows = toasty::sql::query(format!(
        "SELECT id, host_type, resolved_as, manual_protocol_override FROM endpoints \
         WHERE id IN ({id_list})"
    ))
    .exec(conn)
    .await?;
    let mut out = HashMap::new();
    for row in &rows {
        let Value::Record(record) = row else { continue };
        let field = |i: usize| record.fields.get(i);
        let Some(id) = field(0).and_then(as_i64) else {
            continue;
        };
        let host_type = field(1).and_then(as_text).unwrap_or_default();
        let resolved_as = field(2)
            .and_then(as_text)
            .unwrap_or_else(|| "[]".to_string());
        out.insert(
            id,
            RawEndpoint {
                dns_unresolved: host_type == "dns"
                    && (resolved_as == "[]" || resolved_as.is_empty()),
                override_protocol: field(3).and_then(as_i64),
            },
        );
    }
    Ok(out)
}

const fn as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::I64(n) => Some(*n),
        _ => None,
    }
}

fn as_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn parse_error_kind(text: &str) -> Option<ProfileErr> {
    match text {
        "real" => Some(ProfileErr::Real),
        "fast" => Some(ProfileErr::Fast),
        "name" => Some(ProfileErr::Name),
        _ => None,
    }
}

/// `2026-09-11T07:38:50.858130960Z` → epoch nanoseconds. The stored text is
/// fixed-width UTC with nine fractional digits, so it parses directly.
fn parse_nanos(text: &str) -> Option<i64> {
    text.parse::<jiff::Timestamp>()
        .ok()
        .map(|ts| i64::try_from(ts.as_nanosecond()).unwrap_or(i64::MAX))
}
