use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use futures_util::StreamExt;
use tokio::sync::{Semaphore, mpsc};

use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{
    BuildParams, ConfigBuilder, CorePool, CoreType, MultiInboundItem, RealCoreManager, find_binary,
    resolve_core, shadowsocks_method, wait_for_socks5,
};
use xray_tui_db::Database;
use xray_tui_db::models::{
    DELAY_SOURCE_FAST, DELAY_SOURCE_REAL, DELAY_SOURCE_UDP, DnsSetting, PingResultUpdate,
    PingSession, ProfileExtension,
};

use crate::AppState;
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
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
    {
        r
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for TCP ping");
        return;
    };
    let Some(proto) = row.protocols.iter().find(|p| p.id == protocol_id) else {
        state.log_trace("error", "tui::ops::ping", "Protocol not found for TCP ping");
        return;
    };
    if row.endpoint.host.is_empty() {
        state.log_trace("error", "tui::ops::ping", "Profile has no address");
        return;
    }
    let addr = row.endpoint.host.clone();
    let port = if row.endpoint.port > 0 && row.endpoint.port <= 65535 {
        row.endpoint.port as u16
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
    let config_type = proto.config_type;
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
/// Both single real pings and batch phase 2 draw ports from this pool's single
/// allocator, so a warm pooled core and a batch core can never collide on a port.
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

/// Start real ping (HTTP through proxy) using a pooled warm core for single-ping reuse.
///
/// The pool is created lazily on first use. Subsequent single pings reuse the same core:
/// sing-box via SIGHUP reload, xray-core via stop+restart. Batch real ping bypasses
/// the pool entirely — it uses the multi-inbound approach instead.
pub fn start_real_ping(state: &mut AppState, protocol_id: i64) {
    if state.testing_profiles.contains(&protocol_id) {
        return;
    }

    // Find profile row by REAL protocol id (sub-row pings target the exact
    // protocol; different credentials → different exits).
    let endpoint;
    let protocol = if let Some(r) = state
        .endpoints
        .iter()
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
    {
        let Some(p) = r.protocols.iter().find(|p| p.id == protocol_id) else {
            state.log_trace(
                "error",
                "tui::ops::ping",
                "Protocol not found for real ping",
            );
            return;
        };
        endpoint = r.endpoint.clone();
        p.clone()
    } else {
        state.log_trace("error", "tui::ops::ping", "Profile not found for real ping");
        return;
    };

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

    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;

    tokio::spawn(async move {
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            protocol_id,
            test_type: TestType::RealPing,
        });

        let result = pool
            .ping(
                &endpoint,
                &protocol,
                &ping_url,
                &ip_api_url,
                timeout,
                retries,
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

/// Signal all running batch speed tests to stop.
pub fn stop_speed_test(state: &mut AppState) {
    state.speed_test_stop.store(true, Ordering::Relaxed);
}

/// Batch TCP ping all visible (filtered) profiles — delegates to `start_sieve`.
pub fn start_batch_ping(state: &mut AppState) {
    let profile_order: Vec<(i64, i32)> = state
        .filtered_profiles()
        .enumerate()
        .map(|(i, r)| (r.active_protocol().id, i as i32))
        .collect();
    reset_ping_status_for(state, &profile_order);
    start_sieve(state, false, profile_order, true);
}

/// Batch TCP ping all visible profiles, then real ping TCP-successful targets via temp core.
pub fn start_batch_then_real_ping(state: &mut AppState) {
    let profile_order: Vec<(i64, i32)> = state
        .filtered_profiles()
        .enumerate()
        .map(|(i, r)| (r.active_protocol().id, i as i32))
        .collect();
    reset_ping_status_for(state, &profile_order);
    // Skip remaining protocols of an endpoint after one succeeds (default);
    // Real Ping Test All Protocols=true disables the dedup for full coverage.
    let dedup = !state.config.speed_test.real_ping_test_all_protocols;
    start_sieve(state, true, profile_order, dedup);
}

/// Fast-ping every protocol of the selected endpoint (endpoint-scoped batch).
///
/// Collapsed endpoint rows with >1 protocols dispatch here — different
/// credentials can route differently, so each protocol gets its own result.
pub fn start_endpoint_batch_ping(state: &mut AppState) {
    let profile_order = selected_endpoint_order(state);
    reset_ping_status_for(state, &profile_order);
    start_sieve(state, false, profile_order, false);
}

/// Real-ping every protocol of the selected endpoint. `dedup_endpoints=false`
/// so all protocols get real-pinged (their exit IPs may differ).
pub fn start_endpoint_batch_real_ping(state: &mut AppState) {
    let profile_order = selected_endpoint_order(state);
    reset_ping_status_for(state, &profile_order);
    start_sieve(state, true, profile_order, false);
}

/// Start a fresh ping round for the endpoints owning `profile_order` protocols:
/// the new batch's results rebuild `seen`/`failed` from zero, so stale
/// all-unreachable labels from a previous round don't linger.
fn reset_ping_status_for(state: &mut AppState, profile_order: &[(i64, i32)]) {
    for (pid, _) in profile_order {
        if let Some(row) = state
            .endpoints
            .iter()
            .find(|r| r.protocols.iter().any(|p| p.id == *pid))
        {
            state.ping_status.remove(&row.endpoint.id);
        }
    }
}

/// `(protocol_id, sort_order)` for every protocol of the currently selected
/// endpoint row (all of them — not just the active one).
fn selected_endpoint_order(state: &AppState) -> Vec<(i64, i32)> {
    let Some(ep_id) = state.selected_profile_id() else {
        return Vec::new();
    };
    let Some(row) = state.endpoints.iter().find(|r| r.endpoint.id == ep_id) else {
        return Vec::new();
    };
    row.protocols
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id, i as i32))
        .collect()
}

/// Flush accumulated `PingResultUpdate`s to DB. Called at page boundaries and batch end.
/// Returns `true` if the flush succeeded, `false` on failure (logged).
async fn batch_upsert_buffer(
    batch_id: &str,
    db: &Arc<Database>,
    buffer: &mut Vec<PingResultUpdate>,
) -> bool {
    if buffer.is_empty() {
        return true;
    }
    let batch = std::mem::take(buffer);
    let extensions: Vec<ProfileExtension> = batch
        .iter()
        .filter_map(|r| {
            r.latency_ms.map(|ms| ProfileExtension {
                protocol_id: r.protocol_id,
                delay: Some(ms),
                speed: None,
                sort_order: None,
                ip_info: r.ip_info.clone(),
                delay_source: Some(match r.ping_type.as_str() {
                    "real" => DELAY_SOURCE_REAL,
                    "udp" => DELAY_SOURCE_UDP,
                    _ => DELAY_SOURCE_FAST,
                }),
                protocol_row: Default::default(),
            })
        })
        .collect();
    match db
        .batch_flush_ping_buffer(batch_id, &batch, &extensions)
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(
                target: "tui::ops::ping",
                "batch_flush_ping_buffer failed: {e}",
            );
            false
        }
    }
}

