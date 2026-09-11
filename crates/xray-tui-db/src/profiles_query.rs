//! Raw-SQL page queries for the Profiles tab.
//!
//! This module is the ONLY owner of raw SQL in the workspace outside the
//! PRAGMAs in [`crate::database::Database::open`]/`conn`. Rules (see
//! `docs/aegis/specs/2026-09-11-profiles-page-query-design.md` §5):
//!
//! - parameterised `?N` binds only; user text is never interpolated,
//! - it decodes only `endpoints.id` and ordering keys, never model columns,
//! - it is read-only: it never writes `profile_stats`.
//!
//! Ordering is the decision-16 test-priority law, expressed once as
//! [`WEIGHT_TEMPLATE`] and mirrored by the Rust oracle that the parity test in
//! `xray-tui` compares against.

use std::collections::HashMap;

use jiff::Timestamp;
use toasty_core::stmt::Value;

use crate::database::Database;
use crate::error::{DatabaseError, Result};
use crate::models_toasty::{EndpointId, ProfileStats, ProtocolId, PurgatoryView};

/// Default page size for the Profiles window.
pub const DEFAULT_PAGE_SIZE: usize = 200;

// ── Ordering weight (decision 16) ───────────────────────────────────────
//
// `{t}` is the `profile_stats` alias. Bands, mirroring
// `EndpointRow::link_test_tier` exactly:
//   0 real success    -> delay                (ascending = fastest first)
//   1 fast success    -> 1_073_741_824 + delay
//   2 untested        -> 2_147_483_644
//   3 real|name error -> 2_147_483_645        (`Name` shares the real band)
//   4 fast error      -> 2_147_483_646
const WEIGHT_TEMPLATE: &str = "CASE WHEN {t}.error = 1 AND {t}.error_kind IN ('real','name') \
        THEN 2147483645 \
     WHEN {t}.error = 1 AND {t}.error_kind = 'fast' THEN 2147483646 \
     WHEN {t}.latency IS NULL THEN 2147483644 \
     WHEN {t}.latency = 'real' THEN {t}.latency_delay \
     ELSE 1073741824 + {t}.latency_delay END";

/// Render the weight expression for one table alias.
fn weight(alias: &str) -> String {
    WEIGHT_TEMPLATE.replace("{t}", alias)
}

/// DNS-unresolved predicate over the persisted column — the same fact the TUI
/// predicate reads, so the two cannot disagree.
fn dns_flag(alias: &str) -> String {
    format!("CASE WHEN {alias}.host_type = 'dns' AND {alias}.resolved_as = '[]' THEN 1 ELSE 0 END")
}

/// Effective weight inside the representative-link window: an unresolved
/// endpoint collapses to one band, matching `link_test_key` with
/// `tier = 5, latency = i32::MAX` (the newest link then leads).
fn eff_weight() -> String {
    format!(
        "CASE WHEN e2.host_type = 'dns' AND e2.resolved_as = '[]' THEN 0 ELSE {} END",
        weight("ps")
    )
}

/// Projection of the page query.
const PROJ_ID: &str = "r.endpoint_id";

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

/// The page source: the representative-link window joined to its endpoint.
///
/// Emits `SELECT <projection> FROM (<representative-link window>) r JOIN
/// endpoints e … WHERE r.rn = 1`, then the view / group / search predicates.
/// Placeholder numbering follows bind order.
fn base_select(sql: &mut Sql, req: &PageRequest, projection: &str) {
    sql.push("SELECT ");
    sql.push(projection);
    base_from_where(sql, req);
}

