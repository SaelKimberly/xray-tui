//! Generic write-behind drain driver shared by every write path.
//!
//! A [`WriteBehind<S>`] holds staged rows in a [`DashMap`], kept authoritative
//! while a batch runs, and turns them into chunked transactions on a background
//! task. It is the systematized form of `LinkWriter` (see
//! `docs/aegis/specs/2026-10-05-rowcache-write-behind-design.md`): the same
//! four-trigger policy and re-stage contract, with the per-table parts — the key,
//! the coalesce rule, the write and the rank refresh — supplied by a
//! [`CacheSpec`].
//!
//! Why this exists: concurrent single-row writers on one Turso/SQLite
//! connection produced `query failed [5000ms]` timeouts and `snapshot stale`
//! rollbacks during a 51,087-link import (2026-10-05). Batching the writes into
//! one transaction per window, off the caller's task, is the fix.
//!
//! The driver owns the transaction: [`WriteBehind::flush`] opens one
//! connection + transaction per chunk, calls [`CacheSpec::write_window`] then
//! [`CacheSpec::refresh`], and commits — all inside [`crate::retry_on_busy`].
//! Both spec hooks therefore take `&mut impl toasty::Executor` and never open a
//! transaction of their own.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;

/// Default flush trigger: rows.
pub const DEFAULT_FLUSH_ROWS: usize = 512;

/// Default flush trigger: time.
pub const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_millis(200);

/// The timer path's floor is `flush_rows / TIMER_FLOOR_DIVISOR`.
///
/// **The measured trade, 2026-10-01 (WAL, 74,014 endpoints).** The old loop
/// flushed on a single staged row, so a 4,028-row batch trickling over 327 s
/// committed **1,167 times — 3.45 rows per transaction**. This divisor is what
/// fixes it, and the cost is real:
///
/// | divisor | floor (of 512) | commits for 4,028 | staging latency @ 12 rows/s |
/// | --- | --- | --- | --- |
/// | 1 (old behaviour) | 1 | **1,167** | ~0.2 s |
/// | **4** | **128** | **~32** | **~10.7 s** |
/// | 2 | 256 | ~16 | ~21.3 s |
///
/// So durability moves from **200 ms to ~10.7 s**. Nothing is *lost* — batch
/// end, quit and reload flush explicitly — but results are delayed, and that
/// delay is what this constant buys.
pub const TIMER_FLOOR_DIVISOR: usize = 4;

/// The timer path's staleness deadline, as a multiple of `flush_interval`.
///
/// Deliberately LONGER than the time to reach the floor at the observed trickle
/// rate, so the floor decides each commit's size and the deadline is only the
/// net for a trickle too slow to ever reach it. A **tick count cannot express
/// this**: the wait to reach N ticks is rate-dependent, and an 8-tick backoff at
/// 12 rows/s forced a write at ~89 rows — under the floor — pre-empting the
/// coalescing it was meant to back up (measured 46 commits, 87.6 rows each).
/// At the 200 ms default this is a 15 s ceiling on how long a staged row may sit
/// unpersisted.
pub const MAX_STAGED_AGE_TICKS: u32 = 75;

/// One drained row: its map key, the source row it came from, and the coalesced
/// patch built from it.
///
/// The triple is what makes the failure path exact: [`WriteBehind::flush`]
/// removes rows from the pending map as it drains, so a failed chunk must be
/// able to re-stage its rows. Reconstructing rows from patches would be lossy
/// (a patch carries only what the write needs), so `coalesce` returns the row
/// alongside the patch instead.
#[derive(Debug, Clone)]
pub struct Coalesced<K, R, P> {
    /// The pending map key this row was staged under.
    pub key: K,
    /// The source row, retained for re-staging after a failed write.
    pub row: R,
    /// The coalesced patch the write path consumes.
    pub patch: P,
}

