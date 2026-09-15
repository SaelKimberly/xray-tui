//! Native real-ping probe policy: one tunnel to the ping URL's host, one HTTP
//! request over it, then (on success) a second tunnel to fetch the exit IP.
//!
//! Transport, TLS and HTTP live in `xray_tui_native::probe`; this module owns
//! only the policy around it (URL shape, retry fan-out, the latency span, the
//! exit-IP parse and the error strings the event path persists).
//!
//! Latency span: proxy dial → protocol handshake → target TCP + TLS → request →
//! response head. That is the same span the subprocess probe covered minus the
//! loopback SOCKS hop, so persisted delays stay comparable across the cutover.
//!
//! The engine is responsible for DNS of the proxy server (once per attempt,
//! bounded by its own DIAL deadline) and the target hostname travels in the
//! protocol header, so the remote side resolves it — exactly as before.

use std::time::Duration;

use xray_tui_db::models::Endpoint;
use xray_tui_native::addr::TargetAddr;
use xray_tui_native::context::NativeConnectParams;
use xray_tui_native::probe::{self, ProbeMethod, ProbeRequest};
use xray_tui_proto::proto_spec::ProtocolConfig;

use crate::ops::native_connect::endpoint_essentials;

/// Exit-IP fetch budget (an `ip-api` JSON response, not a latency probe).
const IP_INFO_TIMEOUT: Duration = Duration::from_secs(10);

/// Parameters for one native real ping.
pub struct NativeProbeReq<'a> {
    pub ping_url: &'a str,
    pub ip_api_url: &'a str,
    /// Per-attempt budget (the whole dial + request for one attempt).
    pub timeout: Duration,
    /// Concurrent attempts; the fastest 2xx wins.
    pub retries: u32,
}

/// A successful native real ping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeProbeResult {
    pub latency_ms: u64,
    /// `"<ip> | <country>"` when the IP API answered, else `None`.
    pub ip_info: Option<String>,
}

/// One parsed probe URL: the pieces `probe::fetch` needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeUrl {
    pub https: bool,
    pub host: String,
    pub port: u16,
    /// Origin-form path, query string included.
    pub path: String,
}

/// Parse a probe URL into scheme/host/port/path.
///
/// Only `http`/`https` are accepted: the probe has no other transport, and a
/// wrong scheme must fail loudly instead of silently probing port 0.
pub fn parse_probe_url(url: &str) -> Result<ProbeUrl, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("invalid URL {url:?}: {e}"))?;
    let https = match parsed.scheme() {
        "https" => true,
        "http" => false,
        other => return Err(format!("unsupported URL scheme {other:?} in {url:?}")),
    };
    let host = parsed
        .host_str()
        .ok_or_else(|| format!("URL {url:?} has no host"))?
        .to_string();
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| format!("URL {url:?} has no port"))?;
    let mut path = parsed.path().to_string();
    if let Some(query) = parsed.query() {
        path.push('?');
        path.push_str(query);
    }
    Ok(ProbeUrl {
        https,
        host,
        port,
        path,
    })
}

/// `"<ip> | <country>"` from an `ip-api` JSON body; `None` when the body is
/// not the expected object (the probe's latency result does not depend on it).
pub fn parse_ip_info(body: &[u8]) -> Option<String> {
    let json: serde_json::Value = serde_json::from_slice(body).ok()?;
    let ip = json.get("query").and_then(serde_json::Value::as_str)?;
    let country = json
        .get("country")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("-");
    Some(format!("{ip} | {country}"))
}

/// Run one native real ping for `endpoint` + the already-loaded protocol config.
///
/// `Err` carries the user-facing failure text the event path persists.
pub async fn real_ping(
    endpoint: &Endpoint,
    config: &ProtocolConfig,
    req: &NativeProbeReq<'_>,
) -> Result<NativeProbeResult, String> {
    let target = parse_probe_url(req.ping_url)?;
    let params = NativeConnectParams::new(
        config.clone(),
        endpoint_essentials(endpoint),
        TargetAddr::new(target.host.as_str(), target.port),
    );

    let latency_ms = probe_attempts(&params, &target, req).await?;
    let ip_info = fetch_ip_info(&params, req.ip_api_url).await;
    Ok(NativeProbeResult {
        latency_ms,
        ip_info,
    })
}

