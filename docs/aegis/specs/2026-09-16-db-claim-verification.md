# Database performance claims: verification and disposition

Date: 2026-09-16
Status: implemented (two write-path changes shipped; the rest dispositioned)
ADR: `docs/aegis/adr/0002-write-behind-link-writer.md` (amendment 4)
Inputs: `docs/Expert 1.md` … `docs/Expert 4.md`, `docs/Database Performance Lab.pdf`
Method: every engine claim was checked in the pinned engine's source
(`turso` 0.7.2, `turso_core` 0.7.2, `turso_sdk_kit` 0.7.2, `toasty` 0.10,
`toasty-driver-turso` 0.10, under `~/.cargo/registry/`) and every cost claim was
measured on this machine with one throwaway probe
(`crates/xray-tui-db/examples/db_claim_probe.rs`, deleted after the numbers were
recorded here). The feed is **7,656 endpoints / 15,312 links / 6,507 addresses**
in a ~3.7 MB file, seeded by the probe; release build; median of the best 5 of 9
runs after 5 warmups.

**Feed caveat.** Absolute levels are not comparable to the numbers recorded for
the 7,656-endpoint reference feed elsewhere in `docs/` (that feed has long
hostnames and large deferred JSON; the probe's are short and `{}`). Only deltas
*within* this probe are quoted as evidence.

## 1. What shipped

| Path | Before | After | Δ |
| --- | --- | --- | --- |
| `apply_link_patches`, 512-patch window (incl. the rank refresh) | 29.0 ms | **10.0 ms** | **2.9×** |
| `upsert_links_bulk`, 2,000 links (incl. the rank refresh) | 360 ms | **44.3 ms** | **8.1×** |

Both writers now emit **one multi-row upsert per (statement chunk, `ON CONFLICT`
action)** instead of one existence probe plus one literal `UPDATE` per row
(`apply_link_patches`) or one typed upsert per row (`upsert_links_bulk`). The
column-group disjointness is unchanged — the action is bucketed by the groups a
patch carries, `contains`-decided, with `DO NOTHING` for a patch that carries
none. See ADR 0002 amendment 4; the contract tests
(`link_patches_leave_columns_outside_their_groups_alone`,
`apply_link_patches_isolates_column_groups`,
`apply_link_patches_survives_a_stale_snapshot_without_clobbering`,
`subscription_upsert_flow_assembles_group_rows`) pin the semantics and pass
unchanged.

`UPDATE … FROM (VALUES …)` is the shape that would have batched the per-row
`UPDATE` directly; the probe confirms the code comment's claim that this engine
lacks it: `near "(": syntax error`.

## 2. Verdict ledger

Dispositions: **SHIPPED** · **ALREADY DONE** (the repo already does it) ·
**REJECTED-MEASURED** (measured, no win) · **REJECTED-IMPOSSIBLE** (the engine
does not support it) · **REJECTED-NOT-A-LEVER** (valid but not worth it) ·
**TOASTY-BLOCKED** (§3) · **N/A**.

| # | Claim | Source | Verdict | Evidence |
| --- | --- | --- | --- | --- |
| 1 | Materialized rank keys beat a window sort (−82 %) | PDF C-01, E1/E4 §existing; E3 §2B proposes deleting them | **ALREADY DONE** — and the deletion proposal is **REJECTED-IMPOSSIBLE** | `endpoint_rank` + `endpoint_rank_test` covering index shipped (ADR 0003, 1757 ms → 8.6 ms). E3's UDF replacement cannot use an index: `ORDER BY f(…)` is not index-served, so it re-derives the law per row |
| 2 | Inlining ids beats binding parameters | PDF C-02 "TEST", E3 §1, E4 §2.1 | **ALREADY DONE** (as an inline, measured) | turso charges ~0.8 ms per bound parameter: 200 ids = 174 ms vs 9.7 ms inlined (`endpoint_rank.rs:211`); page hydration is one inlined statement for the same reason. The PDF's local `node:sqlite` fixture found the opposite direction — that fixture is not this engine |
| 3 | Multi-row batches beat single inserts (+36 %) | PDF C-03 SHIP | **ALREADY DONE**, and extended by #20 | `endpoint_rank::write` has been one `INSERT OR REPLACE … VALUES` per 400 rows since ADR 0003 |
| 4 | Batched rank refresh beats one upsert per row (−28 %) | PDF C-05 SHIP | **ALREADY DONE** | One `endpoint_rank::refresh` per flush window over the deduped touched set (ADR 0003); measured today: the existence probe, not the refresh, was the patch path's cost |
| 5 | Normalized `lower(host)` lookup avoids a scan | PDF C-04 SHIP, E3 §2E ("O(1) via Expression Indexes") | **REJECTED-MEASURED / REJECTED-NOT-A-LEVER** | The shipped predicate is an **infix** `lower(host) LIKE '%needle%'` (`profiles_query.rs:261-268`), which no B-tree can serve. Measured: infix 1.85 ms, prefix 1.29 ms over 7,656 endpoints; creating `CREATE INDEX … ON endpoints(lower(host))` changed **nothing** (1.85 / 1.65 ms) — this engine does not use an expression index for `LIKE` at all. The claim's O(1) is unattainable; the prefix row is a product change (substring → prefix search), not an index |
| 6 | MVCC / `BEGIN CONCURRENT` removes writer contention (5–10×) | PDF C-06 HOLD, E1 §5, E2 §3.1, E3 §2C, E4 §4.1 | **REJECTED-MEASURED** (+ named hazards) | The capability exists and is wired: `toasty-driver-turso` 0.10 has `Turso::file(p).concurrent_writes()`, which sets `journal_mode='mvcc'` and issues `BEGIN CONCURRENT` (driver `lib.rs:379-410, 687-712`), and conflicts already classify as retryable (`retry.rs`). **Preconditions asserted** (2026-09-16 re-run): the MVCC arm reads `PRAGMA journal_mode` = `mvcc`, `PRAGMA mvcc_checkpoint_threshold` answers (MVCC-only pragma), and a 379 KB `<db>-log` logical log exists after the writes — so MVCC genuinely engaged; the WAL arm reads `wal` with a 0-byte log. Result: reader p50 **56.5 µs WAL → 85.9 µs MVCC**, p95 66.8 → 97.0 µs, writer completed 5,120 rows with 0 errors in both arms. Same conclusion as the 2026-09-15 `endpoint_ip` A/B (27.0 → 27.6 ms page p50). Hazards found: `PRAGMA wal_checkpoint(PASSIVE)` — the batch-end/quit checkpoint — **succeeds on WAL and fails only under MVCC** (`Invalid argument supplied: PASSIVE checkpoint requires experimental_mvcc_passive_checkpoint`; the flag is a `turso::Builder` feature the toasty driver does not expose), MVCC itself is **process-local** (turso refuses it with multiprocess WAL: "MVCC does not support multiprocess access"), and it creates a second file (`<db>-log`) that `Database::open`'s wipe paths — which delete only the `.db` — would leave behind |
| 6b | Hand-create the tables as STRICT | (follow-on from E3's STRICT/custom-type route) | **REJECTED** (no gain, large cost) | Storage and speed are **identical**: 4,000 rows / 83 pages / 339,968 bytes in both schemas, full-surface scan 2.72 ms (heap) vs 2.76 ms (STRICT). What STRICT unlocks is validation (`cannot store TEXT value in INTEGER column`) and custom types — and its type vocabulary is **flag-dependent**: without `experimental_custom_types`, `BIGINT` and `BOOLEAN` are `Parse error: unknown datatype …` in a STRICT table (with the flag they are legal). Those are exactly the types toasty's own DDL emits, so STRICT means hand-owning the entire schema DDL **and** an experimental engine flag, for validation alone. Typed toasty access *on* a hand-created STRICT table does work (verified typed create + read) |
| 6c | Carry a group's addresses as one aggregated blob — `array_agg(addr)` (user proposal), then `unhex(string_agg(hex(x), ''))` (the user's flag-free rewrite, `test_array_agg_result_deserialize_to_ips`) over a split `(family, addr)` table | user proposals | **REJECTED** — array/flag version impossible, blob version equal-in-cost, split version **incorrect** | `array_agg` is refused without the custom-types flag, and with it the driver returns **TEXT** (`Value::String("{\"X'040A0001'\",…}")`), not `Value::List` of blobs. The flag-free rewrite is *correct* — one blob decodes byte-identically to the shipped carrier (277 == 277 addresses on a 200-endpoint resolved page, via a family-byte walk) — but it is the same cost within noise (statement 747–836 µs shipped vs 758–776 µs one-blob over 7 runs; decode 4.7 µs vs 4.6–7.4 µs) while the shipped text parse costs 8–14 µs per page (≤0.4 %). Slicing the blob by a fixed width on a split `(family, addr)` table returned **367** addresses against the packed key's 277 — one width cannot describe a mixed IPv4/IPv6 set, so it silently invents addresses. The packed single column stays; a future blob decoder must WALK the packed key (family byte, then 4/16 octets) |
| 7 | Zero-copy native API drops hydration below 5 ms | PDF C-07 HOLD, E3 §2A | **REJECTED-IMPOSSIBLE / TOASTY-BLOCKED** | There is no zero-copy row API to reach: `turso::Row::get::<T>` returns owned values and `Value::as_blob() -> &Vec<u8>` borrows a copy already owned by the crate; row-level zero copy (`ValueRef`) lives in `turso_core`, which the driver does not expose. Hydration measured 3.5 ms (probe feed) / 43.9 ms (reference feed) — the "488 ms" in the reports is the retired typed `in_list` path (ADR 0001) |
| 8 | WAL tuning (`wal_autocheckpoint`, `max_wal_size`) doubles write throughput | E4 §4.3 | **REJECTED-IMPOSSIBLE** | Neither pragma exists in turso 0.7.2 (~30 `PragmaName` variants; `turso_core/pragma.rs`, `translate/pragma.rs`), and unknown pragma names are **silently ignored** (`translate/pragma.rs:228`), so the recommendation would be a silent no-op. `mmap_size` is likewise absent — the IO layer is the crate's own async IO, not mmap |
| 9 | `PRAGMA synchronous=FULL` is "safe on SSD" | E4 §4.4 | **REJECTED-MEASURED (regression)** | FULL is the default the repo left deliberately: **4.2 ms per commit** against 0.219 ms with `synchronous=NORMAL` (`database.rs:184-186`), and a ping batch issues ~90k commits. The SSD reasoning is not evidence |
| 10 | Raise `cache_size` (and `temp_store=MEMORY`) | E1 §2 (impact claim), E4 §4.4/§4.5 | **REJECTED-MEASURED** | Default is `-2000` = 2 MiB (`turso_core/storage/sqlite3_ondisk.rs:166`). Measured with `cache_size=-20000`: page ids+hydration 3.51 ms → 3.84 ms, and a scan-heavy search 1.85 → 1.81 ms — no win either way. `temp_store` only affects temp B-trees (the sorts are index-driven) |
| 11 | Composite `profile_stats(endpoint_id, last_seen_at DESC)` + partial index on error columns | E1 §1 | **REJECTED-NOT-A-LEVER** | The staleness window reads the indexed `endpoint_rank.rank_newest_seen` (window count measured 0.24 ms). The error sweep measured 1.5 ms → **1.2 ms** with `CREATE INDEX … ON profile_stats(updated_at) WHERE error IS NOT NULL` — −0.35 ms on an open/batch-finish-only path, against a third raw DDL statement. `models_toasty.rs` already priced this and declined |
| 12 | `ANALYZE`, `likely()`/`unlikely()`, `array_agg`, `USING btree/inet`, materialized views | E1 §4/§6, E4 §4.x | **REJECTED-MEASURED / IMPOSSIBLE** | All four were priced in the 2026-09-15 spec §5: `ANALYZE` changed no plan and no timing; `likely()`/`unlikely()` are inert-to-harmful (they hide the predicate from index-seek detection); `array_agg` needs the experimental custom-types flag and returns an internal record payload; materialized views cannot take the page query's shape (`LEFT OUTER JOIN is not yet supported in incremental views`) |
| 13 | Generated columns pre-computing "total traffic" | E1 §4 | **REJECTED-NOT-A-LEVER / TOASTY-BLOCKED** | `profile_stats` already stores `traffic_total_up`/`traffic_total_down` as columns; there is no aggregate to pre-compute. And toasty cannot declare a generated column — only raw DDL reaches one |
| 14 | In-place VACUUM / periodic checkpoints | E3 §2D, E1 future | **N/A** | `experimental_vacuum` exists as a builder flag the driver does not expose; nothing measured shows a free-page problem (the probe DB's file is 3.7 MB for 15k links, the user's 25 MB file is dominated by deferred JSON) |
| 15 | Chunk size 1,000–2,000 beats 400 | E1 §3, E4 §4.1 (P0/"3–5×") | **REJECTED-MEASURED** | Same statement, three widths: rank `INSERT OR REPLACE` 11.4 / 11.5 / 11.3 ms; the new patch upsert 11.1 / 10.6 / 10.6 ms at 400 / 1,000 / 2,000 rows. Within noise, so `LINK_STATEMENT_ROWS` stays 400 |
| 16 | Cache the hottest prepared statements | E1 §3 | **ALREADY DONE** (with a caveat) | The driver already executes every statement through `prepare_cached` (`toasty-driver-turso/src/lib.rs:796`); turso's cache is an unbounded `HashMap<String, _>` keyed by statement text (`turso_sdk_kit/src/rsapi.rs:895, 1089-1133`). The caveat: literal-inlined SQL has a new text per page, so those statements can never hit — the price of #2, borne deliberately. Not measured; named as the open item in §4 |
| 17 | Schema migrations instead of wipe-on-schema-change | E4 §4.2 (P1), E1 §5 | **REJECTED (out of scope)** | Decision 4 keeps the pre-alpha wipe tag, and the database is re-importable fixture data. It is also not a performance claim |
| 18 | Optimize `toasty` batch ops (`EndpointRank::insert_many(..).on_conflict(..)`) | E4 §4.3 | **TOASTY-BLOCKED**; the win was taken in raw SQL instead | toasty 0.10 has no `insert_many`/`on_conflict` (absent from `toasty-0.10.0/src`; only single-item `upsert_by_*` and `insert(expr)`). The import path was therefore per-row typed upserts; #20 replaced it with raw multi-row upserts |
| 19 | In-memory pool sharing, vector search, Postgres wire, FTS/Tantivy, health checks, query timing | E1 §6/§7, E4 §4.4/§4.11-14 | **N/A** (one named lever) | The first three have no consumer here. FTS is available (`fts` is a default feature of the `turso` crate and `USING fts` works, 2026-09-15 spec §5) but it is token-based, so it would change search semantics from substring to full-text; search measured 1.85 ms today, so it is the named lever only if search ever grows |
| 20 | Batch the two remaining per-row write paths | derived from the reports' shape | **SHIPPED** (see §1) | The probe's cost model put 7.4 ms of the 29.0 ms patch window in the per-row `UPDATE`s and 0.6 ms in the probe; the import path's typed upserts were 360 ms/2,000 rows. Both are now multi-row upserts |

## 3. Toasty blockers

Where the *report's* suggestion is not reachable through `toasty` 0.10 /
`toasty-driver-turso` 0.10, with what the workaround costs. The maintained
version of this list — with the inventory of every raw-SQL site and the rules for
adding one — lives in `docs/database-manual-sql.md`; this is the verification
record.

1. **No bulk upsert.** No `insert_many`/`on_conflict`/`insert().values([..])`: the
   only bulk write is a hand-built multi-row statement. Cost: the ORM's type
   safety is replaced by literal rendering in raw SQL (this is why the enum
   spellings are re-asserted by `core_type_str`/`config_type_str`/
   `error_kind_str`), and the `CHECK` constraints plus the drift test are what
   catch a wrong spelling. **Measured 8.1×** on the import path (§1).
2. **`#[index]` is per-field and single-column.** No composite, no
   mixed-direction, no partial, no expression index. Cost: three raw
   `CREATE INDEX IF NOT EXISTS` statements at open (`endpoint_rank_test`,
   `endpoint_rank_window`, `endpoint_ip_by_key`) — the pattern is established and
   idempotent, but toasty's schema and the real schema can drift (only the
   drift test catches it).
