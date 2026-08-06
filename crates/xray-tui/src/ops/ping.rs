use std::collections::{BTreeMap, HashMap, HashSet, hash_map::Entry};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::{Notify, Semaphore, mpsc};
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{CorePool, SinglePingReq};
use xray_tui_db::Database;
use xray_tui_db::models::Protocol as DbProtocol;
use xray_tui_db::models::{Endpoint, EndpointId, EndpointRow, ProfileStats, ProtocolId, TaskKind};

use crate::AppState;
use crate::ops::scheduler::{ScheduleOutcome, SchedulerDb, TaskScheduler};
use crate::state::{link_is_failed, load_protocol_with_config};
use crate::try_send_or_warn;
use crate::types::CoreEvent;

/// Start TCP ping on the given profile. Returns immediately; result arrives via `CoreEvent`.
pub fn start_tcp_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Test already in progress for this profile",
        );
        return;
    }
    // Find the profile and extract address:port. Lookup is by REAL protocol
    // id so sub-row pings target the exact protocol (not the endpoint).
    let row = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
    {
        r
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for TCP ping");
        return;
    };
    let Some(link) = row
        .links
        .iter()
        .find(|l| l.protocol_id.get() == protocol_id)
    else {
        state.log_trace("error", "tui::ops::ping", "Protocol not found for TCP ping");
        return;
    };
    let Some(proto) = row.protocols.get(&link.protocol_id) else {
        state.log_trace(
            "error",
            "tui::ops::ping",
            "Protocol row not found for TCP ping",
        );
        return;
    };
    if row.endpoint.host.is_empty() {
        state.log_trace("error", "tui::ops::ping", "Profile has no address");
        return;
    }
    let addr = row.endpoint.host.clone();
    let port = if row.endpoint.port > 0 {
        row.endpoint.port
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile has invalid port");
        return;
    };

    let tx = if let Some(tx) = &state.core_event_tx {
        tx.clone()
    } else {
        state.log_trace(
            "error",
            "tui::ops::ping",
            "Core event channel not initialized",
        );
        return;
    };

    state.testing_details.insert(protocol_id, TestType::TcpPing);
    state.testing_profiles.insert(protocol_id);
    // The fast-ping adapters dispatch on the protocol kind.
    let config_type = proto.proto_kind.to_i32();
    let timeout_dur = *state.config.speed_test.tcp_timeout_secs;

    tokio::spawn(async move {
        let fmgr = xray_tui_core::FastPingManager::new(timeout_dur);
        let result = fmgr.ping(config_type, &addr, port).await;
        let (latency_ms, error) = match result {
            Ok(dur) => (Some(dur.as_millis() as u64), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::TcpPing,
                latency_ms,
                speed_bps: None,
                ip_info: None,
                error,
            },
            "tcp_ping_result",
        );
    });
}

/// Get the shared core pool, creating it lazily on first use.
///
/// Single real pings draw ports from this pool's single allocator, so a warm
/// pooled core and any future batch core can never collide on a port.
fn get_or_create_pool(state: &mut AppState) -> Arc<CorePool> {
    if let Some(p) = &state.core_pool {
        return p.clone();
    }
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir.join("bin");
    let bin_configs_dir = config_dir.join("binConfigs");
    let proxy_addr = state.config.inbound.listen.clone();
    let base_port = state.config.inbound.socks_port;
    let pool = Arc::new(CorePool::new(
        bin_dir,
        bin_configs_dir,
        proxy_addr,
        base_port,
    ));
    state.core_pool = Some(pool.clone());
    pool
}

/// Start real ping (HTTP through proxy) using a pooled warm core for
/// single-ping reuse.
///
/// The pool is created lazily on first use. Subsequent single pings reuse the
/// same core: sing-box via SIGHUP reload, xray-core via stop+restart. The
/// `Protocol` row is re-loaded WITH its deferred `config` included inside the
/// spawned task — the pool's `ConfigBuilder::build` refuses unloaded configs.
pub fn start_real_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }

    // Find profile row by REAL protocol id (sub-row pings target the exact
    // protocol; different credentials → different exits).
    let endpoint;
    let link;
    let protocol_id_typed;
    if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.links.iter().any(|l| l.protocol_id.get() == protocol_id))
    {
        let Some(l) = r.links.iter().find(|l| l.protocol_id.get() == protocol_id) else {
            state.log_trace(
                "error",
                "tui::ops::ping",
                "Protocol not found for real ping",
            );
            return;
        };
        endpoint = r.endpoint.clone();
        link = l.clone();
        protocol_id_typed = l.protocol_id;
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for real ping");
        return;
    }

    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state
        .testing_details
        .insert(protocol_id, TestType::RealPing);
    state.testing_profiles.insert(protocol_id);

    // Lazily create the core pool on first use
    let pool = get_or_create_pool(state);
    let db = state.db.clone();

    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            protocol_id,
            test_type: TestType::RealPing,
        });

        // Load the protocol row WITH its config included (the config builders
        // and `shadowsocks_method` refuse unloaded configs).
        let protocol = match load_protocol_with_config(&db, protocol_id_typed).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        protocol_id,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some("Protocol row not found for real ping".to_string()),
                    },
                    "real_ping_protocol_missing",
                );
                return;
            }
            Err(e) => {
                try_send_or_warn(
                    &tx,
                    CoreEvent::SpeedTestResult {
                        protocol_id,
                        test_type: TestType::RealPing,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(format!("Failed to load protocol: {e}")),
                    },
                    "real_ping_protocol_load",
                );
                return;
            }
        };

        let result = pool
            .ping(
                &endpoint,
                &link,
                &protocol,
                SinglePingReq {
                    ping_url: &ping_url,
                    ip_api_url: &ip_api_url,
                    timeout,
                    retries,
                },
            )
            .await;

        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::RealPing,
                latency_ms: result.latency_ms,
                speed_bps: None,
                ip_info: result.ip_info,
                error: result.error,
            },
            "real_ping_result",
        );
    });
}

/// Start speed test (download through proxy) on the given profile.
pub fn start_speed_test(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }
    if state.connected_core.is_none() {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Core not connected — proxy required for speed test",
        );
        return;
    }
    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state
        .testing_details
        .insert(protocol_id, TestType::SpeedTest);
    state.testing_profiles.insert(protocol_id);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;
    let test_url = "http://cachefly.cachefly.net/1mb.test".to_string();
    let min_dur = std::time::Duration::from_secs(3);
    let max_dur = std::time::Duration::from_secs(10);

    tokio::spawn(async move {
        let result = xray_tui_core::speed_test::speed_test(
            &proxy_addr,
            proxy_port,
            &test_url,
            min_dur,
            max_dur,
        )
        .await;
        let (speed_bps, error) = match result {
            Ok(bps) => (Some(bps), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::SpeedTest,
                latency_ms: None,
                speed_bps,
                ip_info: None,
                error,
            },
            "speed_test_result",
        );
    });
}

