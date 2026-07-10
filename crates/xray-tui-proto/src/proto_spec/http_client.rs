//! HTTP Client (`http://`) URL parsing.
//!
//! # Format
//! ```text
//! http://[user:pass@]<host>:<port>[#remarks]
//! ```
//!
//! # Query Parameters
//!
//! | Key       | Values                   | Purpose                     | Default   |
//! |-----------|--------------------------|-----------------------------|-----------|
//! | `security`| tls                      | Enable TLS (HTTPS)          | none      |
//! | `sni`     | domain                   | TLS SNI                     | hostname  |
//! | `alpn`    | comma-separated          | ALPN list                   | —         |
//! | `fp`      | chrome, firefox, safari  | uTLS fingerprint            | —         |
//!
//! # Edge Cases
//! - Userinfo is optional (plain HTTP proxy, no auth)
//! - Default port is 80 when omitted
//! - `security=tls` enables TLS (HTTPS) wrapping
//! - IPv6 addresses must be bracketed
//!
//! # References
//! - sing-box: `option/simple.go` — `HTTPOutboundOptions`
//! - mihomo: `adapter/outbound/http.go` — `HttpOption`
//! - Xray-core: `proxy/http/config.proto`
//! - subconverter: `subparser.cpp` `explodeHTTP()`

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::{SecurityConfig, TlsConfig, TlsOpts};
use super::impl_sig_cache;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashHttp, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_server_to_host, clash_tls_to_security, host_spec_to_string, security_to_clash_tls,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct HttpClientConfig {
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<NonZeroU64>,
    #[serde(skip)]
    cred_hash_cache: std::sync::OnceLock<NonZeroU64>,

    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for HttpClientConfig {
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        // Userinfo is optional for HTTP.
        // When present (URL has `@`): raw.userinfo = "user:pass", raw.hostport = "host:port"
        // When absent (no `@`): raw.userinfo == raw.hostport (both are "host:port")
        let has_userinfo = raw.hostport.is_some_and(|hp| raw.userinfo != hp);
        let (username, password) = if has_userinfo && !raw.userinfo.is_empty() {
            if let Some((u, p)) = raw.userinfo.split_once(':') {
                (Some(u.to_string()), Some(p.to_string()))
            } else {
                (Some(raw.userinfo.to_string()), None)
            }
        } else {
            (None, None)
        };

        let hostport = raw
            .hostport
            .ok_or_else(|| ParseError::InvalidHostPort("missing hostport in http URL".into()))?;

        let (parsed_host, parsed_port) = if hostport.contains(':') {
            let (h, p) = utils::parse_hostport(hostport)?;
            (h, p.first().unwrap_or(80))
        } else {
            // No port in URL — parse as host only, default to HTTP port
            let host = utils::parse_host(hostport)?;
            (host, 80)
        };

        let query = utils::parse_query(raw.query);

        // Security mode: tls (HTTPS) or none (default plain HTTP)
        let security = match utils::query_get(&query, "security") {
            Some("tls") => {
                let insecure = utils::query_get_multi(
                    &query,
                    &[
                        "allowInsecure",
                        "allow_insecure",
                        "allowinsecure",
                        "skipVerify",
                    ],
                )
                .and_then(|v| match v {
                    "1" | "true" | "True" => Some(true),
                    "0" | "false" | "False" => Some(false),
                    _ => None,
                });
                SecurityConfig {
                    tls: Some(TlsConfig::Tls(TlsOpts {
                        pin_sha256: None,
                        sni: utils::query_get(&query, "sni").map(TinyText::from),
                        alpn: utils::query_get(&query, "alpn").map(TinyText::from),
                        fp: utils::query_get(&query, "fp").map(TinyText::from),
                        insecure,
                    })),
                    enc: None,
                }
            }
            _ => SecurityConfig::default(),
        };

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            sig_cache: std::sync::OnceLock::new(),
            cred_hash_cache: std::sync::OnceLock::new(),
            host: parsed_host,
            port: parsed_port,
            username,
            password,
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

        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("{u}:{p}@"),
            (Some(u), None) => format!("{u}@"),
            _ => String::new(),
        };

        let query_string = {
            let mut parts: Vec<String> = Vec::new();
            if let Some(tls_config) = &self.security.tls
                && let TlsConfig::Tls(opts) = tls_config
            {
                if opts.sni.is_some() || opts.alpn.is_some() || opts.fp.is_some() {
                    parts.push("security=tls".to_string());
                }
                if let Some(v) = &opts.sni
                    && !super::common::should_skip_param(&self.host, v)
                {
                    parts.push(format!("sni={}", urlencoding::encode(v)));
                }
                if let Some(v) = &opts.alpn {
                    parts.push(format!("alpn={}", urlencoding::encode(v)));
                }
                if let Some(v) = &opts.fp {
                    parts.push(format!("fp={}", urlencoding::encode(v)));
                }
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!("?{}", parts.join("&"))
            }
        };

        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();

        Ok(format!("http://{auth}{hostport}{query_string}{fragment}"))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Http
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

    fn cred_hash(&self) -> u64 {
        let v = self.cred_hash_cache.get_or_init(|| {
            let val = utils::compute_cred_hash(
                Some(&self.host),
                Some(self.port),
                None,
                self.username.as_deref().unwrap_or(""),
                self.password.as_deref().unwrap_or(""),
            );
            NonZeroU64::new(val).unwrap_or(NonZeroU64::MIN)
        });
        v.get()
    }

    fn set_cred_hash_cache(&self, v: NonZeroU64) {
        _ = self.cred_hash_cache.set(v);
    }

    impl_sig_cache!();

    fn transport_type(&self) -> Option<&str> {
        None
    }

    fn security(&self) -> Option<&SecurityConfig> {
        if self.security.is_empty() {
            None
        } else {
            Some(&self.security)
        }
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Http(c) => Ok(Self {
                sig_cache: std::sync::OnceLock::new(),
                cred_hash_cache: std::sync::OnceLock::new(),
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                username: c.username.clone(),
                password: c.password.clone(),
                security: clash_tls_to_security(
                    c.tls,
                    c.servername.as_deref(),
                    c.skip_cert_verify,
                    None,
                    None,
                    None,
                ),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
            }),
            _ => Err(ParseError::Unknown("expected http clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let (tls, servername, skip_cert_verify, _, _) = security_to_clash_tls(&self.security);
        Ok(ClashProxy::Http(ClashHttp {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
            tls,
            servername,
            skip_cert_verify,
            headers: None,
        }))
    }
}

impl HttpClientConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"http");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_http_basic() {
        let url = "http://user:pass@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::HttpClientConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Http);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 8080);
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_http_no_auth() {
        let url = "http://1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::HttpClientConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Http);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 8080);
        assert!(config.username.is_none());
        assert!(config.password.is_none());
    }

    #[test]
    fn test_http_default_port() {
        let url = "http://user:pass@1.2.3.4";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::HttpClientConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.port, 80);
    }

    #[test]
    fn test_http_username_only() {
        let url = "http://user@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::HttpClientConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.username.as_deref(), Some("user"));
        assert!(config.password.is_none());
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "http://user:pass@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = super::HttpClientConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = super::HttpClientConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.username, reparsed.username, "username mismatch");
        assert_eq!(parsed.password, reparsed.password, "password mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input = "http://user:pass@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = super::HttpClientConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: super::HttpClientConfig =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.username, deserialized.username);
        assert_eq!(parsed.password, deserialized.password);
    }

    use super::super::test_helpers::check_roundtrip;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<super::HttpClientConfig>("http://user:pass@1.2.3.4:8080");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<super::HttpClientConfig>("http://user:pass@1.2.3.4:8080");
    }

    #[test]
    fn test_http_with_remarks() {
        let url = "http://user:pass@example.com:80#my-server";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::HttpClientConfig::try_parse(&raw).unwrap();
        assert_eq!(config.remarks.as_deref(), Some("my-server"));
        let rebuilt = config.reconstruct().unwrap();
        assert!(
            rebuilt.contains("#my-server"),
            "reconstruct should preserve fragment: {rebuilt}"
        );
    }
}
