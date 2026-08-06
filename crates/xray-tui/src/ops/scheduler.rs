//! The per-(protocol, endpoint) task gate (design §6.2): at most one live
//! task per `ProfileStats` row, a FIFO `task_queue` of waiting task ids, an
//! orphan sweep that reconciles the in-memory registry against persisted
//! state, and DNS-failure deferral so endpoints with recent DNS failures are
//! skipped.
//!
//! This module is pure scheduling state — it decides *which* task id may run
//! and persists that decision through the [`SchedulerDb`] seam; it never
//! executes tests itself. The caller (the T19 batch pipeline) fires the
//! actual probe with the id it gets back.
//!
//! ## Fire-handshake (contract with T19)
//!
//! There is no `CoreEvent::TaskFired` variant, so the fire signal is the
//! returned / persisted `task_id`, never an event:
//!
//! - [`TaskScheduler::schedule`] returns [`ScheduleOutcome::Started(id)`] /
//!   [`ScheduleOutcome::Queued(id)`] — the caller fires `Started` immediately
//!   and leaves `Queued` in the persisted queue.
//! - [`TaskScheduler::complete`] has no return: it retires the current
//!   `task_id` and persists `task_id = queue.first, queue = rest` (whole-vec
//!   replace — Turso cannot pop scalars). The caller must re-read the link
//!   afterwards and dispatch the new `task_id`, or stop when it is `None`.
//! - Persistence failures are logged and the logical outcome still returned;
//!   the next `schedule`/`sweep_orphans` pass reconciles (an id persisted as
//!   `task_id` but absent from the registry is the orphan case).
//!
//! ## Concurrency
//!
//! Every gate transition ([`TaskScheduler::schedule`], `complete`,
//! `cancel_queued`, `sweep_orphans`) holds one internal async mutex for the
//! whole check-then-act (persist included) and re-reads the persisted link
//! inside the critical section. Concurrent callers on the same link therefore
//! observe each other's writes: at most one `Started` outcome per gate-open,
//! and a stale caller snapshot can never double-advance the gate. The mutex
//! adds no serialization beyond what SQLite's single-writer model already
//! imposes.

use std::sync::atomic::{AtomicU16, Ordering};

use dashmap::DashMap;
use jiff::Timestamp;
use tracing::{debug, error, warn};
use xray_tui_db::Database;
use xray_tui_db::models::{EndpointId, ProfileStats, ProtocolId, TaskKind};

/// Outcomes of [`TaskScheduler::schedule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleOutcome {
    /// The gate was open (or an orphan was replaced): `id` is persisted as
    /// the link's `task_id` — fire the probe now.
    Started(u16),
    /// A live task holds the gate: `id` is appended to the link's persisted
    /// queue — wait for the current task to complete.
    Queued(u16),
    /// The queue is at its limit (or queueing is disabled, `queue_limit ==
    /// 0`): nothing changed, the caller should skip this link this round.
    QueueFull,
    /// The endpoint's DNS failed within `dns_defer_secs`: nothing changed,
    /// re-schedule after the deferral window.
    DnsDeferred,
}

/// Persistence seam for the scheduler. Implemented by [`Database`] for real
/// storage; tests use an in-memory mock so the scheduling logic runs
/// hermetically. Methods are RPITIT (`impl Future + Send`) so the returned
/// futures can be spawned on tokio tasks by the caller (T19).
pub trait SchedulerDb {
    /// Load one `ProfileStats` row for a (protocol, endpoint) pair.
    fn read_link(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
    ) -> impl std::future::Future<Output = xray_tui_db::Result<Option<ProfileStats>>> + Send;

    /// Atomically replace a link's `task_id` + `task_queue` (OCC-guarded in
    /// the real backend).
    fn write_task_state(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
        task_id: Option<u16>,
        queue: &[u16],
    ) -> impl std::future::Future<Output = xray_tui_db::Result<()>> + Send;
}

