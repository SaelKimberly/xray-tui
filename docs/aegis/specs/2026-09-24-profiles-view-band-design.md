# Profiles view-band: kill the Active-view Address filesort — Design Spec

Status: **implemented (T1–T6): band schema/compute/index, page membership +
`rank_host` ordering, reband sweep, all-links purge rewrite — 116-test DB suite
+ 347-test touched-crate suite green. Pending: P0/P2 prereqs, release-scale A/B,
ADR retirement note.** (Decisions below are the user's, recorded 2026-09-24.)
Related: `docs/aegis/specs/2026-09-11-profiles-page-query-design.md` (the page
query), `adr/0001-raw-sql-profiles-page-query.md`, `adr/0003-stored-profile-ordering-keys.md`
(the rank keys this extends), `adr/0006-purge-evidence.md` (live-only
`rank_newest_seen`), `docs/database-manual-sql.md` (raw-SQL rule).
Evidence: `crates/xray-tui/src/ops/ping/flow_cost.rs` (commit `3a6a006`),
`dump-3.log` DB-monitor snapshots.

## 1. Problem

The Profiles page runs on the render task (events.rs: "one `profiles_page` +
`load_page_projection` on the UI task"). The dump (7.6k endpoints) shows
`profiles_page` p50 62 ms / p99 107 ms, and the slowest statement is the
**default Address sort** `endpoint_rank k JOIN endpoints e … ORDER BY e.host`.

`EXPLAIN QUERY PLAN` (reconstructed 30k schema) and the flow_cost lab both
prove the cause: the view filter is a **range** (`rank_newest_seen >= threshold`)
and the default order is a **different column** (`e.host`), so no B-tree can
serve both — SQLite sorts the whole filtered set every page (`USE TEMP B-TREE
FOR ORDER BY`). Cost scales with the Active set, not the page. At the target
"very large" feed (10–100×) this filesort is the freeze.

Lab signal (debug, N=1500, Active 832 — relative only): Address/Port filesort
~46 ms **flat across offsets**; Test (index sort) 4.4→10.1 ms **growing with
offset**; a `band=0` equality-seek 1.04 ms vs the Address filesort 27.2 ms
(~26×).

## 2. Grounding

- `rank_newest_seen` is the newest **LIVE** link's `last_seen_at` (ADR 0006,
  decision 16); it is the sole page membership input already.
- Every link write refreshes the endpoint's `endpoint_rank` row; `repair_missing`
  backfills link-bearing endpoints at open. Band is just another computed rank
  column — free on the write path.
- `PageSort::{Address, Port}` set `needs_endpoints` true and order by an
  `endpoints` column → both filesort identically. `PageSort::Ip` orders by a
  correlated `min(ip_key)` subquery → also filesorts.
- The rank-ordered sorts (Test/Speed/Traffic/ConfigType/LastSeen) do a covering
  **index scan** filtering `rank_newest_seen` inline (no filesort), but that
  scan grows with offset (P3) and is a full scan in the Purgatory view.

## 3. Decisions (from the grilling interview, with advisory corrections)

1. **Band membership.** Add a materialized `band` to `endpoint_rank`
   (`0=active, 1=not-active`), computed in the rank refresh from
   `rank_newest_seen` vs the `now − purgatory_ttl` threshold. **2 bands**
   (Q3=b): purge keeps its all-links semantics (see §3.5), so band never encodes
   purge-eligibility.
2. **Sole membership predicate (advisory-2).** `band=0` is the ONLY Active
   membership predicate for **every** sort and view. No query recomputes
   `rank_newest_seen >= threshold` — that would be a second spelling of "Active"
   (manual-sql §1) and would flip membership on sort change during drift.
3. **Ordering-index scope (Q2=a, advisory-6).** The filesort group is
   **Address, Port, Ip**. Pass 1 fixes **Address only** (the default, the proven
   hot path): denormalize `host` onto `endpoint_rank` as `rank_host` (kept fresh
   by the rank refresh — a `(band, host)` index cannot span two tables) and add
   `(band, rank_host, endpoint_id)`. **Port and Ip are deferred** (measure-then-
   expand); each needs its own `rank_port` / `rank_ip_min` column. The deferral
   is ordering-only — Port/Ip still filter `band=0`, so no split-predicate.
4. **Band freshness (Q1=a, advisory-1).** Band drift is monotonic (0→1 between
   writes; a write resets `rank_newest_seen` and sets band 0). The catch-up is a
   **directional index-range sweep** `UPDATE endpoint_rank SET band=1 WHERE
   band=0 AND rank_newest_seen < active_threshold`, backed by `(band,
   rank_newest_seen)` — continuity-independent (handles arbitrary downtime, not
   a fixed ±Δ window), bounded to the drifted rows. Run **at open before first
   render** and on the existing 10-min retention tick (Q4=a). No new task/cadence.
5. **Purge stays separate (Q3=b, advisory-3).** `purge_expired`'s `.all(last_seen
   < cutoff)` spans ALL links (incl. purged); `rank_newest_seen` is live-only —
   different facts. A fresh-but-purged link (subscription bumps `last_seen_at`,
   `purge_reason` persists — decision 22) must keep its endpoint. So band is NOT
   reused for purge. **P5** rewrites purge as an all-links `NOT EXISTS(fresh
   link)` (De-Morgan-equivalent to `.all()`, vacuous-true for linkless) with a
   parity test set `{linkless, all-stale, one-fresh, fresh-purged+stale-live}`.
6. **Prerequisites.** **P0** move the interactive page fetch off the render task
   (the `ProfilesRowsReady` generation path exists; make sort/view/search/page
   use it) so query cost is never a frozen frame. **P2** cache the filtered
   COUNT (invalidate on writes) and drop the count's needless `endpoints` JOIN
   when no search is active. P0/P2 are lower-risk and make P1b measurable rather
   than urgent.

## 4. Mechanism

- Schema (non-destructive — advisory 2026-09-24): `endpoint_rank` is a DERIVED,
  rebuildable table whose rank values are written by the single RAW path
  `endpoint_rank::write` and whose indexes are already raw at open. So `band`
  and `rank_host` are added as **raw `ALTER TABLE endpoint_rank ADD COLUMN`** in
  `ensure_in` (attempt + ignore the duplicate-column error), **NOT** as toasty
  model fields — `push_schema` is untouched, the `PRAGMA user_version` tag
  stays 13, and **no file wipe**. A large enriched/probed feed is preserved.
  Constraint from the code: `write` is `INSERT OR REPLACE` (full-row), so it
  MUST always emit `band`/`rank_host` (computed in the rank builder from the
  endpoint host + the `now − ttl` threshold) or a refresh would null them.
  One-time backfill: existing rows get `band`/`rank_host` filled once when the
  columns are freshly added (`band IS NULL` → a raw `UPDATE`, or `backfill_all`).
- Indexes (raw DDL at open, cause C2): `(band, rank_host, endpoint_id)` for the
  Active-Address seek; `(band, rank_newest_seen)` for the directional sweep.
- Ordering owner: `endpoint_rank::RankLink`/`compute_rank` computes `band` and
  copies `rank_host`; `profiles_query::order_terms(Address)` orders by
  `k.rank_host` (single-table, seekable) and `view_predicate` emits `band=0`
  for every sort. The Rust ordering law stays the sole owner (ADR 0003).
- Sweep: `Database::reband_expired` (raw), called from `Database::open` before
  first page and from the retention tick beside `purge_expired`.
- Purge: `Database::purge_expired` rewritten to the all-links `NOT EXISTS`
  statement (raw, cause C1/C6), replacing the ORM `.all()` quantifier (dump: 145
  ms).

## 5. Components / ownership

| Change | Owner | Cause / note |
| --- | --- | --- |
| `band`, `rank_host` columns | `endpoint_rank.rs` `ensure_in` (raw `ALTER ADD` + one-time backfill) | non-destructive: no model change, no tag bump, no wipe |
| band/rank_host compute | `endpoint_rank.rs` `RankLink`/`compute_rank`/`write` | one owner of the law |
| two indexes | `endpoint_rank.rs` `ensure_in` | C2 |
| `band=0` predicate + `rank_host` order | `profiles_query.rs` `view_predicate`/`order_terms` | sole membership |
| `reband_expired` sweep | `database.rs` (raw) + call sites `open`, retention tick (`main.rs`) | C6; continuity-independent |
| purge `NOT EXISTS` rewrite | `database.rs` `purge_expired` (raw) | C1; parity test |
| page fetch off render task (P0) | `ops/events.rs` / `ops/profiles.rs` | reuse `ProfilesRowsReady` |
| COUNT cache + JOIN drop (P2) | `profiles_query.rs` `profiles_count_with` + a cache field | — |

## 6. Acceptance (observable)

- Active + Address page shows **no `USE TEMP B-TREE`** in `EXPLAIN QUERY PLAN`;
  release flow_cost `band=0 seek` beats the Address filesort at N∈{7.6k, 50k,
  200k} (E3), and the win grows with N.
- Membership is identical across all sort columns in the Active view at any
  instant, including mid-drift (test: seed a row `band=0, rank_newest_seen <
  threshold`; every sort returns the same set until the sweep runs).
- After a simulated downtime gap ≫ tick, the at-open sweep corrects every
  drifted row before first render (test moves the clock, asserts band).
- `purge_expired` parity: the `NOT EXISTS` rewrite deletes exactly the ORM
  `.all()` set over `{linkless, all-stale, one-fresh, fresh-purged+stale-live}`.
- P0: a forced-slow page query never exceeds the render tick budget.

## 7. Risks
- **No data reset.** The columns are raw ALTER-adds on a rebuildable derived
  table, so an existing feed is preserved (advisory 2026-09-24). Cost moved
  from a wipe to a one-time backfill of `band`/`rank_host` at the upgrade launch.
- Sweep correctness under a moved clock is the load-bearing invariant — the
  downtime-gap test is mandatory.
- `band`/`rank_host` add write-path work on every link write (bounded — computed
  columns already traversed by the rank refresh).
- Port/Ip remain filesorts until their `rank_*` columns land — accepted, gated on
  measurement.
- **Threshold ownership (weighed 2026-09-24).** The band threshold is
  `now_epoch() − ttl`, with `ttl` a process value set once at startup (and on
  settings save) — NOT threaded per-write. This is production-correct (the
  threshold is a single global fact, set before the first page; no runtime
  race) but couples band to real wall-clock, so tests must use real-clock
  fixtures (relative to `now_epoch()`, day-scale margins) rather than synthetic
  timestamps. A reviewer preferred threading an explicit threshold through
  `write`/`refresh`/`backfill` (deterministic synthetic-time tests, no global);
  deferred as a larger refactor. Trigger to revisit: synthetic-time membership
  tests proliferate, or a per-connection ttl becomes needed.

## 8. Non-goals (pass 1)

- Port/Ip seek-ordering (`rank_port`/`rank_ip_min`) — deferred, measure first.
- Keyset pagination for the index sorts (P3) — separate, motivated by the
  offset-growth signal; not required for the filesort fix.
- Search FTS/trigram (P4) — debounce + P0 off-thread only for now.
- MVCC / STRICT / PRAGMA tuning — rejected with numbers (manual-sql §5).

## 9. ADR signal

Durable surfaces touched: the derived `endpoint_rank` shape (raw columns, no
tag bump), the page ordering owner, and the purge predicate. Warrants an ADR
(`0010 — Materialized Active band …`) recording the range-vs-order filesort
cause, the sole-membership rule, the 2-band / purge-separate decision, the
non-destructive raw-column choice over a tag-bump wipe, and the retirement
trigger (toasty aggregation / partial-index support). To be written on acceptance.
