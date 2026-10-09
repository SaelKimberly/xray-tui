# Owned driver and migrations — implementation plan

Date: 2026-10-08.
Spec: `docs/aegis/specs/2026-10-08-db-adapter-and-migrations-design.md` (rev. 1,
**approved** for planning; every advisory pass folded in).
Status: **draft rev. 1 — S1–S3 (adapter + macro) DEFERRED by user decision
(2026-10-08); the plan covers the driver fork + migrations + FK + MVCC.**

## Goal

Land the approved Option D **core**, staged: fork the Turso driver into
`xray-tui-db` (S0a vendor + S0b cache-routing — kills the statement-cache leak
at its boundary, unlocks turso 0.8, deletes ~5 raw-connection workarounds),
route all DDL into one versioned migration list (S4), ship T12 FK cascade (S5),
and re-measure MVCC on 0.8 to decide its default (S6). The adapter +
`xray-tui-db-macro` derive crate (S1–S3) are **deferred**; full toasty removal
(S7) stays optional.

## Architecture / Tech Stack

Rust 2024, toasty 0.11 (`serde` + `jiff` features, **no** `turso` feature) +
our vendored local-only driver over `turso 0.8`. New deps at this scope:
`async-trait`, direct `toasty-core` / `toasty-sql` (both published 0.11.0).
(`xray-tui-db-macro` + `simd-json` belong to the deferred S1–S3.) Owners:
`xray-tui-db` (`driver/`, `schema/`, `database.rs`, `endpoint_rank.rs`,
`endpoint_ip.rs`, `write_behind.rs`, `export.rs`, `profiles_query.rs`), plus the
TUI crate (`ops/db_monitor.rs` unchanged).

## Baseline / Authority Refs

- Spec rev. 1 (approved); `docs/database.md`; `docs/database-manual-sql.md`
  (raw-SQL rule, causes C1/C2/C4, the statement-cache section);
  `AGENTS.md` decisions 4, 11, 21, 22, 23; ADR 0001/0002/0003/0011.
