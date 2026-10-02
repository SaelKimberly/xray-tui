# Plan — Static config weight as an ordering prior

Date: 2026-10-01
Status: EXECUTED 2026-10-01 — all ten tasks landed; deviations recorded under "Execution notes" at the end.
Spec: `docs/aegis/specs/2026-10-01-static-config-weight-design.md` (draft, approved for planning)
Aegis route: `writing-plans`. Execution route: **inline** (tasks share one owner file set and one law; parallel ownership would mean two writers on `endpoint_rank.rs`).

## Goal

A compiled, versioned **weight** for every link's transport/security config,
materialized as `endpoint_rank.rank_weight`, inserted into the decision-16
ordering law inside `tier`, and used to order both the Profiles page and the
batch feed walk.

## Architecture

One law, two representations that must agree:

- **Rust (authoritative):** `RankLink::key` returns an ALL-ASCENDING tuple
  `(tier, u64::MAX - weight, latency, -seen_secs, protocol_id)`, consumed by
  `.min()` in `compute_rank`. Higher weight = better, negated to fit the
  ascending idiom, exactly as `neg_seen` already does.
- **SQL (mirror):** stored pack is the 8-byte big-endian weight itself;
  `ORDER BY rank_weight DESC` + index `endpoint_rank_test_v2(… rank_weight DESC …)`.
  SQLite orders BLOBs by memcmp, so DESC-in-SQL == negated-ascending-in-Rust by
  construction, over the full 64-bit range.

Owners, unchanged: the law stays in `endpoint_rank.rs`, the page stays in
`profiles_query.rs`, the walk stays in `ops/ping.rs`. **No new owner file.**

## Tech Stack

Rust 2024, `toasty` 0.11 + `turso` driver, `cargo-nextest`, `just quality-gate`.

## Baseline / Authority Refs

- `docs/aegis/specs/2026-10-01-static-config-weight-design.md` — the design.
- `docs/aegis/adr/0003-stored-profile-ordering-keys.md` (materialized keys),
  `0008-batch-feed-scaling.md` (id-ordered walk — superseded by T7).
- AGENTS.md decisions 15, 16, 21; project decision 4 (tag bump = wipe).
- `docs/database.md` (table/index/write-path authority),
  `docs/database-manual-sql.md` (raw-SQL site authority).

## Compatibility Boundary

- **No schema tag bump.** `rank_weight` is a RAW column via the existing
  `ALTER TABLE ADD COLUMN` precedent (`endpoint_rank.rs:315-325`). Declaring it
  on the toasty model would cost tag 15 → `open()` deletes the db file.
- **Upgraded databases keep their rows.** The `NOT NULL DEFAULT` all-zero pack is
  valid input ("worst"); the `WEIGHT_VERSION` mismatch is what replaces it.
- **Measurement is untouched** — every latency/error/purge column keeps its
  meaning. Only *ordering* changes.
- **The batch's frozen-ids requirement is new for `PlanScope::All`** and is
  behaviour-visible: an `All` run now reads its id set before the first probe.

## Change Necessity

No config or docs-only path exists: the weight must be computed per link, stored
where SQL can `ORDER BY` it, and inserted into a law that currently has no such
term. Minimum code boundary = one new struct + one new raw column + one new index
+ one new law term + the four construction sites + the walk sort.

## TDD Route

`auto` → **strict**. Authority: persistence change + shared ordering contract +
producer/consumer (DB writes the key, SQL and Rust both read it). Test posture:
each task starts with a failing test that pins the behaviour named in its
Acceptance, then the minimum change, then the full-suite run at the end.

Precondition on T1: the **cell values must be authored first** (spec open item).
Authoring them is where the sub-config gate is checked — if any area needs ws
`path` / xhttp `mode` / kcp `header_type`, stop and reopen spec decision 4.

## Tasks

### T1 — `xray-tui-proto`: the weight type and its tables

**Files:** `crates/xray-tui-proto/src/proto_spec/weight.rs` (new module, registered
in `proto_spec/mod.rs`), its `mod tests`.

**Change:** `ConfigWeight { security, mimicry, sec_cost, transport_cost }` —
`#[repr(C)]`, four `u16`, declaration order = precedence, derived `Ord`,
**higher = better**. `to_be_bytes() -> [u8; 8]` / `from_be_bytes`, the SQL blob
literal form, and `WEIGHT_VERSION: u32`. `weight_of(&ProtocolDiscriminators) ->
ConfigWeight` reads four per-dimension tables. Each lookup is an **exhaustive
`match`** (no `_` arm) so a new `TransportConfig` / `Security` variant is a
compile error, not a silent zero.

