//! Raw-SQL owner of the Profiles page query (ADR 0001).
//!
//! Ordering, filtering, counting and paging are SQL; the page IS the filtered
//! list. The page queries decode ids and keys.
//!
//! Hydration is the exception that proves the rule: [`Database::
//! load_page_projection`] decodes whole rows (endpoint + links + the
//! protocols' display columns) from ONE statement, because binding a page's
//! ids into the typed hydration reads costs ~0.8 ms per parameter in turso.
//! Everything it decodes comes back through toasty's own [`Load`] impls (see
//! the projection section at the bottom of this file), so the models it
//! returns are the same values the typed path returns — minus the deferred
//! JSON carriers the page never reads.
//!
//! The ORDER BY reads the materialized keys in `endpoint_rank`
//! ([`crate::endpoint_rank`]), which the Rust ordering law computes. SQL never
//! re-derives that law: an index-driven scan of 200 rows replaced a
//! `ROW_NUMBER()` window that cost ~975 ms per page on the reference feed.

use std::collections::HashMap;
use std::net::IpAddr;

use crate::Database;
use crate::error::{DatabaseError, Result};
use crate::models_toasty::{
    ConfigType, Endpoint, EndpointId, EndpointRow, ErrorInfo, HostType, Latency, ProfileErr,
    ProfileStats, Protocol, ProtocolId, PurgatoryView, Security, TrafficStats, Transport,
};
use toasty::Deferred;
use toasty::schema::Load;
use toasty_core::stmt::Value;
use xray_tui_proto::proto_spec::{CoreType, ProtocolKind, SecurityType, TransportType};

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
    /// The endpoint's lowest resolved address (`endpoint_ip.ip_key`). The
    /// addresses are the only IP fact stored per endpoint, so this is the one
    /// sort the JSON-array column could not express at all.
    Ip,
}

