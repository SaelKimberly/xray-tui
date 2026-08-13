//! E2E pipeline: real-core server scenarios against the native client.
//!
//! Consumers implement [`E2eCase`] to describe one scenario (label, server
//! config, client params, expectations); a driver (Task 3) runs them against
//! resolved core binaries. Gated behind feature `native-e2e`.

pub mod case;
pub mod config;
pub mod core;
pub mod harness;
pub mod variant;

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

pub use case::{CaseSpec, Flow, ProtocolKind};
pub use core::{CoreKind, CoreUnderTest};
pub use harness::{
    Certs, EchoServer, TlsEchoServer, free_port, generate_certs, probe, spawn_core, spawn_echo,
    spawn_tls_echo,
};
pub use variant::{
    Aes128GcmVariant, Chacha20Poly1305Variant, FingerprintTls, PlainServerRealityClientTls,
    RealityServerPlainClientTls, RealityTls, RealityWrongPbkTls, RealityWrongSidTls,
    SecurityVariant, StandardTls, TlsVariant,
};

use crate::{NativeConnectParams, NativeError};

/// Core version strings for the e2e sweep (single source of truth; tests
/// import these instead of duplicating constants).
pub const SINGBOX_VERSION: &str = "1.13.16";
pub const XRAY_VERSION: &str = "26.3.27";

/// Environment handed to an [`E2eCase`]'s `server_config` builder.
pub struct ServerEnv<'a> {
    pub port: u16,
    pub certs: &'a Certs,
    pub tmp: &'a Path,
    /// The local echo HTTP server (the probe's destination through the
    /// tunnel).
    pub echo: SocketAddr,
    /// The local TLS HTTP server: REALITY inbounds point their
    /// `dest`/`handshake` fallback here (xtls/reality borrows the dest's
    /// TLS `ServerHello` flight, so the fallback must be a real TLS server).
    pub tls_echo: SocketAddr,
}

/// Expected outcome of the initial `connect()` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectExpect {
    /// The tunnel establishes and the probe runs (status/body asserted).
    Ok,
    /// The REALITY client's auth is expected to fail against the server
    /// (wrong pbk/sid, or a non-REALITY server) — `connect()` must return
    /// `Err`, and the client's Spider-X session must reach the server's
    /// dest (its `h2` preface shows up in the `tls_echo` recording). The
    /// probe is skipped.
    ErrRealityFallback,
}

/// Expected probe outcome for an [`E2eCase`].
pub struct E2eExpect {
    /// Whether `connect()` succeeds or falls back.
    pub connect: ConnectExpect,
    pub status: u16,
    pub body: String,
}

impl Default for E2eExpect {
    fn default() -> Self {
        Self {
            connect: ConnectExpect::Ok,
            status: 200,
            body: crate::e2e::config::BODY.to_string(),
        }
    }
}

/// One end-to-end scenario: a core's server config, the native client params
/// that dial it, and the expected probe result.
pub trait E2eCase {
    /// Human-readable scenario name (test label).
    fn label(&self) -> String;
    /// JSON config to write for `core` and spawn on `env.port`.
    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String;
    /// Native client params dialing the core listener on `port` toward `target`.
    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams;
    /// Expected probe (status, body) once the tunnel is live.
    fn expected(&self) -> E2eExpect;
    /// The tunnel's destination for this case: the plain echo by default,
    /// the TLS echo for cases whose probe must flow through a transparent
    /// proxy to the dest (a plain client through a REALITY server).
    fn probe_target(&self, env: &ServerEnv) -> SocketAddr {
        env.echo
    }
    /// True when a REALITY fallback's Spider-X session reaches the server's
    /// dest (the `tls_echo`) rather than terminating at the server itself.
    fn spider_reaches_dest(&self) -> bool {
        true
    }
    /// Optional hook to install harness-CA trust before dialing.
    fn client_trust(&self, _certs: &Certs) {}
}

