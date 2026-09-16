# ADR 0002 — Write-behind `profile_stats` persistence

Date: 2026-09-11
Status: accepted — amended 2026-09-14 (the gate left the writer, and the groups narrowed; amendment 2: the existence probe's predicate), 2026-09-16 (amendment 4: the per-row UPDATE and the existence probe are retired — supersedes amendments 1 and 2; see the amendments at the end)
Supersedes: the per-write autocommit path for `profile_stats` batch workloads
Spec: `docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md`

## Context

A Fast+Real batch over 30k links made the TUI non-interactive and saturated the
NVMe. Measurements on the real engine (file-backed Turso):

| Path | `synchronous=FULL` (the default the app ran) | `synchronous=NORMAL` |
| --- | --- | --- |
| raw single-row commit | 4.217 ms | 0.219 ms |
| typed row upsert | 10.298 ms | 1.056 ms |
| narrow `UPDATE` in one transaction | — | 0.120 ms |

The batch issued **three commits per link** (result upsert, `schedule`,
`complete`), all awaited inside `poll_core_events` on the UI task, whose drain
budget is 256 events per tick. After setting `synchronous=NORMAL` the remaining
cost was still ~95 s of UI-task blocking per batch.

## Decision

1. `PRAGMA synchronous=NORMAL` on every connection (WAL semantics; the durability
   trade is a possible loss of the most recent transactions on power loss, with
   the database still consistent).
2. `ops/link_writer.rs` owns every `profile_stats` mutation. `stage()` records
   the row's changed column group and never awaits; one flush task writes a
   window as one transaction of narrow `UPDATE`s (200 ms / 512 rows), and batch
   end, quit, and the reload path flush explicitly.
3. **The writer's pending map is authoritative for the scheduler gate; the
   database is the durable mirror.** `SchedulerDb` reads through it and stages
   transitions. Every other reader of staged state must do the same.
4. Writers are disjoint by column group: RESULT (latency/speed/error), TASK
   (task_id/task_queue), TRAFFIC, ACTIVITY, SOURCE. No writer writes another's
   columns; `apply_link_patches` reads the row fresh inside the transaction and
   overlays only the patched groups, so a stale snapshot cannot clobber.
5. The subscription/import path is restricted to SOURCE, which also fixes a
   pre-existing lost update: an import refresh could overwrite ping results.

## Alternatives rejected

1. **Flush before every gate read** — reintroduces a commit per transition, i.e.
   the original cost.
2. **Keep the gate on the database and stage only result writes** — leaves two
   of three commits per link in place and adds a second writer to the same row.
3. **Sweep-then-flush on the reload path** — the flush would rewrite the error
   markers the TTL sweep just cleared.
4. **Snapshot-copy drain** — a `stage` landing during the write would be dropped;
   the drain removes its entries and merges back with `or_insert` on failure.

## Consequences

- Positive: batch persistence costs ~1 statement per changed row per window, off
  the UI task; a 256-event drain commits nothing (pinned by a test); the live
  "successful pings flow to top" reorder works because staged results are
  visible to reads.
- Negative: the gate's correctness depends on read-through — a reader that
  bypasses the writer reintroduces the hang (this happened: `BatchShared::read_link`
  read the database and the batch tests hung for >240 s).
- A row that does not exist is inserted by `apply_link_patches` (the old
  `upsert_link` contract), not silently skipped.
- Task state no longer needs per-transition durability; a crash is reconciled by
  the startup orphan sweep.
- Compatibility: no schema change. Durability window ≤ 200 ms of staged rows.

## Retirement trigger

None planned. Revisit if a future writer needs cross-process visibility of
`profile_stats` mutations, or if `flush_interval` proves too coarse.

## Amendment — 2026-09-14: the gate left the writer, and the groups narrowed

Invariants 3 and 4 changed; the rest stand.

- **The scheduler gate no longer reads through the writer** (invariant 3). Task
  state is runtime-only: `TaskScheduler.states` owns it, `profile_stats` lost
  `task_id`/`task_queue`, and the `SchedulerDb` trait — the seam invariant 3
  names — is deleted along with its Database/LinkWriter impls, its test mock,
  `update_scheduler_state` and `sweep_orphans`. The gate now holds no database
  handle, which is the stronger form of what invariant 3 was protecting: the
  ~60k per-batch reads (a connection + SELECT per transition) are impossible, not
  merely cheap, and an id the process does not know cannot be read back.
- **Column groups are RESULT and TRAFFIC** (invariant 4); TASK is gone with the
  columns. Both remaining groups can move a stored ordering key, so every patch
  refreshes its endpoint's keys (there is no TASK-only fast path left).
  `drain()` coalesces the staged groups into ONE patch per link, and
  `apply_link_patches` writes an existing row with a single literal `UPDATE`
  covering only that patch's groups (no per-row SELECT, no bound parameters; one
  existence probe per 400-row chunk) — the "reads the row fresh inside the
  transaction and overlays only the patched groups" behaviour is preserved by
  construction, and `tests/profiles_query.rs::link_patches_leave_columns_outside_their_groups_alone`
  pins it.
