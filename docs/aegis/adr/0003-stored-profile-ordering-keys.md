# ADR 0003 — Materialized per-endpoint ordering keys

Date: 2026-09-14
Status: accepted
Supersedes: ADR 0001 §paging (the SQL window core), decision 21 §paging
Spec: `docs/aegis/specs/2026-09-14-profiles-stored-sort-key-design.md`

## Context

The 2026-09-11 page query made the tab a SQL page but derived each endpoint's
position on every fetch: a `ROW_NUMBER() OVER (PARTITION BY endpoint_id ORDER
BY <weight>)` window over every link, plus correlated subqueries for the
display-link sorts, plus a second full evaluation of the same window for the
count. Measured on the reference feed (7,672 endpoints / 9,048 links):

| Query | Cost |
| --- | --- |
| page fetch (count + page) | 1757 ms |
| …of which the count | 860 ms |
| …of which the window core | ~800 ms |
| ordering the same rows by the same expression WITHOUT the window | 114 ms |
| `in_list` hydration (3 statements, 200 ids each) | ~450 ms |

Every page-boundary move in the TUI blocked the UI task for ~2.2 s. Two engine
facts drove the design: turso re-evaluates the whole window per page (there is
no index it could use), and it parses ~0.8 ms per bind parameter (200 ids =
174 ms; the same statement with inline literals = 9.7 ms).

## Decision

Store each endpoint's ordering keys in a side table and read the page through
an index.

- Table `endpoint_rank(endpoint_id PRIMARY KEY, rank_dns, rank_tier,
  rank_latency, rank_seen, rank_protocol, rank_display_seen, rank_speed,
  rank_traffic, rank_config, rank_newest_seen)` with two indexes: the Test
  order (the scroll path) and the view window.
- **Created additively** at `Database::open` (`CREATE TABLE IF NOT EXISTS` +
  indexes) — NOT via a schema-tag bump, because decision 4 makes a bump a full
  wipe. An older shape of the table is dropped and rebuilt: it is derived
  state, so the backfill is the only cost, and it needs no migration
  machinery.
- **The keys are computed in Rust**, by the same function that is the
  ordering law (`endpoint_rank::RankLink::key`, which
  `EndpointRow::link_test_key` now delegates to). SQL stores the numbers and
  reads them back; it never re-derives the law. One authority for the page
  order, the panel order, and the parity golden.
- The page source is the rank table; the view windows read
  `rank_newest_seen` ("some link is in the band" ⟺ "the newest link is"),
  which keeps the predicate off `profile_stats` — an `EXISTS` there forced the
  planner into a full sort.
- **Maintenance**: every write path that can change a link refreshes the
  endpoints it touched — `Database::upsert_link` (per-row),
  `upsert_links_bulk` (inside the caller's transaction),
  `apply_link_patches` (after commit), and the error-TTL sweep. A bypassed
  writer self-heals: `repair_missing` runs at open and via
  `Database::repair_endpoint_ranks`.

## Alternatives rejected

1. **Keep the window** — 800 ms per fetch is the price of re-deriving a value
   that changes only when a link changes.
2. **A stored, indexed column on `endpoints`** (the shape the spec first
   named): the tuple does not fit one integer (protocol ids are full-range
   hashes) and a `toasty` column needs `push_schema` → a tag bump → a wipe.
   A side table is additive.
3. **Keys computed in SQL** (the prototype's `INSERT … SELECT` over the
   window): puts a second implementation of the decision-16 law in SQL, where
   it can drift from the oracle the panel uses.
4. **Keyset cursors**: still need an index on a stored key — this, minus the
   ability to filter.
5. **In-memory ordered list**: reintroduces the per-frame rebuild T3 removed,
   and does not survive a restart.

## Consequences

- Page fetch: **1757 ms → 8.6 ms** for the default Test sort at any offset
  (`EXPLAIN` shows `SCAN endpoint_rank USING COVERING INDEX`).
  LastSeen/Speed/Traffic/ConfigType ≈ 39 ms (ordered column not indexed);
  Address/Port ≈ 166 ms (sorts the joined rows). The anchor query ≈ 65 ms.
- One-time cost per database: the backfill (typed reads of every endpoint and
  link, then chunked inserts) + index build ≈ 4.6 s at 7.7k endpoints, at the
  first open after the upgrade; the table persists afterwards.
- **Membership now depends on the rank row.** The page drives from the rank
  table, so a write path that changed links without refreshing would hide its
  endpoint. The freshness test pins the three write paths, the parity and
  paging tests pin coverage, and the open-time repair heals a bypassed writer.
  A NULL/missing row is therefore a bug signal, not a normal state.
- Sub-second recency now participates in the tiebreak: the law compares epoch
  nanoseconds (matching the stored `last_seen_at` text the SQL used), where the
  old Rust oracle truncated to seconds.
- Hydration (~450 ms, the `in_list` parameter cost) is unchanged and now the
  dominant term of a page fetch; it is tracked separately from this decision.
