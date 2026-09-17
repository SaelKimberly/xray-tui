use std::collections::{BTreeMap, HashMap, HashSet, hash_map::Entry};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::sync::{Notify, Semaphore, mpsc};
use tokio::task::JoinHandle;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::Database;
use xray_tui_db::LinkGroups;
use xray_tui_db::models::Protocol as DbProtocol;
use xray_tui_db::models::{
    Endpoint, EndpointId, EndpointRow, Latency, ProfileStats, ProtocolId, PurgatoryView, TaskKind,
};
use xray_tui_db::profiles_query::{PageRequest, PageSort};
use xray_tui_native::capability;
use xray_tui_proto::proto_spec::ProtocolConfig;

use crate::AppState;
use crate::ops::ping_native::{self, NativeProbeReq, ProbeClass};
use crate::ops::profiles::PROFILES_PAGE_SIZE;
use crate::ops::scheduler::{ScheduleOutcome, TaskScheduler};
use crate::state::load_protocol_with_config;
use crate::try_send_or_warn;
use crate::types::CoreEvent;
/// Bound on concurrent single-test tasks (menu-triggered pings): the spawned
/// future parks on this permit before doing work, so rapid keypresses queue
/// as tiny futures instead of spawning unbounded concurrent cores. Batch
/// paths already gate through their own semaphore/JoinSet.
static SINGLE_TEST_SEMAPHORE: std::sync::LazyLock<Semaphore> =
    std::sync::LazyLock::new(|| Semaphore::new(16));

/// The code default for `speed_test.real_ping_concurrency`
/// (`default_real_ping_concurrency` in `xray-tui-config`), repeated because the
/// warning below is only actionable against it — the config file's own value is
/// what a stale override silently replaces.
const REAL_CONCURRENCY_DEFAULT: u32 = 100;

/// Below this, a real phase is throughput-bound rather than feed-bound. A real
/// probe holds one slot for a dial plus (on failure) the whole timeout, so the
/// rate is roughly `concurrency / timeout`: the 2026-09-16 run measured 2.45
/// results/s at 5 (3,381 results in 23 min), i.e. ~4 h for a 34k-link plan.
const REAL_CONCURRENCY_WARN_BELOW: u32 = 50;

/// Prefix of the persisted marker text for a row the native engine cannot test.
///
/// The marker is a [`xray_tui_db::models::ProfileErr::Real`] — the only failure
/// kind the frozen `error_kind` CHECK accepts (a new variant would be a schema
/// wipe, decision 4) — so the TEXT is the discriminator. One writer
/// ([`untestable_marker_text`]) and one reader ([`is_untestable_marker`], used
/// by the Test-cell precedence and by the Remove-Bad-Servers guard).
pub const UNTESTABLE_PREFIX: &str = "not testable by the native engine: ";

/// The persisted marker text for a link the native engine cannot test.
#[must_use]
pub fn untestable_marker_text(reason: &str) -> String {
    format!("{UNTESTABLE_PREFIX}{reason}")
}

/// True when a persisted failure marker means "never attempted" (the native
/// engine cannot serve this row) rather than "the probe ran and failed".
#[must_use]
pub fn is_untestable_marker(error: &xray_tui_db::models::ErrorInfo) -> bool {
    error.text.starts_with(UNTESTABLE_PREFIX)
}

/// A link counts as "failed" for the Remove-Bad-Servers sweep only when a probe
/// actually ran: the untestable marker says the native engine cannot serve the
/// row, and deleting a profile for that is wrong.
fn is_removable_failure(link: &ProfileStats) -> bool {
    link.error
        .as_ref()
        .is_some_and(|e| !is_untestable_marker(e))
}

