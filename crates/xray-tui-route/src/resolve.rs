//! Resolver seam: DNS sink trait, TTL result cache, probe streak tracker.
//!
//! Pure bookkeeping only — no network I/O here. The caller (engine) invokes
//! the [`DnsSink`] and feeds outcomes to [`ProbeTracker`]/[`ResolvedCache`].

use std::{collections::HashMap, future::Future, net::IpAddr, pin::Pin};

use crate::{error::RouteError, events::RouteEvent};

/// Async DNS resolution seam, consumed by the routing engine.
///
/// Object-safe: the engine stores `Arc<dyn DnsSink>`.
pub trait DnsSink: Send + Sync {
    /// Resolves `host`, returning all addresses or a [`RouteError::Resolve`].
    fn lookup_ip(
        &self,
        host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<IpAddr>, RouteError>> + Send>>;
}

/// TTL-keyed DNS resolution cache.
///
/// Expiry is by [`jiff::Timestamp`] comparison: an entry is fresh while
/// `now < stored + ttl`, stale from `now >= stored + ttl` (inclusive).
#[derive(Debug, Default)]
pub struct ResolvedCache {
    entries: HashMap<String, (Vec<IpAddr>, jiff::Timestamp)>,
    ttl_secs: i64,
}

impl ResolvedCache {
    /// Creates a cache whose entries live `ttl_secs` past their store time.
    #[must_use]
    pub fn new(ttl_secs: i64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl_secs,
        }
    }

    /// Returns the cached addresses for `host` when `now` is before expiry.
    #[must_use]
    pub fn get_fresh(&self, host: &str, now: jiff::Timestamp) -> Option<&[IpAddr]> {
        let (ips, stored) = self.entries.get(host)?;
        if now >= *stored + jiff::Span::new().seconds(self.ttl_secs) {
            None
        } else {
            Some(ips)
        }
    }

    /// Stores `ips` for `host`, stamped at `now`.
    pub fn put(&mut self, host: String, ips: Vec<IpAddr>, now: jiff::Timestamp) {
        self.entries.insert(host, (ips, now));
    }
}

/// Consecutive-failure streak tracker for probe targets.
///
/// Zero-cost no-op while the probes list is empty. Streaks are keyed per
/// probe; entering failure emits [`RouteEvent::NetworkBreakdown`] exactly
/// once per streak, and the next success emits [`RouteEvent::ProbeRecovered`]
/// exactly once and resets the streak.
#[derive(Debug, Default)]
pub struct ProbeTracker {
    streaks: HashMap<String, u32>,
}

impl ProbeTracker {
    /// Applies one probe cycle.
    ///
    /// `sink_probe_result` is `Some((failed, Some(probe)))` for a per-probe
    /// outcome; probes not in `probes` are ignored. `any_failed_this_cycle`
    /// is informational only — the per-probe streak model is authoritative.
    /// Events go to `tx` when `Some`.
    pub fn update(
        &mut self,
        probes: &[String],
        _any_failed_this_cycle: bool,
        sink_probe_result: Option<(bool, Option<&str>)>,
        tx: &Option<tokio::sync::mpsc::UnboundedSender<RouteEvent>>,
    ) {
        let Some((failed, which)) = sink_probe_result else {
            return;
        };
        let Some(probe) = which else { return };
        if !probes.iter().any(|p| p == probe) {
            return;
        }
        if failed {
            let entry = self.streaks.entry(probe.to_owned()).or_insert(0);
            *entry += 1;
            if *entry == 1 {
                emit(
                    tx.as_ref(),
                    RouteEvent::NetworkBreakdown {
                        failed_probe: probe.to_owned(),
                        at: jiff::Timestamp::now(),
                    },
                );
            }
        } else if self.streaks.remove(probe).is_some() {
            emit(
                tx.as_ref(),
                RouteEvent::ProbeRecovered {
                    probe: probe.to_owned(),
                    at: jiff::Timestamp::now(),
                },
            );
        }
    }
}

