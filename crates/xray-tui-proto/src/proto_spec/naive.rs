//! Naïve (`naive+https://`) URL parsing.
//!
//! # Format
//! ```text
//! naive+https://<username>:<password>@<host>:<port>#<remarks>
//! ```
//!
//! Standard URI format. Username/password in userinfo, server in host:port,
//! remarks in fragment.
//!
//! # Fields
//!
//! | Component     | Source              | Purpose                         |
//! |---------------|---------------------|---------------------------------|
//! | `username`    | userinfo            | Basic auth username              |
//! | `password`    | userinfo            | Basic auth password              |
//! | `host`        | hostport            | Server address                   |
//! | `port`        | hostport            | Server port (default 443)        |
//! | `remarks`     | fragment (#)        | Display name (URL-decoded)       |
//!
//! # Upstream References
//!
//! - sing-box: `option/naive.go` — `NaiveOutboundOptions`
use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};
use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use super::common::{SecurityConfig, TlsConfig};
use super::impl_sig_cache;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashNaive, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_server_to_host, clash_tls_to_security, host_spec_to_string, security_to_clash_tls,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct NaiveConfig {
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<NonZeroU64>,
    #[serde(skip)]
    cred_hash_cache: std::sync::OnceLock<NonZeroU64>,

    pub username: String,
    pub password: String,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for NaiveConfig {
    /// Parse a Naïve URL.
    ///
    /// Format: `naive+https://<username>:<password>@<host>:<port>#<remarks>`
    /// Userinfo and hostport are always present.
    /// Default port is 443.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let hostport = raw.hostport.ok_or_else(|| {
            ParseError::InvalidHost(format!("missing host:port in {}", raw.raw).into())
        })?;

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        let (username, password) = raw.userinfo.split_once(':').ok_or_else(|| {
            ParseError::InvalidUserInfo(format!("{}: missing password", raw.userinfo).into())
        })?;

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            sig_cache: std::sync::OnceLock::new(),
            cred_hash_cache: std::sync::OnceLock::new(),
            username: username.to_string(),
            password: password.to_string(),
            host: parsed_host,
            port: parsed_port,
            security: SecurityConfig {
                tls: Some(TlsConfig::Tls(Default::default())),
                enc: None,
            },
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
        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();
        Ok(format!(
            "naive+https://{}:{}@{hostport}{fragment}",
            urlencoding::encode(&self.username),
            urlencoding::encode(&self.password),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Naive
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
                &self.username,
                &self.password,
            );
            NonZeroU64::new(val).unwrap_or(NonZeroU64::MIN)
        });
        v.get()
    }

    fn set_cred_hash_cache(&self, v: NonZeroU64) {
        _ = self.cred_hash_cache.set(v);
    }

    impl_sig_cache!();

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }

    fn transport_type(&self) -> Option<&str> {
        None
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Naive(c) => Ok(Self {
                sig_cache: std::sync::OnceLock::new(),
                cred_hash_cache: std::sync::OnceLock::new(),
                username: c.username.clone(),
                password: c.password.clone(),
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
            _ => Err(ParseError::Unknown("expected naive clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let (tls, servername, skip_cert_verify, _, _) = security_to_clash_tls(&self.security);
        Ok(ClashProxy::Naive(ClashNaive {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
            tls,
            servername,
            skip_cert_verify,
        }))
    }
}

impl NaiveConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"naive");
        hasher.write(self.host.to_str().as_bytes());
        let mut buf = itoa::Buffer::new();
        hasher.write(buf.format(self.port).as_bytes());
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_naive_basic() {
        let url = "naive+https://user:pass@example.com:443";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = NaiveConfig::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.schema(), SchemeX::Naive);
        assert_eq!(config.host.to_str(), "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.username, "user");
        assert_eq!(config.password, "pass");
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "naive+https://user:pass@example.com:443";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = NaiveConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = NaiveConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.username, reparsed.username, "username mismatch");
        assert_eq!(parsed.password, reparsed.password, "password mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input = "naive+https://user:pass@example.com:443";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = NaiveConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: NaiveConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.username, deserialized.username);
        assert_eq!(parsed.password, deserialized.password);
    }

    use super::super::test_helpers::check_roundtrip;
    use super::NaiveConfig;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<NaiveConfig>("naive+https://user:pass@example.com:443");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<NaiveConfig>("naive+https://user:pass@example.com:443");
    }

    #[test]
    fn test_with_remarks() {
        let url = "naive+https://user:pass@example.com:443#my-server";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = NaiveConfig::try_parse(&raw).unwrap();
        assert_eq!(config.remarks.as_deref(), Some("my-server"));
        let rebuilt = config.reconstruct().unwrap();
        assert!(
            rebuilt.contains("#my-server"),
            "reconstruct should preserve fragment: {rebuilt}"
        );
    }
}