/// The per-table half of the driver: identity, coalescing, write and refresh.
///
/// `Send + Sync + 'static` (and the same on its associated types) because a
/// driver is shared behind an `Arc` and its flush task is spawned: every value
/// that crosses the staging boundary or rides a transaction future is moved
/// between threads.
pub trait CacheSpec: Send + Sync + 'static {
    /// Staged-row identity in the pending map.
    type Key: Eq + Hash + Clone + Copy + Send + Sync + 'static;
    /// The caller's snapshot of one row.
    type Row: Clone + Send + Sync + 'static;
    /// One coalesced write.
    type Patch: Send + Sync + 'static;

    /// The pending-map key of `row`.
    fn key_of(row: &Self::Row) -> Self::Key;

    /// Fold the drained rows into one patch per identity.
    ///
    /// The returned triples keep the source row so a failed flush can re-stage
    /// it; the driver calls this while holding the flush gate, so the result is
    /// the exact set it will write.
    fn coalesce(rows: Vec<Self::Row>) -> Vec<Coalesced<Self::Key, Self::Row, Self::Patch>>;

    /// Write `patches` on the driver's transaction. Returns rows written.
    ///
    /// The driver opened `tx` and owns the commit; this must not commit or open
    /// its own transaction.
    fn write_window<'a>(
        tx: &'a mut impl toasty::Executor,
        patches: &'a [Self::Patch],
    ) -> impl Future<Output = crate::Result<usize>> + Send + 'a;

    /// Refresh derived state (rank keys, etc.) for the flushed rows, on the same
    /// transaction, after [`Self::write_window`]. A spec with nothing derived
    /// returns `Ok(())`.
    fn refresh<'a>(
        tx: &'a mut impl toasty::Executor,
        patches: &'a [Self::Patch],
    ) -> impl Future<Output = crate::Result<()>> + Send + 'a;
}

/// A coalescing write-behind drain for one table.
///
/// `push` records the caller's snapshot; the flush task (or an end-of-op call)
/// writes a whole window in chunked transactions. The pending entries — not the
/// database — are the source of truth while a batch is running.
pub struct WriteBehind<S: CacheSpec> {
    /// One snapshot per key. Keyed by [`CacheSpec::key_of`], so a later push
    /// for the same identity replaces the earlier one.
    pending: DashMap<S::Key, S::Row>,
    /// Number of entries in `pending`, kept exactly in step with it because
    /// every insert and remove goes through `put`/`take`.
    ///
    /// A flush trigger that read `pending.len()` would walk EVERY shard of the
    /// map (measured 1.06 µs per call — 95% of `stage`'s 1.12 µs).
    staged: AtomicU64,
    /// Rows a `wake` trigger or a full window carries.
    flush_rows: usize,
    /// Rows a **bare timer tick** must have staged before it writes anything.
    timer_flush_floor: usize,
    /// The timer tick period.
    flush_interval: Duration,
    /// Ceiling on how long a staged row may sit unpersisted.
    max_staged_age: Duration,
    /// The database the flush task opens its transactions against.
    db: Arc<crate::Database>,
    /// Serializes drain-and-write, so an end-of-op flush and the flush task
    /// cannot write the same snapshot twice.
    gate: tokio::sync::Mutex<()>,
    /// Wakes the flush task when a push crosses `flush_rows`.
    wake: tokio::sync::Notify,
    /// Transactions performed (diagnostics and tests).
    flushes: AtomicU64,
    /// Ties the spec's types to the driver without storing a value.
    _spec: std::marker::PhantomData<S>,
}

impl<S: CacheSpec> WriteBehind<S> {
    /// Build a driver with the given flush policy.
    ///
    /// `flush_rows` is clamped to at least 1; the timer floor is a quarter of
    /// it (see [`TIMER_FLOOR_DIVISOR`]) and the staleness deadline is
    /// [`MAX_STAGED_AGE_TICKS`] intervals.
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

