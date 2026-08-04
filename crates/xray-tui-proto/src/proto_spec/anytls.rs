//! `AnyTLS` (`anytls://`) URL parsing.
//!
//! # Format
//! ```text
//! anytls://<host>:<port>?password=<password>&sni=<sni>#<remarks>
//! ```
//!
//! Password is in query params (not userinfo). TLS is always used.
//! Userinfo portion is always ignored — no auth in userinfo.
//!
//! # Query Parameters
//!
//! | Key              | Values                                       | Purpose                     | Default   |
//! |------------------|----------------------------------------------|-----------------------------|-----------|
//! | `password`       | string                                       | Authentication password     | —         |
//! | `auth`           | string                                       | Password alias              | —         |
//! | `sni`            | domain                                       | TLS SNI override            | hostname  |
//! | `alpn`           | comma-separated (h2,http/1.1)                | ALPN list                   | —         |
//! | `insecure`       | 1/0, true/false                              | Skip TLS cert verification  | `"0"`     |
//! | `allow_insecure` | 1/0, true/false                              | Skip TLS cert verification  | `"0"`     |
//!
//! # Edge Cases
//! - Userinfo is always ignored (no auth in userinfo)
//! - Password is optional (`Option<String>`)
//! - TLS is always on (no `security=none`)
//! - No transport config (raw TCP only)
//!
//! # References
//! - sing-box: `option/anytls.go` — `AnyTLSOutboundOptions`
//! - mihomo: `adapter/outbound/anytls.go` — `AnyTlsOption`
//! - subconverter: `subparser.cpp` `explodeAnyTLS()`

use serde::{Deserialize, Serialize};

use super::utils;
use super::{ParseError, ProtoSpec, ProtoIdentity};
use crate::clash::{ClashAnyTls, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct AnyTlsConfig {
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for AnyTlsConfig {
    /// Parse an `AnyTLS` URL.
    ///
    /// Userinfo is ignored (no auth in userinfo). Password comes from query params.
    /// TLS is always used, configured via sni/alpn/insecure query params.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let hostport = raw
            .hostport
            .ok_or_else(|| ParseError::InvalidUserInfo("anytls: missing hostport".into()))?;

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        let query = utils::parse_query(raw.query);

        // Password from query: password or auth alias
        let password = utils::query_get_multi(&query, &["password", "auth"]).map(str::to_string);

        // TLS config (AnyTLS always uses TLS)
        let insecure = utils::query_get_multi(
            &query,
            &["insecure", "allow_insecure", "allowinsecure", "skipVerify"],
        )
        .and_then(|v| match v {
            "1" | "true" | "True" => Some(true),
            "0" | "false" | "False" => Some(false),
            _ => None,
        });

        let sni = utils::query_get(&query, "sni").map(TinyText::from);
        let alpn = utils::query_get(&query, "alpn").map(TinyText::from);

        let security = if sni.is_some() || alpn.is_some() || insecure.is_some() {
            SecurityConfig {
                tls: Some(TlsConfig::Tls(TlsOpts {
                    pin_sha256: None,
                    sni,
                    alpn,
                    fp: None,
                    insecure,
                })),
                enc: None,
            }
        } else {
            SecurityConfig::default()
        };

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            host: parsed_host,
            port: parsed_port,
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

        let mut parts: Vec<String> = Vec::new();
        if let Some(ref password) = self.password {
            parts.push(format!("password={}", urlencoding::encode(password)));
        }
        if let Some(TlsConfig::Tls(opts)) = &self.security.tls {
            if let Some(ref sni) = opts.sni
                && !should_skip_param(&self.host, sni)
            {
                parts.push(format!("sni={}", urlencoding::encode(sni)));
            }
            if let Some(ref alpn) = opts.alpn {
                parts.push(format!("alpn={}", urlencoding::encode(alpn)));
            }
            if opts.insecure == Some(true) {
                parts.push("insecure=1".to_string());
            }
        }

        let query_string = if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        };

        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();

        Ok(format!("anytls://{hostport}{query_string}{fragment}"))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::AnyTLS
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
        None
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Anytls(c) => Ok(Self {
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                password: Some(c.password.clone()),
                security: SecurityConfig::default(),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
            }),
            _ => Err(ParseError::Unknown("expected anytls clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Anytls(ClashAnyTls {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            password: self.password.clone().unwrap_or_default(),
            tls: Some(true),
            servername: self.security.sni().map(std::string::ToString::to_string),
            skip_cert_verify: self.security.insecure(),
            idle_session_check_interval: None,
            idle_session_timeout: None,
            min_idle_session: None,
        }))
    }
}

impl ProtoIdentity for AnyTlsConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"anytls");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_be_bytes());
        if let Some(sni) = self.security.sni() {
            hasher.write(sni.as_bytes());
        }
        if self.security.insecure() == Some(true) {
            hasher.write(b"insecure");
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[("password", self.password.as_deref().unwrap_or(""))])
    }
}

use crate::proto_spec::common::{
    SecurityConfig, TlsConfig, TlsOpts, clash_server_to_host, host_spec_to_string,
    should_skip_param,
};

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_anytls_basic() {
        let url = "anytls://1.2.3.4:8080?password=secret";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = AnyTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::AnyTLS);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 8080);
        assert_eq!(config.password.as_deref(), Some("secret"));
    }

    #[test]
    fn test_anytls_no_auth() {
        let url = "anytls://1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = AnyTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 8080);
        assert!(config.password.is_none());
    }

    #[test]
    fn test_anytls_with_sni() {
        let url = "anytls://example.com:443?password=secret&sni=real-target.com&insecure=1";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = AnyTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.host.to_str(), "example.com");
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.security.sni(), Some("real-target.com"));
        assert_eq!(config.security.insecure(), Some(true));
    }

    #[test]
    fn test_anytls_auth_alias() {
        let url = "anytls://1.2.3.4:8080?auth=mysecret";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = AnyTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.password.as_deref(), Some("mysecret"));
    }

    #[test]
    fn test_anytls_serde_roundtrip() {
        let input = "anytls://1.2.3.4:8080?password=secret";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = AnyTlsConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: AnyTlsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.port, deserialized.port);
        assert_eq!(parsed.password, deserialized.password);
    }

    use super::super::test_helpers::check_roundtrip;
    use super::AnyTlsConfig;

    #[test]
    fn test_anytls_roundtrip() {
        check_roundtrip::<AnyTlsConfig>("anytls://1.2.3.4:8080?password=secret");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<AnyTlsConfig>("anytls://1.2.3.4:8080?password=secret");
    }

    #[test]
    fn test_anytls_roundtrip_no_auth() {
        check_roundtrip::<AnyTlsConfig>("anytls://1.2.3.4:8080");
    }

    #[test]
    fn test_anytls_roundtrip_with_remarks() {
        check_roundtrip::<AnyTlsConfig>(
            "anytls://1.2.3.4:8080?password=secret&sni=real.com&insecure=1#my-server",
        );
    }
}
