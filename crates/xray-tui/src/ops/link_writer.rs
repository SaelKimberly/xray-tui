//! Write-behind persistence for `profile_stats` mutations.
//!
//! The pending map is authoritative for the scheduler gate; the database is the
//! durable mirror. See
//! `docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md`.
//!
//! Why this exists: a Fast+Real batch over 30k links used to issue three
//! commits per link (result + `schedule` + `complete`), each awaited on the UI
//! task — measured 10.3 ms per typed upsert before the `synchronous=NORMAL`
//! fix, ~1.06 ms after, and ~95 s of UI-task blocking per batch. `stage` never
//! awaits; one flush task turns a window of staged rows into one transaction.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use xray_tui_db::models::{EndpointId, ProfileStats, ProtocolId};
use xray_tui_db::{Database, LinkGroups, LinkPatch};

/// One staged row identity: the link plus the column group it carries.
type StageKey = ((ProtocolId, EndpointId), LinkGroups);

/// Fold one staged column group onto `base`.
///
/// The groups are disjoint by construction, so the order they are applied in
/// never matters: RESULT owns latency/speed/error, PURGE the verdict, TRAFFIC
/// the four counters.
fn merge_group(base: &mut ProfileStats, flag: LinkGroups, staged: &ProfileStats) {
    if flag == LinkGroups::RESULT {
        base.latency.clone_from(&staged.latency);
        base.speed_bps = staged.speed_bps;
        base.error.clone_from(&staged.error);
    } else if flag == LinkGroups::PURGE {
        base.purge_reason = staged.purge_reason;
    } else {
        base.traffic = staged.traffic;
    }
}

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
/// delay is what this constant buys. The plan's `commits <= 20` target was
/// written before the floor/deadline interaction was measured; ~32 is what the
/// durability boundary actually permits at 4, and the target is amended to it.
pub const TIMER_FLOOR_DIVISOR: usize = 4;

/// The timer path's staleness deadline, as a multiple of `flush_interval`.
///
/// Deliberately LONGER than the time to reach the floor at the observed trickle
/// rate, so the floor decides each commit's size and the deadline is only the
/// net for a trickle too slow to ever reach it. A **tick count cannot express
/// this**: the wait to reach N ticks is rate-dependent, and an 8-tick backoff at
/// 12 rows/s forced a write at ~89 rows — under the floor — pre-empting the
/// coalescing it was meant to back up (measured 46 commits, 87.6 rows each).
/// At the 200 ms default this is a 15 s ceiling on how long a staged result may
/// sit unpersisted.
pub const MAX_STAGED_AGE_TICKS: u32 = 75;

/// Coalescing write-behind writer for `profile_stats`.
///
/// `stage` records the caller's snapshot for one column group; the flush task
/// writes a whole window in one transaction via
/// [`Database::apply_link_patches`]. The pending entries — not the database —
/// are the scheduler gate's source of truth while a batch is running.
pub struct LinkWriter {
    /// One snapshot per `(link, group)`: the result and task groups are staged
    /// independently so neither can overwrite the other.
    pending: DashMap<StageKey, ProfileStats>,
    /// Number of entries in `pending`, kept exactly in step with it because
    /// every insert and remove goes through [`Self::put`]/[`Self::take`].
    ///
    /// The flush trigger used to read `pending.len()`, which walks EVERY shard
    /// of the map (measured 1.06 µs per call — 95% of `stage`'s 1.12 µs, and
    /// `stage` runs twice per link in a batch plus once per manual ping).
    staged: AtomicU64,
    flush_rows: usize,
    flush_interval: Duration,
    /// Rows a **bare timer tick** must have staged before it writes anything.
    ///
    /// The size trigger (`wake`) is an explicit "there is a full window" signal
    /// and always writes; a timer tick used to write on a *single* staged row,
    /// which is what produced the 1,167 commits. See [`TIMER_FLOOR_DIVISOR`].
    timer_flush_floor: usize,
    /// Ceiling on how long a staged result may sit unpersisted, used only when
    /// a trickle never reaches the floor. See [`MAX_STAGED_AGE_TICKS`].
    max_staged_age: Duration,
    db: Arc<Database>,
    /// Serializes drain-and-write, so a flush triggered by the batch end and
    /// the flush task cannot write the same snapshot twice.
    gate: tokio::sync::Mutex<()>,
    /// Wakes the flush task when a stage crosses `flush_rows`.
    wake: tokio::sync::Notify,
    /// Transactions performed (diagnostics and tests).
    flushes: AtomicU64,
}

