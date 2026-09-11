# Profiles Tab Page Query — Implementation Plan

Spec: `docs/aegis/specs/2026-09-11-profiles-page-query-design.md` (approved 2026-09-11)

## Goal

Replace the Profiles tab's whole-table load with a bounded SQL page window:
ordering, filtering, counting, and paging move into the database, the
decision-14 endpoint row and its expandable per-protocol panel stay, and no
schema change (no `SCHEMA_VERSION` bump, no wipe).

## Architecture

- New `crates/xray-tui-db/src/profiles_query.rs` is the single owner of raw SQL
  outside the PRAGMAs. It builds parameterised `SELECT`s, decodes only ids and
  cursor keys, and never writes.
- `Database` exposes thin typed wrappers; callers never see SQL.
- Model hydration (endpoints, links, protocol summaries) stays on the existing
  typed path (`load_endpoint_rows`), so JSON columns, timestamps, and embed
  structs are never hand-decoded.
- The TUI holds one page window plus a page offset and the filtered total. The
  in-memory full-set filter/sort machinery is retired.

## Tech Stack

Rust 2024, toasty 0.10 (`toasty::sql::{query,statement}`, `?N` binds, Turso
driver), jiff timestamps, ratatui TUI, tokio.

## Baseline / Authority Refs

- `AGENTS.md` decisions 4 (raw SQL policy), 11 (identity), 14 (row model),
  15 (in-memory sync rule / no zebra), 16 (test-priority ordering).
- `docs/aegis/specs/2026-09-11-profiles-page-query-design.md` (contract).
- `docs/aegis/baseline/2026-09-11-initial-baseline.md`.
- Verified probes (2026-09-11): `GROUP BY`/`MIN`, `ROW_NUMBER() OVER
  (PARTITION BY …)`, `CASE` ordering, row-value comparison, `LIKE`,
  expression indexes, and the §7.2 derived-table shape (oracle-matched).

## Compatibility Boundary

- Existing `data.db` files reopen unchanged (no tag change, no new column).
- `CoreEvent` gains no variant; `EndpointRow` keeps its shape.
- Endpoint uid scheme, schedule gate, and `ProfileStats` writes are untouched.
- `SortColumn::Core` leaves the sort cycle (documented behavior change).

## TDD Route

```
TDD Route:
- Mode: off
- Decision: skipped
- Strict authority: not applicable
- Strict signals: persistence + contract parity exist, but no explicit
  user/project strict request and no recorded auto mode; replaying a strict
  cycle is not authorized by this plan
- Light eligibility: not applicable
- TDD-fit exception: the ordering contract is a *regression* surface — the
  existing Rust comparator is the oracle, so parity tests are written after the
  SQL exists and fail on any drift
- Test posture: post-change regression (golden parity, drift guard, paging)
- Reason: mode `off`; the plan prescribes proportional regression tests, not a
  RED/GREEN cycle
- Verification: `cargo nextest run -p xray-tui-db`, `cargo nextest run -p xray-tui`,
  `cargo clippy --workspace --all-targets`, `just quality-gate code`
```

## Execution Readiness View

```
Intent Lock: page-bounded Profiles tab load over SQL; decision-14 rows kept
Scope Fence: crates/xray-tui-db (new module + Database wrappers),
  crates/xray-tui/src/{ops/{profiles,ping,enrich,events},ui/{profiles,mod},state,types}
Baseline Lock: AGENTS.md decisions 4/11/14/15/16; approved spec
Approved Behavior: §2 acceptance criteria of the spec
Owner / Contract Constraints: profiles_query.rs owns raw SQL; typed path owns hydration
Compatibility Boundary: no schema/tag change; no CoreEvent additions
Retirement Boundary: load_tab_rows, get_active_endpoints(get_stale_endpoints,
  get_active_endpoints_by_group, compute_filtered_indices, filter_cache_valid,
  SortColumn::Core, the DNS-flip in-memory re-sort
Task Batches: T1-T2 (db), T3-T4 (window + nav), T5-T6 (whole-set ops + DNS),
  T7-T8 (sort column, docs, retirement sweep)
Test Obligations: oracle parity goldens, drift guard, timestamp tripwire,
  paging stability, merge sync, TUI smoke
Review Gates: after T2 (db API frozen), after T6 (behavior complete), before T8
Drift / Rewind Rules: parity golden failing => fix the SQL, never the oracle;
  a needed schema change => stop and return to spec
Evidence Required Before Completion: green `just quality-gate code`, TUI smoke
  transcript, retirement grep showing no live caller of the retired symbols
Advisory Boundary: planning guidance only; not completion authority
```

## Change Necessity

```
Change Necessity:
- User-visible need: Profiles tab loads ~23k rows and rebuilds every display
  row per frame; the tab must scale with the viewport, not the feed
- No-change / non-code option: none — the cost is in the load and per-frame
  materialization, not configuration
- Why code change is necessary: filtering/sorting/paging must move into SQL;
  the typed API cannot express GROUP BY/window functions (toasty #421)
- Minimum change boundary: one new db module + typed wrappers + the tab's
  loader/navigation/whole-set call sites
- Decision: code-change
```

