# DB adapter, model macro, and owned migrations — Design Spec

Status: **rev. 1 — Option D core APPROVED for planning; D1/D3/D4 (adapter +
`xray-tui-db-macro`) DEFERRED by user decision (2026-10-08)**. The in-scope
design is the driver fork (§12), migrations (§5), FK cascade, and the MVCC
re-measurement; the adapter/macro design is retained here for a later slice.
Date: 2026-10-08.
Supersedes/extends: ADR 0011 (db-rewamp — the shipped identity/binned-law slices),
ADR 0001 (page raw SQL), ADR 0002 (write-behind), ADR 0003 (rank keys),
`docs/database-manual-sql.md` (the raw-site authority), ADR 0010 (band).
Authority: `docs/database.md`, `docs/database-manual-sql.md`, `AGENTS.md`.
Evidence: repo inventory 2026-10-08 (below); turso 0.8.2 source probes; the
"Toasty removal — research" findings (this spec resolves them as Option D —
own the driver, keep toasty — with S7 (full removal) unplanned, gated on the
deferred S1–S3).

## 1. Problem

`xray-tui-db` is ~11,900 LOC of source + 4,235 LOC of tests built on toasty
0.11 (`toasty-driver-turso` 0.11 → **turso 0.7.2**). Five measured problems:

1. **The typed layer is the cost, not the plan.** The Profiles page costs ~98 ms
   through toasty vs ~1.2 ms for identical raw SQL (AGENTS decision 23); the
   page already bypasses it (T13, `page_ids_direct` + direct reader). The
   surviving typed paths carry the same per-row/per-bind tax.
2. **Raw SQL is scattered.** 27 production raw-SQL sites in `xray-tui-db`
   (+3 in the TUI crate), spread across `database.rs`, `endpoint_rank.rs`,
   `endpoint_ip.rs`, `profiles_query.rs`, `sql_exec.rs`. Every one is a
   deliberate exception with its own cause (ADR list + manual-SQL doc), which
   means toasty is NOT the interface — it is a partial layer with holes.
3. **DDL/index ownership is split.** Table DDL is `push_schema`-emitted from 11
   `#[derive(Model)]` structs (composite PKs, enum/bool `CHECK` constraints); the
   secondary indexes are **3 model `#[index]`** (`profile_stats.endpoint_id`,
   `profile_stats.last_seen_at`, `endpoint_groups.group_id`) plus **3 raw
   `CREATE INDEX`** (`endpoint_ip_by_key`, `endpoint_rank_key`,
   `endpoint_rank_band_window`) and **2 raw `ALTER TABLE ADD COLUMN`** plus the
   raw `rank_weight_meta` table (`endpoint_ip.rs:49`, `endpoint_rank.rs:401,428,435,439,471`).
   Table+index truth is in two places and neither lists the other.
4. **The schema tag is a wipe, not a migration.** `PRAGMA user_version` gates
   `push_schema`; a mismatch makes `open` DELETE the db file (decision 4).
   There is no additive path (add column / add index / add table) — every
   change wipes.
5. **turso 0.8 is blocked by the two-engine trap, not by a version rule.**
   `turso 0.7` and `0.8` are semver-incompatible (0.x); declaring `turso = "0.8"`
   beside `toasty-driver-turso 0.11` (pins `0.7`) links **two turso cores** onto
   one file — our `direct`/`write_conn` connections (0.8) vs the driver's pooled
   one (0.7). It COMPILES; it is wrong at runtime. 0.8 is reachable only by a
   toasty driver bump or by toasty leaving.

### 1.1 What turso 0.8 actually buys (WAL-default path)

Measured/verified, not assumed:

| 0.8 item | Relevance (WAL default) |
| --- | --- |
| group commit / `BEGIN CONCURRENT` | **MVCC only** — this repo A/B-measured MVCC 1.2–4.8× slower on the real-shaped feed and left it off (`specs/2026-09-24-turso-mvcc-rollout-design.md` §12) |
| `experimental_mvcc_passive_checkpoint` builder flag | **MVCC only** — but it lifts the checkpoint blocker that keeps MVCC from being a real option |
| partial-index predicates, `NULLS FIRST/LAST` indexes | could help the band sort — needs `ANALYZE` first (the live db has no `sqlite_stat1`); speculative |
| hash joins, window fns, recursive CTE | not our plan (the page is index-driven) |
| statement cache (ENGINE) | **still unbounded** in 0.8 (`turso_sdk_kit-0.8.x rsapi.rs:1136` `HashMap`) — toasty `main` STILL calls `prepare_cached` (`lib.rs:1254`), so upstream will not fix the leak |
| statement cache (OUR workaround) | **retired by the fork** (§12): the fork routes `Operation::RawSql` to the UNCACHED `prepare` (the leak boundary) and keeps `prepare_cached` for `Insert`/`QuerySql`, so `sql_exec.rs`'s `SqlConn` seam + `Database::write_conn` lose their rationale — they are DELETED, not kept (see §12.2) |
| `WITHOUT ROWID` DELETE/UPDATE | **still refused** (`turso_core-0.8.x translate/delete.rs:51`, `update.rs:261`) — T14 stays blocked |

Net: 0.8's headline does not touch our hot path. Its only possibly-meaningful
candidate was a **fresh MVCC re-measurement** under group commit — and S6 ran it:
the sequential tax is reproducible, the contended row is inconclusive across two
runs, so **WAL stays the default**. 0.8's value here is structural (the owned
driver, the leak fix, one turso core, migrations), not MVCC.

What 0.8 *does* unblock structurally is the version itself: owning our driver
(§12) lets us declare `turso = "0.8"` at S0a — after which the `Builder` flags
(`experimental_mvcc_passive_checkpoint` among them) are reachable, so MVCC
becomes a real option to measure rather than a blocked one.

## 2. Decisions

- **(DEFERRED, S1–S3) D1 — Own the result→model conversion; do not expose raw
  SQL to callers.** A `xray-tui-db` adapter module (traits + helpers) and a
  `xray-tui-db-macro` proc-macro crate provide `#[derive(FromRow)]` and
  `#[derive(Model)]`. Callers write SQL or call typed/batch builders; nobody
  hand-indexes `turso::Row`. **Deferred 2026-10-08** — the hand decoders
  (`profiles_query.rs`, `export.rs`) and the raw writers stay AS-IS.