/// Start UDP test through the connected proxy.
pub fn start_udp_test(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }
    if state.connected_core.is_none() {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Core not connected — proxy required for UDP test",
        );
        return;
    }
    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    state.testing_details.insert(protocol_id, TestType::UdpTest);
    state.testing_profiles.insert(protocol_id);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;

    tokio::spawn(async move {
        let result = xray_tui_core::speed_test::udp_test(
            &proxy_addr,
            proxy_port,
            std::time::Duration::from_secs(5),
        )
        .await;
        let (latency_ms, error) = match result {
            Ok(dur) => (Some(dur.as_millis() as u64), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                protocol_id,
                test_type: TestType::UdpTest,
                latency_ms,
                speed_bps: None,
                ip_info: None,
                error,
            },
            "udp_test_result",
        );
    });
}

/// Signal all running tests to stop.
pub fn stop_speed_test(state: &mut AppState) {
    state.speed_test_stop.store(true, Ordering::Relaxed);
}

/// Remove endpoints whose links carry a persisted failure marker (the old
/// `extension.delay == Some(-1)` sweep, now driven by `ProfileStats.error`).
pub async fn remove_failed_servers(state: &mut AppState) {
    let to_remove: Vec<i64> = state
        .endpoints
        .iter()
        .filter(|r| r.links.iter().any(link_is_failed))
        .map(|r| r.endpoint.id.get())
        .collect();
    let count = to_remove.len();
    for id in to_remove {
        crate::ops::profiles::delete_profile(state, id).await;
    }
    state.multi_select.clear();
    state.log_trace(
        "info",
        "tui::ops::ping",
        &format!("Removed {count} failed server(s)"),
    );
}

// ══════════════════════════════════════════════════════════════════════════
// Batch pipeline (T19) — rebuilt on the TaskScheduler
// ══════════════════════════════════════════════════════════════════════════
//
// Phase 1 schedules one `FastPing` task per link; the fast probes are
// deduplicated by (address, port) — one TCP ping per unique address, the
// result fanned out to every link sharing it (the old `FastCache` semantics).
// `DnsDeferred` links are re-scheduled after the deferral window in a spawned
// task; `QueueFull` links are skipped (the scheduler logs the warning).
// Phase 2 (after every phase-1 task settles) schedules one `RealPing` task
// per link, dispatched through the kept real-ping path (`CorePool::ping` with
// the protocol row reloaded WITH config). With `dedup_endpoints` (the
// `real_ping_test_all_protocols` negation), the first successful real ping on
// an endpoint retires the remaining links' real tasks via
// `scheduler.cancel_queued` + `complete` — cancelled tasks never write error
// markers. The scheduler is the single gate authority: probes run only for
// ids `schedule`/`complete` hand out, and every completion re-reads the link
// (stale snapshots are rejected by the scheduler).
//
// Batches are serialized (one at a time): the fire-handshake does not support
// two batches racing to fire promoted tasks on the same link, and the shared
// progress bar displays one batch.

/// One link in a batch plan: the scheduler identity (link snapshot), the
/// endpoint (probe target + dedup identity), and the protocol row snapshot
/// (fast config type; real probes reload the row WITH config inside the task).
#[derive(Clone)]
struct PlanLink {
    link: ProfileStats,
    endpoint: Endpoint,
    protocol: DbProtocol,
}

/// Outcome of one dispatched probe.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProbeOutcome {
    Ok {
        latency_ms: Option<u64>,
        ip_info: Option<String>,
    },
    Failed(String),
}

/// Probe execution seam. The production impl runs the real engines
/// ([`FastPingManager`] + the pooled core); tests stub it so the batch
/// pipeline is exercised hermetically.
trait BatchProbeRunner: Send + Sync {
    /// One fast probe for a unique (address, port) — the batch dedups calls.
    fn fast<'a>(
        &'a self,
        config_type: i32,
        addr: &'a str,
        port: u16,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>>;

    /// One real probe for a single link.
    fn real<'a>(
        &'a self,
        endpoint: &'a Endpoint,
        link: &'a ProfileStats,
        protocol: &'a DbProtocol,
        req: SinglePingReq<'a>,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>>;
}

/// Production runner: [`FastPingManager`] for fast probes, the shared
/// [`CorePool`] for real probes — the exact dispatch pattern the kept
/// `start_tcp_ping` / `start_real_ping` use.
///
/// While the batch is alive the pool's `batch_active` flag is set, so
/// concurrent single pings take the fresh-core path instead of reusing a core
/// the batch is touching.
struct EngineProbeRunner {
    pool: Arc<CorePool>,
    batch_active: Arc<std::sync::atomic::AtomicBool>,
}

impl EngineProbeRunner {
    fn new(pool: Arc<CorePool>) -> Self {
        let batch_active = pool.batch_active_flag();
        batch_active.store(true, Ordering::Relaxed);
        Self { pool, batch_active }
    }
}

impl Drop for EngineProbeRunner {
    fn drop(&mut self) {
        self.batch_active.store(false, Ordering::Relaxed);
    }
}

impl BatchProbeRunner for EngineProbeRunner {
    fn fast<'a>(
        &'a self,
        config_type: i32,
        addr: &'a str,
        port: u16,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
        Box::pin(async move {
            match xray_tui_core::FastPingManager::new(timeout)
                .ping(config_type, addr, port)
                .await
            {
                Ok(dur) => ProbeOutcome::Ok {
                    latency_ms: Some(dur.as_millis() as u64),
                    ip_info: None,
                },
                Err(e) => ProbeOutcome::Failed(e.to_string()),
            }
        })
    }

    fn real<'a>(
        &'a self,
        endpoint: &'a Endpoint,
        link: &'a ProfileStats,
        protocol: &'a DbProtocol,
        req: SinglePingReq<'a>,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
        Box::pin(async move {
            let result = self.pool.ping(endpoint, link, protocol, req).await;
            match result.error {
                Some(e) => ProbeOutcome::Failed(e),
                None => ProbeOutcome::Ok {
                    latency_ms: result.latency_ms,
                    ip_info: result.ip_info,
                },
            }
        })
    }
}

