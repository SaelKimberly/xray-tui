# RowCache: systematized write-behind for xray-tui-db

Date: 2026-10-05. Status: design (grill-approved, approach A).

## 1. Problem

Large-subscription refreshes freeze the TUI. 2026-10-05 evidence: a 51,087-link
import at 07:01:09Z precedes sustained `query failed [5000ms]` timeouts on
`INSERT/UPDATE endpoint_ip`, `slow query [3334ms] INSERT INTO profile_stats`,
slow `endpoint_rank SELECT` (1.0–3.6s), and 88x `country persist failed:
snapshot stale, rollback and retry`. Write contention on the Turso/SQLite
connection, not LMDB (heed store healthy, `queue-full=0`, no MapFull).

Write batching already exists in three places, each hand-rolled:

- `LinkWriter` (`crates/xray-tui/src/ops/link_writer.rs`): stages `ProfileStats`
  snapshots per `(link, LinkGroups)` in a `DashMap`, one flush task, one
  transaction per window via `Database::apply_link_patches`. Four triggers:
  row floor (`flush_rows`), timer floor (`flush_rows / TIMER_FLOOR_DIVISOR`),
  staleness deadline (`MAX_STAGED_AGE_TICKS` × interval = 15s), explicit
  `flush_soon()` wake at batch end / quit / reload.
- `GeoQueue` (`crates/xray-tui/src/ops/enrich.rs:78`): accumulates
  `(EndpointId, IpAddr, country)` across hosts, single drain task every
  `GEO_DRAIN_INTERVAL` (5s), one transaction per drain via
  `set_endpoint_ip_countries`. Failed drain re-queues.
- Import (`ops/stream_import.rs`, `ops/subscriptions.rs`): 500-URL chunks, one
  transaction per chunk (`upsert_protocols_bulk` + `upsert_links_bulk` +
  `upsert_endpoint_group_links_bulk`), `retry_on_busy` × 5.
- DNS persist batch (`ops/events.rs` EndpointInfoUpdated arm): one spawned
  flush per poll tick for `update_endpoint_resolution` rows.

Heed log writer is EXCLUDED: separate LMDB store, no shared transaction with
Turso, no contention domain overlap.

Goal: one shared write-behind driver in `xray-tui-db` that every write path
uses, with accepted durability loss (~10–15s window) for systematic
contention reduction.

## 2. Grill decisions (locked)

| Question | Decision |
|---|---|
| Template | Adopt `LinkWriter` behavior as the standard |
| Heed | Excluded, separate mechanism |
| Ownership | Shared owner: `&self` + interior mutability, `Arc`-shared, `Send+Sync` |
| Flush errors | Re-stage failing window AND all later windows, bounded retry at op end |
| Patch composition | Per-table patch type (`CacheSpec::Patch`), table owns coalesce |
| Shared gain | Generic drain driver; tables plug in coalesce + single-tx write + refresh |
| Scope | ALL write paths including import SOURCE (crash may drop profiles; next refresh re-adds) |
| Triggers | Same 4-way policy as `LinkWriter` |
| Rank refresh | Driver owns a refresh hook run inside the flush tx |

Rejected: draft `trait RowCache { type Row; push(&mut self); flush(&mut self); }` —
exclusive borrow blocks a shared drain task, sync infallible flush cannot
`retry_on_busy`, no room for gate/timers/hooks.

## 3. Design

### 3.1 `CacheSpec` trait (per table)

```rust
pub trait CacheSpec {
    type Key: Eq + Hash + Clone;
    type Row: Clone;
    type Patch;
    fn key_of(row: &Self::Row) -> Self::Key;
    fn coalesce(rows: Vec<Self::Row>) -> Vec<Self::Patch>;
    async fn write_window(tx: &mut Tx, patches: &[Self::Patch]) -> Result<usize>;
    async fn refresh(tx: &mut Tx, patches: &[Self::Patch]) -> Result<()>;
}
```

Driver owns the transaction: `flush` opens one tx per chunk, calls
`write_window(&mut tx, …)` then `refresh(&mut tx, …)`, then commits.
`Tx` is the toasty transaction type (`impl toasty::Executor`). This mirrors
`Database::apply_link_patches`, which opens the tx and calls
`endpoint_rank::refresh` inside it — the spec never splits write and refresh
across transactions.

### 3.2 Generic driver `WriteBehind<S: CacheSpec>`

Owned mechanics, copied from `LinkWriter`:

- `pending: DashMap<S::Key, S::Row>`, `staged: AtomicU64` (no `pending.len()`
  walk — 1.06µs/call measured).