    /// Insert one staged entry, keeping `staged` in step with the map.
    fn put(&self, row: S::Row) {
        if self.pending.insert(S::key_of(&row), row).is_none() {
            self.staged.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Stage a row's current state. Never touches the database and never
    /// awaits, so callers on the UI task stay responsive.
    pub fn push(&self, row: S::Row) {
        self.put(row);
        if self.staged_len() >= self.flush_rows {
            self.wake.notify_one();
        }
    }

    /// Remove one staged entry, keeping `staged` in step with the map.
    fn take(&self, key: &S::Key) -> Option<S::Row> {
        let removed = self.pending.remove(key).map(|(_, row)| row);
        if removed.is_some() {
            self.staged.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Take every staged row out of the map, coalesced to one triple per
    /// identity.
    ///
    /// The drain is a **remove**, never a snapshot copy: a `push` that lands
    /// while the flush is writing inserts a fresh entry that the next window
    /// picks up. Removing the key a second time after the write would drop that
    /// state.
    fn drain(&self) -> Vec<Coalesced<S::Key, S::Row, S::Patch>> {
        let keys: Vec<S::Key> = self.pending.iter().map(|entry| *entry.key()).collect();
        let rows: Vec<S::Row> = keys.iter().filter_map(|key| self.take(key)).collect();
        S::coalesce(rows)
    }

    /// Write everything staged so far, in `flush_rows`-sized transactions.
    ///
    /// Called by the flush task, at batch end, and on quit. Returns the number
    /// of rows written.
    ///
    /// A failed transaction re-stages that window **and every window after it**
    /// (the drain already removed them from the pending map): the caller retries
    /// the whole remainder on the next tick. Re-staging only the failing window
    /// would drop the rest of the drained batch — silent loss for any window
    /// wider than one chunk.
    pub async fn flush(&self) -> crate::Result<usize> {
        let _guard = self.gate.lock().await;
        let drained = self.drain();
        if drained.is_empty() {
            return Ok(0);
        }
        // Split the triples into the patches the write path consumes and the
        // (key, row) pairs the failure path re-stages. The two vectors stay
        // index-aligned, so chunk `i` of `patches` corresponds to `rows`
        // starting at `i * flush_rows`.
        let mut patches = Vec::with_capacity(drained.len());
        let mut rows = Vec::with_capacity(drained.len());
        for entry in drained {
            patches.push(entry.patch);
            rows.push((entry.key, entry.row));
        }

        let mut written = 0usize;
        for (index, chunk) in patches.chunks(self.flush_rows).enumerate() {
            let db = Arc::clone(&self.db);
            let result = crate::retry_on_busy(
                move || {
                    let db = Arc::clone(&db);
                    async move {
                        let mut conn = db.connection().await?;
                        let mut tx = conn.transaction().await?;
                        let n = S::write_window(&mut tx, chunk).await?;
                        S::refresh(&mut tx, chunk).await?;
                        tx.commit().await?;
                        Ok::<usize, crate::DatabaseError>(n)
                    }
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
                    // Re-stage the failing window AND every later window. A row
                    // pushed while this flush was writing already holds a NEWER
                    // snapshot than the drained one, so re-stage leaves it
                    // alone — see [`Self::restage`].
                    self.restage(&rows[index * self.flush_rows..]);
                    return Err(err);
                }
            }
        }
        Ok(written)
    }

    /// Re-insert drained rows that a failed flush did not write.
    ///
    /// The drain REMOVED them from `pending`, so a row still absent is put back
    /// and counted. A key that was pushed again while the flush was writing is
    /// already [`Entry::Occupied`] with a **newer** snapshot than the drained
    /// one — overwriting it would roll the row back and could re-write a value
    /// the caller has since replaced, so it is left untouched.
    fn restage(&self, rows: &[(S::Key, S::Row)]) {
        for (key, row) in rows {
            match self.pending.entry(*key) {
                Entry::Occupied(_) => {}
                Entry::Vacant(slot) => {
                    slot.insert(row.clone());
                    self.staged.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Number of staged entries.
    #[must_use]
    pub fn staged_len(&self) -> usize {
        usize::try_from(self.staged.load(Ordering::Relaxed)).unwrap_or(usize::MAX)
    }

    /// Transactions performed so far.
    #[must_use]
    pub fn flush_count(&self) -> u64 {
        self.flushes.load(Ordering::Relaxed)
    }

    /// Wake the flush task so it writes on its next loop iteration.
    ///
    /// This is how a **single, user-initiated** row gets prompt persistence
    /// without a commit on the caller's task: it is far below
    /// [`Self::timer_flush_floor`], so the timer path would otherwise hold it
    /// until [`Self::max_staged_age`]. The loop's size-trigger arm always
    /// writes, so a notification is enough.
    pub fn flush_soon(&self) {
        self.wake.notify_one();
    }

    /// Rows a bare timer tick needs staged before it writes.
    #[must_use]
    pub const fn timer_flush_floor(&self) -> usize {
        self.timer_flush_floor
    }

    /// Ceiling on how long a staged row may sit unpersisted.
    #[must_use]
    pub const fn max_staged_age(&self) -> Duration {
        self.max_staged_age
    }

    /// The flush decision as a pure function — no `self`, no clock, no runtime,
    /// no database.
    ///
    /// Extracted so the RULE is testable deterministically. The loop itself is
    /// a real-time race against real DB transactions: under full-suite load it
    /// can miss a tick and observe one floor crossing where two were expected —
    /// a flaky integration test, not a floor bug. This function carries the
    /// contract; the loop carries the plumbing.
    ///
    /// - `woken` — the size trigger or an explicit [`Self::flush_soon`]. Always
    ///   writes, at any depth: it is an explicit "there is a full window" signal.
    /// - `staged >= floor` — the timer path's coalescing rule.
    /// - `stale` — `max_staged_age` elapsed since the last write. The net for a
    ///   trickle too slow to ever reach the floor, so a row is never stranded.
    #[must_use]
    pub const fn should_flush(floor: usize, woken: bool, staged: usize, stale: bool) -> bool {
        woken || staged >= floor || stale
    }

    /// Run the flush loop until the handle is dropped/aborted.
    ///
    /// Two triggers, deliberately asymmetric:
    ///
    /// * **Size trigger** (`wake`, fired by [`Self::push`] at `flush_rows`) —
    ///   always writes. It is an explicit "there is a full window" signal.
    /// * **Timer tick** — writes once `timer_flush_floor` rows are staged, or
    ///   once `max_staged_age` has elapsed since the last write. The floor
    ///   decides how much a transaction carries; the deadline is only the net
    ///   for a trickle too slow to ever reach the floor.
    pub async fn run(self: Arc<Self>) {
        let mut since_write = tokio::time::Instant::now();
        loop {
            let woken = tokio::select! {
                () = self.wake.notified() => true,
                () = tokio::time::sleep(self.flush_interval) => false,
            };
            let staged = self.staged_len();
            if staged == 0 {
                since_write = tokio::time::Instant::now();
                continue;
            }
            let stale = since_write.elapsed() >= self.max_staged_age;
            if !Self::should_flush(self.timer_flush_floor, woken, staged, stale) {
                continue;
            }
            let written = match self.flush().await {
                Ok(n) => n,
                Err(err) => {
                    tracing::warn!(target: "xray_tui_db", "write-behind flush failed: {err}");
                    0
                }
            };
            // A failed flush does NOT reset the clock, so a busy database is
            // retried on the deadline rather than spinning on every tick.
            if written > 0 {
                since_write = tokio::time::Instant::now();
            }
        }
    }

    /// Spawn [`Self::run`] as a background task.
    #[must_use]
    pub fn spawn_flush_task(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let driver = Arc::clone(self);
        tokio::spawn(driver.run())
    }
}

/// One staged country write: the endpoint, the address its country belongs to,
/// and the ISO code.
///
/// The staging identity is the ENDPOINT, not the address: the queue this
/// replaced keyed `endpoint_id → (ip, iso)` and an insert overwrote, so a
/// second resolution of the same endpoint replaced the first. That is
/// [`CountrySpec`]'s coalesce rule.
#[derive(Debug, Clone)]
pub struct CountryRow {
    /// The endpoint whose address carries the country.
    pub endpoint_id: crate::models_toasty::EndpointId,
    /// The resolved address the country belongs to.
    pub ip: std::net::IpAddr,
    /// ISO 3166-1 alpha-2 code from mmdb.
    pub iso: String,
}

/// The write side of a country row: 1:1 with [`CountryRow`], so the patch IS
/// the row.
///
/// Kept as its own type because the driver hands the write path a patch slice
/// and the re-stage path the row — the shapes must not be able to drift apart
/// by accident.
#[derive(Debug, Clone)]
pub struct CountryPatch {
    /// The endpoint whose address carries the country.
    pub endpoint_id: crate::models_toasty::EndpointId,
    /// The resolved address the country belongs to.
    pub ip: std::net::IpAddr,
    /// ISO 3166-1 alpha-2 code from mmdb.
    pub iso: String,
}

/// The write-behind spec for resolved-address countries (`endpoint_ip.country`).
///
/// The first migrated table, and the simplest: one upsert per row, no derived
/// state. `refresh` is a no-op because a country write touches `endpoint_ip`
/// only — rank keys are a function of an endpoint's links and protocols, not of
/// which country its address resolved to.
pub struct CountrySpec;

impl CacheSpec for CountrySpec {
    type Key = crate::models_toasty::EndpointId;
    type Row = CountryRow;
    type Patch = CountryPatch;

    fn key_of(row: &Self::Row) -> Self::Key {
        row.endpoint_id
    }

    fn coalesce(rows: Vec<Self::Row>) -> Vec<Coalesced<Self::Key, Self::Row, Self::Patch>> {
        // Last-writer-wins per endpoint, first-seen order preserved: the queue's
        // `DashMap::insert` overwrote an existing entry in place, and its key
        // order is not part of any contract.
        let mut latest: HashMap<Self::Key, CountryRow> = HashMap::new();
        let mut order: Vec<Self::Key> = Vec::new();
        for row in rows {
            let key = row.endpoint_id;
            if latest.insert(key, row).is_none() {
                order.push(key);
            }
        }
        order
            .into_iter()
            .filter_map(|key| {
                let row = latest.remove(&key)?;
                let patch = CountryPatch {
                    endpoint_id: row.endpoint_id,
                    ip: row.ip,
                    iso: row.iso.clone(),
                };
                Some(Coalesced { key, row, patch })
            })
            .collect()
    }

    async fn write_window<'a>(
        tx: &'a mut impl toasty::Executor,
        patches: &'a [Self::Patch],
    ) -> crate::Result<usize> {
        if patches.is_empty() {
            return Ok(0);
        }
        let rows: Vec<(crate::models_toasty::EndpointId, std::net::IpAddr, String)> = patches
            .iter()
            .map(|patch| (patch.endpoint_id, patch.ip, patch.iso.clone()))
            .collect();
        crate::database::set_endpoint_ip_countries_once(tx, &rows).await?;
        Ok(patches.len())
    }

    // The trait mandates a future; a country write derives nothing, so the
    // async-ness is the contract, not a bug.
    #[allow(
        clippy::unused_async_trait_impl,
        reason = "trait-mandated async signature"
    )]
    async fn refresh<'a>(
        _tx: &'a mut impl toasty::Executor,
        _patches: &'a [Self::Patch],
    ) -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use super::{CacheSpec, Coalesced, WriteBehind};
    use crate::models_toasty::EndpointId;
    use crate::{CountryRow, CountrySpec};

    /// A fake spec backed by a real in-memory table: `write_window` runs one
    /// upsert per patch on the driver's transaction, `refresh` is a no-op.
    ///
    /// A patch with a negative value is a deterministic, NON-busy failure (so
    /// `retry_on_busy` does not retry it), which is how the re-stage path is
    /// exercised without holding a write lock for the retry backoff.
    struct FakeSpec;

    #[derive(Debug, Clone)]
    struct FakeRow {
        key: i64,
        value: i64,
    }

    #[derive(Debug, Clone)]
    struct FakePatch {
        key: i64,
        value: i64,
    }

    impl CacheSpec for FakeSpec {
        type Key = i64;
        type Row = FakeRow;
        type Patch = FakePatch;

        fn key_of(row: &Self::Row) -> Self::Key {
            row.key
        }

        fn coalesce(rows: Vec<Self::Row>) -> Vec<Coalesced<Self::Key, Self::Row, Self::Patch>> {
            // Last-writer-wins per key, preserving first-seen order.
            let mut map = HashMap::new();
            let mut order = Vec::new();
            for row in rows {
                if !map.contains_key(&row.key) {
                    order.push(row.key);
                }
                map.insert(row.key, row);
            }
            order
                .into_iter()
                .map(|key| {
                    let row = map.remove(&key).expect("key was just inserted");
                    let patch = FakePatch {
                        key: row.key,
                        value: row.value,
                    };
                    Coalesced { key, row, patch }
                })
                .collect()
        }

        async fn write_window<'a>(
            tx: &'a mut impl toasty::Executor,
            patches: &'a [Self::Patch],
        ) -> crate::Result<usize> {
            if let Some(poison) = patches.iter().find(|patch| patch.value < 0) {
                return Err(crate::DatabaseError::Generic(format!(
                    "injected failure at key {}",
                    poison.key
                )));
            }
            for patch in patches {
                toasty::sql::query(format!(
                    "INSERT INTO fake_state(key, value) VALUES ({}, {}) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    patch.key, patch.value
                ))
                .exec(&mut *tx)
                .await?;
            }
            Ok(patches.len())
        }

        // The trait mandates a future; a spec with no derived state has nothing
        // to await, so the async-ness is the contract, not a bug.
        #[allow(
            clippy::unused_async_trait_impl,
            reason = "trait-mandated async signature"
        )]
        async fn refresh<'a>(
            _tx: &'a mut impl toasty::Executor,
            _patches: &'a [Self::Patch],
        ) -> crate::Result<()> {
            Ok(())
        }
    }