/// Shared per-batch state, cloned into every spawned probe task.
struct BatchShared {
    sched: Arc<TaskScheduler>,
    db: Arc<Database>,
    tx: mpsc::Sender<CoreEvent>,
    runner: Arc<dyn BatchProbeRunner>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    progress: Arc<(AtomicU16, AtomicU16)>,
    plan: Vec<PlanLink>,
    real_phase: bool,
    dedup_endpoints: bool,
    fast_timeout: Duration,
    real_timeout: Duration,
    real_retries: u32,
    ping_url: String,
    ip_api_url: String,
    defer_delay: Duration,
    real_concurrency: usize,
    /// "Clear error after" (design §6.4): `None` = never sweep.
    error_ttl_hours: Option<i64>,
    total: u16,
    /// Fast config type per link (derived from the plan's protocol kind).
    fast_config: HashMap<(ProtocolId, EndpointId), i32>,
    /// Endpoint rows by id (real probes need the full endpoint).
    endpoints: HashMap<EndpointId, Endpoint>,
    // ── phase tracking ────────────────────────────────────────────────
    pending_fast: AtomicUsize,
    pending_real: AtomicUsize,
    done: AtomicU16,
    phase1_settled: Notify,
    real_settled: Notify,
    // ── fast-probe dedup: one TCP ping per unique (address, port) ─────
    fast_dedup: Mutex<FastDedupInner>,
    // ── real-phase endpoint dedup: endpoints whose real ping succeeded ─
    completed_endpoints: Mutex<HashSet<i64>>,
}

struct FastDedupInner {
    cache: HashMap<(String, u16), ProbeOutcome>,
    in_flight: HashMap<(String, u16), Arc<Notify>>,
}

/// Parameters for one batch run. Built by the entry points from `AppState`;
/// tests construct these directly with a stubbed runner.
pub(crate) struct BatchParams {
    scheduler: Arc<TaskScheduler>,
    db: Arc<Database>,
    tx: mpsc::Sender<CoreEvent>,
    runner: Arc<dyn BatchProbeRunner>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    progress: Arc<(AtomicU16, AtomicU16)>,
    plan: Vec<PlanLink>,
    real_phase: bool,
    dedup_endpoints: bool,
    fast_timeout: Duration,
    real_timeout: Duration,
    real_retries: u32,
    ping_url: String,
    ip_api_url: String,
    defer_delay: Duration,
    real_concurrency: usize,
    error_ttl_hours: Option<i64>,
}

impl From<BatchParams> for BatchShared {
    fn from(p: BatchParams) -> Self {
        let total = u16::try_from(p.plan.len()).unwrap_or(u16::MAX);
        let fast_config = p
            .plan
            .iter()
            .map(|pl| {
                (
                    (pl.link.protocol_id, pl.link.endpoint_id),
                    pl.protocol.proto_kind.to_i32(),
                )
            })
            .collect();
        let endpoints = p
            .plan
            .iter()
            .map(|pl| (pl.endpoint.id, pl.endpoint.clone()))
            .collect();
        Self {
            sched: p.scheduler,
            db: p.db,
            tx: p.tx,
            runner: p.runner,
            stop: p.stop,
            progress: p.progress,
            plan: p.plan,
            real_phase: p.real_phase,
            dedup_endpoints: p.dedup_endpoints,
            fast_timeout: p.fast_timeout,
            real_timeout: p.real_timeout,
            real_retries: p.real_retries,
            ping_url: p.ping_url,
            ip_api_url: p.ip_api_url,
            defer_delay: p.defer_delay,
            real_concurrency: p.real_concurrency,
            error_ttl_hours: p.error_ttl_hours,
            total,
            fast_config,
            endpoints,
            pending_fast: AtomicUsize::new(0),
            pending_real: AtomicUsize::new(0),
            done: AtomicU16::new(0),
            phase1_settled: Notify::new(),
            real_settled: Notify::new(),
            fast_dedup: Mutex::new(FastDedupInner {
                cache: HashMap::new(),
                in_flight: HashMap::new(),
            }),
            completed_endpoints: Mutex::new(HashSet::new()),
        }
    }
}

/// Run one batch to completion. Spawned by the entry points; awaited directly
/// by the tests.
pub(crate) async fn run_batch(params: BatchParams) {
    let shared = Arc::new(BatchShared::from(params));
    let _ = shared.tx.try_send(CoreEvent::BatchProgress {
        total: shared.total,
        completed: 0,
    });

    // ── Phase 1: one FastPing task per link ───────────────────────────
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    let mut deferred: Vec<PlanLink> = Vec::new();
    for plan in &shared.plan {
        if shared.stop.load(Ordering::Relaxed) {
            // Stop at the dispatch boundary: remaining links are never
            // scheduled, so they never write anything.
            break;
        }
        match shared
            .sched
            .schedule(&plan.link, TaskKind::FastPing, shared.db.as_ref())
            .await
        {
            ScheduleOutcome::Started(id) => {
                shared.pending_fast.fetch_add(1, Ordering::Relaxed);
                let fresh = shared
                    .read_link(&plan.link)
                    .await
                    .unwrap_or_else(|| plan.link.clone());
                let shared = shared.clone();
                handles.push(tokio::spawn(run_task_chain(
                    shared,
                    fresh,
                    id,
                    TaskKind::FastPing,
                )));
            }
            ScheduleOutcome::Queued(_) => {
                // The gate holder's completion promotes and fires this task.
                shared.pending_fast.fetch_add(1, Ordering::Relaxed);
            }
            ScheduleOutcome::DnsDeferred => deferred.push(plan.clone()),
            ScheduleOutcome::QueueFull => {
                tracing::warn!(target: "tui::ops::ping", "batch: link skipped, queue full");
            }
        }
    }
    for plan in deferred {
        let shared = shared.clone();
        handles.push(tokio::spawn(retry_deferred_fast(shared, plan)));
    }

    // Wait for every fast task to settle (gate clear per link).
    loop {
        if shared.pending_fast.load(Ordering::Relaxed) == 0 {
            break;
        }
        shared.phase1_settled.notified().await;
    }
    for h in handles {
        let _ = h.await;
    }

    if !shared.real_phase {
        finish_batch(&shared).await;
        return;
    }

    // ── Phase 2: one RealPing task per link, fired per endpoint ───────
    let mut per_endpoint: BTreeMap<i64, Vec<(ProfileStats, u16)>> = BTreeMap::new();
    let mut deferred_real: Vec<PlanLink> = Vec::new();
    for plan in &shared.plan {
        if shared.stop.load(Ordering::Relaxed) {
            break;
        }
        match shared
            .sched
            .schedule(&plan.link, TaskKind::RealPing, shared.db.as_ref())
            .await
        {
            ScheduleOutcome::Started(id) => {
                shared.pending_real.fetch_add(1, Ordering::Relaxed);
                let fresh = shared
                    .read_link(&plan.link)
                    .await
                    .unwrap_or_else(|| plan.link.clone());
                per_endpoint
                    .entry(plan.endpoint.id.get())
                    .or_default()
                    .push((fresh, id));
            }
            ScheduleOutcome::Queued(_) => {
                // Promoted by the gate holder's completion (serialized batches
                // start clean, so this is the rare defensive path).
                shared.pending_real.fetch_add(1, Ordering::Relaxed);
            }
            ScheduleOutcome::DnsDeferred => deferred_real.push(plan.clone()),
            ScheduleOutcome::QueueFull => {
                tracing::warn!(target: "tui::ops::ping", "batch: real ping skipped, queue full");
            }
        }
    }
    let mut deferred_real_handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    for plan in deferred_real {
        let shared = shared.clone();
        deferred_real_handles.push(tokio::spawn(retry_deferred_real(shared, plan)));
    }

    // Fire one endpoint group at a time (bounded by `real_concurrency`); links
    // within an endpoint fire sequentially so `dedup_endpoints` can stop at
    // the first success.
    let sem = Arc::new(Semaphore::new(shared.real_concurrency.max(1)));
    let mut real_handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    for (_, group) in per_endpoint {
        let shared = shared.clone();
        let sem = sem.clone();
        real_handles.push(tokio::spawn(async move {
            let Ok(_permit) = sem.acquire_owned().await else {
                return;
            };
            for (fresh, id) in group {
                if shared.stop.load(Ordering::Relaxed) {
                    shared.retire_real(&fresh, id).await;
                    continue;
                }
                if shared.dedup_endpoints
                    && shared
                        .completed_endpoints
                        .lock()
                        .contains(&fresh.endpoint_id.get())
                {
                    // A sibling already succeeded: cancel queued real tasks and
                    // retire this link's real task without a probe.
                    shared.retire_real(&fresh, id).await;
                    continue;
                }
                run_task_chain(shared.clone(), fresh, id, TaskKind::RealPing).await;
            }
        }));
    }

    // Wait for every real task to settle.
    loop {
        if shared.pending_real.load(Ordering::Relaxed) == 0 {
            break;
        }
        shared.real_settled.notified().await;
    }
    for h in real_handles {
        let _ = h.await;
    }
    // Join the DNS-deferred retries: they may still be sleeping while every
    // already-scheduled task settled (`pending_real == 0` does not account for
    // deferrals). `finish_batch` must run only after ALL work — including
    // their late results and progress events — has been emitted; otherwise the
    // terminal `BatchProgress{0,0}` would clear the progress bar before the
    // retries' events arrive and re-create it, leaving it stuck forever.
    for h in deferred_real_handles {
        let _ = h.await;
    }
    finish_batch(&shared).await;
}