/// Sends `event` when the channel exists; drops it otherwise.
fn emit(tx: Option<&tokio::sync::mpsc::UnboundedSender<RouteEvent>>, event: RouteEvent) {
    if let Some(tx) = tx {
        let _ = tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use tokio::sync::mpsc;

    fn probes(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[tokio::test]
    async fn cache_ttl_expiry_forces_refetch() {
        let mut cache = ResolvedCache::new(60);
        let t0 = jiff::Timestamp::now();
        let ip: IpAddr = "1.2.3.4".parse().unwrap();

        cache.put("h".into(), vec![ip], t0);
        assert_eq!(cache.get_fresh("h", t0), Some(&[ip][..]));
        // Boundary: now == stored+ttl is stale (inclusive).
        let expired = t0 + Duration::from_secs(60);
        assert_eq!(cache.get_fresh("h", expired), None);
        // Just before expiry is fresh.
        assert_eq!(
            cache.get_fresh("h", expired - Duration::from_nanos(1)),
            Some(&[ip][..])
        );
        // After a refetch (put again), fresh again.
        cache.put("h".into(), vec![ip], expired);
        assert_eq!(cache.get_fresh("h", expired), Some(&[ip][..]));
    }

    #[tokio::test]
    async fn probe_streak_fail_then_recover_emits_exactly_once_each() {
        let mut tracker = ProbeTracker::default();
        let list = probes(&["p1", "p2"]);
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Two consecutive failures: one Breakdown, streak continues.
        tracker.update(&list, true, Some((true, Some("p1"))), &Some(tx.clone()));
        tracker.update(&list, true, Some((true, Some("p1"))), &Some(tx.clone()));
        tracker.update(&list, true, Some((true, Some("p2"))), &Some(tx.clone()));
        // Successes: one Recovered each, streaks reset.
        tracker.update(&list, false, Some((false, Some("p1"))), &Some(tx.clone()));
        tracker.update(&list, false, Some((false, Some("p2"))), &Some(tx.clone()));

        let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert!(
            matches!(events[0], RouteEvent::NetworkBreakdown { ref failed_probe, .. } if failed_probe == "p1")
        );
        assert!(
            matches!(events[1], RouteEvent::NetworkBreakdown { ref failed_probe, .. } if failed_probe == "p2")
        );
        assert!(matches!(events[2], RouteEvent::ProbeRecovered { ref probe, .. } if probe == "p1"));
        assert!(matches!(events[3], RouteEvent::ProbeRecovered { ref probe, .. } if probe == "p2"));
        assert_eq!(events.len(), 4);
        // Steady state: further successes emit nothing.
        tracker.update(&list, false, Some((false, Some("p1"))), &Some(tx));
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn streak_reset_on_success_between_failures() {
        let mut tracker = ProbeTracker::default();
        let list = probes(&["p1"]);
        let (tx, mut rx) = mpsc::unbounded_channel();

        tracker.update(&list, true, Some((true, Some("p1"))), &Some(tx.clone()));
        tracker.update(&list, false, Some((false, Some("p1"))), &Some(tx.clone()));
        tracker.update(&list, true, Some((true, Some("p1"))), &Some(tx));

        let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert!(matches!(events[0], RouteEvent::NetworkBreakdown { .. }));
        assert!(matches!(events[1], RouteEvent::ProbeRecovered { .. }));
        assert!(matches!(events[2], RouteEvent::NetworkBreakdown { .. }));
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn probe_list_empty_means_zero_events() {
        let mut tracker = ProbeTracker::default();
        let (tx, mut rx) = mpsc::unbounded_channel();

        tracker.update(&[], true, Some((true, Some("p1"))), &Some(tx.clone()));
        tracker.update(&[], false, Some((false, Some("p1"))), &Some(tx));
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn probe_not_in_list_ignored() {
        let mut tracker = ProbeTracker::default();
        let list = probes(&["p1"]);
        let (tx, mut rx) = mpsc::unbounded_channel();

        tracker.update(&list, true, Some((true, Some("other"))), &Some(tx.clone()));
        tracker.update(&list, false, Some((false, Some("other"))), &Some(tx));
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn tx_none_transitions_without_panic() {
        let mut tracker = ProbeTracker::default();
        let list = probes(&["p1"]);

        tracker.update(&list, true, Some((true, Some("p1"))), &None);
        // Second failure: no duplicate breakdown was (or can be) observed, state OK.
        tracker.update(&list, true, Some((true, Some("p1"))), &None);
        tracker.update(&list, false, Some((false, Some("p1"))), &None);
        tracker.update(&list, false, Some((false, Some("p1"))), &None);
    }

    #[tokio::test]
    async fn sink_no_result_no_state_change() {
        let mut tracker = ProbeTracker::default();
        let list = probes(&["p1"]);
        let (tx, mut rx) = mpsc::unbounded_channel();

        tracker.update(&list, true, None, &Some(tx));
        assert!(rx.try_recv().is_err());
    }
}

/// IP-literal passthrough needs no network; runtime resolves locally.
#[cfg(feature = "dns")]
#[tokio::test]
async fn dns_adapter_ip_literal_passthrough() {
    let adapter = crate::dns_adapter::DnsSinkAdapter {
        resolver: std::sync::Arc::new(xray_tui_dns::DnsResolver::new("/tmp")),
    };
    let ips = adapter.lookup_ip("127.0.0.1".into()).await.unwrap();
    assert_eq!(ips, vec!["127.0.0.1".parse::<IpAddr>().unwrap()]);
}
