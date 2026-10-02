# Hand-written SQL and DDL in the DB layer — why, where, and the rules

This file is the authority for every place `crates/xray-tui-db` does **not** go
through toasty, and for the reason each exception exists. If you are adding a
query and wondering whether it may be raw SQL, the answer is: read this list
first, and prefer the typed path unless your case matches one of the causes
below. Every entry here is either (a) a capability `toasty` **0.11** does not
have, or (b) a measured cost difference on this engine. Entries re-verified
against the vendored `toasty-0.11.0` source are marked as such inline — currently
**C1 and C2 only**; C3–C6 are unverified against 0.11 and are NOT carried forward
on trust. See §7.

Related records: `docs/database.md` (schema, flows, query map),
`docs/aegis/adr/0001-raw-sql-profiles-page-query.md` (the page query),
`docs/aegis/adr/0002-write-behind-link-writer.md` (the link writers),
`docs/aegis/adr/0003-stored-profile-ordering-keys.md` (the rank keys),
`docs/aegis/specs/2026-09-16-db-claim-verification.md` (the measurements behind
the engine facts cited here).

> **Baseline note.** `AGENTS.md` decisions 4 and 21 describe raw SQL as
> "confined to PRAGMAs, plus `profiles_query.rs`". That was already narrower than
> the code when it was written, and the 2026-09-16 write-path change added one
> more site. The exception list below is the accurate one; the decision text was
> corrected to point here rather than restate the list.

## 1. The rule

- **Typed path first.** Single-row reads and writes of a model (endpoints,
  protocols, links, groups, endpoint↔group links, routing rules, DNS settings,
  route probes) go through the typed API — `Model::filter…`, `upsert_by_*`,
  `toasty::create!/update!` — so the enum/JSON/timestamp encodings and the
  `CHECK` constraints stay the ORM's business.
- **Raw SQL needs a cause from §2.** "It is easier to write" is not one. Neither
  is "it is faster" without a number in the table below.
- **Never a second owner of a fact.** A hand-written statement may only write
  columns its own group/owner owns, and may not create a second spelling of a
  stored fact (this is why the `(family, addr)` split in §5 was rejected).
- **Inlined literals, not binds, on bulk statements.** The engine charges
  ~0.8 ms per *bound* parameter (200 ids = 174 ms against 9.7 ms for the same
  statement with literals, `endpoint_rank.rs`). Ids and keys that come from our
  own rows are rendered as literals; user text still goes through `sql_lit`
  (quotes doubled) or a bind.
- **Every raw statement must be exercised by a test.** There is no single drift
  guard: the statement-vs-schema test (`tests/profiles_query.rs`
  `every_statement_runs_against_a_pushed_schema`) executes only the
  **`profiles_query.rs`** paths (`profiles_page` for every sort/view,
  `profiles_anchor`, `load_page_projection`). The other sites are pinned
  indirectly, by the behavior tests that call them (§3, last column) — and a
  **new** raw statement is covered by neither until a test actually runs it. That
  is checklist item 5 in §6.

## 2. The causes

