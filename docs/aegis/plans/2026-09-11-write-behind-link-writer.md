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

## W1 — `Database::apply_link_patches`

**Why**: the batch primitive; the UI path must not call it directly (the writer
does).

**Steps**

1. Add to `crates/xray-tui-db/src/database.rs`:

```rust
/// Apply one narrow `UPDATE` per row inside a single transaction.
///
/// Covers only the mutable columns (`latency`, `speed_bps`, `error`,
/// `task_id`, `task_queue`); every other column is owned by its own writer.
/// The OCC `version` is honoured when the caller's snapshot is current; a
/// stale snapshot costs one re-read for that row instead of failing the batch.
pub async fn apply_link_patches(&self, patches: &[ProfileStats]) -> Result<usize> {
    if patches.is_empty() {
        return Ok(0);
    }
    let mut conn = self.conn().await?;
    let mut tx = conn.transaction().await?;
    let mut applied = 0usize;
    for patch in patches {
        let mut model = patch.clone();
        loop {
            match toasty::update!(model.clone() {
                latency: patch.latency.clone(),
                speed_bps: patch.speed_bps,
                error: patch.error.clone(),
                task_id: patch.task_id,
                task_queue: patch.task_queue.clone(),
            })
            .exec(&mut tx)
            .await
            {
                Ok(()) => {
                    applied += 1;
                    break;
                }
                Err(err) if err.is_condition_failed() => {
                    // Stale snapshot: refresh and retry this row once.
                    let Some(fresh) = ProfileStats::filter_by_protocol_id_and_endpoint_id(
                        patch.protocol_id,
                        patch.endpoint_id,
                    )
                    .first()
                    .exec(&mut tx)
                    .await?
                    else {
                        break; // row deleted mid-batch
                    };
                    model = fresh;
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
    tx.commit().await?;
    Ok(applied)
}
```

2. Test in `tests/integration.rs`:

```rust
#[tokio::test]
async fn apply_link_patches_writes_all_mutable_columns_in_one_transaction() {
    // seed 3 links, patch latency/error/task state, assert every column landed
    // and that a stale-version patch still applies (re-read path).
}

#[tokio::test]
async fn apply_link_patches_empty_is_a_noop() { /* returns Ok(0) */ }
```

**Verification**: `cargo nextest run -p xray-tui-db`.

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
    pending: DashMap<(ProtocolId, EndpointId), ProfileStats>,
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
    pub fn stage(&self, link: &ProfileStats);                        // sync, never awaits
    pub async fn read(&self, key: (ProtocolId, EndpointId)) -> Result<Option<ProfileStats>>;
    pub async fn flush(&self) -> Result<usize>;                      // drain + one transaction
    pub fn staged_len(&self) -> usize;
    pub fn flush_count(&self) -> u64;
}
```

2. `stage` inserts/overwrites `pending[key]` with the row's mutable fields and
   notifies the flush task when `pending.len() >= flush_rows`.
3. `read` returns `pending[key]` when present (read-through), else the typed
   PK read.
4. `flush` takes the mutex, drains `pending` into a `Vec`, calls
   `db.apply_link_patches(&rows)`, and returns the count. On error, the drained
   rows are merged back so a failed flush retries.
5. The flush task loops on `tokio::select! { _ = wake.notified() => {}, _ =
   sleep(flush_interval) => {} }` while `staged_len() > 0`.
6. Unit tests (TUI crate, in-memory DB):
   - `stage_then_read_is_read_through` — no flush, read returns the staged row;
   - `flush_writes_every_staged_row` — after `flush`, a direct DB read matches;
   - `stage_never_awaits` — 1000 stages complete without any flush call
     (`flush_count() == 0`);
   - `flush_is_one_transaction` — `flush_count()` increments once for 1000
     staged rows.

## W3 — Wire the call sites

**Steps**

1. `ops/events.rs` result handler: replace `state.db.upsert_link(link).await`
   with `state.link_writer.stage(link)` for both the error and the success
   branches. The handler stops awaiting any database write.
2. `ops/scheduler.rs`: implement `SchedulerDb` for `Arc<LinkWriter>`:
   `read_link` → `LinkWriter::read`; `write_task_state` → build the row's new
   state and `stage` it (no await). Keep the existing `Database` impl for
   non-batch callers.
3. `state.rs`: add `link_writer: Arc<LinkWriter>`; construct it in
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
