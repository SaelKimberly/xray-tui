# Write-Behind Link Writer — Implementation Plan

Spec: `docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md`
(approved 2026-09-11; flush cadence 200 ms / 512 rows)

## Goal

Remove per-result and per-transition database commits from the UI task so a
30k-link Fast+Real batch stays interactive, while keeping the scheduler gate's
read-your-writes semantics.

## Architecture

- `ops/link_writer.rs` owns the pending map, staging, read-through, and the
  flush task. The pending map is authoritative for the gate; the database is
  the durable mirror.
- `Database::apply_link_patches` writes one narrow `UPDATE` per row inside a
  single transaction.
- Result handling and scheduler transitions `stage()`; batch end and shutdown
  `flush()`.

## Tech Stack

Rust 2024, toasty 0.10 (`toasty::update!`, `Connection::transaction`), Turso,
tokio, DashMap.

## Baseline / Authority Refs

- Spec above; `AGENTS.md` decision 15 (in-memory sync rule), decision 16
  (ordering); `ops/scheduler.rs` module docs (gate + orphan sweep);
  `crates/xray-tui-db/src/retry.rs` (busy retry).
- Measurements: spec §1 (typed upsert 10.298 ms FULL / 1.056 ms NORMAL; narrow
  UPDATE in one tx 0.120 ms).

## Compatibility Boundary

No schema change, no `SCHEMA_VERSION` bump. Existing databases open unchanged.
Durability window ≤ `flush_interval` (200 ms) of staged rows, matching the
`synchronous=NORMAL` boundary already accepted.

## TDD Route

```
TDD Route:
- Mode: off
- Decision: skipped
- Strict authority: not applicable
- Strict signals: persistence + shared gate behaviour exist, but no explicit
  user/project strict request and no recorded auto mode
- Test posture: post-change regression + a gate-correctness regression test
  (spec §7.1), which is the one behaviour a bug would silently break
- Reason: mode `off`; the plan prescribes proportional regression tests
- Verification: cargo nextest -p xray-tui-db, cargo nextest -p xray-tui,
  cargo clippy --workspace --all-targets, just quality-gate code
```

## Files

| File | Action |
| --- | --- |
| `crates/xray-tui-db/src/database.rs` | add `apply_link_patches` |
| `crates/xray-tui-db/tests/integration.rs` | batching + final-state tests |
| `crates/xray-tui/src/ops/link_writer.rs` | create — the writer owner |
| `crates/xray-tui/src/ops/mod.rs` | register the module |
| `crates/xray-tui/src/state.rs` | hold `Arc<LinkWriter>` |
| `crates/xray-tui/src/ops/events.rs` | result handler stages instead of awaiting |
| `crates/xray-tui/src/ops/scheduler.rs` | `SchedulerDb` reads through / stages |
| `crates/xray-tui/src/ops/ping.rs` | flush at batch end |
| `crates/xray-tui/src/main.rs`, `ui/mod.rs` | construct the writer; flush on quit |

## W1 — `Database::apply_link_patches` ✓ (commit `0574310`, revised)

**Why**: the batch primitive; the UI path never calls it directly (the writer
does).

**Final shape** (the module on disk is authoritative):

```rust
pub struct LinkGroups(u8);   // RESULT | TASK | TRAFFIC | ALL, with contains()
pub struct LinkPatch { pub link: ProfileStats, pub groups: LinkGroups }

/// One row per iteration inside ONE transaction: read the row fresh, overlay
/// only the patched groups from the caller's snapshot, write it back.
pub async fn apply_link_patches(&self, patches: &[LinkPatch]) -> Result<usize>;
```

The fresh read inside the transaction is what makes the unpatched groups safe:
a caller's snapshot is authoritative for its own group and stale for the rest,
so a result patch can never write traffic or scheduler state.

**Tests** (`tests/integration.rs`, all green): group-scoped writes for every
row; empty input is a no-op; stale snapshot lands without clobbering a
concurrent `update_scheduler_state`; and `..._isolates_column_groups` — a
TRAFFIC patch leaves result/task columns exactly as persisted, and a RESULT
patch with a deliberately stale snapshot leaves traffic/task intact.

## W2 — `ops/link_writer.rs`

**Why**: the new owner: staging, read-through, flushing.

**Steps**

1. Create the module:

```rust
//! Write-behind persistence for `profile_stats` mutations.
//!
//! The pending map is authoritative for the scheduler gate; the database is
//! the durable mirror (see docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md).

pub struct LinkWriter {
    /// One staged snapshot per (link, column group): the result group and the
    /// task group are staged independently and never overwrite each other.
    pending: DashMap<((ProtocolId, EndpointId), LinkGroups), ProfileStats>,
    flush_rows: usize,
    flush_interval: Duration,
    db: Arc<Database>,
    gate: tokio::sync::Mutex<()>,     // one drain-and-write at a time
    wake: tokio::sync::Notify,
    flushes: AtomicU64,               // diagnostics/tests
}

impl LinkWriter {
    pub fn new(db: Arc<Database>, flush_rows: usize, flush_interval: Duration) -> Arc<Self>;
    pub fn spawn_flush_task(self: &Arc<Self>) -> tokio::task::JoinHandle<()>;
    pub fn stage(&self, link: &ProfileStats, groups: LinkGroups);     // sync, never awaits
    pub async fn read(&self, key: (ProtocolId, EndpointId)) -> Result<Option<ProfileStats>>;
    pub async fn flush(&self) -> Result<usize>;                      // drain + one transaction
    pub fn staged_len(&self) -> usize;
    pub fn flush_count(&self) -> u64;
}
```

