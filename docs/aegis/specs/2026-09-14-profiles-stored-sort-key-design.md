# Profiles sort keys: materialize the per-endpoint ordering

Date: 2026-09-14
Status: implemented (2026-09-14, ADR 0003)
Supersedes (on accept): `2026-09-11-profiles-page-query-design.md` §paging
Related: ADR 0001 (`profiles_query` raw SQL owner), decision 16 (test-priority law), decision 4 (schema tag = wipe), decision 21 (page query)

## Problem (measured)

Page-boundary moves stall ~1.35 s on a 7,672-endpoint / 9,048-link database:

| Term | Cost | Cause |
| --- | --- | --- |
| representative-link window | ~800 ms | `ROW_NUMBER() OVER (PARTITION BY endpoint_id ORDER BY <weight>)` over every link |
| hydration id lists | ~450 ms | `in_list` binds ~0.8 ms/param (200 ids = 174 ms; the same statement with literals = 9.7 ms) |
| filtered count | 98 ms (was 860) | fixed: endpoint-level `EXISTS`, no window |

Ordering 200 rows by the same weight expression *without* the window costs 114 ms, and paging cost is flat across offsets. The window is the term to remove; the per-endpoint key is the only thing the window computes.

## Prototype (measured on the same database)

| Step | Cost |
| --- | --- |
| backfill of the rank table (window, once) | 1016 ms |
| index build (once) | 719 ms |
| **page query, index-driven @0** | **1.14 ms** |
| page query @4800 / @7400 | 4.44 / 6.49 ms |
| count over the rank table | 64 ms |
| query plan | `SCAN … USING COVERING INDEX` |

`last_seen_at` is stored as fixed-width ISO-8601 UTC text
(`2026-09-11T07:38:50.858130960Z`), so lexicographic order is chronological and
it can be copied into the rank row verbatim.

## Decision

Materialize each endpoint's ordering key as stored columns and index them, so a page is an index scan + `LIMIT`/`OFFSET` with no window and no correlated subqueries.

**The keys are computed in Rust, by the same function that is the parity
oracle** (`EndpointRow::sort_links_by_test_priority` / the display-link rule),
not by SQL. SQL only stores and indexes them. One authority for the law: the
page query, the panel order, and the parity golden all read the same
computation; SQL never re-derives it.

Keys per `PageSort` (one stored column each, all nullable — exactly like the rollups they mirror):

| Sort | Stored columns | Index |
| --- | --- | --- |
| Test (default) | `rank_dns`, `rank_weight`, `rank_seen`, `rank_protocol` | `(rank_dns, rank_weight, rank_seen DESC, rank_protocol, id)` |
| LastSeen | `rank_seen` | `(rank_seen, id)` |
| Speed | `rank_speed` | `(rank_speed, id)` |
| Traffic | `rank_traffic` | `(rank_traffic, id)` |
| ConfigType | `rank_config` | `(rank_config, id)` |
| Address / Port | none (real columns) | `(host, id)`, `(port, id)` |

`rank_*` mirror the existing definitions exactly: the representative link is
chosen by decision 16 (`eff_weight`, `last_seen_at DESC`, `protocol_id ASC`, with
the DNS-unresolved collapse), the display-link sorts use the spec's rule (manual
override → best measured link → sentinel).

The page query keeps its owner (`profiles_query.rs`) and its result shape
(`PageMeta`); only its `ORDER BY` changes from expressions to these columns. The
parity golden (7 sorts × both directions vs the Rust oracle) is the acceptance
gate and must stay green.

### Storage: new table, NOT a schema tag bump

`endpoint_rank(endpoint_id PRIMARY KEY, rank_dns, rank_weight, rank_seen,
rank_protocol, rank_speed, rank_traffic, rank_config)`, created by ONE
idempotent `CREATE TABLE IF NOT EXISTS` at `open()` — plus its indexes — and
backfilled when empty.

Rationale: decision 4 makes a tag bump a **wipe**, and AGENTS says never to
recommend one as a change mechanism. A new side table is additive, so the
current tag still describes the database it opens; `endpoints` is untouched.
This is the smallest possible reintroduction of the `ensure_column` shape the
project deliberately deleted, and it is bounded to one table + one backfill.

Backfill cost ≈ the current window query run once (~1 s at 7.7k), once per
database, at first open after upgrade.

### Maintenance owner

The stored keys are a cache of the decision-16 law, so they need ONE writer:

- `LinkWriter::flush` recomputes the affected endpoints' keys in the same
  transaction as the patches (`apply_link_patches` already knows the touched
  endpoint ids) — one statement per endpoint, indexed by `endpoint_id`.
- `clear_expired_errors` (typed bulk UPDATE, bypasses patches) recomputes keys
  for the endpoints it touched. It runs on load, before the page query, so the
  keys are fresh by the time they are read.
- Subscription import / group edits write `SOURCE`/endpoint rows, not links:
  they insert links (new endpoints) → key computed on insert.
- A key that is missing (`NULL`, e.g. an endpoint whose links were all deleted)
  sorts last, matching the oracle's sentinel bands.

Invariant: `rank_*` is derived state. Nothing else may write it; a future writer
that changes a link's latency/error/traffic/config MUST recompute the endpoint's
row (the parity/anti-drift test below catches a miss).

## Verification

1. Existing parity golden: `profiles_query::tests` order parity, 7 sorts × both
   directions, unchanged.
2. New: keys agree with the oracle for a fixture with ties (equal weights,
   equal `last_seen_at`, DNS-unresolved, manual override, no measured link).
3. New: a write path (staged result patch, error sweep, import) leaves the keys
   equal to a full recompute.
4. Measured before/after at 7.7k endpoints + 40k synthetic: page fetch.
5. Full workspace suite + clippy 0 + fmt.

## Compatibility

- **No wipe, no tag bump.** Tag 7 databases open unchanged; the new table is
  added beside them. Existing rows are backfilled on first open.
- Rollback: dropping `endpoint_rank` (or ignoring it) restores the previous
  behaviour, since the window path stays in the code until measurement proves
  the keys are cheaper.
- Hydration (`in_list`, ~450 ms) is NOT addressed here: it is a separate,
  independent term, tracked for a follow-up (single-bind `json_each` form or a
  toasty fix).

## Results (measured on the reference feed after implementation)

| Sort | page fetch (count + page) |
| --- | --- |
| Test (default, scroll path) | **8.6 ms** (was 1757 ms), `EXPLAIN` = `SCAN endpoint_rank USING COVERING INDEX` |
| LastSeen / Speed / Traffic / ConfigType | ~39 ms |
| Address / Port | ~166 ms |
| anchor | ~65 ms |

Implementation notes that differ from the draft above:

- The keys are computed **in Rust** (the oracle), not in SQL; SQL stores and
  reads them.
- The table is a first-class `toasty` model, created by `push_schema` under
  **schema tag 8** — a tag bump, which wipes the database (accepted: pre-alpha,
  and the keys rebuild from a re-import). The earlier additive raw-DDL shape is
  superseded.
- The view windows read a stored `rank_newest_seen` instead of an `EXISTS` over
  `profile_stats`: with the latter the planner sorted every row (221 ms) rather
  than walking the index (1 ms).
- Backfill + index build ≈ 4.6 s, once per database, at the first open.
- Hydration (~450 ms) is unchanged and is now the dominant term of a page
  fetch; tracked separately.
