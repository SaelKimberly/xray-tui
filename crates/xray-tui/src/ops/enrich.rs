//! Background enrichment of endpoint display data.
//!
//! All network- and IO-bound work (DNS resolution, `GeoIP` mmdb lookups,
//! whitelist checks) runs in spawned tokio tasks that report back through
//! `CoreEvent::EndpointInfoUpdated`. The UI thread never blocks on them.
//! Every failure degrades to defaults (no flag / `🏴`) with a `tracing::warn!`.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;
use xray_tui_db::models::{Endpoint, EndpointId, HostType, Protocol};
use xray_tui_host_features::HostFeatures;

use crate::AppState;
use crate::ops::profiles::PROFILES_PAGE_SIZE;
use crate::types::{CoreEvent, EndpointInfo};

/// SNI from a typed protocol row: the `security.sni` column, populated at
/// write time from `config.security().sni()` (covers both `tls` and `reality`
/// variants). The column is queryable without loading the deferred `config`
/// JSON; when the config IS loaded, the typed accessor chain is equivalent.
pub(crate) fn extract_sni(protocol: &Protocol) -> Option<String> {
    use xray_tui_proto::proto_spec::ProtoSpec;
    if !protocol.config.is_unloaded()
        && let Some(sni) = protocol.config.get().0.security().and_then(|s| s.sni())
    {
        return Some(sni.to_string());
    }
    protocol.security.sni.clone()
}

/// True when a resolution must run: no entry, no address and no attempt, or a
/// DNS entry older than the TTL, or `force`.
///
/// An entry with an empty address set AND no attempt timestamp is not an
/// IP-host entry and carries no resolution information: `spawn_outbound_enrich`
/// materializes exactly that shape (it knows the endpoint's exit IP, not its
/// inbound address). Treating it as "already resolved" is what made `[name]`
/// terminal for a DNS host whose first real ping landed before its first
/// resolution attempt.
const fn should_resolve(
    entry: Option<&EndpointInfo>,
    force: bool,
    ttl_secs: i64,
    now_secs: i64,
) -> bool {
    let Some(e) = entry else {
        return true;
    };
    // No address and no attempt is not a resolution — see the note above.
    if e.resolved_ips.is_empty() && e.resolved_at_secs.is_none() {
        return true;
    }
    // An IP host carries its own address and no attempt stamp: its address IS
    // the resolution, so it never re-resolves.
    let Some(ts) = e.resolved_at_secs else {
        return false;
    };
    force || now_secs - ts >= ttl_secs
}

