# Static config weight — ordering prior for links

Date: 2026-10-01
Status: draft (awaiting review)
Related: decisions 15 (batch pipeline), 16 (tiers + ordering law), 21 (SQL page + materialized keys); ADR 0003 (stored ordering keys), ADR 0008 (id-ordered feed walk)
Touches: `endpoint_rank`, `profiles_query`, `ops/ping`, `xray-tui-proto`, `docs/database.md`, `AGENTS.md`, ADR 0008

## Problem

The batch probes links in whatever order the feed walk yields, and the Profiles
table orders links by one law — decision 16's `(tier, latency, -seen, protocol_id)`.
Neither asks a prior question: *for links nothing has measured, which config is
most likely to work today?* Tier 2 (untested) currently falls through to
`latency = i32::MAX`, then recency, then protocol id — i.e. essentially arbitrary.

A static weight answers that for the untested band and orders the failed bands
for retry priority. It is a **prior compiled into the binary**, not a
measurement, and not a substitute for one.

## The law

One ascending tuple, consumed by `.min()` in `compute_rank`:

```rust
// endpoint_rank::RankLink::key — the SINGLE implementation (decision 16)
(tier, u64::MAX - weight, latency, -seen_secs, protocol_id)
//   u8     u64        i32    i64           i64
```

- `tier` first: a hard-failed link never outranks an untested one. The recorded
  "fresh failures dominate stored successes" user decision holds.
- **Inside a tier, the negated weight outranks latency.** A 900 ms REALITY link
  sorts above a 40 ms TCP link. This inversion is the accepted cost of the
  feature and must never be restated as "measurement still dominates".
- `u64::MAX - weight` is the same negate-to-fit idiom `neg_seen` already uses.
  `key()` stays a plain ascending tuple — no `Reverse`, no `Ordering` wrapper —
  because one tuple plus one `.min()` is what makes the law a single
  implementation shared with the panel's in-memory order.

## Weight

```rust
// xray-tui-proto — #[repr(C)], declaration order IS precedence
pub struct ConfigWeight { pub security: u16, pub mimicry: u16,
                          pub sec_cost: u16, transport_cost: u16 }
// derived Ord: lexicographic, DECLARED order, higher = better
// stored/transmitted as 8 big-endian bytes (field 1 = most significant)
```

- **Lexicographic, higher = better.** Stated once; every consumer obeys.
- Four **orthogonal per-dimension tables** — security, mimicry/DPI
  detectability, security overhead, transport overhead — composed, never a
  hand-authored per-combination table. Protocol *kind* is not a dimension.
- **Coarse bands are mandatory.** Fields 2-4 are read only on exact ties, so
  the tables must be authored with tens of values per cell, not thousands.
  Otherwise "four dimensions" is decorative.
- **Exhaustive matches.** A new `TransportConfig` / `Security` variant is a
  compile error at the table lookup, never a silent `0`. Same rule and same
  reason as `capability::transport_supported`.
- `WEIGHT_VERSION` is a compiled constant; changing any cell bumps it.

## Where the weight comes from

Computed from the **protocol discriminators** already in hand on every read
path — `proto_kind`, `transport_type`, `security_type`, `security_sni`,
`security_fp` (`profiles_query::PAGE_PROJECTION:616-623` carries all five; the
typed path holds `row.protocols`). No `Deferred<Json<ProtocolConfig>>` decode, no
JSON on the write path, no side table.

**This is conditional on the cell values.** An area that needs a *sub-config*
fact — ws `path`, xhttp `mode`, kcp `header_type` — lives in the deferred
config, is not free on the refresh path, and forces a decode per flush window.
If the taste-table authoring needs one, the weight moves to a write-time cache
keyed by `protocol_id` and this section is reopened. **That check happens when
the cells are authored**, which is the last open item.

Four `RankLink` construction sites, and what each needs:

| site | source |
| --- | --- |
| `EndpointRow::link_test_key` (`models_toasty.rs:575`) — **the hot path** | none available: it calls `RankLink::from(&ProfileStats)`, a `ProfileStats`-only signature with no protocol in scope. The weight must be threaded in from `EndpointRow.protocols`, which changes `link_test_key`, `sort_links_by_test_priority` (`:583`) and `best_test_priority_key` (`:628`). Runs on every ping result and every load. |
| `rank_of_row` (`endpoint_rank.rs:237`) | free — `EndpointRow.protocols` already in hand (the cheap instance of the site above) |
| `refresh` (`endpoint_rank.rs:699`) | `LEFT JOIN protocols` added to the statement it already issues |
| `backfill_all` (`:596`) | third typed load `Protocol::all()` + `HashMap<ProtocolId, ConfigWeight>` — it loads typed models, not a raw projection |

