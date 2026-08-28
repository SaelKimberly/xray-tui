// Shared test fixtures for the routing-engine integration tests.
#![allow(dead_code)]

use parking_lot::Mutex;
use std::{future::Future, pin::Pin};

use xray_tui_route::{error::RouteError, resolve::DnsSink};

/// Queue-backed fake [`DnsSink`]: pops one scripted result per lookup,
/// yielding a fixed "exhausted" error once the queue is drained.
pub struct SeqSink {
    pub results: Mutex<Vec<Result<Vec<std::net::IpAddr>, RouteError>>>,
}

impl DnsSink for SeqSink {
    fn lookup_ip(
        &self,
        _host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<std::net::IpAddr>, RouteError>> + Send>> {
        let mut q = self.results.lock();
        let r = if q.is_empty() {
            Err(RouteError::Resolve("exhausted".into()))
        } else {
            q.remove(0)
        };
        Box::pin(async move { r })
    }
}