/// One page request. The thresholds are view filters, not ordering inputs.
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub view: PurgatoryView,
    /// View window bounds, in epoch seconds (the unit the key columns store).
    pub active_threshold: i64,
    pub stale_threshold: i64,
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
        // Correlated on the RANK table, not on `endpoints`: the page source is
        // `endpoint_rank k` and the join to `endpoints` is only added for the
        // predicates that read it (`needs_endpoints`), so this sort does not
        // pay for one. `min()` over the packed keys is memcmp — the address
        // order.
        //
        // The sentinel `x'ff'` stands for "no address" and is greater than
        // every key (each key starts with the family byte `0x04`/`0x06`), so
        // an endpoint the resolver has not answered for sorts after the
        // addressed ones ascending and before them descending — the reversal
        // the other sorts rely on. It is not a NULL on purpose: every ordering
        // term here is non-null (see `NO_SEEN`/`NO_SPEED`/`CONFIG_OTHER`),
        // and the anchor query binds a sort term's value back into the
        // comparison, which this engine refuses to type for a NULL.
        PageSort::Ip => vec![
            term(
                "COALESCE((SELECT min(ip.ip_key) FROM endpoint_ip ip \
                 WHERE ip.endpoint_id = k.endpoint_id), x'ff')"
                    .to_string(),
                true,
            ),
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
/// [`Database::profiles_page`] counts — so the page and the footer cannot
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
    match req.view {
        PurgatoryView::All => {
            sql.push("1 = 1");
        }
        PurgatoryView::Active => {
            let ts = sql.bind(req.active_threshold);
            sql.push(&format!("k.rank_newest_seen >= {ts}"));
        }
        PurgatoryView::Stale => {
            let stale = sql.bind(req.stale_threshold);
            let active = sql.bind(req.active_threshold);
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

    /// One page of endpoint ids for a sequential walk of the whole feed, in
    /// display order — the batch plan loader's read.
    ///
    /// [`Self::profiles_page`] re-counts the filtered set on every call, which
    /// made a feed-wide plan walk `O(feed²/200)` (92 `COUNT(*)`s for one
    /// 18,334-endpoint batch, 2026-09-17). A walk asks for the total once
    /// (`with_total`) and then reads ids only. Same builders, same ordering,
    /// same offset paging as the page query — the parity test pins them
    /// together.
    pub async fn profiles_walk_page(
        &self,
        req: &PageRequest,
        with_total: bool,
    ) -> Result<(Vec<EndpointId>, Option<u64>)> {
        let mut conn = self.connection().await?;
        let total = if with_total {
            Some(self.profiles_count_with(&mut conn, req).await?)
        } else {
            None
        };

        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID, needs_endpoints(req));
        order_by(&mut sql, &req.order_terms());
        let limit = sql.bind(i64::try_from(req.limit).unwrap_or(i64::MAX));
        let offset_bind = sql.bind(i64::try_from(req.offset).unwrap_or(i64::MAX));
        sql.push(&format!(" LIMIT {limit} OFFSET {offset_bind}"));

        let rows = sql.exec(&mut conn).await?;
        let ids = rows.iter().map(decode_id).collect::<Result<Vec<_>>>()?;
        Ok((ids, total))
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

// ── Page projection: one statement per page (H1) ────────────────────────

/// The projection's columns, in decode order.
///
/// The SQL text and the positional decoder both read this list, so a column
/// can never be read at the wrong index. Every `endpoints` and
/// `profile_stats` column is present; `protocols` omits its three deferred
/// JSON carriers (`transport_data`, `security_data`, `config`) — no page
/// consumer reads them, and decoding them is what made the typed page
/// hydration cost ~0.5 s on the reference feed (7,672 endpoints).
const PAGE_PROJECTION: &[&str] = &[
    // endpoints (11)
    "e.id",
    "e.host",
    "e.host_type",
    "e.port",
    "e.ports",
    "e.last_source",
    "e.manual_protocol_override",
    // The resolved address set, from the table that owns it (one row per
    // address, PK `(endpoint_id, ip_key)`). The column is the PACKED address,
    // so the aggregate carries its hex and the decoder rebuilds the
    // `IpAddr` — the engine has no `inet_ntoa`-style renderer (measured: the
    // function does not exist), and storing a text copy beside the bytes
    // would be a second, disagreeable spelling of the same fact.
    //
    // The rows come back in the PK scan order, i.e. key order — the same order
    // the IP sort and the panel show — so the list needs no ORDER BY inside
    // the aggregate (a correlated derived table with one would be materialized
    // per row: measured 389 ms for a page against 29 ms for this shape).
    "(SELECT group_concat(hex(ip_key), ',') FROM endpoint_ip ip WHERE ip.endpoint_id = e.id)",
    "e.resolved_at",
    "e.created_at",
    // profile_stats (22)
    "ps.protocol_id",
    "ps.endpoint_id",
    "ps.core_type",
    "ps.config_type",
    "ps.last_used_at",
    "ps.last_seen_at",
    "ps.latency",
    "ps.latency_delay",
    "ps.latency_ip",
    "ps.speed_bps",
    "ps.error",
    "ps.error_kind",
    "ps.error_text",
    "ps.traffic_today_up",
    "ps.traffic_today_down",
    "ps.traffic_total_up",
    "ps.traffic_total_down",
    "ps.created_at",
    "ps.updated_at",
    "ps.version",
    // protocols (10; the JSON carriers are absent on purpose)
    "pr.id",
    "pr.sig",
    "pr.proto_kind",
    "pr.transport_type",
    "pr.security_type",
    "pr.security_sni",
    "pr.security_fp",
    "pr.security_insecure",
    "pr.created_at",
];

/// Positional reader over one projection row.
///
/// The driver hands back each SQLite column as a bare [`Value`]
/// (`I64`/`String`/`Null`), never as the engine's converted shapes, so the
/// JSON-array and timestamp columns are decoded here from their stored text
/// and everything else through toasty's own [`Load`] impl.
struct Projection<'a> {
    fields: &'a [Value],
    at: usize,
}

impl<'a> Projection<'a> {
    fn new(row: &'a Value) -> Result<Self> {
        match row {
            Value::Record(record) => Ok(Self {
                fields: &record.fields,
                at: 0,
            }),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: unexpected row: {other:?}"
            ))),
        }
    }

    /// The next column's raw value.
    fn take(&mut self) -> Result<Value> {
        let value = self.fields.get(self.at).cloned().ok_or_else(|| {
            DatabaseError::Generic(format!(
                "profiles_query: projection row has {} columns, column {} wanted",
                self.fields.len(),
                self.at + 1
            ))
        })?;
        self.at += 1;
        Ok(value)
    }

    /// The next column, decoded through `T`'s own `Load` impl — the same one
    /// the typed reads use, so an enum or scalar can never decode differently
    /// here than it does on the ORM path.
    fn next<T: Load<Output = T>>(&mut self) -> Result<T> {
        let column = self.at;
        let value = self.take()?;
        T::load(value).map_err(|e| {
            DatabaseError::Generic(format!(
                "profiles_query: projection column {column} ({}): {e}",
                PAGE_PROJECTION.get(column).copied().unwrap_or("?")
            ))
        })
    }

    fn next_i64(&mut self) -> Result<i64> {
        let column = self.at;
        match self.take()? {
            Value::I64(n) => Ok(n),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: projection column {column} ({}): expected an integer, got {other:?}",
                PAGE_PROJECTION.get(column).copied().unwrap_or("?")
            ))),
        }
    }

    fn next_opt_i64(&mut self) -> Result<Option<i64>> {
        let column = self.at;
        match self.take()? {
            Value::Null => Ok(None),
            Value::I64(n) => Ok(Some(n)),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: projection column {column} ({}): expected an integer or NULL, got {other:?}",
                PAGE_PROJECTION.get(column).copied().unwrap_or("?")
            ))),
        }
    }

    fn next_endpoint_id(&mut self) -> Result<EndpointId> {
        Ok(EndpointId::new(self.next_i64()?))
    }

    fn next_protocol_id(&mut self) -> Result<ProtocolId> {
        Ok(ProtocolId::new(self.next_i64()?))
    }

    fn next_opt_protocol_id(&mut self) -> Result<Option<ProtocolId>> {
        Ok(self.next_opt_i64()?.map(ProtocolId::new))
    }

    /// SQLite stores BOOLEAN as an INTEGER, which `Load` for `bool` rejects.
    fn next_opt_bool(&mut self) -> Result<Option<bool>> {
        Ok(self.next_opt_i64()?.map(|n| n != 0))
    }

    /// The next TEXT column, NULL as `None`.
    fn next_opt_string(&mut self) -> Result<Option<String>> {
        let column = self.at;
        match self.take()? {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            other => Err(DatabaseError::Generic(format!(
                "profiles_query: projection column {column} ({}): expected text or NULL, got {other:?}",
                PAGE_PROJECTION.get(column).copied().unwrap_or("?")
            ))),
        }
    }

    /// A JSON-array TEXT column (`ports`, `task_queue`): the stored text is
    /// what toasty itself wrote, so parsing it directly is exact.
    fn next_u16_vec(&mut self) -> Result<Vec<u16>> {
        let (column, text) = self.take_json_text()?;
        serde_json::from_str(&text).map_err(|e| {
            DatabaseError::Generic(format!(
                "profiles_query: projection column {column} ({}): invalid JSON array: {e}",
                PAGE_PROJECTION.get(column).copied().unwrap_or("?")
            ))
        })
    }

    fn take_json_text(&mut self) -> Result<(usize, String)> {
        let column = self.at;
        let text: String = self.next()?;
        Ok((column, text))
    }

    /// An epoch-seconds INTEGER timestamp column.
    fn next_ts(&mut self) -> Result<i64> {
        self.next_i64()
    }

    fn next_opt_ts(&mut self) -> Result<Option<i64>> {
        self.next_opt_i64()
    }
}