impl SchedulerDb for Database {
    async fn read_link(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
    ) -> xray_tui_db::Result<Option<ProfileStats>> {
        let mut conn = self.connection().await?;
        Ok(
            ProfileStats::filter_by_protocol_id_and_endpoint_id(protocol_id, endpoint_id)
                .first()
                .exec(&mut conn)
                .await?,
        )
    }

    async fn write_task_state(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
        task_id: Option<u16>,
        queue: &[u16],
    ) -> xray_tui_db::Result<()> {
        self.update_scheduler_state(protocol_id, endpoint_id, task_id, queue)
            .await
    }
}

/// In-memory scheduler state: the live task registry plus per-endpoint DNS
/// failure timestamps. `0` is never a valid task id.
pub struct TaskScheduler {
    /// Live task registry: every id here is either the current `task_id` of
    /// some link or queued in some link's `task_queue`.
    tasks: DashMap<u16, TaskKind>,
    /// Next candidate id; wraps around, skipping `0` and live ids.
    next_id: AtomicU16,
    /// Max queued tasks per link; `0` disables queueing (busy -> skipped).
    queue_limit: u16,
    /// DNS-failure deferral window in seconds (`<= 0` disables deferral).
    dns_defer_secs: i64,
    /// Endpoints whose DNS failed recently, by last failure time.
    dns_failures: DashMap<EndpointId, Timestamp>,
    /// Serializes every gate transition ([`Self::schedule`], [`Self::complete`],
    /// [`Self::cancel_queued`], [`Self::sweep_orphans`]): the check-then-act
    /// re-reads the persisted link inside the critical section, so concurrent
    /// callers on the same link observe each other's writes — at most one
    /// `Started` per gate-open. Contention is bounded: SQLite is single-writer
    /// anyway, so this adds no serialization the DB would not impose.
    gate: tokio::sync::Mutex<()>,
}

impl TaskScheduler {
    /// Create a scheduler with the given queue limit and DNS deferral window.
    #[must_use]
    pub fn new(queue_limit: u16, dns_defer_secs: i64) -> Self {
        Self {
            tasks: DashMap::new(),
            next_id: AtomicU16::new(0),
            queue_limit,
            dns_defer_secs,
            dns_failures: DashMap::new(),
            gate: tokio::sync::Mutex::new(()),
        }
    }

    /// Decide whether a new task for `link` may run now, and persist the
    /// decision. See the module docs for the fire-handshake.
    ///
    /// The caller's `link` is treated as a snapshot: the decision re-reads
    /// the persisted row inside the gate, so a stale snapshot cannot produce
    /// two `Started` outcomes on the same link.
    pub async fn schedule(
        &self,
        link: &ProfileStats,
        kind: TaskKind,
        db: &impl SchedulerDb,
    ) -> ScheduleOutcome {
        // DNS deferral FIRST — nothing is touched. No gate needed: the
        // failure map is a DashMap and the check mutates nothing but expired
        // entries.
        if self.is_dns_deferred(link.endpoint_id, Timestamp::now()) {
            return ScheduleOutcome::DnsDeferred;
        }

        let _guard = self.gate.lock().await;
        let link = self.fresh(link, db).await;

        let Some(current) = link.task_id else {
            // Gate open: allocate, register, persist, fire.
            let id = self.alloc_id();
            self.tasks.insert(id, kind);
            self.persist(
                db,
                &link,
                Some(id),
                &link.task_queue,
                &format!("start task {id}"),
            )
            .await;
            return ScheduleOutcome::Started(id);
        };

        if self.tasks.contains_key(&current) {
            // A live task holds the gate: queue if there is room.
            if self.queue_limit == 0 || link.task_queue.len() >= usize::from(self.queue_limit) {
                warn!(
                    target: "tui::scheduler",
                    "Cannot schedule {kind:?} on xray-tui://{:x}: queue full",
                    link.protocol_id.get(),
                );
                return ScheduleOutcome::QueueFull;
            }
            let id = self.alloc_id();
            self.tasks.insert(id, kind);
            let mut queue = link.task_queue.clone();
            queue.push(id);
            self.persist(
                db,
                &link,
                Some(current),
                &queue,
                &format!("queue task {id}"),
            )
            .await;
            return ScheduleOutcome::Queued(id);
        }

        // Orphan: the persisted task_id points at an id this process does
        // not know (restart, or a persist failure). Replace it and wipe
        // queue ids that are not live here either — fire immediately.
        let id = self.alloc_id();
        self.tasks.insert(id, kind);
        let wiped: Vec<u16> = link
            .task_queue
            .iter()
            .copied()
            .filter(|queued| self.tasks.contains_key(queued))
            .collect();
        self.persist(
            db,
            &link,
            Some(id),
            &wiped,
            &format!("orphan-replace task {id}"),
        )
        .await;
        ScheduleOutcome::Started(id)
    }