| Cause | Consequence |
| --- | --- |
| **C1. No upsert / conflict clause.** `toasty` 0.11 **does** have a multi-row insert — `stmt::CreateMany` (`stmt/create_many.rs`, `lib.rs:124`) — but it builds a BARE `INSERT INTO … VALUES (…), (…)` from `item.into_insert()` and carries **no** `on_conflict` / `or replace` / upsert semantics (`grep -rl "on_conflict\|OnConflict" toasty-0.11.0/src/stmt` finds nothing). `upsert_by_*` **does** exist but is macro-generated and single-row: `toasty-macros-0.11.0/src/model/expand/upsert.rs:60-82` emits `upsert_by_{field}` + `{Model}UpsertBy{suffix}`/`…OrIgnore` (documented at `toasty-macros-0.11.0/src/lib.rs:52`), and the statement it builds is documented as "A typed **single-row** upsert statement" (`toasty-0.11.0/src/stmt/upsert.rs:5`). So it is absent from the hand-written `toasty-0.11.0/src/` by construction, not missing. The blocker is therefore the conflict clause, not the row count: an idempotent re-import needs `ON CONFLICT … DO UPDATE`, which a bare multi-row insert cannot express. *(Re-verified 2026-10-01; the previous "no multi-row insert" wording was true of 0.10 and false of 0.11.)* **The link-writer upserts are NOT thereby retired**: they also rest on C6 (10.0 ms/512-patch vs 29.0 ms measured) and on per-column-group `ON CONFLICT … DO UPDATE SET col = CASE …` bucketing (RESULT/TRAFFIC/PURGE), which the typed `upsert` cannot express — the engine compares ONE `UpsertAction` per statement (`toasty-0.11.0/src/engine/lower.rs:1226`, `engine/upsert.rs:61`), so there is no per-group action to set. Migrating them is a separate task with its own measurement.* | Bulk paths are hand-built multi-row statements. |
| **C2. `#[index]` is per-field and single-column.** *(Re-verified against `toasty-macros-0.11.0/src/lib.rs:224` / `:1155` — still "Creates a non-unique index on the field / the field's flattened column", no composite, mixed-direction, partial or expression form.)* No composite, mixed-direction, partial, or expression index. | Indexes toasty cannot express are raw `CREATE INDEX IF NOT EXISTS` at open. |
| **C3. No per-connection hook.** The driver creates pooled connections itself. | Every per-connection PRAGMA is re-issued in `Database::conn()`. |
| **C4. No raw connection access.** `Db::driver()` yields `&dyn Driver`, not the turso connection. | UDFs, `turso::core`, statement-cache control, `set_query_timeout`, `interrupt`, and experimental driver flags beyond the `experimental_*` setters are unreachable. |
| **C5. Engine shape.** No `UPDATE … FROM (VALUES …)` (parse error); unknown PRAGMAs are silently ignored; `array_agg` needs an experimental flag and returns text; `BIGINT`/`BOOLEAN` are illegal in a STRICT table; expression indexes are not used for `LIKE`. | Some statements cannot be expressed at all; some "obvious" tuning is a no-op. |
| **C6. Measured cost.** Binding, per-row statements, and ORM-round-trips cost more than one hand-built statement on this engine. | The numbers in §3 are the justification. |

## 3. The inventory (every site, with its cause and its number)