/// `endpoints` (11 columns, in [`PAGE_PROJECTION`] order).
fn decode_projected_endpoint(p: &mut Projection<'_>) -> Result<(Endpoint, Vec<IpAddr>)> {
    let id = p.next_endpoint_id()?;
    let host: String = p.next()?;
    let host_type: HostType = p.next()?;
    let port: u16 = p.next()?;
    let ports: Vec<u16> = p.next_u16_vec()?;
    let last_source: Option<String> = p.next()?;
    let manual_protocol_override = p.next_opt_protocol_id()?;
    let resolved_ips = p.next_opt_string()?.map_or_else(Vec::new, |text| {
        let mut ips: Vec<std::net::IpAddr> = text
            .split(',')
            .filter_map(crate::endpoint_ip::key_from_hex)
            .collect();
        // Sorted by address, not by whatever order the aggregate emitted: the
        // display order must be the table's order (IPv4 then IPv6, by
        // address), and an aggregate's order is not a contract.
        ips.sort_unstable();
        ips
    });
    let resolved_at = p.next_opt_ts()?;
    let created_at = p.next_ts()?;
    Ok((
        Endpoint {
            id,
            host,
            host_type,
            port,
            ports,
            last_source,
            manual_protocol_override,
            resolved_at,
            created_at,
            links: Deferred::default(),
            group_links: Deferred::default(),
        },
        resolved_ips,
    ))
}