/// A DNS hostname safe to hand to the resolver: ASCII letters/digits/hyphens
/// in dot-separated labels. Rejects plugin URLs (`host:port?plugin=...`),
/// underscores (Telegram-channel names), spaces and non-ASCII — hickory
/// errors on those ("Label contains invalid characters") and the failure is
/// pure log noise.
fn is_resolvable_hostname(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    if host.is_empty() || host.len() > 253 || host.starts_with('.') || host.contains("..") {
        return false;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}

/// `"{ip} | {country}"` (real-ping `ip_info` format) → `(ip, country-hint)`.
fn parse_ip_info(ip_info: &str) -> Option<(IpAddr, Option<String>)> {
    let (ip_part, country) = ip_info
        .split_once('|')
        .map_or((ip_info, None), |(a, b)| (a, Some(b.trim().to_string())));
    let ip = ip_part.trim().parse::<IpAddr>().ok()?;
    Some((ip, country))
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Fill `country` (mmdb) and `host_features`/`sni_whitelisted` (whitelist
/// checker) for an entry. Selected IP = first IPv4, else first entry.
///
/// Returns the `(address, ISO)` the mmdb just produced, `None` when the
/// country was already known or no database is loaded. The caller persists
/// it — this function never writes, so the two call sites (the DNS resolve
/// and the page seed) own their own durability.
async fn fill_features(
    info: &mut EndpointInfo,
    geo: Option<&Arc<xray_tui_geoip::GeoIp>>,
    checker: Option<&Arc<xray_tui_host_features::HostFeaturesChecker>>,
    sni: Option<&str>,
) -> Option<(IpAddr, String)> {
    let selected = info
        .resolved_ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .or_else(|| info.resolved_ips.first())
        .copied();
    let mut fresh = None;
    if let Some(ip) = selected {
        // A country that came from `endpoint_ip` (or from this session's
        // earlier lookup) needs no mmdb walk — the store is the cache.
        if info.country.is_none()
            && let Some(geo) = geo
        {
            match geo.location_by_ip(ip).await {
                Ok(Some(loc)) => {
                    let iso = loc.country.to_string();
                    info.country = Some(iso.clone());
                    fresh = Some((ip, iso));
                }
                Ok(None) => {}
                Err(e) => tracing::warn!(target: "tui::ops::enrich", "geo lookup failed: {e}"),
            }
        }
        if let Some(checker) = checker {
            info.host_features = checker.ip_features(ip);
        }
    }
    if let Some(checker) = checker
        && let Some(sni_str) = sni
    {
        info.sni_whitelisted = checker.sni_features(sni_str);
    }
    fresh
}

/// The stored country a feature pass would use for an endpoint: the
/// selection rule [`fill_features`] applies (first IPv4, else the first
/// entry), with the country that address carries.
fn stored_country(addrs: &[(IpAddr, Option<String>)]) -> Option<String> {
    addrs
        .iter()
        .find(|(ip, _)| ip.is_ipv4())
        .or_else(|| addrs.first())
        .and_then(|(_, country)| country.clone())
}

/// Resolve (or re-resolve) one endpoint's inbound host in the background.
///
/// TTL gate: fresh DNS entries are skipped (no network) unless `force`.
/// Results arrive via `CoreEvent::EndpointInfoUpdated`; DNS-host results are
/// persisted by the event handler so they survive launches.
///
/// Page-scoped: the row must be in the loaded page. A batch plans the whole
/// feed, so it sends [`CoreEvent::DnsResolveRequest`] and the handler calls
/// [`spawn_dns_resolve_host`] with the facts instead.
pub fn spawn_dns_resolve(state: &mut AppState, endpoint_id: i64, force: bool) {
    let Some(row) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    else {
        return;
    };
    let host = row.endpoint.host.clone();
    let host_type = row.endpoint.host_type;
    let sni = row.active_protocol().and_then(|(_, p)| extract_sni(p));
    spawn_dns_resolve_host(state, endpoint_id, host, host_type, sni, force);
}

/// Bound on concurrent inbound resolutions.
///
/// A feed-wide batch asks for one resolution per DNS endpoint — ~18k on the
/// reference feed — and the trigger it replaces was bounded by
/// `real_ping_concurrency`. Every in-flight lookup holds a permit and one shared
/// hickory resolver's sockets, so without this the whole fan-out opens at once.
/// Acquired INSIDE the spawned task: taking a permit in `poll_core_events` would
/// stall the UI tick on DNS.
static RESOLVE_SEM: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(64)));

/// Resolve one endpoint's inbound host from facts the caller already holds.
///
/// The TTL gate
/// ([`should_resolve`]) is evaluated here, against the session's
/// `endpoint_info`, so repeated requests for the same endpoint collapse to one
/// lookup and a fresh resolution is never repeated.
pub fn spawn_dns_resolve_host(
    state: &mut AppState,
    endpoint_id: i64,
    host: String,
    host_type: HostType,
    sni: Option<String>,
    force: bool,
) {
    if !should_resolve(
        state.endpoint_info.get(&endpoint_id),
        force,
        state.dns_cache_ttl_secs,
        unix_now(),
    ) {
        return;
    }

    let dns = state.dns_resolver.clone();
    let geo = state.geo_ip.clone();
    let checker = state.host_features.clone();
    let scheduler = state.scheduler.clone();
    let db = state.db.clone();
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        // One permit per in-flight lookup — see [`RESOLVE_SEM`]. A FORCED
        // lookup (the `x` key, the connect path) bypasses the bound: the
        // semaphore is fair, so a user-triggered resolve would otherwise queue
        // behind an entire feed's fan-out and look dead, and a single request
        // cannot flood anything.
        let _permit = if force {
            None
        } else {
            match Arc::clone(&RESOLVE_SEM).acquire_owned().await {
                Ok(permit) => Some(permit),
                Err(_) => return,
            }
        };
        let now = unix_now();
        // Whether the resolution produced a usable answer; `false` feeds the
        // scheduler's DNS-failure gate below. IP hosts and hosts without a
        // resolver configured count as fine.
        let mut resolved_ok = true;
        // DNS lookup or direct IP parse
        let (ips, resolved_at) = match host_type {
            HostType::Ipv4 | HostType::Ipv6 => (
                host.parse::<IpAddr>()
                    .map(|ip| vec![ip])
                    .unwrap_or_default(),
                None,
            ),
            HostType::Undefined => {
                resolved_ok = false;
                (Vec::new(), Some(now))
            }
            HostType::Dns => {
                if is_resolvable_hostname(&host) {
                    match &dns {
                        Some(r) => {
                            // Overall deadline: resolver init (DNSCrypt list
                            // download) plus lookups over many name servers
                            // can otherwise stall indefinitely.
                            match tokio::time::timeout(
                                Duration::from_secs(8),
                                r.lookup_ip(&host, false),
                            )
                            .await
                            {
                                Ok(Ok(ips)) => {
                                    tracing::info!(
                                        target: "tui::ops::enrich",
                                        "Resolved {host}: {} IP(s)",
                                        ips.len()
                                    );
                                    // `Box<[IpAddr]>` → `Vec`: reuses the
                                    // allocation, no copy.
                                    (Vec::from(ips), Some(now))
                                }
                                Ok(Err(e)) => {
                                    resolved_ok = false;
                                    if e.to_string().contains("no records found") {
                                        // Host without any DNS record — the
                                        // UI flag carries the signal; don't
                                        // warn per host per TTL.
                                        tracing::debug!(
                                            target: "tui::ops::enrich",
                                            "DNS lookup of {host} found no records"
                                        );
                                    } else {
                                        tracing::warn!(
                                            target: "tui::ops::enrich",
                                            "DNS lookup of {host} failed: {e}"
                                        );
                                    }
                                    (Vec::new(), Some(now))
                                }
                                Err(_) => {
                                    resolved_ok = false;
                                    tracing::warn!(
                                        target: "tui::ops::enrich",
                                        "DNS lookup of {host} timed out"
                                    );
                                    (Vec::new(), Some(now))
                                }
                            }
                        }
                        None => (Vec::new(), None),
                    }
                } else {
                    // Plugin URLs / garbage hostnames can never resolve;
                    // record a failed attempt (TTL-gated) instead of firing
                    // hickory parse errors on every refresh.
                    resolved_ok = false;
                    tracing::debug!(
                        target: "tui::ops::enrich",
                        "Skipping DNS lookup for invalid hostname: {host}"
                    );
                    (Vec::new(), Some(now))
                }
            }
        };

        // Feed the scheduler's DNS-failure gate: a failed resolution marks
        // the endpoint so the batch scheduler skips it for the deferral
        // window; a successful one clears the marker (resolvable again).
        let scheduler_endpoint = EndpointId::new(endpoint_id);
        if resolved_ok {
            scheduler.clear_dns_failure(scheduler_endpoint);
        } else {
            scheduler.mark_dns_failure(scheduler_endpoint);
        }

        let mut info = EndpointInfo {
            resolved_ips: ips,
            country: None,
            host_features: HostFeatures::default(),
            sni_whitelisted: None,
            outbound_ip: None,
            outbound_country: None,
            resolved_at_secs: resolved_at,
        };
        // Phase 1: deliver the resolution immediately — the UI must never
        // wait on geo/whitelist work (the mmdb can download on first use).
        if let Some(t) = tx.as_ref() {
            let _ = t.try_send(CoreEvent::EndpointInfoUpdated {
                endpoint_id,
                info: info.clone(),
            });
        }
        // Phase 2: country + whitelist features. Bounded by the geo crate's
        // own download deadline; a timeout here still degrades to `🏴`.
        // A country the mmdb just produced is written to `endpoint_ip` (the
        // table owns the address's flag; `replace` keeps it across
        // re-resolutions), so the next launch renders it without a lookup.
        if let Some((ip, iso)) =
            fill_features(&mut info, geo.as_ref(), checker.as_ref(), sni.as_deref()).await
            && let Err(e) = db
                .set_endpoint_ip_country(EndpointId::new(endpoint_id), ip, &iso)
                .await
        {
            tracing::warn!(target: "tui::ops::enrich", "country persist failed: {e}");
        }
        if let Some(t) = tx {
            let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
        }
    });
}