- **D2 — ONE schema/migration owner.** Every table, index, and column lands in
  `schema/ddl.rs` as a versioned migration list. Replaces `push_schema` + the
  `PRAGMA user_version` wipe tag. Additive migrations are expressible; the
  pre-alpha wipe remains available but stops being the only option.
- **(DEFERRED, S1–S3) D3 — Batch is first-class.** The macro crate's `Model`
  derive emits the SQL conversion for single and batch contexts: `insert_batch`,
  `upsert_batch`, `update_batch` over `&[Self]`, one multi-row statement per N
  rows — formalizing the proven `LINK_UPSERT_PREFIX` / `exec_link_upsert` shape.
  **Deferred 2026-10-08** — the raw writers keep their current form.
- **(DEFERRED, S1–S3) D4 — Own the JSON codec (`Defer` + `Json<T>`, simd_json).**
  `Defer<T>` = column excluded from default SELECT, `Json<T>` = decoded on load;
  `Defer<Json<T>>` composes (the `protocols.config` column). simd_json is the
  decode path. Scoped as a **connect/batch** benefit (the only decode site is
  `load_protocol_with_config`, 67–121 µs/link), NOT a page win.
  **Deferred 2026-10-08.**
- **D5 — Option D: own the driver fork; turso 0.8 at S0a.** A local-only fork of
  `toasty-driver-turso` lives in `xray-tui-db/src/driver/` (§12), vendored from
  the dev clone (`thirdparty/toasty`, already on turso 0.8). S0a is a PURE swap
  (`exec` keeps `prepare_cached`); **S0b** routes `Operation::RawSql` → uncached
  `prepare` and `Insert`/`QuerySql` → `prepare_cached` (§12.2 — the exact leak
  boundary, no LRU). It declares `turso = "0.8"` directly. S0a keeps
  `push_schema` **functional** (ports toasty's `create_table` loop, the only
  creator of the 11 tables on a fresh file); **S4** swaps its body to
  `schema::migrate` (§12.4). Because the fork is the ONLY driver
  (`toasty`/`toasty-core`/`toasty-sql` do not pin turso), there is no two-engine
  trap and the bump lands at S0a. **D keeps toasty's engine** — it is NOT full
  removal; it wins on 0.8 + DDL ownership + the cache leak only (§12.3), and its
  turso-level half is reused if staged-A later follows.
- **D6 — T12 and T14 are both rejected/deferred, for different reasons.**
  T14 (`WITHOUT ROWID`) is **engine**-blocked: turso refuses DELETE/UPDATE on WR
  tables independent of who writes the CREATE (0.8.2 `delete.rs:51`,
  `update.rs:261`). **T12 (`REFERENCES … ON DELETE CASCADE`) was implemented and
  REVERTED** (S5, 2026-10-08): the driver fork CAN emit the clause (a table-level
  FK spliced into `CREATE TABLE`, since toasty's db `Schema` carries no relation
  info), but the cascade introduces a delete-race WEDGE. `set_country`'s
  missing-row arm CREATES its row (the geo queue pushes with no existence
  re-check, flushed up to 5 s later), and `LinkSpec::refresh` inserts rank rows
  inside the write-behind transaction — so a delete that races either makes the
  flush take the create path for a dead endpoint, raise `FOREIGN KEY constraint
  failed`, and `WriteBehind` re-stage the window and every later one forever.
  Measured: `GEO AFTER DELETE: Err(FOREIGN KEY constraint failed)`. The manual
  ordered deletes (`endpoint_ip::delete_for`, `endpoint_rank::prune`, both
  already inside the deleting transaction) provide the cascade correctly, so the
  FK is a belt that is the thing that breaks. **Not to be re-added without also
  making every rank/address writer skip a missing parent.**

**In-scope workstream (after the S1–S3 deferral): S0a + S0b (driver fork) → S4
(migrations) → S5 (T12 FK) → S6 (MVCC).** It still delivers the three measured
wins the investigation named: (1) the statement-cache leak fixed at its boundary
(+ the ~5 raw-connection workarounds deleted), (2) turso 0.8 unlocked on one
engine, and (3) DDL/index consolidation into one versioned owner.

## 3. Crate layout (DEFERRED — S1–S3, not planned)

> **Deferred 2026-10-08.** The `adapter/` module, `models.rs`, and the
> `xray-tui-db-macro` crate below are the design for the deferred adapter/macro
> slice. The in-scope plan (S0a/S0b, S4–S6) touches only `driver/` and `schema/`.
> Consequence: the config-JSON decode STAYS `serde_json` via toasty's `Json<T>`
> (no simd_json), and **S7 (full toasty removal) needs S1–S3 built first** — it
> is the enabler, so S7 is `unplanned`, not merely optional.

```
crates/xray-tui-db-macro/            # proc-macro crate (like toasty-macros)
  src/lib.rs
  src/from_row.rs                    # #[derive(FromRow)] + per-field attributes
  src/model.rs                       # #[derive(Model)]  (FromRow + TABLE/columns/insert/pk/batch)
  src/db_enum.rs                     # #[derive(DbEnum)] label mapping
crates/xray-tui-db/src/
  adapter/
    mod.rs                           # FromRow, FromValue, ToSql, Value re-exports
    exec.rs                          # query/exec/tx helpers over toasty's pooled conn
    defer.rs                         # Defer<T>, Json<T> (simd_json)
    batch.rs                         # batch builders (multi-row INSERT/ON CONFLICT)
  driver/                            # §12 — local-only fork of toasty-driver-turso
    mod.rs                           # Turso (builder) + Connection, impl toasty Driver
    conn.rs                          # exec: RawSql→uncached prepare, Insert/QuerySql→cached; QueryLog kept; push_schema = create_table loop (S4 swaps to schema::migrate)
  schema/
    mod.rs                           # migration runner (read tag, apply pending)
    ddl.rs                           # ALL tables + indexes + ALTERs, versioned
  models.rs                          # #[derive(Model)] row structs (replaces models_toasty.rs)
```