/// `profile_stats` (22 columns, in [`PAGE_PROJECTION`] order). The three
/// multi-column embeds (`latency`, `error`, `traffic`) are flattened in the
/// table, so they are reassembled by hand — the parity test against
/// [`Database::load_page_rows`] pins every one of them.
fn decode_projected_link(p: &mut Projection<'_>) -> Result<ProfileStats> {
    let protocol_id = p.next_protocol_id()?;
    let endpoint_id = p.next_endpoint_id()?;
    let core_type: CoreType = p.next()?;
    let config_type: ConfigType = p.next()?;
    let last_used_at = p.next_opt_ts()?;
    let last_seen_at = p.next_ts()?;
    let latency_kind: Option<String> = p.next()?;
    let latency_delay: Option<i64> = p.next()?;
    let latency_ip: Option<String> = p.next()?;
    let speed_bps: Option<i64> = p.next()?;
    let error_flag = p.next_opt_bool()?;
    let error_kind: Option<ProfileErr> = p.next()?;
    let error_text: Option<String> = p.next()?;
    let today_up: i64 = p.next()?;
    let today_down: i64 = p.next()?;
    let total_up: i64 = p.next()?;
    let total_down: i64 = p.next()?;
    let created_at = p.next_ts()?;
    let updated_at = p.next_ts()?;
    let version: u64 = p.next()?;

    let latency = latency_delay.and_then(|delay| {
        let delay = i32::try_from(delay).unwrap_or(i32::MAX);
        match latency_kind.as_deref() {
            Some("real") => Some(Latency::Real {
                delay,
                ip: latency_ip,
            }),
            Some("fast") => Some(Latency::Fast { delay }),
            _ => None,
        }
    });
    let error = match (error_flag, error_kind) {
        // `ErrorInfo.text` is not optional: the flag only ever accompanies a
        // text on the write paths, so a missing one is a corrupt row.
        (Some(true), Some(kind)) => Some(ErrorInfo {
            kind,
            text: error_text.unwrap_or_default(),
        }),
        (Some(true), None) => {
            return Err(DatabaseError::Generic(
                "profiles_query: link carries an error flag without a kind".to_string(),
            ));
        }
        _ => None,
    };

    Ok(ProfileStats {
        protocol_id,
        endpoint_id,
        core_type,
        config_type,
        last_used_at,
        last_seen_at,
        latency,
        speed_bps,
        error,
        traffic: TrafficStats {
            today_up,
            today_down,
            total_up,
            total_down,
        },
        created_at,
        updated_at,
        version,
        protocol: Deferred::default(),
        endpoint: Deferred::default(),
    })
}

