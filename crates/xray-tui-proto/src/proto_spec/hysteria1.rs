//! Hysteria1 (`hysteria://` / `hy://`) URL parsing.
//!
//! # Format
//! ```text
//! hysteria://<host>:<port>?<query_params>#<remarks>
//! hy://<host>:<port>?<query_params>#<remarks>
//! ```
//!
//! Optional auth token in userinfo: `hysteria://auth@host:port?...`.
//! Configuration in query parameters, remarks in fragment.
//!
//! # Query Parameters
//!
//! | Key               | Values                          | Purpose                     | Default   |
//! |-------------------|---------------------------------|-----------------------------|-----------|
//! | `auth`            | string                          | Authentication token        | —         |
//! | `protocol`/`type` | udp, wechat-video, faketcp      | Protocol type               | —         |
//! | `obfs`            | string                          | Obfuscation type            | —         |
//! | `up_mbps`/`upmbps`| integer (u32)                   | Upload speed (Mbps)         | 100       |
//! | `down_mbps`/`downmbps`| integer (u32)               | Download speed (Mbps)       | 100       |
//! | `sni`             | domain                          | TLS SNI override            | hostname  |
//! | `insecure`        | 1/0, true/false                 | Skip TLS verification       | false     |
//!
//! # Edge Cases
//! - `insecure` accepts aliases: `insecure`, `allow_insecure`, `allowInsecure`
//! - `protocol` accepts alias `type`
//! - `up_mbps` accepts alias `upmbps`; `down_mbps` accepts alias `downmbps`
//! - Default `up_mbps/down_mbps` of 100 are not stored
//! - IPv6 addresses must be bracketed
//!
//! # References
//! - Hysteria v1: `thirdparty/hysteria/app/cmd/client.go`
//! - sing-box: `option/hysteria.go`
//! - mihomo: `adapter/outbound/hysteria.go`
//! - v2rayN: `HysteriaFmt.cs`

use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText};

use super::ProtoIdentity;
use super::common::{SecurityConfig, TlsConfig, TlsOpts, should_skip_endpoint_param};
use super::core_mapping;
use super::utils;
use super::{
    ConfigKind, EndpointEssentials, ParseError, ParsedProto, ProtoSpec, ProtocolConfig,
    ProtocolEssentials, ProtocolKind,
};
use crate::clash::{ClashHysteria1, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_tls_to_security, clash_to_endpoint, host_kind_for};