The published `toasty-driver-turso` dependency is DROPPED (§12); the fork is
registered with `Db::builder().models(…).build(our_driver)`. The `toasty` feature
list must DROP `turso` (else upstream's driver returns via the feature flag).

Existing modules (`profiles_query.rs`, `endpoint_rank.rs`, `endpoint_ip.rs`,
`write_behind.rs`, `export.rs`) keep their SQL. **`sql_exec.rs::SqlConn` is
RETIRED (§12.2):** its only purpose was the uncached-`prepare` workaround for the
driver's statement cache, which the fork removes at the source — every caller
goes back to the pooled driver, and the `RawConn` arm + `Database::write_conn`
are deleted. `Database::direct` (the page's execution-layer bypass, a real ~80×
read win) STAYS.

## 4. Adapter + macro design (DEFERRED — S1–S3, not planned)

### 4.1 Traits (self-contained, no toasty-core, no serde_value)

```rust
pub trait FromValue: Sized { fn from_value(v: turso::Value, col: &str) -> Result<Self>; }
pub trait ToSql { fn to_sql(self) -> turso::Value; }        // bind/insert values
pub trait FromRow: Sized { fn from_row(row: &turso::Row) -> Result<Self>; }

pub trait Model: FromRow {
    const TABLE: &'static str;
    const COLUMNS: &'static [&'static str];                 // ordered, matches insert_values
    fn insert_values(&self) -> Vec<turso::Value>;
    fn pk(&self) -> Vec<turso::Value>;                      // key for update/delete
}
```

`FromRow` decodes by **column position** (SQL we write controls the SELECT),
with `#[column("name")]` for name-based `row.get(i)` when the query is
`SELECT *`. A name-mismatch is a `Result` error, not a panic (sqlx's
`try_get` discipline).

### 4.2 Derive attributes

| Attribute | Purpose | Replaces (toasty) |
| --- | --- | --- |
| `#[table = "…"]` | table name | `#[derive(Model)]` name default |
| `#[key(a, b)]` | PK columns | `#[key(…)]` |
| `#[column("name")]` | column rename | field-name rule |
| `#[db_enum("label1","label2",…)]` | stored label per variant | `as_db_label`/`from_db_label` |
| `#[defer]` on `Defer<Json<T>>` | excluded from default SELECT | `#[has_many]`/deferred column |
| `#[json]` on `Json<T>` | JSON text column | `#[column(type=text)]` + `Json<T>` |
| `#[flatten]` on an embed | inline an embed's columns | `toasty::Embed` |
| `#[version]` | OCC counter (update path) | `#[version]` |
| `#[skip]` / `#[default]` | not-in-row / default-filled | (new) |

### 4.3 Sharp edges the derive must own (verified against the models)

1. **`Latency`** shares one `delay` column across two variants
   (`#[shared(delay)]`) and `Real` adds `latency_ip`. SQLx's `flatten` covers
   structs, not shared-column enums → **hand-written** `FromRow`/`insert_values`
   for this one type (one enum). The `Model`/`FromRow` derives must allow a
   `#[model(manual)]` opt-out.
2. **Enum labels are toasty's snake_case-of-ident**, NOT `as_str()`: the feed
   stores `http_upgrade`/`x_http`, the wire form says `httpupgrade`/`xhttp`
   (`docs/database-manual-sql.md`, `proto_spec/kinds.rs`). `#[db_enum(…)]` makes
   the label explicit and testable; `HostType::as_db_label` is the precedent.
3. **`CHECK` constraints** are toasty-emitted for enum/bool columns; hand-owned
   DDL reproduces them in `schema/ddl.rs` (a wrong label is then refused at
   write, as today).
4. **Collections** (`Vec<String>`, `Vec<u16>`) are JSON text columns; the derive
   routes them through the same JSON codec as `Json<T>`.
5. **OCC `#[version]`**: the raw writer already does `version = version + 1`
   (`database.rs:738`); the update builder adopts it.

### 4.4 Batch API (D3)

```rust
Endpoint::insert_batch(&mut conn, &rows).await?;                 // INSERT … VALUES (…),(…)
ProfileStats::upsert_batch(&mut conn, &rows, OnConflict::pk(), &UPDATE_COLS).await?;
ProfileStats::update_batch(&mut conn, &rows, &LinkGroups::RESULT).await?;  // group-scoped
```

- One multi-row statement per `STATEMENT_ROWS` (400), matching the shipped
  writers; `ON CONFLICT` action bucketed by column group (ADR 0002) is preserved
  because the batch carries the group mask.
- Ids/keys from our own rows are inlined as literals where the measured win
  applies (C6, ~0.8 ms/bind); user text binds via `sql_lit`. The adapter exposes
  both; the caller picks by the ADR rule.
- Callers pass row-compatible structs (`&[Self]`), never tuples of `Value`.

## 5. Migrations (D2)

```rust
// schema/ddl.rs
pub struct Migration { pub version: i64, pub statements: &'static [&'static str] }

pub const MIGRATIONS: &[Migration] = &[
    // SEEDED at the CURRENT tag (18), NOT 1: an existing tag-18 file is already
    // "at 18" (no-op); a fresh file applies it; every later change is 19+.
    // 11 CREATE TABLE (PK autoindexes implicit) + 6 explicit secondary indexes
    // (3 re-stated from `#[index]`, 3 moved from the raw sites) + rank_weight_meta.
    Migration { version: 18, statements: V18_TABLES },
    // future: Migration { version: 19, statements: &["ALTER TABLE … ", "CREATE INDEX …"] },
];

pub async fn migrate(conn: &turso::Connection) -> Result<()>;  // cursor == latest → no-op; cursor < latest → apply pending; foreign tag → incompatibility path
```

- **One list, one place** — true from **S4**, when the fork's `push_schema` body
  becomes `schema::migrate`. Until then (S0a/S0b–S3) `push_schema` still emits DDL via
  toasty's `create_table` loop, so the list cannot yet be the single owner. At S4
  every table and every index — the 3 model `#[index]` (re-duplicated as explicit
  `CREATE INDEX` here), the 3 raw `CREATE INDEX`, the 2 `ALTER TABLE ADD COLUMN`
  and `rank_weight_meta` — lands here; the PK autoindexes are implicit in the
  `CREATE TABLE`/`PRIMARY KEY` clauses. `endpoint_rank.rs`/`endpoint_ip.rs`
  `ensure*` functions shrink to calls or disappear.
- **Idempotency without the silent-edit trap.** `CREATE INDEX IF NOT EXISTS` in
  an applied migration is a no-op on edit; instead an index change is a NEW
  migration (`DROP INDEX` + `CREATE`), so the version, not the statement text,
  carries the change.
- **The wipe stays but stops being the only path.** The list is SEEDED at
  `version 18` (the current `SCHEMA_VERSION`), so `SCHEMA_VERSION` becomes
  "the latest `MIGRATIONS.version`". A fresh file applies 18; additive changes
  are `19, 20, …`; an identity re-key is a `Step::Rebuild` entry, never a
  side effect of the cursor.

