use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use smallvec::SmallVec;

use crate::clash::ClashProxy;
use crate::urlx::{HostSpec, RawUrlX, SchemeX};

pub mod common;
pub mod utils;

mod anytls;
mod http_client;
mod hysteria1;
mod hysteria2;
mod naive;
pub mod security_rank;
mod shadowtls;
mod socks;
mod ss;
mod ssh;
mod ssr;
mod tailscale;
mod tor;
mod trojan;
mod tuic;
mod vless;
mod vmess;
mod wireguard;
pub use anytls::AnyTlsConfig;
pub use common::{HttpUpgradeConfig, RealityOpts, SecurityConfig, TlsConfig, TlsOpts, XHttpConfig};
pub use http_client::HttpClientConfig;
pub use hysteria1::Hysteria1Config;
pub use hysteria2::Hysteria2Config;
pub use naive::NaiveConfig;
pub use security_rank::protocol_security_rank;
pub use shadowtls::ShadowTlsConfig;
pub use socks::Socks5Config;
pub use ss::SsConfig;
pub use ssh::SshConfig;
pub use ssr::SsrConfig;
pub use tailscale::TailscaleConfig;
pub use tor::TorConfig;
pub use trojan::TrojanConfig;
pub use tuic::TuicConfig;
pub use vless::VlessConfig;
pub use vmess::VmessConfig;
pub use wireguard::WireguardConfig;
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid host: {0}")]
    InvalidHost(Cow<'static, str>),
    #[error("invalid port: {0}")]
    InvalidPort(Cow<'static, str>),
    #[error("missing host")]
    MissingHost,
    #[error("missing port")]
    MissingPort,
    #[error("invalid userinfo: {0}")]
    InvalidUserInfo(Cow<'static, str>),
    #[error("invalid hostport: {0}")]
    InvalidHostPort(Cow<'static, str>),
    #[error("missing conf: {0}")]
    MissingConf(Cow<'static, str>),
    #[error("invalid conf: {0}: {1}")]
    InvalidConf(Cow<'static, str>, Cow<'static, str>),
    #[error("unknown error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("invalid structure for {0}")]
    InvalidStructure(SchemeX),
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(SchemeX),
    #[error("not a proxy config URL (promotion or navigation link)")]
    PromotionUrl,
    /// Private/reserved/loopback IP address as the server host.
    #[error("private/reserved host: {0}")]
    InvalidPrivateHost(Cow<'static, str>),
    /// The protocol is recognized but not yet implemented as a placeholder.
    #[error("protocol not yet implemented: {0}")]
    Unimplemented(&'static str),
}

#[derive(Debug, Clone)]
pub struct FallbackInfo {
    pub raw_url: String,
    pub original_scheme: SchemeX,
    pub original_error: String,
}

pub enum ParseResult {
    Direct(ProtocolConfig),
    Fallback(ProtocolConfig, FallbackInfo),
}

/// Which proxy core to target for JSON config generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreType {
    Xray,
    SingBox,
}

impl CoreType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Xray => "xray",
            Self::SingBox => "sing-box",
        }
    }
}

impl std::str::FromStr for CoreType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "xray" | "xray-core" => Ok(Self::Xray),
            "sing-box" | "singbox" => Ok(Self::SingBox),
            _ => Err(()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtoSpecError {
    #[error("invalid config: {0}")]
    Invalid(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unsupported protocol for {0}")]
    Unsupported(String),
}

impl ProtocolConfig {
    /// Like [`ProtoSpec::try_parse`] but distinguishes direct vs. fallback parses.
    ///
    /// # Errors
    ///
    /// If the URL is not a valid proxy URL for any supported protocol.
    pub fn try_parse_detailed(raw: &RawUrlX<'_>) -> Result<ParseResult, ParseError> {
        let r = match raw.schema {
            SchemeX::Vless => VlessConfig::try_parse(raw).map(Self::Vless),
            SchemeX::Trojan => TrojanConfig::try_parse(raw).map(Self::Trojan),
            SchemeX::Vmess => VmessConfig::try_parse(raw).map(Self::Vmess),
            SchemeX::Hysteria => Hysteria1Config::try_parse(raw).map(Self::Hysteria1),
            SchemeX::Hysteria2 => Hysteria2Config::try_parse(raw).map(Self::Hysteria2),
            SchemeX::SS => SsConfig::try_parse(raw).map(Self::Ss),
            SchemeX::SSR => SsrConfig::try_parse(raw).map(Self::Ssr),
            SchemeX::TUIC => TuicConfig::try_parse(raw).map(Self::Tuic),
            SchemeX::WireGuard => WireguardConfig::try_parse(raw).map(Self::Wireguard),
            SchemeX::ShadowTls => ShadowTlsConfig::try_parse(raw).map(Self::ShadowTls),
            SchemeX::Socks => Socks5Config::try_parse(raw).map(Self::Socks),
            SchemeX::Http => HttpClientConfig::try_parse(raw).map(Self::Http),
            SchemeX::Naive => NaiveConfig::try_parse(raw).map(Self::Naive),
            SchemeX::AnyTLS => AnyTlsConfig::try_parse(raw).map(Self::AnyTls),
            SchemeX::Warp => {
                // Warp is not directly URL-parsable — fall through to fallback
                Err(ParseError::UnsupportedScheme(raw.schema.clone()))
            }
            SchemeX::Undefined | SchemeX::Https => return Err(ParseError::PromotionUrl),
            ref other => return Err(ParseError::UnsupportedScheme(other.clone())),
        };

        let original_err = match r {
            Ok(r) => return Ok(ParseResult::Direct(r)),
            Err(
                e @ (ParseError::InvalidStructure(_)
                | ParseError::MissingHost
                | ParseError::MissingPort
                | ParseError::InvalidUserInfo(_)
                | ParseError::InvalidHostPort(_)
                | ParseError::InvalidHost(_)
                | ParseError::Unknown(_)),
            ) => e,
            unrecoverable @ Err(_) => return Err(unrecoverable.unwrap_err()),
        };
        let original_scheme = raw.schema.clone();
        let original_error = original_err.to_string();
        let v = SsConfig::try_parse(raw)
            .map(Self::Ss)
            .or_else(|_| SsrConfig::try_parse(raw).map(Self::Ssr))
            .or_else(|_| VmessConfig::try_parse(raw).map(Self::Vmess))
            .or_else(|_| VlessConfig::try_parse(raw).map(Self::Vless))
            .or_else(|_| TrojanConfig::try_parse(raw).map(Self::Trojan))
            .or_else(|_| Hysteria2Config::try_parse(raw).map(Self::Hysteria2))
            .or_else(|_| Hysteria1Config::try_parse(raw).map(Self::Hysteria1))
            .or(Err(original_err))?;
        Ok(ParseResult::Fallback(
            v,
            FallbackInfo {
                raw_url: raw.raw.to_string(),
                original_scheme,
                original_error,
            },
        ))
    }

    /// Construct a `ProtocolConfig` from legacy parser fields.
    ///
    /// Wraps the protocol-specific settings as a [`PlaceholderConfig`] with an
    /// opaque JSON blob. This lets the existing config builders continue to work
    /// by reading from `to_settings()`.
    #[must_use]
    pub fn from_legacy_parse(proto_name: &str, settings_json: Vec<u8>) -> Self {
        let placeholder = |name: &str, json: Vec<u8>| PlaceholderConfig {
            proto_name: name.to_string(),
            settings_json: json,
            sig_cache: std::sync::OnceLock::new(),
        };
        // Only for protocols that don't have URL format (PlaceholderConfig variants).
        // Full-protocol types must use ProtocolConfig::try_parse().
        match proto_name.to_lowercase().as_str() {
            "redirect" => Self::Redirect(placeholder(proto_name, settings_json)),
            "tproxy" => Self::TProxy(placeholder(proto_name, settings_json)),
            "mixed" => Self::Mixed(placeholder(proto_name, settings_json)),
            p => {
                // Unknown/unparsed protocol — wrap as Mixed for backward compat
                Self::Mixed(placeholder(p, settings_json))
            }
        }
    }

    /// Extract `protocol_settings` and `stream_settings` as JSON Values.
    ///
    /// For full protocol config types, builds from typed fields.
    /// For [`PlaceholderConfig`] stubs, extracts from the opaque `settings_json` blob.
    #[must_use]
    pub fn to_settings(&self) -> (serde_json::Value, serde_json::Value) {
        match self {
            // Placeholder-based protocols: extract from settings_json
            Self::Redirect(c) | Self::TProxy(c) | Self::Mixed(c) => {
                let extra: serde_json::Value =
                    serde_json::from_slice(&c.settings_json).unwrap_or_default();
                let mut p = extra
                    .get("protocol_settings")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                // Inject `user_id` as `id` into p_settings if absent (legacy
                // parsers store UUID/PW at top level, not inside protocol_settings).
                if let Some(user_id) = extra.get("user_id").and_then(|v| v.as_str())
                    && let Some(obj) = p.as_object_mut()
                    && !obj.contains_key("id")
                    && !obj.contains_key("uuid")
                {
                    obj.entry("id".to_string())
                        .or_insert(serde_json::Value::String(user_id.to_string()));
                }
                let s = extra
                    .get("stream_settings")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                (p, s)
            }
            // Typed protocols: build from typed config fields
            Self::Vmess(c) => {
                let mut p = serde_json::Map::new();
                p.insert("id".into(), json!(c.uuid));
                if let Some(sec) = c.security.type_str() {
                    p.insert("security".into(), json!(sec));
                }
                p.insert("encryption".into(), json!("auto"));
                (
                    Value::Object(p),
                    common::to_xray_stream_settings(&c.security, &c.transport)
                        .unwrap_or_else(|| json!({})),
                )
            }
            Self::Vless(c) => {
                let mut p = serde_json::Map::new();
                p.insert("id".into(), json!(c.uuid));
                p.insert(
                    "encryption".into(),
                    json!(c.encryption.as_deref().unwrap_or("none")),
                );
                if let Some(ref flow) = c.flow {
                    p.insert("flow".into(), json!(flow));
                }
                (
                    Value::Object(p),
                    common::to_xray_stream_settings(&c.security, &c.transport)
                        .unwrap_or_else(|| json!({})),
                )
            }
            Self::Trojan(c) => (
                json!({"password": c.password}),
                common::to_xray_stream_settings(&c.security, &c.transport)
                    .unwrap_or_else(|| json!({})),
            ),
            Self::Hysteria2(c) => {
                let mut p = serde_json::Map::new();
                // Hysteria2 auth token is the password
                p.insert("password".into(), json!(c.auth));
                if let Some(ref up) = c.up
                    && let Ok(v) = up.as_str().parse::<u64>()
                {
                    p.insert("up_mbps".into(), json!(v));
                }
                if let Some(ref down) = c.down
                    && let Ok(v) = down.as_str().parse::<u64>()
                {
                    p.insert("down_mbps".into(), json!(v));
                }
                (Value::Object(p), json!({}))
            }
            Self::Ss(c) => {
                let mut p = serde_json::Map::new();
                p.insert("password".into(), json!(c.password));
                p.insert("method".into(), json!(c.method));
                if let Some(ref plugin) = c.plugin {
                    p.insert("plugin".into(), json!(plugin.as_str()));
                }
                if let Some(ref opts) = c.plugin_opts {
                    let joined: String = opts
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(";");
                    p.insert("plugin_opts".into(), json!(joined));
                }
                (Value::Object(p), json!({}))
            }
            Self::Ssr(c) => {
                let mut p = serde_json::Map::new();
                p.insert("password".into(), json!(c.password));
                p.insert("method".into(), json!(c.method));
                for (k, v) in &c.params {
                    p.insert(k.clone(), json!(v));
                }
                (Value::Object(p), json!({}))
            }
            Self::Tuic(c) => {
                let mut p = serde_json::Map::new();
                p.insert("uuid".into(), json!(c.uuid));
                p.insert("password".into(), json!(c.password));
                if let Some(ref cc) = c.congestion_control {
                    p.insert("congestion_control".into(), json!(cc.as_str()));
                }
                if let Some(ref urm) = c.udp_relay_mode {
                    p.insert("udp_relay_mode".into(), json!(urm.as_str()));
                }
                (Value::Object(p), json!({}))
            }
            Self::Wireguard(c) => {
                let mut p = serde_json::Map::new();
                p.insert("private_key".into(), json!(c.private_key));
                p.insert("public_key".into(), json!(c.public_key));
                p.insert("address".into(), json!(c.address.as_str()));
                if let Some(ref pk) = c.preshared_key {
                    p.insert("preshared_key".into(), json!(pk));
                }
                if let Some(ref r) = c.reserved {
                    p.insert("reserved".into(), json!(r.as_str()));
                }
                if let Some(ref mtu) = c.mtu {
                    p.insert("mtu".into(), json!(mtu.as_str()));
                }
                if let Some(ref k) = c.persistent_keepalive {
                    p.insert("persistent_keepalive".into(), json!(k));
                }
                (Value::Object(p), json!({}))
            }
            Self::Socks(c) => {
                let mut p = serde_json::Map::new();
                if let Some(ref user) = c.username {
                    p.insert("username".into(), json!(user));
                }
                if let Some(ref pass) = c.password {
                    p.insert("password".into(), json!(pass));
                }
                (Value::Object(p), json!({}))
            }
            Self::Http(c) => {
                let mut p = serde_json::Map::new();
                if let Some(ref user) = c.username {
                    p.insert("username".into(), json!(user));
                }
                if let Some(ref pass) = c.password {
                    p.insert("password".into(), json!(pass));
                }
                (Value::Object(p), json!({}))
            }
            Self::Naive(c) => {
                let mut p = serde_json::Map::new();
                p.insert("username".into(), json!(c.username));
                p.insert("password".into(), json!(c.password));
                (Value::Object(p), json!({}))
            }
            Self::AnyTls(c) => {
                let mut p = serde_json::Map::new();
                p.insert("password".into(), json!(c.password));
                (Value::Object(p), json!({}))
            }
            Self::ShadowTls(c) => {
                let mut p = serde_json::Map::new();
                p.insert("password".into(), json!(c.password));
                if let Some(ref ver) = c.version {
                    p.insert("version".into(), json!(ver.as_str()));
                }
                (Value::Object(p), json!({}))
            }
            Self::Tor(c) => {
                let mut p = serde_json::Map::new();
                if let Some(ref dir) = c.data_directory {
                    p.insert("data_dir".into(), json!(dir));
                }
                (Value::Object(p), json!({}))
            }
            Self::Ssh(c) => {
                let mut p = serde_json::Map::new();
                if let Some(ref user) = c.user {
                    p.insert("username".into(), json!(user));
                }
                if let Some(ref pass) = c.password {
                    p.insert("password".into(), json!(pass));
                }
                if let Some(ref key) = c.private_key {
                    p.insert("private_key".into(), json!(key));
                }
                (Value::Object(p), json!({}))
            }
            Self::Tailscale(c) => {
                let mut p = serde_json::Map::new();
                p.insert("auth_key".into(), json!(c.auth_key));
                if let Some(ref url) = c.control_url {
                    p.insert("control_url".into(), json!(url));
                }
                p.insert("ephemeral".into(), json!(c.ephemeral));
                (Value::Object(p), json!({}))
            }
            Self::Hysteria1(c) => {
                let mut p = serde_json::Map::new();
                if let Some(ref a) = c.auth {
                    p.insert("auth".into(), json!(a));
                }
                if let Some(up) = c.up_mbps {
                    p.insert("up_mbps".into(), json!(up));
                }
                if let Some(down) = c.down_mbps {
                    p.insert("down_mbps".into(), json!(down));
                }
                if let Some(ref obfs) = c.obfs {
                    p.insert("obfs".into(), json!(obfs.as_str()));
                }
                (Value::Object(p), json!({}))
            }
        }
    }
}

pub trait ProtoSpec: Serialize + DeserializeOwned + std::fmt::Debug + Clone {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError>;
    /// # Errors
    ///
    /// If internal configuration is invalid.
    fn reconstruct(&self) -> Result<String, ParseError>;
    fn schema(&self) -> SchemeX;
    fn host(&self) -> Option<&HostSpec>;
    fn port(&self) -> Option<u16>;
    fn remarks(&self) -> Option<&str>;
    fn cred_hash(&self) -> u64;
    fn sig(&self) -> u64;
    fn set_sig_cache(&self, v: std::num::NonZeroU64);
    fn set_cred_hash_cache(&self, v: u64);
    fn uid(&self) -> u64 {
        self.sig() ^ self.cred_hash()
    }
    fn security(&self) -> Option<&SecurityConfig> {
        None
    }
    fn transport_type(&self) -> Option<&str>;
    fn security_type(&self) -> Option<&str> {
        self.security().and_then(SecurityConfig::type_str)
    }
    /// Extract country flag emojis detected in this server's remarks string.
    /// Returns deduplicated flag emojis in order of detection.
    fn country_flags(&self) -> SmallVec<[crate::urlx::TinyText; 4]> {
        self.remarks().map(|_| SmallVec::new()).unwrap_or_default()
    }
    /// Generate JSON config for the specified proxy core.
    ///
    /// # Errors
    ///
    /// Returns [`ProtoSpecError`] if the config cannot be serialized.
    fn to_json_config(&self, _core: CoreType) -> Result<serde_json::Value, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "json config not implemented for this protocol".into(),
        ))
    }

    /// Parse this protocol from a Clash YAML proxy entry.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if the Clash proxy doesn't match this protocol type
    /// or contains invalid/missing fields.
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let _ = proxy;
        Err(ParseError::Unknown(
            "clash parsing not implemented for this protocol".into(),
        ))
    }

    /// Serialize this protocol to a Clash YAML proxy entry.
    ///
    /// # Errors
    ///
    /// Returns [`ProtoSpecError`] if the config cannot be converted.
    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "clash serialization not implemented for this protocol".into(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(tag = "schema")]
pub enum ProtocolConfig {
    Vless(VlessConfig),
    Vmess(VmessConfig),
    Trojan(TrojanConfig),
    Hysteria2(Hysteria2Config),
    Ss(SsConfig),
    Ssr(SsrConfig),
    Tuic(TuicConfig),
    Wireguard(WireguardConfig),
    // ── Typed protocol configs ──
    Socks(Socks5Config),
    Http(HttpClientConfig),
    Naive(NaiveConfig),
    AnyTls(AnyTlsConfig),
    ShadowTls(ShadowTlsConfig),
    Tor(TorConfig),
    Ssh(SshConfig),
    Tailscale(TailscaleConfig),
    Hysteria1(Hysteria1Config),
    // ── Legacy placeholder protocols (no URL format) ──
    Redirect(PlaceholderConfig),
    TProxy(PlaceholderConfig),
    Mixed(PlaceholderConfig),
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ProtocolConfig::Vless(c) => c.$method($($arg),*),
            ProtocolConfig::Vmess(c) => c.$method($($arg),*),
            ProtocolConfig::Trojan(c) => c.$method($($arg),*),
            ProtocolConfig::Hysteria2(c) => c.$method($($arg),*),
            ProtocolConfig::Ss(c) => c.$method($($arg),*),
            ProtocolConfig::Ssr(c) => c.$method($($arg),*),
            ProtocolConfig::Tuic(c) => c.$method($($arg),*),
            ProtocolConfig::Wireguard(c) => c.$method($($arg),*),
            ProtocolConfig::Socks(c) => c.$method($($arg),*),
            ProtocolConfig::Http(c) => c.$method($($arg),*),
            ProtocolConfig::Naive(c) => c.$method($($arg),*),
            ProtocolConfig::AnyTls(c) => c.$method($($arg),*),
            ProtocolConfig::ShadowTls(c) => c.$method($($arg),*),
            ProtocolConfig::Tor(c) => c.$method($($arg),*),
            ProtocolConfig::Ssh(c) => c.$method($($arg),*),
            ProtocolConfig::Tailscale(c) => c.$method($($arg),*),
            ProtocolConfig::Hysteria1(c) => c.$method($($arg),*),
            ProtocolConfig::Redirect(c) => c.$method($($arg),*),
            ProtocolConfig::TProxy(c) => c.$method($($arg),*),
            ProtocolConfig::Mixed(c) => c.$method($($arg),*),
        }
    };
}

