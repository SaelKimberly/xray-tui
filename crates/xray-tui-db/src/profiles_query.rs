//! Raw-SQL owner of the Profiles page query (ADR 0001).
//!
//! Ordering, filtering, counting and paging are SQL; the page IS the filtered
//! list. Every statement here is read-only, binds its parameters, and decodes
//! only ids and keys — model hydration stays on the typed path
//! ([`crate::Database::load_page_rows`]).
//!
//! The ORDER BY reads the materialized keys in `endpoint_rank`
//! ([`crate::endpoint_rank`]), which the Rust ordering law computes. SQL never
//! re-derives that law: an index-driven scan of 200 rows replaced a
//! `ROW_NUMBER()` window that cost ~975 ms per page on the reference feed.

use crate::Database;
use crate::error::{DatabaseError, Result};
use crate::models_toasty::{EndpointId, ProfileStats, ProtocolId, PurgatoryView};
use jiff::Timestamp;
use toasty_core::stmt::Value;

/// Default rows per page.
pub const DEFAULT_PAGE_SIZE: usize = 200;

/// Projection of every page-shaped query: the endpoint id, taken from the
/// rank table the query drives from.
const PROJ_ID: &str = "k.endpoint_id";

// ── Request / response ──────────────────────────────────────────────────

/// Sort column of the Profiles tab. `Core` is absent by design: it resolves
/// through runtime `protocol_core_overrides` and cannot be expressed in SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSort {
    /// Decision-16 test priority, ordered by the representative link.
    Test,
    Address,
    Port,
    LastSeen,
    Speed,
    Traffic,
    ConfigType,
}

/// One page request. The thresholds are view filters, not ordering inputs.
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub view: PurgatoryView,
    pub active_threshold: Timestamp,
    pub stale_threshold: Timestamp,
    pub search: Option<String>,
    pub group_id: Option<String>,
    pub sort: PageSort,
    pub ascending: bool,
    pub offset: usize,
    pub limit: usize,
}

impl PageRequest {
    /// The sort this request asks for, with its direction.
    #[must_use]
    pub fn order_terms(&self) -> Vec<OrderTerm> {
        order_terms(self.sort, self.ascending)
    }
}

/// One page of endpoint ids in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageMeta {
    pub ids: Vec<EndpointId>,
    /// Filtered total across all pages (the footer count).
    pub total: u64,
    /// Effective offset after clamping to `total`.
    pub offset: usize,
}

// ── SQL building ────────────────────────────────────────────────────────

/// Statement text plus its ordered binds.
struct Sql {
    text: String,
    params: Vec<Value>,
}

impl Sql {
    const fn new() -> Self {
        Self {
            text: String::new(),
            params: Vec::new(),
        }
    }

