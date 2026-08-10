//! E2E pipeline: real-core server scenarios against the native client.
//!
//! Consumers implement [`E2eCase`] to describe one scenario (label, server
//! config, client params, expectations); a driver (Task 3) runs them against
//! resolved core binaries. Gated behind feature `native-e2e`.

pub mod core;
pub mod harness;

use std::net::SocketAddr;
use std::path::Path;

pub use core::{CoreKind, CoreUnderTest};
pub use harness::{Certs, EchoServer, free_port, generate_certs, probe, spawn_core, spawn_echo};

use crate::NativeConnectParams;

/// Environment handed to an [`E2eCase`]'s `server_config` builder.
pub struct ServerEnv<'a> {
    pub port: u16,
    pub certs: &'a Certs,
    pub tmp: &'a Path,
}

/// Expected probe outcome for an [`E2eCase`].
pub struct E2eExpect {
    pub status: u16,
    pub body: String,
}

/// One end-to-end scenario: a core's server config, the native client params
/// that dial it, and the expected probe result.
pub trait E2eCase {
    /// Human-readable scenario name (test label).
    fn label(&self) -> &'static str;
    /// JSON config to write for `core` and spawn on `env.port`.
    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String;
    /// Native client params dialing the core listener on `port` toward `target`.
    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams;
    /// Expected probe (status, body) once the tunnel is live.
    fn expected(&self) -> E2eExpect;
    /// Optional hook to install harness-CA trust before dialing.
    fn client_trust(&self, _certs: &Certs) {}
}

const ATTEMPTS: u32 = 3;

/// Run the fixed 7-step e2e lifecycle for `case` against `core`.
///
/// The flaky segment (core spawn + connect + probe) is retried with fresh
/// resources; echo/certs/tempdir are created once.
pub async fn run<C: E2eCase + Sync>(case: &C, core: &CoreUnderTest) -> Result<(), String> {
    let expect = case.expected();
    let echo = spawn_echo();
    let certs = generate_certs();
    let dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).map_err(|e| format!("cert write: {e}"))?;
    std::fs::write(&key_path, &certs.key_pem).map_err(|e| format!("key write: {e}"))?;
    let config_path = dir.path().join("config.json");

    for attempt in 1..=ATTEMPTS {
        let port = free_port();
        let env = ServerEnv {
            port,
            certs: &certs,
            tmp: dir.path(),
        };
        let config_json = case.server_config(core.kind, &env);
        if std::fs::write(&config_path, &config_json).is_err() {
            return Err(format!("attempt {attempt}: config write failed"));
        }
        let _core = spawn_core(&core.bin, core.kind, &config_path, port);

        case.client_trust(&certs);
        let params = case.client_params(port, echo.addr);
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
