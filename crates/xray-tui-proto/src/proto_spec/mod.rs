use std::borrow::Cow;
use std::num::NonZeroU64;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use smallvec::SmallVec;

use crate::clash::ClashProxy;
use crate::urlx::{HostSpec, RawUrlX, SchemeX};

pub mod common;
pub mod core_mapping;
pub mod endpoint;
pub mod utils;

mod anytls;
mod error;
mod http_client;
mod hysteria1;
mod hysteria2;
mod kinds;
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
pub use endpoint::{ConfigKind, EndpointEssentials, HostKind, ParsedProto, ProtocolEssentials};
pub use error::SupportError;
pub use http_client::HttpClientConfig;
pub use hysteria1::Hysteria1Config;
pub use hysteria2::Hysteria2Config;
pub use kinds::{ParseProtocolKindError, ProtocolKind, SecurityType, TransportType};
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
    ///
    /// No longer raised by the parsers: host *policy* (private/loopback/
    /// link-local/localhost rejection, gated by `allow_private_ips`) moved to
    /// the config layer (`xray-tui-config::import_export::validate_host`) in
    /// T11, which is the single host-policy authority. Kept for API
    /// compatibility; slated for removal with the legacy surface in T23.
    #[error("private/reserved host: {0}")]
    InvalidPrivateHost(Cow<'static, str>),
    /// The protocol is recognized but not yet implemented as a placeholder.
    #[error("protocol not yet implemented: {0}")]
    Unimplemented(&'static str),
}

/// Which proxy core to target for JSON config generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
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

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// Serde for `CoreType` follows the same as_str dialect as the kind enums in
// `kinds.rs` ("xray" / "sing-box") — required so `ProtocolEssentials` can
// derive `Serialize`/`Deserialize` (T3 parse boundary).
impl Serialize for CoreType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CoreType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s)
            .map_err(|_| serde::de::Error::custom(format_args!("invalid core type: '{s}'")))
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

