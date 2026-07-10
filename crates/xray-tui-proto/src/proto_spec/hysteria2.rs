//! Hysteria2 (`hysteria2://` / `hy2://`) URL parsing.
//!
//! # Format
//! ```text
//! hysteria2://<auth>@<host>:<port>/?<query_params>#<remarks>
//! hy2://<auth>@<host>:<port>/?<query_params>#<remarks>
//! ```
//!
//! Canonical reference: `thirdparty/hysteria/app/cmd/client.go` `parseURI()`.
//! Both `hysteria2://` and `hy2://` schemes are accepted.
//!
//! # Fields
//!
//! | Component     | Source       | Purpose                         |
//! |---------------|--------------|---------------------------------|
//! | `auth`        | userinfo     | Authentication token/password   |
//! | `host`        | host         | Server address                  |
//! | `port`        | port         | Port (supports port hopping)    |
//! | `remarks`     | fragment (#) | Display name                    |
//!
//! # Query Parameters
//!
//! | Key            | Values                    | Purpose                          | Default   |
//! |----------------|---------------------------|----------------------------------|-----------|
//! | `obfs`         | salamander                | Obfuscation type                 | —         |
//! | `obfs-password`| string (min 4 bytes)      | Obfuscation pre-shared key       | —         |
//! | `insecure`     | 1/0, true/false           | Skip TLS verification            | `false`   |
//! | `sni`          | domain                    | TLS SNI override                 | hostname  |
//! | `up`           | bandwidth string          | Upload speed limit               | —         |
//! | `down`         | bandwidth string          | Download speed limit             | —         |
//! | `mportHopInt`  | integer (seconds)         | Port hopping interval            | —         |
//! | `pinSHA256`    | SHA-256 base64 string     | Certificate SHA-256 pin          | —         |
//!
//! # Port Hopping
//! Port supports special syntax from Hysteria's URL parser fork:
//! - Single: `:443`
//! - List: `:443,7788,9999`
//! - Range: `:8888-9999`
//! - Mixed: `:443,7788-8899,10010`
//!
//! # Edge Cases
//! - Auth can be single token (`auth@`) or `user:pass` pair (concatenated to `user:pass`)
//! - No auth → empty auth token (unusual, server may reject)
//! - Port defaults to 443
//! - Default port `"443"` when no port specified
//! - Salamander obfuscation uses BLAKE2b-256 with 8-byte random salt
//! - IPv6 addresses must be bracketed `[::1]`
//!
//! # References
//! - Hysteria2: `app/cmd/client.go` `parseURI()`, `app/internal/url/url.go`
//! - sing-box: `protocol/hysteria2/outbound.go`, `option/hysteria2.go`

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, PortSpec, RawUrlX, SchemeX, TinyText, host_serde, port_spec_serde};