/// Start TCP ping on the given profile. Returns immediately; result arrives via `CoreEvent`.
pub fn start_tcp_ping(state: &mut AppState, endpoint_id: i64, protocol_id: i64) {
    if state.testing_profiles.contains(&(endpoint_id, protocol_id)) {
        state.log_trace(
            "warn",
            "tui::ops::ping",
            "Test already in progress for this profile",
        );
        return;
    }
    // Find the profile and extract address:port. Lookup is by the
    // (endpoint, protocol) pair, never by protocol alone: a `Protocol` row is
    // shared across endpoints (identity dedup excludes host/port), so
    // first-owner resolution would probe and update the wrong server when two
    // endpoints share the same config.
    let Some(row) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    else {
        state.log_trace("error", "tui::ops::ping", "Endpoint not found for TCP ping");
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

    // The fast-ping adapters dispatch on the protocol kind.
    let config_type = proto.proto_kind.to_i32();
    state
        .testing_details
        .insert((endpoint_id, protocol_id), TestType::TcpPing);
    state.testing_profiles.insert((endpoint_id, protocol_id));
    state.mark_rows_dirty(crate::ui::profiles::ROWS_DIRTY_TEST);
    let timeout_dur = *state.config.speed_test.tcp_timeout_secs;

    tokio::spawn(async move {
        let Ok(_permit) = SINGLE_TEST_SEMAPHORE.acquire().await else {
            return;
        };
        let fmgr = xray_tui_core::FastPingManager::new(timeout_dur);
        let result = fmgr.ping(config_type, &addr, port).await;
        let (latency_ms, error) = match result {
            Ok(dur) => (Some(dur.as_millis() as u64), None),
            Err(e) => (None, Some(e.to_string())),
        };
        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                endpoint_id,
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

/// Start real ping (one HTTP probe through the native engine).
///
/// The `Protocol` row is re-loaded WITH its deferred `config` inside the
/// spawned task (the probe consumes the typed [`ProtocolConfig`]), then the
/// native capability gate decides between a probe and the persisted
/// "not testable" marker.
pub fn start_real_ping(state: &mut AppState, endpoint_id: i64, protocol_id: i64) {
    if state.testing_profiles.contains(&(endpoint_id, protocol_id)) {
        return;
    }

    // Resolve by the (endpoint, protocol) pair — see `start_tcp_ping` for why
    // protocol-only lookup is wrong for shared `Protocol` rows.
    let endpoint;
    let protocol_id_typed;
    if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == endpoint_id)
        && let Some(l) = r.links.iter().find(|l| l.protocol_id.get() == protocol_id)
    {
        endpoint = r.endpoint.clone();
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
        .insert((endpoint_id, protocol_id), TestType::RealPing);
    state.testing_profiles.insert((endpoint_id, protocol_id));
    state.mark_rows_dirty(crate::ui::profiles::ROWS_DIRTY_TEST);

    let db = state.db.clone();

    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        let Ok(_permit) = SINGLE_TEST_SEMAPHORE.acquire().await else {
            return;
        };
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            endpoint_id,
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
                        endpoint_id,
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
                        endpoint_id,
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

        let config = protocol.config.get().0.clone();
        // Native capability gate: a row the engine cannot serve fails fast with
        // the persisted `[real]` marker instead of a probe.
        if let Some(reason) = capability::support_reason(protocol.proto_kind, &config) {
            try_send_or_warn(
                &tx,
                CoreEvent::SpeedTestResult {
                    endpoint_id,
                    protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some(untestable_marker_text(reason)),
                },
                "real_ping_untestable",
            );
            return;
        }

        let result = ping_native::real_ping(
            &endpoint,
            &config,
            &NativeProbeReq {
                ping_url: &ping_url,
                ip_api_url: &ip_api_url,
                timeout,
                retries,
            },
        )
        .await;
        let (latency_ms, ip_info, error) = match result {
            Ok(r) => (Some(r.latency_ms), r.ip_info, None),
            Err(e) => (None, None, Some(e.text)),
        };

        try_send_or_warn(
            &tx,
            CoreEvent::SpeedTestResult {
                endpoint_id,
                protocol_id,
                test_type: TestType::RealPing,
                latency_ms,
                speed_bps: None,
                ip_info,
                error,
            },
            "real_ping_result",
        );
    });
}

/// Start speed test (download through proxy) on the given profile.
pub fn start_speed_test(state: &mut AppState, endpoint_id: i64, protocol_id: i64) {
    if state.testing_profiles.contains(&(endpoint_id, protocol_id)) {
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
        .insert((endpoint_id, protocol_id), TestType::SpeedTest);
    state.testing_profiles.insert((endpoint_id, protocol_id));
    state.mark_rows_dirty(crate::ui::profiles::ROWS_DIRTY_TEST);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;
    let test_url = "http://cachefly.cachefly.net/1mb.test".to_string();
    let min_dur = std::time::Duration::from_secs(3);
    let max_dur = std::time::Duration::from_secs(10);

    tokio::spawn(async move {
        let Ok(_permit) = SINGLE_TEST_SEMAPHORE.acquire().await else {
            return;
        };
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
                endpoint_id,
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
pub fn start_udp_test(state: &mut AppState, endpoint_id: i64, protocol_id: i64) {
    if state.testing_profiles.contains(&(endpoint_id, protocol_id)) {
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
    state
        .testing_details
        .insert((endpoint_id, protocol_id), TestType::UdpTest);
    state.testing_profiles.insert((endpoint_id, protocol_id));
    state.mark_rows_dirty(crate::ui::profiles::ROWS_DIRTY_TEST);
    let proxy_addr = state.config.inbound.listen.clone();
    let proxy_port = state.config.inbound.socks_port;

    tokio::spawn(async move {
        let Ok(_permit) = SINGLE_TEST_SEMAPHORE.acquire().await else {
            return;
        };
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
                endpoint_id,
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
    let to_remove: Vec<xray_tui_db::models::EndpointId> = state
        .endpoints
        .iter()
        .filter(|r| r.links.iter().any(is_removable_failure))
        .map(|r| r.endpoint.id)
        .collect();
    let count = to_remove.len();
    if count > 0 {
        // ONE transaction for the whole sweep: the per-endpoint path opens a
        // transaction, re-scans for orphan protocols and prunes rank keys
        // every time, which is the wrong shape for "remove everything failed
        // on this page".
        match state.db.delete_endpoints(&to_remove).await {
            Ok(_) => state.log_trace(
                "info",
                "tui::ops::ping",
                &format!("Removed {count} failed server(s)"),
            ),
            Err(e) => state.log_trace(
                "error",
                "tui::ops::ping",
                &format!("Failed to remove failed servers: {e}"),
            ),
        }
        for id in &to_remove {
            state.multi_select.remove(&id.get());
        }
    }
    state.multi_select.clear();
    state.endpoints_gen = state.endpoints_gen.wrapping_add(1);
    state.filter_cache_valid.set(false);
}

// ══════════════════════════════════════════════════════════════════════════
// Batch pipeline (T19) — rebuilt on the TaskScheduler
// ══════════════════════════════════════════════════════════════════════════
//
// Phase 1 schedules one `FastPing` task per link; the fast probes are
// deduplicated by (address, port) — one TCP ping per unique address, the
// result fanned out to every link sharing it (the old `FastCache` semantics).
// It is bounded by `fast_ping_concurrency` (one probe future per unique
// address at a time). `DnsDeferred` links are re-scheduled after the deferral
// window in a spawned task; `QueueFull` links are skipped (the scheduler logs
// the warning).
// Phase 2 (after every phase-1 task settles) schedules one `RealPing` task per
// link on the native engine (`ops/ping_native.rs` with the protocol row
// reloaded WITH config). Links the native engine cannot serve are retired at
// plan time (kind gate) or by their own probe (config gate) and receive the
// persisted `[real]` marker instead of a probe — `emit_untestable_markers`
// runs after phase 1 so the marker is never cleared by a fast success. With
// `dedup_endpoints` (the `real_ping_test_all_protocols` negation), the first
// successful real ping on an endpoint retires the remaining links' real tasks
// via `scheduler.cancel_queued` + `complete` — cancelled tasks never write
// error markers. The scheduler is the single gate authority: probes run only
// for ids `schedule`/`complete` hand out, and every completion re-reads the
// link (stale snapshots are rejected by the scheduler).
//
// Batches are serialized (one at a time): the fire-handshake does not support
// two batches racing to fire promoted tasks on the same link, and the shared
// progress bar displays one batch.

/// Where a batch's plan comes from.
///
/// The "all" entry points test the FEED, not the viewport: the loaded page is
/// 200 endpoint rows, so a batch scoped to it tested 272 of the 4,523 links in
/// the 2026-09-15 feed while the user expected the whole database. Only links
/// that appear *after* the plan is built can be missed.
#[derive(Debug)]
enum PlanSource {
    /// Every link in the database.
    Feed,
    /// An explicit plan: the selected-endpoint entry points, and tests.
    Links(Vec<PlanLink>),
}

/// One link in a batch plan: the scheduler identity (link snapshot), the
/// endpoint (probe target + dedup identity), and the protocol row snapshot
/// (fast config type; real probes reload the row WITH config inside the task).
#[derive(Clone, Debug)]
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
    Failed {
        text: String,
        /// Why it failed, for the batch's per-class counters (see
        /// [`classify_fast_failure`] and [`ping_native::ProbeFailure`]).
        class: ProbeClass,
        /// The endpoint was unreachable at the transport level, so the real
        /// phase's probe cannot succeed either. Set only by the fast runner
        /// (see [`classify_fast_failure`]).
        hard: bool,
    },
}

impl ProbeOutcome {
    /// A failure the real phase must still probe (config, protocol-load and
    /// engine-capability failures; a real probe answers a different question).
    fn soft_failure(text: impl Into<String>) -> Self {
        Self::Failed {
            text: text.into(),
            class: ProbeClass::Config,
            hard: false,
        }
    }
}

/// Classify a fast probe's failure: the batch's class bucket plus whether the
/// proxy is unreachable at the transport level — the classes the real phase
/// cannot pass either, so phase 2 skips the link.
///
/// Connect-class `PingError`s are hard: a timeout, or an IO error naming a
/// refused connection, a missing route, an unreachable network, or an
/// unresolvable host. Everything else stays probed — fd exhaustion, a
/// TLS-level IO error or a protocol-level failure is local or ambiguous, and
/// the fast adapter's `NotSupported`/`Other` classes say nothing about
/// reachability. The adapter renders OS failures into `PingError::Io` text, so
/// this is the ONE site that reads that text; the class travels as a value
/// from here on.
fn classify_fast_failure(err: &xray_tui_core::ping::PingError) -> (ProbeClass, bool) {
    use xray_tui_core::ping::PingError;
    match err {
        PingError::Timeout(_) => (ProbeClass::Timeout, true),
        PingError::Io(message) => {
            let class = if message.contains("failed to lookup address information")
                || message.contains("Name or service not known")
            {
                ProbeClass::Dns
            } else if message.contains("Connection refused") {
                ProbeClass::Refused
            } else if message.contains("No route to host") {
                ProbeClass::NoRoute
            } else if message.contains("Network is unreachable") {
                ProbeClass::Unreachable
            } else {
                // Local resource failures (fd exhaustion) and ambiguous IO:
                // classified, but not proof the endpoint is unreachable.
                return (ProbeClass::Io, false);
            };
            (class, true)
        }
        PingError::NotSupported => (ProbeClass::Config, false),
        PingError::Other(_) => (ProbeClass::Other, false),
    }
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

    /// One real probe for a single link. `config` is the already-loaded
    /// [`ProtocolConfig`] — the caller gates on it before it gets here.
    fn real<'a>(
        &'a self,
        endpoint: &'a Endpoint,
        config: &'a ProtocolConfig,
        req: NativeProbeReq<'a>,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>>;
}

/// Production runner: [`xray_tui_core::FastPingManager`] for fast probes and
/// the in-process native engine ([`crate::ops::ping_native`]) for real probes.
///
/// The subprocess pool is gone: a real probe dials the native engine directly,
/// so there is no warm process to share and no `batch_active` flag to raise
/// against concurrent single pings.
struct EngineProbeRunner;

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
                Err(e) => {
                    let (class, hard) = classify_fast_failure(&e);
                    ProbeOutcome::Failed {
                        class,
                        hard,
                        text: e.to_string(),
                    }
                }
            }
        })
    }

    fn real<'a>(
        &'a self,
        endpoint: &'a Endpoint,
        config: &'a ProtocolConfig,
        req: NativeProbeReq<'a>,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
        Box::pin(async move {
            match ping_native::real_ping(endpoint, config, &req).await {
                Ok(result) => ProbeOutcome::Ok {
                    latency_ms: Some(result.latency_ms),
                    ip_info: result.ip_info,
                },
                Err(e) => ProbeOutcome::Failed {
                    text: e.text,
                    class: e.class,
                    hard: false,
                },
            }
        })
    }
}

/// Shared per-batch state, cloned into every spawned probe task.
///
/// `pub(crate)` because the quit path (`ui/mod.rs`) has to render the run's
/// summary: a batch cancelled by the runtime drop never reaches `finish_batch`,
/// which is how the 2026-09-16 run left 31k persisted results with no record of
/// the run.
pub(crate) struct BatchShared {
    sched: Arc<TaskScheduler>,
    db: Arc<Database>,
    /// Gate persistence seam: staged transitions, never a commit per call.
    writer: Arc<crate::ops::link_writer::LinkWriter>,
    tx: mpsc::Sender<CoreEvent>,
    runner: Arc<dyn BatchProbeRunner>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    /// Live meters, the same `Arc` the UI renders (no progress event).
    meters: Arc<crate::types::BatchMeters>,
    /// The batch's clock: every span in the summary is measured from here.
    started: Instant,
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
    /// Where `run_batch` publishes the batch's shared state — `AppState::batch`
    /// holds the same slot and the UI reads the summary through it.
    batch_slot: Arc<OnceLock<Arc<Self>>>,
    /// Fast config type per link (`proto_kind`), filled by the plan walk.
    fast_config: DashMap<(ProtocolId, EndpointId), i32>,
    /// Endpoint rows by id (real probes need the full endpoint). `Arc` so a
    /// probe clones the handle instead of holding a shard guard across an await.
    endpoints: DashMap<EndpointId, Arc<Endpoint>>,
    /// Links the native engine cannot serve (kind-level gate, no config load),
    /// keyed by identity and carrying the persisted marker text. The fast half
    /// still probes them; their marker is emitted right after that result.
    untestable: DashMap<(ProtocolId, EndpointId), String>,
    /// The two probe bounds (`fast_ping_concurrency`, `real_ping_concurrency`).
    /// They are independent — the levels overlap now, and each holds its own
    /// permit for the duration of its own probe.
    fast_sem: Arc<Semaphore>,
    real_sem: Arc<Semaphore>,
    // ── settle accounting ─────────────────────────────────────────────
    pending_fast: AtomicUsize,
    pending_real: AtomicUsize,
    /// Deferral retries in flight. A retry sleeps without holding a gate entry,
    /// so the two task counters cannot see it and `finish_batch` must not run
    /// while one is still waiting.
    pending_deferred: AtomicUsize,
    /// Woken on every settle: one `Notify` covers all three counters, and the
    /// waiter re-reads them.
    settled: Notify,
    /// Every spawned chain and retry, so the batch can join the lot before
    /// `finish_batch`. The counters say when work is *settled*; a chain is also
    /// running between its fast settle and its real dispatch, which no counter
    /// covers — the join is what makes the batch's end a hard boundary.
    tasks: Mutex<Vec<JoinHandle<()>>>,
    // ── rate sampling (1 Hz, for the ETA) ─────────────────────────────
    rate_fast: Mutex<(Instant, u32)>,
    rate_real: Mutex<(Instant, u32)>,
    // ── fast-probe dedup: one TCP ping per unique (address, port) ─────
    fast_dedup: Mutex<FastDedupInner>,
    /// Links whose fast probe failed for a hard unreachability reason: their
    /// real half is not probed (the probe cannot pass where the dial did not).
    hard_fast: Mutex<HashSet<(ProtocolId, EndpointId)>>,
    /// The delay the fast level measured, per link, for as long as the batch
    /// runs.
    ///
    /// The RESULT group writes `latency` and `error` together and every patch
    /// is built from the plan-time snapshot, so a real patch would carry
    /// `latency = None` and wipe the measurement the same batch had just taken
    /// (160 of 218 real failures lost their fast delay on 2026-09-15). This is
    /// the one field that must be composed back in.
    fast_latency: Mutex<HashMap<(ProtocolId, EndpointId), i32>>,
    /// The batch's counters (see [`BatchCounters`]).
    counters: BatchCounters,
    /// Wall-clock marks of the two levels, in ms since `started`. The spans
    /// overlap by construction (that is the pipeline), which the summary says.
    plan_ms: AtomicU32,
    fast_started_ms: AtomicU32,
    fast_ended_ms: AtomicU32,
    real_started_ms: AtomicU32,
    real_ended_ms: AtomicU32,
    // ── real-level endpoint dedup: endpoints whose real ping succeeded ─
    completed_endpoints: Mutex<HashSet<i64>>,
}

/// Per-batch counters, folded into ONE summary line at the end (per-result
/// lines moved to `debug` for the same reason: a batch's volume is not a log).
#[derive(Default)]
struct BatchCounters {
    fast_ok: AtomicU32,
    fast_hard_failed: AtomicU32,
    fast_soft_failed: AtomicU32,
    /// Fast-level failures by class (the reason `fast_hard_failed` /
    /// `fast_soft_failed` only count, never explain).
    fast_fail: Mutex<BTreeMap<ProbeClass, u32>>,
    real_ok: AtomicU32,
    real_failed: AtomicU32,
    /// Real-level failures by class.
    real_fail: Mutex<BTreeMap<ProbeClass, u32>>,
    untestable: AtomicU32,
    /// Real halves retired because the fast level proved them unreachable.
    unreachable: AtomicU32,
    deferred: AtomicU32,
    queue_full: AtomicU32,
}

/// Which level of a link's life a deferral retry is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Half {
    Fast,
    Real,
}

impl Half {
    /// The half's name, for the one debug line a deferral writes.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Real => "real",
        }
    }
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
    /// Gate persistence seam (staged transitions).
    writer: Arc<crate::ops::link_writer::LinkWriter>,
    tx: mpsc::Sender<CoreEvent>,
    runner: Arc<dyn BatchProbeRunner>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    meters: Arc<crate::types::BatchMeters>,
    plan: PlanSource,
    real_phase: bool,
    dedup_endpoints: bool,
    fast_timeout: Duration,
    real_timeout: Duration,
    real_retries: u32,
    ping_url: String,
    ip_api_url: String,
    defer_delay: Duration,
    real_concurrency: usize,
    fast_concurrency: usize,
    /// Endpoints per plan page. Production uses [`PROFILES_PAGE_SIZE`]; tests
    /// drive the streaming walk with tiny pages.
    page_size: usize,
    /// "Clear error after" (design §6.4): `None` = never sweep.
    error_ttl_hours: Option<i64>,
    /// Where `run_batch` publishes the batch's shared state as soon as it is
    /// built — `AppState::batch` holds the same slot and the UI reads the
    /// summary through it.
    batch_slot: Arc<OnceLock<Arc<BatchShared>>>,
}

impl BatchShared {
    /// Build the shared state.
    ///
    /// The plan is deliberately NOT here: the walk fills `fast_config`,
    /// `endpoints`, `untestable` and the meters page by page, which is what lets
    /// the first probe start within one page instead of after the whole feed.
    fn new(p: BatchParams) -> Self {
        Self {
            sched: p.scheduler,
            db: p.db.clone(),
            writer: p.writer,
            tx: p.tx,
            runner: p.runner,
            stop: p.stop,
            meters: p.meters,
            started: Instant::now(),
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
            batch_slot: p.batch_slot,
            fast_config: DashMap::new(),
            endpoints: DashMap::new(),
            untestable: DashMap::new(),
            fast_sem: Arc::new(Semaphore::new(p.fast_concurrency.max(1))),
            real_sem: Arc::new(Semaphore::new(p.real_concurrency.max(1))),
            pending_fast: AtomicUsize::new(0),
            pending_real: AtomicUsize::new(0),
            pending_deferred: AtomicUsize::new(0),
            settled: Notify::new(),
            tasks: Mutex::new(Vec::new()),
            rate_fast: Mutex::new((Instant::now(), 0)),
            rate_real: Mutex::new((Instant::now(), 0)),
            fast_dedup: Mutex::new(FastDedupInner {
                cache: HashMap::new(),
                in_flight: HashMap::new(),
            }),
            hard_fast: Mutex::new(HashSet::new()),
            fast_latency: Mutex::new(HashMap::new()),
            counters: BatchCounters::default(),
            plan_ms: AtomicU32::new(0),
            fast_started_ms: AtomicU32::new(0),
            fast_ended_ms: AtomicU32::new(0),
            real_started_ms: AtomicU32::new(0),
            real_ended_ms: AtomicU32::new(0),
            completed_endpoints: Mutex::new(HashSet::new()),
        }
    }