    /// Retire the current task of `link`. Only a completion whose `kind`
    /// matches the live registry entry is honored (race guard); anything
    /// else is stale and leaves the gate untouched.
    ///
    /// FIFO pop: persists `task_id = first registered queue id, queue =
    /// rest`. Queue ids that are no longer registered (a cancel persist
    /// failure or a missed orphan wipe) are skipped and dropped from the
    /// rewritten queue; if nothing live remains, the gate is cleared. The
    /// caller must re-read the link and dispatch the new `task_id` (see
    /// module docs).
    pub async fn complete(&self, link: &ProfileStats, kind: TaskKind, db: &impl SchedulerDb) {
        let _guard = self.gate.lock().await;
        let fresh = self.fresh(link, db).await;
        // Stale completion: the gate advanced since the caller read the
        // link, so the completed task is no longer current — do not advance
        // the gate twice.
        if fresh.task_id != link.task_id {
            debug!(
                target: "tui::scheduler",
                "complete: stale snapshot on xray-tui://{:x} (task_id {:?} -> {:?}) — ignored",
                link.protocol_id.get(),
                link.task_id,
                fresh.task_id,
            );
            return;
        }
        let Some(current) = fresh.task_id else {
            debug!(
                target: "tui::scheduler",
                "complete: no task_id on xray-tui://{:x}",
                link.protocol_id.get(),
            );
            return;
        };
        if self.tasks.get(&current).map(|k| *k) != Some(kind) {
            debug!(
                target: "tui::scheduler",
                "complete: task {current} on xray-tui://{:x} is not {kind:?} — stale completion ignored",
                link.protocol_id.get(),
            );
            return;
        }
        self.tasks.remove(&current);

        // Pop FIFO, skipping ids that are no longer registered: advancing
        // past them also drops them from the rewritten queue. If nothing
        // live remains, the gate is cleared.
        let mut queue = fresh.task_queue.clone();
        let mut next = None;
        while !queue.is_empty() {
            let candidate = queue.remove(0);
            if self.tasks.contains_key(&candidate) {
                next = Some(candidate);
                break;
            }
        }
        self.persist(
            db,
            &fresh,
            next,
            &queue,
            &format!("complete task {current}"),
        )
        .await;
    }

    /// Sibling cancel: drop every queued id whose registry entry is `kind`
    /// (other kinds are preserved). Persists the filtered queue against the
    /// CURRENT gate state; skips the write when nothing matched.
    pub async fn cancel_queued(&self, link: &ProfileStats, kind: TaskKind, db: &impl SchedulerDb) {
        let _guard = self.gate.lock().await;
        let fresh = self.fresh(link, db).await;
        let mut kept = Vec::with_capacity(fresh.task_queue.len());
        let mut changed = false;
        for id in &fresh.task_queue {
            if self.tasks.get(id).map(|k| *k) == Some(kind) {
                self.tasks.remove(id);
                changed = true;
            } else {
                kept.push(*id);
            }
        }
        if !changed {
            return;
        }
        self.persist(
            db,
            &fresh,
            fresh.task_id,
            &kept,
            &format!("cancel queued {kind:?}"),
        )
        .await;
    }