**RED first:** pack round-trip incl. all-zero; memcmp order == `Ord` over swept
values; every discriminators combination yields a weight.

**Acceptance:** exhaustive compiles for all current variants; `Ord` and be-byte
order agree on the sweep.

### T2 — `endpoint_rank`: the law term

**Files:** `crates/xray-tui-db/src/endpoint_rank.rs`, `crates/xray-tui-db/src/models_toasty.rs` (`EndpointRank` unchanged — the weight is **not** a model field).

**Change:** `RankLink` gains `weight: ConfigWeight` (the packed `u64` form is the derived `weight_u64`, used only inside `key()` and the blob write); `From<&ProfileStats>` is replaced by
`RankLink::new(link, weight)` (it cannot see the config). `key()` returns
`(u8, u64, i32, i64, i64)` with `u64::MAX - weight` as term 2. `compute_rank`
returns the representative link's weight; introduce `RankRow` (mirror +
`weight`) as `write()`'s parameter type; add `rank_weight` to `RANK_COLUMNS` and
the values tuple — **and not** to the follow-up `UPDATE`, which exists only for
the SQL-derived `band`/`rank_host`.

**RED first:** inside tier 0, a 900 ms high-weight link sorts above a 40 ms
low-weight one; a failed link still never outranks an untested one; the
existing decision-16 golden updates only where weights differ.

**Acceptance:** both properties hold; `write()` round-trips `rank_weight`.

### T3 — schema: column, index, version stamp

**Files:** `crates/xray-tui-db/src/endpoint_rank.rs` (`ensure_in`, DDL consts).

**Change:** add to the swallowed-error `ALTER` loop —
`ADD COLUMN rank_weight BLOB NOT NULL DEFAULT x'0000000000000000'`. New index
name `endpoint_rank_test_v2` with `rank_weight DESC` after `rank_tier`; leave
`endpoint_rank_test` in place so a rollback keeps working (both created, both
`IF NOT EXISTS`). One-row meta table holding `WEIGHT_VERSION`, checked **inside
the early-return branch** (`:334-347`); a missing table reads as mismatch;
mismatch → `backfill_all`.

**RED first:** on a DB created *before* T3, `rank_weight` is non-NULL for every
row, the version mismatch fires, and keys are recomputed — no `IS NULL` fill,
which would find nothing (the `ADD COLUMN` materializes the default).

**Acceptance:** upgraded DB ends with real weights, not the zero pack.

### T4 — the two DB computation sites

**Files:** `endpoint_rank.rs` (`refresh`, `backfill_all`).

**Change:** `refresh` adds `LEFT JOIN protocols pr ON pr.id = ps.protocol_id` to
the statement it already issues, selecting the five discriminator columns.
`backfill_all` gains a third typed load `Protocol::all()` plus a
`HashMap<ProtocolId, ConfigWeight>` (it loads typed models, not a raw
projection).

**Acceptance:** both produce identical weights to `rank_of_row` for the same
links.

### T5 — the hot in-memory path

**Files:** `crates/xray-tui-db/src/models_toasty.rs:575-633`.

**Change:** `link_test_key`, `sort_links_by_test_priority`, `best_test_priority_key`
take the weight from `self.protocols` (looked up per link by `protocol_id`);
default `0` when a link's protocol row is absent. `rank_of_row` is the same site,
already in hand.

**RED first:** a page whose links differ only by weight re-sorts by weight on a
ping result; a missing protocol row sorts as worst rather than panicking.

### T6 — the page query

**Files:** `crates/xray-tui-db/src/profiles_query.rs`.

**Change:** `order_terms(PageSort::Test)` gains `term(rank_col("rank_weight"),
false)` — **DESCENDING**, between `rank_tier` and `rank_latency`.
`PAGE_PROJECTION` needs no new column (the discriminators are already there).
Verify `Vec<u8>: Into<Value>` for the anchor bind at `:543,546`; if it does not,
convert through the engine's blob value type.

**RED first:** direction sweep — SQL `DESC` and the negated Rust key agree over
swept values; a DESC→ASC flip fails here, not in the field.

