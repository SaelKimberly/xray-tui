//! CorePool — keeps a single warm core process for single-ping reuse.
//!
//! For **batch** real ping, the multi-inbound approach (one core per page) is
//! used instead — see [`crate::config_builder::ConfigBuilder::build_multi`].
//!
//! ## Dual reload strategy
//!
//! | Core      | Reload method                             | Time   |
//! |-----------|-------------------------------------------|--------|
//! | sing-box  | Write config → SIGHUP → poll SOCKS5 ready | ~200ms |
//! | xray-core | Stop → rewrite config → start             | ~500ms |
//!
//! xray-core does not support SIGHUP (only SIGTERM/SIGINT).

use crate::config_builder::{BuildParams, ConfigBuilder};
use crate::core_type::CoreType;
use crate::process::{CoreManager, RealCoreManager};
use crate::protocol::Protocol;
use crate::protocol_core_mapping::resolve_core;
use crate::speed_test::wait_for_socks5;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow};

/// Default idle time before the pooled core is killed.
const POOL_TTL: Duration = Duration::from_secs(30);

/// Deadline for SOCKS5 readiness after reload/restart.
const READY_DEADLINE: Duration = Duration::from_secs(5);

/// A single warm core process reused across sequential single-ping calls.
struct PooledCore {
    core_type: CoreType,
    port: u16,
    manager: Box<dyn CoreManager>,
    last_used: Instant,
}

/// Manages a lazily-created, optionally-warm core for single real ping.
///
/// On first use, spawns the core (pay startup cost once). Subsequent single
/// pings reuse the same core via SIGHUP (sing-box) or stop+restart (xray-core).
/// After [`POOL_TTL`] of inactivity, the core is killed to free resources.
pub struct CorePool {
    core: Mutex<Option<PooledCore>>,
    bin_dir: PathBuf,
    bin_configs_dir: PathBuf,
    proxy_addr: String,
    base_proxy_port: u16,
    next_port: Arc<AtomicU16>,
    /// Set to `true` when a batch operation is active — pool yields to multi-inbound.
    batch_active: Arc<AtomicBool>,
}

