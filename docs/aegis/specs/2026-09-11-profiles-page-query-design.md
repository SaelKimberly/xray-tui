# Profiles Tab Page Query — Design Spec

Date: `2026-09-11`
Status: `proposed` (implementation requires user approval of this document)
Owner surface: `crates/xray-tui-db` (new raw-SQL module), TUI callers in
`crates/xray-tui/src/{ops,ui}`

## 1. Context and measured problem

The Profiles tab loads every row of the typed schema, joins it in memory, and
holds it for the whole session. `Database::load_tab_rows` performs three
full-table scans (`endpoints`, `profile_stats`, `protocols`) and an in-memory
join, because neither a correlated `EXISTS` nor an `endpoint_id IN (…)` list is
viable at feed scale (documented in `database.rs`).

Measured on a copy of the live profile database (2026-09-11):

| Table | Rows |
| --- | --- |
| `endpoints` | 7,672 |
| `profile_stats` | 9,048 |
| `protocols` | 6,111 |
| `endpoint_groups` | 8,453 |

| Cost | Value |
| --- | --- |
| SQL for the three scans (deferred columns excluded) | 33 ms total |
| Deferred blobs never loaded (`config`, `security_data`, `transport_data`) | 1.77 MB + 625 KB + 189 KB |
| Per-frame `build_display_rows` over all filtered rows | ≈ 7.7k `DisplayRowData`, ~12 heap `String`s each, plus a deep clone on cache hit |
| `DataTable::render` row-height pass | one `Vec<u16>` over all rows per frame |

The SQL scan itself is not the dominant cost. The costs are (a) materializing
~23k model structs per load, and (b) rebuilding (or deep-cloning) the whole
display-row vector per frame while only the viewport is drawn.

## 2. Goal and acceptance criteria

Load the Profiles tab as a bounded page produced by SQL, with ordering,
filtering, counting, and pagination performed by the database, while keeping
the decision-14 row model (one row per endpoint with an expandable per-protocol
panel).

Acceptance criteria (observable):

1. With a 7,672-endpoint database, opening the Profiles tab issues queries
   whose result sets are bounded by the page size (`LIMIT`), not by the table
   size; the footer reports the true filtered count.
2. Ordering by `Test` reproduces the decision-16 comparator exactly, for every
   tier, including the `last_seen_at` tiebreak inside the untested and error
   tiers. Verified by a golden test that compares the SQL order against the
   existing Rust comparator over a seeded fixture.
3. Filtering (`Active`/`Stale`/`All`, search text, group membership) is applied
   by SQL and yields the same endpoint sets as the current in-memory filter.
4. Navigating past the last row of the loaded window fetches the adjacent
   window; the selected endpoint stays selected across the boundary.
5. A batch/single test result that changes a row's weight re-anchors the
   window on the selected endpoint; the row moves to its new position.
6. Test All, select-all, remove-failed, and enrichment seeding operate on the
   whole filtered view (not just the loaded window) and behave as today.
7. No schema change: reopening an existing `data.db` does not delete it.

## 3. Non-goals

- No link-centric (star) row model for the tab; the endpoint row and its panel
  stay.
- No new columns on `endpoints`/`profile_stats`; no `SCHEMA_VERSION` bump; no
  database wipe.
- No dependency on toasty issue #421 (typed aggregation). The typed path
  remains the eventual replacement for this module; the module's API shape is
  chosen so a typed implementation can satisfy it later without touching the
  TUI.
- No new index. Measured: the grouped page query runs a full scan over
  `profile_stats`, a `GROUP BY` sorter over links, and an `ORDER BY` sorter over
  the grouped endpoint set; a `MIN`-per-group sorter is not removable by an
  index, and the existing FK indexes already cover per-endpoint lookups. Add an
  index only against a measured need (the named escalation is in §12).

## 4. Decisions (user-owned, resolved)