/// Hysteria v1 protocol configuration — the identity payload (sans host/port).
///
/// The endpoint (server host/port) lives in [`EndpointEssentials`] on the
/// [`ParsedProto`] boundary; this struct only carries endpoint-free protocol
/// parameters, so the same config pointed at different servers shares one
/// identity.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct Hysteria1Config {
    pub auth: Option<String>,
    pub protocol: Option<TinyText>,
    pub obfs: Option<TinyText>,
    pub up_mbps: Option<u32>,
    pub down_mbps: Option<u32>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl Hysteria1Config {
    /// Parse a Hysteria1 URL into the parse boundary: [`ParsedProto`] with the
    /// endpoint essentials (host/port) split out and the identity payload
    /// ([`ProtocolEssentials::config`]) holding only endpoint-free protocol
    /// parameters.
    ///
    /// Auth token is in userinfo (optional). Port defaults to 443 when absent.
    /// TLS is always on. `insecure` accepts 3 alias variants for compatibility.
    pub fn try_parse_proto(raw: &RawUrlX<'_>) -> Result<ParsedProto, ParseError> {
        let userinfo = raw.userinfo;
        let (auth, hostport) = if let Some(hostport) = raw.hostport {
            // When hostport is present, userinfo equals hostport if no '@' was
            // in the URL (no auth), otherwise userinfo is the auth token.
            let auth = (userinfo != hostport).then(|| userinfo.to_string());
            (auth, hostport)
        } else {
            let (auth_str, hostport) = userinfo.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{userinfo}: missing hostport").into())
            })?;
            (Some(auth_str.to_string()), hostport)
        };

        // Port defaults to 443 when not specified in the URL
        let (parsed_host, parsed_port) = if let Ok((h, p)) = utils::parse_hostport(hostport)
            && let Some(port) = p.first()
        {
            (h, port)
        } else {
            // No port in hostport — parse as bare host, default port to 443
            let host = utils::parse_host(hostport)?;
            (host, 443)
        };

        // Endpoint essentials: host/port live here, never in the config payload.
        let mut endpoint = EndpointEssentials::new(parsed_host.to_str().into_owned(), parsed_port);
        endpoint.host_type = host_kind_for(&parsed_host);

        let query = utils::parse_query(raw.query);

        // protocol: "protocol" or "type" (udp, wechat-video, faketcp)
        let protocol = utils::query_get_multi(&query, &["protocol", "type"]).map(TinyText::from);

        // obfs: obfuscation type (e.g., xplus, salamander)
        let obfs = utils::query_get(&query, "obfs").map(TinyText::from);

        // up_mbps / down_mbps: parse u32, default 100, don't store if default
        let up_mbps = utils::query_get_multi(&query, &["up_mbps", "upmbps"])
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|&v| v != 100);
        let down_mbps = utils::query_get_multi(&query, &["down_mbps", "downmbps"])
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|&v| v != 100);

        // Always TLS
        let security = SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                pin_sha256: None,
                sni: utils::query_get(&query, "sni").map(TinyText::from),
                alpn: None,
                fp: None,
                insecure: utils::query_get_multi(
                    &query,
                    &["insecure", "allow_insecure", "allowInsecure"],
                )
                .and_then(|v| match v {
                    "1" | "true" | "yes" => Some(true),
                    "0" | "false" | "no" => Some(false),
                    _ => None,
                }),
                ..Default::default()
            })),
            enc: None,
        };

        let remarks = utils::decode_fragment(raw)?;

        let config = Self {
            auth,
            protocol,
            obfs,
            up_mbps,
            down_mbps,
            security,
            remarks,
        };
        Ok(ParsedProto {
            endpoints: vec![endpoint],
            protocol: ProtocolEssentials {
                proto_kind: ProtocolKind::Hysteria,
                config_type: ConfigKind::ShareUrl,
                core_type: core_mapping::resolve_core(ProtocolKind::Hysteria, None, None),
                config: ProtocolConfig::Hysteria1(config),
            },
        })
    }

    /// Rebuild the share URL from this endpoint-free config plus the endpoint
    /// essentials. Endpoint host/port come from `endpoint`.
    pub fn reconstruct_proto(&self, endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        let endpoint_host = endpoint.host.as_str();
        let hostport = if endpoint_host.contains(':') {
            format!("[{endpoint_host}]:{}", endpoint.port)
        } else {
            format!("{endpoint_host}:{}", endpoint.port)
        };

        let mut base = self.auth.as_ref().map_or_else(
            || format!("hysteria://{hostport}"),
            |a| format!("hysteria://{}@{}", urlencoding::encode(a), hostport),
        );

        let query_string = {
            let mut parts: Vec<String> = Vec::new();
            if let Some(v) = &self.protocol {
                parts.push(format!("protocol={}", urlencoding::encode(v)));
            }
            if let Some(v) = &self.obfs {
                parts.push(format!("obfs={}", urlencoding::encode(v)));
            }
            if let Some(v) = self.up_mbps {
                parts.push(format!("up_mbps={v}"));
            }
            if let Some(v) = self.down_mbps {
                parts.push(format!("down_mbps={v}"));
            }
            // Security config (always TLS for Hysteria)
            if let Some(v) = self.security.insecure() {
                parts.push(format!("insecure={}", if v { "1" } else { "0" }));
            }
            if let Some(v) = self.security.sni()
                && !should_skip_endpoint_param(endpoint_host, v)
            {
                parts.push(format!("sni={}", urlencoding::encode(v)));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!("?{}", parts.join("&"))
            }
        };
        base.push_str(&query_string);

        if let Some(remarks) = &self.remarks {
            let frag = urlencoding::decode(remarks).unwrap_or(std::borrow::Cow::Borrowed(remarks));
            let frag = frag.trim();
            if !frag.is_empty() {
                _ = write!(base, "#{}", urlencoding::encode(frag));
            }
        }

        Ok(base)
    }
}