/// `protocols` (10 columns, in [`PAGE_PROJECTION`] order). `None` when the
/// link's protocol row is absent: the join is LEFT and every `pr.*` column is
/// NULL together, exactly as [`Database::load_endpoint_rows`] drops it.
fn decode_projected_protocol(p: &mut Projection<'_>) -> Result<Option<Protocol>> {
    let id: Option<i64> = p.next()?;
    let sig: Option<i64> = p.next()?;
    let proto_kind: Option<ProtocolKind> = p.next()?;
    let transport_type: Option<TransportType> = p.next()?;
    let security_type: Option<SecurityType> = p.next()?;
    let sni: Option<String> = p.next()?;
    let fp: Option<String> = p.next()?;
    let insecure = p.next_opt_bool()?;
    let created_at: Option<i64> = p.next_opt_ts()?;

    let Some(id) = id else {
        debug_assert_eq!(
            [sig.is_none(), proto_kind.is_none()],
            [true; 2],
            "a LEFT JOIN nulls every protocol column together"
        );
        return Ok(None);
    };
    let (Some(sig), Some(proto_kind), Some(transport_type), Some(security_type), Some(created_at)) =
        (sig, proto_kind, transport_type, security_type, created_at)
    else {
        return Err(DatabaseError::Generic(
            "profiles_query: partial protocol row".to_string(),
        ));
    };
    Ok(Some(Protocol {
        id: ProtocolId::new(id),
        sig,
        proto_kind,
        transport: Transport {
            r#type: transport_type,
            data: Deferred::default(),
        },
        security: Security {
            r#type: security_type,
            sni,
            fp,
            insecure,
            data: Deferred::default(),
        },
        config: Deferred::default(),
        created_at,
        links: Deferred::default(),
    }))
}

impl Database {
    /// One page's rows in ONE statement: the endpoints, their links and the
    /// links' protocol display columns, joined and decoded positionally.
    ///
    /// The typed path ([`Database::load_page_rows`]) spends most of its time
    /// binding the page's ids into three `IN (...)` hydration reads — turso
    /// charges ~0.8 ms per bound parameter, and there are ~600 of them for a
    /// 200-row page (measured: 488 ms end to end, 29 ms for the same rows
    /// returned by a single inlined-id statement). The ids are therefore
    /// inlined as integer literals — the same rule
    /// [`crate::endpoint_rank::refresh`] follows — and the projection carries
    /// only display columns, so no deferred JSON is decoded.
    ///
    /// `ids` are page ids (they all have at least one link — the page source
    /// requires it), returned in the order given. The three deferred JSON
    /// carriers on `Protocol` (`transport.data`, `security.data`, `config`)
    /// come back unloaded: reading one is a bug, not a fallback, because the
    /// connect path re-reads its protocol through
    /// [`Database::load_protocol_with_config`].
    pub async fn load_page_projection(&self, ids: &[EndpointId]) -> Result<Vec<EndpointRow>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        sql.push("SELECT ");
        sql.push(&PAGE_PROJECTION.join(", "));
        sql.push(
            " FROM endpoints e JOIN profile_stats ps ON ps.endpoint_id = e.id \
             LEFT JOIN protocols pr ON pr.id = ps.protocol_id WHERE e.id IN (",
        );
        for (i, id) in ids.iter().enumerate() {
            if i > 0 {
                sql.push(", ");
            }
            sql.push(&id.get().to_string());
        }
        sql.push(")");
        let rows = sql.exec(&mut conn).await?;
        drop(conn);

        let mut endpoints: HashMap<EndpointId, (Endpoint, Vec<IpAddr>)> = HashMap::new();
        let mut links: HashMap<EndpointId, Vec<ProfileStats>> = HashMap::new();
        let mut protocols: HashMap<ProtocolId, Protocol> = HashMap::new();
        for row in &rows {
            let mut p = Projection::new(row)?;
            let (endpoint, resolved_ips) = decode_projected_endpoint(&mut p)?;
            let link = decode_projected_link(&mut p)?;
            let protocol = decode_projected_protocol(&mut p)?;
            links.entry(endpoint.id).or_default().push(link);
            endpoints
                .entry(endpoint.id)
                .or_insert((endpoint, resolved_ips));
            if let Some(protocol) = protocol {
                protocols.entry(protocol.id).or_insert(protocol);
            }
        }

        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            let Some((endpoint, resolved_ips)) = endpoints.remove(id) else {
                continue;
            };
            let links = links.remove(id).unwrap_or_default();
            let protocols = links
                .iter()
                .filter_map(|l| protocols.get(&l.protocol_id).map(|p| (p.id, p.clone())))
                .collect();
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
            row.select_best_measured_link();
            out.push(row);
        }
        Ok(out)
    }
}