| Decision | Choice |
| --- | --- |
| Storage approach | Raw SQL page query through the existing `toasty::sql::{query,statement}` surface; no denormalized weight/rollup columns |
| Windowing | Fixed page window; crossing an edge fetches the adjacent page |
| Paging mechanism | `LIMIT`/`OFFSET` on the ordered grouped query. Keyset cursors are rejected: they cannot avoid the sort without a stored, indexed endpoint weight column (the denormalization this design removes), and OFFSET was measured flat (8.3–9.3 ms at offsets 0 → 7,472 on the live dataset, 14.3 ms with the active-link tiebreak term), because the sort dominates either way |
| Re-anchoring | After a weight change the window is re-fetched at its offset; if the selected endpoint is not in the returned page, `anchor()` locates its offset with one extra ordered-count query |
| `SortColumn::Core` | Removed from the sort cycle (the `SortColumn::Core` variant and its comparator arm are deleted; the Core column stays display-only) — runtime `protocol_core_overrides` cannot be expressed in SQL |
| Database wipe | Not required by this design (no schema change) |
| Page size | Fixed, defined as a constant in the TUI layer (initial value 200 endpoint rows) |

## 5. Architecture: the raw-SQL owner

New module `crates/xray-tui-db/src/profiles_query.rs` is the single owner of
raw SQL in the workspace. It holds:

- the weight expression template (one source, rendered qualified and
  unqualified),
- the page / count / panel / whole-set queries,
- the `toasty_core::stmt::Value` decoding for those queries.

Rules for this module (enforced by review, asserted by tests):

- Only `?N` placeholder binds; user-controlled strings never interpolated.
- The module may reference only the column names defined in
  `models_toasty.rs`; a drift test executes each statement against a pushed
  schema, so a renamed or removed column fails the suite instead of the
  running TUI.
- The module's hand-written surface stays minimal: it *decodes* only
  `endpoints.id` and the ordering keys it needs for `anchor()`, and it
  *predicates* on `endpoints.host`/`port`/`host_type`/`resolved_as`,
  `profile_stats.*` weight columns, `profile_stats.last_seen_at`, and
  `endpoint_groups.group_id`. All model hydration stays on the typed path
  (`load_page_rows` → `load_endpoint_rows`), so no `Value`-level decoding of
  JSON columns, timestamps, or embed structs exists anywhere.
- Every returned row is converted into a typed model at the boundary; no
  `Value` escapes the module.

`Database` gains thin typed wrappers so callers never see SQL.

## 6. Query contract

```rust
pub struct PageRequest {
    pub view: PurgatoryView,          // Active | Stale | All
    pub active_threshold: Timestamp,
    pub stale_threshold: Timestamp,
    pub search: Option<String>,       // endpoint host / port substring
    pub group_id: Option<String>,
    pub sort: PageSort,               // Test | Address | Port | LastSeen | Speed | Traffic | ConfigType
    pub ascending: bool,
    pub offset: usize,                // window start, 0-based
    pub limit: usize,                 // page size
}

pub struct PageMeta {
    pub ids: Vec<EndpointId>,         // page order
    pub total: u64,                   // filtered count (footer: "rows X-Y of N")
    pub offset: usize,                // effective offset after clamping
}
```

- `page(db, req) -> PageMeta` — endpoint ids in display order, the filtered
  total, and the effective offset.
- `anchor(db, req, endpoint_id) -> Option<usize>` — the offset that endpoint
  currently occupies in the ordered view (used when a re-sort moves the
  selected row outside the loaded window).
- `load_page_rows(db, ids) -> Vec<EndpointRow>` — assembles the page through
  the existing `load_endpoint_rows` path (bounded `endpoint_id IN (…)` list of
  page size; the 10k-id statement-parse blowup does not apply).
- `panel_links(db, endpoint_id) -> Vec<ProfileStats>` — decision-14 panel
  contents for one expanded endpoint, ordered by the same weight.
- `plan_all(db, filter) -> Vec<PlanLink>` — Test All / batch planning over the
  whole filtered view.
- `filtered_ids(db, filter) -> Vec<EndpointId>` — Ctrl+A.
- `failed_ids(db) -> Vec<EndpointId>` — remove-failed.
- `enrich_seed(db) -> Vec<EnrichSeed>` — endpoint, resolved cache, and the
  active protocol's SNI for the enrichment spawns.

Every `PageSort` column is SQL-expressible except `Core` (removed, §4).
`Test`, `Address`, and `Port` order by endpoint-level values. `LastSeen`,
`Speed`, `Traffic`, and `ConfigType` order by the active link's column, where
"active" is the manual override if set, else the best measured link (real
before fast, lowest delay) — computed either with a correlated
`… ORDER BY <active rule> LIMIT 1` subquery (the chosen form, §7.5) or with
`ROW_NUMBER() OVER (PARTITION BY ps.endpoint_id ORDER BY <active rule>) = 1`;
both were verified through the driver on 2026-09-11.