macro_rules! impl_sig_cache {
    () => {
        fn sig(&self) -> u64 {
            let v = self.sig_cache.get_or_init(|| {
                std::num::NonZeroU64::new(self.compute_sig()).unwrap_or(std::num::NonZeroU64::MIN)
            });
            v.get()
        }
        fn set_sig_cache(&self, v: std::num::NonZeroU64) {
            _ = self.sig_cache.set(v);
        }
    };
}
pub(crate) use impl_sig_cache;

impl ProtoSpec for ProtocolConfig {
    fn reconstruct(&self) -> Result<String, ParseError> {
        dispatch!(self, reconstruct)
    }
    fn schema(&self) -> SchemeX {
        dispatch!(self, schema)
    }
    fn host(&self) -> Option<&HostSpec> {
        dispatch!(self, host)
    }
    fn port(&self) -> Option<u16> {
        dispatch!(self, port)
    }
    fn remarks(&self) -> Option<&str> {
        dispatch!(self, remarks)
    }
    fn cred_hash(&self) -> u64 {
        dispatch!(self, cred_hash)
    }
    fn sig(&self) -> u64 {
        dispatch!(self, sig)
    }
    fn set_sig_cache(&self, v: std::num::NonZeroU64) {
        dispatch!(self, set_sig_cache, v);
    }
    fn set_cred_hash_cache(&self, v: u64) {
        dispatch!(self, set_cred_hash_cache, v);
    }
    fn security(&self) -> Option<&SecurityConfig> {
        dispatch!(self, security)
    }
    fn transport_type(&self) -> Option<&str> {
        dispatch!(self, transport_type)
    }
    fn security_type(&self) -> Option<&str> {
        dispatch!(self, security_type)
    }
    fn country_flags(&self) -> SmallVec<[crate::urlx::TinyText; 4]> {
        dispatch!(self, country_flags)
    }
    fn to_json_config(&self, core: CoreType) -> Result<serde_json::Value, ProtoSpecError> {
        dispatch!(self, to_json_config, core)
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Vmess(_) => VmessConfig::try_from_clash(proxy).map(Self::Vmess),
            ClashProxy::Vless(_) => VlessConfig::try_from_clash(proxy).map(Self::Vless),
            ClashProxy::Trojan(_) => TrojanConfig::try_from_clash(proxy).map(Self::Trojan),
            ClashProxy::Shadowsocks(_) => SsConfig::try_from_clash(proxy).map(Self::Ss),
            ClashProxy::ShadowsocksR(_) => SsrConfig::try_from_clash(proxy).map(Self::Ssr),
            ClashProxy::Socks5(_) => Socks5Config::try_from_clash(proxy).map(Self::Socks),
            ClashProxy::Http(_) => HttpClientConfig::try_from_clash(proxy).map(Self::Http),
            ClashProxy::Tuic(_) => TuicConfig::try_from_clash(proxy).map(Self::Tuic),
            ClashProxy::Hysteria2(_) => Hysteria2Config::try_from_clash(proxy).map(Self::Hysteria2),
            ClashProxy::Hysteria(_) => Hysteria1Config::try_from_clash(proxy).map(Self::Hysteria1),
            ClashProxy::Wireguard(_) => WireguardConfig::try_from_clash(proxy).map(Self::Wireguard),
            ClashProxy::Naive(_) => NaiveConfig::try_from_clash(proxy).map(Self::Naive),
            ClashProxy::Anytls(_) => AnyTlsConfig::try_from_clash(proxy).map(Self::AnyTls),
            ClashProxy::Shadowtls(_) => ShadowTlsConfig::try_from_clash(proxy).map(Self::ShadowTls),
            ClashProxy::Tor(_) => TorConfig::try_from_clash(proxy).map(Self::Tor),
            ClashProxy::Ssh(_) => SshConfig::try_from_clash(proxy).map(Self::Ssh),
            ClashProxy::Tailscale(_) => TailscaleConfig::try_from_clash(proxy).map(Self::Tailscale),
            ClashProxy::Snell(_)
            | ClashProxy::Direct(_)
            | ClashProxy::Dns(_)
            | ClashProxy::Reject(_) => Err(ParseError::Unknown(
                "cannot convert clash proxy type to outbound protocol".into(),
            )),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        dispatch!(self, to_clash)
    }

