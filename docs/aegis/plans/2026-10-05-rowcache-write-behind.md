# RowCache Write-Behind Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the three hand-rolled write batchers (LinkWriter, GeoQueue, import chunk commits) with one generic `WriteBehind<S: CacheSpec>` driver in `xray-tui-db`.

**Architecture:** New `crates/xray-tui-db/src/write_behind.rs` owns the generic driver (DashMap + atomics + gate Mutex + Notify + 4-trigger loop, copied from `LinkWriter`). Each table plugs in a `CacheSpec` impl (coalesce + tx-scoped write + tx-scoped refresh). Migration order: driver + CountrySpec first (GeoQueue), then SourceSpec (import), then LinkSpec (LinkWriter); old code deleted in the same commit at each step.

**Tech Stack:** Rust 2024, tokio (Notify/Mutex/spawn), DashMap, toasty 0.11 + toasty-driver-turso (Executor/tx), retry_on_busy.

**Spec:** docs/aegis/specs/2026-10-05-rowcache-write-behind-design.md

## Global Constraints

- Rust 2024 edition; workspace clippy `pedantic` + `nursery` at warn; `cargo fmt` clean.
- Every public `Database` method touched keeps its `#[tracing::instrument(target = "db_method", skip_all, fields(retries = ...))]` so DbMonitorLayer keeps attributing.
- toasty `#[index]` is single-column only: mixed-direction composites stay raw DDL, do not "simplify".
- Schema tag stays 14: no model/column changes in this plan (bump = data wipe, never a migration).
- Raw SQL additions require a `docs/database-manual-sql.md` entry (cause + measurement + Toasty blocker).
- `push()` never awaits; no commit on the caller/UI task (standing guard test per table).
- Failed flush re-stages failing window AND all later windows; end ops retry bounded.
- `write_window` + `refresh` both take `&mut Tx`; driver opens one tx per chunk, writes, refreshes, commits.
- Clean cutover per task: old path deleted in the same commit, no shims, no parallel paths.
- Commit per task; full suite green per task.

## Review Focus

- Crash between staged import windows leaves linkless endpoints (today impossible: 0 exist); end-of-import coordinated flush + orphan repair must hold, most likely first.
- A `push` landing mid-flush must survive as a fresh pending entry, never dropped by the drain or overwritten by re-stage.
- Timer-floor/timer-deadline interaction must reproduce LinkWriter's 4-way policy exactly (floor decides size, deadline only nets trickles).
- Staged rows invisible to DB-only readers until flush; any caller that reads back its own stage needs read-through or must break.
- Import end-of-op flush ordering (endpoints first) must precede the linkless-endpoint probe, or the probe false-positives.

---

### Task 1: Generic driver `WriteBehind<S: CacheSpec>`

**Files:**
- Create: `crates/xray-tui-db/src/write_behind.rs`
- Modify: `crates/xray-tui-db/src/lib.rs` (re-export `WriteBehind`, `CacheSpec`)
- Test: inline `#[cfg(test)]` in `write_behind.rs` with a fake in-memory spec

**Interfaces:**
- Consumes: `crate::retry_on_busy`, `toasty::Executor` (tx type alias `Tx`)
- Produces: `pub trait CacheSpec { Key, Row, Patch; key_of; coalesce; write_window(tx, patches); refresh(tx, patches) }`, `pub struct WriteBehind<S: CacheSpec>` with `new(flush_rows, flush_interval)`, `push(&self, row)`, `flush(&self) -> Result<usize>`, `flush_soon()`, `run(self: Arc<Self>, write: Arc<dyn write fn>)`, `should_flush(floor, woken, staged, stale) -> bool` (pure, `const`), `staged_len()`, `flush_count()`, `timer_flush_floor()`, `max_staged_age()`, `spawn_flush_task()`

**Design note:** The driver cannot name the toasty tx type generically without a bound the crate already has: bulk writers take `tx: &mut impl Executor`. `CacheSpec::write_window`/`refresh` take the same `&mut impl Executor` via a generic method. The driver opens the tx through a `Database` handle it holds (`Arc<Database>` like `LinkWriter`), so `WriteBehind::new(db: Arc<Database>, ...)` and `flush()` does `let mut conn = db.conn().await; let mut tx = conn.transaction().await; S::write_window(&mut tx, chunk).await; S::refresh(&mut tx, chunk).await; tx.commit().await` wrapped in `retry_on_busy`. Per-chunk tx (not per-flush): a 4,028-patch flush at 512/chunk = 8 commits, matching today's chunking.