impl ProtocolConfig {
    /// Full parse boundary: scheme dispatch + fallback chain, returning the
    /// complete [`ParsedProto`] (endpoints + protocol essentials) of the
    /// FIRST parser that succeeds.
    ///
    /// The scheme-mapped parser runs first; when it fails with a recoverable
    /// structural error (missing/malformed host, port, userinfo, or unknown),
    /// the chain falls back through `Ss → Ssr → Vmess → Vless → Trojan →
    /// Hysteria2 → Hysteria1`. Unrecoverable errors (unsupported scheme,
    /// promotion/navigation URLs, invalid conf) abort immediately.
    ///
    /// Supersedes the removed `try_parse_detailed`/`ParseResult` (T11): the
    /// endpoints are no longer discarded — callers get them on the returned
    /// [`ParsedProto`].
    ///
    /// # Errors
    ///
    /// If the URL is not a valid proxy URL for any supported protocol.
    pub fn try_parse_proto(raw: &RawUrlX<'_>) -> Result<ParsedProto, ParseError> {
        // Every arm goes through the config's `try_parse_proto` (the parse
        // boundary entry — T4/T5) and keeps the full ParsedProto (endpoints
        // + protocol essentials).
        let r = match raw.schema {
            SchemeX::Vless => VlessConfig::try_parse_proto(raw),
            SchemeX::Trojan => TrojanConfig::try_parse_proto(raw),
            SchemeX::Vmess => VmessConfig::try_parse_proto(raw),
            SchemeX::Hysteria => Hysteria1Config::try_parse_proto(raw),
            SchemeX::Hysteria2 => Hysteria2Config::try_parse_proto(raw),
            SchemeX::SS => SsConfig::try_parse_proto(raw),
            SchemeX::SSR => SsrConfig::try_parse_proto(raw),
            SchemeX::TUIC => TuicConfig::try_parse_proto(raw),
            SchemeX::WireGuard => WireguardConfig::try_parse_proto(raw),
            SchemeX::ShadowTls => ShadowTlsConfig::try_parse_proto(raw),
            SchemeX::Socks => Socks5Config::try_parse_proto(raw),
            SchemeX::Http => HttpClientConfig::try_parse_proto(raw),
            SchemeX::Naive => NaiveConfig::try_parse_proto(raw),
            SchemeX::AnyTLS => AnyTlsConfig::try_parse_proto(raw),
            SchemeX::Warp => {
                // Warp is not directly URL-parsable — no fallback applies.
                Err(ParseError::UnsupportedScheme(raw.schema.clone()))
            }
            SchemeX::Undefined | SchemeX::Https => return Err(ParseError::PromotionUrl),
            ref other => return Err(ParseError::UnsupportedScheme(other.clone())),
        };

        let original_err = match r {
            Ok(parsed) => return Ok(parsed),
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
        SsConfig::try_parse_proto(raw)
            .or_else(|_| SsrConfig::try_parse_proto(raw))
            .or_else(|_| VmessConfig::try_parse_proto(raw))
            .or_else(|_| VlessConfig::try_parse_proto(raw))
            .or_else(|_| TrojanConfig::try_parse_proto(raw))
            .or_else(|_| Hysteria2Config::try_parse_proto(raw))
            .or_else(|_| Hysteria1Config::try_parse_proto(raw))
            .or(Err(original_err))
    }

    /// Reconstruct the share URL for this config given its endpoint.
    ///
    /// Dispatches to the per-config `reconstruct_proto`; placeholder
    /// protocols (no URL format) return
    /// [`ParseError::Unimplemented`].
    ///
    /// # Errors
    ///
    /// If this protocol has no URL format, or the endpoint/config cannot be
    /// rendered.
    pub fn reconstruct_proto(&self, endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        dispatch!(self, reconstruct_proto, endpoint)
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
                    .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
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
                    .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
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

/// Identity computation for protocol configs: pure, deterministic, stateless.
///
/// Crate-private (plain `trait`, no `pub`). The [`ProtoSpec`] trait is sealed
/// through this supertrait — only config types inside this crate implement it.
trait ProtoIdentity {
    fn compute_sig(&self) -> u64;
    fn compute_cred_hash(&self) -> u64;
}

/// Behavioral protocol spec, sealed to this crate via the private
/// [`ProtoIdentity`] supertrait.
///
/// PARSE-CONTRACT MIGRATION (phase A): the parse contract lives on the
/// `*_proto` inherent methods of every config type (`try_parse_proto` /
/// `try_from_clash_proto` / `to_clash_proto` / `reconstruct_proto`), which
/// produce/consume [`ParsedProto`] with the endpoint ([`EndpointEssentials`])
/// split out and [`ProtocolEssentials::config`] carrying only endpoint-free
/// protocol parameters (host-free parse mandate). T4 converted vless/vmess;
/// T4 converted vless/vmess; T5 converted all remaining configs (no config struct carries host/port
/// anymore). This legacy trait is kept as a bridge so `ProtocolConfig`
/// dispatch and the `Proto` consumers in xray-tui-core keep compiling:
/// `try_parse`/`try_from_clash` still work by delegating to the `*_proto`
/// variants and discarding the endpoints; `to_clash`/`reconstruct` return
/// errors because host/port are no longer stored on the config. T11 rewired
/// import/export to the `*_proto` variants (phase D builders take the
/// endpoint separately); the trait itself is slated for removal in T23.
#[allow(private_bounds)] // edition 2024 denies private bounds; deliberate seal
pub trait ProtoSpec: ProtoIdentity {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError>
    where
        Self: Sized;
    /// # Errors
    ///
    /// If internal configuration is invalid.
    fn reconstruct(&self) -> Result<String, ParseError>;
    fn schema(&self) -> SchemeX;
    fn host(&self) -> Option<&HostSpec>;
    fn port(&self) -> Option<u16>;
    fn remarks(&self) -> Option<&str>;
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

/// Build-time overrides applied by config builders at inject time.
///
/// Carries user-facing settings that are not part of the protocol config
/// itself but must be honored when the outbound block is materialized (e.g.
/// the TUI's "skip cert verify" toggle → `tls.insecure`). Tasks 14/15 impls
/// MUST honor [`InjectOptions::skip_cert_verify`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InjectOptions {
    pub skip_cert_verify: bool,
}

/// A protocol config injects its outbound block + stream settings into a core
/// JSON config. `endpoint` supplies host/port for transport/sni fields left
/// unset by the host-free parse mandate (Task 4/5) — impls apply
/// transport.with_host(endpoint.host) / sni defaults at build time, NEVER at
/// parse time. `opts` carries build-time overrides (see [`InjectOptions`])
/// applied at inject time.
///
/// Standalone by design: deliberately NOT a supertrait of [`ProtoSpec`] (no
/// coupling). Task 6 adds the trait plus the [`ProtocolConfig`] dispatch; the
/// per-config implementations land in Tasks 14/15 (stubs in `inject_stub.rs`
/// error with [`SupportError::UnsupportedProtocol`] until then).
pub trait InjectToCoreConf {
    /// # Errors
    ///
    /// If this protocol cannot be injected into the requested core (or, today,
    /// until Tasks 14/15 land the real impls — always).
    fn inject_to(
        &self,
        core_conf: &mut serde_json::Value,
        core_type: CoreType,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError>;
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

impl ProtoIdentity for ProtocolConfig {
    fn compute_sig(&self) -> u64 {
        dispatch!(self, compute_sig)
    }
    fn compute_cred_hash(&self) -> u64 {
        dispatch!(self, compute_cred_hash)
    }
}

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
        // T11: the full parse boundary is `try_parse_proto`; this legacy
        // config-only bridge discards the parsed endpoints.
        Self::try_parse_proto(raw).map(|p| p.protocol.config)
    }
}

impl InjectToCoreConf for ProtocolConfig {
    fn inject_to(
        &self,
        core_conf: &mut serde_json::Value,
        core_type: CoreType,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError> {
        // Reuses the existing `dispatch!` macro — it already forwards extra
        // arguments (`core_conf`, `core_type`, `endpoint`, `opts`) to each
        // variant.
        dispatch!(self, inject_to, core_conf, core_type, endpoint, opts)
    }
}

// ── Identity container ──────────────────────────────────────────────────

/// Materialized identity of a [`Proto`]: `(sig, cred_hash)` with `sig` never
/// zero.
#[derive(Debug, Clone, Copy)]
struct Identity {
    sig: NonZeroU64,
    cred_hash: u64,
}

/// A [`ProtocolConfig`] paired with its lazily-materialized identity.
///
/// `sig`/`cred_hash`/`uid` are computed once (atomically, on first access) and
/// cached. Serializes byte-identical to the wrapped [`ProtocolConfig`]
/// (`spec_blob` format), so deserializing a stored spec produces an identical
/// [`Proto`] whose identity starts deferred (empty `OnceLock`).
#[derive(Debug)]
pub struct Proto {
    config: ProtocolConfig,
    identity: std::sync::OnceLock<Identity>,
}

impl Proto {
    #[must_use]
    pub const fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            // Empty lock == deferred identity; materialized on first access.
            identity: std::sync::OnceLock::new(),
        }
    }

    /// Materialize the identity cache on first access. Race-safe by
    /// construction: `get_or_init` runs the closure at most once and stores a
    /// single deterministic value.
    fn materialize(&self) -> &Identity {
        self.identity.get_or_init(|| Identity {
            sig: NonZeroU64::new(self.config.compute_sig()).unwrap_or(NonZeroU64::MIN),
            cred_hash: self.config.compute_cred_hash(),
        })
    }

    #[must_use]
    pub fn sig(&self) -> u64 {
        self.materialize().sig.get()
    }

    #[must_use]
    pub fn cred_hash(&self) -> u64 {
        self.materialize().cred_hash
    }

    #[must_use]
    pub fn uid(&self) -> u64 {
        self.sig() ^ self.cred_hash()
    }

    #[must_use]
    pub const fn config(&self) -> &ProtocolConfig {
        &self.config
    }

    #[must_use]
    pub fn into_config(self) -> ProtocolConfig {
        self.config
    }

    /// Seed the identity cache (tests only — lets tests assert no recompute).
    #[cfg(test)]
    fn set_identity(&self, identity: Identity) {
        _ = self.identity.set(identity);
    }
}

impl ProtoIdentity for Proto {
    fn compute_sig(&self) -> u64 {
        <ProtocolConfig as ProtoIdentity>::compute_sig(&self.config)
    }
    fn compute_cred_hash(&self) -> u64 {
        <ProtocolConfig as ProtoIdentity>::compute_cred_hash(&self.config)
    }
}

impl ProtoSpec for Proto {
    fn reconstruct(&self) -> Result<String, ParseError> {
        dispatch!(&self.config, reconstruct)
    }
    fn schema(&self) -> SchemeX {
        dispatch!(&self.config, schema)
    }
    fn host(&self) -> Option<&HostSpec> {
        dispatch!(&self.config, host)
    }
    fn port(&self) -> Option<u16> {
        dispatch!(&self.config, port)
    }
    fn remarks(&self) -> Option<&str> {
        dispatch!(&self.config, remarks)
    }
    fn security(&self) -> Option<&SecurityConfig> {
        dispatch!(&self.config, security)
    }
    fn transport_type(&self) -> Option<&str> {
        dispatch!(&self.config, transport_type)
    }
    fn security_type(&self) -> Option<&str> {
        dispatch!(&self.config, security_type)
    }
    fn country_flags(&self) -> SmallVec<[crate::urlx::TinyText; 4]> {
        dispatch!(&self.config, country_flags)
    }
    fn to_json_config(&self, core: CoreType) -> Result<serde_json::Value, ProtoSpecError> {
        dispatch!(&self.config, to_json_config, core)
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        ProtocolConfig::try_from_clash(proxy).map(Self::new)
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        dispatch!(&self.config, to_clash)
    }

    /// # Errors
    ///
    /// If the URL is not a valid proxy URL for any supported protocol.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        ProtocolConfig::try_parse(raw).map(Self::new)
    }
}

impl Serialize for Proto {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.config.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Proto {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::new(ProtocolConfig::deserialize(deserializer)?))
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
    fn transport_type(&self) -> Option<&str> {
        None
    }
    fn to_json_config(&self, _core: CoreType) -> Result<serde_json::Value, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(self.proto_name.clone()))
    }
}