/// Fan out `retries` attempts and return the fastest 2xx latency in ms.
///
/// Spawned tasks (not one polling loop): each attempt runs the engine's TLS /
/// REALITY handshake, which is CPU-bound, and the attempts must not serialize
/// on one worker thread.
async fn probe_attempts(
    params: &NativeConnectParams,
    target: &ProbeUrl,
    req: &NativeProbeReq<'_>,
) -> Result<u64, String> {
    let attempts = req.retries.max(1);
    let mut set = tokio::task::JoinSet::new();
    for _ in 0..attempts {
        let params = params.clone();
        let host = target.host.clone();
        let path = target.path.clone();
        let (port, https, timeout) = (target.port, target.https, req.timeout);
        set.spawn(async move {
            let request = ProbeRequest {
                host: &host,
                port,
                https,
                method: ProbeMethod::Head,
                path: &path,
                timeout,
            };
            probe::fetch(params, &request)
                .await
                .map(|r| (r.status, r.elapsed.as_millis() as u64))
                .map_err(|e| e.to_string())
        });
    }

    let mut best: Option<u64> = None;
    let mut last_error: Option<String> = None;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok((status, ms))) if (200..300).contains(&status) => {
                best = Some(best.map_or(ms, |b| b.min(ms)));
            }
            Ok(Ok((status, _))) => {
                last_error = Some(format!("unexpected status {status}"));
            }
            Ok(Err(e)) => last_error = Some(e),
            Err(e) => last_error = Some(format!("probe task failed: {e}")),
        }
    }

    best.ok_or_else(|| last_error.unwrap_or_else(|| "all attempts failed".to_string()))
}

/// Exit IP + country through a second tunnel; any failure is `None` (the
/// latency result stands on its own — parity with the old probe).
async fn fetch_ip_info(params: &NativeConnectParams, ip_api_url: &str) -> Option<String> {
    let target = parse_probe_url(ip_api_url).ok()?;
    let request = ProbeRequest {
        host: &target.host,
        port: target.port,
        https: target.https,
        method: ProbeMethod::Get,
        path: &target.path,
        timeout: IP_INFO_TIMEOUT,
    };
    let response = probe::fetch(params.clone(), &request).await.ok()?;
    if !(200..300).contains(&response.status) {
        return None;
    }
    parse_ip_info(&response.body)
}

#[cfg(test)]
mod tests {
    use super::{parse_ip_info, parse_probe_url};

    #[test]
    fn probe_url_defaults_the_port_per_scheme() {
        let https = parse_probe_url("https://www.gstatic.com/generate_204").unwrap();
        assert!(https.https);
        assert_eq!(https.host, "www.gstatic.com");
        assert_eq!(https.port, 443);
        assert_eq!(https.path, "/generate_204");

        let http = parse_probe_url("http://example.com").unwrap();
        assert!(!http.https);
        assert_eq!(http.port, 80);
        assert_eq!(http.path, "/");
    }

    #[test]
    fn probe_url_keeps_explicit_port_and_query() {
        let parsed = parse_probe_url("https://ip-api.com:8443/json/?fields=query,country").unwrap();
        assert_eq!(parsed.port, 8443);
        assert_eq!(parsed.path, "/json/?fields=query,country");
    }

    #[test]
    fn probe_url_rejects_unsupported_scheme_and_garbage() {
        assert!(parse_probe_url("socks5://127.0.0.1:1080").is_err());
        assert!(parse_probe_url("www.gstatic.com/generate_204").is_err());
    }

    #[test]
    fn ip_info_reads_query_and_country() {
        let body = br#"{"query":"203.0.113.7","country":"Germany","isp":"x"}"#;
        assert_eq!(
            parse_ip_info(body).as_deref(),
            Some("203.0.113.7 | Germany")
        );
    }

    #[test]
    fn ip_info_missing_country_falls_back_and_garbage_is_none() {
        assert_eq!(
            parse_ip_info(br#"{"query":"203.0.113.7"}"#).as_deref(),
            Some("203.0.113.7 | -")
        );
        assert_eq!(parse_ip_info(b"<html>nope</html>"), None);
        assert_eq!(parse_ip_info(br#"{"country":"Germany"}"#), None);
    }
}
