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

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use xray_tui_db::models::{EndpointId, ProfileStats, ProtocolId};
use xray_tui_db::{Database, LinkGroups, LinkPatch};

use crate::ops::scheduler::SchedulerDb;

/// One staged row identity: the link plus the column group it carries.
type StageKey = ((ProtocolId, EndpointId), LinkGroups);

/// Default flush trigger: rows.
pub const DEFAULT_FLUSH_ROWS: usize = 512;
/// Default flush trigger: time.
pub const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_millis(200);

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
    flush_rows: usize,
    flush_interval: Duration,
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
        Arc::new(Self {
            pending: DashMap::new(),
            flush_rows: flush_rows.max(1),
            flush_interval,
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

    /// Stage a row's current state for `groups`. Never touches the database and
    /// never awaits, so callers on the UI task stay responsive.
    ///
    /// `groups` is normalised to one entry per flag, so a stage of `ALL` and a
    /// later stage of `RESULT` do not shadow each other.
    pub fn stage(&self, link: &ProfileStats, groups: LinkGroups) {
        for flag in [LinkGroups::RESULT, LinkGroups::TASK, LinkGroups::TRAFFIC] {
            if groups.contains(flag) {
                self.pending
                    .insert(((link.protocol_id, link.endpoint_id), flag), link.clone());
            }
        }
        if self.pending.len() >= self.flush_rows {
            self.wake.notify_one();
        }
    }

    /// Read-through: the staged state when present, otherwise the persisted
    /// row.
    ///
    /// The scheduler gate calls this from inside its critical section, so a
    /// task transition that has been staged but not yet flushed is still
    /// visible — the guarantee that stops a second `schedule` from starting the
    /// same link twice.
    pub async fn read(
        &self,
        key: (ProtocolId, EndpointId),
    ) -> xray_tui_db::Result<Option<ProfileStats>> {
        let mut conn = self.db.connection().await?;
        let persisted = ProfileStats::filter_by_protocol_id_and_endpoint_id(key.0, key.1)
            .first()
            .exec(&mut conn)
            .await?;
        Ok(self.overlay_pending(key, persisted))
    }

    /// Overlay every staged group for `key` onto `base`.
    fn overlay_pending(
        &self,
        key: (ProtocolId, EndpointId),
        base: Option<ProfileStats>,
    ) -> Option<ProfileStats> {
        let mut merged = base;
        for flag in [LinkGroups::RESULT, LinkGroups::TASK, LinkGroups::TRAFFIC] {
            let Some(staged) = self.pending.get(&(key, flag)) else {
                continue;
            };
            let Some(row) = merged.as_mut() else {
                merged = Some(staged.clone());
                break;
            };
            if flag == LinkGroups::RESULT {
                row.latency.clone_from(&staged.latency);
                row.speed_bps = staged.speed_bps;
                row.error.clone_from(&staged.error);
            } else if flag == LinkGroups::TASK {
                row.task_id = staged.task_id;
                row.task_queue.clone_from(&staged.task_queue);
            } else {
                row.traffic = staged.traffic;
            }
        }
        merged
    }

    /// Take every staged entry out of the map, keyed by its own snapshot.
    ///
    /// The drain is a **remove**, never a snapshot copy: a `stage` that lands
    /// while the flush is writing inserts a fresh entry that the next window
    /// picks up. Removing the key again after the write would drop that state.
    fn drain(&self) -> Vec<LinkPatch> {
        let keys: Vec<StageKey> = self.pending.iter().map(|entry| *entry.key()).collect();
        let mut patches = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some((_, link)) = self.pending.remove(&key) {
                patches.push(LinkPatch {
                    link,
                    groups: key.1,
                });
            }
        }
        patches
    }

    /// Write everything staged so far, in `flush_rows`-sized transactions.
    ///
    /// Called by the flush task, at batch end, and on quit. Returns the number
    /// of rows written.
    pub async fn flush(&self) -> xray_tui_db::Result<usize> {
        let _guard = self.gate.lock().await;
        let patches = self.drain();
        if patches.is_empty() {
            return Ok(0);
        }
        let mut written = 0usize;
        for chunk in patches.chunks(self.flush_rows) {
            match self.db.apply_link_patches(chunk).await {
                Ok(n) => {
                    written += n;
                    self.flushes.fetch_add(1, Ordering::Relaxed);
                }
                Err(err) => {
                    // Merge the failed chunk back; a newer staged entry wins.
                    for patch in chunk {
                        self.pending
                            .entry((
                                (patch.link.protocol_id, patch.link.endpoint_id),
                                patch.groups,
                            ))
                            .or_insert_with(|| patch.link.clone());
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
        self.pending.len()
    }

    /// Transactions performed so far.
    #[must_use]
    pub fn flush_count(&self) -> u64 {
        self.flushes.load(Ordering::Relaxed)
    }

    /// Run the flush loop until the handle is dropped/aborted.
    pub async fn run(self: Arc<Self>) {
        loop {
            tokio::select! {
                () = self.wake.notified() => {}
                () = tokio::time::sleep(self.flush_interval) => {}
            }
            if self.staged_len() == 0 {
                continue;
            }
            if let Err(err) = self.flush().await {
                tracing::warn!(target: "tui::ops::link_writer", "flush failed: {err}");
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

/// The scheduler gate's persistence seam while a batch is running: reads see
/// staged transitions (read-through) and writes are staged, never committed on
/// the caller's task.
impl SchedulerDb for LinkWriter {
    async fn read_link(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
    ) -> xray_tui_db::Result<Option<ProfileStats>> {
        self.read((protocol_id, endpoint_id)).await
    }

    fn write_task_state(
        &self,
        link: &ProfileStats,
        task_id: Option<u16>,
        queue: &[u16],
    ) -> impl std::future::Future<Output = xray_tui_db::Result<()>> + Send {
        let mut row = link.clone();
        row.task_id = task_id;
        row.task_queue = queue.to_vec();
        self.stage(&row, LinkGroups::TASK);
        std::future::ready(Ok(()))
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
            parent_id: None,
            last_source: None,
            manual_protocol_override: None,
            resolved_as: Vec::new(),
            resolved_at: None,
            created_at: jiff::Timestamp::from_second(1).expect("ts"),
            links: toasty::Deferred::default(),
            group_links: toasty::Deferred::default(),
        })
        .await
        .expect("endpoint");
        db.upsert_protocol(&Protocol {
            id: ProtocolId::new(101),
            sig: 101,
            cred_hash: 0,
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
            }))),
            created_at: jiff::Timestamp::from_second(1).expect("ts"),
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
            last_seen_at: jiff::Timestamp::from_second(1).expect("ts"),
            task_id: None,
            task_queue: Vec::new(),
            latency: None,
            speed_bps: None,
            error: None,
            traffic: TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            },
            created_at: jiff::Timestamp::from_second(1).expect("ts"),
            updated_at: jiff::Timestamp::from_second(1).expect("ts"),
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

    #[tokio::test]
    async fn stage_is_read_through_before_any_flush() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        writer.stage(&with_latency(&base, 33), LinkGroups::RESULT);

        assert_eq!(writer.flush_count(), 0, "staging must not write");
        let seen = writer.read(key()).await.expect("read").expect("row");
        assert_eq!(seen.latency, Some(Latency::Fast { delay: 33 }));
        // The database still holds the old value.
        assert_eq!(persisted(&db).await.latency, None);
    }

    #[tokio::test]
    async fn flush_writes_every_staged_row_and_final_state_matches() {
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        let mut result = with_latency(&base, 44);
        result.speed_bps = Some(1_000_000);
        let mut task = base.clone();
        task.task_id = Some(9);
        task.task_queue = vec![9];

        writer.stage(&result, LinkGroups::RESULT);
        writer.stage(&task, LinkGroups::TASK);
        assert_eq!(writer.staged_len(), 2);

        assert_eq!(writer.flush().await.expect("flush"), 2);
        assert_eq!(writer.staged_len(), 0);
        assert_eq!(writer.flush_count(), 1, "one transaction per window");

        let row = persisted(&db).await;
        assert_eq!(row.latency, Some(Latency::Fast { delay: 44 }));
        assert_eq!(row.speed_bps, Some(1_000_000));
        assert_eq!(row.task_id, Some(9));
        assert_eq!(row.task_queue, vec![9]);
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
        // flush_rows = 1: three staged groups become three transactions.
        let writer = LinkWriter::new(Arc::clone(&db), 1, DEFAULT_FLUSH_INTERVAL);
        let base = persisted(&db).await;
        writer.stage(&with_latency(&base, 1), LinkGroups::ALL);
        assert_eq!(
            writer.staged_len(),
            3,
            "ALL normalises to one entry per group"
        );
        assert_eq!(writer.flush().await.expect("flush"), 3);
        assert_eq!(writer.flush_count(), 3, "one transaction per chunk");
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

    #[tokio::test]
    async fn a_staged_task_transition_is_visible_to_a_read_before_flush() {
        // The gate's read-through guarantee: schedule() must see a staged
        // transition, not the pre-flush database row.
        let (db, writer) = seeded().await;
        let base = persisted(&db).await;
        let mut task = base.clone();
        task.task_id = Some(5);
        task.task_queue = vec![5];
        writer.stage(&task, LinkGroups::TASK);

        let seen = writer.read(key()).await.expect("read").expect("row");
        assert_eq!(seen.task_id, Some(5));
        assert_eq!(seen.task_queue, vec![5]);
    }
}