impl Hysteria1Config {
    /// Serialize this endpoint-free config plus the endpoint to a Clash proxy
    /// entry. Endpoint host/port are taken from `endpoint`.
    pub fn to_clash_proto(
        &self,
        endpoint: &EndpointEssentials,
    ) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let alpn_str = self.security.alpn();
        Ok(ClashProxy::Hysteria(ClashHysteria1 {
            name,
            server: endpoint.host.clone(),
            port: endpoint.port,
            auth_str: self.auth.clone().unwrap_or_default(),
            ports: None,
            obfs: self.obfs.as_ref().map(std::string::ToString::to_string),
            protocol: self.protocol.as_ref().map(std::string::ToString::to_string),
            up: self.up_mbps.map(|v| v.to_string()),
            down: self.down_mbps.map(|v| v.to_string()),
            alpn: alpn_str.map(|s| vec![s.to_string()]),
            servername: self.security.sni().map(std::string::ToString::to_string),
            skip_cert_verify: self.security.insecure(),
        }))
    }

    /// Parse a Clash proxy entry into the parse boundary: `server`/`port`
    /// become the endpoint essentials; the config payload is endpoint-free.
    pub fn try_from_clash_proto(proxy: &ClashProxy) -> Result<ParsedProto, ParseError> {
        match proxy {
            ClashProxy::Hysteria(c) => {
                let config = Self {
                    auth: match c.auth_str.as_str() {
                        "" => None,
                        s => Some(s.to_string()),
                    },
                    protocol: c.protocol.clone().map(TinyText::from),
                    obfs: c.obfs.clone().map(TinyText::from),
                    up_mbps: c
                        .up
                        .as_ref()
                        .and_then(|v| v.parse().ok())
                        .filter(|&v| v != 100),
                    down_mbps: c
                        .down
                        .as_ref()
                        .and_then(|v| v.parse().ok())
                        .filter(|&v| v != 100),
                    security: clash_tls_to_security(
                        Some(true),
                        c.servername.as_deref(),
                        c.skip_cert_verify,
                        c.alpn
                            .as_deref()
                            .and_then(|v| v.first().map(std::string::String::as_str)),
                        None,
                        None,
                    ),
                    remarks: match c.name.as_str() {
                        "" => None,
                        s => Some(TinyText::from(s)),
                    },
                };
                Ok(ParsedProto {
                    endpoints: vec![clash_to_endpoint(&c.server, c.port)],
                    protocol: ProtocolEssentials {
                        proto_kind: ProtocolKind::Hysteria,
                        config_type: ConfigKind::ShareUrl,
                        core_type: core_mapping::resolve_core(ProtocolKind::Hysteria, None, None),
                        config: ProtocolConfig::Hysteria1(config),
                    },
                })
            }
            _ => Err(ParseError::Unknown("expected hysteria1 clash proxy".into())),
        }
    }
}

