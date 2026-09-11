# Write-Behind Link Writer — Design Spec

Date: `2026-09-11`
Status: `proposed` (implementation requires approval of this document)
Owner surface: `crates/xray-tui/src/ops` (new writer owner), `crates/xray-tui-db`
(narrow batch update), `crates/xray-tui/src/ops/{scheduler,events,ping}.rs`

Continues the 2026-09-11 batch-ping investigation (`synchronous=NORMAL` landed
in `36bed3a`).

## 1. Context and measured problem

A Fast+Real batch over 30k+ links makes the TUI non-interactive and saturates
the NVMe while successful pings re-sort to the top.

Measured on the real Turso engine, file-backed database, this machine
(2026-09-11):

| Path | `synchronous=FULL` | `synchronous=NORMAL` (landed) |
| --- | --- | --- |
| raw single-row commit | 4.217 ms | 0.219 ms |
| typed row upsert (`upsert_link` shape) | 10.298 ms | 1.056 ms |
| narrow `UPDATE` in one transaction | — | 0.120 ms |
| `upsert_links_bulk`, 1000 rows, one tx | — | 0.883 ms/row |

The batch pipeline issues **three commits per link** — the result
(`Database::upsert_link`) plus `TaskScheduler::schedule` and `complete`, each
doing `read_link` + `update_scheduler_state` — and all of them are awaited
**inside `poll_core_events` on the UI task**, whose per-tick drain budget is
`EVENT_DRAIN_BUDGET = 256`.

At 30k links that is ~90k commits: ~15 minutes of serialized DB work before the
`synchronous` fix, ~95 seconds after it. Both are unacceptable, and the
remaining cost is per-statement overhead, not fsync.

## 2. Goal and acceptance criteria

Make batch persistence proportional to data changed, not to the number of
events, and keep every database write off the UI task.

1. During a 30k-link batch the UI task never blocks on a database write: no
   single poll/render tick exceeds 50 ms (`EVENT_DRAIN_BUDGET` events never
   await a commit).
2. A 30k-link batch performs on the order of `links / flush_rows` transactions
   instead of ~3 per link, with one narrow `UPDATE` per changed row.
3. Measured end-to-end: a 30k-link batch's persistence cost is ≤ 15 % of the
   post-`synchronous` baseline (§1), verified by a benchmark harness.
4. **Gate correctness is preserved** (§4.2): with a staged, unflushed task
   transition, `schedule` never double-starts a link and `complete` never
   treats a matching completion as stale.
5. After the final flush, the persisted state equals the in-memory state for
   every changed row.

## 3. Non-goals

- No schema change, no `SCHEMA_VERSION` bump, no wipe.
- No change to protocol identity, connect path, routing, or native core.
- No change to *which* rows are written, only when and how.
- UI re-sort throttling and windowed rendering stay in the profiles page-query
  plan (`docs/aegis/plans/2026-09-11-profiles-page-query.md`, tasks T3/T4);
  this spec does not duplicate them, but it must not block them.

## 4. Design

### 4.1 Owner

New owner: `crates/xray-tui/src/ops/link_writer.rs` — `LinkWriter`.

```rust
pub struct LinkWriter {
    /// Latest unflushed state per link, keyed by (protocol_id, endpoint_id).
    /// Also the read-through source for the scheduler gate (see 4.2).
    pending: DashMap<(ProtocolId, EndpointId), ProfileStats>,
    /// Signals the flush task; `Notify` + a background loop with a deadline.
    wake: tokio::sync::Notify,
    db: Arc<Database>,
    /// Flush policy.
    flush_rows: usize,          // default 512
    flush_interval: Duration,   // default 200 ms
}

impl LinkWriter {
    /// Stage the row's current in-memory state. Never waits on the database.
    pub fn stage(&self, link: &ProfileStats);

    /// Read-through: pending state if present, else the database.
    pub async fn read(&self, key: (ProtocolId, EndpointId)) -> Result<Option<ProfileStats>>;

    /// Persist everything staged so far, in one transaction. Awaited at batch
    /// boundaries, on shutdown, and by the flush task.
    pub async fn flush(&self) -> Result<usize>;
}
```

`Database` gains one narrow batch primitive:

```rust
/// One `UPDATE` per row inside a single transaction, covering only the
/// mutable columns (latency*, speed_bps, error*, task_id, task_queue).
pub async fn apply_link_patches(&self, patches: &[ProfileStats]) -> Result<usize>;
```

`upsert_link` stays for the subscription/import path (whole-row semantic);
the batch path no longer uses it.

### 4.2 Authority (the crux)

The scheduler gate treats the persisted row as authoritative: `schedule` and
`complete` re-read the link inside the critical section and reject stale
snapshots or mismatched task ids. Behind a write-behind buffer a raw
`read_link` would return **pre-flush** state, so the gate could see a link as
free while its task is running (duplicate start) or label a valid completion
stale.

**Decision: the `LinkWriter` pending map is authoritative for the gate; the
database is the durable mirror.**

- `SchedulerDb::read_link` is implemented as `LinkWriter::read` (read-through).
- A transition stages the new row state; the gate's next read observes it,
  because the gate mutex serializes transitions and staging is synchronous.
- The database is flushed at the deadline, at `flush_rows`, at batch end, and
  on shutdown; until then it may lag, which is exactly the durability window
  named in §6.
- Task state is therefore *not* required to be durable per transition; the
  existing orphan sweep reconciles a crash (an id persisted as in-flight with
  no live task is cleared on the next startup pass — the behaviour already
  documented in `ops/scheduler.rs`).

Rejected alternatives:

- Flush before every gate read — reintroduces a commit per transition, the
  original cost.
- Keep the gate on the database and stage only result writes — leaves 2 of 3
  commits per link in place and adds a second writer to the same row.
- Per-transition in-memory writer with flush boundaries aligned to transitions
  — same cost as today, plus a correctness surface the read-through removes.

### 4.3 Flush policy

A single background task owns flushing (one writer, so no OCC contention):

- flush when `pending.len() >= flush_rows` (512) or every `flush_interval`
  (200 ms) while non-empty;
- `stage` never awaits; if a flush is in flight, staging keeps filling the map
  (the flush drains a snapshot and re-checks);
- batch end calls `flush()` explicitly, so a finished batch is fully durable;
- on `Database` close/quit, `flush()` runs once;
- after a batch's final flush, one `PRAGMA wal_checkpoint(TRUNCATE)` bounds WAL
  growth (the investigation showed per-commit cost rising as the WAL grew).

Concurrency: one flush task plus `flush()` being callable from the batch end —
both take a `tokio::Mutex` around the drain-and-write step so a snapshot is
never written twice.

### 4.4 Call-site changes

| Site | Today | After |
| --- | --- | --- |
| `SpeedTestResult` handler | `db.upsert_link(link).await` in the UI task | mutate the in-memory link, `writer.stage(link)` |
| `TaskScheduler::schedule` / `complete` | `read_link` + `update_scheduler_state` (2 commits) | read through the writer; `writer.stage(row)` |
| `SchedulerDb` impl | database reads/writes | `LinkWriter::read` + `stage` |
| Batch end (`ops/ping.rs`) | — | `writer.flush()` + checkpoint |
| `clear_expired_errors` | bulk UPDATE | unchanged (rare, runs on reload) |

`update_scheduler_state` and its OCC retry remain for any non-batch caller, but
the batch path stops using them; the `version` column stops being the gate's
concurrency mechanism (the gate mutex plus the single writer replace it). The
typed OCC path must not be reachable concurrently with the writer for the same
row — `apply_link_patches` writes the same columns, so a concurrent
`update_scheduler_state` would be a lost update. Enforced by: the scheduler's
only write path becomes `stage`.

### 4.5 UI visibility

The UI keeps mutating `state.endpoints` rows directly (decision-15 in-memory
sync rule); the writer is persistence-facing only. Nothing in the render path
awaits a flush.

## 5. Blast radius

- `ops/events.rs` — result handler loses its awaited write.
- `ops/scheduler.rs` — `SchedulerDb` implementation; `schedule`/`complete` stop
  awaiting commits.
- `ops/ping.rs` — batch lifecycle calls `flush()`.
- `state.rs` — holds `Arc<LinkWriter>`.
- `main.rs` / `ui/mod.rs` — construct the writer, flush on quit.
- `xray-tui-db` — `apply_link_patches`; integration test for the transaction
  batching.

## 6. Compatibility and durability boundary

- No schema or data-format change; existing databases open unchanged.
- Durability window: staged mutations are lost on a crash/power loss up to
  `flush_interval` (200 ms) — the same class of loss the `synchronous=NORMAL`
  change already accepts, and strictly narrower than "the batch's results".
- A crash mid-batch leaves at most one flush window of results unwritten and
  stale `task_id`s that the startup orphan sweep clears.
- Subscription/import writes keep their existing transactional path.

## 7. Verification

1. **Gate read-through (correctness)** — stage a `schedule` transition without
   flushing, then `schedule` the same link again and `complete` it: the second
   `schedule` must return the queued/busy outcome (no double start) and
   `complete` must accept the matching id. Also cover: staged transition, then
   a `complete` with a mismatched id → stale no-op.
2. **Flush batching** — stage 1000 mutations; assert one transaction and 1000
   narrow updates, and that a fresh read sees them only after `flush()`.
3. **Final-state equality** — after `flush()`, every staged row equals the
   persisted row (all mutable columns).
4. **No UI-task commit** — a poll pass that drains `EVENT_DRAIN_BUDGET` result
   events performs zero commits (asserted by a test writer counting flushes).
5. **Throughput** — a 30k-link synthetic batch: wall time, transaction count,
   and max tick duration; compared against the post-`synchronous` baseline in
   §1, with the target in §2.3.
6. **Crash reconciliation** — stale `task_id` persisted at flush, restart,
   orphan sweep clears the gate.
7. **TUI smoke** — real batch over the large subscription: UI stays responsive
   (keys respond, render keeps up) while results re-sort.

## 8. Risks and tripwires

| Risk | Mitigation |
| --- | --- |
| Gate reads stale DB state | Read-through is the design (§4.2); test §7.1 is the regression |
| Two writers on one row (typed OCC + batch UPDATE) | The scheduler's only write path becomes `stage`; grep gate for `update_scheduler_state` in the batch path |
| Flush task starves under a long transaction | Flush is one transaction per window; `flush_rows` caps its size |
| WAL growth | Checkpoint at batch end; `flush_interval` bounds in-flight work |
| Lost results on crash | Named in §6; the orphan sweep covers task state |
| `pending` map memory during a 100k-link batch | Bounded by distinct changed links (≤ links) and drained each window |

## 9. Retirement

| Old path | Status | Trigger |
| --- | --- | --- |
| `db.upsert_link` in the result handler | retired by this change | writer lands |
| `TaskScheduler` `read_link`/`update_scheduler_state` per transition | retired for the batch path | read-through lands |
| `update_scheduler_state` OCC retry | kept for non-batch callers; delete when none remain | last caller migrates |
| `PROFILES_SQL_DEBUG`-style probes | already removed | — |