## 7. SQL details

### 7.1 Weight expression

One template constant, rendered twice (query-qualified and DDL-unqualified):

```sql
CASE WHEN {t}.error = 1 AND {t}.error_kind IN ('real', 'name') THEN 2147483645
     WHEN {t}.error = 1 AND {t}.error_kind = 'fast'             THEN 2147483646
     WHEN {t}.latency IS NULL                                   THEN 2147483644
     WHEN {t}.latency = 'real'                                  THEN {t}.latency_delay
     ELSE 1073741824 + {t}.latency_delay END
```

Band mapping, mirroring `EndpointRow::link_test_tier` exactly:

| Tier (decision 16) | Condition | Weight |
| --- | --- | --- |
| 0 real success | `latency = 'real'`, no error | `delay` (ascending = fastest first) |
| 1 fast success | `latency = 'fast'`, no error | `1073741824 + delay` |
| 2 untested | no latency, no error | `2147483644` |
| 3 real error | `error_kind IN ('real', 'name')` | `2147483645` |
| 4 fast error | `error_kind = 'fast'` | `2147483646` |

Note `error_kind = 'name'` shares the real-error band: `link_test_tier` maps
`ProfileErr::Real | ProfileErr::Name => 3` (a name-resolution failure surfaces
on a real attempt). A `name` row that also retains a stored `latency` still
orders as an error — fresh failures dominate stored successes.

`dynamic_weight` ascending reproduces the decision-16 tiers. DNS-unresolved is
not in this expression — it is endpoint state and enters the endpoint ordering
as a separate term over `endpoints.host_type`/`resolved_as`, so a DNS
resolution never rewrites link rows.

### 7.2 Page query shape

```sql
SELECT r.endpoint_id
FROM (
    SELECT ps.endpoint_id, ps.protocol_id, ps.last_seen_at,
           <dns_flag over e2>                    AS dns_flag,
           <eff_weight over e2, ps>              AS eff_weight,
           ROW_NUMBER() OVER (
               PARTITION BY ps.endpoint_id
               ORDER BY <eff_weight> ASC, ps.last_seen_at DESC, ps.protocol_id ASC
           ) AS rn
    FROM profile_stats ps JOIN endpoints e2 ON e2.id = ps.endpoint_id
) r
WHERE r.rn = 1
  AND <view / search / group predicates, as EXISTS subqueries>
ORDER BY <sort terms>        -- see 7.5
LIMIT ?limit OFFSET ?offset
```

`endpoints` is joined *inside* the window subquery (as `e2`), because the DNS
term and `eff_weight` read endpoint columns; a links-only subquery cannot see
them (the first probe attempt failed with `no such table: e`). The outer query
therefore filters through the self-contained `EXISTS` predicates of §7.3, and
the `rn = 1` row carries the representative-link tuple the `Test` sort orders by
— no per-row correlated subquery and no aggregate are needed for that sort.

Executed as a whole on 2026-09-11 against the real schema with bound
parameters, over a fixture that includes both divergence cases (an endpoint
whose manual override points away from its minimum-weight link, and one whose
measured link carries an error marker next to an untested sibling): the page
came back in representative-link order `[2, 1, 4, 5, 3, 6]` — real 10, real 30,
real 90, then the two untested endpoints by `last_seen_at` descending, then the
DNS-unresolved endpoint last. `GROUP BY`, `MIN`, the `CASE` ordering
expression, `ROW_NUMBER() OVER (PARTITION BY … ORDER BY …)`,
`LIMIT`/`OFFSET`, and correlated subqueries were each verified separately
through the same driver. The count query is the same `FROM`/`WHERE` without
`ORDER BY`/`LIMIT`, wrapped in `COUNT(*)`.

Paging is by offset, not by cursor. A keyset cursor would only pay off if the
planner could reach the ordered rows through an index; with the ordering
computed from `profile_stats` aggregates it cannot, so the sort is paid either
way. Measured page cost on the live dataset: 8.6 ms (offset 0), 8.3 ms
(offset 5,000), 9.3 ms (offset 7,472); 14.3 ms with the active-link tiebreak
term.

