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
        for flag in [LinkGroups::RESULT, LinkGroups::PURGE, LinkGroups::TRAFFIC] {
            if groups.contains(flag) {
                self.pending
                    .insert(((link.protocol_id, link.endpoint_id), flag), link.clone());
            }
        }
        if self.pending.len() >= self.flush_rows {
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
                let Some((_, staged)) = self.pending.remove(&(link_key, flag)) else {
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
                            self.pending
                                .entry(((pending.link.protocol_id, pending.link.endpoint_id), flag))
                                .or_insert_with(|| pending.link.clone());
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
}