/// Signal the batch's end: total 0 makes the events handler clear the shared
/// progress and re-arm the stop flag. Runs the error-TTL sweep first (design
/// §6.4): batch completion is a natural "errors are fresh now" boundary, so
/// persisted failure markers older than the configured TTL are cleared
/// before the terminal progress event lands. Links the batch did not touch
/// (dedup-cancelled siblings, queue-full/stop skips) are exactly the ones
/// whose stale markers this clears.
async fn finish_batch(shared: &BatchShared) {
    shared.progress.0.store(0, Ordering::Relaxed);
    crate::ops::profiles::clear_expired_errors(&shared.db, shared.error_ttl_hours).await;
    let _ = shared.tx.try_send(CoreEvent::BatchProgress {
        total: 0,
        completed: 0,
    });
}

/// Re-schedule a DNS-deferred fast link after the deferral window.
async fn retry_deferred_fast(shared: Arc<BatchShared>, plan: PlanLink) {
    tokio::time::sleep(shared.defer_delay).await;
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            return;
        }
        match shared
            .sched
            .schedule(&plan.link, TaskKind::FastPing, shared.db.as_ref())
            .await
        {
            ScheduleOutcome::Started(id) => {
                shared.pending_fast.fetch_add(1, Ordering::Relaxed);
                let fresh = shared
                    .read_link(&plan.link)
                    .await
                    .unwrap_or_else(|| plan.link.clone());
                run_task_chain(shared, fresh, id, TaskKind::FastPing).await;
                return;
            }
            ScheduleOutcome::Queued(_) => {
                shared.pending_fast.fetch_add(1, Ordering::Relaxed);
                return;
            }
            ScheduleOutcome::DnsDeferred => {
                tokio::time::sleep(shared.defer_delay.min(Duration::from_millis(250))).await;
            }
            ScheduleOutcome::QueueFull => return,
        }
    }
}

/// Re-schedule a DNS-deferred real link after the deferral window (with the
/// same sibling-dedup check as the main fire path).
async fn retry_deferred_real(shared: Arc<BatchShared>, plan: PlanLink) {
    tokio::time::sleep(shared.defer_delay).await;
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            return;
        }
        match shared
            .sched
            .schedule(&plan.link, TaskKind::RealPing, shared.db.as_ref())
            .await
        {
            ScheduleOutcome::Started(id) => {
                shared.pending_real.fetch_add(1, Ordering::Relaxed);
                let fresh = shared
                    .read_link(&plan.link)
                    .await
                    .unwrap_or_else(|| plan.link.clone());
                if shared.dedup_endpoints
                    && shared
                        .completed_endpoints
                        .lock()
                        .contains(&fresh.endpoint_id.get())
                {
                    shared.retire_real(&fresh, id).await;
                } else {
                    run_task_chain(shared, fresh, id, TaskKind::RealPing).await;
                }
                return;
            }
            ScheduleOutcome::Queued(_) => {
                shared.pending_real.fetch_add(1, Ordering::Relaxed);
                return;
            }
            ScheduleOutcome::DnsDeferred => {
                tokio::time::sleep(shared.defer_delay.min(Duration::from_millis(250))).await;
            }
            ScheduleOutcome::QueueFull => return,
        }
    }
}

/// Drive one scheduled task to completion: dispatch the probe (or retire the
/// task silently under stop), `complete` it, then fire whatever the gate
/// promotes — repeating until the link's gate is clear.
async fn run_task_chain(
    shared: Arc<BatchShared>,
    mut link: ProfileStats,
    mut id: u16,
    mut kind: TaskKind,
) {
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            // Stop pressed at a dispatch boundary: retire this task silently —
            // no result event, no error marker.
            shared.sched.complete(&link, kind, shared.db.as_ref()).await;
            shared.note_settled(kind);
        } else {
            match kind {
                TaskKind::FastPing => {
                    let outcome = shared.fast_probe(&link).await;
                    shared.emit_result(&link, TestType::TcpPing, &outcome);
                    shared.sched.complete(&link, kind, shared.db.as_ref()).await;
                    shared.note_settled(kind);
                }
                TaskKind::RealPing => {
                    let outcome = shared.real_probe(&link).await;
                    // A successful real ping records the endpoint so the
                    // sibling-dedup pass skips its remaining links.
                    if shared.dedup_endpoints && matches!(outcome, ProbeOutcome::Ok { .. }) {
                        shared
                            .completed_endpoints
                            .lock()
                            .insert(link.endpoint_id.get());
                    }
                    shared.emit_result(&link, TestType::RealPing, &outcome);
                    shared.sched.complete(&link, kind, shared.db.as_ref()).await;
                    shared.note_settled(kind);
                }
                _ => return, // SpeedTest/UdpTest tasks are not part of the batch
            }
        }
        // Fire the promoted task, if any. The link snapshot is stale after
        // `complete`, so re-read it (the scheduler rejects stale completions).
        let Some(fresh) = shared.read_link(&link).await else {
            return;
        };
        let Some(next_id) = fresh.task_id else {
            return;
        };
        if next_id == id {
            // The gate did not advance (stale completion) — do not spin.
            return;
        }
        let Some(next_kind) = shared.sched.kind_of(next_id) else {
            // Unregistered/orphan id: the next `schedule` pass reconciles it;
            // nothing to fire.
            return;
        };
        link = fresh;
        id = next_id;
        kind = next_kind;
    }
}