    /// Record a DNS failure for `endpoint`, sweeping expired entries so the
    /// map stays bounded.
    pub fn mark_dns_failure(&self, endpoint: EndpointId) {
        let now = Timestamp::now();
        self.dns_failures.insert(endpoint, now);
        self.sweep_dns_failures(now);
    }

    /// Wipe queued ids that are not live in this process's registry (stale
    /// after a restart or a persist failure). Only the queue is touched;
    /// `task_id` reconciliation happens on the next `schedule` via the
    /// orphan branch. Operates on the current gate state; skips the write
    /// when nothing changed.
    pub async fn sweep_orphans(&self, link: &ProfileStats, db: &impl SchedulerDb) {
        let _guard = self.gate.lock().await;
        let fresh = self.fresh(link, db).await;
        let kept: Vec<u16> = fresh
            .task_queue
            .iter()
            .copied()
            .filter(|id| self.tasks.contains_key(id))
            .collect();
        if kept.len() == fresh.task_queue.len() {
            return;
        }
        self.persist(db, &fresh, fresh.task_id, &kept, "sweep orphaned queue ids")
            .await;
    }

    /// Re-read the persisted link inside the gate so the check-then-act runs
    /// on current state. Falls back to the caller's snapshot when the row is
    /// gone or the read fails (the persist that follows will surface real
    /// DB trouble).
    async fn fresh<'a>(
        &self,
        link: &'a ProfileStats,
        db: &impl SchedulerDb,
    ) -> std::borrow::Cow<'a, ProfileStats> {
        match db.read_link(link.protocol_id, link.endpoint_id).await {
            Ok(Some(fresh)) => std::borrow::Cow::Owned(fresh),
            Ok(None) => std::borrow::Cow::Borrowed(link),
            Err(e) => {
                error!(
                    target: "tui::scheduler",
                    "read_link for xray-tui://{:x}: {e}",
                    link.protocol_id.get(),
                );
                std::borrow::Cow::Borrowed(link)
            }
        }
    }

    /// Allocate a fresh task id: never `0`, never a live id, wrapping.
    fn alloc_id(&self) -> u16 {
        loop {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            if id != 0 && !self.tasks.contains_key(&id) {
                return id;
            }
        }
    }

    /// Persist a scheduler-state change; failures are logged and reconciled
    /// by the next pass (the registry is the source of truth in memory).
    async fn persist(
        &self,
        db: &impl SchedulerDb,
        link: &ProfileStats,
        task_id: Option<u16>,
        queue: &[u16],
        action: &str,
    ) {
        if let Err(e) = db
            .write_task_state(link.protocol_id, link.endpoint_id, task_id, queue)
            .await
        {
            error!(
                target: "tui::scheduler",
                "{action} on xray-tui://{:x}: persist failed: {e}",
                link.protocol_id.get(),
            );
        }
    }

    /// Whether `endpoint` is inside its DNS-deferral window at `now`. Expired
    /// entries are dropped lazily. `dns_defer_secs <= 0` never defers.
    fn is_dns_deferred(&self, endpoint: EndpointId, now: Timestamp) -> bool {
        if self.dns_defer_secs <= 0 {
            return false;
        }
        let Some(entry) = self.dns_failures.get(&endpoint) else {
            return false;
        };
        if now.as_second() - entry.as_second() < self.dns_defer_secs {
            return true;
        }
        drop(entry);
        self.dns_failures.remove(&endpoint);
        false
    }

    /// Drop all DNS-failure entries older than the deferral window.
    fn sweep_dns_failures(&self, now: Timestamp) {
        if self.dns_defer_secs <= 0 {
            self.dns_failures.clear();
            return;
        }
        self.dns_failures
            .retain(|_, ts| now.as_second() - ts.as_second() < self.dns_defer_secs);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use parking_lot::Mutex;
    use toasty::Deferred;
    use xray_tui_db::models::{
        ConfigType, EndpointId, ProfileStats, ProtocolId, TaskKind, TrafficStats,
    };
    use xray_tui_proto::proto_spec::CoreType;

    use super::{ScheduleOutcome, SchedulerDb, TaskScheduler};

    fn ts(secs: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(secs).expect("valid ts")
    }

    fn link(pid: i64, eid: i64, task_id: Option<u16>, queue: Vec<u16>) -> ProfileStats {
        ProfileStats {
            protocol_id: ProtocolId::new(pid),
            endpoint_id: EndpointId::new(eid),
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_used_at: None,
            last_seen_at: ts(0),
            task_id,
            task_queue: queue,
            latency: None,
            speed_bps: None,
            error: None,
            traffic: TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            },
            created_at: ts(0),
            updated_at: ts(0),
            version: 1,
            protocol: Deferred::default(),
            endpoint: Deferred::default(),
        }
    }

    /// In-memory [`SchedulerDb`]: stores the persisted `(task_id, queue)` per
    /// link so tests can assert exactly what the scheduler wrote.
    #[derive(Default)]
    struct MockDb {
        state: Mutex<HashMap<(i64, i64), (Option<u16>, Vec<u16>)>>,
    }

    impl MockDb {
        fn put(&self, l: &ProfileStats) {
            self.state.lock().insert(
                (l.protocol_id.get(), l.endpoint_id.get()),
                (l.task_id, l.task_queue.clone()),
            );
        }

        fn state_of(&self, l: &ProfileStats) -> (Option<u16>, Vec<u16>) {
            self.state
                .lock()
                .get(&(l.protocol_id.get(), l.endpoint_id.get()))
                .cloned()
                .expect("link present in mock db")
        }

        fn writes(&self) -> Vec<(i64, Option<u16>, Vec<u16>)> {
            self.state
                .lock()
                .iter()
                .map(|((pid, _), (task_id, queue))| (*pid, *task_id, queue.clone()))
                .collect()
        }
    }

    impl SchedulerDb for MockDb {
        async fn read_link(
            &self,
            protocol_id: ProtocolId,
            endpoint_id: EndpointId,
        ) -> xray_tui_db::Result<Option<ProfileStats>> {
            let state = self.state.lock();
            Ok(state
                .get(&(protocol_id.get(), endpoint_id.get()))
                .map(|(task_id, queue)| {
                    link(
                        protocol_id.get(),
                        endpoint_id.get(),
                        *task_id,
                        queue.clone(),
                    )
                }))
        }

        async fn write_task_state(
            &self,
            protocol_id: ProtocolId,
            endpoint_id: EndpointId,
            task_id: Option<u16>,
            queue: &[u16],
        ) -> xray_tui_db::Result<()> {
            self.state.lock().insert(
                (protocol_id.get(), endpoint_id.get()),
                (task_id, queue.to_vec()),
            );
            Ok(())
        }
    }

    fn sched() -> TaskScheduler {
        TaskScheduler::new(3, 5)
    }

    // ── gate ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn gate_none_starts_live_queue() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, None, Vec::new());
        db.put(&l);

        // None -> Started, id persisted as task_id.
        let out = s.schedule(&l, TaskKind::FastPing, &db).await;
        let id = match out {
            ScheduleOutcome::Started(id) => id,
            other => panic!("expected Started, got {other:?}"),
        };
        assert_ne!(id, 0);
        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(id));
        assert!(queue.is_empty());

        // Live -> Queued even when the caller passes the ORIGINAL stale
        // snapshot (task_id == None): schedule re-reads the persisted link
        // inside the gate, so it sees the winner's id and queues instead of
        // starting a second probe.
        let out = s.schedule(&l, TaskKind::RealPing, &db).await;
        let qid = match out {
            ScheduleOutcome::Queued(id) => id,
            other => panic!("expected Queued, got {other:?}"),
        };
        assert_ne!(qid, id);
        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(id), "live task_id untouched");
        assert_eq!(queue, vec![qid]);
    }

    #[tokio::test]
    async fn gate_queue_limit_full() {
        // limit 3: 3 queued already -> QueueFull, nothing written.
        let s = TaskScheduler::new(3, 5);
        let db = MockDb::default();
        let live = 1u16;
        let l = link(1, 10, Some(live), vec![2, 3, 4]);
        db.put(&l);
        // Register the queue ids as live so the gate sees a genuinely live task.
        for q in [live, 2, 3, 4] {
            s.tasks.insert(q, TaskKind::FastPing);
        }

        let out = s.schedule(&l, TaskKind::UdpPing, &db).await;
        assert_eq!(out, ScheduleOutcome::QueueFull);
        assert_eq!(
            db.state_of(&l),
            (Some(live), vec![2, 3, 4]),
            "queue full must not touch persisted state"
        );

        // limit 0 disables queueing entirely.
        let s0 = TaskScheduler::new(0, 5);
        for q in [live, 2] {
            s0.tasks.insert(q, TaskKind::FastPing);
        }
        let out = s0
            .schedule(&link(1, 10, Some(live), vec![2]), TaskKind::UdpPing, &db)
            .await;
        assert_eq!(out, ScheduleOutcome::QueueFull);
    }

    #[tokio::test]
    async fn gate_orphan_replaces_and_wipes_queue() {
        let s = sched();
        let db = MockDb::default();
        // task_id=1 persisted but 1 is not in the registry; queue holds 2
        // (orphan too) and 3 (live elsewhere).
        let l = link(1, 10, Some(1), vec![2, 3]);
        db.put(&l);
        s.tasks.insert(3, TaskKind::FastPing);

        let out = s.schedule(&l, TaskKind::RealPing, &db).await;
        let id = match out {
            ScheduleOutcome::Started(id) => id,
            other => panic!("expected Started (orphan replace), got {other:?}"),
        };
        assert_eq!(s.tasks.get(&id).map(|k| *k), Some(TaskKind::RealPing));
        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(id), "orphan replaced with the fresh id");
        assert_eq!(queue, vec![3], "orphan queue ids wiped, live ones kept");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_schedule_same_link_single_winner() {
        let s = Arc::new(TaskScheduler::new(3, 5));
        let db = Arc::new(MockDb::default());
        db.put(&link(1, 10, None, Vec::new()));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let s = Arc::clone(&s);
            let db = Arc::clone(&db);
            handles.push(tokio::spawn(async move {
                // Caller flow: read the link, then schedule. The snapshot is
                // stale by the time the gate opens — schedule re-reads.
                let l =
                    SchedulerDb::read_link(db.as_ref(), ProtocolId::new(1), EndpointId::new(10))
                        .await
                        .unwrap()
                        .expect("link");
                s.schedule(&l, TaskKind::FastPing, db.as_ref()).await
            }));
        }
        let mut outcomes = Vec::new();
        for h in handles {
            outcomes.push(h.await.unwrap());
        }
        let started = outcomes
            .iter()
            .filter(|o| matches!(o, ScheduleOutcome::Started(_)))
            .count();
        assert_eq!(started, 1, "exactly one winner, got {outcomes:?}");
        let queued = outcomes
            .iter()
            .filter(|o| matches!(o, ScheduleOutcome::Queued(_)))
            .count();
        let full = outcomes
            .iter()
            .filter(|o| matches!(o, ScheduleOutcome::QueueFull))
            .count();
        assert_eq!(
            (queued, full),
            (3, 4),
            "queue_limit 3: 1 Started + 3 Queued + 4 QueueFull, got {outcomes:?}"
        );
        let (task_id, _) = db.state_of(&link(1, 10, None, Vec::new()));
        assert!(
            task_id.is_some(),
            "gate persisted after concurrent scheduling"
        );
    }

    // ── alloc_id ───────────────────────────────────────────────────────

    #[test]
    fn alloc_id_skips_zero_and_live_and_wraps() {
        let s = sched();
        // Prime near the wrap: 65533 and 65534 are live, so the first alloc
        // skips both and returns 65535; the next wraps to 0 (skipped) and
        // returns 1.
        s.next_id.store(65533, std::sync::atomic::Ordering::Relaxed);
        s.tasks.insert(65533, TaskKind::FastPing);
        s.tasks.insert(65534, TaskKind::RealPing);

        assert_eq!(s.alloc_id(), 65535);
        let id = s.alloc_id();
        assert_ne!(id, 0);
        assert_eq!(id, 1);
        assert!(!s.tasks.contains_key(&0));
    }

    // ── complete ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn complete_pops_fifo() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8, 9]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);
        s.tasks.insert(8, TaskKind::FastPing);
        s.tasks.insert(9, TaskKind::RealPing);

        s.complete(&l, TaskKind::FastPing, &db).await;

        assert!(
            !s.tasks.contains_key(&7),
            "completed task removed from the registry"
        );
        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(8), "FIFO: next queued id becomes task_id");
        assert_eq!(queue, vec![9], "FIFO: popped id removed, rest preserved");
    }

    #[tokio::test]
    async fn complete_empty_queue_clears_task_id() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), Vec::new());
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);

        s.complete(&l, TaskKind::FastPing, &db).await;

        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, None);
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn complete_kind_mismatch_is_stale_noop() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);

        // RealPing completes, but the live task is FastPing -> stale.
        s.complete(&l, TaskKind::RealPing, &db).await;

        assert!(s.tasks.contains_key(&7), "mismatched task NOT removed");
        assert_eq!(
            db.state_of(&l),
            (Some(7), vec![8]),
            "stale completion must not advance the gate"
        );
    }

    #[tokio::test]
    async fn complete_skips_unregistered_queue_ids() {
        let s = sched();
        let db = MockDb::default();
        // task_id=7 live; queue [8] unregistered (e.g. a cancel_queued
        // persist failure left it dangling), [9] live.
        let l = link(1, 10, Some(7), vec![8, 9]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);
        s.tasks.insert(9, TaskKind::FastPing);

        s.complete(&l, TaskKind::FastPing, &db).await;

        assert!(!s.tasks.contains_key(&7), "completed task removed");
        let (task_id, queue) = db.state_of(&l);
        assert_eq!(
            task_id,
            Some(9),
            "promotion skips the unregistered id and advances to the next live one"
        );
        assert_eq!(queue, Vec::<u16>::new(), "dead id dropped from the queue");
    }

    #[tokio::test]
    async fn complete_all_queued_dead_clears_gate() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);

        s.complete(&l, TaskKind::FastPing, &db).await;

        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, None, "no live id remains — gate cleared");
        assert!(queue.is_empty());
    }

    // ── cancel_queued ──────────────────────────────────────────────────

    #[tokio::test]
    async fn cancel_queued_filters_kind_only() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8, 9, 10]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);
        s.tasks.insert(8, TaskKind::RealPing); // sibling: cancel this
        s.tasks.insert(9, TaskKind::SpeedTest); // keep
        s.tasks.insert(10, TaskKind::RealPing); // sibling: cancel this

        s.cancel_queued(&l, TaskKind::RealPing, &db).await;

        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(7), "live task untouched by sibling cancel");
        assert_eq!(queue, vec![9], "only kind-matched ids removed");
        assert!(!s.tasks.contains_key(&8) && !s.tasks.contains_key(&10));
        assert!(s.tasks.contains_key(&9));
    }

    #[tokio::test]
    async fn cancel_queued_nothing_matched_skips_write() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8]);
        db.put(&l);
        s.tasks.insert(7, TaskKind::FastPing);
        s.tasks.insert(8, TaskKind::FastPing);

        s.cancel_queued(&l, TaskKind::UdpPing, &db).await;

        assert_eq!(db.state_of(&l), (Some(7), vec![8]));
        assert_eq!(db.writes().len(), 1, "no write when nothing matched");
    }

    // ── DNS deferral ───────────────────────────────────────────────────

    #[test]
    fn dns_window_deferred_then_expires() {
        let s = sched();
        let endpoint = EndpointId::new(10);

        assert!(!s.is_dns_deferred(endpoint, ts(100)));
        s.dns_failures.insert(endpoint, ts(100));
        // Inside the 5s window.
        assert!(s.is_dns_deferred(endpoint, ts(103)));
        // Exactly at the boundary the window has closed.
        assert!(!s.is_dns_deferred(endpoint, ts(105)));
        // Expired entry was dropped lazily.
        assert!(!s.dns_failures.contains_key(&endpoint));
    }

    #[tokio::test]
    async fn schedule_dns_deferred_within_window_and_proceeds_after() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, None, Vec::new());
        db.put(&l);

        s.mark_dns_failure(l.endpoint_id);
        let out = s.schedule(&l, TaskKind::FastPing, &db).await;
        assert_eq!(out, ScheduleOutcome::DnsDeferred);
        assert_eq!(db.state_of(&l), (None, Vec::new()), "deferred: no writes");

        // Age the entry past the window (constructed timestamp).
        let old = ts(jiff::Timestamp::now().as_second() - 60);
        s.dns_failures.insert(l.endpoint_id, old);
        let out = s.schedule(&l, TaskKind::FastPing, &db).await;
        assert!(matches!(out, ScheduleOutcome::Started(_)));
        let (task_id, _) = db.state_of(&l);
        assert!(task_id.is_some());
    }

    #[test]
    fn dns_defer_secs_zero_disables_deferral() {
        let s = TaskScheduler::new(3, 0);
        let endpoint = EndpointId::new(10);
        s.dns_failures.insert(endpoint, ts(100));
        assert!(!s.is_dns_deferred(endpoint, ts(100)));
    }

    // ── sweep_orphans ──────────────────────────────────────────────────

    #[tokio::test]
    async fn sweep_orphans_keeps_only_live_ids() {
        let s = sched();
        let db = MockDb::default();
        let l = link(1, 10, Some(7), vec![8, 9]);
        db.put(&l);
        s.tasks.insert(8, TaskKind::FastPing); // 9 is an orphan

        s.sweep_orphans(&l, &db).await;

        let (task_id, queue) = db.state_of(&l);
        assert_eq!(task_id, Some(7), "sweep never touches task_id");
        assert_eq!(queue, vec![8]);
    }

    // ── real-DB round trip ─────────────────────────────────────────────

    #[tokio::test]
    async fn write_task_state_round_trip_through_real_database() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let l = link(1, 10, None, Vec::new());
        db.upsert_link(&l).await.unwrap();

        // SchedulerDb::write_task_state -> update_scheduler_state.
        SchedulerDb::write_task_state(db.as_ref(), l.protocol_id, l.endpoint_id, Some(7), &[8, 9])
            .await
            .unwrap();

        let stored = SchedulerDb::read_link(db.as_ref(), l.protocol_id, l.endpoint_id)
            .await
            .unwrap()
            .expect("link exists");
        assert_eq!(stored.task_id, Some(7));
        assert_eq!(stored.task_queue, vec![8, 9]);

        // Full replace (Turso cannot pop scalars) — the queue is overwritten.
        SchedulerDb::write_task_state(db.as_ref(), l.protocol_id, l.endpoint_id, Some(9), &[])
            .await
            .unwrap();
        let stored = SchedulerDb::read_link(db.as_ref(), l.protocol_id, l.endpoint_id)
            .await
            .unwrap()
            .expect("link exists");
        assert_eq!(stored.task_id, Some(9));
        assert!(stored.task_queue.is_empty());
    }
}