2. `stage` inserts/overwrites `pending[(key, groups)]` with the row snapshot
   and notifies the flush task when `pending.len() >= flush_rows`. Staging both
   groups for one link keeps the result and task snapshots separate, so the
   flush can patch each group with its own OCC version.
3. `read` returns `pending[key]` when present (read-through), else the typed
   PK read.
4. `flush` takes the mutex and **removes** the pending entries into a
   `Vec<LinkPatch>` (one per staged group; a remove, never a snapshot copy —
   a `stage` landing during the write must survive as a new entry for the next
   window), calls `db.apply_link_patches(&patches)`, and returns the count. On
   error the drained rows merge back with `or_insert` semantics so a newer
   staged entry wins over the failed snapshot. On error, the drained
   rows are merged back so a failed flush retries.
5. The flush task loops on `tokio::select! { _ = wake.notified() => {}, _ =
   sleep(flush_interval) => {} }` while `staged_len() > 0`.
6. Unit tests (TUI crate, in-memory DB):
   - `stage_then_read_is_read_through` — no flush, read returns the staged row;
   - `flush_writes_every_staged_row` — after `flush`, a direct DB read matches;
   - `stage_never_awaits` — 1000 stages complete without any flush call
     (`flush_count() == 0`);
   - `flush_is_one_transaction` — `flush_count()` increments once for 1000
     staged rows;
   - `stage_during_in_flight_flush_survives` — with the flush blocked mid-write
     (a test writer), stage a newer value for the same `(link, group)`; after
     release the newer value is what lands (spec §4.3 drain rule).

## W3 — Wire the call sites

**Steps**

1. `ops/events.rs` result handler: replace `state.db.upsert_link(link).await`
   with `state.link_writer.stage(link)` for both the error and the success
   branches. The handler stops awaiting any database write.
2. `ops/scheduler.rs`: implement `SchedulerDb` for `Arc<LinkWriter>`:
   `read_link` → `LinkWriter::read`; `write_task_state` → build the row's new
   state and `stage` it (no await). Keep the existing `Database` impl for
   non-batch callers.
3. **Every other `profile_stats` writer moves to its owned group** (spec §4.1
   table):
   - `CoreEvent::StatsUpdate` (`ops/events.rs:269`) → `stage(link, TRAFFIC)`
     instead of `db.upsert_link(link).await`;
   - `drain_pending_stats_updates` flush (`ops/connect.rs:645`) → stage +
     flush through the writer;
   - `Database::update_last_used` (`ops/connect.rs:193`) stays a narrow typed
     write, but must stop passing through `upsert_link`;
   - `upsert_links_bulk` (subscription/import) is restricted to the `SOURCE`
     group (`core_type`, `config_type`, `last_seen_at` on update; full row on
     insert) so an import refresh can no longer overwrite ping results.
4. `state.rs`: add `link_writer: Arc<LinkWriter>`; construct it in
   `AppState::new` and in the test helpers.
4. Update `ops/scheduler.rs` unit tests to use a writer-backed fake (the
   existing `FakeDb` gains the read-through behaviour, or the tests move to
   `Arc<LinkWriter>` over an in-memory DB).

## W4 — Batch lifecycle

**Steps**

1. `ops/ping.rs`: at the end of a batch (both phases settled), call
   `state.link_writer.flush().await` and then one
   `PRAGMA wal_checkpoint(TRUNCATE)` on a pooled connection.
2. `ui/mod.rs` quit path: `state.link_writer.flush().await` before shutdown.
3. `main.rs`: spawn the flush task after the writer is constructed.

## W5 — Verification

1. **Gate read-through** (spec §7.1): staged `schedule` transition → second
   `schedule` for the same link must not start it again; `complete` with the
   staged id succeeds; mismatched id is a stale no-op.
2. **No UI-task commit**: a poll pass draining `EVENT_DRAIN_BUDGET` results
   leaves `flush_count()` unchanged.
3. **Final-state equality**: after batch-end flush, every staged row equals the
   persisted row across all mutable columns.
4. **Throughput**: a 30k-link synthetic batch in a `#[ignore]`d test that
   reports wall time, transaction count, and per-row cost; the recorded
   baseline is spec §1.
5. **TUI smoke**: real batch on the large subscription — keys respond, render
   keeps up, results re-sort.

## Risks and Rollback

| Risk | Mitigation |
| --- | --- |
| Gate reads stale state | Read-through + §W5.1 regression |
| Two writers on one row | Scheduler's batch write path becomes `stage`; grep for `update_scheduler_state` in the batch path |
| Failed flush loses rows | Drained rows merge back on error and retry on the next tick |
| Memory during a 100k batch | Pending is keyed by link and drained each window |
| Rollback | Revert the merge commits; no schema or data migration exists |

## Retirement Track

| Old path | Status | Trigger |
| --- | --- | --- |
| `upsert_link` in the result handler | deleted in W3 | writer lands |
| `update_scheduler_state` on the batch path | deleted in W3 | read-through lands |
| `update_scheduler_state` (non-batch callers) | keep; delete when none remain | last caller migrates |
