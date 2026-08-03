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

        {
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

                drop(guard); // release lock while HTTP ping runs
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
        }
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