/// One enrichment target: endpoint id, the endpoint, its persisted
/// `endpoint_ip` address set, the persisted `resolved_at` (unix secs), and the
/// SNI of its active protocol (None for linkless endpoints).
type EnrichTarget = (i64, Endpoint, Vec<IpAddr>, Option<i64>, Option<String>);

/// Startup/refresh pass: seed `endpoint_info` for every endpoint that has no
///
/// entry yet — IP hosts (parse host, no DNS) and DNS hosts with a persisted
/// `endpoint_ip` address set (no network). Geo + whitelist
/// features are filled in the same task.
pub fn spawn_enrich_ip_hosts(state: &mut AppState) {
    let targets: Vec<EnrichTarget> = state
        .endpoints
        .iter()
        .filter(|r| {
            matches!(r.endpoint.host_type, HostType::Ipv4 | HostType::Ipv6)
                || !r.resolved_ips.is_empty()
        })
        // An entry with no address and no attempt timestamp carries no
        // resolution information (an outbound-only event materializes one), so
        // it must still be seeded rather than treated as already resolved.
        .filter(|r| {
            state
                .endpoint_info
                .get(&r.endpoint.id.get())
                .is_none_or(|e| e.resolved_ips.is_empty() && e.resolved_at_secs.is_none())
        })
        .map(|r| {
            (
                r.endpoint.id.get(),
                r.endpoint.clone(),
                r.resolved_ips.clone(),
                r.endpoint.resolved_at,
                r.active_protocol().and_then(|(_, p)| extract_sni(p)),
            )
        })
        .collect();
    if targets.is_empty() {
        return;
    }

    // Phase 1 runs SYNCHRONOUSLY here: insert the cached resolution directly
    // into `endpoint_info` instead of round-tripping an event. The old
    // phase-1 `EndpointInfoUpdated` events looked identical to real DNS
    // resolutions to the event handler, which re-persisted
    // `update_endpoint_resolution` + `upsert_resolved_ip_children` for EVERY
    // already-resolved endpoint on every reload (7000 redundant write pairs
    // after a 7000-URL import).
    let mut feature_targets: Vec<(i64, EndpointInfo, Option<String>)> =
        Vec::with_capacity(targets.len());
    for (endpoint_id, ep, cached_as, cached_at, sni) in targets {
        let mut info = if cached_as.is_empty() {
            // IP host — its own address is the "resolution".
            EndpointInfo {
                resolved_ips: vec![
                    ep.host
                        .parse()
                        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
                ],
                country: None,
                host_features: HostFeatures::default(),
                sni_whitelisted: None,
                outbound_ip: None,
                outbound_country: None,
                resolved_at_secs: None,
            }
        } else {
            // DNS host with a persisted resolution — reuse it, no network and
            // no text parse: the stored form is the address itself.
            EndpointInfo {
                resolved_ips: cached_as,
                country: None,
                host_features: HostFeatures::default(),
                sni_whitelisted: None,
                outbound_ip: None,
                outbound_country: None,
                resolved_at_secs: cached_at,
            }
        };
        // An entry the seed is allowed to replace may still carry the exit-IP
        // fields an outbound-only event wrote (that event knows the endpoint's
        // outbound IP, never its inbound address, so it cannot fill
        // `resolved_ips`). Those fields are not the seed's to drop.
        if let Some(existing) = state.endpoint_info.get(&endpoint_id) {
            info.outbound_ip = existing.outbound_ip;
            info.outbound_country.clone_from(&existing.outbound_country);
        }
        state.endpoint_info.insert(endpoint_id, info.clone());
        feature_targets.push((endpoint_id, info, sni));
    }
    state.mark_rows_dirty(crate::ui::profiles::ROWS_DIRTY_COUNTRY);

    // Phase 2 stays in the background: mmdb may download on first use and
    // must never stall the UI task. Feature fields only — the handler's
    // persist branch stays silent because the seeded `resolved_at_secs` now
    // equals the incoming value.
    let geo = state.geo_ip.clone();
    let checker = state.host_features.clone();
    let db = state.db.clone();
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        // Countries land one transaction per page, not one commit per address:
        // `set_endpoint_ip_country` paid a full commit each (~4.2 ms,
        // `docs/database.md`) and those commits contended with the import's
        // 500-URL chunk transactions (`database is locked`). A failed batch
        // logs once and the pass carries on — the next page still writes.
        async fn flush_countries(
            db: &xray_tui_db::Database,
            pending: &mut Vec<(EndpointId, IpAddr, String)>,
            flushes: &mut usize,
        ) {
            let rows = std::mem::take(pending);
            if rows.is_empty() {
                return; // the tail call is unconditional; an empty one writes nothing
            }
            *flushes += 1;
            if let Err(e) = db.set_endpoint_ip_countries(&rows).await {
                tracing::warn!(
                    target: "tui::ops::enrich",
                    "country persist failed for {} rows: {e}",
                    rows.len()
                );
            }
        }

        // The countries already stored for this page's addresses: one read,
        // and every address that has one skips the mmdb entirely (a second
        // launch needs no `geo_ip` at all). Read here rather than in the
        // synchronous seed above so the seed stays callable from a
        // non-async caller.
        let ids: Vec<EndpointId> = feature_targets
            .iter()
            .map(|(id, ..)| EndpointId::new(*id))
            .collect();
        let stored = match db.endpoint_resolutions(&ids).await {
            Ok(map) => map,
            Err(e) => {
                tracing::warn!(target: "tui::ops::enrich", "stored countries read failed: {e}");
                std::collections::HashMap::new()
            }
        };
        let mut pending: Vec<(EndpointId, IpAddr, String)> = Vec::with_capacity(PROFILES_PAGE_SIZE);
        let mut scanned = 0usize;
        let mut countries = 0usize;
        let mut flushes = 0usize;
        for (endpoint_id, mut info, sni) in feature_targets {
            scanned += 1;
            if info.country.is_none() {
                info.country = stored
                    .get(&EndpointId::new(endpoint_id))
                    .and_then(|addrs| stored_country(addrs));
            }
            let fresh =
                fill_features(&mut info, geo.as_ref(), checker.as_ref(), sni.as_deref()).await;
            if let Some((ip, iso)) = fresh {
                countries += 1;
                pending.push((EndpointId::new(endpoint_id), ip, iso));
            }
            if let Some(t) = tx.as_ref() {
                let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
            }
            if pending.len() >= PROFILES_PAGE_SIZE {
                flush_countries(&db, &mut pending, &mut flushes).await;
            }
        }
        // The tail: the last partial page still has to land.
        flush_countries(&db, &mut pending, &mut flushes).await;
        tracing::debug!(
            target: "tui::ops::enrich",
            "geo seed: scanned={scanned} countries={countries} flushes={flushes}"
        );
    });
}

