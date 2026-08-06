//! `ShadowsocksR` (`ssr://`) URL parsing.
//!
//! # Format
//! ```text
//! ssr://<base64url(host:port:protocol:method:obfs:base64(password)/?params)>
//! ```
//!
//! The entire body after `ssr://` is a single base64-encoded payload with
//! 6 colon-separated fields plus optional `/`-delimited query parameters.
//!
//! # Fields
//!
//! | Position | Field      | Purpose                         | Examples                        |
//! |----------|------------|---------------------------------|---------------------------------|
//! | 1        | `host`     | Server address                  | `example.com`                   |
//! | 2        | `port`     | Server port                     | `443`                           |
//! | 3        | `protocol` | Protocol plugin                 | `origin`, `auth_aes128_md5`     |
//! | 4        | `method`   | Encryption cipher               | `rc4-md5`, `aes-256-cfb`        |
//! | 5        | `obfs`     | Obfuscation plugin              | `plain`, `http_simple`          |
//! | 6        | `password` | Base64-encoded shared secret    | `base64(password)`              |
//!
//! # Query Parameters (after `/?`)
//!
//! | Key        | Encoding | Purpose                         |
//! |------------|----------|---------------------------------|
//! | `group`    | base64   | Provider/group name             |
//! | `remarks`  | base64   | Node name (display)             |
//! | `obfsparam`| base64   | Obfuscation parameter (e.g., host) |
//! | `protoparam`| base64  | Protocol parameter              |
//!
//! # Edge Cases
//! - Port is last-5th colon-delimited field (handles IPv6 with multiple colons)
//! - Password may contain `/?` or `?` split for query params
//! - Trailing non-base64 garbage (Telegram annotation) is stripped before decode
//! - `remarks` extracted from query params (base64-decoded, URL-safe alphabet)
//!
//! # References
//! - subconverter: `subparser.cpp` `explodeSSR()`, `subexport.cpp`
//! - sing-box: `option/shadowsocksr.go`

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText};

use super::ProtoIdentity;
use super::common::SecurityConfig;
use super::core_mapping;
use super::utils;
use super::{
    ConfigKind, EndpointEssentials, ParseError, ParsedProto, ProtoSpec, ProtocolConfig,
    ProtocolEssentials, ProtocolKind,
};
use crate::clash::{ClashProxy, ClashSSR};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_to_endpoint, host_kind_for};

/// ShadowsocksR protocol configuration — the identity payload (sans host/port).
///
/// The endpoint (server host/port) lives in [`EndpointEssentials`] on the
/// [`ParsedProto`] boundary; this struct only carries endpoint-free protocol
/// parameters, so the same config pointed at different servers shares one
/// identity.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct SsrConfig {
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub protocol: TinyText,
    pub method: TinyText,
    pub obfs: TinyText,
    pub password: String,
    pub params: std::collections::HashMap<String, String>,
    pub remarks: Option<TinyText>,
}