/// The shared `FROM`/`WHERE` of every profiles query: the representative-link
/// window joined to its endpoint row, plus the view / group / search
/// predicates.
fn base_from_where(sql: &mut Sql, req: &PageRequest) {
    sql.push(" FROM (SELECT ps.endpoint_id, ps.protocol_id, ps.last_seen_at, ");
    let dns = dns_flag("e2");
    sql.push(&dns);
    sql.push(" AS dns_flag, ");
    let eff = eff_weight();
    sql.push(&eff);
    sql.push(" AS eff_weight, ROW_NUMBER() OVER (PARTITION BY ps.endpoint_id ORDER BY ");
    sql.push(&eff);
    sql.push(
        " ASC, ps.last_seen_at DESC, ps.protocol_id ASC) AS rn \
         FROM profile_stats ps JOIN endpoints e2 ON e2.id = ps.endpoint_id\
         ) r JOIN endpoints e ON e.id = r.endpoint_id WHERE r.rn = 1",
    );

    match req.view {
        PurgatoryView::All => {}
        PurgatoryView::Active => {
            let ts = sql.bind(sql_ts(req.active_threshold));
            sql.push(&format!(
                " AND EXISTS (SELECT 1 FROM profile_stats p2 WHERE p2.endpoint_id = r.endpoint_id \
                 AND p2.last_seen_at >= {ts})"
            ));
        }
        PurgatoryView::Stale => {
            let stale = sql.bind(sql_ts(req.stale_threshold));
            let active = sql.bind(sql_ts(req.active_threshold));
            sql.push(&format!(
                " AND EXISTS (SELECT 1 FROM profile_stats p2 WHERE p2.endpoint_id = r.endpoint_id \
                 AND p2.last_seen_at >= {stale}) \
                 AND NOT EXISTS (SELECT 1 FROM profile_stats p3 WHERE p3.endpoint_id = r.endpoint_id \
                 AND p3.last_seen_at >= {active})"
            ));
        }
    }

    if let Some(group_id) = &req.group_id {
        let gid = sql.bind(group_id.clone());
        sql.push(&format!(
            " AND EXISTS (SELECT 1 FROM endpoint_groups eg WHERE eg.endpoint_id = r.endpoint_id \
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

/// A column of the endpoint's DISPLAY link: the manual override when it names
/// an existing link, else the best measured link (real before fast, lowest
/// delay). `expr` uses `{t}` for the link alias.
fn display_link(expr: &str) -> String {
    let over = expr.replace("{t}", "ps2");
    let best = expr.replace("{t}", "ps3");
    format!(
        "COALESCE(\
         (SELECT {over} FROM profile_stats ps2 \
         WHERE ps2.endpoint_id = r.endpoint_id AND ps2.protocol_id = e.manual_protocol_override),\
         (SELECT {best} FROM profile_stats ps3 \
         WHERE ps3.endpoint_id = r.endpoint_id AND ps3.latency IS NOT NULL \
         ORDER BY CASE WHEN ps3.latency = 'real' THEN 0 ELSE 1 END ASC, \
         ps3.latency_delay ASC, ps3.protocol_id ASC LIMIT 1))"
    )
}

/// The full ORDER BY term list for a sort column and direction.
///
/// Every term flips with the direction, mirroring the oracle's
/// `if asc { cmp } else { cmp.reverse() }`: a descending sort is the exact
/// reverse of the ascending one, tiebreaks included.
fn order_terms(sort: PageSort, ascending: bool) -> String {
    // `natural_asc` is the term's direction in the ascending case.
    let dir = |natural_asc: bool| {
        if ascending == natural_asc {
            "ASC"
        } else {
            "DESC"
        }
    };
    match sort {
        PageSort::Test => format!(
            "r.dns_flag {}, r.eff_weight {}, r.last_seen_at {}, r.protocol_id {}, r.endpoint_id {}",
            dir(true),
            dir(true),
            dir(false),
            dir(true),
            dir(true)
        ),
        PageSort::Address => format!("e.host {}, e.id {}", dir(true), dir(true)),
        PageSort::Port => format!("e.port {}, e.id {}", dir(true), dir(true)),
        PageSort::LastSeen => format!(
            "COALESCE({}, '') {}, e.id {}",
            display_link("{t}.last_seen_at"),
            dir(true),
            dir(true)
        ),
        PageSort::Speed => format!(
            "COALESCE({}, -1) {}, e.id {}",
            display_link("COALESCE({t}.speed_bps, -1)"),
            dir(true),
            dir(true)
        ),
        PageSort::Traffic => format!(
            "COALESCE({}, 0) {}, e.id {}",
            display_link("({t}.traffic_total_up + {t}.traffic_total_down)"),
            dir(true),
            dir(true)
        ),
        PageSort::ConfigType => format!(
            "COALESCE({}, 2) {}, e.id {}",
            display_link(
                "CASE {t}.config_type WHEN 'form' THEN 0 WHEN 'share_url' THEN 1 ELSE 2 END"
            ),
            dir(true),
            dir(true)
        ),
    }
}

/// Append `ORDER BY <terms>`.
fn order_by(sql: &mut Sql, sort: PageSort, ascending: bool) {
    sql.push(" ORDER BY ");
    sql.push(&order_terms(sort, ascending));
}

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

fn decode_i64(row: &Value) -> Result<i64> {
    match row {
        Value::Record(record) => match record.fields.first() {
            Some(Value::I64(n)) => Ok(*n),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: unexpected integer column: {other:?}"
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
        base_select(&mut sql, req, PROJ_ID);
        order_by(&mut sql, req.sort, req.ascending);
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
        sql.push("SELECT COUNT(*) FROM (");
        base_select(&mut sql, req, PROJ_ID);
        sql.push(")");
        let rows = sql.exec(conn).await?;
        rows.first().map_or(Ok(0), decode_count)
    }

    /// Every endpoint id in the filtered view, in display order.
    pub async fn profiles_ids(&self, req: &PageRequest) -> Result<Vec<EndpointId>> {
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID);
        order_by(&mut sql, req.sort, req.ascending);
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
        base_select(&mut sql, req, PROJ_ID);
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
        base_select(&mut sql, req, PROJ_ID);
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
    /// One query: the request's ordering is applied by `ROW_NUMBER()` over the
    /// same source the page uses, so it is correct for every sort and
    /// direction (a hard-coded key comparison would only hold for
    /// `Test`/ascending).
    pub async fn profiles_anchor(
        &self,
        req: &PageRequest,
        endpoint_id: EndpointId,
    ) -> Result<Option<usize>> {
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        sql.push("SELECT t.rn - 1 FROM (SELECT r.endpoint_id AS eid, ROW_NUMBER() OVER (ORDER BY ");
        sql.push(&order_terms(req.sort, req.ascending));
        sql.push(") AS rn");
        base_from_where(&mut sql, req);
        sql.push(") t WHERE t.eid = ");
        let bind = sql.bind(endpoint_id.get());
        sql.push(&bind);

        let rows = sql.exec(&mut conn).await?;
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        let offset = decode_i64(row)?;
        Ok(Some(usize::try_from(offset.max(0)).unwrap_or(0)))
    }

    /// Per-endpoint link order for a page's endpoints: the decision-16 order
    /// used by the expanded panel and the sub-row navigation.
    pub async fn profile_link_order(
        &self,
        endpoint_ids: &[EndpointId],
    ) -> Result<HashMap<EndpointId, Vec<ProtocolId>>> {
        if endpoint_ids.is_empty() {
            return Ok(HashMap::new());
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
        let eff = weight("ps");
        sql.push(&eff);
        sql.push(" ASC, ps.last_seen_at DESC, ps.protocol_id ASC");

        let rows = sql.exec(&mut conn).await?;
        let mut out: HashMap<EndpointId, Vec<ProtocolId>> = HashMap::new();
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