The `From<&ProfileStats>` impl cannot compute the weight and must stop being
the only way to build a `RankLink`; it becomes `RankLink::new(link, weight)`.

## Storage

One new raw column on `endpoint_rank`. **No schema tag bump** — the table's own
precedent (`endpoint_rank.rs:315-325`) adds raw columns with
`ALTER TABLE ADD COLUMN` in a swallowed-error loop, which is how `band` and
`rank_host` arrived.

```sql
ALTER TABLE endpoint_rank ADD COLUMN rank_weight BLOB NOT NULL DEFAULT x'0000000000000000'
```

- **8-byte big-endian BLOB, not INTEGER.** SQLite orders BLOBs by memcmp, so
  `ORDER BY rank_weight` equals the Rust order *by construction* across the
  whole 64-bit range. An `INTEGER` would store a value ≥ 2^63 as negative and
  sort **last**, and would reserve bit 15 of the top field as well.
  Precedent: `endpoint_ip.ip_key`, a packed BLOB ordered this way.
- **`NOT NULL` is mandatory.** `profiles_anchor` binds every ordering term's
  value back into a comparison (`profiles_query.rs:543,546`) and the engine
  refuses to type a NULL there (`:275-278`). So the `band`/`rank_host`
  NULL-until-refreshed pattern would make the anchor query **error** on the path
  that runs after every re-sort. Default = the all-zero pack = "worst".
- Written in the **insert column list and values tuple**. NOT in the follow-up
  `UPDATE` — that exists only for the SQL-derived `band`/`rank_host`.
  `INSERT OR REPLACE` nulls any unlisted column, so a missing entry silently
  mis-sorts the page on every refresh.


## Index

New name — **never edit `COVERING_INDEX` in place**:

```
endpoint_rank_test_v2 (rank_dns, rank_tier, rank_weight DESC,
                       rank_latency, rank_seen DESC, rank_protocol, endpoint_id)
```

`CREATE INDEX IF NOT EXISTS` makes an in-place column-list edit a silent no-op
on every existing database (`endpoint_rank.rs:266`, run unconditionally at
`:326-333`): the old index keeps serving and the page silently reverts to the
~240 ms filesort the file warns about.

`profiles_query::order_terms(PageSort::Test)` gains the matching **DESCENDING**
term between `rank_tier` and `rank_latency` (not ASC, as `rank_latency` is).

## Invalidation

Code-owned tables are stale after any upgrade that edits a cell.

- `WEIGHT_VERSION` stored in a one-row meta table, checked inside
  `ensure_in`'s **early-return branch** (`endpoint_rank.rs:334-347`) — on a
  populated table that path returns right after `repair_missing` +
  `backfill_bands`, so no other hook fires.
- **That mismatch is the ONLY trigger that can fill an upgraded database, and
  it is not optional.** `ADD COLUMN ... NOT NULL DEFAULT` leaves **zero** NULLs:
  SQLite materializes the default for every pre-existing row. So a
  `backfill_bands`-style fill keyed on `IS NULL` finds nothing and silently
  does nothing, leaving every endpoint at the all-zero "worst" pack — the
  weight has no effect, nothing errors, and the page looks unchanged. The
  trigger is the version mismatch, or the meta table's absence on a database
  that predates this feature. Mismatch → recompute all keys.
- The meta table's own absence must read as "mismatch", not "current".

## Probe order

`PlanSource::Feed(PlanScope::All)` switches from `PageSort::Id` to
`PageSort::Test`. Consequences, all accepted:

- The walk's order becomes **result-dependent**, so `PlanScope::All` must take
  the frozen-ids path that scoped walks already use (`ops/ping.rs:1355-1359`) —
  offset paging over a mutating predicate skips every row that leaves the scope
  mid-run.
- ADR 0008's decision 1 ("an order no write can move while the batch runs") is
  deliberately given up, along with the ADR's own title.
- The real level no longer runs best-first by construction: the probe order is
  weight-first, not ascending-fast-latency (decision 15).

## Measured (2026-10-01, synthetic 4,000-endpoint feed, 12,000 links)

| measurement | value |
| --- | --- |
| `PageSort::Test` page (200 rows) | **4.43 ms** — plan: `SCAN endpoint_rank AS k USING COVERING INDEX endpoint_rank_test_v2` |
| `PageSort::Id` page (200 rows) | 4.55 ms (the pre-weight baseline shape) |
| frozen full-feed id read | **17.4 ms for 4,000 ids** |
| whole-feed rank-key materialization | 9.6 s (one-time, at open) |