impl LinkWriter {
    /// Build a writer with the given flush policy.
    #[must_use]
    pub fn new(db: Arc<Database>, flush_rows: usize, flush_interval: Duration) -> Arc<Self> {
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
        })
    }

    /// Build a writer with the default policy.
    #[must_use]
    pub fn with_defaults(db: Arc<Database>) -> Arc<Self> {
        Self::new(db, DEFAULT_FLUSH_ROWS, DEFAULT_FLUSH_INTERVAL)
    }

    /// Insert one staged entry, keeping [`Self::staged`] in step with the map.
    fn put(&self, key: StageKey, row: ProfileStats) {
        if self.pending.insert(key, row).is_none() {
            self.staged.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Remove one staged entry, keeping [`Self::staged`] in step with the map.
    fn take(&self, key: &StageKey) -> Option<ProfileStats> {
        let removed = self.pending.remove(key).map(|(_, row)| row);
        if removed.is_some() {
            self.staged.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Stage a row's current state for `groups`. Never touches the database and
    /// never awaits, so callers on the UI task stay responsive.
    ///
    /// `groups` is normalised to one entry per flag, so a stage of `ALL` and a
    /// later stage of `RESULT` do not shadow each other.
    pub fn stage(&self, link: &ProfileStats, groups: LinkGroups) {
        for flag in [LinkGroups::RESULT, LinkGroups::PURGE, LinkGroups::TRAFFIC] {
            if groups.contains(flag) {
                self.put(((link.protocol_id, link.endpoint_id), flag), link.clone());
            }
        }
        if self.staged_len() >= self.flush_rows {
            self.wake.notify_one();
        }
    }

    /// Take every staged entry out of the map, COALESCED to one patch per link.
    ///
    /// The map holds one entry per `(link, group)`, so a link touched by a
    /// result, its traffic poll and the gate's transition would otherwise be
    /// written three times. One patch per link with the union of its groups
    /// writes it once, and the group overlay is the same one the gate reads.
    ///
    /// The drain is a **remove**, never a snapshot copy: a `stage` that lands
    /// while the flush is writing inserts a fresh entry that the next window
    /// picks up. Removing the key again after the write would drop that state.
    fn drain(&self) -> Vec<LinkPatch> {
        let keys: Vec<StageKey> = self.pending.iter().map(|entry| *entry.key()).collect();
        let mut order: Vec<(ProtocolId, EndpointId)> = Vec::new();
        let mut groups: HashMap<(ProtocolId, EndpointId), LinkGroups> = HashMap::new();
        for (link_key, flag) in keys {
            match groups.entry(link_key) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    *e.get_mut() = e.get().union(flag);
                }
                std::collections::hash_map::Entry::Vacant(slot) => {
                    order.push(link_key);
                    slot.insert(flag);
                }
            }
        }

        let mut patches = Vec::with_capacity(order.len());
        for link_key in order {
            let mut merged: Option<ProfileStats> = None;
            // EVERY flag: a group this loop skips is never removed from
            // `pending`, so its entry leaks and re-adds its bit to the next
            // drain's union — a PURGE-only entry would then write a verdict
            // from whatever snapshot the patch happened to carry.
            for flag in [LinkGroups::RESULT, LinkGroups::PURGE, LinkGroups::TRAFFIC] {
                let Some(staged) = self.take(&(link_key, flag)) else {
                    continue;
                };
                match merged.as_mut() {
                    Some(row) => merge_group(row, flag, &staged),
                    None => merged = Some(staged),
                }
            }
            if let Some(link) = merged {
                patches.push(LinkPatch {
                    link,
                    groups: groups.get(&link_key).copied().unwrap_or(LinkGroups::ALL),
                });
            }
        }
        patches
    }

    /// Write everything staged so far, in `flush_rows`-sized transactions.
    ///
    /// Called by the flush task, at batch end, and on quit. Returns the number
    /// of rows written.
    ///
    /// A failed transaction re-stages that window **and every window after it**
    /// (the drain already removed them from the pending map): the caller retries
    /// the whole remainder on the next tick. Re-staging only the failing window
    /// dropped the rest of the drained batch — silent loss for any window wider
    /// than one chunk.
    pub async fn flush(&self) -> xray_tui_db::Result<usize> {
        let _guard = self.gate.lock().await;
        let patches = self.drain();
        if patches.is_empty() {
            return Ok(0);
        }
        let mut written = 0usize;
        for (index, chunk) in patches.chunks(self.flush_rows).enumerate() {
            match self.db.apply_link_patches(chunk).await {
                Ok(n) => {
                    written += n;
                    self.flushes.fetch_add(1, Ordering::Relaxed);
                }
                Err(err) => {
                    for pending in patches.chunks(self.flush_rows).skip(index).flatten() {
                        // A newer staged entry (a result that landed while this
                        // flush was writing) wins over the snapshot it replaced.
                        //
                        // ONE KEY PER GROUP: `stage` and `drain` both address
                        // entries by a single flag, so re-staging under the
                        // union (`pending.groups`) inserts keys neither ever
                        // matches — the retry writes nothing and those results
                        // are lost with `staged-left` as the only hint.
                        for flag in [LinkGroups::RESULT, LinkGroups::PURGE, LinkGroups::TRAFFIC] {
                            if !pending.groups.contains(flag) {
                                continue;
                            }
                            match self
                                .pending
                                .entry(((pending.link.protocol_id, pending.link.endpoint_id), flag))
                            {
                                dashmap::mapref::entry::Entry::Occupied(mut slot) => {
                                    slot.insert(pending.link.clone());
                                }
                                dashmap::mapref::entry::Entry::Vacant(slot) => {
                                    slot.insert(pending.link.clone());
                                    self.staged.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    return Err(err);
                }
            }
        }
        Ok(written)
    }

    /// Number of staged entries (one per link and column group).
    #[must_use]
    pub fn staged_len(&self) -> usize {
        self.staged.load(Ordering::Relaxed) as usize
    }

    /// Transactions performed so far.
    #[must_use]
    pub fn flush_count(&self) -> u64 {
        self.flushes.load(Ordering::Relaxed)
    }

    /// Wake the flush task so it writes on its next loop iteration.
    ///
    /// This is how a **single, user-initiated** result gets prompt persistence
    /// without a commit on the UI task: a manual ping stages 1-3 column groups,
    /// far below [`Self::timer_flush_floor`], so the timer path would otherwise
    /// hold it until [`Self::max_staged_age`]. The loop's size-trigger arm
    /// always writes, so a notification is enough.
    ///
    /// `draining_results_performs_no_commit_on_the_ui_task` is the standing
    /// guard against committing while draining a result; this respects it.
    pub fn flush_soon(&self) {
        self.wake.notify_one();
    }

    /// Rows a bare timer tick needs staged before it writes.
    #[must_use]
    pub const fn timer_flush_floor(&self) -> usize {
        self.timer_flush_floor
    }

    /// Ceiling on how long a staged result may sit unpersisted.
    #[must_use]
    pub const fn max_staged_age(&self) -> Duration {
        self.max_staged_age
    }

    /// The flush decision as a pure function — no `self`, no clock, no runtime,
    /// no database.
    ///
    /// Extracted so the RULE is testable deterministically. The loop itself is a
    /// real-time race against real DB transactions: under full-suite load it can
    /// miss a tick and observe one floor crossing where two were expected — a
    /// flaky integration test, not a floor bug. This function carries the
    /// contract; the integration test carries the plumbing.
    ///
    /// - `woken` — the size trigger or an explicit [`Self::flush_soon`]. Always
    ///   writes, at any depth: it is an explicit "there is a full window" signal.
    /// - `staged >= floor` — the timer path's coalescing rule.
    /// - `stale` — `max_staged_age` elapsed since the last write. The net for a
    ///   trickle too slow to ever reach the floor, so a result is never stranded.
    const fn should_flush(floor: usize, woken: bool, staged: usize, stale: bool) -> bool {
        woken || staged >= floor || stale
    }

    /// Run the flush loop until the handle is dropped/aborted.
    ///
    /// Two triggers, deliberately asymmetric:
    ///
    /// * **Size trigger** (`wake`, fired by [`Self::stage`] at `flush_rows`) —
    ///   always writes. It is an explicit "there is a full window" signal.
    /// * **Timer tick** — writes once `timer_flush_floor` rows are staged, or
    ///   once `max_staged_age` has elapsed since the last write. The floor
    ///   decides how much a transaction carries; the deadline is only the net
    ///   for a trickle too slow to ever reach the floor.
    ///
    /// The old loop wrote on a single staged row from either trigger, which is
    /// what produced the 2026-10-01 run's 1,167 commits for 4,028 rows.
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
                    tracing::warn!(target: "tui::ops::link_writer", "flush failed: {err}");
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
        let writer = Arc::clone(self);
        tokio::spawn(writer.run())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_db::models::{
        ConfigType, Endpoint, HostType, Latency, ProfileStats, Protocol, ProtocolId, Security,
        TrafficStats, Transport,
    };
    use xray_tui_proto::proto_spec::CoreType;
    use xray_tui_proto::proto_spec::common::TransportConfig;
    use xray_tui_proto::proto_spec::{
        ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType, VlessConfig,
    };

    async fn seeded() -> (Arc<Database>, Arc<LinkWriter>) {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        db.upsert_endpoint(&Endpoint {
            id: EndpointId::new(1),
            host: "h.example".to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::new(),
            last_source: None,
            manual_protocol_override: None,
            resolved_at: None,
            created_at: 1,
            links: toasty::Deferred::default(),
            group_links: toasty::Deferred::default(),
        })
        .await
        .expect("endpoint");
        db.upsert_protocol(&Protocol {
            id: ProtocolId::new(101),
            sig: 101,
            proto_kind: ProtocolKind::Vless,
            transport: Transport {
                r#type: TransportType::Tcp,
                data: toasty::Deferred::from(toasty::Json(TransportConfig::Tcp)),
            },
            security: Security {
                r#type: SecurityType::None,
                sni: None,
                fp: None,
                insecure: None,
                data: toasty::Deferred::from(toasty::Json(SecurityConfig::default())),
            },
            config: toasty::Deferred::from(toasty::Json(ProtocolConfig::Vless(VlessConfig {
                uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                uuid_origin: None,
                security: SecurityConfig::default(),
                transport: TransportConfig::Tcp,
                encryption: None,
                flow: None,
                path: None,
                splice: None,
                remarks: None,
                mux: None,
            }))),
            created_at: 1,
            links: toasty::Deferred::default(),
        })
        .await
        .expect("protocol");
        let link = ProfileStats {
            protocol_id: ProtocolId::new(101),
            endpoint_id: EndpointId::new(1),
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_used_at: None,
            last_seen_at: 1,
            latency: None,
            speed_bps: None,
            error: None,
            purge_reason: None,
            traffic: TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            },
            created_at: 1,
            updated_at: 1,
            version: 0,
            protocol: toasty::Deferred::default(),
            endpoint: toasty::Deferred::default(),
        };
        db.upsert_link(&link).await.expect("link");
        let writer = LinkWriter::with_defaults(Arc::clone(&db));
        (db, writer)
    }

    fn key() -> (ProtocolId, EndpointId) {
        (ProtocolId::new(101), EndpointId::new(1))
    }

    async fn persisted(db: &Database) -> ProfileStats {
        persisted_row(db, key()).await
    }

    async fn persisted_row(db: &Database, key: (ProtocolId, EndpointId)) -> ProfileStats {
        #[allow(unused_mut, reason = "exec takes &mut dyn Executor")]
        let mut conn = db.connection().await.expect("conn");
        ProfileStats::filter_by_protocol_id_and_endpoint_id(key.0, key.1)
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row")
    }

    fn with_latency(base: &ProfileStats, delay: i32) -> ProfileStats {
        let mut row = base.clone();
        row.latency = Some(Latency::Fast { delay });
        row
    }

    /// The flush trigger reads a counter instead of `DashMap::len()` (which
    /// walks every shard: 1.06 µs of `stage`'s 1.12 µs). The counter is only
    /// sound while it tracks the map through stage, coalescing, drain, a failed
    /// window's re-stage and the new-entry path of a re-stage, so this pins it
    /// against the map itself at every step.
    #[tokio::test]
    async fn staged_counter_matches_the_map_through_every_transition() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;

        let entries = |writer: &LinkWriter| (writer.staged_len(), writer.pending.len());

        // A stage of two groups on one link: two entries.
        let mut row = with_latency(&base, 10);
        row.purge_reason = Some(xray_tui_db::models::PurgeReason::NotTls);
        writer.stage(&row, LinkGroups::RESULT.union(LinkGroups::PURGE));
        assert_eq!(entries(&writer), (2, 2), "two groups, two entries");

        // Re-staging the SAME (link, group) coalesces: the count must not drift.
        writer.stage(&row, LinkGroups::RESULT);
        writer.stage(&row, LinkGroups::RESULT);
        assert_eq!(entries(&writer), (2, 2), "a replace is not a new entry");

        // A second link, then ALL (three groups) on it.
        let mut sibling = with_latency(&base, 11);
        sibling.endpoint_id = EndpointId::new(2);
        writer.stage(&sibling, LinkGroups::ALL);
        assert_eq!(entries(&writer), (5, 5), "3 new groups on a new link");

        // Drain removes every entry it folds.
        let drained = writer.drain();
        assert_eq!(drained.len(), 2, "one patch per link");
        assert_eq!(entries(&writer), (0, 0), "drain empties both");

        // The failed-window re-stage is an insert into a now-empty map, so it
        // takes the Vacant path for both groups of the patch.
        let mut blocker = db.connection().await.expect("blocker connection");
        let mut lock = blocker.transaction().await.expect("lock transaction");
        toasty::sql::statement("UPDATE profile_stats SET version = version + 0")
            .exec(&mut lock)
            .await
            .expect("take the write lock");
        writer.stage(&row, LinkGroups::RESULT.union(LinkGroups::PURGE));
        assert!(writer.flush().await.is_err(), "held lock fails the flush");
        assert_eq!(entries(&writer), (2, 2), "the remainder is re-staged");

        // And a patch that is ALREADY staged keeps its entry (Occupied path).
        lock.rollback().await.expect("release the lock");
        drop(blocker);
        assert_eq!(writer.flush().await.expect("retry"), 1);
        assert_eq!(entries(&writer), (0, 0), "the retry drains it again");
    }

    #[tokio::test]
    async fn flush_writes_every_staged_row_and_final_state_matches() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        let mut result = with_latency(&base, 44);
        result.speed_bps = Some(1_000_000);
        let mut traffic = base.clone();
        traffic.traffic.total_up = 5;

        writer.stage(&result, LinkGroups::RESULT);
        writer.stage(&traffic, LinkGroups::TRAFFIC);
        assert_eq!(writer.staged_len(), 2, "one entry per (link, group)");

        // Both groups belong to the same link: they coalesce into ONE write,
        // and the merged row carries both the result and the counters.
        assert_eq!(writer.flush().await.expect("flush"), 1);
        assert_eq!(writer.staged_len(), 0);
        assert_eq!(writer.flush_count(), 1, "one transaction per window");

        let row = persisted(&db).await;
        assert_eq!(row.latency, Some(Latency::Fast { delay: 44 }));
        assert_eq!(row.speed_bps, Some(1_000_000));
        assert_eq!(row.traffic.total_up, 5);
    }

    #[tokio::test]
    async fn stage_never_awaits_a_write() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        for delay in 0..500 {
            writer.stage(&with_latency(&base, delay), LinkGroups::RESULT);
        }
        assert_eq!(writer.flush_count(), 0, "no transaction without a flush");
        assert_eq!(writer.staged_len(), 1, "same (link, group) coalesces");
    }

    #[tokio::test]
    async fn flush_chunks_at_flush_rows() {
        let (db, _) = seeded().await;
        // flush_rows = 1: three staged LINKS become three transactions (the
        // three column groups of ONE link would coalesce into a single write).
        let writer = LinkWriter::new(Arc::clone(&db), 1, DEFAULT_FLUSH_INTERVAL);
        let base = persisted(&db).await;
        for idx in 0..3 {
            let mut link = with_latency(&base, idx);
            link.endpoint_id = EndpointId::new(i64::from(idx) + 2);
            link.protocol_id = ProtocolId::new(i64::from(idx) + 202);
            writer.stage(&link, LinkGroups::ALL);
        }
        assert_eq!(
            writer.staged_len(),
            9,
            "ALL normalises to one entry per group (RESULT, PURGE, TRAFFIC), per link"
        );
        assert_eq!(
            writer.flush().await.expect("flush"),
            3,
            "one patch per link"
        );
        assert_eq!(writer.flush_count(), 3, "one transaction per chunk");
    }

    /// The re-staged remainder must be findable by the next drain.
    ///
    /// A real result stages `RESULT | PURGE`, and the error arm used to re-stage
    /// the coalesced patch under that UNION key — a key neither `stage` nor
    /// `drain` ever matches, so the retry wrote nothing and the results were
    /// lost (the pre-PURGE code had the same hole for `ALL`-staged patches).
    #[tokio::test]
    async fn a_failed_window_restages_a_multi_group_patch_so_the_retry_lands() {
        use xray_tui_db::models::PurgeReason;

        let (db, _) = seeded().await;
        let base = persisted(&db).await;
        let writer = LinkWriter::new(Arc::clone(&db), 1, DEFAULT_FLUSH_INTERVAL);

        let mut blocker = db.connection().await.expect("blocker connection");
        let mut lock = blocker.transaction().await.expect("lock transaction");
        toasty::sql::statement("UPDATE profile_stats SET version = version + 0")
            .exec(&mut lock)
            .await
            .expect("take the write lock");

        // One patch carrying TWO groups — the shape every real probe produces.
        let mut result = with_latency(&base, 33);
        result.purge_reason = Some(PurgeReason::RealityFallback);
        writer.stage(&result, LinkGroups::RESULT.union(LinkGroups::PURGE));
        assert!(
            writer.flush().await.is_err(),
            "the write lock fails the flush"
        );

        lock.rollback().await.expect("release the lock");
        drop(blocker);

        assert_eq!(
            writer.flush().await.expect("retry after contention"),
            1,
            "the re-staged patch is found and written by the retry"
        );
        assert_eq!(writer.staged_len(), 0);
        let stored = persisted(&db).await;
        assert_eq!(stored.latency, Some(Latency::Fast { delay: 33 }));
        assert_eq!(
            stored.purge_reason,
            Some(PurgeReason::RealityFallback),
            "both groups land, not just the first"
        );
    }

    /// A failed window must not take the windows after it down with it: the
    /// drain already removed them from the pending map, so only re-staging the
    /// failing window would silently drop the rest of the batch.
    #[tokio::test]
    async fn a_failed_flush_window_restages_the_whole_remainder() {
        fn stage_window(writer: &LinkWriter, base: &ProfileStats, bias: i64) {
            for idx in 0..3 {
                let mut link = with_latency(base, i32::try_from(idx).expect("small idx"));
                link.endpoint_id = EndpointId::new(bias + idx);
                writer.stage(&link, LinkGroups::RESULT);
            }
        }

        let (db, _) = seeded().await;
        let base = persisted(&db).await;
        // flush_rows = 1: three staged links become three windows.
        let writer = LinkWriter::new(Arc::clone(&db), 1, DEFAULT_FLUSH_INTERVAL);

        stage_window(&writer, &base, 10);
        assert_eq!(writer.flush().await.expect("clean flush"), 3);

        // Contention: another connection holds the write lock, so every write
        // the window attempts fails with "database is locked" — the failure an
        // overlapping subscription import produced in the 2026-09-15 batch.
        let mut blocker = db.connection().await.expect("blocker connection");
        let mut lock = blocker.transaction().await.expect("lock transaction");
        toasty::sql::statement("UPDATE profile_stats SET version = version + 0")
            .exec(&mut lock)
            .await
            .expect("take the write lock");

        stage_window(&writer, &base, 20);
        assert!(
            writer.flush().await.is_err(),
            "the held write lock must fail the flush"
        );
        assert_eq!(
            writer.staged_len(),
            3,
            "every window the failed flush did not write is re-staged"
        );

        lock.rollback().await.expect("release the lock");
        drop(blocker);
        assert_eq!(
            writer.flush().await.expect("retry after contention"),
            3,
            "the re-staged remainder lands on the next flush"
        );
        assert_eq!(writer.staged_len(), 0);
    }

    /// Every staged group must leave `pending` when the window is drained.
    ///
    /// A group the fold loop skipped was never removed, so its entry leaked and
    /// re-added its bit to the next drain's union — which is how a PURGE-only
    /// entry could write a verdict from a snapshot that never classified one.
    #[tokio::test]
    async fn a_purge_verdict_survives_the_drain_and_empties_the_window() {
        use xray_tui_db::models::PurgeReason;

        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        let mut result = with_latency(&base, 44);
        result.purge_reason = Some(PurgeReason::NotTls);
        writer.stage(&result, LinkGroups::RESULT.union(LinkGroups::PURGE));
        assert_eq!(writer.staged_len(), 2, "one entry per group");

        writer.flush().await.expect("flush");
        assert_eq!(
            writer.staged_len(),
            0,
            "the drain removes EVERY group's entry, not just the ones it folds"
        );

        let stored = persisted(&db).await;
        assert_eq!(stored.purge_reason, Some(PurgeReason::NotTls));
        assert_eq!(
            stored.latency,
            Some(Latency::Fast { delay: 44 }),
            "and the result half landed too"
        );
    }

    /// The drain removes: a stage landing while the drained snapshot is being
    /// written must survive for the next window (spec §4.3).
    #[tokio::test]
    async fn stage_during_an_in_flight_flush_survives() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;

        writer.stage(&with_latency(&base, 10), LinkGroups::RESULT);
        let drained = writer.drain();
        assert_eq!(drained.len(), 1);

        // Lands "during" the write of the drained snapshot.
        writer.stage(&with_latency(&base, 20), LinkGroups::RESULT);
        db.apply_link_patches(&drained)
            .await
            .expect("in-flight write");

        assert_eq!(writer.staged_len(), 1, "the newer stage survived the drain");
        writer.flush().await.expect("flush");
        assert_eq!(
            persisted(&db).await.latency,
            Some(Latency::Fast { delay: 20 }),
            "the newer value wins"
        );
    }

    /// Poll a condition instead of sleeping a fixed span.
    ///
    /// These tests drive a REAL background task against a REAL database, so a
    /// fixed sleep is a bet on scheduling: under the full-suite load the flush
    /// task can be starved for the whole window and observe zero commits, which
    /// is what `cargo nextest run --workspace` hit. Polling decouples the
    /// assertion from when the task happens to be scheduled.
    async fn wait_for(
        writer: &LinkWriter,
        what: &str,
        mut done: impl FnMut(&LinkWriter) -> bool,
        timeout: Duration,
    ) {
        let deadline = tokio::time::Instant::now() + timeout;
        while !done(writer) {
            assert!(
                tokio::time::Instant::now() < deadline,
                "timed out after {timeout:?} waiting for {what} (flushes={}, staged={})",
                writer.flush_count(),
                writer.staged_len(),
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// The FLOOR path. `flush_rows` drives both the size trigger and the floor,
    /// so the numbers are chosen so the floor sits strictly inside the row count
    /// while the size trigger is never reached: `FLUSH_ROWS = 128` -> floor 32,
    /// `ROWS = 64` -> exactly two floor-driven commits. The deadline
    /// (75 x 5 ms = 375 ms) is out of range for a 320 ms staging window, so it
    /// cannot be what fires.

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_trickle_coalesces_at_the_floor_not_every_tick() {
        const ROWS: usize = 64;
        const FLUSH_ROWS: usize = 128;
        let (db, _default) = seeded().await;
        let interval = Duration::from_millis(10);
        let writer = LinkWriter::new(Arc::clone(&db), FLUSH_ROWS, interval);
        let floor = writer.timer_flush_floor();
        assert_eq!(floor, FLUSH_ROWS / TIMER_FLOOR_DIVISOR);
        assert!(
            ROWS > floor && ROWS < FLUSH_ROWS,
            "the floor must sit strictly inside the row count, or this is not testing it",
        );
        let task = writer.spawn_flush_task();

        let base = persisted(&db).await;
        for i in 0..ROWS {
            // A DISTINCT endpoint_id per stage. `StageKey` is
            // `((protocol_id, endpoint_id), group)`, so staging the same link
            // repeatedly coalesces into ONE entry and the floor can never be
            // reached — an earlier version of this test asserted on a window
            // that never filled, and the single commit it saw was the DEADLINE,
            // not the floor.
            let mut link = with_latency(&base, i32::try_from(i).expect("small i"));
            link.endpoint_id = EndpointId::new(1_000 + i64::try_from(i).expect("small i"));
            writer.stage(&link, LinkGroups::RESULT);
            tokio::time::sleep(interval).await;
        }
        // `ROWS = 64` against `MAX_STAGED_AGE_TICKS = 75` puts the deadline out
        // of range during staging (64 < 75), so every commit here is
        // floor-driven — and that ratio is interval-independent.
        wait_for(
            &writer,
            "the staged rows to drain",
            |w| w.staged_len() == 0,
            Duration::from_secs(20),
        )
        .await;
        // 64 rows / floor 32 is 2 commits, plus at most one deadline flush for
        // the remainder. A regression that ignores the floor and writes per tick
        // yields ~64, far outside this range.
        let commits = writer.flush_count();
        assert!(
            (1..=4).contains(&commits),
            "expected 1-4 floor-driven commits for {ROWS} rows at floor {floor}, got {commits}",
        );
        task.abort();
    }

    /// The DEADLINE path, which the floor case deliberately stays out of: a
    /// trickle that never reaches the floor must still be written, so a result is
    /// never stranded. `ROWS = 8` against a floor of 32, waited past
    /// `max_staged_age`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_trickle_below_the_floor_is_still_written_by_the_deadline() {
        const ROWS: usize = 8;
        const FLUSH_ROWS: usize = 128;
        let (db, _default) = seeded().await;
        let interval = Duration::from_millis(10);
        let writer = LinkWriter::new(Arc::clone(&db), FLUSH_ROWS, interval);
        assert!(
            ROWS < writer.timer_flush_floor(),
            "this case needs rows UNDER the floor",
        );
        let task = writer.spawn_flush_task();

        let base = persisted(&db).await;
        for i in 0..ROWS {
            let mut link = with_latency(&base, i32::try_from(i).expect("small i"));
            link.endpoint_id = EndpointId::new(2_000 + i64::try_from(i).expect("small i"));
            writer.stage(&link, LinkGroups::RESULT);
            tokio::time::sleep(interval).await;
        }
        // Below the floor, the timer path must NOT write: only the staleness
        // deadline can release this.
        wait_for(
            &writer,
            "the deadline to release a below-floor trickle",
            |w| w.flush_count() >= 1,
            Duration::from_secs(30),
        )
        .await;
        assert_eq!(
            writer.staged_len(),
            0,
            "the deadline flush must drain, or a result is stranded",
        );
        task.abort();
    }
    /// The flush RULE, deterministically: no clock, no runtime, no database. The
    /// integration tests above exercise the plumbing; this pins the contract, so
    /// a regression cannot hide behind a scheduling artefact.
    #[test]
    fn the_floor_rule_is_exact() {
        let floor = 128usize;
        let s = LinkWriter::should_flush;

        // The size trigger and an explicit wake always write, at any depth.
        assert!(s(floor, true, 0, false), "wake writes at depth 0");
        assert!(s(floor, true, 1, false));

        // The timer path writes only at the floor or past the deadline.
        assert!(!s(floor, false, 0, false));
        assert!(!s(floor, false, floor - 1, false), "under the floor");
        assert!(s(floor, false, floor, false), "exactly at the floor");
        assert!(s(floor, false, floor + 1, false));

        // The deadline is the net: it writes below the floor, so a trickle too
        // slow to ever reach the floor is never stranded.
        assert!(s(floor, false, 1, true), "stale writes below the floor");
        assert!(s(floor, false, 0, true));

        // A `flush_rows` under the divisor floors to 1, so any staged row writes.
        // That is why the pre-existing `flush_rows = 1` tests cannot notice a
        // floor regression, and why this test is the one that pins the rule.
        assert!(s(1, false, 1, false), "floor 1 writes on any row");
    }

    /// The derived floor and deadline follow the named constants.
    #[tokio::test]
    async fn the_derived_policy_follows_the_named_constants() {
        let (db, _w) = seeded().await;
        let writer = LinkWriter::new(Arc::clone(&db), 512, Duration::from_millis(200));
        assert_eq!(writer.timer_flush_floor(), 512 / TIMER_FLOOR_DIVISOR);
        assert_eq!(
            writer.max_staged_age(),
            Duration::from_millis(200) * MAX_STAGED_AGE_TICKS
        );
        let small = LinkWriter::new(Arc::clone(&db), 1, DEFAULT_FLUSH_INTERVAL);
        assert_eq!(
            small.timer_flush_floor(),
            1,
            "under the divisor floors to 1"
        );
    }
}