/// RAII guard: while phase 2 (batch real ping) is alive, the pool's
/// `batch_active` flag stays set so concurrent single pings skip pool reuse.
struct BatchActiveGuard(Arc<AtomicBool>);

impl Drop for BatchActiveGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

/// Outcome of one session's fast ping within a page.
enum PageOutcome {
    /// Key was already in the cross-page `fast_cache` — carry the cached value.
    Cached(Option<i32>, Option<String>),
    /// Owner ping result: Ok(latency) or Err(PingError).
    Pinged(Result<std::time::Duration, xray_tui_core::ping::PingError>),
}

/// Role of a session in `run_page_pings`.
enum PageRole {
    /// No usable address/port — caller skips.
    Skip,
    /// Key already in `fast_cache` at call time — outcome is `Cached`.
    Cached((String, u16)),
    /// Shares a key with an earlier session in this page — reuse its outcome.
    Follower(usize),
    /// First session with this key in the page — ping it.
    Owner,
}

/// Fast-ping a page concurrently: one TCP ping per unique (address, port) in the
/// page, capped at `concurrency`. Sessions already in `fast_cache` (from a prior
/// page) short-circuit to `Cached`; later sessions sharing a key with an earlier
/// one in this page reuse the owner's outcome. Returns one entry per session,
/// aligned by index; `None` for sessions without a usable address/port (the
/// caller already skips those). A `NotSupported` owner result is NOT cached (it
/// is protocol-specific), so same-key followers get the owner's error outcome
/// rather than re-pinging — the observable behavior matches the previous
/// sequential loop.
async fn run_page_pings(
    fmgr: &xray_tui_core::FastPingManager,
    sessions: &[PingSession],
    fast_cache: &HashMap<(String, u16), (Option<i32>, Option<String>)>,
    concurrency: usize,
) -> Vec<Option<PageOutcome>> {
    let mut roles: Vec<PageRole> = Vec::with_capacity(sessions.len());
    let mut claimed: HashMap<(String, u16), usize> = HashMap::new();
    let mut owners: Vec<(usize, String, u16)> = Vec::new();

    for (i, session) in sessions.iter().enumerate() {
        let Some(addr) = &session.address else {
            roles.push(PageRole::Skip);
            continue;
        };
        let port = session.port.unwrap_or(0) as u16;
        if port == 0 {
            roles.push(PageRole::Skip);
            continue;
        }
        let key = (addr.clone(), port);
        if fast_cache.contains_key(&key) {
            roles.push(PageRole::Cached(key));
            continue;
        }
        if let Some(&owner) = claimed.get(&key) {
            roles.push(PageRole::Follower(owner));
            continue;
        }
        claimed.insert(key, i);
        owners.push((i, addr.clone(), port));
        roles.push(PageRole::Owner);
    }

    let mut outcomes: Vec<Option<PageOutcome>> = (0..sessions.len()).map(|_| None).collect();
    let stream = futures_util::stream::iter(owners.into_iter().map(|(i, addr, port)| {
        let config_type = sessions[i].config_type;
        async move { (i, fmgr.ping(config_type, &addr, port).await) }
    }))
    .buffer_unordered(concurrency.max(1));
    futures_util::pin_mut!(stream);
    while let Some((i, result)) = stream.next().await {
        outcomes[i] = Some(PageOutcome::Pinged(result));
    }

    for (i, role) in roles.into_iter().enumerate() {
        match role {
            PageRole::Skip => {}
            PageRole::Cached(key) => {
                let (ms, err) = fast_cache
                    .get(&key)
                    .expect("run_page_pings: cached key disappeared")
                    .clone();
                outcomes[i] = Some(PageOutcome::Cached(ms, err));
            }
            PageRole::Follower(owner) => {
                if let Some(Some(PageOutcome::Pinged(result))) = outcomes.get(owner) {
                    outcomes[i] = Some(PageOutcome::Pinged(result.clone()));
                }
            }
            PageRole::Owner => {}
        }
    }
    outcomes
}

/// Outcome of one real-ping group attempt (one core start).
enum GroupOutcome {
    /// All ready sessions completed or failed at item level.
    Done,
    /// Group-level failure that halving may fix (config build / core start /
    /// no inbound port ready). Error string is reported if the group bottoms
    /// out at a single profile.
    Retryable(String),
    /// Failure halving cannot fix (missing binary) — report immediately.
    NotRetryable(String),
}

/// Borrowed context for dispatching one wave chunk (one or more core-type
/// groups, each retried with page-halving on group-level failures).
struct RealPingBatchCtx<'a> {
    tx: &'a mpsc::Sender<CoreEvent>,
    completed_endpoints: &'a Arc<std::sync::Mutex<HashSet<(String, u16)>>>,
    dedup_endpoints: bool,
    proxy_addr: &'a str,
    bin_dir: &'a Path,
    bin_configs_dir: &'a Path,
    real_ping_concurrency: usize,
    real_ping_timeout: std::time::Duration,
    real_ping_retries: u32,
    ping_url: &'a str,
    ip_api_url: &'a str,
    stop_flag: &'a AtomicBool,
    log_tx: Option<mpsc::Sender<String>>,
}