impl InjectToCoreConf for PlaceholderConfig {
    fn inject_to(
        &self,
        _core_conf: &mut Value,
        core_type: CoreType,
        _endpoint: Option<&EndpointEssentials>,
        _opts: InjectOptions,
    ) -> Result<(), SupportError> {
        // Redirect / TProxy / Mixed share this one type; the variant is carried
        // in `proto_name` ("redirect" / "tproxy" / "mixed" as written by
        // `from_legacy_parse` and `try_parse_proto`), so it is the kind string.
        //
        // Both cores error: sing-box has NO redirect/tproxy/mixed OUTBOUND
        // type (option/redir.go is inbound-only; the design doc §7 raw
        // settings_json injection predates the runtime contract and would
        // yield an outbound without a `type` key that sing-box rejects at
        // config load — "unknown outbound type"). Config validity is enforced
        // at build time (AGENTS.md decision 2): no raw body can ever be a
        // valid outbound, so these stay unsupported until a real shape exists.
        Err(SupportError::UnsupportedProtocol(
            self.proto_name.clone(),
            core_type,
        ))
    }
}

impl PlaceholderConfig {
    /// Construct a placeholder wrapping an opaque legacy JSON body.
    #[must_use]
    pub const fn new(proto_name: String, settings_json: Vec<u8>) -> Self {
        Self {
            proto_name,
            settings_json,
        }
    }

