//! `ShadowTLS` (`shadowtls://`) URL parsing.
//!
//! # Format
//! ```text
//! shadowtls://<host>:<port>?password=xxx&version=1&sni=xxx#<remarks>
//! ```
//!
//! No userinfo — password is a query parameter.
//! The `sni` field populates TLS server name indication.
//!
//! # Query Parameters
//!
//! | Key       | Type     | Required | Default | Description          |
//! |-----------|----------|----------|---------|----------------------|
//! | `password`| `String` | ❌       | —       | `ShadowTLS` password   |
//! | `version` | `String` | ❌       | —       | Protocol version     |
//! | `sni`     | `String` | ❌       | —       | TLS SNI              |
//!
//! # References
//! - sing-box: `option/shadowtls.go` — `ShadowTLSOutboundOptions`
//! - `thirdparty/sing-box/docs/configuration/outbound/shadowtls.md`

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::{SecurityConfig, TlsConfig, TlsOpts, should_skip_param};
use super::ProtoIdentity;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashProxy, ClashShadowTls};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_server_to_host, clash_tls_to_security, host_spec_to_string, security_to_clash_tls,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct ShadowTlsConfig {
    pub password: Option<String>,
    pub version: Option<TinyText>,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for ShadowTlsConfig {
    /// Parse a `ShadowTLS` URL.
    ///
    /// `ShadowTLS` uses standard `host:port` with config in query params.
    /// No userinfo — password is extracted from query.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let hostport = raw.hostport.ok_or_else(|| {
            ParseError::InvalidHostPort("missing hostport in shadowtls URL".into())
        })?;

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        let query = utils::parse_query(raw.query);
        let remarks = utils::decode_fragment(raw)?;

        let password = utils::query_get(&query, "password").map(str::to_string);
        let version = utils::query_get(&query, "version").map(TinyText::from);

        // Build security config with SNI if present
        let security = if let Some(sni) = utils::query_get(&query, "sni") {
            SecurityConfig {
                tls: Some(TlsConfig::Tls(TlsOpts {
                    pin_sha256: None,
                    sni: Some(TinyText::from(sni)),
                    ..Default::default()
                })),
                enc: None,
            }
        } else {
            SecurityConfig::default()
        };

        Ok(Self {
            password,
            version,
            host: parsed_host,
            port: parsed_port,
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

        let mut query_parts: Vec<String> = Vec::new();
        if let Some(ref password) = self.password {
            query_parts.push(format!("password={}", urlencoding::encode(password)));
        }
        if let Some(ref version) = self.version {
            query_parts.push(format!("version={}", urlencoding::encode(version)));
        }
        // Emit sni when it differs from host or is explicitly set
        if let Some(sni) = self.security.sni()
            && !should_skip_param(&self.host, sni)
        {
            query_parts.push(format!("sni={}", urlencoding::encode(sni)));
        }

        let query_string = if query_parts.is_empty() {
            String::new()
        } else {
            format!("?{}", query_parts.join("&"))
        };

        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();

        Ok(format!("shadowtls://{hostport}{query_string}{fragment}"))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::ShadowTls
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

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Shadowtls(c) => Ok(Self {
                password: Some(c.password.clone()),
                version: c.version.map(|v| TinyText::from(v.to_string())),
                host: clash_server_to_host(&c.server)?,
                port: c.port,
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
            _ => Err(ParseError::Unknown("expected shadowtls clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let (tls, servername, skip_cert_verify, _alpn, _fingerprint) =
            security_to_clash_tls(&self.security);
        Ok(ClashProxy::Shadowtls(ClashShadowTls {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            password: self.password.clone().unwrap_or_default(),
            version: self.version.as_ref().and_then(|v| v.parse::<u32>().ok()),
            tls,
            servername,
            skip_cert_verify,
        }))
    }
}

impl ProtoIdentity for ShadowTlsConfig {
    /// Compute a deterministic signature based on non-credential fields:
    /// protocol tag + host + port + version + sni.
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"shadowtls");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        if let Some(ref version) = self.version {
            hasher.write(version.as_bytes());
        }
        if let Some(sni) = self.security.sni() {
            hasher.write(sni.as_bytes());
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[("password", self.password.as_deref().unwrap_or(""))])
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::PortSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_shadowtls_basic() {
        let url = "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::ShadowTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::ShadowTls);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 443);
        assert_eq!(config.password.as_deref(), Some("pass123"));
        assert_eq!(config.version.as_deref(), Some("1"));
        assert_eq!(config.security.sni(), Some("example.com"));
    }

    #[test]
    fn test_shadowtls_no_auth() {
        let url = "shadowtls://1.2.3.4:443";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = super::ShadowTlsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.password, None);
        assert_eq!(config.version, None);
        assert!(config.security.tls.is_none());
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = super::ShadowTlsConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = super::ShadowTlsConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.password, reparsed.password, "password mismatch");
    }

    #[test]
    fn test_shadowtls_serde_roundtrip() {
        let input = "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = super::ShadowTlsConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: super::ShadowTlsConfig =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.host, deserialized.host, "host mismatch");
        assert_eq!(parsed.port, deserialized.port, "port mismatch");
        assert_eq!(parsed.password, deserialized.password, "password mismatch");
    }

    use super::super::test_helpers::check_roundtrip;
    use super::ShadowTlsConfig;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<ShadowTlsConfig>(
            "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com",
        );
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<ShadowTlsConfig>(
            "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com",
        );
    }
}