impl RealPingBatchCtx<'_> {
    /// Dispatch `items` for one core type with halving retry on group-level
    /// failures: a retryable failure splits the group in half and retests each
    /// half (stack-based, no recursion), down to single-profile groups.
    /// Per-profile ping failures are item level and never retried.
    async fn dispatch_group(
        &self,
        core_type: CoreType,
        items: &[(xray_tui_db::models::EndpointRow, PingSession, u16)],
        buffer: &mut Vec<PingResultUpdate>,
    ) {
        let mut stack: Vec<&[(xray_tui_db::models::EndpointRow, PingSession, u16)]> = vec![items];
        while let Some(batch) = stack.pop() {
            if self.stop_flag.load(Ordering::Relaxed) {
                break;
            }
            match self.run_batch(core_type, batch, buffer).await {
                GroupOutcome::Done => {}
                GroupOutcome::Retryable(_) if batch.len() > 1 => {
                    let mid = batch.len() / 2;
                    stack.push(&batch[mid..]);
                    stack.push(&batch[..mid]);
                }
                GroupOutcome::Retryable(err) | GroupOutcome::NotRetryable(err) => {
                    self.push_failures(batch, &err, buffer);
                }
            }
        }
    }

    /// One core start for `items`: build multi-inbound config, start the core,
    /// wait for all inbound SOCKS5 ports in parallel (a dead port is an
    /// item-level failure), fire capped concurrent HTTP pings, collect results.
    async fn run_batch(
        &self,
        core_type: CoreType,
        items: &[(xray_tui_db::models::EndpointRow, PingSession, u16)],
        buffer: &mut Vec<PingResultUpdate>,
    ) -> GroupOutcome {
        let multi_items: Vec<MultiInboundItem> = items
            .iter()
            .map(|(profile, _session, port)| MultiInboundItem {
                endpoint: &profile.endpoint,
                protocol: profile.active_protocol(),
                assigned_port: *port,
            })
            .collect();

        let base_params = BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: 0,
            http_port: None,
            listen: self.proxy_addr.to_string(),
            sniffing: false,
            clash_api_port: None,
            mux: None,
            clash_mixin: None,
            skip_cert_verify: false,
        };
        let dns = DnsSetting {
            id: "default".to_string(),
            name: None,
            servers: None,
            hosts: None,
            query_strategy: None,
            disable_cache: None,
            disable_fallback: None,
            client_ip: None,
            cache_ttl_secs: None,
        };

        let backend_config =
            match ConfigBuilder::build_multi(&multi_items, core_type, &base_params, &dns) {
                Ok(c) => c,
                Err(e) => return GroupOutcome::Retryable(format!("Config build: {e}")),
            };

        let bin_path = match find_binary(core_type, self.bin_dir) {
            Some(p) => p,
            None => return GroupOutcome::NotRetryable("Binary not found".to_string()),
        };

        let log_tx = if let Some(tx) = &self.log_tx {
            tx.clone()
        } else {
            let (noop_tx, mut noop_rx) = tokio::sync::mpsc::channel::<String>(256);
            tokio::spawn(async move { while noop_rx.recv().await.is_some() {} });
            noop_tx
        };
        let mut manager = RealCoreManager::new(self.bin_configs_dir.to_path_buf(), log_tx);

        if let Err(e) = manager
            .start(core_type, &backend_config, &bin_path, None)
            .await
        {
            return GroupOutcome::Retryable(format!("Core start: {e}"));
        }

        // Wait for ALL inbound SOCKS5 ports in parallel; a port that never
        // comes up is an item-level failure, not a group failure.
        let readiness = futures_util::future::join_all(items.iter().map(|(_, _, port)| {
            wait_for_socks5(self.proxy_addr, *port, std::time::Duration::from_secs(5))
        }))
        .await;
        let mut ready: Vec<(&xray_tui_db::models::EndpointRow, &PingSession, u16)> =
            Vec::with_capacity(items.len());
        for ((_profile, session, port), ok) in items.iter().zip(&readiness) {
            if ok.is_ok() {
                ready.push((_profile, session, *port));
            } else {
                buffer.push(PingResultUpdate {
                    session_id: session.id.clone(),
                    protocol_id: session.protocol_id,
                    status: "failed".to_string(),
                    ping_type: "real".to_string(),
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("SOCKS5 not ready".to_string()),
                });
                let _ = self.tx.try_send(CoreEvent::SpeedTestResult {
                    protocol_id: session.protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("SOCKS5 not ready".to_string()),
                });
            }
        }
        if ready.is_empty() {
            let _ = manager.stop().await;
            return GroupOutcome::Retryable("SOCKS5 not ready".to_string());
        }

        let sem = Arc::new(Semaphore::new(self.real_ping_concurrency));
        let mut handles = Vec::with_capacity(ready.len());
        for (_profile, session, port) in ready {
            if self.stop_flag.load(Ordering::Relaxed) {
                break;
            }
            let permit = match Arc::clone(&sem).acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let tx = self.tx.clone();
            let session = session.clone();
            let port = port;
            let proxy_addr = self.proxy_addr.to_string();
            let ping_url = self.ping_url.to_string();
            let ip_api_url = self.ip_api_url.to_string();
            let real_ping_timeout = self.real_ping_timeout;
            let real_ping_retries = self.real_ping_retries;

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                    protocol_id: session.protocol_id,
                    test_type: TestType::RealPing,
                });
                let rp_result = xray_tui_core::speed_test::real_ping(
                    &proxy_addr,
                    port,
                    &ping_url,
                    &ip_api_url,
                    real_ping_timeout,
                    real_ping_retries,
                )
                .await;
                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                    protocol_id: session.protocol_id,
                    test_type: TestType::RealPing,
                    latency_ms: rp_result.as_ref().ok().map(|r| r.latency_ms),
                    speed_bps: None,
                    ip_info: rp_result.as_ref().ok().and_then(|r| r.ip_info.clone()),
                    error: rp_result
                        .as_ref()
                        .err()
                        .map(std::string::ToString::to_string),
                });
                (session, rp_result)
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((session, Ok(rp_result))) => {
                    // Endpoint succeeded — mark so Phase 1 and future queries
                    // skip remaining profiles on it (all-visible batches only;
                    // endpoint-scoped batches must real-ping every protocol).
                    if self.dedup_endpoints
                        && let Some(addr) = &session.address
                    {
                        let ep_port = session.port.unwrap_or(0) as u16;
                        self.completed_endpoints
                            .lock()
                            .unwrap()
                            .insert((addr.clone(), ep_port));
                    }
                    buffer.push(PingResultUpdate {
                        session_id: session.id.clone(),
                        protocol_id: session.protocol_id,
                        status: "completed".to_string(),
                        ping_type: "real".to_string(),
                        latency_ms: Some(rp_result.latency_ms as i32),
                        speed_bps: None,
                        ip_info: rp_result.ip_info.clone(),
                        error: None,
                    });
                }
                Ok((session, Err(e))) => {
                    buffer.push(PingResultUpdate {
                        session_id: session.id.clone(),
                        protocol_id: session.protocol_id,
                        status: "failed".to_string(),
                        ping_type: "real".to_string(),
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some(e.to_string()),
                    });
                }
                Err(_) => {}
            }
        }

        let _ = manager.stop().await;
        GroupOutcome::Done
    }

    fn push_failures(
        &self,
        items: &[(xray_tui_db::models::EndpointRow, PingSession, u16)],
        error: &str,
        buffer: &mut Vec<PingResultUpdate>,
    ) {
        for (_profile, session, _port) in items {
            buffer.push(PingResultUpdate {
                session_id: session.id.clone(),
                protocol_id: session.protocol_id,
                status: "failed".to_string(),
                ping_type: "real".to_string(),
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some(error.to_string()),
            });
        }
    }
}