impl BatchShared {
    /// Re-read a link's persisted row (the fire-handshake's fresh snapshot).
    async fn read_link(&self, link: &ProfileStats) -> Option<ProfileStats> {
        self.db
            .read_link(link.protocol_id, link.endpoint_id)
            .await
            .ok()
            .flatten()
    }

    /// Fast probe with batch-level dedup: one TCP ping per unique
    /// (address, port); followers await the owner's result and reuse it.
    async fn fast_probe(&self, link: &ProfileStats) -> ProbeOutcome {
        let Some(endpoint) = self.endpoints.get(&link.endpoint_id) else {
            return ProbeOutcome::Failed("Endpoint not found for fast ping".to_string());
        };
        let key = (endpoint.host.clone(), endpoint.port);
        let (is_owner, notify) = {
            let mut inner = self.fast_dedup.lock();
            if let Some(outcome) = inner.cache.get(&key) {
                return outcome.clone();
            }
            let pair = match inner.in_flight.entry(key.clone()) {
                Entry::Occupied(existing) => (false, existing.get().clone()),
                Entry::Vacant(slot) => {
                    let n = Arc::new(Notify::new());
                    slot.insert(n.clone());
                    (true, n)
                }
            };
            // Drop the guard before the follower awaits below — a stuck
            // follower must not hold the dedup lock while waiting.
            drop(inner);
            pair
        };
        let timeout = self.fast_timeout;
        let config_type = self
            .fast_config
            .get(&(link.protocol_id, link.endpoint_id))
            .copied()
            .unwrap_or(0);
        let addr = endpoint.host.clone();
        let port = endpoint.port;
        if is_owner {
            let outcome = self.runner.fast(config_type, &addr, port, timeout).await;
            let mut inner = self.fast_dedup.lock();
            inner.cache.insert(key.clone(), outcome.clone());
            inner.in_flight.remove(&key);
            drop(inner);
            notify.notify_waiters();
            outcome
        } else {
            // Follower: bounded wait — a stuck owner must not wedge the batch.
            tokio::time::timeout(timeout + Duration::from_secs(1), notify.notified())
                .await
                .ok();
            self.fast_dedup
                .lock()
                .cache
                .get(&key)
                .cloned()
                .unwrap_or_else(|| {
                    ProbeOutcome::Failed("fast ping dedup: owner result lost".to_string())
                })
        }
    }

    /// Real probe for one link: reload the protocol row WITH its deferred
    /// config (the builders refuse unloaded configs — mirrors
    /// `start_real_ping`), then run through the pooled core.
    async fn real_probe(&self, link: &ProfileStats) -> ProbeOutcome {
        let protocol = match load_protocol_with_config(&self.db, link.protocol_id).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                return ProbeOutcome::Failed("Protocol row not found for real ping".to_string());
            }
            Err(e) => {
                return ProbeOutcome::Failed(format!("Failed to load protocol: {e}"));
            }
        };
        let Some(endpoint) = self.endpoints.get(&link.endpoint_id) else {
            return ProbeOutcome::Failed("Endpoint not found for real ping".to_string());
        };
        self.runner
            .real(
                endpoint,
                link,
                &protocol,
                SinglePingReq {
                    ping_url: &self.ping_url,
                    ip_api_url: &self.ip_api_url,
                    timeout: self.real_timeout,
                    retries: self.real_retries,
                },
            )
            .await
    }

    /// Send the `SpeedTestResult` event for a completed probe. The
    /// `TestTypeUpdate` re-arms the events handler's per-protocol dedupe guard
    /// (`testing_profiles`) before the result lands — the established pattern
    /// for the two-phase batch (phase 2 re-arms for its real result).
    fn emit_result(&self, link: &ProfileStats, test_type: TestType, outcome: &ProbeOutcome) {
        let (latency_ms, ip_info, error) = match outcome {
            ProbeOutcome::Ok {
                latency_ms,
                ip_info,
            } => (*latency_ms, ip_info.clone(), None),
            ProbeOutcome::Failed(e) => (None, None, Some(e.clone())),
        };
        let _ = self.tx.try_send(CoreEvent::TestTypeUpdate {
            protocol_id: link.protocol_id.get(),
            test_type,
        });
        try_send_or_warn(
            &self.tx,
            CoreEvent::SpeedTestResult {
                protocol_id: link.protocol_id.get(),
                test_type,
                latency_ms,
                speed_bps: None,
                ip_info,
                error,
            },
            "batch_ping_result",
        );
    }

    /// Retire a real task without a probe (stop or sibling-dedup): cancel any
    /// queued real ids, then complete the live task if it is a real task.
    /// Never writes a result event, so no error marker is persisted.
    async fn retire_real(&self, fresh: &ProfileStats, id: u16) {
        self.sched
            .cancel_queued(fresh, TaskKind::RealPing, self.db.as_ref())
            .await;
        if self.sched.kind_of(id) == Some(TaskKind::RealPing) {
            self.sched
                .complete(fresh, TaskKind::RealPing, self.db.as_ref())
                .await;
        }
        self.note_settled(TaskKind::RealPing);
    }

    /// One task settled: update the phase counters, and for a final-phase
    /// settle bump the progress counter + emit a `BatchProgress` event.
    fn note_settled(&self, kind: TaskKind) {
        match kind {
            TaskKind::FastPing => {
                if self.pending_fast.fetch_sub(1, Ordering::Relaxed) == 1 {
                    self.phase1_settled.notify_waiters();
                }
                // Fast-only batches count fast settles as progress.
                if !self.real_phase {
                    self.bump_progress();
                }
            }
            TaskKind::RealPing => {
                if self.pending_real.fetch_sub(1, Ordering::Relaxed) == 1 {
                    self.real_settled.notify_waiters();
                }
                self.bump_progress();
            }
            _ => {}
        }
    }

    fn bump_progress(&self) {
        let done = self.done.fetch_add(1, Ordering::Relaxed) + 1;
        self.progress.1.store(done, Ordering::Relaxed);
        let _ = self.tx.try_send(CoreEvent::BatchProgress {
            total: self.total,
            completed: done,
        });
    }
}

// ── Entry points ───────────────────────────────────────────────────────────