## Existence Check

```
Existence Check:
- Proposed new surface: crates/xray-tui-db/src/profiles_query.rs
- Existing owner / reuse candidate: crates/xray-tui-db/src/database.rs
  (typed query API, PRAGMA-only raw SQL)
- Why existing surface is insufficient: the page query needs GROUP BY /
  ROW_NUMBER / CASE ordering, which the typed API cannot express; putting raw
  SQL in database.rs would destroy its "typed facade, PRAGMA-only raw SQL"
  property and spread the schema knowledge the module rule contains
- Creation proof: spec §5 + approved decision; probe-verified shapes
- Entropy / retirement impact: one module; retire database.rs's three full-scan
  read paths and the TUI's in-memory filter/sort in the same change
- Decision: add-with-proof
```

## Architecture Integrity Lens

```
Invariant: one ordering law (decision 16) and one DNS-unresolved predicate
Canonical owner: xray-tui-db owns ordering SQL; Endpoint.resolved_as owns the
  DNS-unresolved fact; AppState owns the window position
Responsibility overlap: none new — the in-memory comparator becomes a test
  oracle only, and no rollup/weight column is introduced
Higher-level simplification: the tab's filter+sort+count+page collapse into one
  SQL query instead of four in-memory passes
Retirement / falsifier: if parity tests fail, the SQL is wrong, not the
  comparator; if a storable weight becomes necessary, return to the spec's
  named escalation instead of adding triggers ad hoc
Verdict: proceed
```

## Plan Pressure Test

```
Owner / contract / retirement: owner named; retirements listed per task
Architecture integrity / higher-level path: no higher-level owner skipped
Verification scope: parity + drift + paging + merge sync + TUI smoke
Task executability: each task names files, signatures, and commands
Pressure result: proceed
```

## Complexity Budget

```
Artifact class: module + call-site migration
Target files / artifacts: 1 new file (~350 lines), ~8 edited files
Current pressure: database.rs 2.4k lines, ops/profiles.rs 1.8k, ui/profiles.rs 1.6k
Projected post-change pressure: database.rs loses ~120 lines (three read paths),
  ops/profiles.rs loses the filter machinery (~60 lines) and gains page plumbing
Budget result: within-budget
Planned governance: new logic goes to profiles_query.rs; edited files only lose
  code or swap a call
```

## Files

| File | Action |
| --- | --- |
| `crates/xray-tui-db/src/profiles_query.rs` | create — raw-SQL owner |
| `crates/xray-tui-db/src/lib.rs` | modify — export the module |
| `crates/xray-tui-db/src/database.rs` | modify — wrappers, `load_page_rows`, retire `load_tab_rows`/`get_active_endpoints`/`get_stale_*`/`get_active_endpoints_by_group` |
| `crates/xray-tui-db/src/models_toasty.rs` | modify — demote the comparator to a test oracle (no runtime callers) |
| `crates/xray-tui-db/tests/integration.rs` | modify — retire/replace the removed-path tests |
| `crates/xray-tui/src/state.rs` | modify — page state fields |
| `crates/xray-tui/src/ops/profiles.rs` | modify — page loader, retire filter machinery, migrate ops |
| `crates/xray-tui/src/ops/ping.rs` | modify — `plan_all_visible`, `remove_failed_servers` |
| `crates/xray-tui/src/ops/enrich.rs` | modify — seed from the DB |
| `crates/xray-tui/src/ops/events.rs` | modify — DNS row patch, awaited flush, drop flip re-sort |
| `crates/xray-tui/src/ui/profiles.rs` | modify — windowed rows, count footer |
| `crates/xray-tui/src/ui/mod.rs` | modify — page-boundary navigation, Ctrl+A, retire `SortColumn::Core` from the cycle |
| `crates/xray-tui/src/types.rs` | modify — remove `SortColumn::Core` |
| `TUI_MANUAL.md`, `AGENTS.md` | modify — behavior + decision-4 amendment |

---

## T1 — Raw-SQL module

**Files**: create `crates/xray-tui-db/src/profiles_query.rs`; modify
`crates/xray-tui-db/src/lib.rs`; create
`crates/xray-tui-db/tests/profiles_query.rs`.

**Why**: one owner for the page/ordering SQL; the typed API cannot express it.

**Change Necessity**: code-change — the query is the deliverable.

**Impact/Compatibility**: read-only; no schema access changes.

**Steps**

1. Create the module with the full content below.
2. `crates/xray-tui-db/src/lib.rs`: add `pub mod profiles_query;` next to the
   other module declarations and re-export nothing else.
3. Create the test file with the code below.
4. Run `cargo nextest run -p xray-tui-db --test profiles_query` → all pass.