- `gate: tokio::Mutex<()>` serializes drain-and-write; `wake: Notify`.
- `push(&self, row: S::Row)`: clone-free where possible, never awaits, notifies
  at `flush_rows`.
- `flush(&self) -> Result<usize>`: gate-locked drain (remove, not snapshot —
  concurrent stages land as fresh entries for the next window), chunked
  single-tx writes, re-stage of failing + later windows on error.
- `run(self: Arc<Self>)`: size trigger always writes; timer tick writes at
  floor or staleness deadline; failed flush keeps clock (retry on deadline,
  not spin).
- `should_flush(floor, woken, staged, stale)`: pure fn, unit-tested (the
  `LinkWriter` contract test moves here).
- `flush_soon()`, `flush_count()`, `staged_len()`: same diagnostics.

Policy defaults: `flush_rows = 512`, interval 200ms, divisor 4, deadline 15s —
per-table overrides allowed (import chunks at 500 suggest same order).

### 3.3 Durability contract

- Loss window: up to `max_staged_age` (15s default) on crash; explicit flush at
  op end (batch `finish_batch`, import end, quit, reload-before-sweep).
- `finish_batch`-style end ops retry final flush bounded (`FINAL_FLUSH_ATTEMPTS`
  pattern).
- Staged rows are invisible to DB-only readers until flush. Tables whose
  callers read-back (scheduler gate over staged task state — runtime-only,
  unaffected; batch `emit_result` reads) need a read-through accessor over
  the pending map, or callers must stage-then-read via the driver. Open item
  per table at migration time; default is no read-through unless a caller
  proves the need (the `LinkWriter` gate reads its own map today — same rule).

### 3.4 Refresh hook

`refresh` runs inside the flush transaction after `write_window`, for the
flushed endpoints only (not full-table). `endpoint_rank::refresh` is the first
implementation; import bulk path calls the same hook the current code calls
separately today.

## 4. Migration order

1. `GeoQueue` → `WriteBehind<CountrySpec>`: smallest, isolated, no rank hook
   (country writes touch `endpoint_ip`, rank unaffected). Validates driver.
2. Import SOURCE path → `WriteBehind<SourceSpec>`: chunked bulk upserts become
   staged windows; end-of-import flush replaces chunk commits; rank refresh
   moves into the hook.
3. `LinkWriter` → `WriteBehind<LinkSpec>`: last, largest caller surface
   (RESULT/PURGE/TRAFFIC groups become `coalesce`; gate read-through preserved).
   Delete old `LinkWriter` on completion — no shims.

Each step: driver + one spec live alongside old code, old path removed in the
same commit (clean cutover, decision: no parallel write paths).

## 5. Testing

- Unit: `should_flush` truth table (moved from `LinkWriter`), coalesce tests
  per spec (group union, disjoint columns, no shadowing).
- Failure injection: re-stage writes full remainder (the `LinkWriter::flush`
  regression test pattern: fail chunk N, assert chunks N.. retried).
- Guard test: `push`/drain performs no commit on caller task
  (`draining_results_performs_no_commit_on_the_ui_task` pattern per table).
- Contention: `flow_cost.rs` fan-in row (import + geo overlap) as before/after
  measure — target is fewer commits + zero 5s timeouts at same throughput.
- Full suite green per step; `just quality-gate code` at the end.

## 6. Risks

- Import SOURCE in the loss window: crash between stage and flush drops
  profiles the UI may already list (if rows render from staged state) or
  silently miss (if render reads DB). Mitigation: import renders from staged
  state + explicit end-of-import flush; document that a crash loses at most
  the last window, recovered by next refresh.
- Cross-table atomicity loss (import): today one tx commits endpoints +
  protocols + links + group_links (`stream_import.rs:397-403`), so 0 linkless
  endpoints exist. Per-table drivers split this into independent commits; a
  crash between the endpoints window and the links window leaves endpoint rows
  with no links — a NEW state. Accepted tradeoff: end-of-import performs a
  coordinated flush (all four drivers, endpoints-first order) and reports
  staged-left per driver; orphan endpoints (no links after flush) are repaired
  by the next refresh re-adding their links, or swept by `purge_expired` when
  older than retention. A linkless-endpoint count probe joins the §5 suite.
- Generic driver over `DashMap` per table: N maps, N tasks — same as today,
  no regression, but no cross-table scheduling either (approach C rejected;
  revisit only if cross-table contention persists after migration).
- `refresh` inside tx lengthens the flush transaction (rank refresh cost on
  400-endpoint windows measured ~10ms) — acceptable, and current code pays it
  in the same tx already.