const ATTEMPTS: u32 = 3;

/// Run the fixed 7-step e2e lifecycle for `case` against `core`.
///
/// The flaky segment (core spawn + connect + probe) is retried with fresh
/// resources; `echo`/`certs`/`tls_echo`/`tempdir` are created once by the caller
/// (rstest fixtures) and shared across the retries.
pub async fn run_against(
    case: &CaseSpec,
    core: &CoreUnderTest,
    certs: &Certs,
    echo: &EchoServer,
    tls_echo: &TlsEchoServer,
) -> Result<(), String> {
    let expect = case.expected();
    let dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).map_err(|e| format!("cert write: {e}"))?;
    std::fs::write(&key_path, &certs.key_pem).map_err(|e| format!("key write: {e}"))?;
    let config_path = dir.path().join("config.json");

    for attempt in 1..=ATTEMPTS {
        // Fresh recording per attempt: the tls_echo is created once before
        // the loop, so without a reset its buffer accumulates across
        // attempts — a preface that arrived just after attempt N's poll
        // expiry could satisfy attempt N+1 spuriously.
        tls_echo.reset_recording();
        let port = free_port();
        let env = ServerEnv {
            port,
            certs,
            tmp: dir.path(),
            echo: echo.addr,
            tls_echo: tls_echo.addr,
        };
        let config_json = case.server_config(core.kind, &env);
        if std::fs::write(&config_path, &config_json).is_err() {
            return Err(format!("attempt {attempt}: config write failed"));
        }
        let _core = spawn_core(&core.bin, core.kind, &config_path, port);

        case.client_trust(certs);
        let params = case.client_params(port, case.probe_target(&env));
        if matches!(expect.connect, ConnectExpect::ErrRealityFallback) {
            // REALITY fallback scenario: the client's auth must fail. Assert
            // BOTH halves — the connect errors with a REALITY error
            // (RealityFallback), and the client's Spider-X session actually
            // reached the server's dest (its h2 preface shows up in the
            // tls_echo recording) — then skip the probe (there is no tunnel
            // to probe). Accepting *any* connect error here would let an
            // unrelated failure (server not up, timeout) pass the case.
            match crate::connect(params).await {
                Ok(_) => {
                    return Err(format!(
                        "{}: expected RealityFallback, got a connection",
                        case.label()
                    ));
                }
                Err(NativeError::Reality(_)) => {}
                Err(other) => {
                    return Err(format!(
                        "{}: expected RealityFallback (NativeError::Reality), got {other:?}",
                        case.label()
                    ));
                }
            }
            if !case.spider_reaches_dest() {
                // The fallback session terminates at the server itself
                // (plain TLS server) — the spider's bytes never reach the
                // tls_echo. The connect error alone proves the fallback.
                return Ok(());
            }
            // The spider sends its h2 preface right after the fallback
            // session is established; allow 5s for it to traverse the
            // transparent proxy to the dest.
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut saw = false;
            while std::time::Instant::now() < deadline {
                if tls_echo.saw_h2_preface() {
                    saw = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            if saw {
                return Ok(());
            }
            eprintln!(
                "[e2e] {}: attempt {attempt}/{ATTEMPTS} fallback: no h2 preface on the dest within 5s",
                case.label()
            );
            continue;
        }
        let mut tunnel = match crate::connect(params).await {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[e2e] {}: attempt {attempt}/{ATTEMPTS} connect: {e}",
                    case.label()
                );
                continue;
            }
        };
        let (status, body) = probe(&mut tunnel).await;
        if status == expect.status && body == expect.body {
            return Ok(());
        }
        eprintln!(
            "[e2e] {}: attempt {attempt}/{ATTEMPTS} probe status {status} body {body:?}",
            case.label()
        );
    }
    Err(format!(
        "{}: failed after {ATTEMPTS} attempts",
        case.label()
    ))
}