    /// Links planned so far — the walk increments it with each dispatched link.
    fn plan_len(&self) -> u32 {
        self.meters.fast.total.load(Ordering::Relaxed)
    }

    /// Spawn a task the batch must outlive (chains and deferral retries).
    fn spawn_tracked(self: &Arc<Self>, fut: impl Future<Output = ()> + Send + 'static) {
        let handle = tokio::spawn(fut);
        self.tasks.lock().push(handle);
    }

    /// Record the first start of a level, in ms since the batch began (the
    /// first writer wins: `0` means "not started yet").
    fn mark_started(&self, at: &AtomicU32) {
        let ms = u32::try_from(self.started.elapsed().as_millis()).unwrap_or(u32::MAX);
        let _ = at.compare_exchange(0, ms, Ordering::Relaxed, Ordering::Relaxed);
    }

    /// Record a level's latest settle.
    fn mark_settled(&self, at: &AtomicU32) {
        at.store(
            u32::try_from(self.started.elapsed().as_millis()).unwrap_or(u32::MAX),
            Ordering::Relaxed,
        );
    }

    /// One settle on a level: `done += 1`, plus a rate sample at most once a
    /// second (`results/s × 1000`, the render path's ETA input).
    ///
    /// The sample is taken on the settle that crosses the second boundary — no
    /// timer, no thread, and no per-result event.
    fn bump(phase: &crate::types::PhaseMeters, slot: &Mutex<(Instant, u32)>) {
        let done = phase.done.fetch_add(1, Ordering::Relaxed) + 1;
        let mut last = slot.lock();
        if done == 1 {
            // First result of this level: time the window from HERE. A slot
            // created with the batch would include the idle stretch before the
            // level's first settle — the live run of the fast+real pipeline (46
            // real results in 8 s) sampled 1 result over ~5 s of that idle
            // stretch, stored 0/s, and pinned the real ETA at `--` because no
            // later settle crossed a fresh second boundary.
            *last = (Instant::now(), done);
            return;
        }
        let elapsed = last.0.elapsed();
        if elapsed < Duration::from_secs(1) {
            return;
        }
        let elapsed_ms = u64::try_from(elapsed.as_millis())
            .unwrap_or(u64::MAX)
            .max(1);
        let delta = u64::from(done.saturating_sub(last.1));
        // `rate_milli` is results/s × 1000 (`PhaseMeters::eta_secs` divides by
        // it), so a delta over `elapsed_ms` scales by 1e6, not 1e3.
        phase.rate_milli.store(
            u32::try_from(delta * 1_000_000 / elapsed_ms).unwrap_or(u32::MAX),
            Ordering::Relaxed,
        );
        *last = (Instant::now(), done);
    }
}

/// Run one batch to completion. Spawned by the entry points; awaited directly
/// by the tests.
///
/// The plan is a page stream: each page is dispatched as it loads, so the first
/// probe starts within a page instead of after the whole feed (resolving every
/// page first cost ~5.8 s of dead air on the 2026-09-17 reference feed), and
/// every link's real half is dispatched by its own chain once that link's fast
/// half has settled — there is no phase barrier and no gate queue on the path.
pub(crate) async fn run_batch(mut params: BatchParams) {
    let source = std::mem::replace(&mut params.plan, PlanSource::Links(Vec::new()));
    let page_size = params.page_size.max(1);
    let mut walk = PlanWalk::new(source, params.db.clone(), page_size);
    let shared = Arc::new(BatchShared::new(params));
    // Publish the handle before the first probe: a batch's record has to exist
    // for as long as the batch can be interrupted, and the shared state
    // (counters + class histograms + meters) is what the quit path renders.
    let _ = shared.batch_slot.set(Arc::clone(&shared));

    // ── Walk and dispatch: the plan is consumed page by page ──────────
    let walk_started = Instant::now();
    let mut walk_error: Option<xray_tui_db::DatabaseError> = None;
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            // Stop at the dispatch boundary: the rest of the plan is never
            // scheduled, so it never writes anything.
            break;
        }
        match walk.next_page().await {
            Ok(Some(links)) => {
                shared
                    .meters
                    .plan_pages_total
                    .store(walk.pages_total(), Ordering::Relaxed);
                shared.dispatch_page(links).await;
                shared
                    .meters
                    .plan_pages_done
                    .store(walk.pages_done(), Ordering::Relaxed);
            }
            Ok(None) => break,
            Err(e) => {
                // The pages already dispatched are probing: keep them and finish
                // normally rather than discarding visible results for a plan
                // error the summary reports.
                walk_error = Some(e);
                break;
            }
        }
    }
    shared.plan_ms.store(
        u32::try_from(walk_started.elapsed().as_millis()).unwrap_or(u32::MAX),
        Ordering::Relaxed,
    );
    tracing::info!(
        target: "tui::ops::ping",
        "{}",
        plan_line(
            shared.plan_len(),
            walk.pages_done(),
            shared.plan_ms.load(Ordering::Relaxed),
        ),
    );
    if let Some(e) = &walk_error {
        tracing::warn!(
            target: "tui::ops::ping",
            "batch: plan walk stopped after {} page(s): {e}",
            walk.pages_done(),
        );
    }
    if shared.plan_len() == 0 && !shared.stop.load(Ordering::Relaxed) {
        // Nothing was planned at all (an empty feed, or a walk that failed on
        // its first page): no work exists to flush or sweep, so the batch ends
        // with the terminal event alone. A stop before the first page is NOT
        // this case — the feed may hold thousands of links the run never
        // touched, and those are exactly the rows `finish_batch`'s error sweep
        // exists for.
        tracing::warn!(target: "tui::ops::ping", "batch: no links to test");
        let _ = shared.tx.try_send(CoreEvent::BatchEnded);
        return;
    }
    if shared.real_phase {
        warn_if_real_phase_is_slow(shared.real_concurrency, shared.plan_len() as usize);
    }

    // Wait for every chain and deferral retry to settle, then join them: the
    // counters say when work is settled, the join makes the batch's end a hard
    // boundary (a chain also runs between its fast settle and its real
    // dispatch, which no counter covers).
    loop {
        // Register the waiter BEFORE reading the counters: `notify_waiters`
        // only wakes already-registered waiters, so a task that settles in
        // between would otherwise be a lost wakeup and this loop would park
        // forever.
        let settled = shared.settled.notified();
        tokio::pin!(settled);
        settled.as_mut().enable();
        if shared.pending_fast.load(Ordering::Relaxed) == 0
            && shared.pending_real.load(Ordering::Relaxed) == 0
            && shared.pending_deferred.load(Ordering::Relaxed) == 0
        {
            break;
        }
        settled.await;
    }
    let tasks = std::mem::take(&mut *shared.tasks.lock());
    for task in tasks {
        let _ = task.await;
    }
    finish_batch(&shared).await;
}

/// The plan as a page stream.
///
/// A batch dispatches each page as it loads, so the first probe starts within a
/// page of the walk instead of after the whole feed: the loader used to resolve
/// every page first, and on the 2026-09-17 reference feed (18,334 endpoints,
/// ~92 pages × the measured 63 ms per page) that was ~5.8 s of dead air before
/// the first socket dial. It also holds no `Vec<PlanLink>` for the whole feed —
/// only the per-link maps the probes read.
enum PlanWalk {
    /// An explicit plan (the selected-endpoint entry points and the tests): one
    /// page.
    Links(std::vec::IntoIter<Vec<PlanLink>>),
    /// The whole feed, walked page by page through the tab's own query.
    Feed {
        db: Arc<Database>,
        page_size: usize,
        offset: usize,
        pages_done: u32,
        pages_total: u32,
        exhausted: bool,
    },
}

impl PlanWalk {
    fn new(source: PlanSource, db: Arc<Database>, page_size: usize) -> Self {
        match source {
            PlanSource::Links(links) => Self::Links(vec![links].into_iter()),
            PlanSource::Feed => Self::Feed {
                db,
                page_size,
                offset: 0,
                pages_done: 0,
                pages_total: 0,
                exhausted: false,
            },
        }
    }

    const fn pages_done(&self) -> u32 {
        match self {
            Self::Links(_) => 1,
            Self::Feed { pages_done, .. } => *pages_done,
        }
    }

    const fn pages_total(&self) -> u32 {
        match self {
            Self::Links(_) => 1,
            Self::Feed { pages_total, .. } => *pages_total,
        }
    }

    /// The next page of links, or `None` when the walk is exhausted. The first
    /// feed page also carries the feed-wide endpoint count (one `COUNT` for the
    /// whole walk, not one per page).
    async fn next_page(&mut self) -> Result<Option<Vec<PlanLink>>, xray_tui_db::DatabaseError> {
        match self {
            Self::Links(iter) => Ok(iter.next()),
            Self::Feed {
                db,
                page_size,
                offset,
                pages_done,
                pages_total,
                exhausted,
            } => {
                if *exhausted {
                    return Ok(None);
                }
                // `PurgatoryView::All` and no search/group: the run is about the
                // whole database, not about what the tab currently filters to.
                // The thresholds are unused for that view.
                let request = PageRequest {
                    view: PurgatoryView::All,
                    active_threshold: 0,
                    stale_threshold: 0,
                    search: None,
                    group_id: None,
                    sort: PageSort::Address,
                    ascending: true,
                    offset: *offset,
                    limit: *page_size,
                };
                let (ids, total) = db.profiles_walk_page(&request, *offset == 0).await?;
                if let Some(total) = total {
                    let pages = total.div_ceil(u64::try_from(*page_size).unwrap_or(u64::MAX));
                    *pages_total = u32::try_from(pages).unwrap_or(u32::MAX);
                }
                if ids.is_empty() {
                    *exhausted = true;
                    return Ok(None);
                }
                *offset += ids.len();
                *pages_done += 1;
                let rows = db.load_page_projection(&ids).await?;
                Ok(Some(rows.iter().flat_map(plan_row_links).collect()))
            }
        }
    }
}

/// Attempts for the batch-end flush. Contention with a concurrent import is
/// transient (the import commits in 500-link chunks) and the driver already
/// waits `busy_timeout` (5 s) inside each attempt — so this is a short bounded
/// retry, not a stacked backoff. A dedicated failure-injection test is not
/// written: every injected failure costs the driver's full busy wait (measured
/// in `link_writer`'s lock test), and the retry only decides how many times the
/// same single call is repeated.
const FINAL_FLUSH_ATTEMPTS: u32 = 3;

/// Signal the batch's end. Runs the error-TTL sweep first (design §6.4): batch
/// completion is a natural "errors are fresh now" boundary, so persisted failure
/// markers older than the configured TTL are cleared before the terminal event
/// lands. Links the batch did not touch (dedup-retired siblings, queue-full/stop
/// skips) are exactly the ones whose stale markers this clears.
async fn finish_batch(shared: &BatchShared) {
    // Make the batch durable before the sweeps look at the rows: the staged
    // result/task writes land in one transaction here instead of one commit
    // per result on the UI task.
    //
    // Retried: this is the batch's durability point, and the background flush
    // loop may not outlive it. A failed attempt re-stages the whole unwritten
    // remainder (see `LinkWriter::flush`), so a later attempt writes it all.
    let mut flush_error = None;
    for attempt in 0..FINAL_FLUSH_ATTEMPTS {
        match shared.writer.flush().await {
            Ok(_) => {
                flush_error = None;
                break;
            }
            Err(e) => {
                flush_error = Some(e);
                tokio::time::sleep(std::time::Duration::from_millis(50 << attempt.min(3))).await;
            }
        }
    }
    if let Some(e) = flush_error {
        tracing::warn!(
            target: "tui::ops::ping",
            "batch flush failed after {FINAL_FLUSH_ATTEMPTS} attempts: {e}"
        );
    }
    // Bound WAL growth: one checkpoint after the write burst, not per commit.
    // Bounded by a timeout — an in-memory database has no WAL to checkpoint
    // and must never stall the batch.
    if let Ok(Ok(mut conn)) =
        tokio::time::timeout(std::time::Duration::from_secs(2), shared.db.connection()).await
    {
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            // PASSIVE, not TRUNCATE: TRUNCATE blocks until every reader has
            // released its WAL read mark, and the pool keeps connections
            // alive — a busy checkpoint would stall the batch instead of
            // merely leaving the WAL larger.
            toasty::sql::query("PRAGMA wal_checkpoint(PASSIVE)").exec(&mut conn),
        )
        .await;
    }
    crate::ops::profiles::clear_expired_errors(&shared.db, shared.error_ttl_hours).await;
    // One line per batch: the per-result lines are `debug`, so this is the
    // record a reader (and the next investigation) works from.
    tracing::info!(target: "tui::ops::ping", "{}", summary_line(shared));
    let _ = shared.tx.try_send(CoreEvent::BatchEnded);
}

