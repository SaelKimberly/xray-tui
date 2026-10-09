# ADR 0012 — Own the Turso driver; migrations own the schema

Date: 2026-10-08.
Status: **accepted — SHIPPED** (S0a, S0b, S4, S6 landed; S1–S3 deferred by user
decision; S5 and T12 rejected on measured evidence).
Spec: `docs/aegis/specs/2026-10-08-db-adapter-and-migrations-design.md`.
Plan: `docs/aegis/plans/2026-10-08-db-adapter-and-migrations.md`.
Supersedes/extends: ADR 0011 (db-rewamp — the raw-SQL/identity slices), ADR 0001
(the page raw SQL), ADR 0002 (write-behind), `docs/database-manual-sql.md`.

## Context

`xray-tui-db` sat on `toasty` 0.11 + the published `toasty-driver-turso` 0.11
(→ `turso` 0.7.2). Two costs accumulated:

1. **The driver's statement cache is unbounded and keyed by SQL text.** Every
   statement went through `prepare_cached`, and our bulk writers INLINE their
   literals (the engine charges ~0.8 ms per bound parameter, so binding is not an
   option), making every window a new text — compiled and retained for the
   connection's life (measured ~116–310 KiB per text; the page hydration grew RSS
   **+123,580 KiB over 400 calls**). The workaround was a second raw connection
   plus a `SqlConn` seam, which is dodging the cache, not fixing it.
2. **The schema tag was a wipe.** `PRAGMA user_version` gated `push_schema`, and a
   mismatch made `open` DELETE the file (AGENTS decision 4). Every change — a
   column, an index, an identity re-key — destroyed the feed.

Two further constraints made this the right moment: `turso` 0.8 was blocked
(two-engine dual-resolve with the published driver's `turso = "0.7"` pin, and
0.0.x majors do not unify), and the crates in the `toasty-*` graph do NOT pin
turso themselves.

## Decisions

1. **D-A — the crate owns a LOCAL-ONLY driver fork.** `crates/xray-tui-db/src/driver/`
   (`mod.rs`, `value.rs`, `error.rs`, ~700 LoC incl. docs), vendored from the
   `thirdparty/toasty` dev clone (already on `turso = "0.8"`), with the
   sync/serverless/remote arms dropped (this is a file/in-memory engine). `toasty`
   runs with the `turso` FEATURE removed so upstream's driver cannot return via
   the flag; the fork is registered with `Db::builder().models(…).build(our_driver)`.
2. **D-B — route the cache at the boundary.** `Connection::exec` routes
   `Operation::RawSql` (the hand-built, literal-inlined statements that
   `toasty::sql::query`/`statement` produce) through the UNCACHED `prepare`, and
   keeps `prepare_cached` for engine-generated `Insert`/`QuerySql`. No LRU,
   counter, or flush threshold. This fixes the leak where it lives and RETIRES the
   workarounds it had forced: the `sql_exec.rs` seam, `Database::write_conn`,
   `CacheSpec::RAW`, `write_window_raw`/`refresh_raw`, and the manual `BEGIN`.
3. **D-C — migrations own the schema.** `crates/xray-tui-db/src/schema/`
   (`mod.rs` + `ddl.rs`): `PRAGMA user_version` is a CURSOR. The cursor SEEDS at
   `SCHEMA_VERSION` (18), so an existing tag-18 file is a no-op and a fresh file
   applies 18; a supported older version applies the pending steps; only an
   UNKNOWN cursor is `IncompatibleSchema` (and takes the pre-alpha wipe). All
   hand-written DDL lives in `ddl.rs` (the two `endpoint_rank` `ALTER TABLE ADD
   COLUMN`s, `endpoint_rank_key`, `endpoint_rank_band_window`, `endpoint_ip_by_key`,
   `rank_weight_meta`). The `CREATE TABLE`s stay with `push_schema` — that IS the
   schema of record until the typed layer is retired (deferred S7), and copying
   them would be a second spelling of one fact.