### 7.3 Filters

- Active: `EXISTS (SELECT 1 FROM profile_stats p2 WHERE p2.endpoint_id = e.id
  AND p2.last_seen_at >= ?1)`.
- Stale: the same at `?stale` AND `NOT EXISTS` at `?active`.
- Search: `e.host LIKE '%' || ?1 || '%'`, matching the current behaviour
  (`host` only; port matched when the query is numeric). `%` and `_` in the
  user input are escaped with an explicit `ESCAPE` clause.
- Group: `EXISTS (SELECT 1 FROM endpoint_groups eg WHERE eg.endpoint_id = e.id
  AND eg.group_id = ?1)`.

### 7.4 Timestamp comparison constraint

`last_seen_at`, `resolved_at`, and `created_at` are stored as RFC3339 text.
Observed values are fixed-width 30-character nanosecond UTC strings
(`2026-09-11T07:38:50.858130960Z`), so lexicographic order equals chronological
order. jiff trims trailing zeros, so a whole-second timestamp would render
shorter and break that invariant (`…:50Z` sorts after `…:50.858Z`). Bind values
are therefore formatted to the same fixed 30-character shape, and a tripwire
test asserts the rendered width is constant for every timestamp the code
writes.

### 7.5 Sort terms

Each sort column defines one ORDER BY term list. `ascending` from the UI
appends `ASC`/`DESC` to the *primary* term only; the tiebreak terms keep their
fixed direction so paging stays total and deterministic.

| `PageSort` | ORDER BY terms (ascending case) |
| --- | --- |
| `Test` | `<dns_flag> ASC, r.eff_weight ASC, r.last_seen_at DESC, r.protocol_id ASC, e.id ASC` — `r` is the *representative* link (below) |
| `Address` | `e.host ASC, e.id ASC` |
| `Port` | `e.port ASC, e.id ASC` |
| `LastSeen` | `<active last_seen_at> ASC, e.id ASC` |
| `Speed` | `COALESCE(<active speed_bps>, -1) ASC, e.id ASC` |
| `Traffic` | `<active total up + down> ASC, e.id ASC` |
| `ConfigType` | `<active config_type rank> ASC, e.id ASC` |

- `<dns_flag>` = `CASE WHEN e.host_type = 'dns' AND e.resolved_as = '[]' THEN 1
  ELSE 0 END` — unresolved endpoints sort last, matching the decision-16 tier-5
  band, and this matches the current comparator which folds the flag into every
  link's key.
- **The `Test` sort's second, third, and fourth terms come from the
  representative link `r`, not from the display-preference `active` link.**
  Decision 16's endpoint key is `min` over links of
  `(tier, latency, -last_seen_at, protocol_id)` — the link attaining that
  minimum, which is a different link from `active_link()` whenever the
  minimum-weight link is an error or untested row while a sibling carries a
  measurement, and whenever `manual_protocol_override` is set. `r` is selected
  by
  `ORDER BY <dns_flag> ASC, eff_weight ASC, last_seen_at DESC, protocol_id ASC
  LIMIT 1`, where `eff_weight = CASE WHEN <dns_flag> THEN 0 ELSE <weight> END`
  (a constant inside the DNS band, so unresolved endpoints order among
  themselves by newest `last_seen_at` then `protocol_id`, exactly as
  `link_test_key` does with `tier = 5, latency = i32::MAX`). It is realised
  either as
  `ROW_NUMBER() OVER (PARTITION BY ps.endpoint_id ORDER BY …) = 1` over a
  derived table or as a correlated subquery per key column; both were verified
  through the driver.
- `<active …>` (all non-`Test` rows) selects the **display** link: the manual
  override if set, else the best measured link (real before fast, lowest
  delay), ordered by `(CASE WHEN error = 1 THEN 1 ELSE 0 END),
  COALESCE(latency_delay, 999999), last_seen_at DESC` with `LIMIT 1`. This is
  the same link the single-row display reads, so those sorts keep their current
  meaning.
- Nullable columns are wrapped in `COALESCE` with a sentinel so ordering and
  paging never depend on NULL placement.
- `e.id` is appended as the unique final term to every sort.
- `SortColumn::Core` has no entry: it is removed from the cycle (§4).