/// The plan's own record: the pages and links the walk produced and how long it
/// took. Emitted once, after the walk (a stop or a page error ends it early, and
/// the line then reports the partial plan — which is why the counts are here and
/// not derived from the feed).
fn plan_line(links: u32, pages: u32, plan_ms: u32) -> String {
    format!("batch: planned {links} link(s) over {pages} page(s) in {plan_ms} ms")
}

/// The batch's single log record: the planned links and pages, the per-level
/// outcomes with their class histograms, the two overlapping level spans (the
/// pipeline runs both levels at once, so they are not phases) and the writer's
/// own durability answer (`staged-left` is non-zero only if a flush never
/// succeeded).
pub(crate) fn summary_line(shared: &BatchShared) -> String {
    let counters = &shared.counters;
    let load = |counter: &AtomicU32| counter.load(Ordering::Relaxed);
    format!(
        "batch summary: links={} plan={} ms untestable={} queue-full={} deferred={} | fast ok={} hard-fail={} soft-fail={} {} ({}..{} ms) | real ok={} failed={} {} skipped-unreachable={} ({}..{} ms) | levels overlap | wall={} ms stopped={} flushes={} staged-left={}",
        shared.plan_len(),
        shared.plan_ms.load(Ordering::Relaxed),
        load(&counters.untestable),
        load(&counters.queue_full),
        load(&counters.deferred),
        load(&counters.fast_ok),
        load(&counters.fast_hard_failed),
        load(&counters.fast_soft_failed),
        class_histogram(&counters.fast_fail),
        shared.fast_started_ms.load(Ordering::Relaxed),
        shared.fast_ended_ms.load(Ordering::Relaxed),
        load(&counters.real_ok),
        load(&counters.real_failed),
        class_histogram(&counters.real_fail),
        load(&counters.unreachable),
        shared.real_started_ms.load(Ordering::Relaxed),
        shared.real_ended_ms.load(Ordering::Relaxed),
        shared.started.elapsed().as_millis(),
        shared.stop.load(Ordering::Relaxed),
        shared.writer.flush_count(),
        shared.writer.staged_len(),
    )
}

/// Count one failure into its class bucket.
fn bump_class(map: &Mutex<BTreeMap<ProbeClass, u32>>, class: ProbeClass) {
    *map.lock().entry(class).or_default() += 1;
}

/// Render a failure histogram as `[timeout=8924 dns=4119 …]`, in class order.
/// Empty when nothing failed — the reason `hard-fail=` / `failed=` alone is not
/// enough: a run's dominant class is what a fix is aimed at.
fn class_histogram(map: &Mutex<BTreeMap<ProbeClass, u32>>) -> String {
    // Snapshot first: the guard is released before any string work, so a
    // probe task can never block on the summary's formatting.
    let counts: Vec<(ProbeClass, u32)> = map.lock().iter().map(|(c, n)| (*c, *n)).collect();
    if counts.is_empty() {
        return "[]".to_string();
    }
    let mut out = String::from("[");
    for (class, count) in counts {
        if out.len() > 1 {
            out.push(' ');
        }
        out.push_str(class.label());
        out.push('=');
        out.push_str(&count.to_string());
    }
    out.push(']');
    out
}

/// The record of a batch that never reached its own summary — the quit path
/// renders this after flushing, because the runtime drop cancels the batch task
/// (`finish_batch`'s line is how the 2026-09-16 run ended with 31k persisted
/// results and nothing in the log describing the run).
///
/// This is the SAME summary a completed run writes, so the per-class
/// histograms survive an interruption, plus the two counts `finish_batch` does
/// not need: what the levels had settled, and what was still in flight (task
/// slots plus the deferral retries, which hold no gate entry).
pub(crate) fn interrupted_summary_line(shared: &BatchShared) -> String {
    let settled = shared.meters.fast.done.load(Ordering::Relaxed)
        + shared.meters.real.done.load(Ordering::Relaxed);
    let in_flight = shared.pending_fast.load(Ordering::Relaxed)
        + shared.pending_real.load(Ordering::Relaxed)
        + shared.pending_deferred.load(Ordering::Relaxed);
    format!(
        "batch interrupted at quit: {} | settled={settled} in-flight={in_flight}",
        summary_line(shared),
    )
}

/// Warn once per real-phase batch whose concurrency is below
/// [`REAL_CONCURRENCY_WARN_BELOW`]: a real probe holds its slot for a dial plus
/// (on failure) the whole timeout, so the phase's rate is roughly
/// `concurrency / timeout` — the measured run held a 34,562-link plan at 2.45
/// results/s with a saved override of 5 (default [`REAL_CONCURRENCY_DEFAULT`]),
/// which is ~4 h. The value is the user's to set; this only says what it costs.
fn warn_if_real_phase_is_slow(real_concurrency: usize, plan_len: usize) {
    if real_concurrency >= REAL_CONCURRENCY_WARN_BELOW as usize {
        return;
    }
    tracing::warn!(
        target: "tui::ops::ping",
        "real ping concurrency {real_concurrency} is below {REAL_CONCURRENCY_WARN_BELOW} (code default {REAL_CONCURRENCY_DEFAULT}): a real probe holds its slot for a dial plus the full timeout, so the real phase is throughput-bound (~2.45 results/s at 5 on the 2026-09-16 run) and this {plan_len}-link plan will not finish in a session — raise speed_test.real_ping_concurrency"
    );
}

/// One startup line naming the values a batch's behaviour depends on: the
/// analysis of the 2026-09-15 run had to read them out of `config.json`, and
/// the loaded page size is what the plan is bounded by.
pub fn log_startup_envelope(state: &AppState) {
    let speed = &state.config.speed_test;
    tracing::info!(
        target: "tui::ops::ping",
        "startup: version={} page={} rows tcp-timeout={:?} real-timeout={:?} real-retries={} real-concurrency={} fast-concurrency={} dedup-endpoints={} error-ttl={:?}",
        env!("CARGO_PKG_VERSION"),
        state.endpoints.len(),
        speed.tcp_timeout_secs,
        speed.real_ping_timeout_secs,
        speed.real_ping_retries,
        speed.real_ping_concurrency,
        speed.fast_ping_concurrency,
        !speed.real_ping_test_all_protocols,
        speed.error_ttl_hours,
    );
}

/// Drive one link's chain: its fast half, then the real half that half's settle
/// dispatches (real-phase batches). The gate keeps a link's halves from
/// overlapping; this chain, not a queue, hands the link from one to the other.
///
/// A stop at a dispatch boundary retires the task silently — no result event, no
/// error marker.
async fn run_task_chain(
    shared: Arc<BatchShared>,
    link: ProfileStats,
    mut id: u16,
    mut kind: TaskKind,
) {
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            shared.sched.complete(&link, id, kind).await;
            shared.note_settled(kind);
            // The real half of a stopped link is never dispatched, so it is never
            // counted: `after_fast_settle` owns the accounting, and a link that
            // never reached it has nothing outstanding.
            return;
        }
        match kind {
            TaskKind::FastPing => {
                // The fast level's own bound (`fast_ping_concurrency`): the batch
                // used to spawn one probe future per link with no global cap.
                let Ok(permit) = Arc::clone(&shared.fast_sem).acquire_owned().await else {
                    return;
                };
                shared.mark_started(&shared.fast_started_ms);
                let outcome = shared.fast_probe(&link).await;
                shared.emit_result(&link, TestType::TcpPing, &outcome);
                shared.sched.complete(&link, id, TaskKind::FastPing).await;
                shared.note_settled(TaskKind::FastPing);
                drop(permit);
                // The fast result is staged: the link's real half (and its
                // untestable marker, for either batch kind) follows here.
                shared.after_fast_settle(&link).await;
            }
            TaskKind::RealPing => {
                // Defensive: the batch never queues, so this arm is reached only
                // when another chain held this link's gate and promoted our id.
                shared.mark_started(&shared.real_started_ms);
                shared.run_real_task(&link, id).await;
            }
            _ => return, // SpeedTest/UdpTest tasks are not part of the batch
        }
        // Fire the promoted task, if any. The gate owns task state, so ask it for
        // the link's new current id (an id the registry does not know cannot come
        // back: the gate and the registry advance together).
        let Some(next_id) = shared.sched.task_of(&link) else {
            return;
        };
        if next_id == id {
            // The gate did not advance (stale completion) — do not spin.
            return;
        }
        let Some(next_kind) = shared.sched.kind_of(next_id) else {
            return;
        };
        id = next_id;
        kind = next_kind;
    }
}

impl BatchShared {
    /// Schedule one page's links. The maps the probes read are filled here, so a
    /// link's own halves find their config and endpoint without a second pass
    /// over the whole plan.
    async fn dispatch_page(self: &Arc<Self>, links: Vec<PlanLink>) {
        for plan in links {
            let key = (plan.link.protocol_id, plan.link.endpoint_id);
            self.fast_config
                .insert(key, plan.protocol.proto_kind.to_i32());
            self.endpoints
                .insert(plan.endpoint.id, Arc::new(plan.endpoint.clone()));
            // The kind-level testability gate: it needs only the in-memory
            // `proto_kind`, so it is decided here — the one place every batch
            // (production and test) passes through. The config-aware half runs
            // inside the real probe, the only place a loaded config exists.
            if !capability::kind_supported(plan.protocol.proto_kind) {
                self.untestable.insert(
                    key,
                    untestable_marker_text(capability::KIND_UNSUPPORTED_REASON),
                );
            }
            self.meters.fast.total.fetch_add(1, Ordering::Relaxed);
            let link = plan.link;
            if !self.dispatch_fast_link(link.clone()).await {
                self.spawn_defer_retry(link, Half::Fast);
            }
        }
    }

    /// Schedule one link's fast half.
    ///
    /// Returns `false` only for `DnsDeferred`, and the caller owns the retry —
    /// one retry task per deferred half, looping inside itself. `Queued` is the
    /// defensive case (another chain holds this link's gate and its `complete`
    /// promotes ours): the batch itself never queues, because a queued real half
    /// would depend on `task_queue_limit` and a `DnsDeferred` link has no gate
    /// entry to queue behind (`schedule` answers before the lock).
    async fn dispatch_fast_link(self: &Arc<Self>, link: ProfileStats) -> bool {
        match self.sched.schedule(&link, TaskKind::FastPing).await {
            ScheduleOutcome::Started(id) => {
                self.pending_fast.fetch_add(1, Ordering::Relaxed);
                let shared = Arc::clone(self);
                self.spawn_tracked(async move {
                    run_task_chain(shared, link, id, TaskKind::FastPing).await;
                });
                true
            }
            ScheduleOutcome::Queued(_) => {
                self.pending_fast.fetch_add(1, Ordering::Relaxed);
                true
            }
            ScheduleOutcome::DnsDeferred => {
                self.counters.deferred.fetch_add(1, Ordering::Relaxed);
                false
            }
            ScheduleOutcome::QueueFull => {
                self.counters.queue_full.fetch_add(1, Ordering::Relaxed);
                // Per-link: a feed-wide batch would flood the log. The summary
                // carries the count.
                tracing::debug!(target: "tui::ops::ping", "batch: link skipped, queue full");
                true
            }
        }
    }