impl SsrConfig {
    /// Parse an SSR URL into the parse boundary: [`ParsedProto`] with the
    /// endpoint essentials (host/port) split out and the identity payload
    /// ([`ProtocolEssentials::config`]) holding only endpoint-free protocol
    /// parameters.
    pub fn try_parse_proto(raw: &RawUrlX<'_>) -> Result<ParsedProto, ParseError> {
        let clean_userinfo = clean_ssr_userinfo(raw.userinfo);
        let decoded = utils::decode_base64(clean_userinfo)
            .map_err(|_| ParseError::InvalidStructure(SchemeX::SSR))?;
        let text =
            String::from_utf8(decoded).map_err(|_| ParseError::InvalidStructure(SchemeX::SSR))?;

        let parts: Vec<&str> = text.split(':').collect();
        if parts.len() < 6 {
            return Err(ParseError::InvalidStructure(SchemeX::SSR));
        }

        // Index from end: last 5 are port, protocol, method, obfs, password.
        // Everything before is the host (handles IPv6 with colons).
        let raw_host = parts[..parts.len() - 5].join(":");
        let raw_port = parts[parts.len() - 5];
        let protocol = TinyText::from(parts[parts.len() - 4]);
        let method = TinyText::from(parts[parts.len() - 3]);
        let obfs = TinyText::from(parts[parts.len() - 2]);
        let raw_password = parts[parts.len() - 1..].join(":");

        let (password, query_part) = raw_password
            .split_once("/?")
            .or_else(|| raw_password.split_once('?'))
            .unwrap_or((&raw_password, ""));

        let mut params = std::collections::HashMap::new();
        params.insert("protocol".into(), protocol.to_string());
        params.insert("obfs".into(), obfs.to_string());

        if !query_part.is_empty() {
            for pair in query_part.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    params.insert(k.to_string(), v.to_string());
                }
            }
        }

        let remarks = params.remove("remarks").map(|r| {
            base64::prelude::BASE64_URL_SAFE_NO_PAD
                .decode(r.trim_end_matches('='))
                .ok()
                .and_then(|d| String::from_utf8(d).ok().map(TinyText::from))
                .unwrap_or_else(|| TinyText::from(r.as_str()))
        });

        let parsed_host = utils::parse_host(&raw_host)?;
        let parsed_port: u16 = raw_port
            .parse()
            .map_err(|_| ParseError::InvalidPort(raw_port.to_string().into()))?;

        // Endpoint essentials: host/port live here, never in the config payload.
        let mut endpoint = EndpointEssentials::new(parsed_host.to_str().into_owned(), parsed_port);
        endpoint.host_type = host_kind_for(&parsed_host);

        let config = Self {
            security: SecurityConfig::default(),
            protocol,
            method,
            obfs,
            password: password.to_string(),
            params,
            remarks,
        };
        Ok(ParsedProto {
            endpoints: vec![endpoint],
            protocol: ProtocolEssentials {
                proto_kind: ProtocolKind::ShadowsocksR,
                config_type: ConfigKind::ShareUrl,
                core_type: core_mapping::resolve_core(ProtocolKind::ShadowsocksR, None, None),
                config: ProtocolConfig::Ssr(config),
            },
        })
    }

    /// Rebuild the share URL from this endpoint-free config plus the endpoint
    /// essentials. Endpoint host/port come from `endpoint`.
    pub fn reconstruct_proto(&self, endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        let mut query_str = String::new();
        let mut sorted_params: Vec<_> = self.params.iter().collect();
        sorted_params.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in &sorted_params {
            let k_str = k.as_str();
            // protocol and obfs are already in the colon-delimited fields above
            if k_str == "protocol" || k_str == "obfs" {
                continue;
            }
            if !query_str.is_empty() {
                query_str.push('&');
            }
            query_str.push_str(format!("{k}={v}").as_str());
        }
        // Add remarks back to the query string (base64-encoded, URL-safe no-pad)
        if let Some(remarks) = &self.remarks {
            if !query_str.is_empty() {
                query_str.push('&');
            }
            query_str.push_str("remarks=");
            query_str.push_str(&base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(remarks.as_bytes()));
        }

        let host = endpoint.host.as_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", endpoint.port)
        } else {
            format!("{host}:{}", endpoint.port)
        };
        let raw = format!(
            "{hostport}:{proto}:{method}:{obfs}:{password}/?{query_str}",
            proto = self.protocol,
            method = self.method,
            obfs = self.obfs,
            password = self.password,
            query_str = query_str,
        );
        let encoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(raw.as_bytes());
        Ok(format!("ssr://{encoded}"))
    }
}