/// Build the per-link plan for every visible (filtered) endpoint.
fn plan_all_visible(state: &AppState) -> Vec<PlanLink> {
    state.filtered_profiles().flat_map(plan_row_links).collect()
}

/// Build the per-link plan for the currently selected endpoint (collapsed
/// multi-protocol rows).
fn plan_selected_endpoint(state: &AppState) -> Vec<PlanLink> {
    let Some(ep_id) = state.selected_profile_id() else {
        return Vec::new();
    };
    state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == ep_id)
        .map(|row| plan_row_links(row).collect())
        .unwrap_or_default()
}

fn plan_row_links(row: &EndpointRow) -> impl Iterator<Item = PlanLink> + '_ {
    row.links.iter().filter_map(|link| {
        let protocol = row.protocols.get(&link.protocol_id)?.clone();
        Some(PlanLink {
            link: link.clone(),
            endpoint: row.endpoint.clone(),
            protocol,
        })
    })
}

/// Batch fast-ping every link of every visible endpoint.
pub fn start_batch_ping(state: &mut AppState) {
    let plan = plan_all_visible(state);
    start_batch(state, plan, false, false);
}

/// Batch fast-ping every visible link, then real-ping each link. With
/// `real_ping_test_all_protocols` unset (default), one successful real ping
/// on an endpoint retires the remaining links' real tasks.
pub fn start_batch_then_real_ping(state: &mut AppState) {
    let plan = plan_all_visible(state);
    let dedup = !state.config.speed_test.real_ping_test_all_protocols;
    start_batch(state, plan, true, dedup);
}

/// Fast-ping every link of the selected endpoint (collapsed multi-protocol rows).
pub fn start_endpoint_batch_ping(state: &mut AppState) {
    let plan = plan_selected_endpoint(state);
    start_batch(state, plan, false, false);
}

/// Fast-ping then real-ping every link of the selected endpoint. `dedup` is
/// off: every protocol of the endpoint gets a real ping (their exit IPs may
/// differ).
pub fn start_endpoint_batch_real_ping(state: &mut AppState) {
    let plan = plan_selected_endpoint(state);
    start_batch(state, plan, true, false);
}