### 5.1 Policy collision: the wipe, identity re-keys, and the version stamps

Today ONE mechanism (`PRAGMA user_version` mismatch → delete file) serves three
jobs: schema creation (AGENTS decision 4), **identity re-keys**
(`IDENTITY_VERSION`, decision 11), and derived-state resets. Making migrations
the schema owner removes the wipe from the schema job — so the other two jobs
need explicit owners. Resolution:

- **Schema cursor SEEDS at the CURRENT tag, not 1.** `SCHEMA_VERSION` is `18`
  today. The FIRST migration entry is therefore
  `Migration { version: 18, statements: <the full current schema> }` — so an
  existing tag-18 file is already "at 18" (a **no-op**, no `CREATE TABLE`
  re-run, no wipe), a fresh file applies 18, and **every later change is 19+**
  (additive). Starting the list at 1 would make a real tag-18 file read
  "cursor 18 vs latest 1" and either re-run a non-`IF NOT EXISTS` `CREATE TABLE`
  (fail) or silently wipe an already-imported feed — unachievable as written.
  The runner's rule: `cursor == latest` → no-op; `cursor < latest` → apply the
  pending migrations (`> cursor`); an unsupported/foreign tag → the explicit
  incompatibility path (§5.1 below / T4.3).
- **Identity re-key (a DATA change, not a schema change).** `IDENTITY_VERSION`
  stays where it is (`xray-tui-proto::proto_spec::identity`, a wire-format
  constant — it is not a DB fact). The DB records which identity version its
  stored rows were keyed under in a NEW `app_meta` row (`identity_version`,
  reusing the existing `app_meta(key,value)` table — schema tag 16). On open, a
  mismatch means every stored uid is stale. Two migration step kinds express it:

  ```rust
  pub enum Step {
      /// Additive DDL/DML — applied in place, no data loss.
      Sql(&'static [&'static str]),
      /// The pre-alpha WIPE: the schema is sound but the DATA cannot be migrated
      /// (an identity re-key needs the uid hash recomputed in Rust, which SQL
      /// cannot do). Deletes the file and recreates, exactly as decision 4 does
      /// today — now an explicit, named step rather than a tag side effect.
      Rebuild,
  }
  ```

  `Rebuild` is reserved for identity re-keys and is surfaced (a log line naming
  `IDENTITY_VERSION old → new`), never silent. When the re-key can be done as a
  Rust re-write instead (recompute uids and UPDATE in one transaction), it is a
  future `Step::RustDoc(fn)` — the migration list gains a Rust hook; that is a
  later enhancement, not this spec.
- **`WEIGHT_VERSION` / `rank_weight_meta`.** NOT a migration input. It is a
  self-healing derived-state stamp: the TABLE DDL (`rank_weight_meta`, its
  CHECK) moves into the migration list like every other table/index, but the
  **value** stays a one-row stamp whose mismatch recomputes every rank key at
  open (`endpoint_rank.rs:497-522`). Rank keys are rebuildable from live rows, so
  a `WEIGHT_VERSION` bump is never a wipe — unchanged, just relocated DDL.
- **Consequence for decision 4/11 text.** AGENTS decision 4 ("a bump is NOT a
  migration") and decision 11 ("re-keying is a data reset") both describe the
  pre-migration world. After S4 they must say: schema changes are migrations;
  identity re-keys are `Step::Rebuild` (or a Rust data migration); the file is
  no longer deleted for a mere column add. The docs are updated in S4.
- **T12 does NOT land here — REJECTED (S5, 2026-10-08).** It was implemented
  (the fork can splice a table-level `REFERENCES … ON DELETE CASCADE`) and
  reverted: the cascade creates a delete-race WEDGE, because the geo queue's
  `set_country` has a CREATE arm with no existence re-check and `LinkSpec::refresh`
  inserts rank rows inside the write-behind transaction — so a delete racing
  either raises `FOREIGN KEY constraint failed` and `WriteBehind` re-stages
  forever. Reproduced: `GEO AFTER DELETE: Err(FOREIGN KEY constraint failed)`.
  The manual ordered deletes already own the cascade, transactionally. See D6.

## 6. Migration stages (incremental; each independently useful)

| Stage | Change | Green boundary | Value even if later stages stop |
| --- | --- | --- | --- |
| **S0a** | **Vendor the local-only driver** (from the dev clone, already on turso 0.8; COPY, never path-depend) into `src/driver/`; `exec` **keeps `prepare_cached`** (pure swap); `push_schema` **ports toasty's `create_table` loop UNCHANGED**; keep `concurrent_writes()` + `experimental_mvcc_passive_checkpoint`; MOVE `from_turso_value`/`to_turso_value` into `driver/value.rs`; register via `Db::builder().build()`; bump `turso → 0.8`; drop the `toasty-driver-turso` dep + the `turso` feature from `toasty`; add `async-trait` + direct `toasty-core`/`toasty-sql`; **regenerate the hakari hack crate** | db suite **byte-identical** + `just quality-gate` green | **turso 0.8 unlocked; single engine; the driver is ours** |
| **S0b** | Route `exec`: `RawSql`→uncached `prepare`, `Insert`/`QuerySql`→`prepare_cached`; DELETE `SqlConn`/`RawConn`/`raw_turso_error`, `Database::write_conn`, the manual `BEGIN` in `write_behind.rs` | cache/routing test + workspace suite | **cache leak dead; ~5 workarounds deleted** |
| **S1 (DEFERRED)** | `xray-tui-db-macro`: `FromRow` + `DbEnum` + `Defer`/`Json` | — | deferred 2026-10-08 |
| **S2 (DEFERRED)** | `adapter/` + port `profiles_query::Projection` / `export` decode | — | deferred 2026-10-08 |
| **S3 (DEFERRED)** | `Model` derive + batch builders (raw writers) | — | deferred 2026-10-08 |
| **S4** | `schema/` module: ALL DDL into the versioned list; the fork's `push_schema` body swaps from `create_table` → `schema::migrate` (open() unchanged); re-express "incompatible file" as an explicit tag/schema check (the old signal was `push_schema` FAILING) | open/migration tests | **DDL/index consolidation (the user's ask)** |
| **S5 (REJECTED)** | T12 FK cascade was tried and reverted — it wedges the geo/link write-behind on a delete race (see D6). The manual ordered deletes stay the cascade owner. | — | decided: no FK |
| **S6 (MEASURED)** | MVCC vs WAL A/B on 0.8 — **WAL stays the default** (the sequential tax is reproducible; the fan-in row is noise: 1.12 ms vs 10.01 ms across two runs). Details in the plan's execution notes. | `flow_cost` | decision: no flip |
| **S7 (UNPLANNED)** | Full toasty removal (adapter + macro replace the query layer) — **requires the deferred S1–S3** | — | not reachable without S1–S3 |

