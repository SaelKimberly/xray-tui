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
use serde_json::{Value, json};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText};

use super::ProtoIdentity;
use super::common::{
    SecurityConfig, TransportConfig, security_force_insecure, to_xray_stream_settings,
};
use super::core_mapping;
use super::utils;
use super::{
    ConfigKind, CoreType, EndpointEssentials, InjectOptions, InjectToCoreConf, ParseError,
    ParsedProto, ProtoSpec, ProtocolConfig, ProtocolEssentials, ProtocolKind, SupportError,
};
use crate::clash::{ClashProxy, ClashSS};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_to_endpoint, host_kind_for};

/// Shadowsocks protocol configuration — the identity payload (sans host/port).
///
/// The endpoint (server host/port) lives in [`EndpointEssentials`] on the
/// [`ParsedProto`] boundary; this struct only carries endpoint-free protocol
/// parameters, so the same config pointed at different servers shares one
/// identity.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct SsConfig {
    pub method: TinyText,
    pub password: String,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
    pub plugin: Option<TinyText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<HashMap<String, String>>,
}

impl SsConfig {
    /// Parse a Shadowsocks URL into the parse boundary: [`ParsedProto`] with
    /// the endpoint essentials (host/port) split out and the identity payload
    /// ([`ProtocolEssentials::config`]) holding only endpoint-free protocol
    /// parameters.
    ///
    /// Supports three formats:
    /// 1. SIP002: `base64url(method:password)@host:port` (has `@`, hostport present)
    /// 2. Legacy QR: `base64(method:password@host:port)` (no `@` in URL, hostport absent)
    /// 3. Plain: `method:password@host:port` (base64 decode fails but `@` present)
    ///
    /// `decode_base64` tolerates trailing annotation text/emoji (Telegram pattern)
    /// and accepts both URL-safe and standard base64 alphabets.
    ///
    /// The protocol kind is cipher-aware: `2022-blake3-*` methods route to
    /// [`ProtocolKind::Shadowsocks2022`] and the core is resolved with the
    /// method ([`core_mapping::resolve_core`]) so legacy ciphers route to
    /// sing-box and AEAD/2022 ciphers to xray-core.
    pub fn try_parse_proto(raw: &RawUrlX<'_>) -> Result<ParsedProto, ParseError> {
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

        let (parsed_host, parsed_port_spec) = utils::parse_hostport(&hostport)?;
        let parsed_port = parsed_port_spec
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        // Endpoint essentials: host/port live here, never in the config payload.
        let mut endpoint = EndpointEssentials::new(parsed_host.to_str().into_owned(), parsed_port);
        endpoint.host_type = host_kind_for(&parsed_host);
        if parsed_port_spec.length() > 1 {
            endpoint.ports = parsed_port_spec.iter().collect();
        }

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

        // Cipher-aware kind + core: the one config where resolve_core's
        // ss_method argument matters.
        let proto_kind = proto_kind_for_method(method);
        let config = Self {
            method: TinyText::from(method),
            password: password.to_string(),
            security: SecurityConfig::default(),
            remarks,
            plugin,
            plugin_opts,
        };
        Ok(ParsedProto {
            endpoints: vec![endpoint],
            protocol: ProtocolEssentials {
                proto_kind,
                config_type: ConfigKind::ShareUrl,
                core_type: core_mapping::resolve_core(proto_kind, None, Some(method)),
                config: ProtocolConfig::Ss(config),
            },
        })
    }