| Site | What it does | Cause | What the typed path costs | Pinned by |
| --- | --- | --- | --- | --- |
| `profiles_query.rs` (whole file) | The Profiles page: count + ordered ids + one-statement hydration, ids inlined; `profiles_walk_page` is the same ordered-id SELECT with NO count (the batch's plan walk asks for the feed-wide total once instead of once per page, which was `O(feed²/200)`); the plan-scope predicate (`rank_tier IN (…)`, bound, shared by the page, the walk and the count, so a scoped batch and its footer cannot drift) | C2 (covering-index ordering), C6 | 488 ms per page typed (`in_list`, ~600 binds) vs 49.6 ms inlined (ADR 0001); a `ROW_NUMBER()` window sort is 1757 ms vs 8.6 ms index-driven (ADR 0003); the scope predicate is an index range on the same covering index, so it costs no extra storage | `every_statement_runs_against_a_pushed_schema` — the only site with a direct statement-vs-schema guard — plus `walk_pages_match_the_page_ids_and_count_once` for the count-free variant and `plan_scopes_select_by_materialized_tier` for the scope |
| `database.rs` `LINK_UPSERT_PREFIX` + `link_values_sql` + `exec_link_upsert` | `apply_link_patches` and `upsert_links_bulk`: one multi-row `INSERT … VALUES (…),(…) ON CONFLICT(protocol_id, endpoint_id) DO UPDATE …` per (400-row chunk, group action — three groups, so eight bucket shapes) | C1, C5 (`UPDATE … FROM (VALUES …)` is a parse error) | 29.0 ms → 10.0 ms per 512-patch window; 360 ms → 44.3 ms per 2,000 imported links (ADR 0002 amendment 4) | `apply_link_patches_writes_patched_groups_for_every_row`, `…isolates_column_groups`, `…survives_a_stale_snapshot_without_clobbering`, `…applies_a_window_wider_than_one_statement_chunk`, `…upserts_absent_and_near_miss_pairs_exactly`, `link_patch_inserts_a_missing_row_and_refreshes_its_key`; the import path by `subscription_upsert_flow_assembles_group_rows` |
| `database.rs` `link_patch_conflict_sql` | The per-action `DO UPDATE SET` list — the column-group disjointness, in SQL, for THREE groups (RESULT / PURGE / TRAFFIC) | C1 (the action is per-statement, so the groups must be bucketed) | n/a — this *is* the group contract; `contains`-decided, so an unknown bit cannot drop a group. `purge_reason` is its own group because this action writes a FIXED column set from each patch's snapshot, so riding RESULT would let a phase-1 fast half rewrite a verdict it never classified (ADR 0006) | `apply_link_patches_isolates_column_groups` (seeds a verdict, then proves a stale RESULT-only patch leaves it intact while a PURGE-only patch moves it and nothing else), `link_patches_leave_columns_outside_their_groups_alone`, `subscription_upsert_flow_assembles_group_rows` |
| `database.rs` PRAGMAs (`user_version`, `journal_mode=WAL`, `busy_timeout`, `synchronous=NORMAL`, `foreign_keys=ON`) | Connection and durability settings, per connection | C3 | `busy_timeout`/`synchronous` never reach pool-created connections if set once at open — that was the "database is locked" storm | Every `Database::open`/`in_memory` call; the tag semantics by `open_wipes_a_file_with_a_mismatched_schema_tag`, `fresh_open_creates_schema_and_sets_user_version_tag`, `open_reopen_preserves_data` |
| `database.rs` (`sql_lit`, `core_type_str`, `config_type_str`, `error_kind_str`, `purge_reason_str`) | Literal escaping and the `CHECK`-constrained storage spellings (the `purge_reason` spellings are toasty's own, verified by a column-shape probe) | C1 (a hand-built statement has no encoder) | A wrong spelling is rejected by the column `CHECK` or the test that runs it, never silently stored | Any test that round-trips an error/`core_type`/`config_type` through the writers — `apply_link_patches_writes_patched_groups_for_every_row`, `page_projection_matches_the_orm_rows` |
| `endpoint_rank.rs` `ensure_in` | `CREATE INDEX IF NOT EXISTS endpoint_rank_test` (covering, mixed-direction) and `endpoint_rank_window` | C2 | toasty's `#[index]` cannot express either; 1757 ms → 8.6 ms is this index | Runs on every `Database::open`/`in_memory`; the index's *effect* by `page_order_matches_the_rust_oracle_for_every_sort` |
| `endpoint_rank.rs` `write` | Bulk `INSERT OR REPLACE … VALUES` per 400 rows | C1 | Per-row `upsert_by_endpoint_id` is ~1.2 ms per statement: a 400-endpoint window took ~470 ms and a 7.7k-endpoint import ~8 s; the bulk form is ~10× cheaper | `link_writes_keep_the_stored_keys_current`, `a_result_patch_that_inserts_a_link_still_moves_the_key`, `purging_endpoints_drops_their_keys` |
| `endpoint_rank.rs` `refresh` / `load_raw_endpoints` / `backfill` / `repair` / `prune` | Id-inlined reads (`WHERE endpoint_id IN (…)`) and the raw facts behind the rank law | C6 | Binding costs ~0.8 ms per id | `resolving_a_dns_host_moves_its_stored_key`, `purging_endpoints_drops_their_keys`, `load_page_rows_preserves_page_and_link_order`, `seed_ranks` (integration) |
| `endpoint_ip.rs` `ensure` | `CREATE INDEX IF NOT EXISTS endpoint_ip_by_key(ip_key, endpoint_id)` | C2 | The sort wants a two-column index; `#[index]` is single-column | Runs on every `Database::open`/`in_memory`; the index's *effect* by `page_order_matches_the_rust_oracle_for_every_sort` (`PageSort::Ip`) |
| `endpoint_ip.rs` `load` | One id-inlined statement for a page's address sets, key-ordered | C6 | Same 0.8 ms per bound id | `page_rows_assemble_links_and_protocols` (through `load_endpoint_rows`), `large_page_loads_via_batched_in_list` |
| `endpoint_ip.rs` `load_resolved` | The same id-inlined statement plus the `country` column (`load` maps it back to addresses) | C6 | Same 0.8 ms per bound id; one read serves both the page oracle and the enrichment seed | `page_rows_assemble_links_and_protocols`, `large_page_loads_via_batched_in_list`, `stored_country_reaches_the_ui_without_the_mmdb` |
| `endpoint_ip.rs` `countries_of` | Id-inlined `SELECT … WHERE country IS NOT NULL` for the endpoint being rewritten | C6 | It runs once per resolution inside `replace`, and re-reading one endpoint's 1–3 rows is cheaper than re-walking the mmdb | `stored_country_survives_re_resolution` |
| `endpoint_ip.rs` `replace` | Delete-then-insert the whole set, deduped by key (typed delete + typed create per row — the mix that is fine because the set is 1–3 rows and TTL-gated), plus one `countries_of` read to carry the surviving addresses' countries | — | The writes stay typed on purpose (the model for "small set, typed is fine"); only the country read is raw, for the id-inlining reason above | `resolving_a_dns_host_moves_its_stored_key`, `stored_country_survives_re_resolution` |
| `endpoint_rank.rs` `ensure_in` (band columns) | `ALTER TABLE endpoint_rank ADD COLUMN band/rank_host` (attempt + ignore the duplicate-column error), the `endpoint_rank_band_host` + `endpoint_rank_band_window` indexes, and `backfill_bands` (one-time `band IS NULL` fill) | C1/C2/C5 | Raw columns on a DERIVED, rebuildable table avoid a toasty-model change → no `user_version` tag bump → **no file wipe** of a large enriched feed; `#[index]` cannot express the composites. Measured ~26× (band=0 seek 1.04 ms vs Address filesort 27.2 ms, flow_cost `band A/B`) | `backfill_bands_fills_null_band_rows_on_reopen` (the NULL fill, via reopen), `active_and_stale_windows` (band membership through the page) |
| `endpoint_rank.rs` `write` (band/rank_host follow-up) | Per chunk after the `INSERT OR REPLACE`, one `UPDATE … SET band = CASE WHEN rank_newest_seen >= ? …, rank_host = (SELECT host …)` — REPLACE nulls the raw columns, so they are re-set each write | C1 | n/a — it maintains the raw columns the toasty model cannot carry; band is the ttl membership from the just-written `rank_newest_seen` | `active_and_stale_windows`, the page-order parity tests |
| `endpoint_rank.rs` `reband_expired` / `reband_all` | Directional demote `UPDATE … SET band=1 WHERE band=0 AND rank_newest_seen < ?` (retention tick) and the full-recompute `CASE` (startup) | C1/C6 | Membership is a stored `band`, maintained as `now` crosses the threshold; the directional form seeks only the drifted rows via `(band, rank_newest_seen)`, continuity-independent across downtime | `reband_sweep_demotes_rows_that_drifted_during_downtime` (demote), `reband_all_promotes_rows_the_default_backfill_demoted` (promote) |
| `database.rs` `purge_expired` | `SELECT e.id FROM endpoints e WHERE NOT EXISTS (SELECT 1 FROM profile_stats p WHERE p.endpoint_id = e.id AND p.last_seen_at >= ?)` — all-links staleness, replacing toasty's `.all()` quantifier | C6 | toasty's `.all()` compiled to a whole-`endpoints` projection (145 ms, dump-3.log); this is indexed on `profile_stats.last_seen_at`. Deliberately NOT the live-only `band` — a fresh-but-purged link must keep its endpoint (ADR 0006, view-band spec §3.5) | `purge_expired_matches_all_links_semantics` |
| `export.rs` direct reader | Whole-feed export count + one-row-at-a-time projection, with WAL `BEGIN DEFERRED` / MVCC `BEGIN CONCURRENT`, joined `protocols.config`, and deterministic protocol/transport/security/address ordering | C4/C6: Toasty 0.11 public query/raw SQL returns `Vec`; Turso driver drains physical `Rows` before Toasty sees them, so typed/page APIs cannot provide the required bounded row stream. Direct read is file-only and separate from Toasty writes | Public Toasty `.exec` buffers values; no page API preserves physical row streaming. One direct row is decoded and dropped per iteration; feed-wide row/config vectors are forbidden | `export::tests::alive_uses_canonical_link_tier_and_dns_resolution`, `export::tests::resolved_emits_each_dns_address_and_one_ip_literal`, `export::tests::mvcc_reader_rolls_back_probe`; RSS/file-reader smoke in T6 |

## 4. Toasty blockers, with the exact failures

1. **Bulk upsert** — no `insert_many`/`on_conflict` (absent from
   `toasty-0.11.0/src/stmt/` — `CreateMany` exists there but has no conflict
   clause, see C1); single-item `upsert_by_*(…)` only. Cost of the missing
   upsert semantics: the hand-built statement above, plus the manual `CHECK`
   spellings. **Re-verified 2026-10-01 against the vendored 0.11.0 source.**
2. **Composite/mixed-direction/partial/expression indexes** — `#[index]`
   documents "creates a non-unique index on the field". Cost: three raw DDL
   statements at open, invisible to toasty's schema (guarded by every
   `Database::open`/`in_memory` call in the suite executing the DDL).
3. **Per-connection settings** — the driver owns connection creation.
4. **Connection access** — `Db::driver() -> &dyn Driver` exposes no turso
   connection, so `register_external_scalar_function`, `prepare_cached` control,
   `set_query_timeout`, `interrupt`, and `experimental_mvcc_passive_checkpoint`
   are out of reach (RAW SQL is not: `toasty::sql::query/statement` is the
   supported channel and is what every site above uses).
5. **Schema management** — `push_schema` emits `CREATE TABLE` with no
   `IF NOT EXISTS`, so it runs once, guarded by `PRAGMA user_version`
   (decision 4); a tag mismatch **wipes** the file. There is no migration
   machinery, and none is planned while the database is re-importable fixture
   data.
6. **No STRICT tables, generated columns, or materialized views** — see §5.
7. **No caller-supplied driver** — `Database::open` builds `Turso::file(path)`
   internally, so an experiment with a driver flag (`.concurrent_writes()`, any
   `experimental_*`) cannot reuse `open`: a probe has to rebuild the driver *and*
   the `toasty::models!(…)` list itself. A `#[cfg(test)]`/example hook taking a
   driver would remove that duplication.

## 5. Rejected manual paths (so they are not re-proposed)

| Proposal | Verdict | Why |
| --- | --- | --- |
| Enable MVCC (`journal_mode=mvcc` + `BEGIN CONCURRENT`, `Turso::concurrent_writes()`) | rejected | Measured: reader p50 56.5 µs (WAL) → 85.9 µs (MVCC), p95 66.8 → 97.0 µs, writer 5,120 rows / 0 errors in both arms — and the MVCC arm is verified engaged (`journal_mode=mvcc`, 379 KB logical log). Also `PRAGMA wal_checkpoint(PASSIVE)` — the batch-end/quit checkpoint — **fails** under MVCC ("PASSIVE checkpoint requires experimental_mvcc_passive_checkpoint", a builder flag the driver does not expose), MVCC is process-local (`MVCC does not support multiprocess access`), and it adds a `<db>-log` file the wipe paths do not delete |
| Hand-create tables as STRICT | rejected (for now) | STRICT gives **no storage and no speed change** (4,000 rows: 83 pages / 339,968 bytes in both schemas; a full-surface scan 2.72 ms heap vs 2.76 ms STRICT) and its only unlocks are validation and custom types. Its type vocabulary is also **flag-dependent**: without the custom-types flag `BIGINT` and `BOOLEAN` are `Parse error: unknown datatype` in a STRICT table (those are what toasty's DDL emits), and they become legal only with `experimental_custom_types(true)`. So adopting STRICT means hand-owning the **entire** schema DDL *and* turning on an experimental engine flag, for validation alone. (Typed toasty access on a hand-created STRICT table does work: verified create + read.) Revisit only if type validation becomes a requirement |
| Custom types (`CREATE TYPE … BASE blob OPERATOR '<'`) for the address column | rejected | Reachable only with `experimental_custom_types(true)`; the packed-BLOB column already orders and indexes correctly (byte order IS address order) and needs no experimental flag. The operator would buy expressiveness, not speed, and duplicate an existing fact |
| Split `endpoint_ip.ip_key` into `(family, addr)` | rejected | Requires the experimental custom-types flag for `array_agg` (which the driver then returns as TEXT — `String("{\"X'040A0001'\",…}")`, not `Value::List` of blobs, so the cheap decode is unreachable); the split is not cheaper (`group_concat(hex(addr))` 806 µs vs the shipped 724 µs per 200-row page); and it makes `PageSort::Ip` worse (the correct `(family, addr)` term needs two correlated subqueries: 16.49 ms vs 9.97 ms packed; the single-subquery variant cannot produce a pair) |
| One-blob wire via `unhex(string_agg(hex(x), ''))` (the flag-free rewrite of the idea above — `test_array_agg_result_deserialize_to_ips`) | rejected — **correct but not faster** | Not gated (`string_agg`/`hex`/`unhex`/`FILTER` all work with the custom-types flag OFF), and its decoded output is **identical** to the shipped carrier (277 == 277 addresses on a resolved page). But it is the same cost within noise (statement 747–836 µs shipped vs 758–776 µs one-blob; decode 4.7 µs vs 4.6–7.4 µs) while the shipped text parse it would remove costs 8–14 µs per page, i.e. ≤0.4 %. Adopting it would also not fix the mixed-family trap below. If it is ever wanted, the decoder must WALK the packed key (family byte then 4/16 octets), never slice a fixed width |
| Slicing one concatenated blob by a fixed address width | rejected — **incorrect** | On a `(endpoint_id, family, addr)` table the `chunks::<4>()` decode returned **367** addresses where the packed key holds 277: one width cannot describe a mixed IPv4/IPv6 set, so the decoder silently invents addresses. Two `FILTER`ed aggregates (per family) would be required, which is more engine work than the comma list |
| `ANALYZE`, `likely()`/`unlikely()`, `wal_autocheckpoint`, `max_wal_size`, `mmap_size`, `cache_size` tuning | rejected | Measured neutral or non-existent; unknown PRAGMAs are silently ignored, so the tuning would be invisible rather than effective (details in the verification spec §2) |

## 6. Adding a manual path — checklist

1. Name the cause from §2. If none applies, use the typed API.
2. Prefer inlined integer literals for our own ids/keys; `sql_lit` for text.
3. Keep the write inside the transaction that owns the fact, and never write a
   column group another owner writes.
4. Make it idempotent if it runs at open (`IF NOT EXISTS`, `INSERT OR REPLACE`).
5. **Make a test actually run the new statement.** No guard is automatic: the
   statement-vs-schema test covers only `profiles_query.rs`. For a link-writer
   statement that means an `apply_link_patches_*` / `subscription_upsert…` test
   (or a new one) that executes it against the pushed schema; for
   `endpoint_rank`/`endpoint_ip` statements, a test on the path that calls them
   (`link_writes_keep_the_stored_keys_current`,
   `page_rows_assemble_links_and_protocols`, …). A raw statement added without
   such a test is verified by nothing.
6. Record the measurement (probe → this file's table or the relevant ADR). A raw
   statement without a number is a guess, and the next reader will "simplify" it.

## 7. Re-verify a blocker against the vendored source before relabelling it

The causes in §2 are pinned to a **toasty version**, and a version bump can retire
them silently. Nothing detects that: the blocker is prose, so a `0.10 → 0.11`
find-replace looks like a correction while actually *asserting* every absence still
holds.

This is not hypothetical — it happened on 2026-10-01. Blindly relabelling toasty
0.10 → 0.11 would have kept C1's "no multi-row `insert().values([…])`" as fact,
when `toasty-0.11.0/src/stmt/create_many.rs` now provides exactly that
(`CreateMany`, announced at `lib.rs:124`). Re-reading the source showed C1 is
now **half retired**: the multi-row insert exists, the *conflict clause* does not
(`grep -rl "on_conflict\|OnConflict" …/toasty-0.11.0/src/stmt` → nothing). C1 was
requalified to name the capability that is actually missing, which is the one that
still forces the hand-built statement.

So:

1. Cite the vendored path **with its version** (`toasty-0.11.0/src/…`), never a
   bare version number — a stale path is worse than a stale number because it
   reads like a citation that was checked.
2. Before changing a blocker, **grep the vendored source for the missing symbol**
   and read what is there. A capability that APPEARED is a **retirement trigger,
   not a retirement**: record that the trigger fired and what now carries the
   site, then treat the migration as its own task with its own measurement. A
   requalified blocker is a documentation change; rewriting the SQL that depends
   on it is a code change, and it does not ride along on a doc edit.
3. Distinguish "renamed" from "re-verified" in the text, and only claim what was
   checked. **C1 and C2 were re-verified against `toasty-0.11.0` on 2026-10-01;
   C3, C4, C5 and C6 have NOT been and carry no such marker.** C4/C6 were spot
   confirmed incidentally while checking C1 (`Load::Output` for `List<M>` is
   still `Vec<M::Output>`), but their prose is otherwise unverified against 0.11 —
   so a future bump must re-check them rather than trusting the absence of a
   date. A rule that claims more coverage than the table delivers is the same
   defect it was written to catch.

## Weight columns and indexes (2026-10-01)

| Site | Cause | Why raw |
| --- | --- | --- |
| `ALTER TABLE endpoint_rank ADD COLUMN rank_weight BLOB NOT NULL DEFAULT x'0000000000000000'` (`endpoint_rank::ensure_in`, swallowed-error loop beside `band`/`rank_host`) | The weight must be materialized where SQL can `ORDER BY` it. Declaring it on the toasty model would change the pushed schema, and the only lever for that is the `PRAGMA user_version` tag — which decision 4 defines as deleting the database file. The raw-column precedent (`band`, `rank_host`) is the non-destructive path. | The model has no seat for a raw column, so the write path carries a `RankRow` instead. `NOT NULL DEFAULT` is required, not stylistic: `profiles_anchor` binds each ordering term's value back and the engine refuses to type a NULL there, so a NULL weight makes the anchor query ERROR. |
| `CREATE INDEX IF NOT EXISTS endpoint_rank_test_v2 (rank_dns, rank_tier, rank_weight DESC, rank_latency, rank_seen DESC, rank_protocol, endpoint_id)` | Serves the `PageSort::Test` ORDER BY as an index scan (~8.6 ms at 7,672 endpoints) instead of a ~240 ms filesort. | toasty's `#[index]` is single-column and cannot express a mixed-direction composite. **A new name is mandatory**: `IF NOT EXISTS` makes an edited column list a silent no-op on every existing database, so the old index would keep serving the new ORDER BY. |
| `CREATE TABLE IF NOT EXISTS rank_weight_meta (id INTEGER PRIMARY KEY CHECK (id = 0), weight_version INTEGER NOT NULL)` | The weight tables are compiled into the binary, so an app upgrade silently invalidates every stored weight. | A one-row stamp is the smallest thing that can say "these numbers came from a different build". A mismatch recomputes every rank key at open — the only trigger that can replace the all-zero default that `ADD COLUMN` materializes for pre-existing rows. |
| `INSERT … SELECT`-free weight backfill (`backfill_all` reads `Protocol::all()`) | SQL cannot call the Rust `weight_of`. Duplicating the four tables as a SQL `CASE` expression would create a second owner of the law — exactly the drift ADR 0003 exists to prevent. | The weight is derived from `transport_type`/`security_type`/`security_sni`/`security_fp`, four small scalar columns the protocol row already stores. |
| `refresh`'s `LEFT JOIN protocols pr ON pr.id = ps.protocol_id` | The link row carries no protocol, so the weight cannot be derived from `profile_stats` alone. | Four discriminator columns (`transport_type`, `security_type`, `security_sni`, `security_fp` — `proto_kind` is deliberately not a weight dimension) cost one indexed lookup per link inside a statement the refresh path already issues; the deferred `config` JSON stays unloaded. They are parsed with `TransportType::from_db_label`, NOT `FromStr`: toasty's embed writes `http_upgrade`/`x_http` where the wire form says `httpupgrade`/`xhttp`, and the wrong parser silently persists a zero weight for 314 of ~21k real protocol rows. |
