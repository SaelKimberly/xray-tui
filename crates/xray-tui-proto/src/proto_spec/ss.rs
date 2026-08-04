//! Shadowsocks (`ss://`) URL parsing.
//!
//! # Format (SIP002 — Modern Standard)
//! ```text
//! ss://<base64url_no_pad(method:password)>@<host>:<port>#<remarks>?plugin=...
//! ```
//!
//! # Legacy `QRCode` Format (also accepted)
//! ```text
//! ss://<base64(method:password@host:port)>
//! ```
//! Detected by presence/absence of `@` in the base64-decoded userinfo.
//!
//! # Plain Format (go-shadowsocks2 compatibility)
//! ```text
//! ss://<method>:<password>@<host>:<port>
//! ```
//!
//! # Fields
//!
//! | Component     | Source              | Purpose                         |
//! |---------------|----------------------|---------------------------------|
//! | `method`      | userinfo (method:password) | Encryption cipher         |
//! | `password`    | userinfo (method:password) | Shared secret             |
//! | `host`        | hostport             | Server address                  |
//! | `port`        | hostport             | Server port                     |
//! | `remarks`     | fragment (#)         | Display name (URL-decoded)      |
//! | `plugin`      | query `plugin`       | SIP003 plugin (e.g., obfs-local)|
//!
//! # Valid Ciphers
//! - Legacy: `rc4-md5`, `aes-256-cfb`, `chacha20`, `salsa20`, etc.
//! - AEAD: `aes-128-gcm`, `aes-256-gcm`, `chacha20-ietf-poly1305`, `xchacha20-ietf-poly1305`
//! - AEAD-2022: `2022-blake3-aes-128-gcm`, `2022-blake3-aes-256-gcm`
//!
//! # Edge Cases
//! - Base64 can be URL-safe (`-`/`_`) or standard (`+`/`/`), with/without padding
//! - Legacy format: whole `method:password@host:port` base64-encoded (no `@` in URL)
//! - AEAD-2022 passwords are already base64, not double-encoded
//! - Port defaults to 8388 if missing (shadowsocks-rust convention)
//! - IPv6 addresses must be bracketed
//!
//! # References
//! - shadowsocks-rust: `src/config.rs` SIP002 `from_url()`/`to_url()`
//! - SIP002 spec: <https://github.com/shadowsocks/shadowsocks-org/issues/27>
//! - subconverter: `subparser.cpp` `explodeSS()`
//! - go-shadowsocks2: `parseURL()` (plain format)

use std::collections::HashMap;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::SecurityConfig;
use super::ProtoIdentity;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashProxy, ClashSS};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_server_to_host, host_spec_to_string};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct SsConfig {
    pub method: TinyText,
    pub password: String,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
    pub plugin: Option<TinyText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<HashMap<String, String>>,
}

impl ProtoSpec for SsConfig {
    /// Parse a Shadowsocks URL.
    ///
    /// Supports three formats:
    /// 1. SIP002: `base64url(method:password)@host:port` (has `@`, hostport present)
    /// 2. Legacy QR: `base64(method:password@host:port)` (no `@` in URL, hostport absent)
    /// 3. Plain: `method:password@host:port` (base64 decode fails but `@` present)
    ///
    /// `decode_base64` tolerates trailing annotation text/emoji (Telegram pattern)
    /// and accepts both URL-safe and standard base64 alphabets.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let (userinfo, hostport) = if let Some(hostport) = raw.hostport {
            // SIP002 format: base64(method:password)@host:port
            let decoded = utils::decode_base64(raw.userinfo).map_err(|e| {
                ParseError::InvalidUserInfo(format!("{}: {e}", raw.userinfo).into())
            })?;
            let text = String::from_utf8(decoded).map_err(|e| {
                ParseError::InvalidUserInfo(format!("{}: {e}", raw.userinfo).into())
            })?;
            (text, hostport.to_string())
        } else {
            // Legacy QR format: base64(method:password@host:port) — no @ in URL
            let decoded = utils::decode_base64(raw.userinfo).map_err(|e| {
                ParseError::InvalidUserInfo(format!("{}: {e}", raw.userinfo).into())
            })?;
            let text = String::from_utf8(decoded).map_err(|e| {
                ParseError::InvalidUserInfo(format!("{}: {e}", raw.userinfo).into())
            })?;
            let (ui, hp) = text.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{}: missing hostport", raw.userinfo).into())
            })?;
            (ui.to_string(), hp.to_string())
        };

        let (parsed_host, parsed_port) = utils::parse_hostport(&hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        // Split at first ':' to get method:password
        let (method, password) = userinfo.split_once(':').ok_or_else(|| {
            ParseError::InvalidUserInfo(format!("{}: missing password", raw.userinfo).into())
        })?;

        let remarks = utils::decode_fragment(raw)?;

        let query = utils::parse_query(raw.query);
        let plugin = utils::query_get(&query, "plugin").map(TinyText::from);
        let plugin_opts = utils::query_get(&query, "plugin_opts").map(|s| {
            s.split(';')
                .filter_map(|pair| {
                    pair.split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect::<HashMap<String, String>>()
        });

        Ok(Self {
            method: TinyText::from(method),
            password: password.to_string(),
            host: parsed_host,
            port: parsed_port,
            security: SecurityConfig::default(),
            remarks,
            plugin,
            plugin_opts,
        })
    }

    fn reconstruct(&self) -> Result<String, ParseError> {
        let userinfo = format!("{}:{}", self.method, self.password);
        let encoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(userinfo.as_bytes());
        let host = self.host.to_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", self.port)
        } else {
            format!("{host}:{}", self.port)
        };
        let mut query_parts: Vec<String> = Vec::new();
        if let Some(plugin) = &self.plugin {
            query_parts.push(format!("plugin={}", urlencoding::encode(plugin)));
        }
        if let Some(opts) = &self.plugin_opts {
            let encoded_opts = opts
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(";");
            query_parts.push(format!(
                "plugin_opts={}",
                urlencoding::encode(&encoded_opts)
            ));
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
        Ok(format!("ss://{encoded}@{hostport}{query_string}{fragment}"))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::SS
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
            ClashProxy::Shadowsocks(c) => Ok(Self {
                method: TinyText::from(c.cipher.as_str()),
                password: c.password.clone(),
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                security: SecurityConfig::default(),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
                plugin: c.plugin.clone().map(TinyText::from),
                plugin_opts: c.plugin_opts.as_ref().map(|opts_str| {
                    opts_str
                        .split(';')
                        .filter_map(|pair| {
                            pair.split_once('=')
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                        })
                        .collect::<HashMap<String, String>>()
                }),
            }),
            _ => Err(ParseError::Unknown(
                "expected shadowsocks clash proxy".into(),
            )),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Shadowsocks(ClashSS {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            cipher: self.method.to_string(),
            password: self.password.clone(),
            udp: None,
            udp_over_tcp: None,
            plugin: self.plugin.as_ref().map(std::string::ToString::to_string),
            plugin_opts: self.plugin_opts.as_ref().map(|opts| {
                opts.iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(";")
            }),
        }))
    }
}