    /// Rebuild the share URL from this endpoint-free config plus the endpoint
    /// essentials. Endpoint host/port come from `endpoint`.
    pub fn reconstruct_proto(&self, endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        let userinfo = format!("{}:{}", self.method, self.password);
        let encoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(userinfo.as_bytes());
        let host = endpoint.host.as_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", endpoint.port)
        } else {
            format!("{host}:{}", endpoint.port)
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
}

impl SsConfig {
    /// Serialize this endpoint-free config plus the endpoint to a Clash proxy
    /// entry. Endpoint host/port are taken from `endpoint`.
    pub fn to_clash_proto(
        &self,
        endpoint: &EndpointEssentials,
    ) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Shadowsocks(ClashSS {
            name,
            server: endpoint.host.clone(),
            port: endpoint.port,
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

    /// Parse a Clash proxy entry into the parse boundary: `server`/`port`
    /// become the endpoint essentials; the config payload is endpoint-free.
    /// The kind is derived from the Clash `cipher` method (cipher-aware).
    pub fn try_from_clash_proto(proxy: &ClashProxy) -> Result<ParsedProto, ParseError> {
        match proxy {
            ClashProxy::Shadowsocks(c) => {
                let method = TinyText::from(c.cipher.as_str());
                let proto_kind = proto_kind_for_method(&method);
                let config = Self {
                    method,
                    password: c.password.clone(),
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
                };
                Ok(ParsedProto {
                    endpoints: vec![clash_to_endpoint(&c.server, c.port)],
                    protocol: ProtocolEssentials {
                        proto_kind,
                        config_type: ConfigKind::ShareUrl,
                        core_type: core_mapping::resolve_core(proto_kind, None, Some(&c.cipher)),
                        config: ProtocolConfig::Ss(config),
                    },
                })
            }
            _ => Err(ParseError::Unknown(
                "expected shadowsocks clash proxy".into(),
            )),
        }
    }
}

/// Cipher-aware Shadowsocks kind: `2022-blake3-*` methods are
/// [`ProtocolKind::Shadowsocks2022`], everything else is
/// [`ProtocolKind::Shadowsocks`].
fn proto_kind_for_method(method: &str) -> ProtocolKind {
    if method.starts_with("2022-blake3-") {
        ProtocolKind::Shadowsocks2022
    } else {
        ProtocolKind::Shadowsocks
    }
}

/// Legacy [`ProtoSpec`] bridge — kept so `ProtocolConfig` dispatch (and the
/// `Proto` consumer in xray-tui-core) compile unchanged.
///
/// DEGRADED PATH (documented): `try_parse`/`try_from_clash` still work by
/// delegating to the `*_proto` variants and discarding the parsed endpoints;
/// `to_clash`/`reconstruct` return errors because the config no longer stores
/// host/port. Import/export rewires to the `*_proto` variants in T11 (phase D
/// builders take the endpoint separately).
impl ProtoSpec for SsConfig {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let parsed = Self::try_parse_proto(raw)?;
        match parsed.protocol.config {
            ProtocolConfig::Ss(config) => Ok(config),
            // Parser invariant: an ss URL always yields an SsConfig.
            _ => Err(ParseError::Unknown(
                "ss URL parsed to a non-ss config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::reconstruct_proto`] with the endpoint.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::InvalidHost(
            "ss config no longer stores host/port; use SsConfig::reconstruct_proto(endpoint)"
                .into(),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::SS
    }

    /// `None` — the endpoint host moved to [`EndpointEssentials`] (T5).
    fn host(&self) -> Option<&HostSpec> {
        None
    }

    /// `None` — the endpoint port moved to [`EndpointEssentials`] (T5).
    fn port(&self) -> Option<u16> {
        None
    }

    fn remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    fn transport_type(&self) -> Option<&str> {
        None
    }

    /// # Errors
    ///
    /// If the Clash proxy doesn't match this protocol type.
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        let parsed = Self::try_from_clash_proto(proxy)?;
        match parsed.protocol.config {
            ProtocolConfig::Ss(config) => Ok(config),
            _ => Err(ParseError::Unknown(
                "ss clash proxy parsed to a non-ss config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::to_clash_proto`] with the endpoint.
    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "ss config no longer stores host/port; use SsConfig::to_clash_proto(endpoint)".into(),
        ))
    }
}

impl ProtoIdentity for SsConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"ss");
        // Endpoint (host/port) intentionally absent from the identity — it
        // lives on the ParsedProto boundary, never in the config payload (T5).
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

impl InjectToCoreConf for SsConfig {
    fn inject_to(
        &self,
        core_conf: &mut Value,
        core_type: CoreType,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError> {
        match core_type {
            CoreType::Xray => self.inject_xray(core_conf, endpoint, opts),
            CoreType::SingBox => self.inject_singbox(core_conf, endpoint, opts),
        }
    }
}

impl SsConfig {
    /// xray-core outbound for this config, ported field-by-field from the old
    /// xray builder's `Protocol::Shadowsocks | Protocol::Shadowsocks2022` arm.
    /// xray-core's `CipherType` enum only covers AEAD + 2022-blake3; refusing
    /// here prevents the core from dying on startup with "unknown cipher
    /// method" (build-time validation).
    fn inject_xray(
        &self,
        core_conf: &mut Value,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError> {
        let Some(ep) = endpoint else {
            return Err(SupportError::MissingField("server", "ss"));
        };
        if !core_mapping::xray_supports_ss_method(self.method.as_str()) {
            return Err(SupportError::Config(format!(
                "Shadowsocks cipher '{}' is not supported by xray-core; \
                 supported: aes-128-gcm, aes-256-gcm, chacha20-poly1305, \
                 xchacha20-poly1305, 2022-blake3-*",
                self.method.as_str()
            )));
        }
        let security = security_force_insecure(&self.security, opts.skip_cert_verify);
        let stream = to_xray_stream_settings(&security, &TransportConfig::Tcp);
        *core_conf = json!({
            "tag": "proxy",
            "protocol": "shadowsocks",
            "settings": {
                "servers": [{
                    "address": ep.host,
                    "port": ep.port,
                    "method": self.method.as_str(),
                    "password": self.password
                }]
            },
        });
        if let Some(ss) = stream {
            core_conf["streamSettings"] = ss;
        }
        Ok(())
    }

    /// sing-box outbound for this config, ported field-by-field from the old
    /// builder's `Protocol::Shadowsocks | Protocol::Shadowsocks2022` arm
    /// (`method`/`password` with build-time cipher validation via
    /// `core_mapping::singbox_supports_ss_method` — legacy cfb/ctr/rc4-md5/
    /// none methods build on sing-box, unknown ones are refused so the config
    /// is never written invalid), plus the typed `plugin`/`plugin_opts`
    /// (sing-box `ShadowsocksOutboundOptions` keys the old builder dropped).
    fn inject_singbox(
        &self,
        core_conf: &mut Value,
        endpoint: Option<&EndpointEssentials>,
        _opts: InjectOptions,
    ) -> Result<(), SupportError> {
        let Some(ep) = endpoint else {
            return Err(SupportError::MissingField("server", "ss"));
        };
        if !core_mapping::singbox_supports_ss_method(self.method.as_str()) {
            return Err(SupportError::Config(format!(
                "Shadowsocks cipher '{}' is not supported by sing-box; \
                 supported: modern AEAD/2022-blake3 + legacy cfb/ctr/rc4-md5 \
                 methods",
                self.method.as_str()
            )));
        }
        let mut out = json!({
            "tag": "proxy",
            "type": "shadowsocks",
            "server": ep.host,
            "server_port": ep.port,
            "method": self.method,
            "password": self.password,
        });
        if let Some(plugin) = &self.plugin {
            out["plugin"] = json!(plugin);
        }
        if let Some(opts) = &self.plugin_opts {
            let joined: Vec<String> = opts.iter().map(|(k, v)| format!("{k}={v}")).collect();
            out["plugin_opts"] = json!(joined.join(";"));
        }
        *core_conf = out;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;

    use super::super::{
        ConfigKind, CoreType, HostKind, ParsedProto, ProtoIdentity, ProtoSpec, ProtocolConfig,
        ProtocolKind,
    };
    use super::SsConfig;
    use crate::urlx::{RawUrlX, SchemeX};

    fn parse(url: &str) -> ParsedProto {
        SsConfig::try_parse_proto(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    fn config(parsed: ParsedProto) -> SsConfig {
        match parsed.protocol.config {
            ProtocolConfig::Ss(c) => c,
            other => panic!("expected SsConfig, got {other:?}"),
        }
    }

    /// The identity payload must be endpoint-free: no top-level `host`/`port`
    /// keys in the serialized config.
    fn assert_no_top_level_host_port(cfg: &SsConfig) {
        let json = serde_json::to_value(cfg).expect("serialize");
        let obj = json.as_object().expect("config is an object");
        assert!(
            !obj.contains_key("host"),
            "config payload must not carry a top-level host key: {json}"
        );
        assert!(
            !obj.contains_key("port"),
            "config payload must not carry a top-level port key: {json}"
        );
    }

    /// Reconstruct round-trip via the endpoint: parse → `reconstruct_proto(endpoint)`
    /// → re-parse must reproduce the same `ParsedProto` (endpoints + config).
    fn assert_reconstruct_roundtrip(url: &str) {
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed.clone());
        let out = cfg
            .reconstruct_proto(&endpoint)
            .unwrap_or_else(|e| panic!("reconstruct failed for {url}: {e}"));
        let reparsed = parse(&out);
        assert_eq!(parsed, reparsed, "reconstruct round-trip failed for: {url}");
    }

    // ── URL parse: endpoints + config ─────────────────────────────────────

    #[test]
    fn test_ss_basic() {
        // aes-256-gcm:password — a real AEAD cipher (xray-core supports it).
        let url = "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@1.2.3.4:8080";
        let parsed = parse(url);
        assert_eq!(parsed.endpoints.len(), 1);
        let ep = &parsed.endpoints[0];
        assert_eq!(ep.host, "1.2.3.4");
        assert_eq!(ep.host_type, HostKind::Ipv4);
        assert_eq!(ep.port, 8080);
        assert_eq!(ep.ports, vec![8080]);

        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Shadowsocks);
        assert_eq!(parsed.protocol.config_type, ConfigKind::ShareUrl);
        assert_eq!(parsed.protocol.core_type, CoreType::Xray);
        let cfg = config(parsed);
        assert_eq!(cfg.method, "aes-256-gcm");
        assert_eq!(cfg.password, "password");
        assert_no_top_level_host_port(&cfg);
    }

    #[test]
    fn test_ss_legacy_qr_format() {
        // Legacy QR: whole method:password@host:port base64-encoded, no `@` in URL.
        let b64 =
            base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(b"aes-256-gcm:sekrit@example.com:443");
        let parsed = parse(&format!("ss://{b64}"));
        assert_eq!(parsed.endpoints[0].host, "example.com");
        assert_eq!(parsed.endpoints[0].port, 443);
        let cfg = config(parsed);
        assert_eq!(cfg.method, "aes-256-gcm");
        assert_eq!(cfg.password, "sekrit");
    }

    // ── ss cipher routing: kind + core ────────────────────────────────────

    #[test]
    fn ss_cipher_routes_kind_and_core() {
        // 2022-blake3 method -> Shadowsocks2022 + Xray (xray supports the family).
        let parsed = parse("ss://MjAyMi1ibGFrZTMtYWVzLTEyOC1nY206cGFzcw@1.2.3.4:8388");
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Shadowsocks2022);
        assert_eq!(parsed.protocol.core_type, CoreType::Xray);

        // Legacy method -> Shadowsocks + SingBox (xray has no cfb/ctr/rc4).
        let parsed = parse("ss://YWVzLTI1Ni1jZmI6cGFzcw@1.2.3.4:8388");
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Shadowsocks);
        assert_eq!(parsed.protocol.core_type, CoreType::SingBox);

        // AEAD (non-2022) -> Shadowsocks + Xray.
        let parsed = parse("ss://YWVzLTI1Ni1nY206cGFzcw@1.2.3.4:8388");
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Shadowsocks);
        assert_eq!(parsed.protocol.core_type, CoreType::Xray);
    }

    // ── Identity: endpoint-free uid ───────────────────────────────────────

    #[test]
    fn uid_identical_across_servers_different_across_methods() {
        let url_a = "ss://Y2xlb2Y6cGFzc3dvcmQ@a.example.com:8080"; // cleof:password
        let url_b = "ss://Y2xlb2Y6cGFzc3dvcmQ@b.example.com:8081"; // cleof:password
        let url_c = "ss://Y2xlb2Y6cGFzczEyMw==@a.example.com:8080"; // cleof:pass123
        let a = parse(url_a);
        let b = parse(url_b);
        let c = parse(url_c);
        assert_eq!(
            a.uid(),
            b.uid(),
            "same protocol on different servers must dedup to one uid"
        );
        assert_ne!(a.uid(), c.uid(), "different password -> different uid");
        assert_ne!(a.sig(), 0);
    }

    #[test]
    fn ss_password_is_credential_not_sig() {
        let url_a = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080"; // cleof:password
        let url_b = "ss://Y2xlb2Y6cGFzczEyMw==@1.2.3.4:8080"; // cleof:pass123
        let a = config(parse(url_a));
        let b = config(parse(url_b));
        assert_eq!(
            a.compute_sig(),
            b.compute_sig(),
            "password must not change sig"
        );
        assert_ne!(a.compute_cred_hash(), b.compute_cred_hash());
    }

    // ── Reconstruct round-trip via endpoint ───────────────────────────────

    #[test]
    fn reconstruct_roundtrip_via_endpoint() {
        assert_reconstruct_roundtrip("ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080");
        assert_reconstruct_roundtrip("ss://Y2xlb2Y6cGFzc3dvcmQ@example.com:443#my-server");
        assert_reconstruct_roundtrip(
            "ss://YWVzLTI1Ni1nY206cGFzcw@[2001:db8::1]:8388?plugin=obfs-local%3Bobfs%3Dhttp",
        );
    }

    #[test]
    fn ss_reconstruct_with_remarks() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@example.com:443#my-server";
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        assert_eq!(cfg.remarks.as_deref(), Some("my-server"));
        let rebuilt = cfg.reconstruct_proto(&endpoint).unwrap();
        assert!(
            rebuilt.contains("#my-server"),
            "reconstruct should preserve fragment: {rebuilt}"
        );
    }

    // ── Clash round-trip via *_proto ──────────────────────────────────────

    #[test]
    fn clash_roundtrip_from_url_via_proto() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        let proxy = cfg.to_clash_proto(&endpoint).expect("to clash");
        let reparsed = SsConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(
            reparsed.endpoints[0], endpoint,
            "endpoint round-trips through clash"
        );
        assert_eq!(
            reparsed.protocol.config,
            ProtocolConfig::Ss(cfg),
            "config round-trips through clash"
        );
    }

    #[test]
    fn clash_proxy_roundtrip_via_proto() {
        use crate::clash::{ClashProxy, ClashSS};

        let proxy = ClashProxy::Shadowsocks(ClashSS {
            name: "test".into(),
            server: "1.2.3.4".into(),
            port: 8080,
            cipher: "aes-256-gcm".into(),
            password: "sekrit".into(),
            udp: None,
            udp_over_tcp: None,
            plugin: Some("obfs-local".into()),
            plugin_opts: Some("obfs=http".into()),
        });
        let parsed = SsConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(parsed.endpoints[0].host, "1.2.3.4");
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Ipv4);
        assert_eq!(parsed.endpoints[0].port, 8080);
        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Shadowsocks);
        assert_eq!(parsed.protocol.core_type, CoreType::Xray);
        let cfg = match &parsed.protocol.config {
            ProtocolConfig::Ss(c) => c,
            other => panic!("expected SsConfig, got {other:?}"),
        };
        assert_eq!(cfg.method, "aes-256-gcm");
        assert_eq!(cfg.plugin.as_deref(), Some("obfs-local"));
        assert_eq!(
            cfg.plugin_opts
                .as_ref()
                .and_then(|m| m.get("obfs"))
                .map(String::as_str),
            Some("http")
        );
        assert_no_top_level_host_port(cfg);
        let out = cfg.to_clash_proto(&parsed.endpoints[0]).expect("to clash");
        match (out, proxy) {
            (ClashProxy::Shadowsocks(out), ClashProxy::Shadowsocks(orig)) => assert_eq!(out, orig),
            _ => panic!("expected shadowsocks clash proxy"),
        }
    }