```rust
//! Raw-SQL page queries for the Profiles tab.
//!
//! This module is the ONLY owner of raw SQL in the workspace outside the
//! PRAGMAs in [`crate::database::Database::open`]/`conn`. Rules (see
//! `docs/aegis/specs/2026-09-11-profiles-page-query-design.md` §5):
//!
//! - parameterised `?N` binds only; user text is never interpolated,
//! - it decodes only `endpoints.id` and ordering keys, never model columns,
//! - it is read-only: it never writes `profile_stats`.

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
//   0 real success   -> delay                (ascending = fastest first)
//   1 fast success   -> 1_073_741_824 + delay
//   2 untested       -> 2_147_483_644
//   3 real|name error-> 2_147_483_645        (`Name` shares the real band)
//   4 fast error     -> 2_147_483_646
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

/// DNS-unresolved predicate, over the persisted column (one owner with the TUI
/// predicate).
fn dns_flag(alias: &str) -> String {
    format!(
        "CASE WHEN {alias}.host_type = 'dns' AND {alias}.resolved_as = '[]' THEN 1 ELSE 0 END"
    )
}

/// Effective weight inside the representative-link window: unresolved
/// endpoints collapse to a single band, matching `link_test_key` with
/// `tier = 5, latency = i32::MAX`.
fn eff_weight(alias: &str) -> String {
    format!(
        "CASE WHEN {alias}.host_type = 'dns' AND {alias}.resolved_as = '[]' THEN 0 ELSE {} END",
        weight("ps")
    )
}

// ── Request / response ──────────────────────────────────────────────────

/// Sort column of the Profiles tab. `Core` is absent by design: it resolves
/// through runtime `protocol_core_overrides` and cannot be expressed in SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSort {
    /// Decision-16 test priority (the representative link).
    Test,
    Address,
    Port,
    LastSeen,
    Speed,
    Traffic,
    ConfigType,
}

/// One page request. Thresholds are view filters, not ordering inputs.
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
    /// Filtered total across pages (footer count).
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
    fn new() -> Self {
        Self { text: String::new(), params: Vec::new() }
    }

    fn push(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn bind<T: Into<Value>>(&mut self, v: T) -> String {
        self.params.push(v.into());
        format!("?{}", self.params.len())
    }

    async fn exec(&self, conn: &mut toasty::Connection) -> Result<Vec<Value>> {
        let mut query = toasty::sql::query(&self.text);
        for param in &self.params {
            query = query.bind(param.clone());
        }
        Ok(query.exec(conn).await?)
    }
}

/// Fixed-width RFC3339 binding, matching the stored 30-character
/// nanosecond format (`2026-09-11T07:38:50.858130960Z`) so lexicographic
/// comparison equals chronological comparison.
fn sql_ts(ts: Timestamp) -> String {
    let secs = ts.as_second();
    let nanos = ts.subsec_nanosecond().unsigned_abs();
    let base = Timestamp::from_second(secs)
        .map(|t| t.to_string())
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    match base.strip_suffix('Z') {
        Some(head) => format!("{head}.{nanos:09}Z"),
        None => base,
    }
}

/// Escape LIKE metacharacters for an `ESCAPE '\'` clause.
fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// Projection of the page query: display order only needs the id.
const PROJ_ID: &str = "r.endpoint_id";
/// Projection of the anchor query: the full ordering key.
const PROJ_KEYS: &str =
    "r.dns_flag, r.eff_weight, r.last_seen_at, r.protocol_id, r.endpoint_id";

/// `SELECT <projection> FROM (<representative-link window>) r JOIN endpoints e
/// … WHERE r.rn = 1` plus the view/search/group predicates. Placeholder
/// numbering is implicit in bind order.
fn base_select(sql: &mut Sql, req: &PageRequest, projection: &str) {
    sql.push("SELECT ");
    sql.push(projection);
    sql.push(
        " FROM (\
           SELECT ps.endpoint_id, ps.protocol_id, ps.last_seen_at, ",
    );
    let dns = dns_flag("e2");
    sql.push(&dns);
    sql.push(" AS dns_flag, ");
    let eff = eff_weight("ps");
    sql.push(&eff);
    sql.push(
        " AS eff_weight, ROW_NUMBER() OVER (PARTITION BY ps.endpoint_id ORDER BY ",
    );
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

    if let Some(search) = &req.search {
        if !search.is_empty() {
            let pattern = sql.bind(format!("%{}%", escape_like(&search.to_lowercase())));
            sql.push(&format!(
                " AND (lower(e.host) LIKE {pattern} ESCAPE '\\' \
                   OR CAST(e.port AS TEXT) LIKE {pattern} ESCAPE '\\')"
            ));
        }
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
                      ps3.latency_delay ASC, ps3.protocol_id ASC LIMIT 1)\
         )"
    )
}

