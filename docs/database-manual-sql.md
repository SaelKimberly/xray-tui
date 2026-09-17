# Hand-written SQL and DDL in the DB layer — why, where, and the rules

This file is the authority for every place `crates/xray-tui-db` does **not** go
through toasty, and for the reason each exception exists. If you are adding a
query and wondering whether it may be raw SQL, the answer is: read this list
first, and prefer the typed path unless your case matches one of the causes
below. Every entry here is either (a) a capability `toasty` 0.10 does not have,
or (b) a measured cost difference on this engine.

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
| **C1. No bulk write API.** `toasty` 0.10 has no `insert_many`, no `on_conflict`, no multi-row `insert().values([…])` — only single-item `upsert_by_*`. | Bulk paths are hand-built multi-row statements. |
| **C2. `#[index]` is per-field and single-column.** No composite, mixed-direction, partial, or expression index. | Indexes toasty cannot express are raw `CREATE INDEX IF NOT EXISTS` at open. |
| **C3. No per-connection hook.** The driver creates pooled connections itself. | Every per-connection PRAGMA is re-issued in `Database::conn()`. |
| **C4. No raw connection access.** `Db::driver()` yields `&dyn Driver`, not the turso connection. | UDFs, `turso::core`, statement-cache control, `set_query_timeout`, `interrupt`, and experimental driver flags beyond the `experimental_*` setters are unreachable. |
| **C5. Engine shape.** No `UPDATE … FROM (VALUES …)` (parse error); unknown PRAGMAs are silently ignored; `array_agg` needs an experimental flag and returns text; `BIGINT`/`BOOLEAN` are illegal in a STRICT table; expression indexes are not used for `LIKE`. | Some statements cannot be expressed at all; some "obvious" tuning is a no-op. |
| **C6. Measured cost.** Binding, per-row statements, and ORM-round-trips cost more than one hand-built statement on this engine. | The numbers in §3 are the justification. |

## 3. The inventory (every site, with its cause and its number)