impl SsrConfig {
    /// Serialize this endpoint-free config plus the endpoint to a Clash proxy
    /// entry. Endpoint host/port are taken from `endpoint`.
    pub fn to_clash_proto(
        &self,
        endpoint: &EndpointEssentials,
    ) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::ShadowsocksR(ClashSSR {
            name,
            server: endpoint.host.clone(),
            port: endpoint.port,
            cipher: self.method.to_string(),
            password: self.password.clone(),
            protocol: self.protocol.to_string(),
            obfs: self.obfs.to_string(),
            protocol_param: self.params.get("protocol_param").cloned(),
            obfs_param: self.params.get("obfs_param").cloned(),
        }))
    }

    /// Parse a Clash proxy entry into the parse boundary: `server`/`port`
    /// become the endpoint essentials; the config payload is endpoint-free.
    pub fn try_from_clash_proto(proxy: &ClashProxy) -> Result<ParsedProto, ParseError> {
        match proxy {
            ClashProxy::ShadowsocksR(c) => {
                let mut params = std::collections::HashMap::new();
                params.insert("protocol".into(), c.protocol.clone());
                params.insert("obfs".into(), c.obfs.clone());
                if let Some(pp) = &c.protocol_param {
                    params.insert("protocol_param".into(), pp.clone());
                }
                if let Some(op) = &c.obfs_param {
                    params.insert("obfs_param".into(), op.clone());
                }
                let config = Self {
                    security: SecurityConfig::default(),
                    protocol: TinyText::from(c.protocol.as_str()),
                    method: TinyText::from(c.cipher.as_str()),
                    obfs: TinyText::from(c.obfs.as_str()),
                    password: c.password.clone(),
                    params,
                    remarks: match c.name.as_str() {
                        "" => None,
                        s => Some(TinyText::from(s)),
                    },
                };
                Ok(ParsedProto {
                    endpoints: vec![clash_to_endpoint(&c.server, c.port)],
                    protocol: ProtocolEssentials {
                        proto_kind: ProtocolKind::ShadowsocksR,
                        config_type: ConfigKind::ShareUrl,
                        core_type: core_mapping::resolve_core(
                            ProtocolKind::ShadowsocksR,
                            None,
                            None,
                        ),
                        config: ProtocolConfig::Ssr(config),
                    },
                })
            }
            _ => Err(ParseError::Unknown("expected ssr clash proxy".into())),
        }
    }
}

/// Strip trailing non-base64 garbage (Telegram annotation text and decorative
/// hyphens) from the SSR userinfo before base64 decoding.
///
/// Strategy:
/// 1. If the base64 has `=` padding, everything after the last `=` that is
///    hyphens followed by non-ASCII is stripped.
/// 2. For no-pad base64, find the first occurrence of 3+ consecutive decorative
///    `-` or `_` that is followed by non-ASCII and truncate there.
/// 3. If neither heuristic triggers, return the string unchanged.
fn clean_ssr_userinfo(s: &str) -> &str {
    // Try padded-base64 heuristic first
    if let Some(last_eq) = s.rfind('=') {
        let after = &s[last_eq + 1..];
        match after.trim_start_matches(['-', '_']).as_bytes().first() {
            None => return &s[..=last_eq],
            Some(n) if !n.is_ascii() => return &s[..=last_eq],
            _ => (),
        }
    }

    // For NO_PAD base64: find 3+ consecutive '-' or '_' that are followed
    // by non-ASCII (Telegram annotation text). 3+ consecutive hyphens are
    // virtually never valid URL-safe base64 data.
    let bytes = s.as_bytes();
    let mut hyphen_run: u32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'-' || b == b'_' {
            hyphen_run += 1;
        } else {
            if hyphen_run >= 3 && (i >= bytes.len() || !bytes[i].is_ascii()) {
                return &s[..i - hyphen_run as usize];
            }
            hyphen_run = 0;
        }
    }
    // Handle case where the run extends to the end
    if hyphen_run >= 3 {
        return &s[..s.len() - hyphen_run as usize];
    }

    s
}