### T7 — the walk

**Files:** `crates/xray-tui/src/ops/ping.rs` (`PlanWalk::new` / `next_page`).

**Change:** `PlanSource::Feed(PlanScope::All)` orders by `PageSort::Test` instead
of `PageSort::Id` and takes the **frozen-ids** path (`ping.rs:1355-1359`) —
its order is now result-dependent, so offset paging over the live predicate
would skip every endpoint that leaves the scope mid-run.

**RED first:** extend `a_scoped_walk_freezes_its_set_before_probing` to `All` —
under a running batch, every endpoint in the frozen set is visited exactly once.

**Acceptance:** no skipped endpoints; the frozen read is measured (T10).

### T8 — regression sweep

Extend the parity golden (all 8 sorts × both directions, over a fixture whose
links differ in weight); add the anchor-on-default test, the
version-rebuild test, and the upgraded-DB index-shape test (a fresh-DB test
proves nothing — an in-place `CREATE INDEX IF NOT EXISTS` edit is a silent
no-op).

### T9 — authority docs (same change)

AGENTS.md decisions 15 + 16; ADR 0008 (superseding decision — title, decision 1
and the `:20` evidence row); `docs/database.md` (entity block `:140-146`, index
row `:266`, write-path list `:282-288`, query map `:436`);
`docs/database-manual-sql.md` (raw column, new index name, meta table).

### T10 — measure

Page fetch with `PageSort::Test` vs the 8.6 ms baseline; the frozen-ids read vs
the 3.6 ms/page streaming walk; confirm the page is an index scan and not a
filesort.

## Verification

- `cargo nextest run --workspace` — full suite green.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `just quality-gate code`.
- Targeted: the new tests named in T1-T8 by name.
- Manual: launch against a copy of a pre-change database and confirm the page
  order changed and the batch probes the reliable links first.

## Risks

- **Unmeasured walk cost** — the frozen-ids pass replaces a streaming walk
  (T10 measures it; if it regresses plan latency badly, revisit).
- **Authoring the cell values can reopen the storage decision** (T1 gate).
- **The prior ages silently** — mitigated only by `WEIGHT_VERSION`, and by never
  letting weight outrank `tier`.
- **Index shape** is invisible to a fresh-DB test; the upgraded-DB test is the
  only guard.

## Retirement

`endpoint_rank_test` (old index) is retained deliberately for rollback and
retired once the new one has proven itself on a real upgraded database — one
release, then dropped. `From<&ProfileStats>` is **deleted**, not kept as a
fallback: a weight-less `RankLink` is exactly the silent-zero bug class this
feature must not have.
## Execution notes (2026-10-01)

Deviations from the plan as written, and why:

1. **`weight_of` takes four positional discriminators**, not a
   `ProtocolDiscriminators` struct — the DB path has them as four nullable
   columns and the typed path as four fields; a struct would have bought a
   conversion on both sides and nothing else.
2. **`RankLink.weight` is a `ConfigWeight`**, not a `u64`. The packed `u64` is
   derived (`weight_u64`) exactly where the law and the blob write need it.
3. **`refresh` joins four columns, not five** — `proto_kind` is deliberately not
   a weight dimension.
4. **A defect found in review, fixed after the fact**: `refresh` parsed the raw
   `transport_type` column with `TransportType::from_str`, which accepts the
   WIRE spellings (`httpupgrade`, `xhttp`) while toasty's embed WRITES the
   snake_case idents (`http_upgrade`, `x_http`). Verified against a real feed
   (145 + 169 rows), every such link persisted a ZERO weight while every typed
   path computed a real one, splitting the stored page order from the panel.
   Fixed with `TransportType::from_db_label` + a test pinning the label set.
5. **The upgraded-DB index-shape test does not exist as planned.** What ships is
   `the_test_order_has_an_index_that_includes_the_weight`, which asserts the
   index's own DDL contains `rank_weight DESC` — enough to catch an in-place
   column-list edit (the actual failure mode), because the old index keeps
   existing on upgraded databases and the new one is created under a NEW name.
6. **Measurements** landed in the spec's "Measured" section: the Test page is
   4.43 ms against a covering-index scan, and the frozen feed read (17.4 ms /
   4,000 ids) is CHEAPER than the streaming walk it replaced (~90 ms at that
   size), so the cost recorded as a risk did not materialize.