**Order:** S0a first (the pure dependency swap; the fork + turso 0.8), then S0b
(the cache routing + workaround deletions), then **S4** (migrations) — which is
the natural second slice once the driver is ours, since **S4 depends on
S0a/S0b** (the fork must exist to swap its `push_schema` body). The former "bump
turso LAST" constraint is GONE (§12.4): the fork is the only driver, so 0.8 lands
at S0a. **S1–S3 are DEFERRED (2026-10-08)** and are not on this critical path.

### 6.1 (DEFERRED) Scope boundary for S1–S3 — the RAW paths ONLY

**Not in the current plan.** Kept for the later adapter/macro slice: under D,
toasty **keeps its engine**, and its `#[derive(Model)]`/`Load` impls already do
result→model conversion for the typed query paths, so the `FromRow`/`Model` macro
would serve only the paths that are **already hand-decoded outside toasty**:

| In scope for S1–S3 | Out of scope (S7) |
| --- | --- |
| `profiles_query.rs::Projection` (hand decode, ~180 LOC) | toasty's own `Load` impls for typed queries |
| `export.rs`'s row decode | the 68 typed CRUD sites' result decoding |
| the bulk upsert/insert writers (`exec_link_upsert`, `upsert_*_bulk`) — formalized behind `Model::*_batch` | toasty's query builder / relation loading |
| raw `SELECT`s whose rows nobody types today | anything toasty currently reads correctly |

Replacing toasty's `Load` layer is **S7** — it happens only *with* the deferred
S1–S3 built first (they are its enabler), so S7 is `unplanned`, not merely
optional.

## 7. Ownership

| Change | Owner |
| --- | --- |
| (DEFERRED) `FromRow`/`Model`/`DbEnum` derives, `Defer`/`Json` | `xray-tui-db-macro` |
| (DEFERRED) traits, exec/tx, batch builders | `xray-tui-db/src/adapter/` |
| tables + indexes + ALTERs, migration runner | `xray-tui-db/src/schema/` |
| **driver fork** | `xray-tui-db/src/driver/` |
| turso bump + MVCC re-measure | `Cargo.toml`, `flow_cost.rs` |

## 8. Acceptance

- **S0a:** the fork registers via `build()` and the full db suite passes
  **byte-identically**; **a fresh file still gets all 11 tables** (the ported
  `create_table` loop is unchanged); `cargo tree` shows ONE turso core at `0.8`;
  `just quality-gate` green after the lock-changing manifest edits.
- **S0b:** the statement-cache routing test passes (a `RawSql` loop plateaus, a
  repeated `Insert` stays cached); the `SqlConn`/`RawConn` seam and
  `Database::write_conn` are DELETED while `direct` stays and the db suite passes.
- **D naming.** Option D (= S0a/S0b + keeping toasty) is a distinct choice from
  the full-removal staged-A; S7 is the unplanned bridge (needs the deferred S1–S3).
- **(DEFERRED) S1/S2:** the derived decode returns byte-identical rows to the
  current decoder for every page fixture (parity test).
- **(DEFERRED) S3:** every ported path passes the existing behavior tests
  unchanged.
- **S4:** `PRAGMA user_version` reaches the latest migration on a fresh file AND
  on an existing file with an older tag, by applying pending migrations — a
  reopen does NOT wipe; `sqlite_master` lists exactly the tables/indexes in
  `ddl.rs` (a statement-vs-schema drift test). **Incompatible-file detection is
  re-expressed:** today `open()` treats a `push_schema` FAILURE (its `CREATE
  TABLE` has no `IF NOT EXISTS`) as "pre-existing/foreign file → wipe &
  recreate" (`database.rs:320-370`); under migrations that signal is gone (a
  migrating `push_schema` returns `Ok`), so S4 must detect an incompatible file
  by an EXPLICIT tag/schema check (the `user_version` cursor + a schema-shape
  probe), and a foreign/unsupported file must be detected — never silently
  migrated into a broken shape. Pinned by a test that opens a foreign-tagged file
  and asserts the wipe/recreate path fires.
- **(REJECTED) S5:** T12 FK cascade was implemented, reproduced as a
  delete-race wedge (`GEO AFTER DELETE: Err(FOREIGN KEY constraint failed)`),
  and reverted. The manual ordered deletes remain the cascade owner.
- **S6 (DONE):** MVCC vs WAL A/B on turso 0.8, synthetic 8,000-endpoint feed, two
  runs per arm. **WAL stays the default**: the sequential geo rows show a
  reproducible MVCC tax (1.1–1.6× slower), and the contended fan-in row swung
  1.12 ms → 10.01 ms across two identical runs (noise at n=3), so there is no
  repeatable gain to justify a flip. The remaining path to a different answer is
  a real-feed A/B with more repetitions, not a code change.

## 9. Risks

- **0.7→0.8 on-disk format (S0a, flagged).** turso's changelog is SILENT on
  format stability across 0.7→0.8. If 0.8 cannot read a 0.7-written file,
  `try_open_db` fails → `open_with_concurrent_writes`'s recovery arm
  (`database.rs:318-330`) DELETES and recreates — a silent data wipe. S0a MUST
  gate on opening a real 0.7-written fixture and asserting it reads + no-ops
  (plan T0a.1); a real format change needs a stated disposition, not the recovery
  arm.
- **(DEFERRED) Derive scope creep.** The macro tempts re-implementing an ORM.
  Guard: the trait set is `FromRow`/`ToSql`/`Model` only; no relations, no query
  builder, no migrations-from-structs inference. (S1–S3, not in this plan.)
- **Driver-fork drift.** The fork must track `toasty-core`'s `Driver`/`Connection`
  traits — `toasty` is pre-1.0 and its driver crate already took a breaking
  change in 0.11 (`feat(turso)!`, `552e5b5e`); the `Connection` trait gained
  `push_schema`/`applied_migrations`/`apply_migration`. So a toasty bump is "take
  the bump AND re-sync our driver". Re-sync trigger: a `toasty-core` bump.
  Retirement trigger: upstream fixes the cache leak AND ships a turso-0.8 driver
  (§12.3).
