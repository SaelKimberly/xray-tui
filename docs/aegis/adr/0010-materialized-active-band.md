# 0010 — Materialized Active band: the page's view membership is a stored column, not a per-query threshold

Status: accepted (2026-09-24)
Related: `specs/2026-09-24-profiles-view-band-design.md`, ADR 0001 (raw page
query), ADR 0003 (materialized ordering keys), ADR 0006 (live-only
`rank_newest_seen`), `docs/database-manual-sql.md`.

## Context

The Profiles page runs on the render task. On the reference feed (~7.6k
endpoints) `profiles_page` measured p50 62 ms / p99 107 ms, and the slowest
statement was the **default Address sort**. `EXPLAIN QUERY PLAN` and the
`flow_cost` lab both showed the cause: the Active view filters a **range**
(`rank_newest_seen >= now − ttl`) while the default order is a **different
column** (`endpoints.host`). No B-tree can range-filter one column and emit
rows pre-sorted by another, so SQLite sorts the whole filtered set every page
(`USE TEMP B-TREE FOR ORDER BY`). Cost scales with the Active set, not the
page; at 10–100× the feed this filesort is a UI freeze.

## Decision

**Materialize the Active membership as a stored `band` column on the derived
`endpoint_rank` table, and make `band` the SOLE view-membership predicate for
every sort.** The default Address sort also reads a denormalized `rank_host`
column so its order is an index seek, not a cross-table filesort.

1. `band ∈ {0 active, 1 not-active}`, computed in the rank refresh from
   `rank_newest_seen` vs `now − purgatory_ttl`. **2 bands** — the page has one
   Active window; the retention/purge boundary is a separate concern.
2. **Sole membership.** Every view query uses `band = 0` / `band IS NOT 0`; no
   query recomputes `rank_newest_seen >= threshold`. A second spelling of
   "Active" would flip the membership set when the sort column changes during
   drift (manual-sql §1: never a second owner of a fact).
3. **Non-destructive schema.** `band`/`rank_host` are raw
   `ALTER TABLE endpoint_rank ADD COLUMN` at open (attempt + ignore the
   duplicate-column error), NOT toasty model fields — `push_schema` is
   untouched, the `user_version` tag stays 13, and **no file wipe**. A large
   enriched/probed feed is preserved. `endpoint_rank` is derived and rebuilt at
   open, so this is safe.
4. **Continuity-independent reband.** Band drift is monotonic (0→1 as `now`
   advances between writes). A directional index-range sweep
   `UPDATE … SET band=1 WHERE band=0 AND rank_newest_seen < threshold`
   (`reband_expired`, on `(band, rank_newest_seen)`) catches exactly the
   drifted rows regardless of downtime length; a full `CASE` reband
   (`reband_all`) runs once at startup where the configured ttl may differ
   from the open-time default in either direction. Run at startup before the
   first page and on the existing 10-min retention tick.
5. **Ordering scope: Address only (pass 1).** The filesort group is Address,
   Port, Ip; only Address (the default, the proven hot path) is fixed now, via
   `rank_host` + `(band, rank_host, endpoint_id)`. Port/Ip stay filesorts under
   `band=0` membership (correct, just unsorted-by-index) until measurement
   justifies their own `rank_port`/`rank_ip_min` columns.
6. **Purge stays all-links, separate from band.** `purge_expired` uses an
   all-links `NOT EXISTS(fresh link)` (De-Morgan of the old `.all()`), NOT
   band: `rank_newest_seen` is live-only (ADR 0006), so a fresh-but-purged link
   (a re-listed subscription) must keep its endpoint — band would wrongly
   reclaim it.

## Alternatives considered

- **Tag-bump + toasty model fields** (the first framing): rejected — it wipes
  the file (decision 4), destroying a large enriched feed, for columns a
  derived table can gain by raw ALTER.
- **`(rank_newest_seen, host)` composite index**: cannot avoid the filesort —
  a range on the leading column does not sort the second within the range.
- **Fixed ±Δ reband window**: rejected — assumes continuous uptime; a downtime
  gap > Δ leaves drifted rows stuck band=0. The directional sweep is gap-proof.
- **3 bands unifying purge onto `band=2`**: rejected — band keys off live-only
  `rank_newest_seen`, purge off all links; different facts (the fresh-purged +
  stale-live divergence).
- **Global-static ttl threaded into every write**: rejected for the per-write
  threshold; the ttl is a process value set once at startup (and settings
  save), read at write/sweep time.

## Consequences

- Active + Address page is an index seek, no temp b-tree (lab: band=0 seek
  1.04 ms vs Address filesort 27.2 ms at N=1500, debug — ~26×; release
  N∈{50k,200k} pending).
- Membership is identical across sort columns at any instant (no mid-drift
  flip); `PageRequest.active_threshold` is now advisory (kept, unread).
- One extra `UPDATE` per rank write (maintains the raw columns), plus a cheap
  directional sweep each tick and one full reband at startup.
- Purge behaviour is unchanged (all-links, vacuous-true for linkless).
- Port/Ip still filesort until a future pass; deferred by measurement.

## Evidence

`flow_cost` `band A/B` + the Active sort matrix; `EXPLAIN QUERY PLAN` on the
reband/page shapes; tests `active_and_stale_windows`,
`purgatory_ids_match_assembled_rows_on_mixed_dataset`,
`reband_sweep_demotes_rows_that_drifted_during_downtime`,
`purge_expired_matches_all_links_semantics` (all green, 116-test DB suite).

Retirement trigger: toasty aggregation / partial-index support that lets the
page express membership + ordering typed in one statement.