/// Legacy [`ProtoSpec`] bridge — kept so `ProtocolConfig` dispatch (and the
/// `Proto`/`ParseResult` consumers in xray-tui-config) compile unchanged.
///
/// DEGRADED PATH (documented): `try_parse`/`try_from_clash` still work by
/// delegating to the `*_proto` variants and discarding the parsed endpoints;
/// `to_clash`/`reconstruct` return errors because the config no longer stores
/// host/port. Import/export rewires to the `*_proto` variants in T11 (phase D
/// builders take the endpoint separately).
impl ProtoSpec for Hysteria1Config {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let parsed = Self::try_parse_proto(raw)?;
        match parsed.protocol.config {
            ProtocolConfig::Hysteria1(config) => Ok(config),
            // Parser invariant: a hysteria URL always yields a Hysteria1Config.
            _ => Err(ParseError::Unknown(
                "hysteria URL parsed to a non-hysteria1 config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::reconstruct_proto`] with the endpoint.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::InvalidHost(
            "hysteria1 config no longer stores host/port; use Hysteria1Config::reconstruct_proto(endpoint)"
                .into(),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Hysteria
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
        Some("quic")
    }

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }

    /// # Errors
    ///
    /// If the Clash proxy doesn't match this protocol type.
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        let parsed = Self::try_from_clash_proto(proxy)?;
        match parsed.protocol.config {
            ProtocolConfig::Hysteria1(config) => Ok(config),
            _ => Err(ParseError::Unknown(
                "hysteria clash proxy parsed to a non-hysteria1 config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::to_clash_proto`] with the endpoint.
    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "hysteria1 config no longer stores host/port; use Hysteria1Config::to_clash_proto(endpoint)"
                .into(),
        ))
    }
}

impl ProtoIdentity for Hysteria1Config {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"hysteria");
        // Endpoint (host/port) intentionally absent from the identity — it
        // lives on the ParsedProto boundary, never in the config payload (T5).
        if let Some(v) = &self.up_mbps {
            hasher.write(&v.to_le_bytes());
        }
        if let Some(v) = &self.down_mbps {
            hasher.write(&v.to_le_bytes());
        }
        if let Some(v) = &self.protocol {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.obfs {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.security.insecure() {
            hasher.write(if v { b"true" } else { b"false" });
        }
        if let Some(v) = self.security.sni() {
            hasher.write(v.as_bytes());
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[("auth", self.auth.as_deref().unwrap_or(""))])
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ConfigKind, CoreType, HostKind, ParsedProto, ProtoSpec, ProtocolConfig, ProtocolKind,
    };
    use super::Hysteria1Config;
    use crate::urlx::{RawUrlX, SchemeX};

    fn parse(url: &str) -> ParsedProto {
        Hysteria1Config::try_parse_proto(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    fn config(parsed: ParsedProto) -> Hysteria1Config {
        match parsed.protocol.config {
            ProtocolConfig::Hysteria1(c) => c,
            other => panic!("expected Hysteria1Config, got {other:?}"),
        }
    }

    /// The identity payload must be endpoint-free: no top-level `host`/`port`
    /// keys in the serialized config.
    fn assert_no_top_level_host_port(cfg: &Hysteria1Config) {
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
    fn test_hysteria1_basic() {
        let url = "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com";
        let parsed = parse(url);
        assert_eq!(parsed.endpoints.len(), 1);
        let ep = &parsed.endpoints[0];
        assert_eq!(ep.host, "example.com");
        assert_eq!(ep.host_type, HostKind::Dns);
        assert_eq!(ep.port, 443);

        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Hysteria);
        assert_eq!(parsed.protocol.config_type, ConfigKind::ShareUrl);
        assert_eq!(parsed.protocol.core_type, CoreType::SingBox);
        let cfg = config(parsed);
        assert_eq!(cfg.protocol.as_deref(), Some("udp"));
        assert_eq!(cfg.obfs.as_deref(), Some("xplus"));
        assert_eq!(cfg.up_mbps, Some(200));
        assert_eq!(cfg.down_mbps, Some(200));
        assert_eq!(cfg.security.insecure(), Some(true));
        assert_eq!(cfg.security.sni(), Some("real.example.com"));
        assert!(cfg.auth.is_none());
        assert_no_top_level_host_port(&cfg);
    }

    #[test]
    fn test_hy_scheme_with_auth() {
        let url = "hy://auth123@server.example.com:8443?protocol=faketcp&up_mbps=50&insecure=0";
        let parsed = parse(url);
        assert_eq!(parsed.endpoints[0].host, "server.example.com");
        assert_eq!(parsed.endpoints[0].port, 8443);
        let cfg = config(parsed);
        assert_eq!(cfg.auth.as_deref(), Some("auth123"));
        assert_eq!(cfg.protocol.as_deref(), Some("faketcp"));
        assert_eq!(cfg.up_mbps, Some(50));
        assert!(cfg.down_mbps.is_none());
        assert_eq!(cfg.security.insecure(), Some(false));
    }

    #[test]
    fn test_hysteria1_default_port() {
        // No explicit port defaults to 443
        let parsed = parse("hysteria://example.com?protocol=udp");
        assert_eq!(parsed.endpoints[0].port, 443);
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Dns);
    }

    #[test]
    fn test_hysteria1_ipv6() {
        let parsed = parse("hysteria://[2001:db8::1]:443?protocol=udp");
        assert_eq!(parsed.endpoints[0].host, "2001:db8::1");
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Ipv6);
        assert_eq!(parsed.endpoints[0].port, 443);
    }

    // ── Identity: endpoint-free uid ───────────────────────────────────────

    #[test]
    fn uid_identical_across_servers_different_across_auth() {
        let url_a = "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&insecure=1&sni=real.example.com";
        let url_b = "hysteria://other.example.com:8443?protocol=udp&obfs=xplus&up_mbps=200&insecure=1&sni=real.example.com";
        let url_c = "hysteria://tok@example.com:443?protocol=udp&obfs=xplus&up_mbps=200&insecure=1&sni=real.example.com";
        let a = parse(url_a);
        let b = parse(url_b);
        let c = parse(url_c);
        assert_eq!(
            a.uid(),
            b.uid(),
            "same protocol on different servers must dedup to one uid"
        );
        assert_ne!(a.uid(), c.uid(), "different auth -> different uid");
        assert_ne!(a.sig(), 0);
    }

    // ── Reconstruct round-trip via endpoint ───────────────────────────────

    #[test]
    fn reconstruct_roundtrip_via_endpoint() {
        assert_reconstruct_roundtrip(
            "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com",
        );
        assert_reconstruct_roundtrip(
            "hy://auth123@server.example.com:8443?protocol=faketcp&up_mbps=50&insecure=0",
        );
        assert_reconstruct_roundtrip("hysteria://example.com:443?protocol=udp#My%20Server");
        assert_reconstruct_roundtrip("hysteria://[2001:db8::1]:443?protocol=udp");
    }

    // ── Clash round-trip via *_proto ──────────────────────────────────────

    #[test]
    fn clash_roundtrip_from_url_via_proto() {
        let url = "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com";
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        let proxy = cfg.to_clash_proto(&endpoint).expect("to clash");
        let reparsed = Hysteria1Config::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(
            reparsed.endpoints[0], endpoint,
            "endpoint round-trips through clash"
        );
        assert_eq!(
            reparsed.protocol.config,
            ProtocolConfig::Hysteria1(cfg),
            "config round-trips through clash"
        );
    }

    #[test]
    fn clash_proxy_roundtrip_via_proto() {
        use crate::clash::{ClashHysteria1, ClashProxy};

        let proxy = ClashProxy::Hysteria(ClashHysteria1 {
            name: "test".into(),
            server: "example.com".into(),
            port: 443,
            auth_str: "auth123".into(),
            ports: None,
            obfs: Some("xplus".into()),
            protocol: Some("udp".into()),
            up: Some("200".into()),
            down: Some("200".into()),
            alpn: None,
            servername: Some("real.example.com".into()),
            skip_cert_verify: Some(true),
        });
        let parsed = Hysteria1Config::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(parsed.endpoints[0].host, "example.com");
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Dns);
        assert_eq!(parsed.endpoints[0].port, 443);
        let cfg = match &parsed.protocol.config {
            ProtocolConfig::Hysteria1(c) => c,
            other => panic!("expected Hysteria1Config, got {other:?}"),
        };
        assert_eq!(cfg.auth.as_deref(), Some("auth123"));
        assert_eq!(cfg.up_mbps, Some(200));
        assert_eq!(cfg.security.insecure(), Some(true));
        assert_eq!(cfg.security.sni(), Some("real.example.com"));
        assert_no_top_level_host_port(cfg);
        let out = cfg.to_clash_proto(&parsed.endpoints[0]).expect("to clash");
        match (out, proxy) {
            (ClashProxy::Hysteria(out), ClashProxy::Hysteria(orig)) => assert_eq!(out, orig),
            _ => panic!("expected hysteria1 clash proxy"),
        }
    }

    // ── Serde ─────────────────────────────────────────────────────────────

    #[test]
    fn test_serde_roundtrip() {
        let cfg = config(parse(
            "hysteria://example.com:443?protocol=udp&up_mbps=200&insecure=1&sni=real.example.com",
        ));
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: Hysteria1Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, deserialized);
        assert_no_top_level_host_port(&deserialized);
    }

    // ── Legacy trait bridge ───────────────────────────────────────────────

    #[test]
    fn legacy_bridge_parse_works_but_reconstruct_to_clash_error() {
        let url = "hysteria://example.com:443?protocol=udp";
        let bridged = Hysteria1Config::try_parse(&RawUrlX::from(url)).expect("bridged parse");
        assert_eq!(bridged.schema(), SchemeX::Hysteria);
        assert_eq!(bridged.protocol.as_deref(), Some("udp"));
        // host/port accessors are gone — the endpoint lives on ParsedProto.
        assert_eq!(bridged.host(), None);
        assert_eq!(bridged.port(), None);
        // Degraded legacy paths error instead of fabricating a host.
        assert!(bridged.reconstruct().is_err());
        assert!(bridged.to_clash().is_err());
    }
}