impl CorePool {
    /// Create a new pool. No core is spawned until first use.
    #[must_use]
    pub fn new(
        bin_dir: PathBuf,
        bin_configs_dir: PathBuf,
        proxy_addr: String,
        base_proxy_port: u16,
    ) -> Self {
        Self {
            core: Mutex::new(None),
            bin_dir,
            bin_configs_dir,
            proxy_addr,
            base_proxy_port,
            next_port: Arc::new(AtomicU16::new(base_proxy_port + 1)),
            batch_active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Return a shared flag that batch operations can set to signal the pool
    /// to yield (no pool reuse during batch).
    #[must_use]
    pub fn batch_active_flag(&self) -> Arc<AtomicBool> {
        self.batch_active.clone()
    }

    /// Shared monotonically-increasing port allocator. Batch real ping and the
    /// pool both draw from this counter so they can never collide.
    #[must_use]
    pub fn port_allocator(&self) -> Arc<AtomicU16> {
        self.next_port.clone()
    }

    /// Run a real ping for a single profile, using the pool if possible.
    ///
    /// If a warm core of the correct type exists and is within TTL, reuse it.
    /// Otherwise spawn a fresh core. After the ping, the core stays alive for
    /// reuse (up to [`POOL_TTL`]).
    pub async fn ping(
        &self,
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        ping_url: &str,
        ip_api_url: &str,
        timeout: Duration,
        retries: u32,
    ) -> super::super::PingResult {
        // If batch is active, don't use the pool — batch uses multi-inbound.
        if self.batch_active.load(Ordering::Relaxed) {
            return self
                .fresh_ping(endpoint, protocol, ping_url, ip_api_url, timeout, retries)
                .await;
        }
        self.pooled_ping(endpoint, protocol, ping_url, ip_api_url, timeout, retries)
            .await
    }

    /// Pool-aware ping: acquire warm core or spawn fresh, reuse after.
    async fn pooled_ping(
        &self,
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        ping_url: &str,
        ip_api_url: &str,
        timeout: Duration,
        retries: u32,
    ) -> super::super::PingResult {
        let config_type = protocol.config_type;
        let proto = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        let needed_core = resolve_core(proto, None);

        // Hold the pool lock across the entire reuse path — including the HTTP
        // ping itself. A concurrent single ping must not SIGHUP/reload the
        // same core while requests are in flight (latency measured against the
        // wrong server, connection killed), and TTL eviction must not kill it
        // mid-ping. The guard drops at the end of this block, after the result
        // is built; only then does the TTL reaper run.
        let result = {
            let mut guard = self.core.lock().await;

            // Check if existing pooled core can be reused
            let should_reuse = guard
                .as_ref()
                .is_some_and(|c| c.core_type == needed_core && c.last_used.elapsed() < POOL_TTL);

            if should_reuse {
                let pooled = guard.as_mut().unwrap();
                pooled.last_used = Instant::now();
                let port = pooled.port;
                let core_type = pooled.core_type;

                // Build single-profile config
                let params = self.build_single_params(port);
                let dns = self.default_dns();
                let backend_config =
                    match ConfigBuilder::build(endpoint, protocol, core_type, &params, &[], &dns) {
                        Ok(c) => c,
                        Err(e) => {
                            // Config build failed — evict pooled core and return error
                            let old = guard.take();
                            if let Some(mut p) = old {
                                let _ = p.manager.stop().await;
                            }
                            return super::super::PingResult {
                                profile_key: super::super::ProfileKey {
                                    config_type,
                                    address: endpoint.host.clone(),
                                    port: endpoint.port as u16,
                                },
                                latency_ms: None,
                                ip_info: None,
                                error: Some(format!("Build config: {e}")),
                            };
                        }
                    };

                if core_type == CoreType::SingBox {
                    // SIGHUP reload path
                    if let Err(e) = pooled.manager.rewrite_config(&backend_config, None).await {
                        tracing::warn!(target: "core::pool", "rewrite_config failed: {e}, restarting");
                        let _ = pooled.manager.stop().await;
                        match pooled
                            .manager
                            .start(core_type, &backend_config, &self.bin_path(core_type), None)
                            .await
                        {
                            Ok(()) => {}
                            Err(e2) => {
                                let _ = guard.take();
                                return super::super::PingResult {
                                    profile_key: super::super::ProfileKey {
                                        config_type,
                                        address: endpoint.host.clone(),
                                        port: endpoint.port as u16,
                                    },
                                    latency_ms: None,
                                    ip_info: None,
                                    error: Some(format!("Core restart: {e2}")),
                                };
                            }
                        }
                    } else if let Err(e) = pooled.manager.sighup_reload() {
                        tracing::warn!(target: "core::pool", "SIGHUP failed: {e}, restarting");
                        let _ = pooled.manager.stop().await;
                        match pooled
                            .manager
                            .start(core_type, &backend_config, &self.bin_path(core_type), None)
                            .await
                        {
                            Ok(()) => {}
                            Err(e2) => {
                                let _ = guard.take();
                                return super::super::PingResult {
                                    profile_key: super::super::ProfileKey {
                                        config_type,
                                        address: endpoint.host.clone(),
                                        port: endpoint.port as u16,
                                    },
                                    latency_ms: None,
                                    ip_info: None,
                                    error: Some(format!("Core restart: {e2}")),
                                };
                            }
                        }
                    }
                } else {
                    // xray-core: stop + restart
                    let _ = pooled.manager.stop().await;
                    if let Err(e) = pooled
                        .manager
                        .start(core_type, &backend_config, &self.bin_path(core_type), None)
                        .await
                    {
                        let _ = guard.take();
                        return super::super::PingResult {
                            profile_key: super::super::ProfileKey {
                                config_type,
                                address: endpoint.host.clone(),
                                port: endpoint.port as u16,
                            },
                            latency_ms: None,
                            ip_info: None,
                            error: Some(format!("Core restart: {e}")),
                        };
                    }
                }

                // Wait for SOCKS5 readiness — evict core on failure
                if wait_for_socks5(&self.proxy_addr, port, READY_DEADLINE)
                    .await
                    .is_err()
                {
                    let _ = guard.take();
                    return super::super::PingResult {
                        profile_key: super::super::ProfileKey {
                            config_type,
                            address: endpoint.host.clone(),
                            port: endpoint.port as u16,
                        },
                        latency_ms: None,
                        ip_info: None,
                        error: Some("Core not ready: SOCKS5 timeout".to_string()),
                    };
                }

                // The guard is still held here: the HTTP ping runs under the
                // pool lock, so a concurrent single ping cannot reload this
                // core mid-flight, and TTL eviction cannot kill it until the
                // ping completes.
                let rp_result = crate::speed_test::real_ping(
                    &self.proxy_addr,
                    port,
                    ping_url,
                    ip_api_url,
                    timeout,
                    retries,
                )
                .await;
                self.build_result(config_type, endpoint, rp_result)
            } else {
                // Evict stale or wrong-type core
                if let Some(mut old) = guard.take() {
                    let _ = old.manager.stop().await;
                }
                drop(guard);
                // Fall through to fresh ping
                self.fresh_ping_and_cache(
                    endpoint,
                    protocol,
                    needed_core,
                    ping_url,
                    ip_api_url,
                    timeout,
                    retries,
                )
                .await
            }
        };

        // TTL reaper: if a reuse ping pushed the pooled core past POOL_TTL,
        // stop+evict it now so its port is freed promptly (rather than
        // lingering until the next reuse check). Runs only after the guard
        // above has been released, so it can never kill a core mid-ping;
        // after a fresh ping the core was just cached (last_used = now), so
        // this is a no-op.
        self.reap_stale_core().await;

        result
    }

    /// Fresh ping: spawn new core, test, then cache it in the pool.
    async fn fresh_ping_and_cache(
        &self,
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        core_type: CoreType,
        ping_url: &str,
        ip_api_url: &str,
        timeout: Duration,
        retries: u32,
    ) -> super::super::PingResult {
        let config_type = protocol.config_type;
        let port = self.next_port.fetch_add(1, Ordering::Relaxed);

        let (log_line_tx, mut log_rx) = tokio::sync::mpsc::channel(512);
        tokio::spawn(async move {
            while let Some(line) = log_rx.recv().await {
                tracing::warn!(target: "core::real_ping::pool", "{line}");
            }
        });
        let mut manager = RealCoreManager::new(self.bin_configs_dir.clone(), log_line_tx);
        let params = self.build_single_params(port);
        let dns = self.default_dns();

        let backend_config =
            match ConfigBuilder::build(endpoint, protocol, core_type, &params, &[], &dns) {
                Ok(c) => c,
                Err(e) => {
                    return super::super::PingResult {
                        profile_key: super::super::ProfileKey {
                            config_type,
                            address: endpoint.host.clone(),
                            port: endpoint.port as u16,
                        },
                        latency_ms: None,
                        ip_info: None,
                        error: Some(format!("Build config: {e}")),
                    };
                }
            };

        let bin_path = match crate::bin_manager::find_binary(core_type, &self.bin_dir) {
            Some(p) => p,
            None => {
                return super::super::PingResult {
                    profile_key: super::super::ProfileKey {
                        config_type,
                        address: endpoint.host.clone(),
                        port: endpoint.port as u16,
                    },
                    latency_ms: None,
                    ip_info: None,
                    error: Some("Binary not found".to_string()),
                };
            }
        };

        if let Err(e) = manager
            .start(core_type, &backend_config, &bin_path, None)
            .await
        {
            return super::super::PingResult {
                profile_key: super::super::ProfileKey {
                    config_type,
                    address: endpoint.host.clone(),
                    port: endpoint.port as u16,
                },
                latency_ms: None,
                ip_info: None,
                error: Some(format!("Core start: {e}")),
            };
        }

        // Wait for SOCKS5 readiness — return error on failure
        if wait_for_socks5(&self.proxy_addr, port, READY_DEADLINE)
            .await
            .is_err()
        {
            return super::super::PingResult {
                profile_key: super::super::ProfileKey {
                    config_type,
                    address: endpoint.host.clone(),
                    port: endpoint.port as u16,
                },
                latency_ms: None,
                ip_info: None,
                error: Some("Core not ready: SOCKS5 timeout".to_string()),
            };
        }

        let rp_result = crate::speed_test::real_ping(
            &self.proxy_addr,
            port,
            ping_url,
            ip_api_url,
            timeout,
            retries,
        )
        .await;

        // Cache the core for reuse
        let mut guard = self.core.lock().await;
        // Only cache if nothing else was cached in the meantime
        if guard.is_none() {
            *guard = Some(PooledCore {
                core_type,
                port,
                manager: Box::new(manager),
                last_used: Instant::now(),
            });
        } else {
            // Another task already cached — kill this extra core
            let _ = manager.stop().await;
        }

        self.build_result(config_type, endpoint, rp_result)
    }

    /// Fresh ping with caching (reuses warm core for future pings).
    async fn fresh_ping(
        &self,
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        ping_url: &str,
        ip_api_url: &str,
        timeout: Duration,
        retries: u32,
    ) -> super::super::PingResult {
        let config_type = protocol.config_type;
        let proto = Protocol::try_from_i32(config_type).unwrap_or(Protocol::Custom);
        let core_type = resolve_core(proto, None);
        self.fresh_ping_and_cache(
            endpoint, protocol, core_type, ping_url, ip_api_url, timeout, retries,
        )
        .await
    }

    /// Stop+evict the pooled core if it has been idle past [`POOL_TTL`],
    /// freeing its port promptly. The under-TTL case leaves the warm core in
    /// the pool untouched for reuse.
    async fn reap_stale_core(&self) {
        let mut guard = self.core.lock().await;
        // Inspect BEFORE taking — the normal (under-TTL) case must leave the
        // warm core in the pool for reuse.
        if guard
            .as_ref()
            .is_some_and(|pooled| pooled.last_used.elapsed() >= POOL_TTL)
            && let Some(mut stale) = guard.take()
        {
            let _ = stale.manager.stop().await;
        }
    }

    /// Build `BuildParams` for a single-profile pool core.
    fn build_single_params(&self, port: u16) -> BuildParams {
        BuildParams {
            v2ray_api_enabled: false,
            clash_api_enabled: false,
            log_level: "error".to_string(),
            socks_port: port,
            http_port: None,
            listen: self.proxy_addr.clone(),
            sniffing: false,
            clash_api_port: None,
            mux: None,
            clash_mixin: None,
            skip_cert_verify: false,
        }
    }

    fn default_dns(&self) -> DnsSetting {
        DnsSetting {
            id: "default".to_string(),
            name: None,
            servers: None,
            hosts: None,
            query_strategy: None,
            disable_cache: None,
            disable_fallback: None,
            client_ip: None,
            cache_ttl_secs: None,
        }
    }

    fn bin_path(&self, core_type: CoreType) -> PathBuf {
        crate::bin_manager::find_binary(core_type, &self.bin_dir).unwrap_or_default()
    }

    fn build_result(
        &self,
        config_type: i32,
        endpoint: &Endpoint,
        rp_result: Result<crate::speed_test::RealPingResult, crate::speed_test::SpeedTestError>,
    ) -> super::super::PingResult {
        match rp_result {
            Ok(rp) => super::super::PingResult {
                profile_key: super::super::ProfileKey {
                    config_type,
                    address: endpoint.host.clone(),
                    port: endpoint.port as u16,
                },
                latency_ms: Some(rp.latency_ms),
                ip_info: rp.ip_info,
                error: None,
            },
            Err(e) => super::super::PingResult {
                profile_key: super::super::ProfileKey {
                    config_type,
                    address: endpoint.host.clone(),
                    port: endpoint.port as u16,
                },
                latency_ms: None,
                ip_info: None,
                error: Some(e.to_string()),
            },
        }
    }
}

impl Drop for CorePool {
    fn drop(&mut self) {
        // CoreManager's own Drop kills the process via kill_on_drop.
        // Just ensure we don't leak the temp dir by taking the pooled core.
        if let Ok(mut guard) = self.core.try_lock() {
            drop(guard.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_builder::BackendConfig;
    use crate::process::ProcessError;
    use async_trait::async_trait;
    use std::sync::atomic::AtomicUsize;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::{Notify, watch};

    /// Records `stop()` calls instead of managing a real core process.
    struct FakeManager {
        stopped: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl CoreManager for FakeManager {
        async fn start(
            &mut self,
            _core_type: CoreType,
            _config: &BackendConfig,
            _binary_path: &std::path::Path,
            _clash_mixin: Option<&serde_json::Value>,
        ) -> Result<(), ProcessError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), ProcessError> {
            self.stopped.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn is_running(&self) -> bool {
            true
        }

        fn running_core_type(&self) -> Option<CoreType> {
            Some(CoreType::Xray)
        }

        fn sighup_reload(&self) -> Result<u32, ProcessError> {
            Ok(0)
        }

        async fn rewrite_config(
            &self,
            _config: &BackendConfig,
            _clash_mixin: Option<&serde_json::Value>,
        ) -> Result<(), ProcessError> {
            Ok(())
        }
    }

    fn pool_with_core(last_used: Instant) -> (CorePool, Arc<AtomicUsize>) {
        let pool = CorePool::new(
            PathBuf::from("/tmp/not-used-bin"),
            PathBuf::from("/tmp/not-used-configs"),
            "127.0.0.1".to_string(),
            10800,
        );
        let stopped = Arc::new(AtomicUsize::new(0));
        {
            let mut guard = pool.core.try_lock().unwrap();
            *guard = Some(PooledCore {
                core_type: CoreType::Xray,
                port: 10801,
                manager: Box::new(FakeManager {
                    stopped: stopped.clone(),
                }),
                last_used,
            });
        }
        (pool, stopped)
    }

    #[tokio::test]
    async fn port_allocator_is_monotonic_and_shared() {
        let pool = CorePool::new(
            PathBuf::from("/tmp/not-used-bin"),
            PathBuf::from("/tmp/not-used-configs"),
            "127.0.0.1".to_string(),
            10800,
        );
        let a = pool.port_allocator();
        let b = pool.port_allocator();
        assert!(Arc::ptr_eq(&a, &b), "same shared counter");
        let p1 = a.fetch_add(1, Ordering::Relaxed);
        let p2 = b.fetch_add(1, Ordering::Relaxed);
        assert_eq!(p2, p1 + 1);
    }

    #[tokio::test]
    async fn reap_keeps_fresh_core_in_pool() {
        // Regression: an under-TTL pooled core must survive the reaper so a
        // second reuse doesn't pay the full cold-start cost.
        let (pool, stopped) = pool_with_core(Instant::now());
        pool.reap_stale_core().await;
        let guard = pool.core.lock().await;
        assert!(guard.is_some(), "under-TTL core must survive the TTL reaper");
        drop(guard);
        assert_eq!(stopped.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn reap_evicts_stale_core() {
        // An over-TTL core is stopped and evicted, freeing its port promptly.
        let (pool, stopped) = pool_with_core(Instant::now() - POOL_TTL - Duration::from_secs(1));
        pool.reap_stale_core().await;
        let guard = pool.core.lock().await;
        assert!(guard.is_none(), "over-TTL core must be evicted");
        drop(guard);
        assert_eq!(stopped.load(Ordering::Relaxed), 1);
    }

    /// Minimal fake SOCKS5 listener for the concurrency regression test.
    ///
    /// Answers the SOCKS5 greeting (`[0x05, 0x00]`) so `wait_for_socks5`
    /// passes, then holds open any connection that sends a CONNECT request
    /// (the real-ping connection — the readiness probe closes right after the
    /// greeting reply) without answering, until [`HoldingSocks::release_all`].
    /// This gives the test a deterministic window during which the pooled
    /// core is mid-`real_ping`.
    struct HoldingSocks {
        /// Signaled once a real-ping connection is being held open.
        mid_ping: Arc<Notify>,
        release: watch::Sender<bool>,
    }

    impl HoldingSocks {
        /// Bind the fake server on `127.0.0.1:10801` — the port the pooled
        /// core from [`pool_with_core`] uses.
        async fn start() -> Self {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", 10801))
                .await
                .expect("bind fake socks5 server on pool core port");
            let (release, release_rx) = watch::channel(false);
            let mid_ping = Arc::new(Notify::new());
            let mid_ping_task = mid_ping.clone();
            tokio::spawn(async move {
                loop {
                    let Ok((mut stream, _)) = listener.accept().await else {
                        break;
                    };
                    let mut release_rx = release_rx.clone();
                    let mid_ping = mid_ping_task.clone();
                    tokio::spawn(async move {
                        let mut greeting = [0u8; 3];
                        if stream.read_exact(&mut greeting).await.is_err() {
                            return;
                        }
                        if stream.write_all(&[0x05, 0x00]).await.is_err() {
                            return;
                        }
                        // Read the CONNECT header to tell the real-ping
                        // connection apart from the readiness probe, which is
                        // dropped right after the greeting reply.
                        let mut head = [0u8; 4];
                        if stream.read_exact(&mut head).await.is_err() {
                            return;
                        }
                        mid_ping.notify_one();
                        // Hold without answering CONNECT until released.
                        while !*release_rx.borrow() {
                            if release_rx.changed().await.is_err() {
                                return;
                            }
                        }
                    });
                }
            });
            Self { mid_ping, release }
        }

        /// Wait until the first real-ping connection is held open — the first
        /// ping is in flight with the pool lock held.
        async fn await_mid_ping(&self) {
            self.mid_ping.notified().await;
        }

        /// Close every held connection, failing the in-flight pings.
        fn release_all(&self) {
            let _ = self.release.send(true);
        }
    }

    fn vmess_endpoint_and_protocol() -> (Endpoint, ProtocolRow) {
        let endpoint = Endpoint {
            id: 0,
            host: "example.com".to_string(),
            host_type: "dns".to_string(),
            port: 443,
            port_spec_str: None,
            parent_id: None,
            last_source: None,
            created_at: 0,
            manual_protocol_override: None,
            resolved_as: None,
            resolved_at: None,
        };
        let extra = serde_json::json!({
            "remarks": "pool concurrency test",
            "user_id": "test-uuid",
        });
        let protocol = ProtocolRow {
            id: 0,
            endpoint_id: 0,
            sig: 0,
            cred_hash: 0,
            proto_kind: String::new(),
            last_used_at: None,
            spec_blob: serde_json::to_vec(&extra).unwrap_or_default(),
            config_type: Protocol::Vmess.to_i32(),
            core_type: String::new(),
            transport: Some("tcp".to_string()),
            security: Some("auto".to_string()),
            created_at: 0,
            last_seen_at: 0,
            extension: Default::default(),
            endpoint: Default::default(),
            server_stat: Default::default(),
        };
        (endpoint, protocol)
    }

    fn spawn_ping(
        pool: Arc<CorePool>,
        endpoint: Arc<Endpoint>,
        protocol: Arc<ProtocolRow>,
    ) -> tokio::task::JoinHandle<super::super::super::PingResult> {
        tokio::spawn(async move {
            // IP-literal URLs: reqwest's `socks5://` proxy resolves hostnames
            // client-side, and the test must not depend on DNS.
            pool.ping(
                &endpoint,
                &protocol,
                "http://127.0.0.1/",
                "http://127.0.0.1/ip",
                Duration::from_secs(10),
                1,
            )
            .await
        })
    }

    #[tokio::test]
    async fn concurrent_pings_do_not_reload_core_mid_ping() {
        // Regression (M7): with the pool lock held across the HTTP ping, a
        // concurrent single ping must not be able to reload (stop/restart or
        // SIGHUP) the same pooled core while the first ping's requests are in
        // flight. The fake socks server holds the first ping mid-`real_ping`;
        // if the lock were dropped early, the second ping would reload the
        // core (a second stop) while the first was still in flight.
        let server = HoldingSocks::start().await;
        let (pool, stopped) = pool_with_core(Instant::now());
        let pool = Arc::new(pool);
        let (endpoint, protocol) = vmess_endpoint_and_protocol();
        let endpoint = Arc::new(endpoint);
        let protocol = Arc::new(protocol);

        let first = spawn_ping(pool.clone(), endpoint.clone(), protocol.clone());
        tokio::time::timeout(Duration::from_secs(5), server.await_mid_ping())
            .await
            .expect("first ping never reached real_ping");

        let second = spawn_ping(pool.clone(), endpoint.clone(), protocol.clone());
        // Give the second ping time to reach the pool lock. With the fix it
        // blocks there; without it, it would reload the core immediately.
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(
            stopped.load(Ordering::Relaxed),
            1,
            "second ping must not reload the core while the first is in flight"
        );

        // Fail both in-flight pings, then let the second one run its reload.
        server.release_all();
        let (first_result, second_result) = tokio::join!(first, second);
        first_result.expect("first ping task panicked");
        second_result.expect("second ping task panicked");

        // Both pings eventually reloaded the core: exactly 2 stops total.
        assert_eq!(stopped.load(Ordering::Relaxed), 2);
    }
}