/// ORDER BY terms for a sort column. The primary term takes the UI direction;
/// the tiebreaks keep their own direction so paging stays total.
fn order_by(sql: &mut Sql, sort: PageSort, ascending: bool) {
    let dir = if ascending { "ASC" } else { "DESC" };
    match sort {
        PageSort::Test => {
            sql.push(&format!(
                " ORDER BY r.dns_flag {dir}, r.eff_weight {dir}, r.last_seen_at DESC, \
                  r.protocol_id ASC, r.endpoint_id ASC"
            ));
        }
        PageSort::Address => sql.push(&format!(" ORDER BY e.host {dir}, e.id ASC")),
        PageSort::Port => sql.push(&format!(" ORDER BY e.port {dir}, e.id ASC")),
        PageSort::LastSeen => {
            let expr = format!("COALESCE({}, '')", display_link("{t}.last_seen_at"));
            sql.push(&format!(" ORDER BY {expr} {dir}, e.id ASC"));
        }
        PageSort::Speed => {
            let expr = format!("COALESCE({}, -1)", display_link("COALESCE({t}.speed_bps, -1)"));
            sql.push(&format!(" ORDER BY {expr} {dir}, e.id ASC"));
        }
        PageSort::Traffic => {
            let expr = format!(
                "COALESCE({}, 0)",
                display_link("({t}.traffic_total_up + {t}.traffic_total_down)")
            );
            sql.push(&format!(" ORDER BY {expr} {dir}, e.id ASC"));
        }
        PageSort::ConfigType => {
            let expr = format!(
                "COALESCE({}, 2)",
                display_link(
                    "CASE {t}.config_type WHEN 'form' THEN 0 WHEN 'share_url' THEN 1 ELSE 2 END"
                )
            );
            sql.push(&format!(" ORDER BY {expr} {dir}, e.id ASC"));
        }
    }
}

fn decode_id(row: &Value) -> Result<EndpointId> {
    match row {
        Value::Record(record) => match record.fields.first() {
            Some(Value::I64(id)) => Ok(EndpointId::new(*id)),
            other => Err(DatabaseError::Generic(format!("unexpected id column: {other:?}"))),
        },
        other => Err(DatabaseError::Generic(format!("unexpected row: {other:?}"))),
    }
}