    /// # Errors
    ///
    /// If the URL is not a valid proxy URL for any supported protocol.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        match Self::try_parse_detailed(raw) {
            Ok(ParseResult::Direct(c) | ParseResult::Fallback(c, _)) => Ok(c),
            Err(e) => Err(e),
        }
    }
}

// ── Placeholder config type ─────────────────────────────────────────────

/// Stub config for protocols not yet implemented.
/// All `ProtoSpec` methods return errors or defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct PlaceholderConfig {
    pub proto_name: String,
    /// Opaque JSON blob containing `protocol_settings/stream_settings` from legacy parsing.
    /// Stored as `{"protocol_settings": {...}, "stream_settings": {...}}` JSON.
    pub settings_json: Vec<u8>,
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<std::num::NonZeroU64>,
}

impl ProtoSpec for PlaceholderConfig {
    fn try_parse(_raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        Err(ParseError::Unimplemented("placeholder protocol"))
    }
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::Unimplemented("placeholder protocol"))
    }
    fn schema(&self) -> SchemeX {
        SchemeX::Undefined
    }
    fn host(&self) -> Option<&HostSpec> {
        None
    }
    fn port(&self) -> Option<u16> {
        None
    }
    fn remarks(&self) -> Option<&str> {
        None
    }
    fn cred_hash(&self) -> u64 {
        0
    }
    impl_sig_cache!();
    fn set_cred_hash_cache(&self, _v: u64) {}
    fn transport_type(&self) -> Option<&str> {
        None
    }
    fn to_json_config(&self, _core: CoreType) -> Result<serde_json::Value, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(self.proto_name.clone()))
    }
}

