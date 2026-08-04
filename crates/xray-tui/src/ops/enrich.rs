//! Background enrichment of endpoint display data.
//!
//! All network- and IO-bound work (DNS resolution, GeoIP mmdb lookups,
//! whitelist checks) runs in spawned tokio tasks that report back through
//! `CoreEvent::EndpointInfoUpdated`. The UI thread never blocks on them.
//! Every failure degrades to defaults (no flag / `🏴`) with a `tracing::warn!`.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use xray_tui_db::models::{Endpoint, ProtocolRow};

use crate::AppState;
use crate::profile_to_fields;
use crate::types::{CoreEvent, EndpointInfo};

/// Rebuild the same `Profile` connect.rs builds from an endpoint row + protocol.
pub fn protocol_row_to_profile(
    ep: &Endpoint,
    p: &ProtocolRow,
) -> xray_tui_config::import_export::Profile {
    xray_tui_config::import_export::Profile {
        id: p.id,
        sig: p.sig,
        cred_hash: p.cred_hash,
        proto_kind: p.proto_kind.clone(),
        spec_blob: p.spec_blob.clone(),
        config_type: p.config_type,
        core_type: p.core_type.clone(),
        address: ep.host.clone(),
        port: ep.port,
        transport: p.transport.clone(),
        security: p.security.clone(),
        created_at: p.created_at,
        remarks: None,
    }
}

/// SNI from the profile's typed config: `security().sni()` covers both
/// `tls` and `reality` variants. Falls back to the form-field path (first key
/// "sni" or "*.sni") for opaque/legacy blobs.
fn extract_sni(profile: &xray_tui_config::import_export::Profile) -> Option<String> {
    use xray_tui_proto::proto_spec::ProtoSpec;
    if let Some(config) = xray_tui_config::import_export::profile_config(profile) {
        if let Some(sni) = config.security().and_then(|s| s.sni()) {
            return Some(sni.to_string());
        }
    }
    profile_to_fields(profile)
        .into_iter()
        .find(|(k, _)| k == "sni" || k.ends_with(".sni"))
        .map(|(_, v)| v)
}

/// True when a resolution must run: no entry, or a DNS entry older than the
/// TTL, or `force`. IP-host entries (`resolved_at_secs: None`) never re-resolve.
fn should_resolve(entry: Option<&EndpointInfo>, force: bool, ttl_secs: i64, now_secs: i64) -> bool {
    match entry {
        None => true,
        Some(e) => match e.resolved_at_secs {
            None => false,
            Some(ts) => force || now_secs - ts >= ttl_secs,
        },
    }
}

