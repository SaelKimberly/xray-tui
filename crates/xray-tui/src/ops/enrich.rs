//! Background enrichment of endpoint display data.
//!
//! All network- and IO-bound work (DNS resolution, `GeoIP` mmdb lookups,
//! whitelist checks) runs in spawned tokio tasks that report back through
//! `CoreEvent::EndpointInfoUpdated`. The UI thread never blocks on them.
//! Every failure degrades to defaults (no flag / `🏴`) with a `tracing::warn!`.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use xray_tui_db::models::{Endpoint, HostType, Protocol};
use xray_tui_host_features::HostFeatures;

use crate::AppState;
use crate::types::{CoreEvent, EndpointInfo};

/// SNI from a typed protocol row: the `security.sni` column, populated at
/// write time from `config.security().sni()` (covers both `tls` and `reality`
/// variants). The column is queryable without loading the deferred `config`
/// JSON; when the config IS loaded, the typed accessor chain is equivalent.
fn extract_sni(protocol: &Protocol) -> Option<String> {
    use xray_tui_proto::proto_spec::ProtoSpec;
    if !protocol.config.is_unloaded()
        && let Some(sni) = protocol.config.get().0.security().and_then(|s| s.sni())
    {
        return Some(sni.to_string());
    }
    protocol.security.sni.clone()
}

/// True when a resolution must run: no entry, or a DNS entry older than the
/// TTL, or `force`. IP-host entries (`resolved_at_secs: None`) never re-resolve.
const fn should_resolve(
    entry: Option<&EndpointInfo>,
    force: bool,
    ttl_secs: i64,
    now_secs: i64,
) -> bool {
    match entry {
        None => true,
        Some(e) => match e.resolved_at_secs {
            None => false,
            Some(ts) => force || now_secs - ts >= ttl_secs,
        },
    }
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
async fn fill_features(
    info: &mut EndpointInfo,
    geo: Option<&Arc<xray_tui_geoip::GeoIp>>,
    checker: Option<&Arc<xray_tui_host_features::HostFeaturesChecker>>,
    sni: Option<&str>,
) {
    let selected = info
        .resolved_ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .or_else(|| info.resolved_ips.first())
        .copied();
    if let Some(ip) = selected {
        if let Some(geo) = geo {
            match geo.location_by_ip(ip).await {
                Ok(Some(loc)) => info.country = Some(loc.country),
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
}

/// Resolve (or re-resolve) one endpoint's inbound host in the background.
///
/// TTL gate: fresh DNS entries are skipped (no network) unless `force`.
/// Results arrive via `CoreEvent::EndpointInfoUpdated`; DNS-host results are
/// persisted by the event handler so they survive launches.
pub fn spawn_dns_resolve(state: &mut AppState, endpoint_id: i64, force: bool) {
    let Some(row) = state
        .endpoints
        .iter()
        .find(|r| r.endpoint.id.get() == endpoint_id)
    else {
        return;
    };
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
    let host = row.endpoint.host.clone();
    let host_type = row.endpoint.host_type;
    let sni = row.active_protocol().and_then(|(_, p)| extract_sni(p));
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
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
                                    (ips, Some(now))
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
        let scheduler_endpoint = xray_tui_db::models::EndpointId::new(endpoint_id);
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
        fill_features(&mut info, geo.as_ref(), checker.as_ref(), sni.as_deref()).await;
        if let Some(t) = tx {
            let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
        }
    });
}

/// Startup/refresh pass: seed `endpoint_info` for every endpoint that has no
///
/// entry yet — IP hosts (parse host, no DNS) and DNS hosts with a persisted
/// `resolved_as` (from the endpoints table; no network). Geo + whitelist
/// features are filled in the same task.
pub fn spawn_enrich_ip_hosts(state: &mut AppState) {
    // One target per endpoint: endpoint id, the endpoint, its persisted
    // `resolved_as` list, the persisted `resolved_at` (unix secs), and the
    // SNI of its active protocol (None for linkless endpoints).
    let targets: Vec<(i64, Endpoint, Vec<String>, Option<i64>, Option<String>)> = state
        .endpoints
        .iter()
        .filter(|r| {
            matches!(r.endpoint.host_type, HostType::Ipv4 | HostType::Ipv6)
                || !r.endpoint.resolved_as.is_empty()
        })
        .filter(|r| !state.endpoint_info.contains_key(&r.endpoint.id.get()))
        .map(|r| {
            (
                r.endpoint.id.get(),
                r.endpoint.clone(),
                r.endpoint.resolved_as.clone(),
                r.endpoint.resolved_at.map(|t| t.as_second()),
                r.active_protocol().and_then(|(_, p)| extract_sni(p)),
            )
        })
        .collect();
    if targets.is_empty() {
        return;
    }

    let geo = state.geo_ip.clone();
    let checker = state.host_features.clone();
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        for (endpoint_id, ep, cached_as, cached_at, sni) in targets {
            let mut info = if !cached_as.is_empty() {
                // DNS host with a persisted resolution — reuse it, no network.
                EndpointInfo {
                    resolved_ips: cached_as
                        .iter()
                        .filter_map(|s| s.parse::<IpAddr>().ok())
                        .collect(),
                    country: None,
                    host_features: HostFeatures::default(),
                    sni_whitelisted: None,
                    outbound_ip: None,
                    outbound_country: None,
                    resolved_at_secs: cached_at,
                }
            } else {
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
            };
            // Phase 1 first: cached IPs/host reach the UI before geo work
            // (mmdb may download 70MB on first use — must not stall seeding).
            if let Some(t) = tx.as_ref() {
                let _ = t.try_send(CoreEvent::EndpointInfoUpdated {
                    endpoint_id,
                    info: info.clone(),
                });
            }
            fill_features(&mut info, geo.as_ref(), checker.as_ref(), sni.as_deref()).await;
            if let Some(t) = tx.clone() {
                let _ = t.try_send(CoreEvent::EndpointInfoUpdated { endpoint_id, info });
            }
        }
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

/// Record the exit (egress) IP + country of a real ping on the endpoint that
///
/// owns `protocol_id`. The IP is parsed from real-ping `ip_info`
/// (`"{ip} | {country}"`); the country hint string is replaced by the mmdb ISO
/// code. Sends a full copy of the entry with outbound fields set.
pub fn spawn_outbound_enrich(state: &mut AppState, protocol_id: i64, ip_info: Option<String>) {
    let Some(ip_info) = ip_info else {
        return;
    };
    let Some((outbound_ip, _hint)) = parse_ip_info(&ip_info) else {
        return;
    };
    let Some(endpoint_id) = state
        .endpoints
        .iter()
        .find(|r| {
            r.links
                .iter()
                .any(|l| l.protocol_id == xray_tui_db::models::ProtocolId::new(protocol_id))
        })
        .map(|r| r.endpoint.id.get())
    else {
        return;
    };

    let geo = state.geo_ip.clone();
    let mut info = state
        .endpoint_info
        .get(&endpoint_id)
        .cloned()
        .unwrap_or_default();
    info.outbound_ip = Some(outbound_ip);
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
                Ok(Some(loc)) => info.outbound_country = Some(loc.country),
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
        // IP host (resolved_at None) → never
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
        let (_, protocol, _) = crate::state::parsed_to_rows(&parsed.parsed)
            .pop()
            .expect("typed rows");
        assert_eq!(extract_sni(&protocol).as_deref(), Some("chat.example.com"));
    }
}