4. **D-D — MVCC stays opt-in.** Measured on 0.8 (S6): the sequential geo tax is
   reproducible (1.1-1.6x) and the contended row is noise across runs. The lab's
   larger MVCC failure count is an artifact of its own 16-way ROW OVERLAP (every
   writer gets the same geo slice) plus its broken import arm — a corrected probe
   (`geo_under_mvcc_probe`) shows **0 retry exhaustion on the DISJOINT rows
   production writes**, and `WriteBehind::flush` re-stages on any error anyway, so
   an exhausted write is deferred rather than lost. **No retry-budget change is
   shipped.** `XRAY_TUI_TURSO_CONCURRENT_WRITES=1` remains the opt-in, and a
   real-feed A/B is the trigger to revisit.

   **Both former blockers are now fixed, so MVCC is VIABLE, not broken — the
   default is a policy choice, measured below.**
   - *Checkpoint gap (was: MVCC ran with NO checkpoint).* The engine rejected the
     app's only log-bounding statement outright — `PRAGMA wal_checkpoint(PASSIVE)`
     → `PASSIVE checkpoint requires experimental_mvcc_passive_checkpoint` — so an
     MVCC database grew its logical log without bound (measured 59 KiB after 500
     rows, still climbing). `file_driver` now enables that flag with the MVCC
     opt-in, and `ping.rs::wal_checkpoint_enabled` (was `!concurrent_writes`) is
     open in both modes. Verified: same statement `Ok`, log drains to 0, pinned by
     `database::tests::mvcc_checkpoint_succeeds_on_the_open_path` (which FAILS with
     the exact engine error if the flag is removed).
   - *Write failures.* Not a production problem, after two fixes: 0 retry
     exhaustion on the DISJOINT rows production writes, 0 at the production
     2-writer overlap, and at most ONE at 32-way single-row overlap (re-staged by
     `WriteBehind::flush`, not dropped). The fixes:
     (a) `driver::error::classify_turso_error` matched `"conflict"` CASE-
     SENSITIVELY, so the engine's `Conflict: {0}` spelling fell through to
     `driver_operation_failed` and was never retried at any budget — now
     case-insensitive, with both spellings pinned by test;
     (b) `endpoint_ip::set_country` was a SELECT-then-UPDATE-or-CREATE TOCTOU:
     under MVCC two writers could both miss and both INSERT the same composite PK,
     and that `Constraint` is NOT a busy error, so no retry budget could help. It
     is now ONE `ON CONFLICT … DO UPDATE` statement, which also removes the N
     reads per geo window. (Under WAL the TOCTOU could not fire — the transaction
     holds the write lock — which is why only the MVCC layout exposed it.)
   - *Remaining costs of a flip:* (a) an MVCC file is NOT readable by stock SQLite
     tooling (it answers `file is not a database`), and the conversion is ONE-WAY
     (the code converts a fresh file INTO MVCC, never back) — external
     backup/inspection/debug tooling stops working on the app's own data; (b)
     throughput is mixed (MVCC faster on disjoint writes, 1.1-1.6x slower on the
     sequential geo mix, ~3x slower on heavy row overlap). Because (a) is a
     user-visible tooling loss and (b) is unresolved on a real feed, the DEFAULT
     stays WAL; the opt-in is sound and now correctly checkpointed.

## Rejected

- **S1–S3 (an adapter + `xray-tui-db-macro` derive crate) — DEFERRED** by user
  decision to reduce scope. Design retained in spec §3–§4; the hand decoders stay
  AS-IS. It is also the enabler for S7, so **S7 (full toasty removal) is
  unplanned**, not merely optional.
- **T12 (FK cascade) — REJECTED on measured evidence.** Implemented (the fork can
  splice a table-level FK into `CREATE TABLE`, since toasty's db `Schema` carries
  no relation info) and reverted: the cascade creates a delete-race WEDGE.
  `endpoint_ip::set_country`'s missing-row arm CREATES its row (the geo queue
  pushes with no existence re-check, flushed up to 5 s later) and
  `LinkSpec::refresh` inserts rank rows inside the write-behind transaction, so a
  delete racing either raises `FOREIGN KEY constraint failed` and `WriteBehind`
  re-stages its window and every later one, forever. Reproduced:
  `GEO AFTER DELETE: Err(FOREIGN KEY constraint failed)`. The manual ordered
  deletes (`endpoint_ip::delete_for`, `endpoint_rank::prune`) already provide the
  cascade correctly and transactionally, so the FK was a belt that is the thing
  that breaks. **Do not re-add without also making every rank/address writer skip
  a missing parent.**
- **T14 (`WITHOUT ROWID`) — still engine-blocked** (turso refuses DELETE/UPDATE
  on WR tables, 0.8.2 `delete.rs:51` / `update.rs:261`).
- **`profile_stats` / `endpoint_groups` FKs** — their parent is not guaranteed to
  exist first (the import writes links and group links independently of their
  parents); 9 db tests fail on the constraint, so it is simply wrong there.

## Consequences

- The crate depends on its OWN driver source: a `toasty-core` bump is "take the
  bump AND re-sync the fork". `toasty` is pre-0.x-1.0 and its driver crate
  already took a breaking change in 0.11. Retirement trigger: upstream fixes the
  cache leak AND ships a `turso`-0.8 driver.
- `turso` is pinned directly (0.8.2) on ONE core; every `turso::Builder` flag
  (incl. `experimental_mvcc_passive_checkpoint`) is reachable.
- `Database::direct` (the page's execution-layer bypass, ~80× read win) STAYS —
  it is unrelated to the cache.
- `CONTEXT.md`/`ARCHITECTURE.md`/`ROADMAP.md`/`AGENTS.md`/
  `docs/database-manual-sql.md` all restated the removed mechanisms as fact and
  are updated to the new owners.
- The identity re-key path changed: a re-key is now its OWN migration step and
  must NOT bump `SCHEMA_VERSION` (a cursor bump would report a compatible file as
  `IncompatibleSchema` and wipe an imported feed). AGENTS decision 11 corrected.

## Verification (all re-run at HEAD)

- 186 `xray-tui-db` tests / 2277 workspace tests green.
- `tests/cache_routing.rs`: 400 unique-text `RawSql` page-projection statements
  on one pooled connection grow **0 KiB** uncached; with the routing flipped to
  `Cached` they grow **59,336 KiB** and the test FAILS — the guard discriminates.
- 0.7→0.8 on-disk format gate: a real 0.7-written file opens under 0.8, reads
  back, and reopens at tag 18 with NO wipe.
- `tests/schema_migrate.rs`: `LATEST == SCHEMA_VERSION`; a fresh file gets the
  tables + both raw columns + the three raw indexes; a current-file reopen
  preserves data; three reopens tolerate the duplicate-column ALTER; an unknown
  cursor is recreated.
- `cargo tree`: ONE turso core; zero `toasty-driver-turso`. hakari/audit/deny/
  machete unchanged; clippy adds zero diagnostics in the new code.
