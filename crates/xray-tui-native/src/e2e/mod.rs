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
pub use harness::{Certs, EchoServer, probe, spawn_core, spawn_echo, generate_certs, free_port};

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
    fn client_trust(&self, certs: &Certs) {
        drop(certs);
    }
}