/// Legacy [`ProtoSpec`] bridge — kept so `ProtocolConfig` dispatch (and the
/// `Proto` consumer in xray-tui-core) compile unchanged.
///
/// DEGRADED PATH (documented): `try_parse`/`try_from_clash` still work by
/// delegating to the `*_proto` variants and discarding the parsed endpoints;
/// `to_clash`/`reconstruct` return errors because the config no longer stores
/// host/port. Import/export rewires to the `*_proto` variants in T11 (phase D
/// builders take the endpoint separately).
impl ProtoSpec for SsrConfig {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let parsed = Self::try_parse_proto(raw)?;
        match parsed.protocol.config {
            ProtocolConfig::Ssr(config) => Ok(config),
            // Parser invariant: an ssr URL always yields an SsrConfig.
            _ => Err(ParseError::Unknown(
                "ssr URL parsed to a non-ssr config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::reconstruct_proto`] with the endpoint.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::InvalidHost(
            "ssr config no longer stores host/port; use SsrConfig::reconstruct_proto(endpoint)"
                .into(),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::SSR
    }

    /// `None` — the endpoint host moved to [`EndpointEssentials`] (T5).
    fn host(&self) -> Option<&HostSpec> {
        None
    }

    /// `None` — the endpoint port moved to [`EndpointEssentials`] (T5).
    fn port(&self) -> Option<u16> {
        None
    }

    fn remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    fn transport_type(&self) -> Option<&str> {
        None
    }

    /// # Errors
    ///
    /// If the Clash proxy doesn't match this protocol type.
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        let parsed = Self::try_from_clash_proto(proxy)?;
        match parsed.protocol.config {
            ProtocolConfig::Ssr(config) => Ok(config),
            _ => Err(ParseError::Unknown(
                "ssr clash proxy parsed to a non-ssr config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::to_clash_proto`] with the endpoint.
    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "ssr config no longer stores host/port; use SsrConfig::to_clash_proto(endpoint)".into(),
        ))
    }
}

impl ProtoIdentity for SsrConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"ssr");
        // Endpoint (host/port) intentionally absent from the identity — it
        // lives on the ParsedProto boundary, never in the config payload (T5).
        let mut sorted_keys: Vec<&String> = self.params.keys().collect();
        sorted_keys.sort();
        for k in &sorted_keys {
            if k.as_str() == "remarks" || k.as_str() == "password" {
                continue;
            }
            hasher.write(k.as_bytes());
            if let Some(v) = self.params.get(*k) {
                hasher.write(v.as_bytes());
            }
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[
            ("password", self.password.as_str()),
            ("method", self.method.as_str()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ConfigKind, CoreType, HostKind, ParsedProto, ProtoSpec, ProtocolConfig, ProtocolKind,
    };
    use super::SsrConfig;
    use crate::urlx::{RawUrlX, SchemeX};

    const SSR_URL: &str = "ssr://ZXhhbXBsZS5jb206NDQzOm9yaWdpbjpyYzQtbWQ1OnBsYWluOmNHRnpjM2R2Y21RLz9ncm91cD1WR1Z6ZEVkeWIzVncmcmVtYXJrcz1WR1Z6ZEZObGNuWmxjZw";

    fn parse(url: &str) -> ParsedProto {
        SsrConfig::try_parse_proto(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    fn config(parsed: ParsedProto) -> SsrConfig {
        match parsed.protocol.config {
            ProtocolConfig::Ssr(c) => c,
            other => panic!("expected SsrConfig, got {other:?}"),
        }
    }

    /// The identity payload must be endpoint-free: no top-level `host`/`port`
    /// keys in the serialized config.
    fn assert_no_top_level_host_port(cfg: &SsrConfig) {
        let json = serde_json::to_value(cfg).expect("serialize");
        let obj = json.as_object().expect("config is an object");
        assert!(
            !obj.contains_key("host"),
            "config payload must not carry a top-level host key: {json}"
        );
        assert!(
            !obj.contains_key("port"),
            "config payload must not carry a top-level port key: {json}"
        );
    }

    /// Reconstruct round-trip via the endpoint: parse → reconstruct_proto(endpoint)
    /// → re-parse must reproduce the same ParsedProto (endpoints + config).
    fn assert_reconstruct_roundtrip(url: &str) {
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed.clone());
        let out = cfg
            .reconstruct_proto(&endpoint)
            .unwrap_or_else(|e| panic!("reconstruct failed for {url}: {e}"));
        let reparsed = parse(&out);
        assert_eq!(parsed, reparsed, "reconstruct round-trip failed for: {url}");
    }

    // ── URL parse: endpoints + config ─────────────────────────────────────

    #[test]
    fn test_ssr_basic() {
        let parsed = parse(SSR_URL);
        assert_eq!(parsed.endpoints.len(), 1);
        let ep = &parsed.endpoints[0];
        assert_eq!(ep.host, "example.com");
        assert_eq!(ep.host_type, HostKind::Dns);
        assert_eq!(ep.port, 443);

        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::ShadowsocksR);
        assert_eq!(parsed.protocol.config_type, ConfigKind::ShareUrl);
        assert_eq!(parsed.protocol.core_type, CoreType::SingBox);
        let cfg = config(parsed);
        assert_eq!(cfg.method, "rc4-md5");
        assert_eq!(cfg.remarks.as_deref(), Some("TestServer"));
        assert_no_top_level_host_port(&cfg);
    }

    #[test]
    fn test_ssr_trailing_text() {
        // Valid base64 with trailing Chinese annotation text (Telegram pattern)
        let url = "ssr://MTE2LjE2Mi4xMjAuMjY6NTYxOmF1dGhfYWVzMTI4X21kNTpjaGFjaGEyMC1pZXRmOnBsYWluOmJXSnNZVzVyTVhCdmNuUT0vP2dyb3VwPWFIUjBjSE02THk5Mk1uSmhlWE5sTG1OdmJRPT0mcHJvdG9wYXJhbT1OVEUzTmpBNlRFeE1NRGt3ZFdrNGIyeHNPQT0=必进：【全网导航】》下载地址：";
        let parsed = parse(url);
        assert_eq!(parsed.endpoints[0].host, "116.162.120.26");
        assert_eq!(parsed.endpoints[0].port, 561_u16);
        let cfg = config(parsed);
        assert_eq!(cfg.protocol, "auth_aes128_md5");
        assert_eq!(cfg.method, "chacha20-ietf");
        assert_eq!(cfg.obfs, "plain");
    }

    #[test]
    fn test_ssr_no_query() {
        // Valid SSR URL with no /? query params and a # fragment
        let url = "ssr://MTMuMzcuMjguMjM6NTk0NzpvcmlnaW46Y2hhY2hhMjAtaWV0ZjpwbGFpbjpOVGswTnc#@dark_telecom";
        let parsed = parse(url);
        assert_eq!(parsed.endpoints[0].host, "13.37.28.23");
        assert_eq!(parsed.endpoints[0].port, 5947_u16);
        let cfg = config(parsed);
        assert_eq!(cfg.protocol, "origin");
        assert_eq!(cfg.method, "chacha20-ietf");
        assert_eq!(cfg.obfs, "plain");
    }

    #[test]
    fn test_ssr_garbage_returns_err() {
        // Chinese text only — not a valid SSR URL
        let url = "ssr://的格式";
        assert!(SsrConfig::try_parse_proto(&RawUrlX::from(url)).is_err());
    }

    // ── Identity: endpoint-free uid ───────────────────────────────────────

    #[test]
    fn uid_identical_across_servers_different_across_passwords() {
        // Same payload body, different embedded host:port -> same uid.
        let url_a = SSR_URL;
        let url_b = "ssr://ZXhhbXBsZTIuY29tOjQ0MzpvcmlnaW46cmM0LW1kNTpwbGFpbjpjR0Z6YzNkdmNtUS8/Z3JvdXA9VkdWemRFZHliM1Z3JnJlbWFya3M9VkdWemRGTmxjblpsY2c";
        let a = parse(url_a);
        let b = parse(url_b);
        assert_eq!(
            a.uid(),
            b.uid(),
            "same protocol on different servers must dedup to one uid"
        );
        assert_ne!(a.sig(), 0);
    }

    // ── Reconstruct round-trip via endpoint ───────────────────────────────

    #[test]
    fn reconstruct_roundtrip_via_endpoint() {
        assert_reconstruct_roundtrip(SSR_URL);
        assert_reconstruct_roundtrip(
            "ssr://MTMuMzcuMjguMjM6NTk0NzpvcmlnaW46Y2hhY2hhMjAtaWV0ZjpwbGFpbjpOVGswTnc#@dark_telecom",
        );
    }

    // ── Clash round-trip via *_proto ──────────────────────────────────────

    #[test]
    fn clash_roundtrip_from_url_via_proto() {
        // ClashSSR only carries protocol_param/obfs_param as extra params, so
        // this URL uses only those (no group/remarks query extras that Clash
        // cannot represent — those are covered by the parse/reconstruct tests).
        let url = "ssr://ZXhhbXBsZS5jb206NDQzOm9yaWdpbjpyYzQtbWQ1OnBsYWluOmNHRnpjM2R2Y21RLz9yZW1hcmtzPVZHVnpkRk5sY25abGNn";
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        let proxy = cfg.to_clash_proto(&endpoint).expect("to clash");
        let reparsed = SsrConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(
            reparsed.endpoints[0], endpoint,
            "endpoint round-trips through clash"
        );
        assert_eq!(
            reparsed.protocol.config,
            ProtocolConfig::Ssr(cfg),
            "config round-trips through clash"
        );
    }

    #[test]
    fn clash_proxy_roundtrip_via_proto() {
        use crate::clash::{ClashProxy, ClashSSR};

        let proxy = ClashProxy::ShadowsocksR(ClashSSR {
            name: "test".into(),
            server: "example.com".into(),
            port: 443,
            cipher: "rc4-md5".into(),
            password: "sekrit".into(),
            protocol: "origin".into(),
            obfs: "plain".into(),
            protocol_param: Some("breakwa9".into()),
            obfs_param: Some("example.com".into()),
        });
        let parsed = SsrConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(parsed.endpoints[0].host, "example.com");
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Dns);
        assert_eq!(parsed.endpoints[0].port, 443);
        let cfg = match &parsed.protocol.config {
            ProtocolConfig::Ssr(c) => c,
            other => panic!("expected SsrConfig, got {other:?}"),
        };
        assert_eq!(
            cfg.params.get("protocol_param").map(String::as_str),
            Some("breakwa9")
        );
        assert_eq!(
            cfg.params.get("obfs_param").map(String::as_str),
            Some("example.com")
        );
        assert_no_top_level_host_port(cfg);
        let out = cfg.to_clash_proto(&parsed.endpoints[0]).expect("to clash");
        match (out, proxy) {
            (ClashProxy::ShadowsocksR(out), ClashProxy::ShadowsocksR(orig)) => {
                assert_eq!(out, orig)
            }
            _ => panic!("expected ssr clash proxy"),
        }
    }

    // ── Serde ─────────────────────────────────────────────────────────────

    #[test]
    fn test_serde_roundtrip() {
        let cfg = config(parse(SSR_URL));
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: SsrConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, deserialized);
        assert_no_top_level_host_port(&deserialized);
    }

    // ── Legacy trait bridge ───────────────────────────────────────────────

    #[test]
    fn legacy_bridge_parse_works_but_reconstruct_to_clash_error() {
        let bridged = SsrConfig::try_parse(&RawUrlX::from(SSR_URL)).expect("bridged parse");
        assert_eq!(bridged.schema(), SchemeX::SSR);
        assert_eq!(bridged.method, "rc4-md5");
        // host/port accessors are gone — the endpoint lives on ParsedProto.
        assert_eq!(bridged.host(), None);
        assert_eq!(bridged.port(), None);
        // Degraded legacy paths error instead of fabricating a host.
        assert!(bridged.reconstruct().is_err());
        assert!(bridged.to_clash().is_err());
    }
}