impl ProtoIdentity for SsConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"ss");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        if let Some(plugin) = &self.plugin {
            hasher.write(plugin.as_bytes());
        }
        if let Some(opts) = &self.plugin_opts {
            for (k, v) in opts {
                hasher.write(k.as_bytes());
                hasher.write(b"=");
                hasher.write(v.as_bytes());
                hasher.write(b";");
            }
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[
            ("method", self.method.as_str()),
            ("password", self.password.as_str()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Proto, ProtocolConfig, ProtoSpec};
    use crate::urlx::SchemeX;

    #[test]
    fn test_ss_basic() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = SsConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::SS);
        assert_eq!(config.host.to_str(), "1.2.3.4");
        assert_eq!(config.method, "cleof");
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = SsConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = SsConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.method, reparsed.method, "method mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = SsConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: SsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.method, deserialized.method);
    }

    use super::super::test_helpers::check_roundtrip;
    use super::SsConfig;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<SsConfig>("ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<SsConfig>("ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080");
    }

    #[test]
    fn ss_password_is_credential_not_sig() {
        let url_a = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080"; // cleof:password
        let url_b = "ss://Y2xlb2Y6cGFzczEyMw==@1.2.3.4:8080"; // cleof:pass123
        let a = Proto::new(ProtocolConfig::Ss(
            SsConfig::try_parse(&crate::urlx::RawUrlX::from(url_a)).expect("failed"),
        ));
        let b = Proto::new(ProtocolConfig::Ss(
            SsConfig::try_parse(&crate::urlx::RawUrlX::from(url_b)).expect("failed"),
        ));
        assert_eq!(a.sig(), b.sig(), "password must not change sig");
        assert_ne!(a.cred_hash(), b.cred_hash(), "password changes cred_hash");
        assert_ne!(a.uid(), b.uid());
    }

    #[test]
    fn ss_host_port_are_identity_in_sig() {
        let url_a = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let url_b = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.5:8080";
        let a = Proto::new(ProtocolConfig::Ss(
            SsConfig::try_parse(&crate::urlx::RawUrlX::from(url_a)).expect("failed"),
        ));
        let b = Proto::new(ProtocolConfig::Ss(
            SsConfig::try_parse(&crate::urlx::RawUrlX::from(url_b)).expect("failed"),
        ));
        assert_ne!(a.sig(), b.sig(), "different host -> different sig");
        assert_eq!(a.cred_hash(), b.cred_hash());
    }

    #[test]
    fn ss_reconstruct_with_remarks() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@example.com:443#my-server";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = SsConfig::try_parse(&raw).unwrap();
        assert_eq!(config.remarks.as_deref(), Some("my-server"));
        let rebuilt = config.reconstruct().unwrap();
        assert!(
            rebuilt.contains("#my-server"),
            "reconstruct should preserve fragment: {rebuilt}"
        );
    }
}