- Verified upstream facts (this plan's evidence): `toasty-driver-turso 0.11.0`
  pins `turso = "0.7"`; `toasty main` bumped to 0.8 in `c767f6c9`
  (Cargo.toml/lock ONLY); `Connection`/`Driver` trait surfaces; `QueryLog::TARGET`;
  `Capability::SQLITE` public; the two-engine semver-major dual-resolve.
- The one destructive step is S4's migration runner re-expressing the wipe
  (`Step::Rebuild`); S0a/S0b, S5, S6 are non-destructive.

## Compatibility Boundary

- **No file wipe in S0a/S0b, S5, S6.** S0a keeps `push_schema`'s `create_table`
  loop, so a fresh file still gets the 11 tables and an existing file still
  reopens under tag 18.
- S4 introduces the migration cursor; a tag **mismatch applies pending
  migrations** (reopen does NOT wipe) — this is the behaviour change the spec
  authorises, with `Step::Rebuild` reserved for identity re-keys.
- Public crate APIs are workspace-internal; no external compatibility promise.
- `CoreEvent`, the profiles row model, and every TUI surface are unchanged.

## TDD Route

`TDD Route: mode=off, decision=skipped, authority=none (no explicit TDD request;
project default off), reason=this is a like-for-like driver swap + a schema-owner
move, behind existing oracles (the db suite byte-identical at S0a, the
statement-vs-schema and migration tests at S4) — no new behaviour to drive
RED/GREEN; verification=nextest workspace + clippy --all-features + just
quality-gate + the S0b cache/routing test + the S6 real-feed A/B.`

## Change Necessity

Source changes required: the driver is a dependency we must own to fix the cache
at its boundary and to declare turso 0.8 (no config/docs substitute); the DDL
list is source. Minimum boundary = the files per task. No new abstraction beyond
`driver/` and `schema/` at this scope.

## Ripple Signal Triage

Fires (dependency direction, source-of-truth, persistence, contract,
retirement). Canonical owners after this change: `xray-tui-db/src/driver/` owns
the DB engine seam; `schema/ddl.rs` owns the schema; toasty keeps the typed query
layer and the hand decoders (`profiles_query.rs`, `export.rs`) stay. Downstream
consumers: `database.rs`, `endpoint_rank.rs`, `endpoint_ip.rs`,
`write_behind.rs`, `export.rs`, `profiles_query.rs`, `models_toasty.rs` (all in
`xray-tui-db`), plus the db tests. **Retirement:** `sql_exec.rs::SqlConn` +
`RawConn`, `Database::write_conn`, the manual `BEGIN` in `write_behind.rs`, the
`toasty-driver-turso` dep, the `ensure*` DDL — each with its trigger in
S0a/S0b/S4/S5.

## Plan Pressure Test

Owner fit ✓ (one new seam, one DDL owner). Higher-level path: the fork replaces
the driver dependency rather than adding a layer. Verification scope ✓ (parity
oracles + the migration tests). Executability ✓. Route: **inline** — S0a is a
pure dependency swap (byte-identical suite), S0b is the contained behaviour
change, and S4 depends on S0a/S0b's `push_schema` body.

## Ground rules

1. **S0a is a PURE dependency swap** (vendor + register + turso 0.8, still
   `prepare_cached`): the db suite must pass **byte-identically**. If the fork or
   0.8 misbehaves, revert S0a alone. **S0b is the behaviour change** (op-route +
   delete the workarounds). Nothing further starts until `just quality-gate` is
   green.
2. **S1–S3 are DEFERRED** (adapter + macro). The hand decoders and the raw bulk
   writers stay AS-IS; nothing in S4–S6 depends on them.
3. **`push_schema` stays functional through S0a/S0b** (ports `create_table`). The
   migration runner arrives at S4 and swaps that body.
4. Each stage is its own commit; each stage's green boundary is stated.

---

## S0a — Vendor the driver + swap the dependency (PURE swap, byte-identical)

**Goal:** the driver is OURS at turso 0.8, with the local arm behaviour-identical
to today (still `prepare_cached`). Nothing else moves. If the fork or 0.8 has a
problem, this is one contained revert.

### T0a.1 — Vendor the local-only driver

**Files:** `crates/xray-tui-db/src/driver/{mod.rs,conn.rs,value.rs,error.rs}`
(new); `crates/xray-tui-db/Cargo.toml`; `Cargo.toml` (workspace deps).
**Fork source:** the **fresh dev clone** `thirdparty/toasty` @ `4f3ed1a`, whose
`toasty-driver-turso` is ALREADY on `turso = "0.8"` (`Cargo.toml:161`) — its
local arm is otherwise identical to the published 0.11.0 crate (verified: the
whole delta is the remote/serverless arm — `TursoPath::Remote` gains
`default_begin_sql`, a new `with_transaction_mode`, libsql URL defaults). Using
the clone removes the "will the 0.11 driver compile against 0.8" risk outright.
**Vendor a COPY — do NOT path-depend into `thirdparty/`** (an untracked reference
tree): copy the source into `crates/xray-tui-db/src/driver/` and own it.
**Change:** copy the local arm, dropping sync/serverless/remote. Implement all
**9** required trait methods (spec §12.2): `Driver::{url, capability, connect,
generate_migration, reset_db}` (only `max_connections` defaulted),
`Connection::{exec, push_schema, applied_migrations, apply_migration}` (only
`is_valid`/`ping` defaulted). `exec` **keeps `prepare_cached`** at this stage
(pure swap). `push_schema` ports toasty's `create_table` loop **unchanged**;
`applied_migrations`→`Ok(vec![])`, `apply_migration`→`Ok(())`;
`generate_migration` reuses `toasty_sql::MigrationStatement::from_diff` +
`Serializer::sqlite`; `reset_db` is local-file only. **KEEP `concurrent_writes()`**
(and its `default_begin_sql = "BEGIN CONCURRENT"` mapping) — it is how MVCC is
switched on today (`file_driver`:240-244) and S6's whole benchmark rests on it.
**Also keep `experimental_mvcc_passive_checkpoint` reachable** (S6 names it).
**Move (not delete) the shared codec:** `from_turso_value` (used by
`exec_raw`/`exec_direct`, the `direct`-reader path, which STAYS) must move from
`sql_exec.rs` into `driver/value.rs`; re-point `profiles_query.rs:246`'s import.
`to_turso_value` is already local to `profiles_query.rs:249` — move it beside
`from_turso_value` for symmetry (both directions of the turso↔toasty Value codec
in one owner), or leave it. The error classification (`export::turso_error`)
stays in `export.rs`; only `raw_turso_error` (used solely by the deleted
`RawConn`) goes.
**Compatibility:** behaviour-identical to the published driver's local arm.
**Verification:** `cargo check -p xray-tui-db`; `cargo nextest run -p xray-tui-db`;
**AND a 0.7→0.8 on-disk-format gate:** open a REAL 0.7-written `data.db` fixture
(copy of a current feed) with the 0.8 fork and assert it READS the tag-18 rows
and takes the **no-op** path (no wipe). turso's changelog is SILENT on format
stability across 0.7→0.8 — silence is not proof, and if 0.8 cannot read a 0.7
file, `try_open_db` fails → `open_with_concurrent_writes`'s recovery arm
(`database.rs:318-330`) DELETES the file and recreates it, i.e. a silent data
wipe disguised as recovery. If the format DID change, S0a needs a stated
disposition (a one-time documented wipe or a re-export path), never the silent
recovery arm.

### T0a.2 — Register the fork + manifest + hakari + turso 0.8

**Files:** `crates/xray-tui-db/src/database.rs` (`file_driver`:240,
`try_open_db`:449, `in_memory`:523, test drivers:2780/2825),
`crates/xray-tui-db/src/models_toasty.rs` (test drivers:1147/1338),
`crates/xray-tui-db/Cargo.toml`, `Cargo.toml`,
`crates/xray-tui-hakari/Cargo.toml` (regenerated).
**Change:** replace `toasty_driver_turso::Turso` with `crate::driver::Turso`
everywhere; drop the `toasty-driver-turso` dep; add `async-trait`, direct
`toasty-core = "0.11"` / `toasty-sql = "0.11"`, `turso = "0.8"`; **drop the
`turso` feature from the `toasty` dep** (else upstream's driver returns via the
feature flag); bump the workspace `turso` pin to 0.8; `cargo hakari generate` +
`manage-deps`. **Keep `toasty`/`toasty-core`/`toasty-sql` at published 0.11.0**
and verify the vendored driver compiles against them (it imports only
`TransactionMode`/`IsolationLevel`/`Operation`/`RawSqlRet`/`Transaction`/
`TypedValue` + `toasty_sql` — all in published 0.11.0, verified). If a dev-only
core API appears, fall back to the published 0.11 driver source + the
manifest-only 0.8 pin (toasty main's `c767f6c9` proved that compiles).
**Verification:** `cargo tree -i turso@0.8` shows ONE turso core;
`just hakari-check` green; `cargo nextest run --workspace`; `just quality-gate`.

**S0a exit:** db suite byte-identical; one turso core at 0.8; the fork compiled
against published 0.11 core. No behaviour change yet.

---

## S0b — Route the cache + retire the raw workarounds (the behaviour change)

### T0b.1 — Route `exec` by operation (the cache fix)

**Files:** `crates/xray-tui-db/src/driver/conn.rs`.
**Change:** in `exec`, `Operation::RawSql` → `conn.prepare(...)` (UNCACHED);
`Operation::Insert` / `Operation::QuerySql` → `conn.prepare_cached(...)`.
**Verification:** a test (in `driver/conn.rs` tests or
`crates/xray-tui-db/tests/`) that loops fresh-text `RawSql` on ONE pooled
connection and asserts RSS/`cached_statements` **plateaus** while a repeated
`Insert` stays cached. Pin the routing boundary, not the engine internals.

### T0b.2 — Delete the raw workarounds

**Files:** `crates/xray-tui-db/src/sql_exec.rs` (DELETE the `SqlConn`/`RawConn`
seam + `raw_turso_error`), `crates/xray-tui-db/src/lib.rs` (drop the `sql_exec`
module + exports), `crates/xray-tui-db/src/write_behind.rs` (the `CacheSpec::RAW`
const + `write_window_raw` + `refresh_raw` at :136/:141/:150, `LinkSpec::RAW =
true` at :916, and the `db.write_conn()` branch at :364 → the flush runs on the
pooled driver; drop the manual `BEGIN`/`BEGIN CONCURRENT` at :367 → the driver's
transaction), `crates/xray-tui-db/src/database.rs` (`Database::write_conn` →
DELETE with its `open_direct_conn` wiring at :411/:444; `direct` → KEEP;
`apply_link_patches_tx` / `exec_link_upsert` / `upsert_*_bulk` → pooled driver),
`crates/xray-tui-db/src/endpoint_rank.rs` (`write` → pooled driver).
**Change:** the seam + the WriteBehind RAW half existed ONLY as the statement-cache
workaround, which T0b.1 removed at the driver. Delete them; the bulk writers keep
literal inlining but run on the POOLED driver, and `LinkSpec` flushes through it.
**The batching/off-task mechanism STAYS** (spec §12.5): the `stage()`/coalesce/
window behaviour is untouched — only the raw-connection half goes.
**Compatibility:** same SQL, same rows; writers keep literal inlining.
**Verification:** `cargo nextest run -p xray-tui-db` (all write/parity tests pass
unchanged) + `cargo nextest run --workspace`.

**S0b exit:** the cache/routing test passes; the seam + `write_conn` + manual
`BEGIN` are gone; the workspace suite is green.

---

## Deferred — S1–S3 (adapter + `xray-tui-db-macro`)

**Excluded from this plan by user decision (2026-10-08) to reduce scope.** The
design is retained in the spec (§3–§4, D1/D3/D4) and can be executed later:

- **S1** — `xray-tui-db-macro`: `#[derive(FromRow)]`/`Model`/`DbEnum` +
  `Defer<Json<T>>` (simd-json decode).
- **S2** — `adapter/` + port `profiles_query::Projection` + `export::decode_row`.
- **S3** — `Model` batch builders over the existing raw writers.

**None is a dependency of S4/S5/S6:** the hand decoders (`profiles_query.rs`,
`export.rs`) and the raw bulk writers (`exec_link_upsert`, `upsert_*_bulk`)
stay AS-IS. Re-scope later only if that raw-decode duplication becomes a
maintenance cost the driver fork does not address.

---

## S4 — Migrations own the schema (highest value; depends on S0a/S0b)

### T4.1 — `schema/ddl.rs`: ALL DDL, versioned

**Files:** `crates/xray-tui-db/src/schema/{mod.rs,ddl.rs}` (new).
**Change:** one `MIGRATIONS: &[Migration]` list **SEEDED at `version 18`** (the
current `SCHEMA_VERSION` — NOT 1): `Migration { version: 18, statements: V18_TABLES }`
holding **11 CREATE TABLE** (PK autoindexes implicit), **6 explicit secondary
indexes** (3 re-stated from `#[index]`, 3 moved from `endpoint_ip.rs:49` /
`endpoint_rank.rs:401,439`), the **2 `ALTER TABLE ADD COLUMN`**
(`endpoint_rank.rs:428,471`), and the `rank_weight_meta` table (`:435`) — so an
existing tag-18 file is already at 18 (no-op) and a fresh file applies 18. Every
later change is 19+. Delete `endpoint_ip::ensure` / `endpoint_rank::ensure*` DDL.
An index change is a NEW migration (`DROP INDEX` + `CREATE`), never an
edit-in-place.
**Verification:** `sqlite_master` lists exactly the tables/indexes in `ddl.rs`
(a statement-vs-schema drift test); a tag-18 file opens as a no-op.

### T4.2 — Migration runner + `push_schema` body swap

**Files:** `crates/xray-tui-db/src/schema/mod.rs`, `driver/conn.rs`
(`push_schema`), `crates/xray-tui-db/src/database.rs` (open/in_memory).
**Change:** `schema::migrate(conn)` reads the `user_version` cursor, applies
pending migrations, sets the tag; `driver::push_schema` calls it (so
`Database::open`'s existing call site is unchanged). `Step::Rebuild` for
identity re-keys (`app_meta.identity_version`); `rank_weight_meta` handling
unchanged.
**Verification:** a fresh file → latest tag; an older-tag file → **migrations
applied, no wipe** (reopen preserves data).

### T4.3 — Re-express incompatible-file detection

**Files:** `crates/xray-tui-db/src/database.rs` (`open_with_concurrent_writes`
:320-370).
**Change:** the old signal was `push_schema()` **failing** (its `CREATE TABLE`
had no `IF NOT EXISTS`); a migrating `push_schema` returns `Ok`, so detect
incompatibility by an EXPLICIT tag/schema check — an unsupported/foreign tag or
a schema-shape probe → wipe & recreate; a supported-but-older tag → migrate.
**Verification:** a test opening a foreign-tagged file asserts the wipe/recreate
path fires; an older-supported-tag file migrates instead.

### T4.4 — Docs sync (decision 4/11)

**Files:** `AGENTS.md` (decision 4, 11), `docs/database.md`,
`docs/database-manual-sql.md` (new sites: the migration list).
**Change:** schema changes are migrations; identity re-keys are `Step::Rebuild`;
the file is no longer deleted for a mere column add.
**Verification:** doc read-back; no code.

---

## S5 — T12 FK cascade (REJECTED — see the execution notes)

> **Not executed as written.** Implemented, reproduced as a delete-race wedge,
> and reverted; the manual ordered deletes stay the cascade owner. The task
> below is kept only as the record of what was attempted.

### T5.1 — FK cascade DDL + per-connection pragma

**Files:** `crates/xray-tui-db/src/schema/ddl.rs` (the FK `CREATE TABLE`
variants), `crates/xray-tui-db/src/database.rs` (`conn()`: `foreign_keys=ON`),
the manual ordered deletes (`delete_endpoints`, `purge_expired`, `delete_group`,
`endpoint_ip::delete_for`).
**Change:** `REFERENCES … ON DELETE CASCADE` on `endpoint_ip.endpoint_id`,
`profile_stats(protocol_id, endpoint_id)`, `endpoint_groups(endpoint_id,
group_id)`, `endpoint_rank.endpoint_id`; add the pragma to `conn()`; delete the
manual ordered deletes so the DB owns referential integrity.
**Verification:** a cascade fires **through a pooled connection** (not only
`open()`'s). T14 (`WITHOUT ROWID`) stays blocked — do NOT carry it.

---

## S6 — MVCC re-measurement on 0.8

### T6.1 — A/B on a real-feed copy

**Files:** `crates/xray-tui/src/ops/ping/flow_cost.rs` (the ADR-0008 lab).
**Change:** re-run the WAL vs MVCC arms on 0.8 with
`experimental_mvcc_passive_checkpoint` enabled (the fork's builder exposes it),
reporting total wall time, p50/p95/p99 wait, conflicts/retries, checkpoint
viability.
**Change gate:** only if the measurement beats the retired 1.2–4.8× tax does
`XRAY_TUI_TURSO_CONCURRENT_WRITES` default flip; otherwise WAL stays default and
this task closes with the recorded number. **Outcome: no flip** (execution notes).
**Follow-up (only if the default flips):** spec §12.5 — revisit the
`WriteBehind` *flush cadence* under MVCC. Do NOT drop the layer: `stage()` is
non-blocking on the UI task and the batch pipeline wants one transaction per
window for link-write + rank-refresh atomicity. Pre-S6, `WriteBehind` is
untouched.
**Verification:** the lab's AFTER-numbers block + a written result.

---

## S7 (unplanned) — Full toasty removal

**Unplanned, not merely optional:** its enabler (the adapter + `xray-tui-db-macro`,
S1–S3) is DEFERRED, so S7 cannot proceed without building S1–S3 first. Bucket:
replace toasty's `Load` layer + the 68 typed CRUD sites with the adapter/macro;
retarget `db_monitor` to an adapter-emitted `db::query` event (only then does the
`toasty::query` target change). Not planned here; the fork (`driver/`) survives
this step, so it is not throwaway.

---

## Verification (whole plan)

- Per stage: `cargo nextest run --workspace` + `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`.
- S0a: `cargo nextest run --workspace` byte-identical + `cargo tree` single turso core + `just quality-gate`. S0b: the cache/routing test.
- S4: fresh + older-tag migration tests + the statement-vs-schema drift test +
  the foreign-file test.
- S5: cascade-through-`conn()` test.
- S6: the real-feed A/B.
- Closeout: `just quality-gate`.

## Risks

- **Driver drift** (toasty is 0.x): a `toasty-core` bump can break the fork
  before we can take it. Retirement trigger: upstream fixes the cache leak AND
  ships a turso-0.8 driver.
- **S4 destructive change**: the wipe becomes an explicit `Step::Rebuild`; a
  supported older tag migrates, never wipes.
- **Test port cost**: `xray-tui-db/tests/*` reference `toasty::*` (~105 hits in
  2 files) only where the driver swap touches them; mechanical, 1:1.
- **(Deferred) adapter/macro/simd-json**: not scoped here; see the deferred
  section.

## Retirement

| Retired | Trigger |
| --- | --- |
| `sql_exec.rs::SqlConn`/`RawConn`/`raw_turso_error` | S0b (the fork's `RawSql` arm is uncached) |
| `Database::write_conn` | S0b (same) |
| manual `BEGIN`/`BEGIN CONCURRENT` in `write_behind.rs` | S0b (moved into the driver) |
| `toasty-driver-turso` dep | S0a |
| `endpoint_ip::ensure` / `endpoint_rank::ensure*` DDL | S4 (into `ddl.rs`) |
| `push_schema`'s `create_table` body | S4 (→ `schema::migrate`) |
| the manual ordered deletes | S5 (FK cascade) |
| toasty's `Load` layer + typed CRUD | S7 (unplanned — needs the deferred S1–S3) |

## Execution route

`Execution route: inline (aegis:executing-plans).` S0a (pure dependency swap)
and S0b (contained behaviour change) are sequential; S4 depends on S0a/S0b's
`push_schema` body; S5/S6 are small. No genuinely independent parallel slices
that would pay for subagent coordination.
`User confirmation required: no` — no authorization, privacy, paid-resource, or
irreversible boundary is opened by S0a/S0b; S4's migration change is pre-approved
by the spec (wipe remains available via `Step::Rebuild`).

## Execution notes

### S0a — SHIPPED (2026-10-08)

Landed: `crates/xray-tui-db/src/driver/{mod.rs,error.rs,value.rs}` — a
local-only fork (~700 LoC incl. docs) vendored from `thirdparty/toasty` @
`4f3ed1a` (already on `turso = "0.8"`), with the sync/serverless/remote arms
dropped. Manifest: `toasty-driver-turso` removed; `toasty`'s `turso` feature
removed; direct `toasty-core = "0.11"` / `toasty-sql = "0.11"` / `turso = "0.8"`
/ `async-trait` added; hakari regenerated. All 9 required trait methods
implemented; `push_schema` still ports `create_table` UNCHANGED (fresh file gets
all 11 tables).

**Deviation, deliberate:** `Operation::RawSql` → uncached `prepare` and
`Insert`/`QuerySql` → `prepare_cached` (T0b.1) landed WITH the fork rather than a
commit later — the routing is the fork's content, and authoring the driver twice
(cached, then routed) is pure churn. The suite below proves behaviour
equivalence, so the isolation S0a was meant to buy is retained in effect; only
the commit boundary moved.

Evidence:
- `cargo nextest run -p xray-tui-db` — **176 passed, 2 skipped**.
- `cargo nextest run --workspace` — **2271 passed, 19 skipped**.
- `cargo tree -p xray-tui-db` — **ONE turso core** (`turso v0.8.2`); zero
  `toasty-driver-turso`.
- **0.7→0.8 on-disk format gate (PASSED).** Built a throwaway 0.7.2 binary,
  wrote a real file (WAL, `read_version=2`, tag 18), opened it with 0.8 and
  asserted the row reads back and the tag survives a reopen with **no wipe**.
  The throwaway builder + test were deleted after the run.
- `cargo clippy -p xray-tui-db --all-targets --all-features` — the **driver adds
  zero** diagnostics (13 remain, all pre-existing in `database.rs` /
  `profiles_query.rs` / `tests/`).
- `rustfmt` — the driver files are clean (the repo is broadly fmt-dirty
  pre-existing; only the new files were normalized).
- `cargo hakari generate --diff` (clean) + `manage-deps --dry-run` (no
  operations) + `verify` (ok).
- `cargo audit` — **3** allowed warnings: `rustls-pemfile` unmaintained (lock-only,
  toasty's optional postgres driver) and `lru` 0.16.4 unsound (turso→tantivy), both
  pre-existing, **plus `yoke-derive 0.8.3` yanked** — new here, arriving with the
  turso 0.8.2 bump via `turso_core → icu_* → yoke` (a registry warning, not a
  RUSTSEC advisory; build-time proc-macro, no fixed version reachable while ICU
  2.3 pins the major). All three triaged in `docs/crypto-dependencies.md` §4.
  `cargo deny check advisories` — ok. `cargo machete` — only the pre-existing
  `xray-tui-config → xray-tui-core`.

### S0b — SHIPPED (2026-10-08)

Landed: `Operation::RawSql` → UNCACHED `prepare`; `Insert`/`QuerySql` →
`prepare_cached` (the routing is the fork's, verified against the real
`toasty::sql::query`/`statement` builders, which construct `Operation::RawSql`).
RETIRED, all in this stage: `crates/xray-tui-db/src/sql_exec.rs` (the whole
`SqlConn`/`RawConn`/`raw_turso_error`/`from_turso_value` seam — deleted),
`Database::write_conn` (+ its construction and accessor), `CacheSpec::RAW`,
`write_window_raw`/`refresh_raw`, `LinkSpec`'s raw impl, and the manual
`BEGIN`/`BEGIN CONCURRENT` in `WriteBehind::flush`. The bulk writers
(`apply_link_patches_tx`, `exec_link_upsert`, `upsert_*_bulk`,
`endpoint_rank::refresh`/`write`) now take `&mut impl toasty::Executor` and run on
the pooled driver, keeping literal inlining. `Database::direct` STAYS (the page's
execution-layer bypass). Contention classification MOVED to
`driver::error::classify_turso_error` with its test restored
(`contention_is_retryable_and_real_errors_are_not`).

**The discriminating guard.** `crates/xray-tui-db/tests/cache_routing.rs` drives
400 unique-text `RawSql` statements in the PAGE-PROJECTION shape (200 ids inlined
per text) on one pooled connection and asserts RSS plateaus. A tiny synthetic
`INSERT` does NOT discriminate (400 of them fit under any bound — the first
version of this guard passed even with the leaky routing); the page shape does.

Evidence (each side measured):
- **Cached routing** (the leak, injected to prove the guard): grew **59,336 KiB** —
  the guard FAILS, as intended.
- **Uncached routing** (shipped): grew **0 KiB** over the same 400 texts — PASS.
- `cargo nextest run -p xray-tui-db` — **177 passed, 2 skipped**.
- `cargo nextest run --workspace` — **2272 passed, 19 skipped**.
- `cargo clippy -p xray-tui-db --all-targets --all-features` — the driver adds
  **zero** diagnostics (13 remain, all pre-existing per `git blame`).
- Docs synced to the new owner: `AGENTS.md` (the `sql_exec.rs` row → the driver
  fork; the architecture line), `CONTEXT.md:136`, `ARCHITECTURE.md`,
  `ROADMAP.md`, and `docs/database-manual-sql.md` (the `SqlConn` row → the driver
  fork; the leak section's "Fix" rewritten; the "import window still open" note
  resolved — the raw/toasty two-tx split no longer exists).

**Deletion.** `sql_exec.rs` (139 lines) removed; the seam's job is now the
driver's. No test was lost: the one `sql_exec.rs` test
(`raw_busy_errors_are_retryable`) is restored as
`driver::error::tests::contention_is_retryable_and_real_errors_are_not`, and the
write-behind parity test became `file_db_link_window_writes_the_staged_values`
(there is one write path now, so a raw-vs-pooled comparison is moot).

### S4 — SHIPPED (2026-10-08)

Landed: `crates/xray-tui-db/src/schema/{mod.rs,ddl.rs}` — the schema's single
owner. `PRAGMA user_version` is now a migration CURSOR, not a wipe tag, and the
runner has four explicit outcomes: `Current` (cursor == `LATEST`; ensure the
idempotent raw DDL, no table touch), `Applied` (cursor 0; create the tables then
the raw DDL, set the cursor), `IncompatibleSchema` (any other cursor — a
pre-migration or foreign file, reported so `open` applies its documented wipe),
plus a `push_schema`-failure mapping to `IncompatibleSchema` for an untagged
foreign file (a `CREATE TABLE` without `IF NOT EXISTS` failing IS the signal).

**Design choice — the migration SEEDS at the current tag, and tables stay with
`push_schema`.** Two decisions the spec left open, both resolved toward "one
owner, no second spelling":

1. The cursor seeds at `SCHEMA_VERSION` (18), NOT 1. An existing tag-18 file is
   therefore already current and a fresh file applies 18 — no data touched
   either way; every later change is 19+.
2. The `CREATE TABLE` statements are NOT copied into `ddl.rs`. A toasty model's
   table is emitted by `push_schema` from the model definition, and that IS the
   schema of record until the typed layer is retired (the deferred S7). Hand-
   writing 13 long `CREATE TABLE`s would create a second spelling of the same
   fact with no reader forcing them to agree. So the v18 path calls
   `push_schema` for the tables and `ddl.rs` for everything toasty cannot
   express — which is exactly the raw set that was scattered across
   `endpoint_ip.rs` and `endpoint_rank.rs`, now in one file with its causes.

`ddl.rs` holds: `endpoint_ip_by_key`, `endpoint_rank_key`,
`endpoint_rank_band_window`, `rank_weight_meta`, and the two
`ALTER TABLE endpoint_rank ADD COLUMN`s (`band`, `rank_weight`) — which are
NOT idempotent (no `IF NOT EXISTS` on this engine), so the runner tolerates the
duplicate-column error and propagates any other. Deleted: `endpoint_ip::ensure`
and the DDL half of `endpoint_rank::ensure_in` (its DATA half — repair, band
backfill, weight recompute — stays).

Evidence:
- `cargo nextest run -p xray-tui-db` — **182 passed, 2 skipped**.
- `crates/xray-tui-db/tests/schema_migrate.rs` (new, 5 tests): `LATEST ==
  SCHEMA_VERSION`; a fresh file gets the tables AND both raw columns AND the
  three raw indexes; a reopen of a current file preserves data (a no-op); three
  consecutive reopens tolerate the duplicate-column ALTER; an unknown cursor
  (17) is RECREATED, not migrated (the seeded row is gone).
- `open_wipes_a_file_with_a_mismatched_schema_tag` (pre-existing) now exercises
  the `IncompatibleSchema(8)` path; `open_recreates_incompatible_schema` the
  untagged-foreign path.
- `open_reopen_preserves_data` / `route_probes_survive_reopen_via_schema_tag`
  unchanged and green — the "reopen does NOT wipe" acceptance.

### S5 — PROBED AND REJECTED (2026-10-08), with an empirical reproduction

`REFERENCES … ON DELETE CASCADE` on the derived child tables was implemented
(the crate's own driver fork splices a TABLE-LEVEL FK into each
`CREATE TABLE`, since toasty's db `Schema` carries no relation info and
`push_schema` cannot emit one) and then **reverted**: the cascade creates a
delete-race WEDGE that the manual ordered deletes do not have.

The reproduction (`crates/xray-tui-db/tests/foreign_keys.rs`, written, run, then
deleted with the revert): seed an endpoint, `delete_endpoints` it, then run the
DEFERRED country write for it — the same ordering the geo queue produces, because
`ops::enrich::queue_country` pushes into the `WriteBehind<CountrySpec>` driver
with **no existence re-check** and the flush runs up to `GEO_DRAIN_INTERVAL`
(5 s) later. Measured output:

```
country_write_for_a_deleted_endpoint_is_reported ... GEO AFTER DELETE: Err(toasty error: FOREIGN KEY constraint failed)
```

`endpoint_ip::set_country`'s MISSING-row arm CREATES the row ("the lookup and
the address write race by design"); before the FK that was a harmless insert,
with the FK it is a violation → `WriteBehind::flush` re-stages the window AND
every later one and returns `Err`, permanently wedging ALL country writes.

`endpoint_rank` has the same defect by a different route: `LinkSpec::refresh`
runs `endpoint_rank::refresh` → `write` (`INSERT OR REPLACE INTO endpoint_rank`)
INSIDE the write-behind transaction for every touched endpoint, so a link patch
that races a delete inserts a rank row for a dead endpoint — the LINK writer
(every ping result) then wedges the same way. (`apply_link_patches_once`'s
post-commit refresh swallows the error, so that path degrades to a silently
missing rank row instead — also wrong.)

**What this does NOT change:** the manual ordered deletes already provide the
cascade correctly and transactionally — `delete_endpoints_once` and
`purge_expired_once` run `endpoint_ip::delete_for` + `endpoint_rank::prune`
inside the SAME transaction that deletes the endpoint. FK cascade was a belt to
those braces, and the belt is the thing that breaks.

**Disposition:** T12 is REJECTED, not deferred. Do not re-add these FKs without
also making every rank/address writer skip a missing parent — and note that
would be MORE code than the manual deletes it would replace. Recorded in the
spec §2 D6 and ADR 0011. `profile_stats`/`endpoint_groups` FKs were also
rejected: their parent is not guaranteed to exist first (the import writes links
and group links independently of their parents — 9 fixtures failed on it), so a
constraint there is simply wrong.

### S6 — MEASURED (2026-10-08): WAL stays the default on turso 0.8

A/B on turso 0.8.2 with the ADR-0008 lab (`flow_cost_contention`), a synthetic
8,000-endpoint feed seeded twice (WAL file + a fresh MVCC file, since 0.8 cannot
convert in place), two independent runs per arm, `REPS=3` per row. The trickle
row is one wall-clock measurement per run; the import arms FAIL in BOTH arms
(the documented lab limitation — its slice comes from `load_page_rows`, which
filters to protocols whose deferred `config` is loaded, and `upsert_protocols_bulk`
refuses a protocol with an unloaded config), so the import rows are DISCARDED and
only the geo rows are a valid A/B.

| row (ns/op) | WAL run 1 | WAL run 2 | MVCC run 1 | MVCC run 2 | verdict |
| --- | --- | --- | --- | --- | --- |
| seq geo flush BATCHED (100 rows) | 3.64 ms | 3.83 ms | 5.04 ms | 4.18 ms | **MVCC 1.1–1.4× slower** (reproducible) |
| seq geo PER-ADDRESS | 123 µs | 128 µs | 193 µs | 149 µs | **MVCC 1.2–1.6× slower** (reproducible) |
| fan-in geo PER-ADDRESS / 16 writers | 3.41 ms | 3.10 ms | 1.12 ms | 10.01 ms | **inconclusive** — MVCC 1.12–10.01 ms (n=3, noisy) vs WAL's stable ~3.1–3.4 ms |
| flush trickle (4028 rows @ 12/s) | 32 flushes | 32 flushes | 32 flushes | 32 flushes | identical (write-behind floor unchanged) |

**The `write failures` line this section first reported (408/408 vs 468/470) is
NOT an MVCC write-loss signal.** The counter (`flow_cost.rs:2381`) sums the
import, geo AND mix arms. Two contributions, neither an MVCC defect:

1. The **import arm fails in BOTH modes** — `write_import_once` calls
   `upsert_protocols_bulk`, which rejects a protocol whose deferred `config` was
   not loaded (the slice comes from `load_page_rows`, which stopped issuing
   `.include()` in `fcf2a5f`). That is ~1 failure per sample in WAL and MVCC
   alike, which is exactly why WAL prints `FAILED ON ALL 512 SAMPLES`.
2. MVCC's EXCESS is the **overlapping geo writes** of the mix arm (below), not a
   capacity limit on disjoint rows.

**The lab's MVCC failure count is 16-way row OVERLAP, not the production
shape — and NOT retry exhaustion on disjoint rows.** The mix arm hands every one
of its 16 writers the SAME geo slice, so all 16 transactions touch the same
`(endpoint_id, ip_key)` rows; a corrected probe
(`database::tests::geo_under_mvcc_probe`, which now builds GENUINELY disjoint id
blocks — an earlier version of it accidentally `% 64`-folded every batch onto the
same 64 rows and its "conflicts on disjoint rows" reading was that artifact)
measures the two shapes apart at 16 writers × 24 transactions × 8 rows:

| geo writers (16×) | WAL exhausted | MVCC exhausted | wall |
| --- | --- | --- | --- |
| disjoint (production shape: distinct endpoints) | 0 | **0** | MVCC 52–147 ms vs WAL 451 ms — MVCC FASTER |
| overlapping (all writers on 8 rows) | 0 | **0** (was 0–1 before the `ON CONFLICT` fix) | MVCC 124–211 ms vs WAL 36–43 ms |

At production geometry (1–2 geo writers, the geo drain being ONE `WriteBehind`
owner) it is 0 exhausted at the generic 5-attempt budget in every run. So
**no retry-budget change is shipped** — an unjustified constant is worse than the
status quo — and a write that IS exhausted is not lost anyway: `WriteBehind::flush`
calls `restage` on any error, so the window is written on the next drain.

**Separate probe — do MVCC conflicts lose writes?** No, at the budget the app
actually uses. `database::tests::mvcc_load_probe` (ignored; 32 writers × 60
transactions, 2 runs) wraps the tx-scoped write in **ONE** `retry_on_busy(…, 5)` —
production's exact budget; nesting the public `apply_link_patches` would have
measured 25 attempts and proved nothing. Result: **`retry_exhausted = 0` on the
DISJOINT rows, WAL and MVCC alike**, with MVCC the faster arm in most runs
(84–195 ms vs WAL's 192–775 ms) but not all — the spread exceeds the difference,
so call the wall times parity. On the single-row arm MVCC is consistently slower
(291–532 ms vs 227–385 ms) and exhausts at most **1 of 1920**, which
`WriteBehind::flush` re-stages.
(An earlier version of this probe `% 64`-folded every writer onto the same 64
rows, so its "disjoint" label was wrong; the fixture now gives each writer a
distinct id block.)

**Decision: MVCC stays an OPT-IN — the reason is throughput and tooling, not
correctness, and the verdict is weak.** Correctness is clear on every shape
measured: 0 write loss on disjoint rows, on the production 2-writer overlap, and
even the single-row case loses at most 1 write — which `WriteBehind::flush`
re-stages rather than drops. MVCC wins on the shape production has (disjoint rows)
and loses on heavy overlap and on the geo mixed stream (1.1–1.6×). What actually
keeps WAL the default is the **on-disk-format cost**: an MVCC file is unreadable by
stock SQLite tooling and the conversion is one-way. **The trigger for a different
answer is a real-feed A/B with more repetitions** — not a code change. This is
materially weaker than the 2026-09-24 rollout's 1.2–4.8× tax, so a future run may
well flip it; it is a live question, not a closed one.

**Follow-up landed with this measurement: the MVCC checkpoint gap is FIXED, and
two MVCC-only geo defects were fixed.** (a) `driver::error::classify_turso_error`
matched `"conflict"` case-sensitively, so the engine's `Conflict: {0}` spelling
classified as a hard error and was NEVER retried at any budget; it is now
case-insensitive, pinned with both spellings. (b) `set_country` was a
SELECT-then-UPDATE-or-CREATE TOCTOU that under MVCC let two writers both INSERT
the same PK — a `Constraint`, which is not a busy error, so no retry budget could
help; it is now one `ON CONFLICT … DO UPDATE` (also dropping the N reads per geo
window), pinned by `set_country_is_an_atomic_upsert`. After both, the production
2-writer overlap measures 0 exhaustion in both journal modes.
`PRAGMA wal_checkpoint(PASSIVE)` was rejected under MVCC
(`PASSIVE checkpoint requires experimental_mvcc_passive_checkpoint`), and
`ping.rs::wal_checkpoint_enabled` was `!concurrent_writes` — so an MVCC database
ran with NO checkpoint and grew its logical log unbounded (59 KiB after 500 rows,
still climbing; 0 with the flag). `file_driver` now enables
`experimental_mvcc_passive_checkpoint` with the MVCC opt-in and the gate is open
in both modes. Pinned by
`database::tests::mvcc_checkpoint_succeeds_on_the_open_path`, which fails with the
engine's own error when the flag is removed. **MVCC is therefore viable; the
default remains WAL on the on-disk-format and throughput grounds above.**

**Deviation from T6.1's stated method (recorded).** The plan asked for
`experimental_mvcc_passive_checkpoint` and for p50/p95/p99 wait plus checkpoint
viability. The FIRST RUN did neither — the flag was unreachable (`file_driver`
called only `.concurrent_writes()`) and the lab reports ns/op per arm, not wait
percentiles. **The checkpoint half is now closed**: a follow-up measured the
statement directly (see the "MVCC checkpoint gap is FIXED" block above),
`file_driver` enables the flag, and the gap is pinned by a real assertion
(`mvcc_checkpoint_succeeds_on_the_open_path`). The wait-percentile rows are still
not produced, and are not needed for a verdict that turns on the checkpoint,
on-disk format and throughput.

**Honest limits.** Synthetic 8,000-endpoint feed, not the 74k reference; two runs
per arm for the geo rows; the fan-in row is n=3 per run; no real-feed copy was
available in this environment, so "the 74k feed behaves the same" is an
inference, not a measurement.

**What 0.8 DID buy** (independent of this decision): the crate now owns its
driver, so the statement-cache leak is fixed at its source, `turso` is pinned
directly on ONE core, `experimental_mvcc_passive_checkpoint` is reachable, and
the schema is migrated rather than wiped. The MVCC default was never the
justification for the upgrade.

### S7 — unplanned (deferred enabler)