    /// Wrap this placeholder as an endpoint-less [`ParsedProto`] — the parse
    /// boundary entry for orphan protocols (Redirect/TProxy/Mixed) that have
    /// no URL format and no endpoint. `endpoints` EMPTY is legal for these.
    ///
    /// The kind is derived from `proto_name`; unknown names fall back to
    /// [`ProtocolKind::Mixed`], the same backward-compat rule as
    /// [`ProtocolConfig::from_legacy_parse`].
    #[must_use]
    pub fn try_parse_proto(&self) -> ParsedProto {
        let (proto_kind, config) = match self.proto_name.to_lowercase().as_str() {
            "redirect" => (
                ProtocolKind::Redirect,
                ProtocolConfig::Redirect(self.clone()),
            ),
            "tproxy" => (ProtocolKind::TProxy, ProtocolConfig::TProxy(self.clone())),
            _ => (ProtocolKind::Mixed, ProtocolConfig::Mixed(self.clone())),
        };
        ParsedProto {
            endpoints: vec![],
            protocol: ProtocolEssentials {
                proto_kind,
                config_type: ConfigKind::ShareUrl,
                core_type: core_mapping::resolve_core(proto_kind, None, None),
                config,
            },
        }
    }

    /// Placeholder protocols have no Clash representation — always an error
    /// (mirrors the legacy trait default).
    ///
    /// # Errors
    ///
    /// Always — placeholder protocols have no Clash format.
    pub fn try_from_clash_proto(_proxy: &ClashProxy) -> Result<ParsedProto, ParseError> {
        Err(ParseError::Unknown(
            "clash parsing not implemented for this protocol".into(),
        ))
    }

    /// Placeholder protocols have no Clash representation — always an error
    /// (mirrors the legacy trait default).
    ///
    /// # Errors
    ///
    /// Always — placeholder protocols have no Clash format.
    pub fn to_clash_proto(
        &self,
        _endpoint: &EndpointEssentials,
    ) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "clash serialization not implemented for this protocol".into(),
        ))
    }

    /// Placeholder protocols have no URL format — always an error (mirrors
    /// the legacy [`Self::reconstruct`]).
    ///
    /// # Errors
    ///
    /// Always — placeholder protocols have no URL format.
    pub fn reconstruct_proto(&self, _endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        Err(ParseError::Unimplemented("placeholder protocol"))
    }
}

impl ProtoIdentity for PlaceholderConfig {
    fn compute_sig(&self) -> u64 {
        // Opaque legacy blob: we cannot decompose semantic fields reliably,
        // so the sig is a deterministic rapidhash over the ENTIRE body
        // (proto_name + settings_json). Same body -> same uid (dedup); never
        // zero (mapped to NonZeroU64::MIN by Proto::materialize).
        //
        // NOTE: the hashed body INCLUDES the volatile `remarks` field for
        // placeholder-scheme profiles (set_legacy_fields writes it into the
        // settings JSON). Renaming a profile's remark therefore changes its
        // uid, so a subscription refresh treats the row as new and duplicates
        // it. This is the mandated whole-body-hash design for opaque configs —
        // the remark is intentionally part of the identity; do NOT special-case
        // it out of the hash here.
        use rapidhash::v3::{DEFAULT_RAPID_SECRETS, RapidStreamHasherV3};
        let mut hasher = RapidStreamHasherV3::new(&DEFAULT_RAPID_SECRETS);
        hasher.write(self.proto_name.as_bytes());
        hasher.write(&self.settings_json);
        hasher.finish()
    }