/// Refresh whitelist features (ip/cidr + SNI) for every endpoint once the
///
/// checker has loaded. Runs on every launch — features are never persisted, so
/// cached entries get fresh membership. Sends a full copy of each entry.
pub fn spawn_whitelist_pass(state: &mut AppState) {
    let Some(checker) = state.host_features.clone() else {
        return;
    };
    let targets: Vec<(i64, Option<String>, EndpointInfo)> = state
        .endpoints
        .iter()
        .map(|r| {
            (
                r.endpoint.id.get(),
                r.active_protocol().and_then(|(_, p)| extract_sni(p)),
                state
                    .endpoint_info
                    .get(&r.endpoint.id.get())
                    .cloned()
                    .unwrap_or_default(),
            )
        })
        .collect();
    if targets.is_empty() {
        return;
    }
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        for (endpoint_id, sni, mut info) in targets {
            let selected = info
                .resolved_ips
                .iter()
                .find(|ip| ip.is_ipv4())
                .or_else(|| info.resolved_ips.first())
                .copied();
            if let Some(ip) = selected {
                info.host_features = checker.ip_features(ip);
            }
            info.sni_whitelisted = sni.as_deref().and_then(|s| checker.sni_features(s));

            if let Some(t) = tx.clone() {
                let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
            }
        }
    });
}