- `LinkWriter::read`/`overlay_pending`/`db_read_count` were deleted with their
  only caller: nothing reads a link through the writer any more.

## Amendment 2 — 2026-09-14: the existence probe's predicate is load-bearing

`apply_link_patches`' per-chunk existence probe must stay a `VALUES`-CTE join:

```sql
WITH pairs(p, e) AS (VALUES (1, 2), (3, 4), …)
SELECT ps.protocol_id, ps.endpoint_id FROM profile_stats ps
JOIN pairs ON ps.protocol_id = pairs.p AND ps.endpoint_id = pairs.e
```

The amendment above described "one existence probe per 400-row chunk" without
its shape, and the first implementation emitted a
`(protocol_id = .. AND endpoint_id = ..) OR …` chain. That chain is **left-deep**:
its expression depth grows one level per pair, so SQLite/turso reject it from ~99
pairs on (`SQLITE_MAX_EXPR_DEPTH` = 100) with `Parse error: Expression tree is
too large (maximum depth 100)`. `LINK_PATCH_CHUNK_ROWS` is 400 and the flush
window 512, so **every flush window wider than ~99 rows failed its whole
transaction**; `LinkWriter::flush` merged the failed chunk back and retried, so a
Fast-ping batch over three subscriptions persisted *nothing* (no ping result, no
traffic) and logged the failure every 200 ms until the window shrank. 90 pairs
still parse, which is why the failure needed a real batch to appear.

Measured on a 20k-link table, 400 pairs per statement (this engine, debug build):

| Predicate | Cost | Note |
| --- | --- | --- |
| `OR` chain, 90 pairs × 5 statements | 18 ms | index-driven; unscalable (dies at 99 pairs/statement) |
| row-value `(p, e) IN ((..), ..)` | 757 ms | parses at any width, but scans — not index-driven |
| `… IN (VALUES (p, e), ..)` | 30 s | the same scan, an order of magnitude worse |
| `UNION ALL` of 400 per-pair `SELECT`s | — | overflows turso's parser stack |
| cross-product `pid IN (…) AND eid IN (…)` | 40 ms | verdict stays exact (the set is built from returned rows), but it over-fetches |
| `VALUES`-CTE join (chosen) | 8 ms | flat at 5,000 pairs, exact pairs only, composite-key index |

The chosen shape's cost tracks the pair count, not the table: 400 pairs cost
8.5 ms at 20k rows and 7.7 ms at 50k rows, and 7.3 ms when the chunk's pairs are
all ABSENT (the fresh-import shape) — index probes in both directions, no
materialize-and-scan cliff. `endpoint_rank::write`'s bulk
`INSERT OR REPLACE … VALUES (…),(…)` was audited for the same class: it parses at
400 / 512 / 1,000 / 5,000 / 20,000 rows, so a `VALUES` list is depth-flat and
needs no chunking constraint either.

Pinned by two tests: `apply_link_patches_applies_a_window_wider_than_the_expression_depth_limit`
(a 500-row window — the class guard for any future predicate shape) and
`apply_link_patches_probe_is_exact_for_absent_and_near_miss_pairs` (the UPDATE vs
INSERT verdict at 140 rows, absent pairs included — the failure mode that is
silent, since an `UPDATE` matching zero rows drops the link without an error).

## Amendment 3 — 2026-09-15: the import's group, the flush remainder, the batch's own record

Evidence: the 09-15 Fast+Real batch (775 log lines) cross-checked against
`profile_stats` — 172 of 446 emitted results never reached the database, and 3 of
3 subscription refreshes ran while the batch was writing.

1. **The import is back inside the group contract.** `upsert_links_bulk` had
   drifted to a whole-row typed upsert, so an UPDATE wrote the fresh parse's
   `latency = NULL`, `error = NULL` and zeroed traffic over whatever the ping
   pipeline had just persisted (proved by a throwaway test, then by
   `import_refresh_preserves_result_and_traffic`). The update branch now carries
   `core_type`/`config_type`/`last_seen_at`/`updated_at`; the RESULT and TRAFFIC
   columns are written through `on_create`, where there is nothing to clobber —
   toasty's branch-specific upsert, so no existence probe and no second writer
   path. The spec's §4.1 table is the contract; this ADR amendment is why it is
   enforced in code rather than documented.
2. **A failed flush window re-stages the whole remainder.** `drain()` empties
   the pending map, and the flush loop returned at the first failing chunk —
   every later window in that drained batch was dropped from a local `Vec`. One
   chunk per batch (< 512 rows) hid it; a 30k-link batch did not.
   `a_failed_flush_window_restages_the_whole_remainder` (a held write lock =
   `database is locked`) fails on the old shape with `staged_len == 1` against
   the expected 3.