    fn row(key: i64, value: i64) -> FakeRow {
        FakeRow { key, value }
    }

    async fn fake_db() -> Arc<crate::Database> {
        let db = Arc::new(crate::Database::in_memory().await.expect("in-memory db"));
        let mut conn = db.connection().await.expect("conn");
        toasty::sql::statement(
            "CREATE TABLE IF NOT EXISTS fake_state \
             (key INTEGER PRIMARY KEY, value INTEGER NOT NULL)",
        )
        .exec(&mut conn)
        .await
        .expect("create fake_state");
        db
    }

    async fn stored(db: &crate::Database) -> HashMap<i64, i64> {
        let mut conn = db.connection().await.expect("conn");
        let rows = toasty::sql::query("SELECT key, value FROM fake_state")
            .exec(&mut conn)
            .await
            .expect("read fake_state");
        rows.iter()
            .filter_map(|value| match value {
                toasty::stmt::Value::Record(fields) => match (fields.first(), fields.get(1)) {
                    (
                        Some(toasty::stmt::Value::I64(key)),
                        Some(toasty::stmt::Value::I64(value)),
                    ) => Some((*key, *value)),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    #[test]
    fn should_flush_truth_table() {
        // woken always writes
        assert!(WriteBehind::<FakeSpec>::should_flush(128, true, 0, false));
        // timer floor
        assert!(WriteBehind::<FakeSpec>::should_flush(
            128, false, 128, false
        ));
        assert!(!WriteBehind::<FakeSpec>::should_flush(
            128, false, 127, false
        ));
        // staleness net
        assert!(WriteBehind::<FakeSpec>::should_flush(128, false, 1, true));
        // nothing staged, not woken, not stale
        assert!(!WriteBehind::<FakeSpec>::should_flush(128, false, 0, false));
    }

    /// The coalesce rule is last-writer-wins per key, and `flush` reports the
    /// rows written.
    #[tokio::test]
    async fn push_coalesces_and_flushes() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 512, Duration::from_millis(200));

        driver.push(row(1, 10));
        driver.push(row(1, 20)); // replaces the row above
        driver.push(row(2, 30));
        assert_eq!(driver.staged_len(), 2, "one entry per key");

        assert_eq!(driver.flush().await.expect("flush"), 2);
        assert_eq!(driver.flush_count(), 1, "one transaction per window");
        assert_eq!(driver.staged_len(), 0);

        let stored = stored(&db).await;
        assert_eq!(stored.get(&1), Some(&20), "last writer wins");
        assert_eq!(stored.get(&2), Some(&30));
    }

    /// Flushing an empty driver is a no-op that opens no transaction.
    #[tokio::test]
    async fn flush_with_nothing_staged_is_a_noop() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 512, Duration::from_millis(200));
        assert_eq!(driver.flush().await.expect("flush"), 0);
        assert_eq!(driver.flush_count(), 0);
    }

