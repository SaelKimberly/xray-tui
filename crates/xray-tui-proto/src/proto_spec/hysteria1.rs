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

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::{SecurityConfig, TlsConfig, TlsOpts, should_skip_param};
use super::ProtoIdentity;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashHysteria1, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_server_to_host, clash_tls_to_security, host_spec_to_string};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct Hysteria1Config {
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub auth: Option<String>,
    pub protocol: Option<TinyText>,
    pub obfs: Option<TinyText>,
    pub up_mbps: Option<u32>,
    pub down_mbps: Option<u32>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for Hysteria1Config {
    /// Parse a Hysteria1 URL.
    ///
    /// Auth token is in userinfo (optional). Port defaults to 443 when absent.
    /// TLS is always on. `insecure` accepts 3 alias variants for compatibility.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let (auth, hostport) = if let Some(hostport) = raw.hostport {
            // When hostport is present, userinfo equals hostport if no '@' was
            // in the URL (no auth), otherwise userinfo is the auth token.
            let userinfo = raw.userinfo;
            let auth = (userinfo != hostport).then(|| userinfo.to_string());
            (auth, hostport)
        } else {
            let userinfo = raw.userinfo;
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
            })),
            enc: None,
        };

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            host: parsed_host,
            port: parsed_port,
            auth,
            protocol,
            obfs,
            up_mbps,
            down_mbps,
            security,
            remarks,
        })
    }

    fn reconstruct(&self) -> Result<String, ParseError> {
        let host = self.host.to_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", self.port)
        } else {
            format!("{host}:{}", self.port)
        };

        let mut base = match &self.auth {
            Some(a) => format!("hysteria://{}@{}", urlencoding::encode(a), hostport),
            None => format!("hysteria://{hostport}"),
        };

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
                && !should_skip_param(&self.host, v)
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

    fn schema(&self) -> SchemeX {
        SchemeX::Hysteria
    }

    fn host(&self) -> Option<&HostSpec> {
        Some(&self.host)
    }

    fn port(&self) -> Option<u16> {
        Some(self.port)
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

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Hysteria(c) => Ok(Self {
                host: clash_server_to_host(&c.server)?,
                port: c.port,
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
            }),
            _ => Err(ParseError::Unknown("expected hysteria1 clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let alpn_str = self.security.alpn();
        Ok(ClashProxy::Hysteria(ClashHysteria1 {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
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
}

impl ProtoIdentity for Hysteria1Config {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"hysteria");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(self.port.to_string().as_bytes());
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
    use super::super::ProtoSpec;
    use super::Hysteria1Config;
    use crate::urlx::PortSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_hysteria1_basic() {
        let url = "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.schema(), SchemeX::Hysteria);
        assert_eq!(config.host.to_str(), "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.protocol.as_deref(), Some("udp"));
        assert_eq!(config.obfs.as_deref(), Some("xplus"));
        assert_eq!(config.up_mbps, Some(200));
        assert_eq!(config.down_mbps, Some(200));
        assert_eq!(config.security.insecure(), Some(true));
        assert_eq!(config.security.sni(), Some("real.example.com"));
        assert!(config.auth.is_none());
    }

    #[test]
    fn test_hy_scheme() {
        let url = "hy://auth123@server.example.com:8443?protocol=faketcp&up_mbps=50&insecure=0";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.schema(), SchemeX::Hysteria);
        assert_eq!(config.host.to_str(), "server.example.com");
        assert_eq!(config.port, 8443);
        assert_eq!(config.auth.as_deref(), Some("auth123"));
        assert_eq!(config.protocol.as_deref(), Some("faketcp"));
        assert_eq!(config.up_mbps, Some(50));
        assert!(config.down_mbps.is_none());
        assert_eq!(config.security.insecure(), Some(false));
    }

    #[test]
    fn test_hysteria1_ipv6() {
        let url = "hysteria://[2001:db8::1]:443?protocol=udp";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.host.to_str(), "2001:db8::1");
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_hysteria1_default_port() {
        // No explicit port defaults to 443
        let url = "hysteria://example.com?protocol=udp";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_hysteria1_auth() {
        let url = "hysteria://mytoken@example.com:443?protocol=udp&up_mbps=200";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.auth.as_deref(), Some("mytoken"));
    }

    #[test]
    fn test_hysteria1_with_remark() {
        let url = "hysteria://example.com:443?protocol=udp#My%20Server";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.remarks(), Some("My Server"));
    }

    #[test]
    fn test_hysteria1_upmbps_alias() {
        let url = "hy://example.com:443?upmbps=150&downmbps=300";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.up_mbps, Some(150));
        assert_eq!(config.down_mbps, Some(300));
    }

    #[test]
    fn test_hysteria1_type_alias() {
        let url = "hysteria://example.com:443?type=wechat-video";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.protocol.as_deref(), Some("wechat-video"));
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = Hysteria1Config::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.protocol, reparsed.protocol, "protocol mismatch");
        assert_eq!(parsed.up_mbps, reparsed.up_mbps, "up_mbps mismatch");
        assert_eq!(parsed.down_mbps, reparsed.down_mbps, "down_mbps mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input =
            "hysteria://example.com:443?protocol=udp&up_mbps=200&insecure=1&sni=real.example.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = Hysteria1Config::try_parse(&raw).expect("failed to parse");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: Hysteria1Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host, "host mismatch");
        assert_eq!(parsed.port, deserialized.port, "port mismatch");
        assert_eq!(parsed.protocol, deserialized.protocol, "protocol mismatch");
    }

    use super::super::test_helpers::check_roundtrip;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<Hysteria1Config>(
            "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com",
        );
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<Hysteria1Config>(
            "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com",
        );
    }
}