- **Two-engine trap.** ELIMINATED by the fork (§12): it is the only driver, so
  `turso 0.8` cannot dual-resolve. (It would only return if we kept the
  published driver, which D5 drops.)
- **Test port cost.** ~4,235 LOC reference `toasty::*` in 2 files; mechanical but
  real. No new behavior is introduced, so tests port 1:1.
- **T14 stays blocked.** Do not carry "T14 unblocked" — it is engine-blocked;
  re-open only on a turso release that lifts WR DELETE/UPDATE.
- **(DEFERRED) simd_json.** It is a parser; its serialize side is slower than
  serde_json. Config-JSON decode STAYS `serde_json` via toasty while S1–S3 are
  deferred. Measured before adopting, if ever.

## 10. Non-goals

- **The adapter + `xray-tui-db-macro` (D1/D3/D4, S1–S3) are DEFERRED
  (2026-10-08).** The hand decoders and raw writers stay AS-IS; nothing in the
  in-scope plan (S0a/S0b, S4, S5, S6) depends on them.
- No relations / eager-loading engine (the page assembles joins in memory).
- No query builder / DSL (SQL is written by hand; the derive converts results).
- No MVCC-default flip — S6 measured it and WAL stays the default (see §12.5 and the plan's S6 notes).
- No STRICT tables (dropped for validation-only, `database-manual-sql.md` §5).
- **No full toasty removal in this plan (Option D keeps the query engine)** —
  §12's driver fork keeps the typed query layer. Duplicating or replacing toasty's
  `Load` layer is **S7**, which is `unplanned` because it requires the deferred
  S1–S3 as its enabler.
- microrm is a **concept source only**, never a dependency (sync +
  `libsqlite3-sys`).

## 11. Prior art — microrm (`thirdparty/microrm`)

`microrm` (SQLite-only, blocking or async, "no DSLs, minimal attributes") is the
closest reference for a lightweight typed layer. What to adopt, adapt, reject:

| Concept | microrm mechanism | Verdict for us |
| --- | --- | --- |
| Derive-per-field part types | `#[derive(Entity)]` generates one `EntityPart` type per field (`microrm-macros entity.rs`) | **Reject.** Heavy type-system machinery (phantom `EntityPartList`, `BuildSeal`) for a codebase that wants plain Rust. Our `FromRow` decodes by index — no per-field types. |
| Compile-time schema graph | `#[derive(Schema)]` builds a typed graph of `Table`/`Index`/`Relation` items, walked by `DatabaseItemVisitor` | **Reject.** More machinery than owning a static DDL list; our migrations are SQL text (the format we already hand-write). |
| Typed index spec (`Index<UNIQUE, E, EPL>`, `index_cols!`) | names entity **fields** only | **Reject — cannot express our indexes.** No sort direction, no partial predicate, and `band`/`rank_weight` are raw `ALTER TABLE ADD COLUMN` columns, not model fields. It covers only the 3 simple single-column `#[index]`; `endpoint_rank_key` (mixed-direction covering) and `endpoint_rank_band_window` stay **hand DDL** in `ddl.rs`. No "one typed place for indexes" claim. |
| `Value` derive for JSON fields | `#[derive(Value)]` — a field stored inline as JSON | **Adopt the idea** → our `Json<T>`/`Defer<Json<T>>` (D4), but decoded with simd_json and not tied to a derive requirement. |
| Versioned migration + compatibility check | `Schema::install` compares a stored schema signature; mismatch → `Err(IncompatibleSchema)`; `MigratableEntity<From>` / `migrate_entity` for row transforms | **Adapt the two halves.** (a) A stored version cursor = our migration cursor (§5.1). (b) The row-transform hook (`MigratableEntity`) is exactly the `Step::RustDoc(fn)` future hook for identity re-keys — cite it as precedent. **Reject** the "refuse to open on mismatch" behavior (we apply pending migrations instead). |
| `insert`/`insert_ref`/`insert_and_return`, `update_entity` | per-entity builders over a `Transaction` | **Adopt the shape** → our `Model` batch builders (`insert_batch`/`upsert_batch`/`update_batch`, D3), widened to multi-row. |
| `read`/`borrow`/`bind` on a statement row | typed column read by index with `Readable`/`Bindable` | **Adopt the idea** → our `FromValue`/`ToSql` by `turso::Row::get(i)`. |

Net: microrm confirms the shape (derive + explicit schema + versioned
migration + per-entity insert/update) but carries more type-level machinery than
this codebase needs, and its index spec cannot express our two complex indexes.
We take the shape, drop the machinery, and keep SQL hand-written. (Dep note:
microrm is sync + `libsqlite3-sys` — **concepts only, never a dependency**.)

**SQLx** is the other named reference. Adopt its **`FromRow`/`Row::try_get`
shape** (by-name, `Result`, attributes `rename`/`flatten`/`json`/`default`/
`try_from`) — that is the template for D1. **Reject the rest:** SQLx is a full
driver + pool + compile-time-query stack; adopting it re-introduces a second
engine alongside turso (the exact DualEngine trap) and its async runtime
coupling. We take the derive ergonomics, not SQLx the framework.

## 12. Option D — own driver fork (keeps toasty's engine)

**This is a DISTINCT option (call it D), not staged-A.** It keeps toasty's query
engine, so it does **NOT** shed the "unneeded features" that motivated the whole
investigation — it wins only on **turso 0.8 + DDL ownership + the cache leak**.
D and staged-A are complementary: D's turso-level half (`value.rs`, `error.rs`,
prepare/row handling) is reused if A later follows, so D is not throwaway.

**Decision (this revision):** take D as S0a/S0b — stop depending on the published
`toasty-driver-turso`; vendor a **local-only fork** as
`crates/xray-tui-db/src/driver/`, registered via
`Db::builder().models(…).build(our_driver)`.

### 12.1 Why this is viable (verified, not assumed)

- **Out-of-tree drivers ARE first-class.** `toasty::Db::builder().build(driver)`
  takes `impl Driver` (`toasty-0.11.0/src/db/builder.rs`); `toasty-driver-sqlite`
  (570 LoC, the realistic size guide) impls `Driver` exactly this way.