Sort semantics for non-`Test` columns ignore the DNS flag, exactly like the
current code, where only `SortColumn::Test` consults
`best_test_priority_key(…, dns_unresolved)`.

## 8. TUI changes

- `AppState.endpoints` becomes the loaded window; `state.page: PageMeta` holds
  the ids, the effective offset, and the filtered total.
- `filtered_profiles()` and `compute_filtered_indices` (in-memory filter and
  sort over the full set) are retired for the Profiles tab;
  `filter_cache_valid` is replaced by the page offset. `filtered_len()` reads
  `PageMeta.total`.
- `selected_index` indexes the loaded window; selection identity stays the
  endpoint id (`selected_profile_id`).
- Navigation: `Down` on the last row fetches the next page and selects its
  first row; `Up` on the first row fetches the previous page and selects its
  last row.
- Live re-sort: a test result that changes an endpoint's weight re-runs the
  page query anchored on the selected endpoint, so the row appears at its new
  position and the window follows it.
- Panel: expanding an endpoint issues `panel_links` for that endpoint only.
  `EndpointRow::sort_links_by_test_priority` loses its runtime callers and is
  retained solely as the test oracle for the SQL ordering (§11.1); the links
  arriving from `panel_links` are already in weight order.
  `select_best_measured_link` and `active_link` remain: the single-row display
  still prefers the best measured link, which is a display choice distinct from
  the weight ordering (a link may carry both a measurement and a later error
  marker).
- Rendering stays windowed; the per-frame `DisplayRowData` build is bounded by
  the page size.
- Retirement (clean cutover, no fallback path): `load_tab_rows`,
  `get_active_endpoints`, `get_stale_endpoints`, `get_active_endpoints_by_group`,
  `compute_filtered_indices`, and `filter_cache_valid` are deleted with their
  last caller. A failed page query logs and shows an empty window, matching
  today's error handling; there is no secondary load path.

## 9. Whole-set operations

Each operation gets a database query instead of an in-memory iteration:

| Operation | Today | After |
| --- | --- | --- |
| Test All (batch plan) | `plan_all_visible` over `filtered_profiles()` | `plan_all(db, filter)` |
| Ctrl+A select-all | `filtered_profiles()` ids | `filtered_ids(db, filter)` |
| Remove failed | iterate `state.endpoints` | `failed_ids(db)` |
| Enrichment seed | iterate `state.endpoints` | `enrich_seed(db)` |
| Footer count | `filtered_len()` | `PageMeta.total` |

Behaviour is unchanged from the user's perspective: these already covered
"everything", and in-memory iteration was only an artefact of the full load.

## 10. Project-rule change and ADR signal

`AGENTS.md` decision 4 states "ZERO raw SQL except PRAGMAs". This design adds a
second, bounded exception: `profiles_query.rs`, parameterised only, with a
schema-drift test. The amendment text for decision 4, the rationale, and the
retirement condition (delete the module once toasty #421 provides typed
aggregation and the page query is expressed typed) belong in an ADR —
`ADR signal: yes` (owner: `xray-tui-db`, affects AGENTS decision 4). The ADR is
written when the implementation lands, not before.

## 11. Verification

1. **Ordering golden** — for a seeded fixture covering every tier and tiebreak
   (real/fast success, `real`/`name`/`fast` errors, untested, DNS-unresolved,
   equal weights with different `last_seen_at`), assert the SQL page order
   equals the order produced by the existing `EndpointRow` comparator. Run it
   for **every** `PageSort` column, ascending and descending. The fixture MUST
   include the cases where the representative link and the display link
   diverge: an endpoint whose minimum-weight link is an error (or untested)
   while a sibling carries a measurement, and an endpoint with
   `manual_protocol_override` set to a non-minimum link. This is the contract
   that keeps the SQL expressions and the decision-16 law from drifting.
2. **Filter equivalence** — same fixture: SQL `page`/`count` results equal the
   current in-memory filter for each view, search term, and group.
3. **Drift guard** — every statement in the module executes against a pushed
   schema in CI, so a renamed column fails the build.
4. **Timestamp width tripwire** — rendered bind timestamps are fixed width.
5. **Paging stability** — a 5k-endpoint in-memory fixture: paging by offset
   visits every endpoint exactly once, in order, for each sort column; `total`
   matches; `anchor()` returns the offset the row actually occupies; the page
   fetch is timed to catch planner regressions.