fn start_batch(state: &mut AppState, plan: Vec<PlanLink>, real_phase: bool, dedup_endpoints: bool) {
    if plan.is_empty() {
        state.log_trace(
            "info",
            "tui::ops::ping",
            if real_phase {
                "No profiles to test"
            } else {
                "No profiles to ping"
            },
        );
        return;
    }
    // Batches are serialized: the shared gate does not support two batches
    // racing to fire promoted tasks on the same link, and the progress bar
    // displays one batch.
    if state.batch_progress.is_some() {
        state.log_trace("warn", "tui::ops::ping", "A batch is already running");
        return;
    }
    // A fresh user gesture always starts with a clear stop flag.
    state.speed_test_stop.store(false, Ordering::Relaxed);
    let Some(tx) = state.core_event_tx.clone() else {
        return;
    };
    let pool = get_or_create_pool(state);
    let runner: Arc<dyn BatchProbeRunner> = Arc::new(EngineProbeRunner::new(pool));
    let db = state.db.clone();
    let scheduler = state.scheduler.clone();
    let stop = state.speed_test_stop.clone();
    let total = u16::try_from(plan.len()).unwrap_or(u16::MAX);
    let progress = Arc::new((AtomicU16::new(total), AtomicU16::new(0)));
    state.batch_progress = Some(progress.clone());
    let fast_timeout = *state.config.speed_test.tcp_timeout_secs;
    let real_timeout = *state.config.speed_test.real_ping_timeout_secs;
    let real_retries = state.config.speed_test.real_ping_retries;
    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let real_concurrency = state.config.speed_test.real_ping_concurrency.max(1);
    let error_ttl_hours = state.config.speed_test.error_ttl_hours;
    // Sleep the full deferral window once, then re-schedule (the window is
    // measured in whole seconds and comes from the speed-test config via
    // `TaskScheduler::set_limits`).
    let defer_delay = Duration::from_secs(scheduler.dns_defer_secs().max(1) as u64);

    tokio::spawn(run_batch(BatchParams {
        scheduler,
        db,
        tx,
        runner,
        stop,
        progress,
        plan,
        real_phase,
        dedup_endpoints,
        fast_timeout,
        real_timeout,
        real_retries,
        ping_url,
        ip_api_url,
        defer_delay,
        real_concurrency,
        error_ttl_hours,
    }));
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::sync::atomic::AtomicUsize;

    use tokio::sync::mpsc;
    use xray_tui_db::models::{ErrorInfo, Latency, ProfileErr};

    use crate::ops::profiles::test_support::{fake_row, test_state, ts};
    use crate::ops::scheduler::TaskScheduler;

    use super::*;

    /// Deterministic probe runner: fixed outcomes + call recording + an
    /// optional gate that blocks real probes (for the stop-mid-batch test).
    struct StubRunner {
        fast_outcome: ProbeOutcome,
        real_outcome: Mutex<ProbeOutcome>,
        fast_calls: Arc<AtomicUsize>,
        real_calls: Arc<AtomicUsize>,
        real_probed: Arc<Mutex<HashSet<i64>>>,
        real_gate: Mutex<Option<Arc<Notify>>>,
        /// When set, every fast probe marks the endpoint's DNS failure — used
        /// to land a deferral deterministically between phase 1 and phase 2.
        dns_mark_on_fast: Mutex<Option<(Arc<TaskScheduler>, EndpointId)>>,
    }

    impl StubRunner {
        fn new() -> Self {
            Self {
                fast_outcome: ProbeOutcome::Ok {
                    latency_ms: Some(10),
                    ip_info: None,
                },
                real_outcome: Mutex::new(ProbeOutcome::Ok {
                    latency_ms: Some(50),
                    ip_info: Some("1.2.3.4".to_string()),
                }),
                fast_calls: Arc::new(AtomicUsize::new(0)),
                real_calls: Arc::new(AtomicUsize::new(0)),
                real_probed: Arc::new(Mutex::new(HashSet::new())),
                real_gate: Mutex::new(None),
                dns_mark_on_fast: Mutex::new(None),
            }
        }
    }

    impl BatchProbeRunner for StubRunner {
        fn fast<'a>(
            &'a self,
            _config_type: i32,
            _addr: &'a str,
            _port: u16,
            _timeout: Duration,
        ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.fast_calls.fetch_add(1, Ordering::Relaxed);
                if let Some((sched, eid)) = &*self.dns_mark_on_fast.lock() {
                    sched.mark_dns_failure(*eid);
                }
                self.fast_outcome.clone()
            })
        }

        fn real<'a>(
            &'a self,
            _endpoint: &'a Endpoint,
            link: &'a ProfileStats,
            _protocol: &'a DbProtocol,
            _req: SinglePingReq<'a>,
        ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.real_calls.fetch_add(1, Ordering::Relaxed);
                self.real_probed.lock().insert(link.protocol_id.get());
                // Clone the gate out of the lock so the await below does not
                // hold the mutex guard across the yield point.
                let gate = self.real_gate.lock().clone();
                if let Some(gate) = gate {
                    gate.notified().await;
                }
                self.real_outcome.lock().clone()
            })
        }
    }

    struct Harness {
        state: AppState,
        tx: mpsc::Sender<CoreEvent>,
        runner: Arc<StubRunner>,
    }

    async fn harness(rows: Vec<EndpointRow>) -> Harness {
        let mut state = test_state(rows.clone()).await;
        // Persist the plan rows so the scheduler gate (`write_task_state`
        // requires the row) and the real-probe protocol loads work.
        for row in &rows {
            for link in &row.links {
                state.db.upsert_link(link).await.expect("upsert link");
                if let Some(proto) = row.protocols.get(&link.protocol_id) {
                    state
                        .db
                        .upsert_protocol(proto)
                        .await
                        .expect("upsert protocol");
                }
            }
        }
        let (tx, rx) = mpsc::channel(512);
        state.core_event_rx = Some(rx);
        state.core_event_tx = Some(tx.clone());
        let runner = Arc::new(StubRunner::new());
        Harness { state, tx, runner }
    }

    fn plan_from_rows(rows: &[EndpointRow]) -> Vec<PlanLink> {
        rows.iter().flat_map(plan_row_links).collect()
    }

    fn build_params(
        h: &Harness,
        plan: Vec<PlanLink>,
        real_phase: bool,
        dedup: bool,
    ) -> BatchParams {
        let total = u16::try_from(plan.len()).unwrap_or(u16::MAX);
        BatchParams {
            scheduler: h.state.scheduler.clone(),
            db: h.state.db.clone(),
            tx: h.tx.clone(),
            runner: h.runner.clone(),
            stop: h.state.speed_test_stop.clone(),
            progress: Arc::new((AtomicU16::new(total), AtomicU16::new(0))),
            plan,
            real_phase,
            dedup_endpoints: dedup,
            fast_timeout: Duration::from_secs(2),
            real_timeout: Duration::from_secs(2),
            real_retries: 1,
            ping_url: "http://127.0.0.1/".to_string(),
            ip_api_url: "http://127.0.0.1/ip".to_string(),
            defer_delay: Duration::from_millis(50),
            real_concurrency: 8,
            error_ttl_hours: None,
        }
    }

    fn start_test_batch(
        h: &mut Harness,
        plan: Vec<PlanLink>,
        real_phase: bool,
        dedup: bool,
    ) -> tokio::task::JoinHandle<()> {
        let p = build_params(h, plan, real_phase, dedup);
        h.state.batch_progress = Some(p.progress.clone());
        tokio::spawn(run_batch(p))
    }

    /// Poll events until the batch's final `BatchProgress{0,0}` clears the
    /// shared progress (or the deadline expires).
    async fn await_batch_done(state: &mut AppState) {
        for _ in 0..300 {
            let _ = state.poll_core_events().await;
            if state.batch_progress.is_none() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("batch did not finish within the deadline");
    }

    async fn assert_gate_clear(db: &Database, link: &ProfileStats) {
        let stored = SchedulerDb::read_link(db, link.protocol_id, link.endpoint_id)
            .await
            .expect("read link")
            .expect("link persisted");
        assert_eq!(stored.task_id, None, "gate must be clear after the batch");
    }

    // ── phase 1 + phase 2 on a 3-link batch ─────────────────────────────

    #[tokio::test]
    async fn batch_three_links_schedules_fast_then_real() {
        let rows = vec![
            fake_row(1, "10.0.0.1", 1),
            fake_row(2, "10.0.0.2", 1),
            fake_row(3, "10.0.0.3", 1),
        ];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        // Phase 1: one fast task per link (3 unique addresses → 3 probes).
        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 3);
        // Phase 2: one real task per link.
        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 3);
        for row in &h.state.endpoints {
            for link in &row.links {
                // Real ping superseded the fast latency; no error markers.
                assert!(
                    matches!(
                        link.latency,
                        Some(Latency::Real { delay: 50, ip: Some(ref s) }) if s == "1.2.3.4"
                    ),
                    "unexpected latency {link:?}"
                );
                assert!(link.error.is_none(), "no marker expected: {link:?}");
                assert_gate_clear(&h.state.db, link).await;
            }
        }
    }

    // ── error-TTL sweep at batch completion ──────────────────────────────

    #[tokio::test]
    async fn batch_completion_sweeps_stale_error_markers() {
        // A link with a persisted error whose updated_at predates the TTL.
        // The batch is stopped before dispatch, so no probe touches the link
        // — the `finish_batch` sweep must clear the stale marker from the DB
        // (links the batch never re-tests are exactly the ones it clears).
        let rows = vec![fake_row(1, "10.0.0.1", 1)];
        let mut h = harness(rows.clone()).await;
        let mut link = h.state.endpoints[0].links[0].clone();
        link.error = Some(ErrorInfo {
            kind: ProfileErr::Fast,
            text: "old failure".to_string(),
        });
        h.state.db.upsert_link(&link).await.expect("upsert link");
        let mut conn = h.state.db.connection().await.unwrap();
        xray_tui_db::models::ProfileStats::filter_by_protocol_id_and_endpoint_id(
            link.protocol_id,
            link.endpoint_id,
        )
        .update()
        .updated_at(ts(jiff::Timestamp::now().as_second() - 48 * 3600))
        .exec(&mut conn)
        .await
        .unwrap();

        let plan = plan_from_rows(&rows);
        let mut params = build_params(&h, plan, false, false);
        params.error_ttl_hours = Some(24);
        h.state.batch_progress = Some(params.progress.clone());
        h.state.speed_test_stop.store(true, Ordering::Relaxed);
        tokio::spawn(run_batch(params)).await.unwrap();
        await_batch_done(&mut h.state).await;

        let db: &Database = &h.state.db;
        let stored = SchedulerDb::read_link(db, link.protocol_id, link.endpoint_id)
            .await
            .expect("read link")
            .expect("link persisted");
        assert!(
            stored.error.is_none(),
            "stale marker swept at batch completion: {stored:?}"
        );
    }

    #[tokio::test]
    async fn fast_dedup_fans_out_to_links_sharing_address() {
        // Two links on one endpoint share the (address, port) key: one TCP
        // ping, result fanned to both links.
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, false, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(
            h.runner.fast_calls.load(Ordering::Relaxed),
            1,
            "one TCP ping per unique address"
        );
        for link in &h.state.endpoints[0].links {
            assert!(matches!(link.latency, Some(Latency::Fast { delay: 10 })));
            assert!(link.error.is_none());
            assert_gate_clear(&h.state.db, link).await;
        }
    }

    // ── sibling cancel (dedup_endpoints) ─────────────────────────────────

    #[tokio::test]
    async fn sibling_cancel_skips_remaining_links_after_first_success() {
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        // dedup_endpoints=true (the default): first success cancels siblings.
        let handle = start_test_batch(&mut h, plan, true, true);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(
            h.runner.real_calls.load(Ordering::Relaxed),
            1,
            "only the first link of the endpoint was real-pinged"
        );
        let links = &h.state.endpoints[0].links;
        let with_real = links
            .iter()
            .filter(|l| matches!(l.latency, Some(Latency::Real { .. })));
        assert_eq!(with_real.count(), 1, "exactly one link got a real result");
        for link in links {
            // The cancelled sibling never wrote a marker or a latency.
            assert!(
                link.error.is_none(),
                "cancelled link must not write a marker"
            );
            assert_gate_clear(&h.state.db, link).await;
        }
    }

    #[tokio::test]
    async fn real_ping_test_all_protocols_tests_every_link() {
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        // real_ping_test_all_protocols=true → dedup off → both links tested.
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 2);
        assert_eq!(h.runner.real_probed.lock().len(), 2);
        for link in &h.state.endpoints[0].links {
            assert!(matches!(
                link.latency,
                Some(Latency::Real { delay: 50, .. })
            ));
            assert!(link.error.is_none());
        }
    }

    // ── all-fail error markers ───────────────────────────────────────────

    #[tokio::test]
    async fn all_fail_writes_real_error_markers_on_every_link() {
        let rows = vec![fake_row(1, "10.0.0.1", 2), fake_row(2, "10.0.0.2", 1)];
        let mut h = harness(rows.clone()).await;
        *h.runner.real_outcome.lock() = ProbeOutcome::Failed("timeout".to_string());
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 3);
        for row in &h.state.endpoints {
            for link in &row.links {
                assert_eq!(
                    link.error.as_ref().map(|e| e.kind),
                    Some(ProfileErr::Real),
                    "every failing link persists a Real error marker: {link:?}"
                );
            }
        }
    }

    // ── DNS deferral ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn dns_deferred_links_are_retried_after_the_window() {
        let rows = vec![fake_row(1, "10.0.0.1", 1)];
        let mut h = harness(rows.clone()).await;
        // Short window (2s): the batch must defer and then re-schedule. Two
        // seconds (not one) keeps the assertion robust to second-boundary
        // crossings: the batch always waits at least ~1s and at most ~2s.
        h.state.scheduler = Arc::new(TaskScheduler::new(3, 2));
        h.state.scheduler.mark_dns_failure(EndpointId::new(1));
        let plan = plan_from_rows(&rows);
        let started = std::time::Instant::now();
        let handle = start_test_batch(&mut h, plan, false, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(
            h.runner.fast_calls.load(Ordering::Relaxed),
            1,
            "the deferred link was eventually probed after the window"
        );
        assert!(
            started.elapsed() >= Duration::from_millis(900),
            "the retry must wait out the deferral window, took {:?}",
            started.elapsed()
        );
        assert!(matches!(
            h.state.endpoints[0].links[0].latency,
            Some(Latency::Fast { .. })
        ));
    }

    #[tokio::test]
    async fn phase2_dns_deferred_retry_is_joined_before_finish() {
        let rows = vec![fake_row(1, "10.0.0.1", 1), fake_row(2, "10.0.0.2", 1)];
        let mut h = harness(rows.clone()).await;
        // 3s window. Endpoint 2 is marked when the phase-1 fast probe for
        // endpoint 1 runs (deterministically BEFORE phase-2 scheduling), so
        // link 200's real task is DNS-deferred and re-scheduled after the
        // window.
        h.state.scheduler = Arc::new(TaskScheduler::new(3, 3));
        *h.runner.dns_mark_on_fast.lock() = Some((h.state.scheduler.clone(), EndpointId::new(2)));
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        // Both links got fast AND real results; no markers; gates clear.
        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 2);
        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 2);
        for row in &h.state.endpoints {
            for link in &row.links {
                assert!(matches!(link.latency, Some(Latency::Real { .. })));
                assert!(link.error.is_none());
                assert_gate_clear(&h.state.db, link).await;
            }
        }
        // Regression (reviewer F1): the deferred-real retry was spawned but
        // never joined, so the terminal `BatchProgress{0,0}` fired before the
        // retry's late events re-created the progress — stuck bar + rejected
        // future batches. The fix joins the retries first, so the terminal
        // event is last and the progress bar is truly cleared.
        assert!(
            h.state.batch_progress.is_none(),
            "progress must be cleared after the batch (terminal event last)"
        );

        // A subsequent batch starts cleanly.
        let plan2 = plan_from_rows(&rows);
        let handle2 = start_test_batch(&mut h, plan2, false, false);
        handle2.await.unwrap();
        await_batch_done(&mut h.state).await;
        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 4);
    }

    // ── stop mid-batch ───────────────────────────────────────────────────

    #[tokio::test]
    async fn stop_mid_batch_writes_no_error_markers() {
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let gate = Arc::new(Notify::new());
        *h.runner.real_gate.lock() = Some(gate.clone());
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);

        // Wait for the first real probe to be in flight, then stop.
        for _ in 0..300 {
            if h.runner.real_calls.load(Ordering::Relaxed) >= 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(h.runner.real_calls.load(Ordering::Relaxed) >= 1);
        h.state.speed_test_stop.store(true, Ordering::Relaxed);
        gate.notify_waiters();

        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        // No link carries an error marker; gates are clear.
        for link in &h.state.endpoints[0].links {
            assert!(
                link.error.is_none(),
                "stopped batch must not mark: {link:?}"
            );
            assert_gate_clear(&h.state.db, link).await;
        }
        // Only the in-flight probe ran; the sibling was retired without a probe.
        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 1);
    }

    // ── progress events ──────────────────────────────────────────────────

    #[tokio::test]
    async fn progress_events_track_total_and_done() {
        let rows = vec![
            fake_row(1, "10.0.0.1", 1),
            fake_row(2, "10.0.0.2", 1),
            fake_row(3, "10.0.0.3", 1),
        ];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, false, false);
        handle.await.unwrap();

        let mut rx = h.state.core_event_rx.take().expect("event receiver");
        let mut events: Vec<(u16, u16)> = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            if let CoreEvent::BatchProgress { total, completed } = ev {
                events.push((total, completed));
            }
        }
        assert!(events.contains(&(3, 0)), "initial event: {events:?}");
        for k in 1..=3 {
            assert!(events.contains(&(3, k)), "missing (3,{k}) in {events:?}");
        }
        assert_eq!(
            events.last(),
            Some(&(0, 0)),
            "final clear event: {events:?}"
        );
    }
}