3. **No per-connection hook.** The driver creates pooled connections itself, so
   every per-connection pragma must be re-issued by `Database::conn()`
   (`busy_timeout`, `synchronous`) — the historical "database is locked" storms
   were this, and any new per-connection setting has to be added there, not in
   `open()`.
4. **No raw connection access.** `Db::driver()` returns `&dyn Driver` (not the
   turso connection), so nothing can reach `turso::core` — no
   `register_external_scalar_function` (the UDF route in E3 §2B), no
   `set_query_timeout`/`interrupt`, no statement-cache inspection (E1 §3's
   "cache the hottest statements", #16's caveat), and no
   `experimental_mvcc_passive_checkpoint` (which is what #6's checkpoint failure
   needs). RAW SQL still works (`toasty::sql::query`/`statement`), which is how
   every raw change here is made.
5. **`push_schema` is not idempotent** (`CREATE TABLE` without `IF NOT EXISTS`),
   which is why a schema tag — and a file wipe — is the only change policy
   (decision 4).
6. **No STRICT/custom types, generated columns, or materialized views** in the
   model layer. `#[column(type = "inet")]` compiles and emits the name, but
   STRICT (which is what makes a custom type validate) is unreachable.
7. **No caller-supplied driver.** `Database::open` builds `Turso::file(path)`
   internally, so any experimental driver flag (`.concurrent_writes()`, the
   `experimental_*` set) cannot be set without editing `open` — and an A/B probe
   must rebuild the driver plus the full `toasty::models!(…)` list itself,
   duplicating `try_open_db` (this probe did exactly that for the MVCC arm). A
   test/bench hook that accepts a driver would remove that duplication.

## 4. Open items (named, not silently dropped)

- **Statement-cache growth from literal inlining** (#16). Inlining is a measured
  win over binding, but it makes every page's hydration statement a new cache
  entry in an unbounded map. Not measured; the lever if it ever matters is a
  stable statement shape (binds) *per page* — which re-introduces the 0.8 ms/bind
  cost.
- **Search is a scan** (#5). 1.85 ms over 7,656 endpoints with a rare needle, 6.0
  ms when the needle matches every host — acceptable today. A prefix fast-path
  (index-serviceable) or FTS (semantic change) are the two options; both are
  product decisions.
- **IP sort** remains ~166 ms per fetch; the materialized `rank_ip` key (166.9 →
  0.94 ms) stays deliberately declined (2026-09-15 spec §4).
- **MVCC** (#6) is re-evaluated only if turso makes it multiprocess-safe and
  exposes the passive-checkpoint flag through the driver — and only with a
  write-throughput workload that shows contention, which this app's single-flush
  writer does not produce.

## 5. Probe

Two throwaway probes produced the numbers above (deleted after the results were
recorded here, per the convention set by the 2026-09-15 spec §8):

- `db_claim_probe.rs` — the feed seeding, page ids + hydration, the patch path
  (before/after), the import path (before/after), the rank write at three widths,
  the retired existence probe at three widths, the error sweep with and without a
  partial index, the search predicate with and without an expression index and
  with a larger cache, `EXPLAIN QUERY PLAN`, `UPDATE … FROM (VALUES …)`
  feasibility, `cache_size`, and the WAL-vs-MVCC reader A/B.
- `db_shape_probe.rs` — the STRICT facts (DDL legality, validation, storage,
  scan cost, typed toasty access on a hand-created STRICT table), the
  `(family, addr)` split against the packed key (aggregate cost, sort cost,
  `array_agg` shape with and without the custom-types flag), and the MVCC A/B
  re-run with its preconditions asserted (journal-mode value, the MVCC-only
  pragma, the logical-log file, and the checkpoint called the way the app calls
  it — via `toasty::sql::query`, since a checkpoint returns a row).

The rules for touching the manual SQL these findings justify live in
`docs/database-manual-sql.md`.