| Site | What it does | Cause | What the typed path costs | Pinned by |
| --- | --- | --- | --- | --- |
| `profiles_query.rs` (whole file) | The Profiles page: count + ordered ids + one-statement hydration, ids inlined | C2 (covering-index ordering), C6 | 488 ms per page typed (`in_list`, ~600 binds) vs 49.6 ms inlined (ADR 0001); a `ROW_NUMBER()` window sort is 1757 ms vs 8.6 ms index-driven (ADR 0003) | `every_statement_runs_against_a_pushed_schema` — the only site with a direct statement-vs-schema guard |
| `database.rs` `LINK_UPSERT_PREFIX` + `link_values_sql` + `exec_link_upsert` | `apply_link_patches` and `upsert_links_bulk`: one multi-row `INSERT … VALUES (…),(…) ON CONFLICT(protocol_id, endpoint_id) DO UPDATE …` per (400-row chunk, group action) | C1, C5 (`UPDATE … FROM (VALUES …)` is a parse error) | 29.0 ms → 10.0 ms per 512-patch window; 360 ms → 44.3 ms per 2,000 imported links (ADR 0002 amendment 4) | `apply_link_patches_writes_patched_groups_for_every_row`, `…isolates_column_groups`, `…survives_a_stale_snapshot_without_clobbering`, `…applies_a_window_wider_than_one_statement_chunk`, `…upserts_absent_and_near_miss_pairs_exactly`, `link_patch_inserts_a_missing_row_and_refreshes_its_key`; the import path by `subscription_upsert_flow_assembles_group_rows` |
| `database.rs` `link_patch_conflict_sql` | The per-action `DO UPDATE SET` list — the column-group disjointness, in SQL | C1 (the action is per-statement, so the groups must be bucketed) | n/a — this *is* the group contract; `contains`-decided, so an unknown bit cannot drop a group | `apply_link_patches_isolates_column_groups`, `link_patches_leave_columns_outside_their_groups_alone`, `subscription_upsert_flow_assembles_group_rows` |
| `database.rs` PRAGMAs (`user_version`, `journal_mode=WAL`, `busy_timeout`, `synchronous=NORMAL`, `foreign_keys=ON`) | Connection and durability settings, per connection | C3 | `busy_timeout`/`synchronous` never reach pool-created connections if set once at open — that was the "database is locked" storm | Every `Database::open`/`in_memory` call; the tag semantics by `open_wipes_a_file_with_a_mismatched_schema_tag`, `fresh_open_creates_schema_and_sets_user_version_tag`, `open_reopen_preserves_data` |
| `database.rs` (`sql_lit`, `core_type_str`, `config_type_str`, `error_kind_str`) | Literal escaping and the `CHECK`-constrained storage spellings | C1 (a hand-built statement has no encoder) | A wrong spelling is rejected by the column `CHECK` or the test that runs it, never silently stored | Any test that round-trips an error/`core_type`/`config_type` through the writers — `apply_link_patches_writes_patched_groups_for_every_row`, `page_projection_matches_the_orm_rows` |
| `endpoint_rank.rs` `ensure_in` | `CREATE INDEX IF NOT EXISTS endpoint_rank_test` (covering, mixed-direction) and `endpoint_rank_window` | C2 | toasty's `#[index]` cannot express either; 1757 ms → 8.6 ms is this index | Runs on every `Database::open`/`in_memory`; the index's *effect* by `page_order_matches_the_rust_oracle_for_every_sort` |
| `endpoint_rank.rs` `write` | Bulk `INSERT OR REPLACE … VALUES` per 400 rows | C1 | Per-row `upsert_by_endpoint_id` is ~1.2 ms per statement: a 400-endpoint window took ~470 ms and a 7.7k-endpoint import ~8 s; the bulk form is ~10× cheaper | `link_writes_keep_the_stored_keys_current`, `a_result_patch_that_inserts_a_link_still_moves_the_key`, `purging_endpoints_drops_their_keys` |
| `endpoint_rank.rs` `refresh` / `load_raw_endpoints` / `backfill` / `repair` / `prune` | Id-inlined reads (`WHERE endpoint_id IN (…)`) and the raw facts behind the rank law | C6 | Binding costs ~0.8 ms per id | `resolving_a_dns_host_moves_its_stored_key`, `purging_endpoints_drops_their_keys`, `load_page_rows_preserves_page_and_link_order`, `seed_ranks` (integration) |
| `endpoint_ip.rs` `ensure` | `CREATE INDEX IF NOT EXISTS endpoint_ip_by_key(ip_key, endpoint_id)` | C2 | The sort wants a two-column index; `#[index]` is single-column | Runs on every `Database::open`/`in_memory`; the index's *effect* by `page_order_matches_the_rust_oracle_for_every_sort` (`PageSort::Ip`) |
| `endpoint_ip.rs` `load` | One id-inlined statement for a page's address sets, key-ordered | C6 | Same 0.8 ms per bound id | `page_rows_assemble_links_and_protocols` (through `load_endpoint_rows`), `large_page_loads_via_batched_in_list` |
| `endpoint_ip.rs` `load_resolved` | The same id-inlined statement plus the `country` column (`load` maps it back to addresses) | C6 | Same 0.8 ms per bound id; one read serves both the page oracle and the enrichment seed | `page_rows_assemble_links_and_protocols`, `large_page_loads_via_batched_in_list`, `stored_country_reaches_the_ui_without_the_mmdb` |
| `endpoint_ip.rs` `countries_of` | Id-inlined `SELECT … WHERE country IS NOT NULL` for the endpoint being rewritten | C6 | It runs once per resolution inside `replace`, and re-reading one endpoint's 1–3 rows is cheaper than re-walking the mmdb | `stored_country_survives_re_resolution` |
| `endpoint_ip.rs` `replace` | Delete-then-insert the whole set, deduped by key (typed delete + typed create per row — the mix that is fine because the set is 1–3 rows and TTL-gated), plus one `countries_of` read to carry the surviving addresses' countries | — | The writes stay typed on purpose (the model for "small set, typed is fine"); only the country read is raw, for the id-inlining reason above | `resolving_a_dns_host_moves_its_stored_key`, `stored_country_survives_re_resolution` |

## 4. Toasty blockers, with the exact failures

1. **Bulk upsert** — no `insert_many`/`on_conflict` (absent from
   `toasty-0.10.0/src`); single-item `upsert_by_*(…)` only. Cost of the missing
   API: the hand-built statement above, plus the manual `CHECK` spellings.
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