    // ── Serde ─────────────────────────────────────────────────────────────

    #[test]
    fn test_serde_roundtrip() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let cfg = config(parse(url));
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: SsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, deserialized);
        assert_no_top_level_host_port(&deserialized);
    }

    // ── Legacy trait bridge ───────────────────────────────────────────────

    #[test]
    fn legacy_bridge_parse_works_but_reconstruct_to_clash_error() {
        let url = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
        let bridged = SsConfig::try_parse(&RawUrlX::from(url)).expect("bridged parse");
        assert_eq!(bridged.schema(), SchemeX::SS);
        assert_eq!(bridged.method, "cleof");
        // host/port accessors are gone — the endpoint lives on ParsedProto.
        assert_eq!(bridged.host(), None);
        assert_eq!(bridged.port(), None);
        // Degraded legacy paths error instead of fabricating a host.
        assert!(bridged.reconstruct().is_err());
        assert!(bridged.to_clash().is_err());
    }

    // ── Xray inject_to (Task 14) ──────────────────────────────────────────

    use super::super::{EndpointEssentials, InjectOptions, InjectToCoreConf, SupportError};

    fn ss_aead() -> SsConfig {
        config(parse("ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@1.2.3.4:8080"))
    }

    #[test]
    fn xray_inject_writes_proxy_outbound() {
        let cfg = ss_aead();
        let ep = EndpointEssentials::new("1.2.3.4", 8080);
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::Xray,
            Some(&ep),
            InjectOptions::default(),
        )
        .expect("ss inject");
        assert_eq!(conf["tag"], "proxy");
        assert_eq!(conf["protocol"], "shadowsocks");
        let server = &conf["settings"]["servers"][0];
        assert_eq!(server["address"], "1.2.3.4");
        assert_eq!(server["port"], 8080);
        assert_eq!(server["method"], "aes-256-gcm");
        assert_eq!(server["password"], "password");
        // No TLS/transport → no streamSettings
        assert!(conf.get("streamSettings").is_none());
    }

    #[test]
    fn xray_inject_2022_blake3_method_builds() {
        let cfg = config(parse(
            "ss://MjAyMi1ibGFrZTMtYWVzLTEyOC1nY206cGFzc3dvcmQ@1.2.3.4:8080",
        ));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::Xray,
            Some(&EndpointEssentials::new("1.2.3.4", 8080)),
            InjectOptions::default(),
        )
        .expect("2022-blake3 inject");
        assert_eq!(
            conf["settings"]["servers"][0]["method"],
            "2022-blake3-aes-128-gcm"
        );
    }

    #[test]
    fn xray_inject_rejects_legacy_cipher() {
        // aes-256-cfb is not in xray-core's CipherType enum; the build must
        // refuse instead of the core dying on startup.
        let cfg = config(parse("ss://YWVzLTI1Ni1jZmI6cGFzc3dvcmQ@1.2.3.4:8080"));
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(
                &mut conf,
                CoreType::Xray,
                Some(&EndpointEssentials::new("1.2.3.4", 8080)),
                InjectOptions::default(),
            )
            .expect_err("aes-256-cfb must be rejected for xray");
        assert!(
            err.to_string().contains("aes-256-cfb"),
            "error must name the cipher: {err}"
        );
    }

    #[test]
    fn xray_inject_without_endpoint_is_rejected() {
        let cfg = ss_aead();
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(&mut conf, CoreType::Xray, None, InjectOptions::default())
            .expect_err("orphan ss must be rejected");
        assert!(matches!(err, SupportError::MissingField("server", "ss")));
    }

    #[test]
    fn singbox_inject_writes_proxy_outbound() {
        let cfg = ss_aead();
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("1.2.3.4", 8080)),
            InjectOptions::default(),
        )
        .expect("ss sing-box inject");
        assert_eq!(conf["tag"], "proxy");
        assert_eq!(conf["type"], "shadowsocks");
        assert_eq!(conf["server"], "1.2.3.4");
        assert_eq!(conf["server_port"], 8080);
        assert_eq!(conf["method"], "aes-256-gcm");
        assert_eq!(conf["password"], "password");
    }

    #[test]
    fn singbox_inject_legacy_cipher_builds_ok() {
        // aes-256-cfb is legacy — sing-box builds it, xray-core cannot.
        let cfg = config(parse("ss://YWVzLTI1Ni1jZmI6cGFzcw@1.2.3.4:8388"));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("1.2.3.4", 8388)),
            InjectOptions::default(),
        )
        .expect("legacy cipher must build on sing-box");
        assert_eq!(conf["type"], "shadowsocks");
        assert_eq!(conf["method"], "aes-256-cfb");
    }

    #[test]
    fn singbox_inject_unknown_cipher_is_rejected() {
        // salsa20 is supported by neither core — refuse at build time.
        let cfg = SsConfig {
            method: "salsa20".into(),
            password: "password".into(),
            security: super::super::common::SecurityConfig::default(),
            remarks: None,
            plugin: None,
            plugin_opts: None,
        };
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(
                &mut conf,
                CoreType::SingBox,
                Some(&EndpointEssentials::new("1.2.3.4", 8388)),
                InjectOptions::default(),
            )
            .expect_err("unknown cipher must be rejected");
        assert!(
            err.to_string().contains("salsa20"),
            "error must name the cipher: {err}"
        );
    }

    #[test]
    fn singbox_inject_without_endpoint_is_rejected() {
        let cfg = ss_aead();
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(&mut conf, CoreType::SingBox, None, InjectOptions::default())
            .expect_err("orphan ss must be rejected");
        assert!(matches!(err, SupportError::MissingField("server", "ss")));
    }
}