- **The full import surface is public and small.** A fork imports
  `toasty_core::driver::{Capability, ConnectContext, ConnectionUrl, Driver,
  ExecResponse, QueryLogConfig, log::QueryLog, operation::{…}}` — the same set
  `toasty-driver-sqlite-0.11.0/src/lib.rs:32-36` uses. `Capability::SQLITE` is a
  public associated const (`toasty-core-0.11.0/src/driver/capability.rs:660`).
  `toasty-sql::Serializer::sqlite(&schema.db)` is public → SQL is reused, not
  re-implemented. **Gate checked.**
- **Local-only shrinks 2,122 → ~570 LoC.** The published driver's
  sync/serverless/remote arms (`Turso::remote`, the serverless builder, URL
  parsing — `#[cfg]`-off for us today) are dead for an embedded file/in-memory
  db. The local exec core (`exec_sql`+`exec_sql_inner`) is **74 LoC**.
- **Query compilation does NOT read the live schema.** `exec()` serializes SQL
  from the in-memory app `Schema` (`Serializer::sqlite(&schema.db)`), never from
  `sqlite_master` — so the DDL the fork's `push_schema` emits is INDEPENDENT of
  query compilation, and the migration list (S4) and toasty's query layer cannot
  drift.

### 12.2 What the fork retires (re-derived, not asserted)

**Cache fix is an OPERATION-ROUTED `prepare`, not a blanket uncached one.** The
driver's `exec` already matches on `Operation` (`lib.rs:1262`); route each arm by
whether its SQL text is stable:

| `Operation` arm | SQL text | Prepare strategy |
| --- | --- | --- |
| `RawSql` | hand-built, literal-inlined (bulk writers, page projection) — a NEW text every call | **uncached `prepare`** — this IS the leak boundary (it is the arm that caused the unbounded map) |
| `Insert` / `QuerySql` | engine-generated, bound params, stable text — `prepare_cached` compiles once | **`prepare_cached`** — keeps the compile-once win for the typed paths S0a keeps on toasty |

No counter, no LRU, no `cacheflush` threshold: the boundary is the same one the
leak lives on. (A blanket uncached `prepare` would REGRESS the typed CRUD — their
stable text recompiles every call. `Connection::cacheflush()` exists in turso 0.8
as a fallback lever, but is not needed for this.)

Owning the driver **deletes workarounds** rather than keeping them:

| Workaround | Why it existed | After the fork |
| --- | --- | --- |
| `sql_exec.rs::SqlConn` seam (`RawConn` arm, `raw_turso_error`) | the `RawSql` path through the pooled driver grew an unbounded per-text cache; a raw `turso::Connection::query` uses UNCACHED `prepare` | **DELETED.** The fork routes `RawSql` → uncached `prepare`, so the pooled path no longer leaks — the seam's entire reason is gone. |
| `Database::write_conn` (a second raw connection) | "used ONLY by the inlined-literal writers so they run with the UNCACHED `prepare` and stop growing the driver's per-text statement cache" (`database.rs:111-113`) — a **cache** rationale, NOT a perf one | **DELETED.** The inlined-literal writers go through the pooled driver and keep their real win (literal inlining vs ~0.8 ms/bind, ADR 0002). |
| `Database::direct` (a raw connection) | the page's execution-layer bypass — a measured **~80×** read win (T13/decision 23), NOT a cache fix | **STAYS.** Different reason; unaffected by the fork. |
| Manual `BEGIN CONCURRENT` in `write_behind.rs:367` | the raw `SqlConn` path issued its own `BEGIN` | **MOVED into the fork.** The driver owns transactions (`Serializer::sqlite_with_default_begin`), so the journal-mode-aware `BEGIN` becomes the driver's `TransactionMode::Default` mapping — `write_behind.rs` calls the driver. |

**Required trait methods (no default impls) — NINE total.** Both traits carry
`#[async_trait]` (`driver.rs:125,170`), so the fork needs `async-trait`.

- `Driver` requires 5 (`driver.rs:126-152`; only `max_connections` is defaulted):
  `url` (return our path), `capability` (`&Capability::SQLITE`),
  `connect` (open the local connection — the important one),
  `generate_migration` (~8 lines: `toasty_sql::MigrationStatement::from_diff` +
  `Serializer::sqlite`, identical to upstream), and `reset_db` (local-only: drop
  the cached handle + `remove_file` — the file/remote arms upstream carries
  collapse to the file arm).
- `Connection` requires 4 (`driver.rs:249-261`; only `is_valid`/`ping` are
  defaulted): `exec`, `push_schema`, `applied_migrations`, `apply_migration`.

At S0a `push_schema` **ports toasty's `create_table` loop UNCHANGED** — it is the
ONLY thing that creates the 11 tables on a fresh file (`Database::open` calls it
at `database.rs:336,360`; `in_memory` at `:542`), and the 0.8 bump changes no DDL.
S4 swaps its body to `schema::migrate`. `applied_migrations` → `Ok(vec![])`,
`apply_migration` → `Ok(())` (our db crate does not enable toasty's `migration`
feature). The nine bodies are the ~570-LoC budget; `generate_migration` and
`reset_db` are small because our target is local-file only.

Related fix the fork enables: **turso 0.8** — the fork's manifest declares
`turso = "0.8"` directly. toasty `main`'s 0.8 bump (`c767f6c9`) touched
**`Cargo.toml` + `Cargo.lock` ONLY** — the driver source needed no change, so a
vendored copy is a dep-line edit, not a port. **Mechanism: VENDOR (own the
manifest), never `[patch.crates-io]`** — the released driver's `turso = "0.7"`
requirement excludes 0.8, which is exactly why a patch cannot work and a
git-dep/vendor is required. Also mandatory: **drop `turso` from our `toasty`
feature list** (`crates/xray-tui-db/Cargo.toml:9`) or upstream's driver returns
via the feature flag.

### 12.3 Cost to state honestly

- **A recurring version-TRACKING obligation, not a one-line pin.** `toasty` is
  pre-1.0 (0.11); its driver crate already took a breaking change in 0.11
  (`feat(turso)!` serverless/Turso-Cloud, `552e5b5e`), and `toasty-core`'s
  `Driver`/`Connection` traits are the seam the engine calls — a toasty bump can
  break our driver before we can take it. The 0.11 `Connection` trait already
  gained `push_schema`/`applied_migrations`/`apply_migration`. So this converts
  "take a toasty bump" into "take a toasty bump AND re-sync our driver".