6. **Re-anchor** — after mutating a row's weight (via the typed write path),
   `anchor()` finds it at its new offset and the re-fetched window contains it.
7. **TUI smoke** — launch the TUI (tui-test tools), verify the Profiles tab
   renders, `↓` past the window edge loads the next page, `o` cycles sort
   columns (no `Core`), `g` group filter and `/` search narrow the list, and
   expanding a row shows the panel.

## 12. Risks and tripwires

| Risk | Mitigation |
| --- | --- |
| Raw SQL drifts from the typed schema | Drift test (§11.3); single owner module |
| The weight CASE and the Rust comparator diverge | Golden test (§11.1); the Rust comparator is kept as the oracle, not as a runtime path |
| Mixed-precision timestamp text breaks range filters | Fixed-width binding + width tripwire (§7.4) |
| Planner change alters the query plan | The page query is a bounded scan plus a link-group sort and an endpoint sort; a regression shows up as page latency, timed in §11.5; no criterion target is added by this spec |
| Per-page cost grows with total links, not page size | Accepted and measured: 8.3–9.3 ms across offsets 0 → 7,472 (7,672 endpoints / 9,048 links), 14.3 ms with the tiebreak term. The win delivered is memory and per-frame work, not an asymptotic scan. Named escalation if that stops being acceptable: add an `endpoints.weight` column with an index on `(weight, id)`, maintained by SQL triggers over `profile_stats` — raw DDL plus a column the typed model does not know, added idempotently in `open()`. That reintroduces a second owner of the ordering law (now in triggers) and is explicitly deferred. Any index added later must be created with `CREATE INDEX IF NOT EXISTS` in `open()` on every open — `push_schema` is skipped whenever the `user_version` tag matches, so a one-shot DDL would never run against existing files |
| The endpoint-grouped page keeps a sorter | Accepted: the GROUP BY sorter runs over links, the ORDER BY sorter over the grouped endpoint set |
| `resolved_as` is a JSON text column | The DNS flag predicate compares it to `'[]'`, which is how the typed `Vec<String>` renders when empty (verified against the live database); the drift test covers it |

## Appendix A — Aegis working drafts

`TaskIntentDraft`

- Outcome: the Profiles tab reads a page from SQL; ordering, filtering,
  counting, and pagination are database-side; the decision-14 row model is
  preserved.
- Success evidence: acceptance criteria §2, verified by §11.
- Stop condition: all acceptance criteria verified; the in-memory full-load
  path retired.
- Non-goals: §3.
- Risks: §12.

`BaselineReadSetHint`

- Required: `AGENTS.md` (decisions 4, 11, 14, 16), `crates/xray-tui-db/src/database.rs`,
  `crates/xray-tui-db/src/models_toasty.rs`, `crates/xray-tui/src/ops/profiles.rs`,
  `crates/xray-tui/src/ui/profiles.rs`, `crates/xray-tui/src/ui/widgets/data_table.rs`,
  `crates/xray-tui/src/ops/{ping,enrich}.rs`.
- External: toasty 0.10 `src/sql.rs`, `src/stmt/paginate.rs`; turso_core 0.7.2
  `translate/{window,select,expression_index}.rs`; toasty issue #421.

`BaselineUsageDraft`

- Required baseline refs: listed above.
- Cited in design refs: decisions 4, 14, 16; `load_tab_rows` rationale;
  `link_test_tier`/`link_test_key`; `DataTable` windowing fields.
- Missing refs: none.
- Decision: continue.

`ImpactStatementDraft`

- Affected layers: `xray-tui-db` (new module, `Database` wrappers, `load_tab_rows`
  retirement), `xray-tui` ops (`profiles`, `ping`, `enrich`) and UI
  (`profiles`, `mod`).
- Owners: `xray-tui-db` owns the SQL and the ordering expression; the TUI owns
  window/paging state.
- Invariants: endpoint identity and uid scheme untouched; decision-14 row model;
  decision-16 ordering; no schema change.
- Retirement: `load_tab_rows` and the three full-scan read paths,
  `compute_filtered_indices`, `filter_cache_valid`, and the `SortColumn::Core`
  variant are deleted with this change; `link_test_key` survives as a test
  oracle only.
- Compatibility: existing `data.db` files reopen unchanged.
- Non-goals: §3.