    /// Everything a link owes once its fast result is staged.
    ///
    /// The untestable marker comes first and applies to BOTH batch kinds: the
    /// single pass this replaces ran before the fast-only early return, so a
    /// fast-only batch marks such rows today — dropping the marker would leave a
    /// genuine-looking failure behind and `remove_failed_servers` would delete
    /// the row (the marker outranks a measurement in the Test cell). A marked
    /// link is never a real candidate.
    async fn after_fast_settle(self: &Arc<Self>, link: &ProfileStats) {
        let key = (link.protocol_id, link.endpoint_id);
        let untestable = self.untestable.get(&key).map(|r| r.value().clone());
        if let Some(reason) = untestable {
            self.counters.untestable.fetch_add(1, Ordering::Relaxed);
            self.emit_untestable_marker(link, &reason);
            return;
        }
        if !self.real_phase {
            return;
        }
        // The fast level already proved this link's proxy unreachable (refused,
        // no route, unresolvable, dial timeout): the real probe would only spend
        // its whole timeout re-learning that. The row keeps its `[fast]` marker,
        // which is the honest statement about it.
        if self.hard_fast.lock().contains(&key) {
            self.counters.unreachable.fetch_add(1, Ordering::Relaxed);
            return;
        }
        // A candidate is known: it enters the real level's denominator, and
        // every path out of `dispatch_real_probe` counts it exactly once into
        // `real.done` (probe, sibling retire, stop retire, queue-full).
        self.meters.real.total.fetch_add(1, Ordering::Relaxed);
        if !self.dispatch_real_probe(link.clone(), true).await {
            self.spawn_defer_retry(link.clone(), Half::Real);
        }
    }

    /// Dispatch a link's real half.
    ///
    /// The permit is taken BEFORE the sibling check on purpose: for an endpoint
    /// whose links settle together, the probes `dedup_endpoints` saves are the
    /// ones still waiting for capacity when a sibling's real ping succeeded.
    ///
    /// `counted` says this half's candidate is already in `real.total`; a
    /// deferral retry passes `false` so a retried half is never counted twice.
    /// Returns `false` only for `DnsDeferred` (the caller re-enters later).
    async fn dispatch_real_probe(self: &Arc<Self>, link: ProfileStats, counted: bool) -> bool {
        let Ok(permit) = Arc::clone(&self.real_sem).acquire_owned().await else {
            return true;
        };
        if self.stop.load(Ordering::Relaxed) {
            if counted {
                self.note_real_done();
            }
            return true;
        }
        if self.dedup_endpoints
            && self
                .completed_endpoints
                .lock()
                .contains(&link.endpoint_id.get())
        {
            // A sibling already succeeded: retire this half without a probe and
            // without a result event, so no marker is written.
            if counted {
                self.note_real_done();
            }
            return true;
        }
        match self.sched.schedule(&link, TaskKind::RealPing).await {
            ScheduleOutcome::Started(id) => {
                self.pending_real.fetch_add(1, Ordering::Relaxed);
                self.mark_started(&self.real_started_ms);
                self.run_real_task(&link, id).await;
                drop(permit);
                true
            }
            ScheduleOutcome::Queued(_) => {
                // Promoted by the gate holder's completion (defensive).
                self.pending_real.fetch_add(1, Ordering::Relaxed);
                true
            }
            ScheduleOutcome::DnsDeferred => {
                self.counters.deferred.fetch_add(1, Ordering::Relaxed);
                drop(permit);
                false
            }
            ScheduleOutcome::QueueFull => {
                self.counters.queue_full.fetch_add(1, Ordering::Relaxed);
                if counted {
                    self.note_real_done();
                }
                true
            }
        }
    }

    /// One real probe, from schedule to settle.
    async fn run_real_task(&self, link: &ProfileStats, id: u16) {
        let outcome = self.real_probe(link).await;
        // A successful real ping records the endpoint so the sibling-dedup pass
        // skips its remaining links.
        if self.dedup_endpoints && matches!(outcome, ProbeOutcome::Ok { .. }) {
            self.completed_endpoints
                .lock()
                .insert(link.endpoint_id.get());
        }
        self.emit_result(link, TestType::RealPing, &outcome);
        self.sched.complete(link, id, TaskKind::RealPing).await;
        self.note_settled(TaskKind::RealPing);
        self.note_real_done();
    }

    /// Spawn the one retry task a deferred half gets.
    fn spawn_defer_retry(self: &Arc<Self>, link: ProfileStats, half: Half) {
        self.pending_deferred.fetch_add(1, Ordering::Relaxed);
        let shared = Arc::clone(self);
        self.spawn_tracked(async move {
            shared.defer_retry(link, half).await;
        });
    }

    /// A deferred half sleeps out the window and re-enters its own dispatch.
    ///
    /// It holds no gate entry: `DnsDeferred` is answered before the gate is
    /// touched, so there is nothing to promote — and that is exactly why a
    /// link's two halves cannot separate, since the fast half's re-entry runs
    /// the whole chain and the real half's re-entry is reachable only from a
    /// settled fast half.
    async fn defer_retry(self: &Arc<Self>, link: ProfileStats, half: Half) {
        loop {
            tokio::time::sleep(self.defer_delay).await;
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            let reached_gate = match half {
                Half::Fast => self.dispatch_fast_link(link.clone()).await,
                Half::Real => self.dispatch_real_probe(link.clone(), false).await,
            };
            if reached_gate {
                break;
            }
            // Still deferred: the window is whole seconds, so wait it out again
            // rather than spinning on the gate.
            tracing::debug!(
                target: "tui::ops::ping",
                "batch: {} half still DNS-deferred",
                half.as_str(),
            );
            tokio::time::sleep(self.defer_delay.min(Duration::from_millis(250))).await;
        }
        if self.pending_deferred.fetch_sub(1, Ordering::Relaxed) == 1 {
            self.settled.notify_waiters();
        }
    }

    /// Fast probe with batch-level dedup: one TCP ping per unique
    /// (address, port); followers await the owner's result and reuse it.
    async fn fast_probe(&self, link: &ProfileStats) -> ProbeOutcome {
        // Clone out of the map: a `DashMap` guard is not `Send`, so it must not
        // be held across the probe's awaits.
        let Some(endpoint) = self
            .endpoints
            .get(&link.endpoint_id)
            .map(|e| Arc::clone(e.value()))
        else {
            return ProbeOutcome::soft_failure("Endpoint not found for fast ping");
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
            .map_or(0, |v| *v.value());
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
                .unwrap_or_else(|| ProbeOutcome::soft_failure("fast ping dedup: owner result lost"))
        }
    }