- **Cost split.** The toasty-facing half (trait impls, `Operation`→SQL via the
  serializer, `ExecResponse`) is **throwaway if staged-A follows**; only the
  turso-level code (value mapping, error classification, prepare/query/row) is
  reused. So D is a real win **only because we intend to keep toasty** — which
  is why D is its own option and not a step of A.
- **Retirement trigger:** if upstream fixes the cache leak AND ships a turso-0.8
  driver, the fork can be retired back to the published crate. Upstream `main`
  still calls `prepare_cached` (verified against the fresh clone
  `thirdparty/toasty` @ `4f3ed1a` AND the published crate — both `lib.rs:1254`),
  so that is not imminent.
- **Clone reference / vendor source.** `thirdparty/toasty` (fresh `main`,
  `4f3ed1a`) is the VENDOR SOURCE: its `toasty-driver-turso` is already on
  `turso = "0.8"`, so vendoring its local arm removes the "will the 0.11 driver
  compile against 0.8" risk. It differs from the published 0.11.0 crate **only in
  the remote/serverless arm** (`TursoPath::Remote` gains `default_begin_sql`; a
  new `with_transaction_mode`; libsql URL scheme defaults) + one import — the
  local `exec`/`push_schema`/`prepare_cached:1254` arm is unchanged. **Vendor a
  COPY — never path-depend into `thirdparty/`** (an untracked reference tree).
  Keep `toasty`/`toasty-core`/`toasty-sql` at published `0.11.0` and verify the
  dev driver compiles against them (it imports only
  `TransactionMode`/`IsolationLevel`/`Operation`/`RawSqlRet`/`Transaction`/
  `TypedValue` + `toasty_sql`, all in published 0.11.0); if a dev-only core API
  appears, fall back to the published 0.11 driver source + the manifest-only 0.8
  pin (toasty main's `c767f6c9` proved that compiles).

### 12.4 Effect on the plan

The adapter/macro/migrations design (§3–§5) is unchanged; the fork is the
executor underneath and *simplifies* the ordering. **Contract (one owner, no
indirection):**
`Db::build()` does NOT call `push_schema` (verified — `builder.rs` has no such
call; only OUR `Database::open` calls it, at `database.rs:336,360`, plus
`in_memory` at `:542`) — and that call is **the only thing that creates the 11
tables on a fresh file**. So the fork does NOT get a no-op `push_schema` at S0a:
it **ports toasty's `create_table` loop unchanged** (0.8 changes no DDL), and
**S4 swaps that body to `schema::migrate`**, after which `Database::open`'s
`push_schema` call delegates to the migration runner and no DDL travels through
the toasty trait. **"One list, one place" (§5) is therefore true from S4, not
S0a.** D5, S0a, S4, and §5 all state this same contract.

- The former "bump turso LAST" constraint (problem 5) is GONE — the fork is the
  only driver, so 0.8 lands at S0a.
- **No `db_monitor` retarget is needed.** The `toasty::query` event is emitted by
  `toasty_core::driver::log::QueryLog` (`pub const TARGET = "toasty::query"`,
  `log.rs:21`), NOT by the driver crate — the fork keeps using `QueryLog`, so
  `db_monitor.rs` and `main.rs`'s `LogVisitor` keep working untouched at
  S0a/S0b–S6. Only a full toasty-core removal (S7) would lose it. **(Already removed
  from S5 and §7 — this note is the record.)**

### 12.5 WriteBehind vs "write directly" — do NOT simplify before S6 measures

Raised question: if turso 0.8's MVCC removes write contention, could the
`WriteBehind<Spec>` batching layer be dropped and writes go straight to the DB?

**Not before S6, and probably not after either.** Three independent reasons, all
measured in this repo:

1. **MVCC is not the default, and was measured slower.** WAL is the default
   (`specs/2026-09-24-turso-mvcc-rollout-design.md` §12: "WAL stays the
   default"); the real-shaped-feed A/B had MVCC **1.2–4.8× slower** on every
   geo row. Under the shipped default (WAL), removing `WriteBehind` reverts the
   commit count from **32 → 1,167 per batch** — the metric that made the layer
   worth having.
2. **MVCC removes lock WAIT, never the work.** The retired spec's own table: the
   contention motivating MVCC was already addressed at the source (multi-row
   writers, one geo transaction per drain). What remains is per-statement
   round-trip cost (`upsert_protocols_bulk` at ~117 µs/row) — no journal mode
   touches that, and `WriteBehind` is what batches it.
3. **`stage()` is non-blocking on the UI task.** The batch pipeline stages
   results without awaiting a commit; a direct write would put every ping
   result's commit back on the UI task — the exact freeze the layer exists to
   prevent.

**So this is NOT part of Option D.** `WriteBehind` survives S0a/S0b–S6 unchanged
in its **batching/staging contract** — its **raw-connection half** IS deleted at
S0b (§12.2/T0b.2: `CacheSpec::RAW`, `write_window_raw`/`refresh_raw`, the
`:364` `write_conn()` branch). It
becomes a REAL question only if **S6 flipped the MVCC default** — it did NOT
(measured 2026-10-08: WAL stays default) — at which point the *flush cadence* could be revisited,
because MVCC would make each commit cheaper and one-per-statement less costly.
Even then the layer is likely to stay: reason 3 (UI non-blocking) is independent
of the journal mode, and the batch pipeline still wants ONE transaction per
window for the link-write + rank-refresh atomicity (ADR 0002/0003). Record it as
an S6-adjacent follow-up, gated on the S6 number — never a pre-S6 simplification.

**What the fork legitimately deletes is WriteBehind's RAW-connection half, NOT
its batching.** `CacheSpec` carries a `RAW` const plus `write_window_raw` /
`refresh_raw` (`write_behind.rs:136,141,150`; `LinkSpec: RAW = true` at `:916`),
and the flush task branches on `db.write_conn()` (`:364`) to run the window on
the uncached raw connection. That half exists SOLELY as the statement-cache
workaround — exactly what S0b removes at the driver — so `CacheSpec::RAW`,
`write_window_raw`, `refresh_raw`, the `:364` branch, and `Database::write_conn`
are deleted together, and `LinkSpec` flushes through the POOLED driver. The
batching + off-task staging (the 32-vs-1,167-commit win, the non-blocking
`stage()`) is untouched by the journal mode and by the fork.