3. **Busy contention is retried at the batch boundary, not inside the patch
   writer.** `apply_link_patches` deliberately does NOT get a retry loop: the
   driver's `busy_timeout` is already 5 s per attempt and the background flush
   task re-attempts every 200 ms, so a stacked `retry_on_busy` would multiply a
   5 s stall by six. What was missing is the batch's own durability point, so
   `finish_batch` retries its final flush `FINAL_FLUSH_ATTEMPTS` times with a
   short backoff and the summary reports `flushes` / `staged-left`.
4. **The batch stages its own results.** The events handler writes results onto
   the loaded 200-row page and stages only what it finds there, so a page reload
   mid-batch (an import moves every endpoint's stored ordering key) dropped the
   rest with a `debug`-less log line identical to a persisted one.
   `BatchShared::stage_result` (one mapping, shared with the handler as
   `ops::events::apply_test_result`) stages RESULT from the batch's own snapshot;
   the handler keeps its staging for single pings and logs a page miss.
5. **Phase 2 no longer re-probes the unreachable.** A fast failure classified
   hard (timeout/refused/no-route/unresolvable, from `PingError` in the runner)
   retires the link's real task and leaves the `[fast]` marker: 253 of 350
   phase-2 probes in the measured run were re-tests of hosts phase 1 had already
   failed, ~226 s of the 555 s phase.

Records: one `batch summary` INFO line per batch (`summary_line`) and one
`startup` envelope; per-result success lines are `debug`, failures stay `warn`.
That is what the next investigation reads instead of 32k per-result lines.

## Amendment 4 — 2026-09-16: the per-row UPDATE and the existence probe are retired

Amendment 2's probe and its per-row `UPDATE` are both **gone**. Both writers now
emit one multi-row upsert per (statement chunk, `ON CONFLICT` action), so the
probe's predicate shape — and `SQLITE_MAX_EXPR_DEPTH` as a constraint on this
path — no longer exist. Superseded parts: amendment 1's "one existence probe per
400-row chunk" and "writes an existing row with a single literal `UPDATE`",
amendment 2 in full (its measurements stay as the record of why the per-row form
was abandoned), and invariant 4's statement shapes.

Why: `UPDATE … FROM (VALUES …)` is a parse error on this engine (measured
2026-09-16: `near "(": syntax error`), so the per-row form was **one statement
per link** and the probe was a separate statement per chunk. An
`INSERT … VALUES (…),(…) ON CONFLICT(protocol_id, endpoint_id) DO UPDATE SET …`
does the same work in one statement per chunk: the `VALUES` list is depth-flat
(amendment 2's audit of 400 / 512 / 1,000 / 5,000 / 20,000 rows), the action both
inserts a missing row and updates an existing one (so the probe's verdict is no
longer needed), and `excluded.<column>` gives each group its own expression.

The action is per-STATEMENT, so the patches are bucketed by the action they need
— both groups / RESULT / TRAFFIC / none — and a patch with no group gets
`DO NOTHING` (creates a missing row, writes no column). `contains` decides the
buckets, exactly as the per-row form's `groups.contains(...)` did, so an unknown
bit in `LinkGroups` cannot silently drop a group's columns. Invariant 4 stands as
stated: RESULT and TRAFFIC remain disjoint — an existing row is written with
exactly its patch's groups, and `last_used_at`, `last_seen_at` and `created_at`
are written by their own owners only (the `VALUES` tuple carries them for the
CREATE case, where there is nothing to clobber).

Measured 2026-09-16 (reference feed: 7,656 endpoints / 15,312 links / 6,507
addresses, ~3.7 MB file, release build, same machine, same probe binary for the
before and after rows — `docs/aegis/specs/2026-09-16-db-claim-verification.md`):

| Path | Before (per-row + probe) | After (multi-row upsert) | Δ |
| --- | --- | --- | --- |
| `apply_link_patches`, 512-patch window (incl. the rank refresh) | 29.0 ms | 10.0 ms | **2.9×** |
| `upsert_links_bulk`, 2,000 links (incl. the rank refresh) | 360 ms | 44.3 ms | **8.1×** |

Statement width is NOT the lever it was assumed to be: the same upsert at 400 /
1,000 / 2,000 rows per statement measured 11.1 / 10.6 / 10.6 ms (within noise),
and the rank writer's `INSERT OR REPLACE` at 400 / 1,000 / 2,000 measured
11.4 / 11.5 / 11.3 ms. `LINK_STATEMENT_ROWS` therefore stays 400, and
`LINK_PATCH_CHUNK_ROWS` is renamed `LINK_STATEMENT_ROWS` because it now bounds
both writers' statements.

Pinned by the same two tests, renamed to drop the probe:
`apply_link_patches_applies_a_window_wider_than_one_statement_chunk` (a 500-row
window spanning two chunks and all four action buckets) and
`apply_link_patches_upserts_absent_and_near_miss_pairs_exactly` (the INSERT vs
UPDATE verdict at 140 rows, absent pairs included — the failure mode that is
silent, since a conflict target that never matches would try to insert a
duplicate row).