/// `"{ip} | {country}"` (real-ping ip_info format) → `(ip, country-hint)`.
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
        .find(|r| r.endpoint.id == endpoint_id)
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
    let host = row.endpoint.host.clone();
    let host_type = row.endpoint.host_type.clone();
    let active = row.active_protocol().clone();
    let ep = row.endpoint.clone();
    let sni = extract_sni(&protocol_row_to_profile(&ep, &active));
    let tx = state.core_event_tx.clone();

    tokio::spawn(async move {
        let now = unix_now();
        // DNS lookup or direct IP parse
        let (ips, resolved_at) = match host_type.as_str() {
            "ipv4" | "ipv6" => (
                host.parse::<IpAddr>()
                    .map(|ip| vec![ip])
                    .unwrap_or_default(),
                None,
            ),
            _ => match &dns {
                Some(r) => {
                    // Overall deadline: resolver init (DNSCrypt list
                    // download) plus lookups over many name servers can
                    // otherwise stall indefinitely.
                    match tokio::time::timeout(Duration::from_secs(8), r.lookup_ip(&host, false))
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
                            tracing::warn!(
                                target: "tui::ops::enrich",
                                "DNS lookup of {host} failed: {e}"
                            );
                            (Vec::new(), Some(now))
                        }
                        Err(_) => {
                            tracing::warn!(
                                target: "tui::ops::enrich",
                                "DNS lookup of {host} timed out"
                            );
                            (Vec::new(), Some(now))
                        }
                    }
                }
                None => (Vec::new(), None),
            },
        };

        let mut info = EndpointInfo {
            resolved_ips: ips,
            country: None,
            host_features: Default::default(),
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
/// entry yet — IP hosts (parse host, no DNS) and DNS hosts with a persisted
/// `resolved_as` (from the endpoints table; no network). Geo + whitelist
/// features are filled in the same task.
pub fn spawn_enrich_ip_hosts(state: &mut AppState) {
    let targets: Vec<(i64, Endpoint, ProtocolRow, Option<String>, Option<i64>)> = state
        .endpoints
        .iter()
        .filter(|r| {
            let ht = r.endpoint.host_type.as_str();
            ht == "ipv4" || ht == "ipv6" || r.endpoint.resolved_as.is_some()
        })
        .filter(|r| !state.endpoint_info.contains_key(&r.endpoint.id))
        .map(|r| {
            (
                r.endpoint.id,
                r.endpoint.clone(),
                r.active_protocol().clone(),
                r.endpoint.resolved_as.clone(),
                r.endpoint.resolved_at,
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
        for (endpoint_id, ep, active, cached_as, cached_at) in targets {
            let sni = extract_sni(&protocol_row_to_profile(&ep, &active));
            let mut info = if let Some(as_str) = cached_as {
                // DNS host with a persisted resolution — reuse it, no network.
                EndpointInfo {
                    resolved_ips: as_str
                        .split(',')
                        .filter_map(|s| s.parse::<IpAddr>().ok())
                        .collect(),
                    country: None,
                    host_features: Default::default(),
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
                    host_features: Default::default(),
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
/// checker has loaded. Runs on every launch — features are never persisted, so
/// cached entries get fresh membership. Sends a full copy of each entry.
pub fn spawn_whitelist_pass(state: &mut AppState) {
    let Some(checker) = state.host_features.clone() else {
        return;
    };
    let targets: Vec<(i64, Endpoint, ProtocolRow, EndpointInfo)> = state
        .endpoints
        .iter()
        .map(|r| {
            (
                r.endpoint.id,
                r.endpoint.clone(),
                r.active_protocol().clone(),
                state
                    .endpoint_info
                    .get(&r.endpoint.id)
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
        for (endpoint_id, ep, active, mut info) in targets {
            let sni = extract_sni(&protocol_row_to_profile(&ep, &active));
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
        .find(|r| r.protocols.iter().any(|p| p.id == protocol_id))
        .map(|r| r.endpoint.id)
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
                    tracing::warn!(target: "tui::ops::enrich", "outbound geo lookup failed: {e}")
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
    use crate::format_ts;

    #[test]
    fn test_format_ts() {
        assert_eq!(format_ts(1_752_595_200), "2025-07-15T16:00:00");
        assert_eq!(format_ts(0), "1970-01-01T00:00:00");
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
        // Real vless+reality URL → typed spec blob → SecurityConfig::sni()
        let parsed = xray_tui_config::import_export::parse_share_url(
            "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443?security=reality&sni=chat.example.com&encryption=none&type=tcp",
            &xray_tui_config::import_export::ValidationSettings::default(),
        )
        .expect("parse vless url");
        let profile = xray_tui_config::import_export::Profile {
            id: parsed.sig ^ parsed.cred_hash,
            sig: parsed.sig,
            cred_hash: parsed.cred_hash,
            proto_kind: parsed.proto_kind,
            spec_blob: parsed.spec_blob,
            config_type: parsed.config_type,
            core_type: parsed.core_type,
            address: parsed.host,
            port: i32::from(parsed.port),
            transport: parsed.transport,
            security: parsed.security,
            created_at: 0,
            remarks: None,
        };
        assert_eq!(extract_sni(&profile).as_deref(), Some("chat.example.com"));
    }
}