    fn compute_cred_hash(&self) -> u64 {
        // Opaque blob has no extractable credentials.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_try_parse_proto_emits_endpointless_parsed_proto() {
        // Placeholder configs wrap into the parse boundary as orphan
        // protocols: empty endpoints, kind derived from proto_name.
        let blob = serde_json::json!({
            "protocol_settings": {"password": "sekrit"},
            "stream_settings": {}
        });
        let json = serde_json::to_vec(&blob).unwrap();

        let redirect = PlaceholderConfig::new("redirect".into(), json.clone());
        let parsed = redirect.try_parse_proto();
        assert!(parsed.endpoints.is_empty(), "orphan protocol: no endpoint");
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Redirect);
        assert_eq!(parsed.protocol.config_type, ConfigKind::ShareUrl);
        assert_eq!(parsed.protocol.core_type, CoreType::SingBox);
        assert_eq!(
            parsed.protocol.config,
            ProtocolConfig::Redirect(redirect.clone())
        );

        let tproxy = PlaceholderConfig::new("tproxy".into(), json.clone());
        let parsed = tproxy.try_parse_proto();
        assert!(parsed.endpoints.is_empty());
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::TProxy);
        assert_eq!(parsed.protocol.config, ProtocolConfig::TProxy(tproxy));

        // Unknown proto_name falls back to Mixed (from_legacy_parse rule).
        let mixed = PlaceholderConfig::new("wireguard".into(), json);
        let parsed = mixed.try_parse_proto();
        assert!(parsed.endpoints.is_empty());
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Mixed);
        assert_eq!(parsed.protocol.config, ProtocolConfig::Mixed(mixed));

        // The *_proto clash/reconstruct paths mirror the legacy errors.
        assert!(
            PlaceholderConfig::try_from_clash_proto(&ClashProxy::Direct(
                crate::clash::ClashDirect {
                    name: "d".into(),
                    udp: None
                }
            ))
            .is_err()
        );
        assert!(
            redirect
                .to_clash_proto(&EndpointEssentials::new("x", 1))
                .is_err()
        );
        assert!(
            redirect
                .reconstruct_proto(&EndpointEssentials::new("x", 1))
                .is_err()
        );
    }

    #[test]
    fn placeholder_config_sig_is_deterministic_nonzero_body_hash() {
        let blob = serde_json::json!({
            "protocol_settings": {"password": "sekrit"},
            "stream_settings": {}
        });
        let json = serde_json::to_vec(&blob).unwrap();
        let a = Proto::new(ProtocolConfig::from_legacy_parse("wireguard", json.clone()));
        let b = Proto::new(ProtocolConfig::from_legacy_parse("wireguard", json));
        let c = Proto::new(ProtocolConfig::from_legacy_parse(
            "wireguard",
            serde_json::to_vec(&serde_json::json!({
                "protocol_settings": {"password": "other"},
                "stream_settings": {}
            }))
            .unwrap(),
        ));
        assert_ne!(a.sig(), 0, "sig must never be zero");
        assert_eq!(a.sig(), b.sig(), "same body -> same sig (dedup)");
        assert_ne!(a.sig(), c.sig(), "different body -> different sig");
        assert_eq!(
            a.cred_hash(),
            0,
            "opaque blob has no extractable credentials"
        );
        assert_eq!(a.uid(), a.sig(), "uid == sig when cred_hash is 0");
    }

    #[test]
    fn proto_serde_roundtrip_byte_identical_to_config() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let config = ProtocolConfig::try_parse(&RawUrlX::from(url)).unwrap();
        let proto = Proto::new(config.clone());
        assert_eq!(
            serde_json::to_string(&proto).unwrap(),
            serde_json::to_string(&config).unwrap(),
            "Proto must serialize byte-identical to ProtocolConfig (spec_blob format)"
        );
        let bytes = serde_json::to_vec(&proto).unwrap();
        let reparsed: Proto = serde_json::from_slice(&bytes).unwrap();
        assert!(
            reparsed.identity.get().is_none(),
            "deserialized Proto must start with deferred identity (empty OnceLock)"
        );
        assert_eq!(
            serde_json::from_slice::<ProtocolConfig>(&bytes).unwrap(),
            config,
            "Proto bytes must decode to the same ProtocolConfig"
        );
    }

    #[test]
    fn proto_materialization_consistency() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let config = ProtocolConfig::try_parse(&RawUrlX::from(url)).unwrap();
        let proto = Proto::new(config);
        let sig = proto.sig();
        let cred_hash = proto.cred_hash();
        assert_ne!(sig, 0, "sig must never be zero");
        assert_eq!(proto.uid(), sig ^ cred_hash, "uid == sig ^ cred_hash");
        assert_eq!(proto.sig(), sig, "sig is stable across calls");
        assert_eq!(
            proto.cred_hash(),
            cred_hash,
            "cred_hash is stable across calls"
        );
        assert_eq!(proto.uid(), sig ^ cred_hash, "uid is stable across calls");
        assert!(
            proto.identity.get().is_some(),
            "identity must materialize on first access"
        );
    }

    #[test]
    fn proto_set_identity_seeds_cache() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let config = ProtocolConfig::try_parse(&RawUrlX::from(url)).unwrap();
        let proto = Proto::new(config);
        let seeded_sig = std::num::NonZeroU64::new(12_345).unwrap();
        let seeded_cred_hash = 67_890;
        proto.set_identity(Identity {
            sig: seeded_sig,
            cred_hash: seeded_cred_hash,
        });
        assert_eq!(proto.sig(), 12_345, "seeded sig returned without recompute");
        assert_eq!(
            proto.cred_hash(),
            67_890,
            "seeded cred_hash returned without recompute"
        );
        assert_eq!(proto.uid(), 0x3039 ^ 0x1_0932, "uid == sig ^ cred_hash");
    }

    #[test]
    fn core_type_display_matches_as_str() {
        assert_eq!(CoreType::Xray.to_string(), "xray");
        assert_eq!(CoreType::SingBox.to_string(), "sing-box");
        assert_eq!(CoreType::Xray.as_str(), "xray");
        assert_eq!(CoreType::SingBox.as_str(), "sing-box");
    }

    #[test]
    fn core_type_from_str_round_trip() {
        for original in [CoreType::Xray, CoreType::SingBox] {
            let parsed: CoreType = original.to_string().parse().expect("round-trip parse");
            assert_eq!(parsed, original);
        }
        // Accepted aliases.
        assert_eq!("xray-core".parse::<CoreType>(), Ok(CoreType::Xray));
        assert_eq!("singbox".parse::<CoreType>(), Ok(CoreType::SingBox));
        assert_eq!("SING-BOX".parse::<CoreType>(), Ok(CoreType::SingBox));
        // Unknown strings must error.
        assert!("auto".parse::<CoreType>().is_err());
        assert!("".parse::<CoreType>().is_err());
    }

    // ── InjectToCoreConf dispatch (Task 6) ────────────────────────────────
    // NOTE: these stub-behavior tests are replaced in T14/T15 when the real
    // per-config `inject_to` impls land.

    fn vless_config() -> VlessConfig {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws";
        let parsed = VlessConfig::try_parse_proto(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"));
        match parsed.protocol.config {
            ProtocolConfig::Vless(c) => c,
            other => panic!("expected VlessConfig, got {other:?}"),
        }
    }

    /// Parse a share URL into its `ProtocolConfig` (endpoints discarded).
    fn config_from_url(url: &str) -> ProtocolConfig {
        ProtocolConfig::try_parse(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    /// One `ProtocolConfig` per dispatch variant, paired with the kind string
    /// its dispatch arm must route to.
    fn all_variants_with_kinds() -> Vec<(ProtocolConfig, &'static str)> {
        use crate::clash::{ClashProxy, ClashSsh, ClashTailscale, ClashTor};

        let placeholder = |name: &str| {
            ProtocolConfig::from_legacy_parse(name, serde_json::to_vec(&json!({})).unwrap())
        };
        let tor = TorConfig::try_from_clash_proto(&ClashProxy::Tor(ClashTor {
            name: "tor-node".into(),
            server: "127.0.0.1".into(),
            port: 9050,
        }))
        .expect("tor clash parse")
        .protocol
        .config;
        let ssh = SshConfig::try_from_clash_proto(&ClashProxy::Ssh(ClashSsh {
            name: "ssh-box".into(),
            server: "example.com".into(),
            port: 22,
            user: "root".into(),
            password: Some("sekrit".into()),
            private_key: None,
            private_key_path: Some("/home/user/.ssh/id_ed25519".into()),
            host_key: Some(vec!["ssh-ed25519 AAA".into()]),
            host_key_algorithms: Some(vec!["ssh-ed25519".into()]),
            client_version: Some("SSH-2.0-myclient".into()),
        }))
        .expect("ssh clash parse")
        .protocol
        .config;
        let tailscale =
            TailscaleConfig::try_from_clash_proto(&ClashProxy::Tailscale(ClashTailscale {
                name: "ts-node".into(),
                server: "100.64.0.1".into(),
                port: 100,
                hostname: "node1".into(),
                auth_key: Some("tskey-auth-abc".into()),
                control_url: Some("https://control.example.com".into()),
                state_dir: Some("/var/lib/tailscale".into()),
                ephemeral: true,
                accept_routes: true,
                exit_node: Some("100.64.0.2".into()),
                exit_node_allow_lan_access: Some(true),
            }))
            .expect("tailscale clash parse")
            .protocol
            .config;

        vec![
            (
                config_from_url(
                    "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws",
                ),
                "vless",
            ),
            (
                config_from_url(
                    "vmess://eyJhZGQiOiIxOTIuMjAwLjE2MC4xNiIsImFpZCI6IjAiLCJhbHBuIjoiIiwiZnAiOiIiLCJob3N0IjoiIiwiaWQiOiI5YjRjMmVkYS0zNDFlLTQ4OGYtYTNiMi0xZGM3MTZiOWYzNmEiLCJpbnNlY3VyZSI6IjEiLCJuZXQiOiJ3cyIsInBhdGgiOiIvIiwicG9ydCI6Ijg0NDMiLCJwcyI6IkBDbG91ZENpdHl5Iiwic2N5IjoiYXV0byIsInNuaSI6InN0ZWFtLmF2YWFhYWwuaXIiLCJ0bHMiOiJ0bHMiLCJ0eXBlIjoiLS0tIiwidiI6IjIifQ==",
                ),
                "vmess",
            ),
            (
                config_from_url(
                    "trojan://humanity@172.64.152.23:443?security=tls&type=ws&path=/assignment&sni=www.creationlong.org",
                ),
                "trojan",
            ),
            (
                config_from_url(
                    "hy2://linux.do@[2a01:4f9:4b:f378::1]:13599?security=tls&insecure=1&sni=www.bing.com",
                ),
                "hy2",
            ),
            (
                // AEAD cipher so the xray arm's build-time validation passes.
                config_from_url("ss://YWVzLTI1Ni1nY206cGFzcw@1.2.3.4:8388"),
                "ss",
            ),
            (
                config_from_url(
                    "ssr://ZXhhbXBsZS5jb206NDQzOm9yaWdpbjpyYzQtbWQ1OnBsYWluOmNHRnpjM2R2Y21RLz9ncm91cD1WR1Z6ZEVkeWIzVncmcmVtYXJrcz1WR1Z6ZEZObGNuWmxjZw",
                ),
                "ssr",
            ),
            (
                config_from_url(
                    "tuic://36106e0f-4d9a-470b-a3fd-535f3b7a1e92:dongtaiwang.com@5.178.101.117:30006?congestion_control=cubic&udp_relay_mode=native&alpn=h3",
                ),
                "tuic",
            ),
            (
                config_from_url(
                    "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280",
                ),
                "wireguard",
            ),
            (config_from_url("socks://user:pass@1.2.3.4:1080"), "socks"),
            (config_from_url("http://user:pass@1.2.3.4:8080"), "http"),
            (
                config_from_url("naive+https://user:pass@example.com:443"),
                "naive",
            ),
            (
                config_from_url("anytls://1.2.3.4:8080?password=secret"),
                "any-tls",
            ),
            (
                config_from_url(
                    "shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com",
                ),
                "shadow-tls",
            ),
            (tor, "tor"),
            (ssh, "ssh"),
            (tailscale, "tailscale"),
            (
                config_from_url(
                    "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com",
                ),
                "hy",
            ),
            (placeholder("redirect"), "redirect"),
            (placeholder("tproxy"), "tproxy"),
            (placeholder("mixed"), "mixed"),
        ]
    }

    /// Core whose dispatch arm reports `UnsupportedProtocol` for a variant:
    /// the sing-box-only protocols (real sing-box `inject_to` landed in T15)
    /// have no Xray shape and error for Xray permanently.
    fn reject_core_for(config: &ProtocolConfig) -> CoreType {
        match config {
            ProtocolConfig::Vless(_)
            | ProtocolConfig::Vmess(_)
            | ProtocolConfig::Trojan(_)
            | ProtocolConfig::Hysteria2(_)
            | ProtocolConfig::Ss(_)
            | ProtocolConfig::Wireguard(_)
            | ProtocolConfig::Socks(_)
            | ProtocolConfig::Http(_) => CoreType::SingBox,
            _ => CoreType::Xray,
        }
    }

    /// Cores that can build an outbound for the variant (T14 xray + T15
    /// sing-box for the shared set; sing-box only for the sing-box-native set).
    fn supported_core(config: &ProtocolConfig) -> CoreType {
        match config {
            ProtocolConfig::Vless(_)
            | ProtocolConfig::Vmess(_)
            | ProtocolConfig::Trojan(_)
            | ProtocolConfig::Hysteria2(_)
            | ProtocolConfig::Ss(_)
            | ProtocolConfig::Wireguard(_)
            | ProtocolConfig::Socks(_)
            | ProtocolConfig::Http(_) => CoreType::Xray,
            _ => CoreType::SingBox,
        }
    }

    #[test]
    fn dispatch_routes_to_variant() {
        // vless: both the Xray (T14) and sing-box (T15) arms land.
        let vless = ProtocolConfig::Vless(vless_config());
        let endpoint = EndpointEssentials::new("1.2.3.4", 443);
        vless
            .inject_to(
                &mut json!({}),
                CoreType::Xray,
                Some(&endpoint),
                InjectOptions::default(),
            )
            .expect("xray inject must succeed in T14");
        vless
            .inject_to(
                &mut json!({}),
                CoreType::SingBox,
                Some(&endpoint),
                InjectOptions::default(),
            )
            .expect("sing-box inject must succeed in T15");

        // tuic: real sing-box shape since T15; an orphan (no endpoint) is
        // rejected with the missing-server error, not UnsupportedProtocol.
        let tuic = config_from_url(
            "tuic://36106e0f-4d9a-470b-a3fd-535f3b7a1e92:dongtaiwang.com@5.178.101.117:30006?congestion_control=cubic&udp_relay_mode=native&alpn=h3",
        );
        let err = tuic
            .inject_to(
                &mut json!({}),
                CoreType::SingBox,
                None,
                InjectOptions::default(),
            )
            .expect_err("orphan tuic must be rejected by the real impl");
        assert!(
            matches!(err, SupportError::MissingField("server", "tuic")),
            "expected MissingField(server), got {err:?}"
        );
    }

    #[test]
    fn endpoint_param_accepted() {
        // Orphan configs (no endpoint) cannot build an xray outbound that
        // needs a server — the impl must reject, never panic.
        let vless = ProtocolConfig::Vless(vless_config());
        let endpoint = EndpointEssentials::new("1.2.3.4", 443);
        vless
            .inject_to(
                &mut json!({}),
                CoreType::Xray,
                Some(&endpoint),
                InjectOptions::default(),
            )
            .expect("endpoint-bearing inject must succeed");
        let err = vless
            .inject_to(
                &mut json!({}),
                CoreType::Xray,
                None,
                InjectOptions::default(),
            )
            .expect_err("orphan inject must be rejected");
        assert!(matches!(err, SupportError::MissingField("server", "vless")));
    }

    #[test]
    fn placeholder_rejected_on_both_cores() {
        // sing-box has no redirect/tproxy/mixed OUTBOUND type, so the raw
        // settings_json can never be a valid outbound — both cores refuse at
        // build time (config validity enforced, AGENTS.md decision 2).
        let raw = json!({
            "protocol_settings": { "redirect_address": "1.2.3.4" },
            "stream_settings": {},
        });
        let placeholder = PlaceholderConfig::new(
            "redirect".into(),
            serde_json::to_vec(&raw).expect("serialize"),
        );
        for core in [CoreType::SingBox, CoreType::Xray] {
            let mut conf = json!({});
            let err = placeholder
                .inject_to(&mut conf, core, None, InjectOptions::default())
                .expect_err("placeholder must be rejected on both cores");
            assert!(matches!(
                &err,
                SupportError::UnsupportedProtocol(kind, got) if kind == "redirect" && *got == core
            ));
        }
    }

    #[test]
    fn all_variants_route_to_their_kind_string() {
        let variants = all_variants_with_kinds();
        assert_eq!(variants.len(), 20, "all 20 dispatch arms must be covered");
        let endpoint = EndpointEssentials::new("1.2.3.4", 443);
        // Since T15 every non-placeholder variant has a real sing-box shape:
        // with an endpoint, dispatch must succeed for all 17 (tor/tailscale
        // ignore the endpoint). The redirect/tproxy/mixed placeholders have
        // NO outbound shape on either core (sing-box has no such outbound
        // type) and are rejected at build time.
        for (config, expected) in &variants {
            if matches!(expected.as_ref(), "redirect" | "tproxy" | "mixed") {
                continue;
            }
            let core = supported_core(config);
            config
                .inject_to(
                    &mut json!({}),
                    core,
                    Some(&endpoint),
                    InjectOptions::default(),
                )
                .unwrap_or_else(|e| panic!("{expected:?} must build via {core:?}, got {e:?}"));
        }
        // The sing-box-only variants still report their kind string via the
        // Xray arm (no Xray shape, permanently).
        let mut kinds = Vec::new();
        for (config, expected) in &variants {
            if supported_core(config) == CoreType::SingBox {
                match config.inject_to(
                    &mut json!({}),
                    CoreType::Xray,
                    Some(&endpoint),
                    InjectOptions::default(),
                ) {
                    Err(SupportError::UnsupportedProtocol(kind, _)) => {
                        assert_eq!(
                            kind.as_str(),
                            *expected,
                            "dispatch arm for {config:?} must report kind {expected:?}"
                        );
                        kinds.push(kind);
                    }
                    other => panic!(
                        "expected UnsupportedProtocol for {config:?} via Xray, got {other:?}"
                    ),
                }
            }
        }
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(
            kinds.len(),
            12,
            "the 12 sing-box-only variants must report distinct kind strings"
        );
    }
}