    /// Real probe for one link: reload the protocol row WITH its deferred
    /// config (the builders refuse unloaded configs — mirrors
    /// `start_real_ping`), then run through the pooled core.
    async fn real_probe(&self, link: &ProfileStats) -> ProbeOutcome {
        let protocol = match load_protocol_with_config(&self.db, link.protocol_id).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                return ProbeOutcome::soft_failure("Protocol row not found for real ping");
            }
            Err(e) => {
                return ProbeOutcome::soft_failure(format!("Failed to load protocol: {e}"));
            }
        };
        let config = protocol.config.get().0.clone();
        // Config-level capability gate: kind-level refusals were already retired
        // at plan time, so only a native-capable kind whose CONFIG the engine
        // refuses reaches here. Its marker IS the probe's result — phase 2, so
        // it lands after the fast result and is not cleared by it.
        if let Some(reason) = capability::support_reason(protocol.proto_kind, &config) {
            return ProbeOutcome::soft_failure(untestable_marker_text(reason));
        }
        let Some(endpoint) = self
            .endpoints
            .get(&link.endpoint_id)
            .map(|e| Arc::clone(e.value()))
        else {
            return ProbeOutcome::soft_failure("Endpoint not found for real ping");
        };
        self.runner
            .real(
                &endpoint,
                &config,
                NativeProbeReq {
                    ping_url: &self.ping_url,
                    ip_api_url: &self.ip_api_url,
                    timeout: self.real_timeout,
                    retries: self.real_retries,
                },
            )
            .await
    }

    /// Stage one link's RESULT columns from a probe outcome.
    ///
    /// The batch owns its link snapshots, so persistence does not depend on the
    /// UI's loaded page: the events handler applies results to `state.endpoints`
    /// and stages only what it finds there, and a page reload mid-batch (an
    /// import moves every endpoint's ordering keys) used to drop those results
    /// silently. Staging here is what makes "emitted == persisted" hold.
    fn stage_result(&self, link: &ProfileStats, test_type: TestType, outcome: &ProbeOutcome) {
        let (latency_ms, ip_info, error) = match outcome {
            ProbeOutcome::Ok {
                latency_ms,
                ip_info,
            } => (*latency_ms, ip_info.as_deref(), None),
            ProbeOutcome::Failed { text, .. } => (None, None, Some(text.as_str())),
        };
        let mut row = link.clone();
        if let Some(delay) = self
            .fast_latency
            .lock()
            .get(&(link.protocol_id, link.endpoint_id))
        {
            row.latency = Some(Latency::Fast { delay: *delay });
        }
        if crate::ops::events::apply_test_result(
            &mut row, test_type, latency_ms, None, ip_info, error,
        ) {
            self.writer.stage(&row, LinkGroups::RESULT);
        }
    }

    /// Send the `SpeedTestResult` event for a completed probe. The
    /// `TestTypeUpdate` re-arms the events handler's per-protocol dedupe guard
    /// (`testing_profiles`) before the result lands — the real half re-arms for
    /// its own result the same way.
    fn emit_result(&self, link: &ProfileStats, test_type: TestType, outcome: &ProbeOutcome) {
        // Persist first: the event below is a UI notification and may be
        // dropped when the channel is full (`try_send`), the write may not.
        self.stage_result(link, test_type, outcome);
        // The real half reads this: a hard fast failure means the proxy never
        // answered, and a real probe can only spend its timeout finding that out
        // again (253 of 350 real probes did exactly that, 2026-09-15).
        match (test_type, outcome) {
            (
                TestType::TcpPing,
                ProbeOutcome::Ok {
                    latency_ms: Some(delay),
                    ..
                },
            ) => {
                self.counters.fast_ok.fetch_add(1, Ordering::Relaxed);
                self.fast_latency.lock().insert(
                    (link.protocol_id, link.endpoint_id),
                    i32::try_from(*delay).unwrap_or(i32::MAX),
                );
            }
            (
                TestType::TcpPing,
                ProbeOutcome::Ok {
                    latency_ms: None, ..
                },
            ) => {
                self.counters.fast_ok.fetch_add(1, Ordering::Relaxed);
            }
            (TestType::TcpPing, ProbeOutcome::Failed { class, hard, .. }) => {
                if *hard {
                    self.counters
                        .fast_hard_failed
                        .fetch_add(1, Ordering::Relaxed);
                    self.hard_fast
                        .lock()
                        .insert((link.protocol_id, link.endpoint_id));
                } else {
                    self.counters
                        .fast_soft_failed
                        .fetch_add(1, Ordering::Relaxed);
                }
                bump_class(&self.counters.fast_fail, *class);
            }
            (_, ProbeOutcome::Ok { .. }) => {
                self.counters.real_ok.fetch_add(1, Ordering::Relaxed);
            }
            (_, ProbeOutcome::Failed { class, .. }) => {
                self.counters.real_failed.fetch_add(1, Ordering::Relaxed);
                bump_class(&self.counters.real_fail, *class);
            }
        }
        let (latency_ms, ip_info, error) = match outcome {
            ProbeOutcome::Ok {
                latency_ms,
                ip_info,
            } => (*latency_ms, ip_info.clone(), None),
            ProbeOutcome::Failed { text, .. } => (None, None, Some(text.clone())),
        };
        let endpoint_id = link.endpoint_id.get();
        let _ = self.tx.try_send(CoreEvent::TestTypeUpdate {
            endpoint_id,
            protocol_id: link.protocol_id.get(),
            test_type,
        });
        try_send_or_warn(
            &self.tx,
            CoreEvent::SpeedTestResult {
                endpoint_id,
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

    /// One task settled: the level's `done` advances (with a rate sample for the
    /// ETA) and the waiter is woken when nothing of that level is outstanding.
    fn note_settled(&self, kind: TaskKind) {
        match kind {
            TaskKind::FastPing => {
                self.mark_settled(&self.fast_ended_ms);
                if self.pending_fast.fetch_sub(1, Ordering::Relaxed) == 1 {
                    self.settled.notify_waiters();
                }
                Self::bump(&self.meters.fast, &self.rate_fast);
            }
            TaskKind::RealPing => {
                self.mark_settled(&self.real_ended_ms);
                if self.pending_real.fetch_sub(1, Ordering::Relaxed) == 1 {
                    self.settled.notify_waiters();
                }
            }
            _ => {}
        }
    }

    /// One candidate's real half reached a terminal state (probe, sibling
    /// retire, stop retire): the level's `done` advances. Every link counted
    /// into `real.total` reaches this exactly once.
    fn note_real_done(&self) {
        Self::bump(&self.meters.real, &self.rate_real);
    }

    /// Emit the persisted `[real]` marker for a link the native engine cannot
    /// serve (kind-level gate, decided at plan time).
    ///
    /// Called from [`Self::after_fast_settle`], i.e. right after that link's fast
    /// result: the reverse order would let the fast success (which writes
    /// `error = None`) clear the marker.
    fn emit_untestable_marker(&self, link: &ProfileStats, text: &str) {
        // Persist from the link's own snapshot: the events handler only stages
        // what the loaded page still holds.
        self.stage_result(
            link,
            TestType::RealPing,
            &ProbeOutcome::soft_failure(text.to_string()),
        );
        let (protocol_id, endpoint_id) = (link.protocol_id.get(), link.endpoint_id.get());
        let _ = self.tx.try_send(CoreEvent::TestTypeUpdate {
            endpoint_id,
            protocol_id,
            test_type: TestType::RealPing,
        });
        try_send_or_warn(
            &self.tx,
            CoreEvent::SpeedTestResult {
                endpoint_id,
                protocol_id,
                test_type: TestType::RealPing,
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some(text.to_string()),
            },
            "untestable_marker",
        );
    }
}

// ── Entry points ───────────────────────────────────────────────────────────

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

/// Batch fast-ping every link in the database.
pub fn start_batch_ping(state: &mut AppState) {
    start_batch(state, PlanSource::Feed, false, false);
}

/// Batch fast-ping every link in the database, then real-ping each link.
///
/// With `real_ping_test_all_protocols` unset (default), one successful real
/// ping on an endpoint retires the remaining links' real tasks.
pub fn start_batch_then_real_ping(state: &mut AppState) {
    let dedup = !state.config.speed_test.real_ping_test_all_protocols;
    start_batch(state, PlanSource::Feed, true, dedup);
}

/// Fast-ping every link of the selected endpoint (collapsed multi-protocol rows).
pub fn start_endpoint_batch_ping(state: &mut AppState) {
    let plan = plan_selected_endpoint(state);
    start_batch(state, PlanSource::Links(plan), false, false);
}

/// Fast-ping then real-ping every link of the selected endpoint. `dedup` is
/// off: every protocol of the endpoint gets a real ping (their exit IPs may
/// differ).
pub fn start_endpoint_batch_real_ping(state: &mut AppState) {
    let plan = plan_selected_endpoint(state);
    start_batch(state, PlanSource::Links(plan), true, false);
}

fn start_batch(state: &mut AppState, plan: PlanSource, real_phase: bool, dedup_endpoints: bool) {
    // A feed-wide plan is loaded by the batch task (off the UI thread), so only
    // an explicit plan can be known-empty here.
    if let PlanSource::Links(plan) = &plan
        && plan.is_empty()
    {
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
    // A fresh user gesture always starts with a clear stop flag AND a clear
    // gate: task ids are process-local and nothing from the previous batch is
    // live any more.
    state.speed_test_stop.store(false, Ordering::Relaxed);
    state.scheduler.reset();
    let Some(tx) = state.core_event_tx.clone() else {
        return;
    };
    let runner: Arc<dyn BatchProbeRunner> = Arc::new(EngineProbeRunner);
    let db = state.db.clone();
    let writer = state.link_writer.clone();
    let scheduler = state.scheduler.clone();
    let stop = state.speed_test_stop.clone();
    // The meters hold no denominator until the walk fills one, which the status
    // bar renders as "Testing..." — and the batch handle published alongside
    // them is what says a batch is alive.
    let meters = Arc::new(crate::types::BatchMeters::default());
    state.batch_progress = Some(Arc::clone(&meters));
    let batch_slot: Arc<OnceLock<Arc<BatchShared>>> = Arc::new(OnceLock::new());
    state.batch = Some(batch_slot.clone());
    let fast_timeout = *state.config.speed_test.tcp_timeout_secs;
    let real_timeout = *state.config.speed_test.real_ping_timeout_secs;
    let real_retries = state.config.speed_test.real_ping_retries;
    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let real_concurrency = state.config.speed_test.real_ping_concurrency.max(1);
    let fast_concurrency = state.config.speed_test.fast_ping_concurrency.max(1);
    let error_ttl_hours = state.config.speed_test.error_ttl_hours;
    // Sleep the full deferral window once, then re-schedule (the window is
    // measured in whole seconds and comes from the speed-test config via
    // `TaskScheduler::set_limits`).
    let defer_delay = Duration::from_secs(scheduler.dns_defer_secs().max(1) as u64);

    tokio::spawn(run_batch(BatchParams {
        scheduler,
        db,
        writer,
        tx,
        runner,
        stop,
        meters,
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
        fast_concurrency,
        page_size: PROFILES_PAGE_SIZE,
        error_ttl_hours,
        batch_slot,
    }));
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::sync::atomic::AtomicUsize;

    use tokio::sync::mpsc;
    use xray_tui_db::models::{ErrorInfo, Latency, ProfileErr};

    use crate::ops::profiles::test_support::{fake_row, test_state};
    use crate::ops::scheduler::TaskScheduler;

    use super::*;

    /// Deterministic probe runner: fixed outcomes + call recording + an
    /// optional gate that blocks real probes (for the stop-mid-batch test).
    struct StubRunner {
        fast_outcome: ProbeOutcome,
        real_outcome: Mutex<ProbeOutcome>,
        fast_calls: Arc<AtomicUsize>,
        real_calls: Arc<AtomicUsize>,
        real_gate: Mutex<Option<Arc<Notify>>>,
        /// When set, every fast probe marks the endpoint's DNS failure — used
        /// to land a deferral deterministically between phase 1 and phase 2.
        dns_mark_on_fast: Mutex<Option<(Arc<TaskScheduler>, EndpointId)>>,
        /// Per-address fast outcome, for the tests that need a mixed batch
        /// (an unreachable link next to a reachable one).
        fast_by_addr: Mutex<std::collections::HashMap<String, ProbeOutcome>>,
        /// Per-address fast delay, for the tests that need a deterministic
        /// fast-settle order (the real half is dispatched from that settle).
        fast_delay_by_addr: Mutex<std::collections::HashMap<String, Duration>>,
        /// Endpoint ids in the order their real probe STARTED.
        real_endpoint_order: Mutex<Vec<i64>>,
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
                real_gate: Mutex::new(None),
                dns_mark_on_fast: Mutex::new(None),
                fast_by_addr: Mutex::new(std::collections::HashMap::new()),
                fast_delay_by_addr: Mutex::new(std::collections::HashMap::new()),
                real_endpoint_order: Mutex::new(Vec::new()),
            }
        }
    }

    impl BatchProbeRunner for StubRunner {
        fn fast<'a>(
            &'a self,
            _config_type: i32,
            addr: &'a str,
            _port: u16,
            _timeout: Duration,
        ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.fast_calls.fetch_add(1, Ordering::Relaxed);
                if let Some((sched, eid)) = &*self.dns_mark_on_fast.lock() {
                    sched.mark_dns_failure(*eid);
                }
                let delay = self.fast_delay_by_addr.lock().get(addr).copied();
                if let Some(delay) = delay {
                    tokio::time::sleep(delay).await;
                }
                if let Some(outcome) = self.fast_by_addr.lock().get(addr) {
                    return outcome.clone();
                }
                self.fast_outcome.clone()
            })
        }

        fn real<'a>(
            &'a self,
            endpoint: &'a Endpoint,
            _config: &'a ProtocolConfig,
            _req: NativeProbeReq<'a>,
        ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.real_calls.fetch_add(1, Ordering::Relaxed);
                self.real_endpoint_order.lock().push(endpoint.id.get());
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
            // The page query drives from `endpoints` (joined to the stored
            // ordering keys), so a DB-backed plan needs the endpoint row too.
            state
                .db
                .upsert_endpoint(&row.endpoint)
                .await
                .expect("upsert endpoint");
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
        BatchParams {
            scheduler: h.state.scheduler.clone(),
            db: h.state.db.clone(),
            writer: h.state.link_writer.clone(),
            tx: h.tx.clone(),
            runner: h.runner.clone(),
            stop: h.state.speed_test_stop.clone(),
            meters: Arc::new(crate::types::BatchMeters::default()),
            plan: PlanSource::Links(plan),
            real_phase,
            dedup_endpoints: dedup,
            fast_timeout: Duration::from_secs(2),
            real_timeout: Duration::from_secs(2),
            real_retries: 1,
            ping_url: "http://127.0.0.1/".to_string(),
            ip_api_url: "http://127.0.0.1/ip".to_string(),
            defer_delay: Duration::from_millis(50),
            real_concurrency: 8,
            fast_concurrency: 8,
            // Tiny pages: the streaming walk is what a feed batch exercises.
            page_size: 2,
            error_ttl_hours: None,
            batch_slot: Arc::new(OnceLock::new()),
        }
    }

    fn start_test_batch(
        h: &mut Harness,
        plan: Vec<PlanLink>,
        real_phase: bool,
        dedup: bool,
    ) -> tokio::task::JoinHandle<()> {
        start_test_batch_with(h, plan, real_phase, dedup, 8)
    }

    /// Same, with the real level's concurrency pinned: one slot makes the real
    /// halves' order and the sibling-dedup decisions deterministic.
    fn start_test_batch_with(
        h: &mut Harness,
        plan: Vec<PlanLink>,
        real_phase: bool,
        dedup: bool,
        real_concurrency: usize,
    ) -> tokio::task::JoinHandle<()> {
        let mut p = build_params(h, plan, real_phase, dedup);
        p.real_concurrency = real_concurrency;
        h.state.batch_progress = Some(p.meters.clone());
        h.state.batch = Some(p.batch_slot.clone());
        tokio::spawn(run_batch(p))
    }

    /// Poll events until the batch's terminal `BatchEnded` clears the shared
    /// meters (or the deadline expires).
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

    /// The gate is the batch's only task-state authority, so "clear" means the
    /// scheduler no longer knows the link.
    fn assert_gate_clear(sched: &crate::ops::scheduler::TaskScheduler, link: &ProfileStats) {
        assert_eq!(
            sched.task_of(link),
            None,
            "gate must be clear after the batch"
        );
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
                assert_gate_clear(h.state.scheduler.as_ref(), link);
            }
        }
    }

    // ── the batch tests the feed, not the viewport ───────────────────────

    /// "All profiles" means the database, not the loaded page: the page is 200
    /// endpoint rows, so a page-scoped run tested 272 of the 4,523 links in the
    /// 2026-09-15 feed. The walk covers every page of the tab's query and hands
    /// them over one at a time — the batch dispatches each page as it loads.
    #[tokio::test]
    async fn the_plan_walk_covers_every_page_of_links() {
        let rows: Vec<EndpointRow> = (1..=5)
            .map(|i| fake_row(i, &format!("10.0.0.{i}"), 1))
            .collect();
        let h = harness(rows.clone()).await;

        // page_size 2 over 5 endpoints: three pages, one of them partial — the
        // walk must stop on the feed count, not on a short page.
        let mut walk = PlanWalk::new(PlanSource::Feed, h.state.db.clone(), 2);
        let mut hosts: Vec<String> = Vec::new();
        let mut pages = 0;
        while let Some(links) = walk.next_page().await.expect("walk page") {
            pages += 1;
            assert_eq!(
                walk.pages_total(),
                3,
                "the first page carries the feed-wide count"
            );
            hosts.extend(links.iter().map(|pl| pl.endpoint.host.clone()));
        }
        assert_eq!(pages, 3, "5 endpoints at 2 per page");
        hosts.sort();
        let mut expected: Vec<String> = rows.iter().map(|r| r.endpoint.host.clone()).collect();
        expected.sort();
        assert_eq!(hosts, expected, "every link in the feed is planned");
    }

    /// A feed-scoped batch does not depend on the loaded page at all: with an
    /// empty page it still probes every link the database holds.
    #[tokio::test]
    async fn a_feed_batch_probes_links_outside_the_loaded_page() {
        let rows: Vec<EndpointRow> = (1..=3)
            .map(|i| fake_row(i, &format!("10.0.0.{i}"), 1))
            .collect();
        let mut h = harness(rows.clone()).await;
        h.state.endpoints.clear();

        let p = BatchParams {
            plan: PlanSource::Feed,
            ..build_params(&h, Vec::new(), false, false)
        };
        h.state.batch_progress = Some(p.meters.clone());
        run_batch(p).await;

        assert_eq!(
            h.runner.fast_calls.load(Ordering::Relaxed),
            3,
            "the feed's links are probed even though the page is empty"
        );
        for row in &rows {
            let link = xray_tui_db::models::ProfileStats::filter_by_protocol_id_and_endpoint_id(
                row.links[0].protocol_id,
                row.links[0].endpoint_id,
            )
            .first()
            .exec(&mut h.state.db.connection().await.expect("conn"))
            .await
            .expect("read")
            .expect("row");
            assert_eq!(
                link.latency,
                Some(Latency::Fast { delay: 10 }),
                "a feed link's result is persisted: {link:?}"
            );
        }
    }

    // ── the batch, not the page, is the record ───────────────────────────

    /// A batch persists its own results: the UI page is a view, and a page
    /// that no longer holds the row (an import moves every endpoint's ordering
    /// keys while the batch runs) must not cost the write. The handler used to
    /// be the only writer of a result, so every link it could not find in the
    /// loaded page was logged and dropped.
    #[tokio::test]
    async fn batch_persists_results_the_page_no_longer_holds() {
        let rows = vec![fake_row(1, "10.0.0.1", 1), fake_row(2, "10.0.0.2", 1)];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        // The page is replaced mid-batch: the events handler finds no row.
        h.state.endpoints.clear();

        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();

        // `finish_batch` flushes: the result is in the database.
        for row in &rows {
            let link = xray_tui_db::models::ProfileStats::filter_by_protocol_id_and_endpoint_id(
                row.links[0].protocol_id,
                row.links[0].endpoint_id,
            )
            .first()
            .exec(&mut h.state.db.connection().await.expect("conn"))
            .await
            .expect("read")
            .expect("row");
            assert!(
                matches!(
                    &link.latency,
                    Some(Latency::Real { delay: 50, ip: Some(ip) }) if ip == "1.2.3.4"
                ),
                "the batch's own staging must persist the result: {link:?}"
            );
            assert!(link.error.is_none(), "no marker expected: {link:?}");
        }
    }

    // ── phase 2 skips what phase 1 proved unreachable ────────────────────

    /// A link whose fast probe hard-failed (refused / unreachable /
    /// unresolvable / timeout) is not real-probed: the probe can only spend its
    /// whole timeout re-learning the same thing. A soft failure (a TLS-level
    /// error, say) still gets its probe — that answers a different question.
    #[tokio::test]
    async fn phase_two_skips_links_the_fast_probe_proved_unreachable() {
        let rows = vec![
            fake_row(1, "10.0.0.1", 1),
            fake_row(2, "10.0.0.2", 1),
            fake_row(3, "10.0.0.3", 1),
        ];
        let mut h = harness(rows.clone()).await;
        {
            let mut by_addr = h.runner.fast_by_addr.lock();
            by_addr.insert(
                "10.0.0.1".to_string(),
                ProbeOutcome::Failed {
                    text: "IO: Connection refused (os error 111)".to_string(),
                    class: ProbeClass::Refused,
                    hard: true,
                },
            );
            by_addr.insert(
                "10.0.0.2".to_string(),
                ProbeOutcome::soft_failure("TLS error: handshake error: alert: 2 40"),
            );
        }
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 3);
        assert_eq!(
            h.runner.real_calls.load(Ordering::Relaxed),
            2,
            "only the soft-failed and the reachable link are real-probed"
        );

        let persisted = |row: &EndpointRow| {
            let (protocol_id, endpoint_id) = (row.links[0].protocol_id, row.links[0].endpoint_id);
            let db = h.state.db.clone();
            async move {
                xray_tui_db::models::ProfileStats::filter_by_protocol_id_and_endpoint_id(
                    protocol_id,
                    endpoint_id,
                )
                .first()
                .exec(&mut db.connection().await.expect("conn"))
                .await
                .expect("read")
                .expect("row")
            }
        };

        let skipped = persisted(&rows[0]).await;
        assert!(
            matches!(
                skipped.error.as_ref().map(|e| e.kind),
                Some(ProfileErr::Fast)
            ),
            "the unreachable link keeps its fast marker: {skipped:?}"
        );
        assert_eq!(
            skipped.error.as_ref().map(|e| e.text.as_str()),
            Some("IO: Connection refused (os error 111)")
        );

        for row in &rows[1..] {
            let link = persisted(row).await;
            assert!(
                matches!(
                    &link.latency,
                    Some(Latency::Real { delay: 50, ip: Some(ip) }) if ip == "1.2.3.4"
                ),
                "a probed link carries the real result: {link:?}"
            );
        }
    }

    // ── a phase-2 patch keeps the phase-1 measurement ───────────────────

    /// The RESULT group writes `latency` and `error` together, and every batch
    /// patch is built from the plan-time snapshot — so a phase-2 failure used
    /// to carry `latency = None` and erase the delay phase 1 had just measured
    /// for the same link (160 of 218 real failures lost it on 2026-09-15; the
    /// events handler used to mask this by staging from its live page row).
    #[tokio::test]
    async fn a_real_failure_keeps_the_fast_measurement() {
        let rows = vec![fake_row(1, "10.0.0.1", 1)];
        let mut h = harness(rows.clone()).await;
        *h.runner.real_outcome.lock() =
            ProbeOutcome::soft_failure("timeout on probe attempt (limit 5s)");

        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();

        let link = xray_tui_db::models::ProfileStats::filter_by_protocol_id_and_endpoint_id(
            rows[0].links[0].protocol_id,
            rows[0].links[0].endpoint_id,
        )
        .first()
        .exec(&mut h.state.db.connection().await.expect("conn"))
        .await
        .expect("read")
        .expect("row");
        assert_eq!(
            link.latency,
            Some(Latency::Fast { delay: 10 }),
            "the fast delay survives a real-ping failure: {link:?}"
        );
        assert_eq!(
            link.error.as_ref().map(|e| e.kind),
            Some(ProfileErr::Real),
            "and the marker still lands: {link:?}"
        );
    }

    // ── one summary line per batch ───────────────────────────────────────

    /// The batch's record is ONE line carrying every counter: per-result lines
    /// moved to `debug` (a 5-minute run wrote 32k of them on 2026-09-15, and
    /// reconstructing the run from them was the whole cost of the
    /// investigation). This pins the record's fields so a later edit cannot
    /// drop one silently.
    #[tokio::test]
    async fn the_batch_summary_reports_every_counter() {
        let rows = vec![fake_row(1, "10.0.0.1", 1)];
        let h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        let params = build_params(&h, plan, true, false);
        let meters = params.meters.clone();
        let shared = BatchShared::new(params);
        let counters = &shared.counters;
        meters.fast.total.store(4, Ordering::Relaxed);
        meters.real.total.store(3, Ordering::Relaxed);
        counters.fast_ok.store(7, Ordering::Relaxed);
        counters.fast_hard_failed.store(5, Ordering::Relaxed);
        counters.fast_soft_failed.store(3, Ordering::Relaxed);
        counters.real_ok.store(2, Ordering::Relaxed);
        counters.real_failed.store(4, Ordering::Relaxed);
        counters.untestable.store(1, Ordering::Relaxed);
        counters.unreachable.store(5, Ordering::Relaxed);
        counters.deferred.store(6, Ordering::Relaxed);
        counters.queue_full.store(8, Ordering::Relaxed);
        shared.plan_ms.store(120, Ordering::Relaxed);
        shared.fast_started_ms.store(3, Ordering::Relaxed);
        shared.fast_ended_ms.store(180, Ordering::Relaxed);
        shared.real_started_ms.store(60, Ordering::Relaxed);
        shared.real_ended_ms.store(340, Ordering::Relaxed);
        // The classes are what a fix is aimed at: `hard-fail=5` alone cannot
        // say whether the run failed on timeouts, DNS or refusals.
        bump_class(&counters.fast_fail, ProbeClass::Timeout);
        bump_class(&counters.fast_fail, ProbeClass::Timeout);
        bump_class(&counters.fast_fail, ProbeClass::Dns);
        bump_class(&counters.real_fail, ProbeClass::Tls);

        let line = summary_line(&shared);
        for expected in [
            "links=4",
            "plan=120 ms",
            "untestable=1",
            "queue-full=8",
            "deferred=6",
            "fast ok=7 hard-fail=5 soft-fail=3 [timeout=2 dns=1] (3..180 ms)",
            "real ok=2 failed=4 [tls=1] skipped-unreachable=5 (60..340 ms)",
            "stopped=false",
            "staged-left=0",
        ] {
            assert!(line.contains(expected), "missing {expected:?} in: {line}");
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
        .updated_at(jiff::Timestamp::now().as_second() - 48 * 3600)
        .exec(&mut conn)
        .await
        .unwrap();

        let plan = plan_from_rows(&rows);
        let mut params = build_params(&h, plan, false, false);
        params.error_ttl_hours = Some(24);
        h.state.batch_progress = Some(params.meters.clone());
        h.state.speed_test_stop.store(true, Ordering::Relaxed);
        tokio::spawn(run_batch(params)).await.unwrap();
        await_batch_done(&mut h.state).await;

        // The batch's staged patches are the only place its result lives until
        // the flush: make them durable, then read the row back.
        h.state.link_writer.flush().await.expect("flush");
        let stored = h
            .state
            .db
            .read_link_row(link.protocol_id, link.endpoint_id)
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
            assert_gate_clear(h.state.scheduler.as_ref(), link);
        }
    }

    // ── sibling dedup (dedup_endpoints, best-effort) ─────────────────────

    /// The first real success on an endpoint retires its remaining links'
    /// real halves. Pipelining makes this best-effort: siblings whose fast
    /// halves settle together may start before the success lands, so the check
    /// sits AFTER the real permit (one slot here) — the saving is the links
    /// still waiting for capacity, which is the throughput-bound case the
    /// option exists for.
    #[tokio::test]
    async fn sibling_dedup_retires_links_waiting_behind_a_success() {
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        // dedup_endpoints=true (the default); one real slot makes the order
        // deterministic.
        let handle = start_test_batch_with(&mut h, plan, true, true, 1);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(
            h.runner.real_calls.load(Ordering::Relaxed),
            1,
            "the sibling waiting for the real slot is retired by the success"
        );
        let links = &h.state.endpoints[0].links;
        let with_real = links
            .iter()
            .filter(|l| matches!(l.latency, Some(Latency::Real { .. })));
        assert_eq!(with_real.count(), 1, "exactly one link got a real result");
        for link in links {
            // The retired sibling never wrote a marker or a latency.
            assert!(link.error.is_none(), "retired link must not write a marker");
            assert_gate_clear(h.state.scheduler.as_ref(), link);
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
        for link in &h.state.endpoints[0].links {
            assert!(matches!(
                link.latency,
                Some(Latency::Real { delay: 50, .. })
            ));
            assert!(link.error.is_none());
        }
    }

    /// The real level is the long pole (measured 2.6 results/s against 128 for
    /// the fast level on 2026-09-16), so a run stopped part-way should have
    /// tested the fastest links first. The pipeline inherits that ordering from
    /// the fast level: a link's real half starts when its own fast half settles,
    /// so the probe order is the fast completion order — ascending fast latency,
    /// with no sort and no barrier.
    #[tokio::test]
    async fn real_probes_follow_fast_completion_order() {
        let rows = vec![
            fake_row(1, "10.0.0.1", 1),
            fake_row(2, "10.0.0.2", 1),
            fake_row(3, "10.0.0.3", 1),
        ];
        let mut h = harness(rows.clone()).await;
        {
            let mut delays = h.runner.fast_delay_by_addr.lock();
            delays.insert("10.0.0.1".to_string(), Duration::from_millis(150));
            delays.insert("10.0.0.2".to_string(), Duration::from_millis(50));
            delays.insert("10.0.0.3".to_string(), Duration::from_millis(100));
        }
        let plan = plan_from_rows(&rows);
        // One real slot: the semaphore is fair, so the recorded start order is
        // the settle order.
        let handle = start_test_batch_with(&mut h, plan, true, false, 1);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        let order = h.runner.real_endpoint_order.lock().clone();
        let expected: Vec<i64> = rows.iter().map(|r| r.endpoint.id.get()).collect::<Vec<_>>();
        // Declared slowest-first: the 50/100/150 ms fast probes must settle —
        // and therefore real-probe — in the reverse order.
        assert_eq!(
            order,
            vec![expected[1], expected[2], expected[0]],
            "real probes follow the fast settle order"
        );
    }

    // ── native testability gate ──────────────────────────────────────────
    #[tokio::test]
    async fn untestable_kind_is_marked_and_still_fast_probed() {
        let mut rows = vec![fake_row(1, "10.0.0.1", 1)];
        // A kind with no native implementation. The plan-time gate reads the
        // in-memory `proto_kind` only — the config is never loaded for it.
        for proto in rows[0].protocols.values_mut() {
            proto.proto_kind = xray_tui_proto::proto_spec::ProtocolKind::Tuic;
        }
        let mut h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        // Fast-only batch: the marker must still land (the gate runs for every
        // ping operation) and the fast probe must still run (R1).
        let handle = start_test_batch(&mut h, plan, false, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        assert_eq!(
            h.runner.fast_calls.load(Ordering::Relaxed),
            1,
            "the fast probe still runs for an untestable row"
        );
        assert_eq!(
            h.runner.real_calls.load(Ordering::Relaxed),
            0,
            "no real probe for an untestable row"
        );
        // The marker lands AFTER the fast result: a fast success writes
        // `error = None`, so the reverse order would erase it.
        let link = &h.state.endpoints[0].links[0];
        let error = link.error.as_ref().expect("marker persisted");
        assert!(crate::ops::ping::is_untestable_marker(error), "{error:?}");
        assert!(error.text.contains("no native implementation"), "{error:?}");
    }

    #[test]
    fn untestable_markers_are_not_removable_failures() {
        let mut row = fake_row(1, "10.0.0.1", 1);
        let link = &mut row.links[0];
        link.error = Some(ErrorInfo {
            kind: ProfileErr::Real,
            text: untestable_marker_text("no native implementation for this protocol kind"),
        });
        assert!(
            !is_removable_failure(link),
            "an untestable row must survive Remove Bad Servers"
        );
        link.error = Some(ErrorInfo {
            kind: ProfileErr::Real,
            text: "timeout".to_string(),
        });
        assert!(is_removable_failure(link), "a real failure still counts");
        link.error = None;
        assert!(!is_removable_failure(link));
    }

    // ── all-fail error markers ───────────────────────────────────────────

    #[tokio::test]
    async fn all_fail_writes_real_error_markers_on_every_link() {
        let rows = vec![fake_row(1, "10.0.0.1", 2), fake_row(2, "10.0.0.2", 1)];
        let mut h = harness(rows.clone()).await;
        *h.runner.real_outcome.lock() = ProbeOutcome::soft_failure("timeout");
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

    /// A DNS deferral is a `DnsDeferred` answer from `schedule`, which is
    /// returned BEFORE the gate is touched — so the deferred half holds no gate
    /// entry and only the retry task keeps it alive. `finish_batch` must wait for
    /// that retry (else the terminal event lands first, the retry's late result
    /// re-creates the meters, and the next batch is rejected as "already
    /// running").
    ///
    /// The marker lands inside this link's own fast probe, i.e. deterministically
    /// before its real half is scheduled: the deferral is therefore on the REAL
    /// half, which re-enters without ever touching the fast half again.
    #[tokio::test]
    async fn a_deferred_half_is_joined_before_finish_and_the_next_batch_starts() {
        let rows = vec![fake_row(1, "10.0.0.1", 1)];
        let mut h = harness(rows.clone()).await;
        h.state.scheduler = Arc::new(TaskScheduler::new(3, 3));
        *h.runner.dns_mark_on_fast.lock() = Some((h.state.scheduler.clone(), EndpointId::new(1)));
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch(&mut h, plan, true, false);
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        // The link got both results, no marker, and its gate is clear.
        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 1);
        assert_eq!(
            h.runner.real_calls.load(Ordering::Relaxed),
            1,
            "the deferred real half was probed after the window"
        );
        let link = &h.state.endpoints[0].links[0];
        assert!(matches!(link.latency, Some(Latency::Real { .. })));
        assert!(link.error.is_none());
        assert_gate_clear(h.state.scheduler.as_ref(), link);
        assert!(
            h.state.batch_progress.is_none(),
            "meters must be cleared after the batch (terminal event last)"
        );

        // A subsequent batch starts cleanly.
        let plan2 = plan_from_rows(&rows);
        let handle2 = start_test_batch(&mut h, plan2, false, false);
        handle2.await.unwrap();
        await_batch_done(&mut h.state).await;
        assert_eq!(h.runner.fast_calls.load(Ordering::Relaxed), 2);
    }

    // ── stop mid-batch ───────────────────────────────────────────────────

    #[tokio::test]
    async fn stop_mid_batch_writes_no_error_markers() {
        let rows = vec![fake_row(1, "10.0.0.1", 2)];
        let mut h = harness(rows.clone()).await;
        let gate = Arc::new(Notify::new());
        *h.runner.real_gate.lock() = Some(gate.clone());
        let plan = plan_from_rows(&rows);
        // One real slot: the sibling's real half waits for capacity, so a stop
        // retires it without a probe.
        let handle = start_test_batch_with(&mut h, plan, true, false, 1);

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
            assert_gate_clear(h.state.scheduler.as_ref(), link);
        }
        // Only the in-flight probe ran; the sibling was retired without a probe.
        assert_eq!(h.runner.real_calls.load(Ordering::Relaxed), 1);
    }

    // ── meters and the terminal event ────────────────────────────────────

    /// The meters are two independent levels, so a numerator can never be read
    /// against another level's denominator (the 2026-09-16 run's bar paired
    /// phase-2 results with the whole 34,562-link plan). The real level's
    /// denominator is its own candidate set: the untestable and hard-failed
    /// links never enter it.
    #[tokio::test]
    async fn meters_track_both_levels_and_each_denominator_is_its_own() {
        let rows = vec![
            fake_row(1, "10.0.0.1", 1),
            fake_row(2, "10.0.0.2", 1),
            fake_row(3, "10.0.0.3", 1),
        ];
        let mut h = harness(rows.clone()).await;
        // Link 2's fast probe proves its proxy unreachable, so its real half is
        // skipped: 3 planned links, 2 real candidates.
        h.runner.fast_by_addr.lock().insert(
            "10.0.0.2".to_string(),
            ProbeOutcome::Failed {
                text: "IO: Connection refused (os error 111)".to_string(),
                class: ProbeClass::Refused,
                hard: true,
            },
        );
        let plan = plan_from_rows(&rows);
        let handle = start_test_batch_with(&mut h, plan, true, false, 8);
        // Keep the batch's own meters: `await_batch_done` clears the state copy.
        let meters = h
            .state
            .batch_progress
            .clone()
            .expect("the batch publishes its meters");
        handle.await.unwrap();
        await_batch_done(&mut h.state).await;

        let load = |a: &AtomicU32| a.load(Ordering::Relaxed);
        assert_eq!(load(&meters.fast.total), 3, "every planned link is counted");
        assert_eq!(load(&meters.fast.done), 3);
        assert_eq!(
            load(&meters.real.total),
            2,
            "the hard-failed link is never a real candidate"
        );
        assert_eq!(load(&meters.real.done), 2);
        assert!(load(&meters.real.done) <= load(&meters.real.total));
        assert!(load(&meters.fast.done) <= load(&meters.fast.total));
    }

    /// The ETA needs a denominator, a rate sample and work left; anything else
    /// renders as `--` rather than a wrong estimate.
    #[test]
    fn phase_meters_eta_needs_a_denominator_a_rate_and_work_left() {
        let m = crate::types::PhaseMeters::default();
        assert_eq!(m.eta_secs(), None, "no denominator yet");
        m.total.store(100, Ordering::Relaxed);
        assert_eq!(m.eta_secs(), None, "no rate sample yet");
        m.rate_milli.store(2000, Ordering::Relaxed);
        assert_eq!(m.eta_secs(), Some(50), "100 left at 2/s");
        m.done.store(100, Ordering::Relaxed);
        assert_eq!(m.eta_secs(), None, "complete");
    }

    /// The plan line is the batch's own record of the walk (the store can drop
    /// it under a log flood, so the shape is pinned here rather than observed).
    #[test]
    fn plan_line_names_the_pages_links_and_walk_time() {
        assert_eq!(
            plan_line(56140, 136, 5446),
            "batch: planned 56140 link(s) over 136 page(s) in 5446 ms"
        );
        assert_eq!(
            plan_line(200, 1, 62),
            "batch: planned 200 link(s) over 1 page(s) in 62 ms"
        );
    }

    /// The rate window starts at the level's FIRST result, not at batch
    /// construction: otherwise the idle stretch before a level's first settle
    /// reads as ~0 results/s and pins that level's ETA at `--` (the 2026-09-17
    /// live run: 46 real results in 8 s, first sample 1 result over ~5 s → 0,
    /// and no later settle crossed a fresh second boundary).
    #[test]
    fn rate_window_starts_at_the_levels_first_result() {
        let meters = crate::types::PhaseMeters::default();
        meters.total.store(1000, Ordering::Relaxed);
        // A slot backdated as if the batch had been running for 5 idle seconds.
        let backdated = |secs: u64| {
            Instant::now()
                .checked_sub(Duration::from_secs(secs))
                .expect("backdate")
        };
        let slot = Mutex::new((backdated(5), 0));
        BatchShared::bump(&meters, &slot);
        assert_eq!(
            meters.rate_milli.load(Ordering::Relaxed),
            0,
            "one result is not a rate"
        );
        assert_eq!(meters.eta_secs(), None, "no estimate from a single result");

        // The next window has real activity: 1 s, one more result → 1/s.
        *slot.lock() = (backdated(1), 1);
        BatchShared::bump(&meters, &slot);
        assert_eq!(meters.rate_milli.load(Ordering::Relaxed), 1000);
        assert_eq!(meters.eta_secs(), Some(998), "998 left at 1/s");
    }

    /// One terminal event per batch, and no per-settle progress event: a
    /// feed-wide batch used to send ~34k of them, each describing state the
    /// shared meters already publish.
    #[tokio::test]
    async fn a_batch_emits_one_terminal_event_and_no_progress_events() {
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
        let mut ended = 0;
        while let Ok(ev) = rx.try_recv() {
            if matches!(ev, CoreEvent::BatchEnded) {
                ended += 1;
            }
        }
        assert_eq!(ended, 1, "exactly one terminal event per batch");
    }

    /// The quit path reports an interrupted run with the summary a completed run
    /// writes. Before this, quitting mid-batch printed `3381 of 34562
    /// final-phase probe(s) reported, no batch summary` — the per-class
    /// histograms only `summary_line` renders died with the batch task (the
    /// runtime drop cancels it before `finish_batch`).
    #[tokio::test]
    async fn an_interrupted_run_renders_the_batch_summary() {
        let rows = vec![fake_row(1, "10.0.0.1", 1), fake_row(2, "10.0.0.2", 1)];
        let h = harness(rows.clone()).await;
        let plan = plan_from_rows(&rows);
        let params = build_params(&h, plan, true, false);
        let meters = params.meters.clone();
        let shared = BatchShared::new(params);
        shared.counters.real_ok.store(4, Ordering::Relaxed);
        shared.counters.real_failed.store(6, Ordering::Relaxed);
        meters.fast.done.store(7, Ordering::Relaxed);
        meters.real.done.store(10, Ordering::Relaxed);
        shared.pending_real.store(2, Ordering::Relaxed);
        shared.pending_deferred.store(1, Ordering::Relaxed);
        bump_class(&shared.counters.real_fail, ProbeClass::Timeout);
        bump_class(&shared.counters.real_fail, ProbeClass::Timeout);
        bump_class(&shared.counters.real_fail, ProbeClass::Dns);

        let line = interrupted_summary_line(&shared);
        for expected in [
            "batch interrupted at quit:",
            "batch summary:",
            "real ok=4 failed=6 [timeout=2 dns=1]",
            "settled=17",
            "in-flight=3",
        ] {
            assert!(line.contains(expected), "missing {expected:?} in: {line}");
        }
    }
}