/// Dispatch a batch of real-ping sessions through multi-inbound config.
/// Groups profiles by `core_type` (xray/sing-box), builds multi-inbound config,
/// starts one core per group, fires concurrent HTTP pings, collects results.
/// Tracks `completed_endpoints` for wave-ordering skip logic.
/// Returns number of flushed results.
async fn dispatch_real_ping_batch(
    sessions: &[xray_tui_db::models::PingSession],
    db: &Arc<Database>,
    tx: &tokio::sync::mpsc::Sender<CoreEvent>,
    progress: &(AtomicU16, AtomicU16),
    completed_endpoints: &Arc<std::sync::Mutex<HashSet<(String, u16)>>>,
    dedup_endpoints: bool,
    batch_id: &str,
    port_allocator: Arc<AtomicU16>,
    proxy_addr: &str,
    bin_dir: &std::path::Path,
    bin_configs_dir: &std::path::Path,
    real_ping_concurrency: usize,
    real_ping_timeout: std::time::Duration,
    real_ping_retries: u32,
    ping_url: &str,
    ip_api_url: &str,
    stop_flag: &AtomicBool,
    buffer: &mut Vec<PingResultUpdate>,
    log_tx: Option<tokio::sync::mpsc::Sender<String>>,
) -> u16 {
    let mut xray_items: Vec<(
        xray_tui_db::models::EndpointRow,
        xray_tui_db::models::PingSession,
        u16,
    )> = Vec::new();
    let mut singbox_items: Vec<(
        xray_tui_db::models::EndpointRow,
        xray_tui_db::models::PingSession,
        u16,
    )> = Vec::new();

    for session in sessions {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        let profile = if let Ok(Some(p)) = db.get_endpoint_by_protocol_id(session.protocol_id).await
        {
            p
        } else {
            buffer.push(PingResultUpdate {
                session_id: session.id.clone(),
                protocol_id: session.protocol_id,
                status: "failed".to_string(),
                ping_type: "real".to_string(),
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some("Endpoint not found".to_string()),
            });
            let _ = tx.try_send(CoreEvent::SpeedTestResult {
                protocol_id: session.protocol_id,
                test_type: TestType::RealPing,
                latency_ms: None,
                speed_bps: None,
                ip_info: None,
                error: Some("Endpoint not found".to_string()),
            });
            continue;
        };
        let proto = Protocol::try_from_i32(session.config_type).unwrap_or(Protocol::Custom);
        let core_type =
            resolve_core(proto, None, shadowsocks_method(profile.active_protocol()).as_deref());
        // Draw from the pool's shared allocator — batch and single pings can
        // never collide on a port.
        let assigned_port = port_allocator.fetch_add(1, Ordering::Relaxed);

        match core_type {
            CoreType::Xray => xray_items.push((profile, session.clone(), assigned_port)),
            CoreType::SingBox => singbox_items.push((profile, session.clone(), assigned_port)),
            CoreType::Auto => xray_items.push((profile, session.clone(), assigned_port)),
        }
    }

    // Run each core-type group with halving retry on group-level failures
    // (config build / core start / no inbound ready): split the group in half
    // and retest until the failure resolves or bottoms out at a per-profile
    // core — v2rayN's page-halving pattern. Per-profile failures are item
    // level and never retried.
    let ctx = RealPingBatchCtx {
        tx,
        completed_endpoints,
        dedup_endpoints,
        proxy_addr,
        bin_dir,
        bin_configs_dir,
        real_ping_concurrency,
        real_ping_timeout,
        real_ping_retries,
        ping_url,
        ip_api_url,
        stop_flag,
        log_tx,
    };
    for (core_type, items) in [
        (CoreType::Xray, &xray_items),
        (CoreType::SingBox, &singbox_items),
    ] {
        if items.is_empty() || stop_flag.load(Ordering::Relaxed) {
            continue;
        }
        ctx.dispatch_group(core_type, items, buffer).await;
    }

    let flushed = buffer.len() as u16;
    batch_upsert_buffer(batch_id, db, buffer).await;
    // Clean up sessions stranded by race: demoted to real/queued while
    // endpoint already completed in this batch dispatch. Only for
    // all-visible batches — endpoint-scoped batches must real-ping EVERY
    // protocol (dedup=false), so one success must not cancel its siblings
    // before their waves run.
    if dedup_endpoints && let Err(e) = db.cancel_stranded_real_pings(batch_id).await {
        tracing::warn!(target: "ops::ping", "cancel_stranded_real_pings: {e}");
    }
    progress.1.fetch_add(flushed, Ordering::Relaxed);
    flushed
}

/// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
///
/// Two-phase batch ping: Fast Ping (TCP/UDP/QUIC handshake), then optional Real Ping.
/// `profile_order` is the explicit `(protocol_id, sort_order)` list to test;
/// `dedup_endpoints` gates the real-ping endpoint dedup (all-visible batches
/// dedup; endpoint-scoped batches real-ping every protocol of one endpoint).
#[allow(clippy::needless_collect)]
pub fn start_sieve(
    state: &mut AppState,
    real_ping_enabled: bool,
    profile_order: Vec<(i64, i32)>,
    dedup_endpoints: bool,
) {
    if profile_order.is_empty() {
        state.log_trace(
            "info",
            "tui::ops::ping",
            if real_ping_enabled {
                "No profiles to test"
            } else {
                "No profiles to ping"
            },
        );
        return;
    }

    let tx = match &state.core_event_tx {
        Some(tx) => tx.clone(),
        None => return,
    };
    let state_log_tx = state.core_log_tx.clone();
    let db = state.db.clone();
    let stop_flag = state.speed_test_stop.clone();
    let batch_id = uuid::Uuid::new_v4().to_string();
    let page_size = state.config.speed_test.batch_page_size.max(1);

    let fast_timeout = *state.config.speed_test.tcp_timeout_secs;
    let real_ping_timeout = *state.config.speed_test.real_ping_timeout_secs;
    let retries = state.config.speed_test.real_ping_retries;
    let ping_url = state.config.speed_test.ping_url.clone();
    let ip_api_url = state.config.speed_test.ip_api_url.clone();
    let proxy_addr = state.config.inbound.listen.clone();

    let config_dir_path = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");
    let bin_dir = config_dir_path.join("bin");
    let bin_configs_dir = config_dir_path.join("binConfigs");
    let real_ping_concurrency = state.config.speed_test.real_ping_concurrency.max(1);
    let fast_ping_concurrency = state.config.speed_test.fast_ping_concurrency.max(1);

    // Phase 2 (real ping) draws ports from the pool's shared allocator so batch
    // cores and a warm pooled core can never collide. Create the pool here
    // (before the spawn) — `state` isn't available inside the spawned task.
    let pool = get_or_create_pool(state);

    // Phase 2 wave-ordered window: profiles to accumulate before waking Phase 2
    let real_ping_window = state.config.speed_test.real_ping_window.max(1);

    // Set up shared batch progress for status bar display
    #[allow(clippy::cast_possible_truncation)]
    let total_count = profile_order.len() as u16;
    let progress = Arc::new((AtomicU16::new(total_count), AtomicU16::new(0)));
    state.batch_progress = Some(progress.clone());
    tokio::spawn(async move {
        // 1. Snapshot visible profiles into ping_sessions table
        let count = db
            .create_ping_batch(&batch_id, None, Some(&profile_order))
            .await;
        let count = match count {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(target: "tui::ops::ping", "create_ping_batch failed: {e}");
                progress.0.store(0, Ordering::Relaxed);
                let _ = tx.try_send(CoreEvent::BatchProgress {
                    total: 0,
                    completed: 0,
                });
                return;
            }
        };
        if count == 0 {
            tracing::warn!(target: "tui::ops::ping", "create_ping_batch returned 0 — no matching profile_cores?");
            progress.0.store(0, Ordering::Relaxed);
            let _ = tx.try_send(CoreEvent::BatchProgress {
                total: 0,
                completed: 0,
            });
            return;
        }
        // Adjust progress total to match actual inserted count
        #[allow(clippy::cast_possible_truncation)]
        if (count as u16) != total_count {
            progress.0.store(count as u16, Ordering::Relaxed);
        }

        let mut buffer: Vec<PingResultUpdate> = Vec::new();
        let mut stall_count = 0u32;
        // Phase 1 dedup cache: key=(addr, port) → (latency_ms, error)
        let mut fast_cache: std::collections::HashMap<
            (String, u16),
            (Option<i32>, Option<String>),
        > = std::collections::HashMap::new();

        // 2. Initialize fast ping manager (Phase 2 uses multi-inbound directly, no RealPingManager)
        let fmgr = xray_tui_core::FastPingManager::new(fast_timeout);

        // ── Shared state for concurrent Phase 1+2 ──
        // Endpoints that had a successful real ping — no more profiles on these need testing.
        let completed_endpoints: Arc<std::sync::Mutex<HashSet<(String, u16)>>> =
            Arc::new(std::sync::Mutex::new(HashSet::new()));
        let mut accumulated_successes = 0usize;

        // Phase 2 consumer — spawned first so it's ready for wake-up from Phase 1
        let mut phase2_handle: Option<(mpsc::Sender<()>, tokio::task::JoinHandle<()>)> =
            if real_ping_enabled {
                let (phase2_tx, mut phase2_rx) = mpsc::channel::<()>(1);
                let phase2_completed = completed_endpoints.clone();
                let phase2_db = db.clone();
                let phase2_tx_ev = tx.clone();
                let phase2_stop = stop_flag.clone();
                let phase2_progress = progress.clone();
                let phase2_page_size = page_size;
                let phase2_timeout = real_ping_timeout;
                let phase2_retries = retries;
                let phase2_ping_url = ping_url.clone();
                let phase2_ip_api_url = ip_api_url.clone();
                let phase2_proxy_addr = proxy_addr.clone();
                let phase2_pool = pool.clone();
                let phase2_bin_dir = bin_dir.clone();
                let phase2_configs_dir = bin_configs_dir.clone();
                let phase2_concurrency = real_ping_concurrency;
                let phase2_batch_id = batch_id.clone();
                let phase2_dedup_endpoints = dedup_endpoints;
                let phase2_log_tx = state_log_tx.clone();

                let handle = tokio::spawn(async move {
                    // Keep `batch_active` set for the whole phase-2 task so
                    // concurrent single pings yield to multi-inbound.
                    let batch_flag = phase2_pool.batch_active_flag();
                    batch_flag.store(true, Ordering::Relaxed);
                    let _batch_active = BatchActiveGuard(batch_flag);

                    let mut buffer: Vec<PingResultUpdate> = Vec::new();

                    loop {
                        // Wake path: block for a Phase-1 nudge; None = final drain.
                        let woke = phase2_rx.recv().await == Some(());
                        // One full pass: waves 1..N, page_size chunks per wave.
                        // Empty wave terminates the pass — ranks are stable, so
                        // no queued row exists at any higher rank once wave k is
                        // empty. Late-demoted sessions are caught by the next
                        // pass, which restarts from wave 1.
                        loop {
                            let mut dispatched_any = false;
                            let mut wave = 1i64;
                            loop {
                                if phase2_stop.load(Ordering::Relaxed) {
                                    break;
                                }
                                let sessions = match phase2_db
                                    .get_batch_for_real_ping(
                                        &phase2_batch_id,
                                        wave,
                                        phase2_page_size,
                                        phase2_dedup_endpoints,
                                    )
                                    .await
                                {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!(target: "tui::ops::ping", "get_batch_for_real_ping failed: {e}");
                                        break;
                                    }
                                };
                                if sessions.is_empty() {
                                    break;
                                }
                                dispatched_any = true;
                                let flushed = dispatch_real_ping_batch(
                                    &sessions,
                                    &phase2_db,
                                    &phase2_tx_ev,
                                    &phase2_progress,
                                    &phase2_completed,
                                    phase2_dedup_endpoints,
                                    &phase2_batch_id,
                                    phase2_pool.port_allocator(),
                                    &phase2_proxy_addr,
                                    &phase2_bin_dir,
                                    &phase2_configs_dir,
                                    phase2_concurrency,
                                    phase2_timeout,
                                    phase2_retries,
                                    &phase2_ping_url,
                                    &phase2_ip_api_url,
                                    &phase2_stop,
                                    &mut buffer,
                                    phase2_log_tx.clone(),
                                )
                                .await;
                                if flushed == 0 {
                                    break;
                                }
                                wave += 1;
                            }
                            if !dispatched_any {
                                break;
                            }
                            // Coalesce: give concurrently-demoting Phase 1 a
                            // beat to land its sessions before re-passing.
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        }
                        if !woke {
                            break; // final drain complete — sender dropped
                        }
                    }

                    // Final buffer flush for any remaining Phase 2 results
                    batch_upsert_buffer(&phase2_batch_id, &phase2_db, &mut buffer).await;
                });

                Some((phase2_tx, handle))
            } else {
                None
            };

        // Sender clone for Phase 1 to wake up Phase 2
        let phase2_sender: Option<mpsc::Sender<()>> =
            phase2_handle.as_ref().map(|(tx, _)| tx.clone());

        // ── Phase 1: Fast Ping ──
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let sessions = db
                .get_batch_page_ready_for_fast_ping(&batch_id, page_size)
                .await;
            let Ok(sessions) = sessions else {
                tracing::error!(target: "tui::ops::ping", "get_batch_page_ready_for_fast_ping failed");
                progress.0.store(0, Ordering::Relaxed);
                break;
            };
            if sessions.is_empty() {
                break;
            }

            let outcomes =
                run_page_pings(&fmgr, &sessions, &fast_cache, fast_ping_concurrency).await;

            let mut demoted_count = 0u16;
            for (i, session) in sessions.iter().enumerate() {
                let _ = tx.try_send(CoreEvent::TestTypeUpdate {
                    protocol_id: session.protocol_id,
                    test_type: TestType::TcpPing,
                });

                let addr = match &session.address {
                    Some(a) => a.clone(),
                    None => continue,
                };
                let port = session.port.unwrap_or(0) as u16;
                if port == 0 {
                    continue;
                }

                // Track sessions demoted to Phase 2 (no buffer push this page)
                let mut demoted_here = false;

                // Dedup: skip ping if (addr, port) already tested in this page
                let cache_key = (addr.clone(), port);
                let pushed = match &outcomes[i] {
                    Some(PageOutcome::Cached(ms, err)) => {
                        if let Some(ms) = ms {
                            if real_ping_enabled {
                                let ep_done = completed_endpoints
                                    .lock()
                                    .unwrap()
                                    .contains(&(addr.clone(), port));
                                if ep_done {
                                    // Endpoint already succeeded in Phase 2 — cancel this profile
                                    buffer.push(PingResultUpdate {
                                        session_id: session.id.clone(),
                                        protocol_id: session.protocol_id,
                                        status: "cancelled".to_string(),
                                        ping_type: "fast".to_string(),
                                        latency_ms: None,
                                        speed_bps: None,
                                        ip_info: None,
                                        error: Some(
                                            "Endpoint already tested in Real Ping".to_string(),
                                        ),
                                    });
                                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                        protocol_id: session.protocol_id,
                                        test_type: TestType::TcpPing,
                                        latency_ms: None,
                                        speed_bps: None,
                                        ip_info: None,
                                        error: Some(
                                            "Endpoint already tested in Real Ping".to_string(),
                                        ),
                                    });
                                    demoted_here = false;
                                    false
                                } else {
                                    // Demote to real ping
                                    let _ = db
                                        .update_session_ping_type(&session.id, "real", "queued")
                                        .await;
                                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                        protocol_id: session.protocol_id,
                                        test_type: TestType::TcpPing,
                                        latency_ms: Some(*ms as u64),
                                        speed_bps: None,
                                        ip_info: None,
                                        error: None,
                                    });
                                    demoted_here = true;
                                    false
                                }
                            } else {
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "completed".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: Some(*ms),
                                    speed_bps: None,
                                    ip_info: None,
                                    error: None,
                                });
                                true
                            }
                        } else {
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                protocol_id: session.protocol_id,
                                status: "failed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: None,
                                speed_bps: None,
                                ip_info: None,
                                error: err.clone(),
                            });
                            true
                        }
                    }
                    Some(PageOutcome::Pinged(Ok(dur))) => {
                        #[allow(clippy::cast_possible_truncation)]
                        let ms = dur.as_millis() as i32;
                        if real_ping_enabled {
                            let ep_done = completed_endpoints
                                .lock()
                                .unwrap()
                                .contains(&(addr.clone(), port));
                            if ep_done {
                                // Endpoint already succeeded in Phase 2 — cancel
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "cancelled".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Endpoint already tested in Real Ping".to_string()),
                                });
                                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                    protocol_id: session.protocol_id,
                                    test_type: TestType::TcpPing,
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Endpoint already tested in Real Ping".to_string()),
                                });
                                fast_cache.insert(cache_key, (Some(ms), None));
                                demoted_here = false;
                                false
                            } else {
                                // Demote to real ping
                                let _ = db
                                    .update_session_ping_type(&session.id, "real", "queued")
                                    .await;
                                let _ = tx.try_send(CoreEvent::SpeedTestResult {
                                    protocol_id: session.protocol_id,
                                    test_type: TestType::TcpPing,
                                    latency_ms: Some(ms as u64),
                                    speed_bps: None,
                                    ip_info: None,
                                    error: None,
                                });
                                fast_cache.insert(cache_key, (Some(ms), None));
                                demoted_here = true;
                                false
                            }
                        } else {
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                protocol_id: session.protocol_id,
                                status: "completed".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: Some(ms),
                                speed_bps: None,
                                ip_info: None,
                                error: None,
                            });
                            fast_cache.insert(cache_key, (Some(ms), None));
                            true
                        }
                    }
                    Some(PageOutcome::Pinged(Err(
                        xray_tui_core::ping::PingError::NotSupported,
                    ))) => {
                        if real_ping_enabled {
                            // Check if endpoint already completed in Phase 2
                            let ep_done = completed_endpoints
                                .lock()
                                .unwrap()
                                .contains(&(addr.clone(), port));
                            if ep_done {
                                buffer.push(PingResultUpdate {
                                    session_id: session.id.clone(),
                                    protocol_id: session.protocol_id,
                                    status: "cancelled".to_string(),
                                    ping_type: "fast".to_string(),
                                    latency_ms: None,
                                    speed_bps: None,
                                    ip_info: None,
                                    error: Some("Endpoint already tested in Real Ping".to_string()),
                                });
                                demoted_here = false;
                                false
                            } else {
                                // Demote to real ping for Phase 2 — no buffer push
                                let _ = db
                                    .update_session_ping_type(&session.id, "real", "queued")
                                    .await;
                                // Don't cache — NotSupported is protocol-specific
                                demoted_here = true;
                                false
                            }
                        } else {
                            // No real ping phase — emit Cancelled immediately
                            buffer.push(PingResultUpdate {
                                session_id: session.id.clone(),
                                protocol_id: session.protocol_id,
                                status: "cancelled".to_string(),
                                ping_type: "fast".to_string(),
                                latency_ms: None,
                                speed_bps: None,
                                ip_info: None,
                                error: Some("Not supported by fast ping".to_string()),
                            });
                            true
                        }
                    }
                    Some(PageOutcome::Pinged(Err(e))) => {
                        buffer.push(PingResultUpdate {
                            session_id: session.id.clone(),
                            protocol_id: session.protocol_id,
                            status: "failed".to_string(),
                            ping_type: "fast".to_string(),
                            latency_ms: None,
                            speed_bps: None,
                            ip_info: None,
                            error: Some(e.to_string()),
                        });
                        fast_cache.insert(cache_key, (None, Some(e.to_string())));
                        true
                    }
                    None => continue,
                };
                // Send SpeedTestResult if we pushed to buffer
                // (demoted sessions already emitted above)
                if pushed {
                    let last = buffer.last().unwrap();
                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                        protocol_id: session.protocol_id,
                        test_type: TestType::TcpPing,
                        latency_ms: last.latency_ms.map(|v| v as u64),
                        speed_bps: None,
                        ip_info: None,
                        error: last.error.clone(),
                    });
                }
                if demoted_here {
                    demoted_count += 1;
                    accumulated_successes += 1;
                }
            }
            // Flush buffer at page boundary
            let flushed = buffer.len() as u16;
            let flush_ok = batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);

            // Stall guard: break if too many consecutive iterations with NO work
            // (demoted sessions count as progress — they'll be picked up by Phase 2).
            if flushed == 0 && demoted_count == 0 {
                stall_count += 1;
            } else if !flush_ok {
                stall_count += 1;
            } else {
                stall_count = 0;
            }
            if stall_count > 3 {
                tracing::warn!(
                    target: "tui::ops::ping",
                    "Phase 1 stall guard triggered after {stall_count} iterations",
                );
                break;
            }
            // Wake up Phase 2 if enough demoted sessions accumulated
            if accumulated_successes >= real_ping_window {
                if let Some(ref tx) = phase2_sender {
                    let _ = tx.try_send(());
                }
                accumulated_successes = 0;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // After Phase 1: mark any remaining 'queued' fast sessions as failed
        // (orphan profiles with no endpoint or empty address/port)
        if let Ok(remaining) = db
            .get_batch_page_ready_for_fast_ping(&batch_id, 65536)
            .await
        {
            for session in &remaining {
                buffer.push(PingResultUpdate {
                    session_id: session.id.clone(),
                    protocol_id: session.protocol_id,
                    status: "failed".to_string(),
                    ping_type: "fast".to_string(),
                    latency_ms: None,
                    speed_bps: None,
                    ip_info: None,
                    error: Some("No address/port".to_string()),
                });
            }
            let flushed = remaining.len() as u16;
            batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
            progress.1.fetch_add(flushed, Ordering::Relaxed);
        }

        // ── Phase 2: Real Ping — running concurrently as spawned task ──
        // Signal Phase 2 to drain (drop sender) and wait for completion.
        if let Some((_sender, handle)) = phase2_handle.take() {
            drop(_sender);
            let _ = handle.await;
        }

        // Final flush + cancel remaining + cleanup
        if stop_flag.load(Ordering::Relaxed) {
            // Emit Cancelled events for any non-terminal sessions
            if let Ok(sessions) = db.get_ping_sessions_by_batch(&batch_id, 65536, 0).await {
                for session in &sessions {
                    if session.status == "completed"
                        || session.status == "failed"
                        || session.status == "cancelled"
                    {
                        continue;
                    }
                    let test_type = if session.ping_type == "real" {
                        TestType::RealPing
                    } else {
                        TestType::TcpPing
                    };
                    let _ = tx.try_send(CoreEvent::SpeedTestResult {
                        protocol_id: session.protocol_id,
                        test_type,
                        latency_ms: None,
                        speed_bps: None,
                        ip_info: None,
                        error: Some("Cancelled".to_string()),
                    });
                }
            }
            let _ = db.cancel_ping_batch(&batch_id).await;
        }
        let flushed = buffer.len() as u16;
        batch_upsert_buffer(&batch_id, &db, &mut buffer).await;
        progress.1.fetch_add(flushed, Ordering::Relaxed);
        let _ = db.cleanup_ping_batch(&batch_id).await;
        progress.0.store(0, Ordering::Relaxed); // signal batch is done; enables stale cleanup
        let _ = tx.try_send(CoreEvent::BatchProgress {
            total: 0,
            completed: 0,
        });
    });
}