/// Seed the outbound-IP country cache at profile load.
///
/// Collects the distinct exit IPs persisted on links (`Latency::Real.ip`),
/// looks each up in mmdb and fills `state.outbound_country_cache` — async, so
/// IPs render immediately and flags a moment later. Survives reruns: the
/// source IPs are persisted and this re-runs at every load.
pub fn spawn_outbound_countries(state: &mut AppState) {
    let mut ips = std::collections::HashSet::new();
    for row in &state.endpoints {
        for link in &row.links {
            if let Some(xray_tui_db::models::Latency::Real { ip: Some(ip), .. }) = &link.latency {
                ips.insert(ip.clone());
            }
        }
    }
    if ips.is_empty() {
        return;
    }
    let geo = state.geo_ip.clone();
    let cache = state.outbound_country_cache.clone();
    tokio::spawn(async move {
        let Some(geo) = geo else { return };
        for ip in ips {
            let Ok(ipaddr) = ip.parse::<std::net::IpAddr>() else {
                continue;
            };
            let country = match geo.location_by_ip(ipaddr).await {
                Ok(Some(loc)) => Some(loc.country.to_string()),
                _ => None,
            };
            cache.lock().push(ip, country);
        }
    });
}

/// Record the exit (egress) IP + country of a real ping on the endpoint that
///
/// owns `protocol_id`. The IP is parsed from real-ping `ip_info`
/// (`"{ip} | {country}"`); the country hint string is replaced by the mmdb ISO
/// code. Sends a full copy of the entry with outbound fields set.
pub fn spawn_outbound_enrich(state: &mut AppState, endpoint_id: i64, ip_info: Option<String>) {
    let Some(ip_info) = ip_info else {
        return;
    };
    let Some((outbound_ip, _hint)) = parse_ip_info(&ip_info) else {
        return;
    };
    // The caller knows the endpoint that ran the probe (the SpeedTestResult
    // event carries it). Never resolve by protocol: shared `Protocol` rows
    // would point the exit IP at the first owner's endpoint.
    if !state
        .endpoints
        .iter()
        .any(|r| r.endpoint.id.get() == endpoint_id)
    {
        return;
    }

    let geo = state.geo_ip.clone();
    let mut info = state
        .endpoint_info
        .get(&endpoint_id)
        .cloned()
        .unwrap_or_default();
    info.outbound_ip = Some(outbound_ip);
    let cache = state.outbound_country_cache.clone();
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        // Phase 1: show the exit IP immediately; the country lookup follows
        // (mmdb may download on first use — must not delay the IP).
        if let Some(t) = tx.as_ref() {
            let _ = t.try_send(CoreEvent::EndpointInfoUpdated {
                endpoint_id,
                info: info.clone(),
            });
        }
        if let Some(geo) = &geo {
            match geo.location_by_ip(outbound_ip).await {
                Ok(Some(loc)) => {
                    info.outbound_country = Some(loc.country.to_string());
                    cache
                        .lock()
                        .push(outbound_ip.to_string(), Some(loc.country.to_string()));
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(target: "tui::ops::enrich", "outbound geo lookup failed: {e}");
                }
            }
        }
        if let Some(t) = tx {
            let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flag a DNS endpoint shows must come from `endpoint_ip` when it is
    /// already stored: the seed reads it back and the mmdb is never asked.
    /// (The state's `geo_ip` is whatever the fixture config built — the
    /// assertion holds because the stored country short-circuits the
    /// lookup, not because a database happened to be present.)
    #[tokio::test]
    async fn stored_country_reaches_the_ui_without_the_mmdb() {
        use crate::ops::profiles::test_support::{fake_row, test_state};
        let mut row = fake_row(7, "dns.example", 1);
        row.endpoint.host_type = HostType::Dns;
        row.endpoint.resolved_at = Some(60);
        row.resolved_ips = vec!["1.1.1.1".parse().expect("ip")];
        let mut state = test_state(vec![row]).await;
        state
            .db
            .set_endpoint_ip_country(EndpointId::new(7), "1.1.1.1".parse().expect("ip"), "US")
            .await
            .expect("stored country");

        spawn_enrich_ip_hosts(&mut state);
        for _ in 0..100 {
            let _ = state.poll_core_events().await;
            if state
                .endpoint_info
                .get(&7)
                .and_then(|i| i.country.clone())
                .is_some()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(
            state
                .endpoint_info
                .get(&7)
                .and_then(|i| i.country.clone())
                .as_deref(),
            Some("US"),
            "the stored country is the one the row renders"
        );
    }

    #[test]
    fn resolvable_hostname_accepts_valid_domains() {
        for host in [
            "example.com",
            "cdn.example.com",
            "like.myolddomain.kesug.com",
            "x.com",
            "a-b.example.com",
            "example.com.", // trailing-dot FQDN
        ] {
            assert!(is_resolvable_hostname(host), "{host} should be resolvable");
        }
    }

    #[test]
    fn resolvable_hostname_rejects_garbage() {
        // Observed in dumps: plugin URLs, port suffixes, Telegram-channel
        // names with underscores, spaces and non-ASCII — all hickory errors.
        for host in [
            "",
            "1.2.3.4:8388?plugin=obfs-local;obfs=tls;obfs-host=x",
            "v2ray_configs_poolsTELEGRAM.kesug.com",
            "foo bar.com",
            "foo..com",
            ".foo.com",
            "-foo.com",
            "foo-.com",
            "foo@bar.com",
            "exämple.com",
            "foo_com.com",
        ] {
            assert!(!is_resolvable_hostname(host), "{host:?} should be rejected");
        }
    }

    #[test]
    fn test_iso_to_flag() {
        assert_eq!(crate::iso_to_flag("US"), "\u{1F1FA}\u{1F1F8}");
        assert_eq!(crate::iso_to_flag("USA"), "\u{1F3F4}");
        assert_eq!(crate::iso_to_flag(""), "\u{1F3F4}");
        assert_eq!(crate::iso_to_flag("U1"), "\u{1F3F4}");
    }

    #[test]
    fn test_should_resolve() {
        let now = 1_000_000i64;
        let ttl = 300;
        // No entry → resolve
        assert!(should_resolve(None, false, ttl, now));
        // Fresh DNS entry → skip
        let fresh = EndpointInfo {
            resolved_ips: vec!["1.2.3.4".parse().unwrap()],
            resolved_at_secs: Some(now),
            ..Default::default()
        };
        assert!(!should_resolve(Some(&fresh), false, ttl, now));
        // Stale → resolve
        let stale = EndpointInfo {
            resolved_at_secs: Some(now - ttl - 1),
            ..fresh.clone()
        };
        assert!(should_resolve(Some(&stale), false, ttl, now));
        // force → resolve regardless
        assert!(should_resolve(Some(&fresh), true, ttl, now));
        // An outbound-only entry (no address, no attempt) carries no resolution
        // information, so it must NOT read as "already resolved": treating it
        // that way is what made `[name]` terminal for a DNS host whose first
        // real ping landed before its first resolution attempt.
        let outbound_only = EndpointInfo {
            outbound_ip: Some("5.6.7.8".parse().unwrap()),
            ..Default::default()
        };
        assert!(should_resolve(Some(&outbound_only), false, ttl, now));
        // ...but an entry whose address set is non-empty with no attempt stamp
        // is an IP host, and never re-resolves.
        let ip_host = EndpointInfo {
            resolved_at_secs: None,
            ..fresh
        };
        assert!(!should_resolve(Some(&ip_host), false, ttl, now));
        assert!(!should_resolve(Some(&ip_host), true, ttl, now));
    }

    #[test]
    fn test_parse_ip_info() {
        assert_eq!(
            parse_ip_info("1.2.3.4 | Germany").map(|(ip, _)| ip),
            Some("1.2.3.4".parse().unwrap())
        );
        assert_eq!(
            parse_ip_info("1.2.3.4 | Germany").map(|(_, c)| c),
            Some(Some("Germany".to_string()))
        );
        assert!(parse_ip_info("not-an-ip | x").is_none());
    }

    #[test]
    fn test_extract_sni() {
        // Real vless+reality URL → typed rows → Security embed sni column
        let parsed = xray_tui_config::import_export::parse_share_url(
            "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443?security=reality&sni=chat.example.com&encryption=none&type=tcp",
            &xray_tui_config::import_export::ValidationSettings::default(),
        )
        .expect("parse vless url");
        let protocol = crate::state::protocol_from_parsed(&parsed.parsed);
        assert_eq!(extract_sni(&protocol).as_deref(), Some("chat.example.com"));
    }
}
