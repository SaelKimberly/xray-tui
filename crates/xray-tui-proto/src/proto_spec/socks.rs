//! SOCKS5 (`socks://`) URL parsing.
//!
//! # Format
//! ```text
//! socks://[user:pass@]host:port[#remarks]
//! ```
//!
//! Standard URI format with optional username/password authentication.
//! Server address and port in hostport. Username and password (optional) in userinfo.
//! Remarks in fragment.
//!
//! # Fields
//!
//! | Component     | Source              | Purpose                         |
//! |---------------|----------------------|---------------------------------|
//! | `host`        | hostport             | Server address                  |
//! | `port`        | hostport             | Server port (default 1080)      |
//! | `username`    | userinfo (optional)  | SOCKS5 username                 |
//! | `password`    | userinfo (optional)  | SOCKS5 password                 |
//! | `remarks`     | fragment (#)         | Display name (URL-decoded)      |
//!
//! # Edge Cases
//! - Auth is optional; userinfo may contain username only (no colon) or username:password
//! - When no auth is present, `raw.userinfo` equals `raw.hostport` — checked to avoid
//!   misinterpreting the hostport string as a username
//! - IPv6 addresses must be bracketed in the URL
//! - Default port is 1080 if not specified (parse error if unparseable)
//!
//! # References
//! - sing-box: `option/simple.go` — `SOCKSOutboundOptions`
//! - mihomo: `adapter/outbound/socks5.go` — `Socks5Option`
//! - Xray-core: `proxy/socks/config.proto`

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::SecurityConfig;
use super::impl_sig_cache;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashProxy, ClashSocks5};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_server_to_host, clash_tls_to_security, host_spec_to_string, security_to_clash_tls,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct Socks5Config {
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

impl ProtoSpec for Socks5Config {
    /// Parse a SOCKS5 URL.
    ///
    /// Two code paths:
    /// 1. Hostport present (standard): userinfo from `raw.userinfo` (may be empty/
    ///    equal to hostport = no auth, or contain `user:pass`).
    /// 2. Hostport absent (fallback): split `raw.userinfo` at `@` to extract
    ///    auth and hostport.
    ///
    /// Userinfo is split at the first `:` for username:password. A lone username
    /// (no colon) is stored with password = `None`.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let (userinfo, hostport_str) = if let Some(hostport) = raw.hostport {
            // Standard: socks://[user:pass@]host:port[#remarks]
            (raw.userinfo, hostport)
        } else {
            // Fallback: socks://user:pass@host:port collapsed into userinfo
            let (ui, hp) = raw.userinfo.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{}: missing hostport", raw.userinfo).into())
            })?;
            (ui, hp)
        };

        // Parse userinfo at first ':' for username:password (or just username)
        let (username, password) = {
            // When no auth is present, raw.userinfo == raw.hostport (same string).
            // Treat empty or hostport-identical userinfo as "no auth".
            let has_auth = !userinfo.is_empty() && userinfo != hostport_str;
            if has_auth {
                match userinfo.split_once(':') {
                    Some((u, p)) => (Some(u.to_string()), Some(p.to_string())),
                    None => (Some(userinfo.to_string()), None),
                }
            } else {
                (None, None)
            }
        };

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport_str)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            sig_cache: std::sync::OnceLock::new(),
            cred_hash_cache: std::sync::OnceLock::new(),
            host: parsed_host,
            port: parsed_port,
            username,
            password,
            security: SecurityConfig::default(),
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

        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();

        Ok(format!("socks://{auth}{hostport}{fragment}"))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Socks
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
            let username = self.username.as_deref().unwrap_or("");
            let password = self.password.as_deref().unwrap_or("");
            let val = utils::compute_cred_hash(
                Some(&self.host),
                Some(self.port),
                None,
                username,
                password,
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

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Socks5(c) => Ok(Self {
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
            _ => Err(ParseError::Unknown("expected socks5 clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let (tls, servername, skip_cert_verify, _, _) = security_to_clash_tls(&self.security);
        Ok(ClashProxy::Socks5(ClashSocks5 {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
            tls,
            servername,
            skip_cert_verify,
            udp: None,
        }))
    }
}

impl Socks5Config {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"socks5");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use super::super::test_helpers::check_roundtrip;
    use super::Socks5Config;
    use crate::urlx::SchemeX;

    #[test]
    fn test_basic() {
        let url = "socks://user:pass@1.2.3.4:1080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Socks5Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.schema(), SchemeX::Socks);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 1080);
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_no_auth() {
        let url = "socks://1.2.3.4:1080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Socks5Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.port, 1080);
        assert!(config.username.is_none());
        assert!(config.password.is_none());
    }

    #[test]
    fn test_ipv6() {
        let url = "socks://[2001:db8::1]:1080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Socks5Config::try_parse(&raw).expect("failed to parse");
        assert_eq!(config.host.to_str(), "2001:db8::1");
        assert_eq!(config.port, 1080);
    }

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<Socks5Config>("socks://user:pass@1.2.3.4:1080");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<Socks5Config>("socks://user:pass@1.2.3.4:1080");
    }

    #[test]
    fn test_serde_roundtrip() {
        let url = "socks://user:pass@1.2.3.4:1080";
        let raw = crate::urlx::RawUrlX::from(url);
        let parsed = Socks5Config::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: Socks5Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.port, deserialized.port);
        assert_eq!(parsed.username, deserialized.username);
        assert_eq!(parsed.password, deserialized.password);
    }
}