    fn push(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn bind<T: Into<Value>>(&mut self, v: T) -> String {
        self.params.push(v.into());
        format!("?{}", self.params.len())
    }

    async fn exec(&self, conn: &mut toasty::Connection) -> Result<Vec<Value>> {
        let mut query = toasty::sql::query(self.text.clone());
        for param in &self.params {
            query = query.bind(param.clone());
        }
        Ok(query.exec(conn).await?)
    }
}

/// Fixed-width RFC3339 binding for range predicates.
///
/// Matches the stored 30-character nanosecond format
/// (`2026-09-11T07:38:50.858130960Z`), so lexicographic comparison equals
/// chronological comparison. jiff trims trailing zeros, hence the explicit
/// `{:09}` fraction.
#[must_use]
pub fn sql_ts(ts: Timestamp) -> String {
    let base = Timestamp::from_second(ts.as_second())
        .map_or_else(|_| "1970-01-01T00:00:00Z".to_string(), |t| t.to_string());
    let nanos = ts.subsec_nanosecond().unsigned_abs();
    let head = base.strip_suffix('Z').unwrap_or(&base);
    format!("{head}.{nanos:09}Z")
}

/// Escape LIKE metacharacters for an `ESCAPE '\'` clause.
fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// ── Ordering ────────────────────────────────────────────────────────────

/// One ORDER BY term: the expression and its EFFECTIVE direction. A descending
/// request reverses every term (tiebreaks included), mirroring the Rust
/// oracle's `cmp.reverse()`.
#[derive(Debug, Clone)]
pub struct OrderTerm {
    pub expr: String,
    pub asc: bool,
}

/// A stored rank column, referenced bare so the ORDER BY can be served by the
/// covering index (wrapping it in `COALESCE` forces a sort of every row:
/// measured 240 ms versus 1 ms). Every endpoint that has links has a rank row
/// — the write paths and the backfill keep it that way, and the freshness test
/// pins it — so a NULL here means a write path skipped the refresh, and the
/// row lands at the window's edge instead of silently mid-list.
fn rank_col(column: &str) -> String {
    format!("k.{column}")
}

/// The ORDER BY term list for a sort column and direction.
#[must_use]
pub fn order_terms(sort: PageSort, ascending: bool) -> Vec<OrderTerm> {
    let term = |expr: String, natural_asc: bool| OrderTerm {
        expr,
        asc: natural_asc == ascending,
    };
    match sort {
        PageSort::Test => vec![
            term(rank_col("rank_dns"), true),
            term(rank_col("rank_tier"), true),
            term(rank_col("rank_latency"), true),
            term(rank_col("rank_seen"), false),
            term(rank_col("rank_protocol"), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::LastSeen => vec![
            term(rank_col("rank_display_seen"), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::Speed => vec![
            term(rank_col("rank_speed"), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::Traffic => vec![
            term(rank_col("rank_traffic"), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::ConfigType => vec![
            term(rank_col("rank_config"), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::Address => vec![
            term("e.host".to_string(), true),
            term("k.endpoint_id".to_string(), true),
        ],
        PageSort::Port => vec![
            term("e.port".to_string(), true),
            term("k.endpoint_id".to_string(), true),
        ],
    }
}

/// Append `ORDER BY <terms>`.
fn order_by(sql: &mut Sql, order: &[OrderTerm]) {
    sql.push(" ORDER BY ");
    for (i, term) in order.iter().enumerate() {
        if i > 0 {
            sql.push(", ");
        }
        sql.push(&term.expr);
        sql.push(if term.asc { " ASC" } else { " DESC" });
    }
}

/// The page source: endpoints that have at least one link, left-joined to
/// their materialized ordering keys, plus the view / group / search
/// predicates.
///
/// Membership comes from the `EXISTS` terms — the SAME terms
/// [`Database::profiles_count`] counts — so the page and the footer cannot
/// disagree, and an endpoint whose rank row is missing (or stale) still lists;
/// only its position degrades.
fn base_from_where(sql: &mut Sql, req: &PageRequest, join_endpoints: bool) {
    sql.push(" FROM endpoint_rank k");
    if join_endpoints {
        sql.push(" JOIN endpoints e ON e.id = k.endpoint_id");
    }
    sql.push(" WHERE ");
    view_predicate(sql, req);
    if let Some(group_id) = &req.group_id {
        let gid = sql.bind(group_id.clone());
        sql.push(&format!(
            " AND EXISTS (SELECT 1 FROM endpoint_groups eg WHERE eg.endpoint_id = k.endpoint_id \
             AND eg.group_id = {gid})"
        ));
    }
    if let Some(search) = &req.search
        && !search.is_empty()
    {
        let pattern = sql.bind(format!("%{}%", escape_like(&search.to_lowercase())));
        sql.push(&format!(
            " AND (lower(e.host) LIKE {pattern} ESCAPE '\\' \
             OR CAST(e.port AS TEXT) LIKE {pattern} ESCAPE '\\')"
        ));
    }
}

/// The view predicate: "has a link matching the window". `Active` and `Stale`
/// share both computed bounds and let the view pick the ones it uses.
fn view_predicate(sql: &mut Sql, req: &PageRequest) {
    // "Some link falls in the band" is the same question as "the newest link
    // does", so the window reads the stored maximum instead of scanning
    // `profile_stats` per row.
    let nanos = |ts: Timestamp| i64::try_from(ts.as_nanosecond()).unwrap_or(i64::MAX);
    match req.view {
        PurgatoryView::All => {
            sql.push("1 = 1");
        }
        PurgatoryView::Active => {
            let ts = sql.bind(nanos(req.active_threshold));
            sql.push(&format!("k.rank_newest_seen >= {ts}"));
        }
        PurgatoryView::Stale => {
            let stale = sql.bind(nanos(req.stale_threshold));
            let active = sql.bind(nanos(req.active_threshold));
            sql.push(&format!(
                "k.rank_newest_seen >= {stale} AND k.rank_newest_seen < {active}"
            ));
        }
    }
}

/// `SELECT <projection>` over the page source.
fn base_select(sql: &mut Sql, req: &PageRequest, projection: &str, join_endpoints: bool) {
    sql.push("SELECT ");
    sql.push(projection);
    base_from_where(sql, req, join_endpoints);
}

/// Whether a query needs the `endpoints` row: the search predicate reads the
/// host/port, and the Address/Port sorts order by them.
fn needs_endpoints(req: &PageRequest) -> bool {
    req.search.as_ref().is_some_and(|s| !s.is_empty())
        || matches!(req.sort, PageSort::Address | PageSort::Port)
}

// ── Value decoding ──────────────────────────────────────────────────────

fn decode_id(row: &Value) -> Result<EndpointId> {
    match row {
        Value::Record(record) => match record.fields.first() {
            Some(Value::I64(id)) => Ok(EndpointId::new(*id)),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: unexpected id column: {other:?}"
            ))),
        },
        other => Err(DatabaseError::Generic(format!(
            "profiles_query: unexpected row: {other:?}"
        ))),
    }
}

fn decode_count(row: &Value) -> Result<u64> {
    match row {
        Value::Record(record) => match record.fields.first() {
            Some(Value::I64(n)) => Ok(u64::try_from(*n).unwrap_or(0)),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: unexpected count column: {other:?}"
            ))),
        },
        other => Err(DatabaseError::Generic(format!(
            "profiles_query: unexpected row: {other:?}"
        ))),
    }
}

fn decode_pair(row: &Value) -> Result<(EndpointId, ProtocolId)> {
    let Value::Record(record) = row else {
        return Err(DatabaseError::Generic(format!(
            "profiles_query: unexpected row: {row:?}"
        )));
    };
    match (record.fields.first(), record.fields.get(1)) {
        (Some(Value::I64(eid)), Some(Value::I64(pid))) => {
            Ok((EndpointId::new(*eid), ProtocolId::new(*pid)))
        }
        other => Err(DatabaseError::Generic(format!(
            "profiles_query: unexpected pair row: {other:?}"
        ))),
    }
}

// ── Public queries ──────────────────────────────────────────────────────

impl Database {
    /// One page of endpoint ids in display order, plus the filtered total.
    pub async fn profiles_page(&self, req: &PageRequest) -> Result<PageMeta> {
        let mut conn = self.connection().await?;
        let total = self.profiles_count_with(&mut conn, req).await?;
        let offset = req.offset.min(usize::try_from(total).unwrap_or(usize::MAX));

        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID, needs_endpoints(req));
        order_by(&mut sql, &req.order_terms());
        let limit = sql.bind(i64::try_from(req.limit).unwrap_or(i64::MAX));
        let offset_bind = sql.bind(i64::try_from(offset).unwrap_or(i64::MAX));
        sql.push(&format!(" LIMIT {limit} OFFSET {offset_bind}"));

        let rows = sql.exec(&mut conn).await?;
        let ids = rows.iter().map(decode_id).collect::<Result<Vec<_>>>()?;
        Ok(PageMeta { ids, total, offset })
    }

    /// The filtered total across all pages (the footer count).
    pub async fn profiles_count(&self, req: &PageRequest) -> Result<u64> {
        let mut conn = self.connection().await?;
        self.profiles_count_with(&mut conn, req).await
    }

    async fn profiles_count_with(
        &self,
        conn: &mut toasty::Connection,
        req: &PageRequest,
    ) -> Result<u64> {
        let mut sql = Sql::new();
        sql.push("SELECT COUNT(*)");
        base_from_where(&mut sql, req, needs_endpoints(req));
        let rows = sql.exec(conn).await?;
        rows.first().map_or(Ok(0), decode_count)
    }

    /// Every endpoint id in the filtered view, in display order.
    pub async fn profiles_ids(&self, req: &PageRequest) -> Result<Vec<EndpointId>> {
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID, needs_endpoints(req));
        order_by(&mut sql, &req.order_terms());
        let rows = sql.exec(&mut conn).await?;
        rows.iter().map(decode_id).collect()
    }

    /// `(endpoint_id, protocol_id)` for every link of the filtered view — the
    /// batch-planning input.
    pub async fn profiles_link_pairs(
        &self,
        req: &PageRequest,
    ) -> Result<Vec<(EndpointId, ProtocolId)>> {
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        sql.push(
            "SELECT ps.endpoint_id, ps.protocol_id FROM profile_stats ps WHERE ps.endpoint_id IN (",
        );
        base_select(&mut sql, req, PROJ_ID, needs_endpoints(req));
        sql.push(") ORDER BY ps.endpoint_id, ps.protocol_id");
        let rows = sql.exec(&mut conn).await?;
        rows.iter().map(decode_pair).collect()
    }

    /// Endpoints carrying a persisted failure marker on any link.
    pub async fn profiles_failed_ids(&self) -> Result<Vec<EndpointId>> {
        let mut conn = self.connection().await?;
        let rows = toasty::sql::query(
            "SELECT DISTINCT ps.endpoint_id FROM profile_stats ps WHERE ps.error = 1",
        )
        .exec(&mut conn)
        .await?;
        rows.iter().map(decode_id).collect()
    }

    /// Endpoint ids eligible for enrichment seeding (IP hosts and resolved DNS
    /// hosts) across the filtered view.
    pub async fn profiles_enrich_seed_ids(&self, req: &PageRequest) -> Result<Vec<EndpointId>> {
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID, true);
        // Mirrors `ops/enrich.rs`: IP hosts, plus DNS hosts that already carry
        // a cached resolution. An unresolved DNS host must NOT be seeded — an
        // empty `endpoint_info` entry blocks the startup seeding pass and
        // makes `should_resolve` treat the endpoint as a never-retried IP host.
        sql.push(" AND (e.host_type IN ('ipv4','ipv6') OR e.resolved_as <> '[]')");
        let rows = sql.exec(&mut conn).await?;
        rows.iter().map(decode_id).collect()
    }

    /// Offset the endpoint currently occupies in the ordered view — used to
    /// re-anchor the window when a re-sort moves the selected row out of it.
    ///
    /// Counts the rows that sort strictly before the target, using the SAME
    /// term expressions and directions the page orders by, so it is exact for
    /// every sort and direction rather than only the default one.
    pub async fn profiles_anchor(
        &self,
        req: &PageRequest,
        endpoint_id: EndpointId,
    ) -> Result<Option<usize>> {
        let mut conn = self.connection().await?;
        let terms = req.order_terms();

        // The target's own key values, read through the same expressions.
        let key_projection = terms
            .iter()
            .map(|t| t.expr.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let mut key_sql = Sql::new();
        key_sql.push(&format!("SELECT {key_projection}"));
        // The Address/Port sorts read endpoint columns, so their key query
        // needs the join the page query also takes for them.
        key_sql.push(" FROM endpoint_rank k");
        if needs_endpoints(req) {
            key_sql.push(" JOIN endpoints e ON e.id = k.endpoint_id");
        }
        key_sql.push(" WHERE k.endpoint_id = ");
        let bind = key_sql.bind(endpoint_id.get());
        key_sql.push(&bind);
        let rows = key_sql.exec(&mut conn).await?;
        let Some(Value::Record(record)) = rows.first() else {
            return Ok(None);
        };
        let target = record.fields.clone();
        if target.len() != terms.len() {
            return Err(DatabaseError::Generic(
                "profiles_query: anchor key arity mismatch".to_string(),
            ));
        }

        // Lexicographic "sorts strictly before the target": equal on every
        // earlier term, strictly before on this one. The term expressions and
        // directions are the ones `ORDER BY` uses, so this holds for every
        // sort and both directions.
        let mut sql = Sql::new();
        sql.push("SELECT COUNT(*)");
        base_from_where(&mut sql, req, needs_endpoints(req));
        sql.push(" AND (");
        for (i, term) in terms.iter().enumerate() {
            if i > 0 {
                sql.push(" OR ");
            }
            sql.push("(");
            for (j, prefix) in terms.iter().take(i).enumerate() {
                let eq = sql.bind(target[j].clone());
                sql.push(&format!("{} = {eq} AND ", prefix.expr));
            }
            let cmp = sql.bind(target[i].clone());
            let op = if term.asc { "<" } else { ">" };
            sql.push(&format!("{} {op} {cmp})", term.expr));
        }
        sql.push(")");

        let rows = sql.exec(&mut conn).await?;
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        let offset = decode_count(row)?;
        Ok(Some(usize::try_from(offset).unwrap_or(0)))
    }
}

impl Database {
    /// Per-endpoint link order for a page's endpoints: the decision-16 order
    /// used by the expanded panel and the sub-row navigation.
    pub async fn profile_link_order(
        &self,
        endpoint_ids: &[EndpointId],
    ) -> Result<std::collections::HashMap<EndpointId, Vec<ProtocolId>>> {
        if endpoint_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        sql.push(
            "SELECT ps.endpoint_id, ps.protocol_id FROM profile_stats ps WHERE ps.endpoint_id IN (",
        );
        for (i, id) in endpoint_ids.iter().enumerate() {
            if i > 0 {
                sql.push(", ");
            }
            let bind = sql.bind(id.get());
            sql.push(&bind);
        }
        sql.push(") ORDER BY ps.endpoint_id, ");
        // The stored rank row already carries the representative link; the
        // per-link order still comes from the law as SQL expresses it below.
        sql.push(
            "CASE WHEN ps.error = 1 AND ps.error_kind IN ('real','name') THEN 2147483645 \
             WHEN ps.error = 1 AND ps.error_kind = 'fast' THEN 2147483646 \
             WHEN ps.latency IS NULL THEN 2147483644 \
             WHEN ps.latency = 'real' THEN ps.latency_delay \
             ELSE 1073741824 + ps.latency_delay END ASC, ps.last_seen_at DESC, ps.protocol_id ASC",
        );

        let rows = sql.exec(&mut conn).await?;
        let mut out: std::collections::HashMap<EndpointId, Vec<ProtocolId>> =
            std::collections::HashMap::new();
        for row in &rows {
            let (eid, pid) = decode_pair(row)?;
            out.entry(eid).or_default().push(pid);
        }
        Ok(out)
    }
}

/// Order one endpoint's already-loaded links by [`Database::profile_link_order`].
pub fn order_links(links: &mut [ProfileStats], order: Option<&Vec<ProtocolId>>) {
    let Some(order) = order else {
        return;
    };
    let rank = |pid: ProtocolId| order.iter().position(|p| *p == pid).unwrap_or(usize::MAX);
    links.sort_by_key(|l| rank(l.protocol_id));
}