/// Remove profiles whose extension.delay == Some(-1) (failed TCP ping).
pub async fn remove_failed_servers(state: &mut AppState) {
    let to_remove: Vec<i64> = state
        .endpoints
        .iter()
        .filter(|r| r.extensions.values().any(|e| e.delay == Some(-1)))
        .map(|r| r.endpoint.id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn test_session(id: &str, addr: String, port: u16, config_type: i32) -> PingSession {
        PingSession {
            id: id.to_string(),
            batch_id: "test-batch".to_string(),
            protocol_id: 1,
            config_type,
            core_type: "auto".to_string(),
            address: Some(addr),
            port: Some(port as i32),
            triplet_rank: 0,
            ping_type: "fast".to_string(),
            status: "queued".to_string(),
            latency_ms: None,
            speed_bps: None,
            ip_info: None,
            error: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Binds a localhost listener and returns (port, accepted-connection counter).
    /// The counter is bumped from the accept task; the test polls it after a
    /// settle sleep.
    async fn spawn_listener_counter() -> (u16, Arc<AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let task_count = count.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((_stream, _)) => {
                        task_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => break,
                }
            }
        });
        (port, count)
    }

    #[tokio::test]
    async fn run_page_pings_all_reachable() {
        let fmgr = xray_tui_core::FastPingManager::new(std::time::Duration::from_secs(5));
        let mut sessions = Vec::new();
        let mut counts = Vec::new();
        for i in 0..10 {
            let (port, count) = spawn_listener_counter().await;
            counts.push(count);
            sessions.push(test_session(
                &format!("s{i}"),
                "127.0.0.1".to_string(),
                port,
                Protocol::Vmess.to_i32(),
            ));
        }
        let outcomes = run_page_pings(&fmgr, &sessions, &HashMap::new(), 8).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(outcomes.len(), 10);
        for o in &outcomes {
            assert!(matches!(o, Some(PageOutcome::Pinged(Ok(_)))));
        }
        for c in &counts {
            assert_eq!(c.load(Ordering::SeqCst), 1, "one ping per distinct key");
        }
    }

    #[tokio::test]
    async fn run_page_pings_dedups_same_key() {
        let fmgr = xray_tui_core::FastPingManager::new(std::time::Duration::from_secs(5));
        let (port_a, count_a) = spawn_listener_counter().await;
        let (port_b, count_b) = spawn_listener_counter().await;
        let sessions = vec![
            test_session(
                "s0",
                "127.0.0.1".to_string(),
                port_a,
                Protocol::Vmess.to_i32(),
            ),
            test_session(
                "s1",
                "127.0.0.1".to_string(),
                port_a,
                Protocol::Vmess.to_i32(),
            ),
            test_session(
                "s2",
                "127.0.0.1".to_string(),
                port_a,
                Protocol::Vmess.to_i32(),
            ),
            test_session(
                "s3",
                "127.0.0.1".to_string(),
                port_b,
                Protocol::Vmess.to_i32(),
            ),
        ];
        let outcomes = run_page_pings(&fmgr, &sessions, &HashMap::new(), 4).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(count_a.load(Ordering::SeqCst), 1, "same key pings once");
        assert_eq!(count_b.load(Ordering::SeqCst), 1, "distinct key pings once");
        for o in &outcomes {
            assert!(matches!(o, Some(PageOutcome::Pinged(Ok(_)))));
        }
    }

    #[tokio::test]
    async fn run_page_pings_respects_concurrency() {
        // Unroutable TEST-NET-1 addresses with a short connect timeout: each
        // connect blocks until the adapter timeout, so wall time proves the cap
        // — 20 pings at cap 5 finish in ~4 × timeout, not 20 × timeout. (In
        // environments where the gateway RSTs or has no route, connects fail
        // instantly and the wall-time bound passes vacuously.)
        let timeout = std::time::Duration::from_millis(400);
        let fmgr = xray_tui_core::FastPingManager::new(timeout);
        let sessions: Vec<PingSession> = (0..20)
            .map(|i| {
                test_session(
                    &format!("s{i}"),
                    format!("192.0.2.{}", i % 10),
                    10000 + i as u16,
                    Protocol::Vmess.to_i32(),
                )
            })
            .collect();
        let start = std::time::Instant::now();
        let outcomes = run_page_pings(&fmgr, &sessions, &HashMap::new(), 5).await;
        let elapsed = start.elapsed();
        // ceil(20/5) × 400ms ≈ 1.6s capped; 20 × 400ms = 8s sequential.
        assert!(
            elapsed < std::time::Duration::from_millis(3000),
            "wall time {elapsed:?} — cap not applied?"
        );
        assert_eq!(outcomes.len(), 20);
        for o in &outcomes {
            assert!(matches!(o, Some(PageOutcome::Pinged(Err(_)))));
        }
    }

    #[tokio::test]
    async fn run_page_pings_cached() {
        let fmgr = xray_tui_core::FastPingManager::new(std::time::Duration::from_secs(5));
        let (port_a, count_a) = spawn_listener_counter().await;
        let sessions = vec![test_session(
            "s0",
            "127.0.0.1".to_string(),
            port_a,
            Protocol::Vmess.to_i32(),
        )];
        let cache: HashMap<(String, u16), (Option<i32>, Option<String>)> =
            [(("127.0.0.1".to_string(), port_a), (Some(12), None))]
                .into_iter()
                .collect();
        let outcomes = run_page_pings(&fmgr, &sessions, &cache, 4).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(
            count_a.load(Ordering::SeqCst),
            0,
            "cached keys must not ping"
        );
        assert!(matches!(
            &outcomes[0],
            Some(PageOutcome::Cached(Some(12), None))
        ));
    }
}