impl PlaceholderConfig {
    /// Construct a placeholder wrapping an opaque legacy JSON body.
    #[must_use]
    pub fn new(proto_name: String, settings_json: Vec<u8>) -> Self {
        Self {
            proto_name,
            settings_json,
            sig_cache: std::sync::OnceLock::new(),
        }
    }

    fn compute_sig(&self) -> u64 {
        // Opaque legacy blob: we cannot decompose semantic fields reliably,
        // so the sig is a deterministic rapidhash over the ENTIRE body
        // (proto_name + settings_json). Same body -> same uid (dedup); never
        // zero (mapped to NonZeroU64::MIN by the macro).
        use rapidhash::v3::{RapidStreamHasherV3, DEFAULT_RAPID_SECRETS};
        let mut hasher = RapidStreamHasherV3::new(&DEFAULT_RAPID_SECRETS);
        hasher.write(self.proto_name.as_bytes());
        hasher.write(&self.settings_json);
        hasher.finish()
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::ProtoSpec;
    use crate::urlx::RawUrlX;

    pub fn check_roundtrip<T>(url: &str)
    where
        T: ProtoSpec + std::fmt::Debug + PartialEq,
    {
        let raw = RawUrlX::from(url);
        let parsed = T::try_parse(&raw).unwrap_or_else(|e| panic!("parse failed for {url}: {e}"));
        parsed.sig();
        let reconstructed = parsed
            .reconstruct()
            .unwrap_or_else(|e| panic!("reconstruct failed for {url}: {e}"));
        let re_raw = RawUrlX::from(reconstructed.as_str());
        let reparsed = T::try_parse(&re_raw)
            .unwrap_or_else(|e| panic!("reparse failed for {reconstructed}: {e}"));
        reparsed.sig();
        assert_eq!(parsed, reparsed, "roundtrip failed for: {url}");
    }

    /// Test Clash roundtrip: parse URL -> config -> to_clash -> try_from_clash -> config
    pub fn check_clash_roundtrip<T>(url: &str)
    where
        T: ProtoSpec + std::fmt::Debug + PartialEq,
    {
        let raw = crate::urlx::RawUrlX::from(url);
        let parsed = T::try_parse(&raw).unwrap_or_else(|e| panic!("parse failed for {url}: {e}"));
        let clash = parsed
            .to_clash()
            .unwrap_or_else(|e| panic!("to_clash failed for {url}: {e}"));
        let reparsed = T::try_from_clash(&clash)
            .unwrap_or_else(|e| panic!("try_from_clash failed for {url}: {e}"));
        assert_eq!(parsed, reparsed, "clash roundtrip failed for: {url}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_config_sig_is_deterministic_nonzero_body_hash() {
        let blob = serde_json::json!({
            "protocol_settings": {"password": "sekrit"},
            "stream_settings": {}
        });
        let json = serde_json::to_vec(&blob).unwrap();
        let a = ProtocolConfig::from_legacy_parse("wireguard", json.clone());
        let b = ProtocolConfig::from_legacy_parse("wireguard", json.clone());
        let c = ProtocolConfig::from_legacy_parse("wireguard", serde_json::to_vec(&serde_json::json!({
            "protocol_settings": {"password": "other"},
            "stream_settings": {}
        })).unwrap());
        assert_ne!(a.sig(), 0, "sig must never be zero");
        assert_eq!(a.sig(), b.sig(), "same body -> same sig (dedup)");
        assert_ne!(a.sig(), c.sig(), "different body -> different sig");
        assert_eq!(a.cred_hash(), 0, "opaque blob has no extractable credentials");
        assert_eq!(a.uid(), a.sig(), "uid == sig when cred_hash is 0");
    }
}