- [ ] **Step 1: Write the failing test — should_flush truth table**

```rust
#[test]
fn should_flush_truth_table() {
    // woken always writes
    assert!(WriteBehind::<FakeSpec>::should_flush_for_test(128, true, 0, false));
    // timer floor
    assert!(WriteBehind::<FakeSpec>::should_flush_for_test(128, false, 128, false));
    assert!(!WriteBehind::<FakeSpec>::should_flush_for_test(128, false, 127, false));
    // staleness net
    assert!(WriteBehind::<FakeSpec>::should_flush_for_test(128, false, 1, true));
    // nothing staged, not woken, not stale
    assert!(!WriteBehind::<FakeSpec>::should_flush_for_test(128, false, 0, false));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-db write_behind::tests::should_flush_truth_table`
Expected: FAIL with "unresolved module / not found" (driver does not exist yet)

- [ ] **Step 3: Write minimal driver implementation**

```rust
// crates/xray-tui-db/src/write_behind.rs
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use dashmap::DashMap;

pub const DEFAULT_FLUSH_ROWS: usize = 512;
pub const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_millis(200);
pub const TIMER_FLOOR_DIVISOR: usize = 4;
pub const MAX_STAGED_AGE_TICKS: u32 = 75;

pub trait CacheSpec {
    type Key: Eq + Hash + Clone + Copy;
    type Row: Clone;
    type Patch;
    fn key_of(row: &Self::Row) -> Self::Key;
    fn coalesce(rows: Vec<Self::Row>) -> Vec<Self::Patch>;
    fn write_window<'a>(
        tx: &'a mut impl toasty::Executor,
        patches: &'a [Self::Patch],
    ) -> impl std::future::Future<Output = crate::Result<usize>> + 'a;
    fn refresh<'a>(
        tx: &'a mut impl toasty::Executor,
        patches: &'a [Self::Patch],
    ) -> impl std::future::Future<Output = crate::Result<()>> + 'a;
}

pub struct WriteBehind<S: CacheSpec> {
    pending: DashMap<S::Key, S::Row>,
    staged: AtomicU64,
    flush_rows: usize,
    timer_flush_floor: usize,
    flush_interval: Duration,
    max_staged_age: Duration,
    gate: tokio::sync::Mutex<()>,
    wake: tokio::sync::Notify,
    flushes: AtomicU64,
    write: Arc<dyn Fn(Arc<crate::Database>, Vec<S::Patch>) -> () + Send + Sync>,
    _spec: std::marker::PhantomData<S>,
}
```

NOTE: the `write` closure above is a placeholder shape — the real driver holds `db: Arc<crate::Database>` and opens the tx itself (see design note), calling `S::write_window(&mut tx, chunk)` then `S::refresh(&mut tx, chunk)` then commit, all inside `retry_on_busy`. Implement with the db handle, not a closure:

```rust
pub struct WriteBehind<S: CacheSpec> {
    pending: DashMap<S::Key, S::Row>,
    staged: AtomicU64,
    flush_rows: usize,
    timer_flush_floor: usize,
    flush_interval: Duration,
    max_staged_age: Duration,
    db: Arc<crate::Database>,
    gate: tokio::sync::Mutex<()>,
    wake: tokio::sync::Notify,
    flushes: AtomicU64,
    _spec: std::marker::PhantomData<S>,
}

impl<S: CacheSpec> WriteBehind<S> {
    #[must_use]
    pub fn new(db: Arc<crate::Database>, flush_rows: usize, flush_interval: Duration) -> Arc<Self> {
        let flush_rows = flush_rows.max(1);
        Arc::new(Self {
            pending: DashMap::new(),
            staged: AtomicU64::new(0),
            flush_rows,
            timer_flush_floor: (flush_rows / TIMER_FLOOR_DIVISOR).max(1),
            flush_interval,
            max_staged_age: flush_interval.saturating_mul(MAX_STAGED_AGE_TICKS),
            db,
            gate: tokio::sync::Mutex::new(()),
            wake: tokio::sync::Notify::new(),
            flushes: AtomicU64::new(0),
            _spec: std::marker::PhantomData,
        })
    }

    fn put(&self, row: S::Row) {
        if self.pending.insert(S::key_of(&row), row).is_none() {
            self.staged.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn push(&self, row: S::Row) {
        self.put(row);
        if self.staged_len() >= self.flush_rows {
            self.wake.notify_one();
        }
    }

    fn take(&self, key: &S::Key) -> Option<S::Row> {
        let removed = self.pending.remove(key).map(|(_, row)| row);
        if removed.is_some() {
            self.staged.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    fn drain(&self) -> Vec<S::Patch> {
        let keys: Vec<S::Key> = self.pending.iter().map(|e| *e.key()).collect();
        let rows: Vec<S::Row> = keys.iter().filter_map(|k| self.take(k)).collect();
        S::coalesce(rows)
    }

    pub async fn flush(&self) -> crate::Result<usize> {
        let _guard = self.gate.lock().await;
        let patches = self.drain();
        if patches.is_empty() {
            return Ok(0);
        }
        let mut written = 0usize;
        // Re-stage list for the failure path: keys must be recoverable from patches,
        // so CacheSpec::Patch must expose its rows. See Task 1 design adjustment below.
        for (index, chunk) in patches.chunks(self.flush_rows).enumerate() {
            let db = Arc::clone(&self.db);
            let result = crate::retry_on_busy(
                || async {
                    let mut conn = db.conn().await?;
                    let mut tx = conn.transaction().await?;
                    let n = S::write_window(&mut tx, chunk).await?;
                    S::refresh(&mut tx, chunk).await?;
                    tx.commit().await?;
                    Ok::<usize, crate::DatabaseError>(n)
                },
                5,
            )
            .await;
            match result {
                Ok(n) => {
                    written += n;
                    self.flushes.fetch_add(1, Ordering::Relaxed);
                }
                Err(err) => {
                    // Re-stage failing chunk AND all later chunks.
                    for pending_row in patches
                        .chunks(self.flush_rows)
                        .skip(index)
                        .flat_map(|c| S::rows_of(c))
                    {
                        let key = S::key_of(&pending_row);
                        match self.pending.entry(key) {
                            dashmap::mapref::entry::Entry::Occupied(mut slot) => {
                                slot.insert(pending_row);
                            }
                            dashmap::mapref::entry::Entry::Vacant(slot) => {
                                slot.insert(pending_row);
                                self.staged.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    return Err(err);
                }
            }
        }
        Ok(written)
    }
}
```

DESIGN ADJUSTMENT (required, do not skip): `coalesce(rows) -> patches` is lossy — the failure path needs rows back from patches to re-stage. Add `fn rows_of(patches: &[Self::Patch]) -> Vec<Self::Row>` to `CacheSpec` (patches must retain enough to rebuild rows; for LinkSpec the patch already carries the full `ProfileStats` snapshot, so this is exact). Alternatively `coalesce` returns `Vec<(Key, Row, Patch)>` triples. Pick triples: `fn coalesce(rows: Vec<Row>) -> Vec<Coalesced<Row, Patch>>` where `struct Coalesced<R, P> { key: impl Key, row: R, patch: P }` — the drain removes by key, flush writes patches, re-stage re-inserts rows. Implement triples; do NOT implement lossy `rows_of` reconstruction.

Also implement: `staged_len()`, `flush_count()`, `flush_soon()`, `timer_flush_floor()`, `max_staged_age()`, `const fn should_flush(floor, woken, staged, stale) -> bool { woken || staged >= floor || stale }`, `run(self: Arc<Self>)` loop (size trigger always writes; timer at floor or stale; failed flush keeps clock), `spawn_flush_task(&self) -> JoinHandle<()>`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-db write_behind`
Expected: PASS (truth table + any new unit tests)

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/write_behind.rs crates/xray-tui-db/src/lib.rs
git commit -m "feat(db): generic WriteBehind drain driver with CacheSpec"
```

### Task 2: CountrySpec — migrate GeoQueue

**Files:**
- Modify: `crates/xray-tui-db/src/write_behind.rs` (add `CountrySpec` + `CountryPatch`)
- Modify: `crates/xray-tui/src/ops/enrich.rs` (replace `GeoQueue`/`queue_country`/`take_queued`/`spawn_geo_drain`/`drain_once` internals with `WriteBehind<CountrySpec>`)
- Test: `crates/xray-tui-db/src/write_behind.rs` inline (coalesce: last-writer-wins per endpoint)