The weight term costs nothing measurable on the page: the new index is served
as a covering scan at the same speed as the id order, so the ~240 ms filesort
the missing-index case would cause did not appear. The freeze is also CHEAPER
than the streaming walk it replaces at this size — one 17.4 ms pass against
~4.5 ms × 20 pages ≈ 90 ms — so the cost the plan recorded as a risk did not
materialize. Both scale linearly; re-measure on the 7.7k reference feed before
trusting either number at scale.

## Non-goals

- No visible score. No column, no cell, no Settings dump.
- No user-tunable weights; no per-ISP/per-country model.
- No learned correction from observed failures.
- No change to the write-behind group model or the purge verdict.

## Known risks

- A hardcoded prior ages silently and is per-deployment, not per-config:
  REALITY-vs-none is a guess with a build date, and its advantage is a property
  of *(config × ISP × DPI vendor)*.
- Ordering failed bands by class encodes a second unmeasured guess.
- Both are mitigated only by measurement — which is why the weight never
  outranks `tier`.

## Authority docs that move in the same change

- **AGENTS.md decision 16** — the law text and `RankLink::key` as its single
  implementation.
- **AGENTS.md decision 15** — "the real level runs best-first BY CONSTRUCTION …
  the probe order IS the fast completion order". Verified: this sentence exists
  only in AGENTS.md, in no `docs/` file.
- **ADR 0008** — decision 1, the title ("id-ordered walk"), and the `:20`
  evidence row. Needs a superseding decision, not a wording edit.
- **`docs/database.md`** — the `endpoint_rank` entity block (`:140-146`), the
  `endpoint_rank_test` index row (`:266`), the write-path list (`:282-288`),
  the query map (`:436`).
- **`docs/database-manual-sql.md`** — the raw column, the new index name and the
  meta table are hand-written DDL sites and need cause/measurement entries.

## Verification

| check | what it pins |
| --- | --- |
| law parity (existing, extended) | SQL `PageSort::Test` order == `load_page_rows` in-memory order, all 8 sorts × both directions, over a fixture carrying differing weights |
| direction sweep | `rank_weight DESC` in SQL and `u64::MAX - weight` in `key()` agree over swept values; a DESC/ASC flip fails here |
| pack round-trip | `ConfigWeight` → 8 BE bytes → `ConfigWeight`, including the all-zero pack |
| pack/SQL order | memcmp order of stored packs == Rust `Ord`, sampled across every cell pair |
| anchor with default | a rank row carrying only the NOT NULL default still answers `profiles_anchor` without error |
| index shape | `the_test_order_has_an_index_that_includes_the_weight` asserts the index's own DDL carries `rank_weight DESC` after `rank_dns`/`rank_tier`. That catches the real failure mode (an in-place column-list edit under `IF NOT EXISTS`, which keeps the old index). It does NOT open a database that already had the old index — that case is covered structurally, because the new index is created under a NEW name and therefore cannot be a no-op |
| anchor on the default pack | `anchoring_works_on_a_row_carrying_only_the_weight_default` — every row reset to the all-zero pack, then `profiles_anchor` over it in both directions at three offsets |
| storage labels | `toasty_storage_labels_for_multiword_transports_parse` — `http_upgrade`/`x_http` parse as the DB label and are REJECTED by the wire parser. Added after review found `refresh` using `FromStr` and silently persisting a zero weight for 314 real rows |
| exhaustiveness | a new transport/security variant fails to compile (enforced by construction, not by test) |
| version rebuild | a stale `WEIGHT_VERSION` forces a recompute at open; a matching one does not |
| walk order | `PlanScope::All` under a running batch probes the frozen set with no skipped endpoint (extends the existing `a_scoped_walk_freezes_its_set_before_probing`) |
| perf | Test page fetch and walk cost vs the recorded 8.6 ms / 3.6 ms-per-page baselines |

## Acceptance

1. A link with no measurement sorts above another untested link when its weight
   is higher, at every tier.
2. A hard-failed link never sorts above an untested link.
3. `profiles_anchor` re-anchors correctly across a re-sort on a database built
   before this change (default-valued column, refreshed and not).
4. A batch started on a database whose `WEIGHT_VERSION` is stale reorders its
   keys at open and probes the frozen set.
5. The Test page stays index-served on an upgraded database.
6. All authority docs above state the new law.

## Open items

- **The four tables' cell values** — the actual taste data. The sub-config
  condition in "Where the weight comes from" is checked at exactly this step.
- **Phasing** — scope is decided (walk reorders in the same change); open is
  whether display-only ships first.
- Acceptance criteria are proposed above and need review.