    /// A flush wider than `flush_rows` becomes one transaction per chunk.
    #[tokio::test]
    async fn flush_chunks_at_flush_rows() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 1, Duration::from_millis(200));
        for key in 1..=3 {
            driver.push(row(key, key * 10));
        }
        assert_eq!(driver.flush().await.expect("flush"), 3);
        assert_eq!(driver.flush_count(), 3, "one transaction per chunk");
    }

    /// A failed window re-stages that window AND every later one: the drain
    /// already removed them, so re-staging only the failing window would drop
    /// the rest of the batch.
    ///
    /// Windows are drained in `DashMap` order, so the failing window's index —
    /// and therefore how many windows were written before it — is not fixed.
    /// The invariant under test is order-independent: **nothing is lost** — every
    /// drained row is either persisted or re-staged — and the retry lands the
    /// whole remainder.
    #[tokio::test]
    async fn a_failed_window_restages_the_whole_remainder() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 1, Duration::from_millis(200));

        driver.push(row(1, -1)); // poison: its window fails
        driver.push(row(2, 2));
        driver.push(row(3, 3));
        assert!(driver.flush().await.is_err(), "the poison patch fails");

        let persisted = stored(&db).await;
        assert_eq!(
            persisted.len() + driver.staged_len(),
            3,
            "every row is either persisted or re-staged — none is lost"
        );
        assert!(
            driver.staged_len() >= 1,
            "the failing window is re-staged, not dropped"
        );

        // Replace the poison. The Occupied path keeps the count unchanged.
        driver.push(row(1, 1));
        let remainder = driver.staged_len();
        assert_eq!(
            driver
                .flush()
                .await
                .expect("retry after the poison is gone"),
            remainder,
            "the re-staged remainder lands on the next flush"
        );
        assert_eq!(driver.staged_len(), 0);

        let stored = stored(&db).await;
        assert_eq!(stored.get(&1), Some(&1));
        assert_eq!(stored.get(&2), Some(&2));
        assert_eq!(stored.get(&3), Some(&3));
    }

    /// The drain removes: a push landing while the drained snapshot is being
    /// written must survive for the next window.
    #[tokio::test]
    async fn stage_during_an_in_flight_flush_survives() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 512, Duration::from_millis(200));

        driver.push(row(1, 10));
        let drained = driver.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(driver.staged_len(), 0);

        // Lands "during" the write of the drained snapshot.
        driver.push(row(1, 20));
        assert_eq!(driver.staged_len(), 1, "the newer push survives the drain");

        let patches: Vec<FakePatch> = drained.into_iter().map(|entry| entry.patch).collect();
        let mut conn = db.connection().await.expect("conn");
        FakeSpec::write_window(&mut conn, &patches)
            .await
            .expect("in-flight write");

        assert_eq!(driver.flush().await.expect("flush"), 1);
        assert_eq!(stored(&db).await.get(&1), Some(&20), "the newer value wins");
    }

    /// A re-stage must not clobber a newer push that landed during the failed
    /// flush: the drained snapshot is stale, so the entry the caller has since
    /// staged wins.
    #[tokio::test]
    async fn restage_does_not_clobber_a_newer_push() {
        let db = fake_db().await;
        let driver = WriteBehind::<FakeSpec>::new(Arc::clone(&db), 512, Duration::from_millis(200));

        driver.push(row(1, 10));
        let drained = driver.drain();
        assert_eq!(driver.staged_len(), 0);

        // The newer value lands while the drained snapshot is (unsuccessfully)
        // being written.
        driver.push(row(1, 99));
        let rows: Vec<(i64, FakeRow)> = drained
            .into_iter()
            .map(|entry| (entry.key, entry.row))
            .collect();
        driver.restage(&rows);

        assert_eq!(
            driver.staged_len(),
            1,
            "the newer push stays the single pending entry"
        );
        assert_eq!(driver.flush().await.expect("flush"), 1);
        assert_eq!(
            stored(&db).await.get(&1),
            Some(&99),
            "the re-stage did not roll the row back to the stale drained value"
        );
    }

    fn country_row(id: i64, ip: &str, iso: &str) -> CountryRow {
        CountryRow {
            endpoint_id: EndpointId::new(id),
            ip: ip.parse().expect("ip"),
            iso: iso.to_owned(),
        }
    }

    /// The country buffer's identity is the ENDPOINT, and the queue's insert
    /// overwrote: two resolutions of one endpoint must collapse to the last,
    /// not write both.
    #[test]
    fn country_coalesce_last_writer_wins() {
        let rows = vec![
            country_row(1, "1.1.1.1", "US"),
            country_row(1, "1.1.1.1", "DE"),
            country_row(2, "2.2.2.2", "FR"),
        ];
        let patches = CountrySpec::coalesce(rows);
        assert_eq!(patches.len(), 2);
        let one = patches
            .iter()
            .find(|p| p.key == EndpointId::new(1))
            .expect("endpoint 1 survived");
        assert_eq!(one.patch.iso, "DE");
        assert_eq!(one.row.iso, "DE");
    }
}