fn decode_count(row: &Value) -> Result<u64> {
    match row {
        Value::Record(record) => match record.fields.first() {
            Some(Value::I64(n)) => Ok(u64::try_from(*n).unwrap_or(0)),
            other => Err(DatabaseError::Generic(format!("unexpected count column: {other:?}"))),
        },
        other => Err(DatabaseError::Generic(format!("unexpected row: {other:?}"))),
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

    /// The filtered total (footer count).
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

    /// Offset the endpoint currently occupies in the ordered view. Used to
    /// re-anchor the window when a re-sort moves the selected row out of it.
    pub async fn profiles_anchor(
        &self,
        req: &PageRequest,
        endpoint_id: EndpointId,
    ) -> Result<Option<usize>> {
        let mut conn = self.connection().await?;
        let mut keys = Sql::new();
        base_select(&mut keys, req, PROJ_KEYS);
        keys.push(" AND r.endpoint_id = ");
        let target = keys.bind(endpoint_id.get());
        let rows = keys.exec(&mut conn).await?;
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        let (d0, w1, seen2, pid3) = decode_keys(row)?;

        let mut count = Sql::new();
        count.push("SELECT COUNT(*) FROM (");
        base_select(&mut count, req, PROJ_KEYS);
        count.push(") q WHERE ");
        let b0 = count.bind(d0);
        let b1 = count.bind(w1);
        let b2 = count.bind(seen2);
        let b3 = count.bind(pid3);
        let b4 = count.bind(endpoint_id.get());
        count.push(&format!(
            "q.k0 < {b0} OR (q.k0 = {b0} AND q.k1 < {b1}) \
             OR (q.k0 = {b0} AND q.k1 = {b1} AND q.k2 > {b2}) \
             OR (q.k0 = {b0} AND q.k1 = {b1} AND q.k2 = {b2} AND q.k3 < {b3}) \
             OR (q.k0 = {b0} AND q.k1 = {b1} AND q.k2 = {b2} AND q.k3 = {b3} AND q.k4 < {b4})"
        ));
        let rows = count.exec(&mut conn).await?;
        Ok(Some(usize::try_from(rows.first().map_or(Ok(0), decode_count)?).unwrap_or(0)))
    }

    /// Per-endpoint link order for the page's endpoints: the decision-16 order
    /// used by both the expanded panel and the sub-row navigation.
    pub async fn profile_link_order(
        &self,
        endpoint_ids: &[EndpointId],
    ) -> Result<HashMap<EndpointId, Vec<ProtocolId>>> {
        if endpoint_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let mut conn = self.connection().await?;
        let mut sql = Sql::new();
        sql.push("SELECT ps.endpoint_id, ps.protocol_id FROM profile_stats ps WHERE ps.endpoint_id IN (");
        for (i, id) in endpoint_ids.iter().enumerate() {
            if i > 0 {
                sql.push(", ");
            }
            let bind = sql.bind(id.get());
            sql.push(&bind);
        }
        sql.push(") ORDER BY ps.endpoint_id, ");
        let eff = eff_weight("ps");
        sql.push(&eff);
        sql.push(" ASC, ps.last_seen_at DESC, ps.protocol_id ASC");

        let rows = sql.exec(&mut conn).await?;
        let mut out: HashMap<EndpointId, Vec<ProtocolId>> = HashMap::new();
        for row in &rows {
            let Value::Record(record) = row else {
                return Err(DatabaseError::Generic(format!("unexpected row: {row:?}")));
            };
            let (Some(Value::I64(eid)), Some(Value::I64(pid))) =
                (record.fields.first(), record.fields.get(1))
            else {
                return Err(DatabaseError::Generic(format!("unexpected row: {row:?}")));
            };
            out.entry(EndpointId::new(*eid))
                .or_default()
                .push(ProtocolId::new(*pid));
        }
        Ok(out)
    }

    /// Order one endpoint's already-loaded links by [`Self::profile_link_order`].
    pub fn order_links(
        links: &mut Vec<ProfileStats>,
        order: Option<&Vec<ProtocolId>>,
    ) {
        let Some(order) = order else { return };
        let rank = |pid: ProtocolId| order.iter().position(|p| *p == pid).unwrap_or(usize::MAX);
        links.sort_by_key(|l| rank(l.protocol_id));
    }
}
```

The anchor query also needs a decoder for its key row:

```rust
/// Decode the anchor query's key row: `(dns_flag, eff_weight, last_seen,
/// protocol_id)`.
fn decode_keys(row: &Value) -> Result<(i64, i64, String, i64)> {
    let Value::Record(record) = row else {
        return Err(DatabaseError::Generic(format!("unexpected row: {row:?}")));
    };
    match (
        record.fields.first(),
        record.fields.get(1),
        record.fields.get(2),
        record.fields.get(3),
    ) {
        (
            Some(Value::I64(dns)),
            Some(Value::I64(weight)),
            Some(Value::String(seen)),
            Some(Value::I64(pid)),
        ) => Ok((*dns, *weight, seen.clone(), *pid)),
        other => Err(DatabaseError::Generic(format!("unexpected key row: {other:?}"))),
    }
}
```

**Verification**: `cargo nextest run -p xray-tui-db --test profiles_query`.
The db-crate test file covers the **mechanics** (the ordering *parity* golden
lives in the TUI crate, where the comparators are — see T4):

```rust
// tests/profiles_query.rs

/// Fixture: real/fast success, real+name/fast errors, untested, dns
/// unresolved, dns resolved, a manual override pointing away from the
/// minimum-weight link, and a measured link with an error marker beside an
/// untested sibling. Mirrors the verified probe.
async fn seed_fixture() -> Database { /* typed inserts + raw UPDATEs, as in the probe */ }

#[tokio::test]
async fn paging_visits_every_endpoint_exactly_once() {
    let db = seed_fixture().await;
    let mut seen = Vec::new();
    let mut offset = 0;
    loop {
        let req = PageRequest { offset, limit: 2, ..request(PageSort::Test, true) };
        let page = db.profiles_page(&req).await.unwrap();
        assert!(page.total >= page.ids.len() as u64);
        if page.ids.is_empty() { break; }
        seen.extend(page.ids.iter().map(EndpointId::get));
        offset += 2;
    }
    let mut expected: Vec<i64> = /* all fixture endpoint ids */;
    expected.sort_unstable();
    let mut got = seen.clone();
    got.sort_unstable();
    assert_eq!(got, expected, "every endpoint exactly once");
    assert_eq!(got.len(), expected.len());
}

#[tokio::test]
async fn count_matches_the_page_total_and_narrows_with_search() {
    let db = seed_fixture().await;
    let all = db.profiles_count(&request(PageSort::Test, true)).await.unwrap();
    assert_eq!(all, /* fixture endpoint count */);
    let searched = db.profiles_count(&PageRequest {
        search: Some("h1.".to_string()),
        ..request(PageSort::Test, true)
    }).await.unwrap();
    assert_eq!(searched, 1);
}

#[tokio::test]
async fn anchor_returns_the_rows_position_in_the_ordered_view() {
    let db = seed_fixture().await;
    let page = db.profiles_page(&request(PageSort::Test, true)).await.unwrap();
    for (position, id) in page.ids.iter().enumerate() {
        let anchored = db.profiles_anchor(&request(PageSort::Test, true), *id).await.unwrap();
        assert_eq!(anchored, Some(position), "anchor for {id:?}");
    }
}

#[tokio::test]
async fn link_order_puts_the_representative_first_and_covers_the_page() {
    let db = seed_fixture().await;
    let ids: Vec<EndpointId> = /* page endpoint ids */;
    let order = db.profile_link_order(&ids).await.unwrap();
    assert_eq!(order.len(), ids.len());
    assert!(order.values().all(|v| !v.is_empty()));
}

#[tokio::test]
async fn timestamps_bind_at_fixed_width() {
    for secs in [0, 1_700_000_000, 1_760_000_000, 4_000_000_000] {
        let rendered = sql_ts(Timestamp::from_second(secs).unwrap());
        assert_eq!(rendered.len(), 30, "{rendered}");
        assert!(rendered.ends_with('Z'));
    }
}

#[tokio::test]
async fn every_statement_runs_against_a_pushed_schema() {
    // Drift guard: a renamed or removed column must fail here, not in the TUI.
    let db = Database::in_memory().await.unwrap();
    let req = PageRequest { offset: 0, limit: 10, ..request(PageSort::Test, true) };
    db.profiles_page(&req).await.unwrap();
    db.profiles_count(&req).await.unwrap();
    db.profiles_anchor(&req, EndpointId::new(1)).await.unwrap();
    db.profile_link_order(&[EndpointId::new(1)]).await.unwrap();
    for sort in [PageSort::Address, PageSort::Port, PageSort::LastSeen,
                 PageSort::Speed, PageSort::Traffic, PageSort::ConfigType] {
        let req = PageRequest { sort, ..req.clone() };
        db.profiles_page(&req).await.unwrap();
        db.profiles_count(&req).await.unwrap();
    }
    for view in [PurgatoryView::Active, PurgatoryView::Stale, PurgatoryView::All] {
        let req = PageRequest { view, ..req.clone() };
        db.profiles_page(&req).await.unwrap();
    }
}
```

---

## T2 — Database wrappers and page hydration

**Files**: modify `crates/xray-tui-db/src/database.rs`.

**Steps**

1. Add `pub async fn load_page_rows(&self, ids: &[EndpointId]) -> Result<Vec<EndpointRow>>`:
   fetch the `Endpoint` rows for `ids` (`Endpoint` filtered by
   `stmt::in_list`), call the existing private `load_endpoint_rows`, then order
   each row's links with `order_links` using `profile_link_order(ids)`.
   Preserve the caller's id order (page order) — the current implementation
   sorts by id, which the page query already supersedes.
2. Add the whole-set queries used by T5, all built on `base_select`:

```rust
/// Every endpoint id in the filtered view, in page order (Ctrl+A).
pub async fn profiles_ids(&self, req: &PageRequest) -> Result<Vec<EndpointId>>;

/// `(endpoint_id, protocol_id)` for every link of the filtered view
/// (batch planning; the TUI hydrates the protocol rows typed).
pub async fn profiles_link_pairs(&self, req: &PageRequest) -> Result<Vec<(EndpointId, ProtocolId)>>;

/// Endpoints carrying a persisted failure marker on any link
/// (remove-failed).
pub async fn profiles_failed_ids(&self) -> Result<Vec<EndpointId>>;
// SQL: SELECT DISTINCT ps.endpoint_id FROM profile_stats ps WHERE ps.error = 1

/// Endpoints eligible for enrichment seeding (IP hosts and resolved DNS
/// hosts) across the whole view.
pub async fn profiles_enrich_seed(&self, req: &PageRequest) -> Result<Vec<Endpoint>>;
// SQL: SELECT r.endpoint_id FROM (…) r JOIN endpoints e … WHERE e.host_type <> 'undefined'
//      (typed hydration afterwards)

/// The display link of every filtered endpoint (active protocol per
/// endpoint), used to resolve the enrichment SNI.
pub async fn profiles_active_protocol_ids(
    &self,
    req: &PageRequest,
) -> Result<HashMap<EndpointId, ProtocolId>>;
// SQL: the display-link rule of `display_link`, projecting protocol_id.
```
3. Retire `load_tab_rows`, `get_active_endpoints`,
   `get_stale_endpoints`, `get_stale_ids`, `get_stale_count`,
   `get_active_endpoints_by_group` and their tests. `get_endpoint` /
   `get_endpoint_by_protocol_id` / `endpoints_by_parent` keep working.
4. Keep `clear_expired_errors` and the write methods unchanged.

**Verification**: `cargo nextest run -p xray-tui-db`.

---

## T3 — TUI page state and loader

**Files**: modify `crates/xray-tui/src/state.rs`,
`crates/xray-tui/src/ops/profiles.rs`.

**Steps**

1. `state.rs`: add

```rust
/// Loaded Profiles window (one SQL page).
pub struct ProfilesPage {
    pub ids: Vec<i64>,
    pub total: u64,
    pub offset: usize,
    pub sort: xray_tui_db::profiles_query::PageSort,
    pub ascending: bool,
}

pub page: ProfilesPage,
```

   Remove `cached_filtered_indices: RefCell<Vec<usize>>` and
   `filter_cache_valid: Cell<bool>`; keep `endpoints_gen` (dirty marker) and
   `reload_gen`.
2. `ops/profiles.rs`: `filtered_profiles(state)` becomes
   `state.endpoints.iter()`; `filtered_len(state)` returns
   `state.page.total as usize`; delete `compute_filtered_indices`,
   `config_type_rank` (its rank moves into the SQL CASE), and every
   `filter_cache_valid.set(...)` call (they become `state.page_dirty = true`
   or disappear — the page is re-fetched, not re-filtered).
3. `load_profiles_rows` returns the page: build a `PageRequest` from
   `ProfilesLoad` + `state.page`, call `db.profiles_page`,
   `db.load_page_rows`, and return `(rows, PageMeta)`.
4. `apply_profiles_rows(state, rows, meta)` writes `state.endpoints`,
   `state.page`, clamps `selected_index` into the window, and keeps the
   enrichment seeding calls.
5. `clamp_selection` clamps to the window length and clears `selected_sub`
   when the row is gone.

**Verification**: `cargo nextest run -p xray-tui` (unit tests in
`ops/profiles.rs` that build `EndpointRow` fixtures keep compiling; tests that
assert on `filter_cache_valid` are deleted with the field).

---

## T4 — Window navigation, re-anchor, panel order

**Files**: modify `crates/xray-tui/src/ui/mod.rs`,
`crates/xray-tui/src/ops/profiles.rs`.

**Steps**

1. `Down` on the last row of the window: set
   `selected_index = 0`, `state.page.offset += page.limit`, re-fetch; when the
   fetch returns empty, re-clamp to the previous page. `Up` on the first row:
   `state.page.offset = offset.saturating_sub(limit)`, fetch, select the last
   row.
2. `PageUp`/`PageDown`/`Home`/`End` keep their index arithmetic inside the
   window; `End` jumps to the last page (`offset = total - total % limit`).
3. After a ping result (`ops/events.rs`) or a weight-changing write, mark
   `state.page_dirty = true`. The reload path re-fetches the page; if the
   selected endpoint id is not in the returned ids, call `profiles_anchor` with
   the selection's request and set `state.page.offset` to that value, then
   re-fetch once.
4. Expand: the panel reads `row.links` already ordered by `order_links`; drop
   `sort_links_by_test_priority` and `select_best_measured_link` runtime calls
   — `active_link()` still resolves the display link from the loaded links
   (manual override first, else `selected_protocol`, which `load_page_rows`
   sets from the best measured link using the same rule as before).

**Verification**: `cargo nextest run -p xray-tui`; TUI smoke in T8.

**Parity golden (owns the decision-16 contract).** The comparators live in this
crate, so the oracle test lives here — `crates/xray-tui/src/ops/profiles.rs`
test module, using the same fixture as the db test:

```rust
#[tokio::test]
async fn sql_page_order_matches_the_rust_comparator_for_every_sort() {
    let state = fixture_state().await;          // AppState over an in-memory DB
    let oracle_rows = state.endpoints.clone();  // typed full read for the oracle
    for (column, page_sort) in [
        (SortColumn::Test, PageSort::Test),
        (SortColumn::Address, PageSort::Address),
        (SortColumn::Port, PageSort::Port),
        (SortColumn::LastSeen, PageSort::LastSeen),
        (SortColumn::Speed, PageSort::Speed),
        (SortColumn::Traffic, PageSort::Traffic),
        (SortColumn::ConfigType, PageSort::ConfigType),
    ] {
        for ascending in [true, false] {
            // Oracle: the pre-change comparator, unchanged in this task.
            let mut expected: Vec<i64> =
                oracle_indices(&state, &oracle_rows, column, ascending)
                    .into_iter()
                    .map(|i| oracle_rows[i].endpoint.id.get())
                    .collect();
            // SQL: the page query at limit = total.
            let req = PageRequest { sort: page_sort, ascending, offset: 0, limit: 10_000, ..base_req() };
            let page = state.db.profiles_page(&req).await.unwrap();
            let got: Vec<i64> = page.ids.iter().map(EndpointId::get).collect();
            assert_eq!(got, expected, "{column:?} ascending={ascending}");
        }
    }
}
```

`oracle_indices` is the pre-change `compute_filtered_indices` body (filter +
sort arms) kept **in the test module only**; the production function is deleted
in T3. Fixing a mismatch means fixing the SQL, never the oracle.

---

## T5 — Whole-set operations

**Files**: modify `crates/xray-tui/src/ops/{ping,enrich,profiles}.rs`,
`crates/xray-tui/src/ui/mod.rs`.

**Steps**

1. `ping.rs::plan_all_visible` → `db.profiles_plan(&req)` returning
   `Vec<PlanLink>` for the whole filtered view (links ⨝ endpoints; protocols
   loaded typed without config, as today).
2. `ping.rs::remove_failed_servers` → `db.profiles_failed_ids()`.
3. `enrich.rs::spawn_enrich_ip_hosts` / `spawn_outbound_countries` seed from
   `db.profiles_enrich_seed()` instead of `state.endpoints`.
4. `ui/mod.rs` Ctrl+A → `db.profiles_ids(&req)`.
5. Footer: render `rows X–Y of N` from `state.page`.

**Verification**: `cargo nextest run -p xray-tui`.

---

## T6 — DNS predicate unification

**Files**: modify `crates/xray-tui/src/ops/profiles.rs`,
`crates/xray-tui/src/ops/events.rs`.

**Steps**

1. `endpoint_dns_unresolved` reads the persisted column:

```rust
pub(crate) fn endpoint_dns_unresolved(_state: &AppState, row: &EndpointRow) -> bool {
    row.endpoint.host_type == HostType::Dns && row.endpoint.resolved_as.is_empty()
}
```

2. In the `EndpointInfoUpdated` merge arm, after the entry merge, patch the
   loaded row:

```rust
if let Some(entry) = state.endpoint_info.get(&endpoint_id)
    && let Some(row) = state.endpoints.iter_mut().find(|r| r.endpoint.id.get() == endpoint_id)
{
    row.endpoint.resolved_as = entry.resolved_ips.iter().map(ToString::to_string).collect();
    row.endpoint.resolved_at = entry.resolved_at_secs.and_then(|s| Timestamp::from_second(s).ok());
}
```

3. Replace the spawned flush with an awaited single-transaction write at the
   end of the pass:

```rust
if !persist_batch.is_empty() {
    let mut conn = state.db.connection().await?;
    let tx = ...; // toasty transaction over the pooled connection
    for (eid, resolved_as, resolved_at, ip_addrs) in persist_batch { /* typed writes */ }
    tx.commit().await?;
    state.page_dirty = true;
}
```

   (If toasty 0.10's transaction API cannot wrap `update_endpoint_resolution`,
   keep the awaited per-row writes in the same pass — the ordering guarantee is
   what matters, not the transaction.)

4. Delete the DNS-flip in-memory re-sort (`events.rs` ~line 551) and its
   `endpoint_dns_unresolved` import.

**Verification**: `cargo nextest run -p xray-tui` including the new merge-sync
test.

---

## T7 — Sort column and count display

**Files**: modify `crates/xray-tui/src/types.rs`,
`crates/xray-tui/src/ui/mod.rs`, `crates/xray-tui/src/ui/profiles.rs`.

**Steps**

1. Remove `SortColumn::Core` and its comparator arm; the `o` cycle keeps the
   other seven, mapping each to `PageSort` in one function
   (`fn page_sort(col: SortColumn) -> PageSort`).
2. Changing sort sets `state.page.offset = 0` and marks the page dirty.
3. Header/footer shows the filtered total and the current window range.

**Verification**: `cargo nextest run -p xray-tui`.

---

## T8 — Docs, ADR, retirement sweep, full verification

**Files**: modify `AGENTS.md`, `TUI_MANUAL.md`, add
`docs/aegis/adr/0001-raw-sql-profiles-page-query.md`, update
`docs/aegis/INDEX.md`.

**Steps**

1. AGENTS decision 4: amend "ZERO raw SQL except PRAGMAs" with the bounded
   exception (`profiles_query.rs`, parameterised only, read-only, drift-tested)
   and the retirement trigger (delete once toasty #421 makes the query
   expressible typed).
2. ADR: context, decision, alternatives (stored weight column + rollups;
   link-centric rows; wait for #421), consequences (per-page cost is a link
   scan + sorts), compatibility (no schema change), retirement trigger.
3. `TUI_MANUAL.md`: Profiles row model unchanged; note the removed `Core` sort
   option and the paging behavior.
4. Retirement sweep: `grep -rn "load_tab_rows\|get_active_endpoints\|get_stale_endpoints\|get_active_endpoints_by_group\|compute_filtered_indices\|filter_cache_valid\|SortColumn::Core" crates` returns only the oracle-comparator test and this plan.
5. Run `just quality-gate code`, then the TUI smoke: launch, sort by Test,
   scroll past a page boundary, expand a row, run a single ping, verify the row
   re-sorts and the window follows the selection.

---

## Risks and Rollback

| Risk | Mitigation / rollback |
| --- | --- |
| SQL ordering drifts from decision 16 | Parity golden per sort; fix the SQL, never the oracle |
| Raw SQL drifts from the schema | Drift test executing every statement on a pushed schema |
| Awaited DNS flush blocks a poll pass | Bounded by one batched write per pass; revert to the spawned flush if measured jank appears (the row patch stays) |
| `PageSort` coverage gaps | `page_sort` is an exhaustive match; `Core` removal is compile-checked |
| Rollback | Single-branch change: revert the merge commit; no schema or data migration exists to undo |

## Retirement Track

| Old path | Status | Trigger |
| --- | --- | --- |
| `load_tab_rows` and the three full-scan reads | delete in T2 | last caller migrates in T3 |
| `compute_filtered_indices` / `filter_cache_valid` | delete in T3 | SQL filtering lands |
| `sort_links_by_test_priority` runtime callers | delete in T4 | `order_links` supplies the order |
| DNS-flip in-memory re-sort | delete in T6 | page re-fetch covers it |
| `SortColumn::Core` | delete in T7 | approved decision |
| `profiles_query.rs` itself | keep, with retirement trigger | toasty #421 provides typed aggregation |

## ADR Signal (preserve for completion backfill)

- Decision: profiles page/order/filter SQL lives in one raw-SQL module; no
  denormalized weight or rollup columns; offset paging.
- Alternatives: `endpoints.weight` + index + triggers; link-centric rows;
  wait for toasty #421.
- Compatibility: existing databases reopen unchanged.