use super::common::{SecurityConfig, TlsConfig, TlsOpts, should_skip_param};
use super::impl_sig_cache;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashHysteria2, ClashProxy};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_server_to_host, clash_tls_to_security, host_spec_to_string, port_spec_first,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct Hysteria2Config {
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<NonZeroU64>,
    #[serde(skip)]
    cred_hash_cache: std::sync::OnceLock<NonZeroU64>,

    pub auth: String,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_spec_serde")]
    pub port: PortSpec,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub obfs: Option<TinyText>,
    pub obfs_password: Option<TinyText>,
    pub up: Option<TinyText>,
    pub down: Option<TinyText>,
    pub hop_interval: Option<u32>,
    pub pin_sha256: Option<TinyText>,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for Hysteria2Config {
    /// Parse a Hysteria2 URL.
    ///
    /// Auth token is the userinfo (single token or user:pass pair).
    /// Port supports Hysteria's extended `PortSpec` (ranges, lists, mixed).
    /// Security defaults to "tls" (QUIC always uses TLS).
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let (auth, hostport) = if let Some(hostport) = raw.hostport {
            (raw.userinfo, hostport)
        } else {
            let userinfo = raw.userinfo;
            let (auth, hostport) = userinfo.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{userinfo}: missing hostport").into())
            })?;
            (auth, hostport)
        };

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;

        let query = utils::parse_query(raw.query);

        // obfs: obfuscation type (e.g., "salamander")
        let obfs = utils::query_get(&query, "obfs").map(TinyText::from);
        // obfs-password: pre-shared key for salamander obfuscation
        let obfs_password = utils::query_get(&query, "obfs-password").map(TinyText::from);
        // up/down: bandwidth limits (canonical impl doesn't parse these from URL)
        let up = utils::query_get(&query, "up").map(TinyText::from);
        let security = SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                pin_sha256: None,
                sni: utils::query_get(&query, "sni").map(TinyText::from),
                alpn: None,
                fp: None,
                insecure: utils::query_get(&query, "insecure").and_then(|v| match v {
                    "1" | "true" | "yes" => Some(true),
                    "0" | "false" | "no" => Some(false),
                    _ => None,
                }),
            })),
            enc: None,
        };
        let down = utils::query_get(&query, "down").map(TinyText::from);
        // hop_interval: port hopping interval in seconds (keys: mportHopInt, hop_interval)
        let hop_interval = utils::query_get_multi(&query, &["mportHopInt", "hop_interval"])
            .and_then(|v| v.parse().ok());
        // pin_sha256: certificate SHA-256 pin (keys: pinSHA256, pin_sha256)
        let pin_sha256 =
            utils::query_get_multi(&query, &["pinSHA256", "pin_sha256"]).map(TinyText::from);
        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            sig_cache: std::sync::OnceLock::new(),
            cred_hash_cache: std::sync::OnceLock::new(),
            auth: auth.to_string(),
            host: parsed_host,
            port: parsed_port,
            security,
            obfs,
            obfs_password,
            up,
            down,
            hop_interval,
            pin_sha256,
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

        let query_string = {
            let mut parts: Vec<String> = Vec::new();
            // Security config (always Tls for Hysteria2)
            if self.security.tls.is_some() {
                if self.security.insecure() == Some(true) {
                    parts.push("insecure=1".to_string());
                }
                if let Some(v) = self.security.sni()
                    && !should_skip_param(&self.host, v)
                {
                    parts.push(format!("sni={}", urlencoding::encode(v)));
                }
            }
            if let Some(ref v) = self.obfs {
                parts.push(format!("obfs={}", urlencoding::encode(v)));
            }
            if let Some(ref v) = self.obfs_password {
                parts.push(format!("obfs-password={}", urlencoding::encode(v)));
            }
            if let Some(ref v) = self.up {
                parts.push(format!("up={}", urlencoding::encode(v)));
            }
            if let Some(ref v) = self.down {
                parts.push(format!("down={}", urlencoding::encode(v)));
            }
            if let Some(v) = &self.hop_interval {
                parts.push(format!("mportHopInt={v}"));
            }
            if let Some(v) = &self.pin_sha256 {
                parts.push(format!("pinSHA256={}", urlencoding::encode(v)));
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

        Ok(format!(
            "hysteria2://{auth}@{hostport}{query_string}{fragment}",
            auth = self.auth,
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Hysteria2
    }

    fn host(&self) -> Option<&HostSpec> {
        Some(&self.host)
    }

    fn port(&self) -> Option<u16> {
        self.port.first()
    }

    fn remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    fn cred_hash(&self) -> u64 {
        let v = self.cred_hash_cache.get_or_init(|| {
            let val = utils::compute_cred_hash(
                Some(&self.host),
                None,
                Some(&self.port),
                &self.auth,
                &self.auth,
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
        Some("quic")
    }

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Hysteria2(c) => {
                let port = PortSpec::new_with(c.port);
                Ok(Self {
                    auth: c.password.clone(),
                    host: clash_server_to_host(&c.server)?,
                    port,
                    hop_interval: c.hop_interval,
                    up: c.up.clone().map(TinyText::from),
                    down: c.down.clone().map(TinyText::from),
                    obfs: c.obfs.clone().map(TinyText::from),
                    obfs_password: c.obfs_password.clone().map(TinyText::from),
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
                    pin_sha256: None,
                    sig_cache: std::sync::OnceLock::new(),
                    cred_hash_cache: std::sync::OnceLock::new(),
                })
            }
            _ => Err(ParseError::Unknown("expected hysteria2 clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let first_port = port_spec_first(&self.port);
        let alpn_str = self.security.alpn();
        Ok(ClashProxy::Hysteria2(ClashHysteria2 {
            name,
            server: host_spec_to_string(&self.host),
            port: first_port,
            password: self.auth.clone(),
            ports: None,
            hop_interval: self.hop_interval,
            up: self.up.as_ref().and_then(|v| v.parse().ok()),
            down: self.down.as_ref().and_then(|v| v.parse().ok()),
            obfs: self.obfs.as_ref().map(std::string::ToString::to_string),
            obfs_password: self
                .obfs_password
                .as_ref()
                .map(std::string::ToString::to_string),
            skip_cert_verify: self.security.insecure(),
            servername: self.security.sni().map(std::string::ToString::to_string),
            alpn: alpn_str.map(|s| vec![s.to_string()]),
        }))
    }
}

impl Hysteria2Config {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"hysteria2");
        let sec_type = self.security.type_str().unwrap_or("none");
        hasher.write(sec_type.as_bytes());
        if let Some(ref v) = self.obfs {
            hasher.write(v.as_bytes());
        }
        if let Some(ref v) = self.obfs_password {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.security.insecure() {
            hasher.write(if v { b"true" } else { b"false" });
        }
        if let Some(v) = self.security.sni() {
            hasher.write(v.as_bytes());
        }
        if let Some(ref v) = self.up {
            hasher.write(v.as_bytes());
        }
        if let Some(ref v) = self.down {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.hop_interval {
            hasher.write(v.to_string().as_bytes());
        }
        if let Some(v) = &self.pin_sha256 {
            hasher.write(v.as_bytes());
        }
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_hysteria2_basic() {
        let url = "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria2Config::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Hysteria2);
        assert_eq!(config.obfs.as_deref(), Some("salamander"));
        assert_eq!(config.security.insecure(), Some(true));
    }

    #[test]
    fn test_hy2_ipv6() {
        let url =
            "hy2://linux.do@[2a01:4f9:4b:f378::1]:13599?security=tls&insecure=1&sni=www.bing.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria2Config::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Hysteria2);
        assert_eq!(config.host.to_str(), "2a01:4f9:4b:f378::1");
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = Hysteria2Config::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = Hysteria2Config::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.auth, reparsed.auth, "auth mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input = "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = Hysteria2Config::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: Hysteria2Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.host, deserialized.host);
        assert_eq!(parsed.auth, deserialized.auth);
    }

    #[test]
    fn test_hysteria2_full() {
        let url = "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com&up=50mbps&down=100mbps&mportHopInt=10&pinSHA256=abc123deadbeef";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = Hysteria2Config::try_parse(&raw).expect("failed to parse full config");
        assert_eq!(config.schema(), SchemeX::Hysteria2);
        assert_eq!(config.obfs.as_deref(), Some("salamander"));
        assert_eq!(config.obfs_password.as_deref(), Some("password123"));
        assert_eq!(config.security.insecure(), Some(true));
        assert_eq!(config.security.sni(), Some("jnir.pichondan.com"));
        assert_eq!(config.up.as_deref(), Some("50mbps"));
        assert_eq!(config.down.as_deref(), Some("100mbps"));
        assert_eq!(config.hop_interval, Some(10));
        assert_eq!(config.pin_sha256.as_deref(), Some("abc123deadbeef"));

        // Roundtrip: reconstruct and re-parse
        let reconstructed = config.reconstruct().expect("reconstruct");
        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = Hysteria2Config::try_parse(&raw2).expect("re-parse");
        assert_eq!(reparsed.hop_interval, Some(10));
        assert_eq!(reparsed.pin_sha256.as_deref(), Some("abc123deadbeef"));
    }
    use super::super::test_helpers::check_roundtrip;
    use super::Hysteria2Config;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<Hysteria2Config>(
            "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com",
        );
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<Hysteria2Config>(
            "hysteria2://b4bd0613-ff7c-4f2f-954d-185915e6ddad@206.71.158.41:35000?security=tls&obfs=salamander&obfs-password=password123&insecure=1&sni=jnir.pichondan.com",
        );
    }
}