**Interfaces:**
- Consumes: `Database::set_endpoint_ip_countries(&[(EndpointId, IpAddr, String)])` (database.rs:1385), `WriteBehind` from Task 1
- Produces: `pub struct CountrySpec`, `pub struct CountryPatch { endpoint_id: EndpointId, ip: IpAddr, iso: String }`; `enrich::queue_country(endpoint_id, ip, iso)` keeps its signature (callers unchanged); drain cadence stays `GEO_DRAIN_INTERVAL` semantics via driver policy (flush_rows = GEO_FLUSH_AT, interval = drain interval)

**Design note:** `CountrySpec::coalesce` = last-writer-wins per `EndpointId` (today's DashMap insert overwrites). `write_window` calls `set_endpoint_ip_countries` — but that method opens its OWN connection + retry internally; the driver already opened a tx. Refactor: extract the inner statement (`set_endpoint_ip_countries_once(tx, rows)`) so the driver tx is used; keep the public method as a one-shot wrapper for non-driver callers. Same pattern as `apply_link_patches` / `apply_link_patches_once` (database.rs:1131-1138). `refresh` = no-op (country writes touch `endpoint_ip`, rank unaffected — spec §4).

- [ ] **Step 1: Write the failing test — coalesce last-writer-wins**

```rust
#[test]
fn country_coalesce_last_writer_wins() {
    let rows = vec![
        country_row(1, "1.1.1.1", "US"),
        country_row(1, "1.1.1.1", "DE"),
        country_row(2, "2.2.2.2", "FR"),
    ];
    let patches = CountrySpec::coalesce(rows);
    assert_eq!(patches.len(), 2);
    let one = patches.iter().find(|p| p.endpoint_id == EndpointId::new(1)).unwrap();
    assert_eq!(one.iso, "DE");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-db country_coalesce_last_writer_wins`
Expected: FAIL with "CountrySpec not defined"

- [ ] **Step 3: Implement CountrySpec + enrich.rs cutover**

```rust
pub struct CountryRow { pub endpoint_id: EndpointId, pub ip: std::net::IpAddr, pub iso: String }
pub struct CountrySpec;
impl CacheSpec for CountrySpec {
    type Key = EndpointId;
    type Row = CountryRow;
    type Patch = CountryRow; // 1:1, no composition
    fn key_of(row: &Self::Row) -> Self::Key { row.endpoint_id }
    fn coalesce(rows: Vec<Self::Row>) -> Vec<Coalesced<Self::Row, Self::Patch>> {
        // last-writer-wins per endpoint: later row overwrites earlier
        let mut map = std::collections::HashMap::new();
        let mut order = Vec::new();
        for row in rows {
            if !map.contains_key(&row.endpoint_id) {
                order.push(row.endpoint_id);
            }
            map.insert(row.endpoint_id, row);
        }
        order.into_iter().map(|k| {
            let row = map.remove(&k).unwrap();
            Coalesced { key: k, row: row.clone(), patch: row }
        }).collect()
    }
    // write_window / refresh as async fns using the driver's &mut tx
}
```

In `enrich.rs`: delete `GeoQueue`, `GEO_QUEUE`, `take_queued`, `drain_once`; `queue_country` becomes `driver.push(CountryRow {...})`; `spawn_geo_drain` becomes `driver.spawn_flush_task()` + shutdown-aware wrapper that does a final `flush()` on quit (today's "one last drain" behavior). Failed-drain re-queue is now the driver's re-stage (no separate code).

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-db write_behind && cargo test -p xray-tui enrich`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/write_behind.rs crates/xray-tui/src/ops/enrich.rs
git commit -m "feat(db): migrate GeoQueue onto WriteBehind<CountrySpec>"
```

### Task 3: SourceSpec — migrate import bulk path

**Files:**
- Modify: `crates/xray-tui-db/src/write_behind.rs` (add `SourceSpec` with 4-family patch)
- Modify: `crates/xray-tui/src/ops/stream_import.rs` (chunk persist → stage + end-of-import coordinated flush)
- Modify: `crates/xray-tui/src/ops/subscriptions.rs` (same for the non-streaming bulk persist at :592-596)
- Test: inline + linkless-endpoint count probe (spec §6: query endpoints with no links, assert 0 after coordinated flush)

**Interfaces:**
- Consumes: `upsert_endpoints_bulk`, `upsert_protocols_bulk`, `upsert_links_bulk`, `upsert_endpoint_group_links_bulk` (all `tx: &mut impl Executor` free fns, database.rs:1810-1939), `endpoint_rank::refresh`, `purge_expired`
- Produces: `pub struct SourceSpec` with patch carrying all four families per batch; coordinated `flush_all_drivers(endpoints_first)` helper; `staged-left` per-driver report at import end

**Design note (spec §6 atomicity, do not skip):** Per-table drivers split today's 4-table single tx. This task uses ONE `SourceSpec` driver (not four) whose `write_window` runs all four bulk upserts in the driver's tx — preserving single-tx atomicity per window and avoiding the linkless-endpoint state entirely within a window. Cross-WINDOW atomicity (window 1 committed, window 2 staged at crash) is the accepted tradeoff: end-of-import coordinated `flush()` + linkless-endpoint probe; orphans repaired by next refresh / swept by `purge_expired`. This is simpler than four drivers + ordering and keeps the "0 linkless endpoints" invariant per committed window.

**`SourceSpec::Key` = `u64` monotonic batch id (REQUIRED, do not use a constant or content-derived key):** the driver stores rows in `DashMap<Key, Row>`, so a non-unique key lets a later chunk overwrite an earlier still-pending batch and its rows are never written. The tail path (`stream_import.rs:262` `while let Some(tail) = batcher.take_batch()`) stages several batches back-to-back, so concurrent pending batches are the normal case. The caller mints ids from an `AtomicU64` (`fetch_add(1)`) per staged chunk; `Row = SourceBatch { seq: u64, ... }`, `key_of` returns `row.seq`. `u64` satisfies Task 1's `Key: Clone + Copy` bound.

**`ImportOutcome` redefinition under deferred writes (REQUIRED — `a_dropped_batch_is_reported_not_swallowed` pins today's semantics):** today `persist_batch` returns `(batch_links, dropped, summary)` synchronously and `links` = STORED count. Under staging:
- Parse-time accounting stays per batch at stage time: `batch_links` (deduped links parsed) accumulates into `staged_count`; parse failures/`batch.len()` drops accumulate into `dropped_total` exactly as today. `on_progress(staged_count)` keeps its meaning (parsed, not yet stored).
- Persist-time failures surface ONLY at end-of-import flush: the final bounded-retry `flush()` reports `stored: usize` (rows written) and `staged_left: usize`. `flush_dropped = staged_count - stored` folds into `dropped_total` and appends to `ended_early` with the same `"; plus {base}"` shape as the existing dropped_total block (`stream_import.rs:310-319`). A flush failure is therefore still reported, never swallowed — attributed to the import as a whole rather than to one batch.
- `ImportOutcome.links` = STORED count after the final flush (reconciled: `stored`, not `staged_count`). Update the `ImportOutcome` doc comment (`links` = "only what was STORED" invariant preserved) and the `a_dropped_batch_is_reported_not_swallowed` test still passes: inject a flush failure, assert `dropped_total > 0` and `ended_early.is_some()`.
- `subscriptions.rs` non-streaming bulk persist (`:592-596`) gets the same treatment: stage, final flush, reconcile.

- [ ] **Step 1: Write the failing test — linkless probe**

```rust
#[tokio::test]
async fn import_flush_leaves_no_linkless_endpoints() {
    let db = Database::in_memory().await.unwrap();
    let driver = WriteBehind::<SourceSpec>::new(Arc::new(db.clone()), 512, Duration::from_millis(200));
    driver.push(source_batch_with(3, 2)); // 3 endpoints, 2 links (one orphan by construction)
    driver.flush().await.unwrap();
    let orphans = linkless_endpoint_count(&db).await;
    // This test documents the accepted state: intra-window orphans can only come
    // from the input batch itself, never from split commits (single tx per window).
    assert_eq!(orphans, 1);
}
```

Plus a second test: full 3-endpoint/3-link batch flushes to 0 orphans.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-db import_flush_leaves_no_linkless`
Expected: FAIL with "SourceSpec not defined"

- [ ] **Step 3: Implement SourceSpec + cut over both import sites**

`stream_import.rs` chunk persist (lines ~389-406): split `persist_batch` into parse-half (returns row vecs + `batch_links` + summary, no DB) and stage-half (`driver.push(SourceBatch { seq: next_seq(), endpoints, protocols, links, group_links })` per chunk; accumulate `staged_count += batch_links`, `dropped_total` for parse drops as today). After stream end + tail drain: final `driver.flush().await` with `FINAL_FLUSH_ATTEMPTS`-style bounded retry (copy the `ping.rs:1449` loop, 50ms<<attempt backoff); reconcile `stored` vs `staged_count`, fold `staged_count - stored` into `dropped_total` + `ended_early` (`"; plus {base}"` shape); `ImportOutcome.links = stored`. Same for `subscriptions.rs:592-596`. Delete the old closures; `retry_on_busy` now lives in the driver.

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-db && cargo test -p xray-tui stream_import subscriptions`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-db/src/write_behind.rs crates/xray-tui/src/ops/stream_import.rs crates/xray-tui/src/ops/subscriptions.rs
git commit -m "feat(db): migrate import bulk persist onto WriteBehind<SourceSpec>"
```

### Task 4: LinkSpec — migrate LinkWriter, delete old code

**Files:**
- Modify: `crates/xray-tui-db/src/write_behind.rs` (add `LinkSpec`)
- Modify: `crates/xray-tui/src/ops/link_writer.rs` → DELETE file; all `link_writer.stage/flush/flush_soon/run/spawn_flush_task` callers switch to `WriteBehind<LinkSpec>`
- Modify callers: `crates/xray-tui/src/ops/events.rs` (stage sites :421, :676, :823; guard test :1828), `crates/xray-tui/src/ops/connect.rs` (:646-656), `crates/xray-tui/src/ops/profiles.rs` (:217), `crates/xray-tui/src/ops/ping.rs` (:877, :1023, :1450, :2256, :2316), `crates/xray-tui/src/state.rs` (field type), `crates/xray-tui/src/ops/ping/flow_cost.rs` (bench harness)
- Modify: `crates/xray-tui-db/src/lib.rs` (move `LinkGroups`, `LinkPatch` re-export if still needed by callers, or move type into write_behind.rs)
- Test: port `link_writer.rs` tests (should_flush table, drain coalesce, re-stage regression, no-commit-on-UI-task guard, reload-before-sweep ordering) onto `WriteBehind<LinkSpec>`

**Interfaces:**
- Consumes: `Database::apply_link_patches` inner (needs tx-scoped split like Task 2: extract `apply_link_patches_once_tx(tx, patches)`; keep public wrapper), `endpoint_rank::refresh` (already inside apply path — database.rs:1193; LinkSpec::refresh = no-op to avoid double refresh, DOCUMENT this)
- Produces: `pub struct LinkSpec`; `LinkGroups`/`LinkPatch` unchanged shapes; scheduler-gate read-through preserved (gate reads driver's pending map — expose `get_staged(key) -> Option<Row>` on driver, replacing today's direct `pending` access)

**Design note:** `LinkSpec::coalesce` = the `LinkWriter::drain` merge verbatim (one patch per link, union of RESULT/PURGE/TRAFFIC groups, `merge_group` overlay). `write_window` = chunked `exec_link_upsert` path (today's `apply_link_patches_once` body minus its own tx/retry — driver owns both). `refresh` = no-op: `apply_link_patches` already refreshes rank inside its tx; calling it again double-writes. If the tx-scoped extraction changes that, refresh becomes `endpoint_rank::refresh` — check database.rs:1193 at implementation time and document the choice inline.

Gate read-through: `TaskScheduler`/batch code reads staged task state through the writer's map today. Add `pub fn get(&self, key) -> Option<Row>` (DashMap get, no await) to the driver; LinkSpec callers use it where they used `pending` reads.

- [ ] **Step 1: Write the failing test — port drain coalesce**

```rust
#[test]
fn link_coalesce_unions_groups_per_link() {
    // one link staged with RESULT then TRAFFIC: one patch, union groups,
    // merged snapshot carries both columns (mirror of link_writer drain test)
    let rows = vec![result_row(7, 100), traffic_row(7, 100)];
    let out = LinkSpec::coalesce(rows);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].patch.groups, LinkGroups::RESULT.union(LinkGroups::TRAFFIC));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-db link_coalesce_unions_groups`
Expected: FAIL with "LinkSpec not defined"

- [ ] **Step 3: Implement LinkSpec + switch all callers + delete link_writer.rs**

Port `merge_group`, group constants, `LinkPatch` unchanged. Switch every `link_writer` reference (grep `link_writer` after — must be zero hits outside history). Delete `crates/xray-tui/src/ops/link_writer.rs`, remove `mod link_writer` from `ops/mod.rs`. Port all tests from the deleted file: should_flush table (already in driver — extend with link-specific floor values), re-stage regression, no-commit guard, reload-before-sweep.

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-db && cargo test -p xray-tui ops:: && cargo nextest run --workspace`
Expected: PASS; `grep -r link_writer crates/ --include=*.rs` returns nothing

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(db): migrate LinkWriter onto WriteBehind<LinkSpec>, delete old writer"
```

### Task 5: Contention verification + quality gate

**Files:**
- Modify: `crates/xray-tui/src/ops/ping/flow_cost.rs` (add after-row: driver-based fan-in vs old numbers in comments)
- Modify: `docs/database-manual-sql.md` (entries for any new raw SQL; tx-scoped extractions are NOT new SQL — note as moved, not added)
- Test: `flow_cost_report` fan-in row

**Interfaces:**
- Consumes: all four tasks' drivers
- Produces: measured before/after (commits, 5s timeouts, p99) + `just quality-gate code` green

- [ ] **Step 1: Run the fan-in bench**

Run: `cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost 2>&1 | tail -30`
Expected: fan-in row shows fewer commits than the pre-migration baseline in comments; zero `snapshot is stale` / `database is locked` at same throughput. Record numbers.

- [ ] **Step 2: Run the workspace suite**

Run: `cargo nextest run --workspace 2>&1 | tail -5`
Expected: all green (Summary line)

- [ ] **Step 3: Run the quality gate (code subset)**

Run: `just quality-gate code 2>&1 | tail -10`
Expected: fmt-check + clippy (`--workspace --all-targets --all-features -- -D warnings`) + nextest green

- [ ] **Step 4: Commit measurements**

```bash
git add crates/xray-tui/src/ops/ping/flow_cost.rs docs/database-manual-sql.md
git commit -m "docs: RowCache contention after-numbers and manual-sql entries"
```

## Self-Review

**1. Spec coverage:** §3.1 trait → Task 1 (with triples adjustment + tx-scoped signatures); §3.2 driver mechanics → Task 1 (all methods, 4 triggers, re-stage); §3.3 durability (loss window, bounded end retry, read-through rule) → Tasks 2-4 (final flushes, gate `get`); §3.4 refresh-in-tx → Tasks 1-4 (driver opens tx, spec fns take `&mut tx`); §4 migration order → Tasks 2,3,4 in order; §5 testing (unit, injection, guard, fan-in, suite) → Tasks 1-5; §6 risks (loss window, atomicity + probe, N maps, refresh cost) → Tasks 3 (atomicity design + probe), 5 (fan-in). Heed exclusion honored (no task touches it). No gaps.

**2. Placeholder scan:** No TBD/TODO/"appropriate handling"/"similar to Task N" — every step has file paths, signatures, commands, expected output. The Task 1 `write` closure sketch is explicitly marked placeholder-shape with the real implementation (db handle) given immediately below; acceptable since both forms are shown.

**3. Type consistency:** `CacheSpec::{Key: Eq+Hash+Clone+Copy, Row: Clone, Patch}` + `Coalesced<Row, Patch>` triples used uniformly Tasks 1-4. `WriteBehind::new(db, flush_rows, flush_interval) -> Arc<Self>`; `push/flush/flush_soon/run/spawn_flush_task/staged_len/flush_count/timer_flush_floor/max_staged_age/should_flush/get` names stable across tasks. `LinkGroups`/`LinkPatch` shapes unchanged in Task 4. `EndpointId` key for CountrySpec matches today's DashMap key.

**4. Review Focus:** All five lines have owning tests: linkless orphans → Task 3 probe; mid-flush push survival → Task 1 re-stage test (extend: stage-during-flush case — add to Task 1 Step 1 as second test); floor/deadline interaction → Task 1 truth table; staged invisibility → Task 4 gate `get` + Task 3 probe ordering; flush ordering vs probe → Task 3 (endpoints-first coordinated flush before probe).
