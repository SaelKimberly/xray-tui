//! VLESS (`vless://`) URL parsing.
//!
//! # Format
//! ```text
//! vless://<uuid>@<host>:<port>?<query_params>#<remarks>
//! ```
//!
//! Standard URI format (NOT base64-encoded). UUID goes in userinfo,
//! all configuration in query parameters, remarks in fragment.
//!
//! # Query Parameters
//!
//! | Key        | Values                                          | Purpose                     | Default   |
//! |------------|-------------------------------------------------|-----------------------------|-----------|
//! | `type`     | tcp, ws, grpc, http, kcp, quic, httpupgrade     | Transport/network type      | `"tcp"`   |
//! | `security` | none, tls, reality                               | TLS/security mode           | `"none"`  |
//! | `encryption`| none                                           | Payload encryption          | `"none"`  |
//! | `flow`     | xtls-rprx-vision, xtls-rprx-vision-udp443       | XTLS flow control           | —         |
//! | `host`     | domain                                          | HTTP Host header            | —         |
//! | `sni`      | domain                                          | TLS SNI override            | —         |
//! | `path`     | URL path                                        | WS path / gRPC serviceName  | —         |
//! | `alpn`     | comma-separated (h2,http/1.1)                   | ALPN list                   | —         |
//! | `fp`       | chrome, firefox, safari, random, randomized       | uTLS fingerprint            | —         |
//! | `pbk`      | base64 key                                      | REALITY public key          | —         |
//! | `sid`      | hex string                                      | REALITY short ID            | —         |
//! | `spx`      | path                                            | REALITY spider X            | —         |
//! | `splice`   | 1/0, true/false                                 | Splice mode                 | —         |
//!
//! # Edge Cases
//! - Userinfo may contain `@` for combined `userinfo@hostport` format
//! - UUID is validated via `uuid::Uuid::parse_str`
//! - For `type=grpc`, path is read from `serviceName` query param
//! - For `type=kcp`/`mkcp`, path is read from `seed` query param
//! - REALITY is VLESS-only (not supported by `VMess`)
//! - IPv6 addresses must be bracketed `[::1]`
//! - Empty `type` defaults to `"tcp"`, empty `security` to `"none"`
//!
//! # References
//! - Xray-core: `proxy/vless/account.go`, `proxy/vless/encoding/addons.proto`
//! - sing-box: `option/vless.go`
//! - v2rayN: `VLESSFmt.cs`
//! - outbound: `dialer/v2ray/v2ray.go` `ParseVlessURL`

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText};

use super::ProtoIdentity;
use super::common::{
    RealityOpts, SecurityConfig, TlsConfig, TlsOpts, TransportConfig, security_force_insecure,
    should_skip_endpoint_param, to_singbox_tls, to_singbox_transport, to_xray_stream_settings,
    validate_xray_reality,
};
use super::core_mapping;
use super::utils;
use super::{
    ConfigKind, CoreType, EndpointEssentials, InjectOptions, InjectToCoreConf, ParseError,
    ParsedProto, ProtoSpec, ProtocolConfig, ProtocolEssentials, ProtocolKind, SupportError,
};
use crate::clash::{ClashProxy, ClashVless};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_alpn_as_str, clash_tls_to_security, clash_to_endpoint, clash_transport_to_transport,
    host_kind_for, security_to_clash_reality, security_to_clash_tls, transport_to_clash,
};

/// VLESS protocol configuration — the identity payload (sans host/port).
///
/// The endpoint (server host/port) lives in [`EndpointEssentials`] on the
/// [`ParsedProto`] boundary; this struct only carries endpoint-free protocol
/// parameters, so the same config pointed at different servers shares one
/// identity. Host-bearing fields on transports/security hold only EXPLICIT
/// protocol parameters (the host-free parse mandate) — never the endpoint.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct VlessConfig {
    pub uuid: String,
    pub uuid_origin: Option<TinyText>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub transport: TransportConfig,
    pub encryption: Option<TinyText>,
    pub flow: Option<TinyText>,
    pub path: Option<TinyText>,
    pub splice: Option<bool>,
    pub remarks: Option<TinyText>,
}

impl VlessConfig {
    /// Parse a VLESS URL into the parse boundary: [`ParsedProto`] with the
    /// endpoint essentials (host/port) split out and the identity payload
    /// ([`ProtocolEssentials::config`]) holding only endpoint-free protocol
    /// parameters.
    ///
    /// UUID is extracted from userinfo, server address from host:port,
    /// all configuration from query parameters, remarks from fragment.
    ///
    /// Supports combined `userinfo@hostport` or separate hostport components.
    /// UUID validated via `uuid::Uuid::parse_str`.
    #[allow(clippy::too_many_lines)]
    pub fn try_parse_proto(raw: &RawUrlX<'_>) -> Result<ParsedProto, ParseError> {
        let (username, hostport) = if let Some(hostport) = raw.hostport {
            (raw.userinfo, hostport)
        } else {
            let userinfo = raw.userinfo;
            let (username, hostport) = userinfo.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{userinfo}: missing hostport").into())
            })?;
            (username, hostport)
        };

        let (parsed_host, parsed_port_spec) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port_spec
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;
        let (uuid, uuid_origin) = if uuid::Uuid::parse_str(username).is_ok() {
            (username.to_string(), None)
        } else {
            let generated = uuid::Uuid::new_v5(&uuid::Uuid::nil(), username.as_bytes()).to_string();
            (generated, Some(TinyText::from(username)))
        };

        // Endpoint essentials: host/port live here, never in the config payload.
        let mut endpoint = EndpointEssentials::new(parsed_host.to_str().into_owned(), parsed_port);
        endpoint.host_type = host_kind_for(&parsed_host);
        if parsed_port_spec.length() > 1 {
            // Multi-port spec: endpoints[0] carries the primary port plus the
            // full port list.
            endpoint.ports = parsed_port_spec.iter().collect();
        }

        let query = utils::parse_query(raw.query);

        // type/transport: tcp/ws/grpc/http/kcp/quic/httpupgrade. Defaults to "tcp".
        let transport_type = utils::query_get(&query, "type")
            .unwrap_or("tcp")
            .to_string();
        let path = utils::query_get(&query, "path").map(TinyText::from);
        // encryption: typically "none" (VLESS relies on TLS, not payload encryption)
        let encryption = utils::query_get(&query, "encryption")
            .filter(|v| *v != "none")
            .map(TinyText::from);
        // flow: xtls-rprx-vision for XTLS direct transmission (TLS 1.3 required)
        let flow = utils::query_get(&query, "flow").map(TinyText::from);
        // splice: boolean splice mode flag
        let splice = utils::query_get(&query, "splice").and_then(|v| match v {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        });

        // TLS/security config
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
                        ..Default::default()
                    })),
                    enc: None,
                }
            }
            Some("reality") => SecurityConfig {
                tls: Some(TlsConfig::Reality(RealityOpts {
                    sni: utils::query_get(&query, "sni").map(TinyText::from),
                    fp: utils::query_get(&query, "fp").map(TinyText::from),
                    pbk: utils::query_get(&query, "pbk").map(str::to_string),
                    sid: utils::query_get(&query, "sid").map(TinyText::from),
                    spx: utils::query_get(&query, "spx").map(TinyText::from),
                })),
                enc: None,
            },
            _ => SecurityConfig::default(),
        };

        let remarks = utils::decode_fragment(raw)?;

        // HOST-FREE PARSE MANDATE (identity invariant): the endpoint host must
        // never leak into the config payload. Only EXPLICIT query params
        // (`host=`, `sni=`) are protocol parameters and are stored; `with_host`
        // is called with `None` as the server address, so absent params leave
        // the ws/grpc/http host fields unset (the builder injects the endpoint
        // host at build time in phase D).
        let host = utils::query_get(&query, "host").map(str::to_string);
        let sni_from_query = utils::query_get(&query, "sni").map(str::to_string);

        let mut transport =
            TransportConfig::from_type_and_path(Some(&transport_type), path.as_deref())?
                .unwrap_or(TransportConfig::Tcp);
        transport = transport.with_host(host, sni_from_query, None);

        // Extract mode and extra for XHttp, validate mode
        if let TransportConfig::XHttp(xcfg) = &mut transport {
            if let Some(mode) = utils::query_get(&query, "mode") {
                if mode.is_empty() {
                    // Empty mode — keep the default ("auto") set by from_type_and_path
                } else if matches!(mode, "auto" | "packet-up" | "stream-up" | "stream-one") {
                    xcfg.mode = Some(TinyText::from(mode));
                } else if let Some(recovered) = recover_xhttp_mode(mode) {
                    tracing::warn!(
                        target: "proto_spec::vless",
                        "Recovered XHttp mode from '{}' to '{}'",
                        mode, recovered
                    );
                    xcfg.mode = Some(TinyText::from(recovered));
                } else {
                    return Err(ParseError::InvalidConf(
                        "mode".into(),
                        mode.to_string().into(),
                    ));
                }
            }
            if let Some(extra) = utils::query_get(&query, "extra") {
                let bytes = extra.as_bytes().to_vec();
                match serde_json::from_slice(&bytes) {
                    Ok(v) => xcfg.extra = Some(v),
                    Err(_) => {
                        return Err(ParseError::InvalidConf(
                            "extra".into(),
                            extra.to_string().into(),
                        ));
                    }
                }
            }
        }

        let path = match &transport {
            TransportConfig::Ws(ws) => ws.path.clone(),
            TransportConfig::Grpc(g) => g.path.clone(),
            TransportConfig::Http(h) => h.path.clone(),
            TransportConfig::HttpUpgrade(cfg) => cfg.path.clone(),
            TransportConfig::XHttp(cfg) => cfg.path.clone(),
            _ => path,
        };

        let config = Self {
            uuid,
            uuid_origin,
            transport,
            security,
            encryption,
            flow,
            path,
            splice,
            remarks,
        };
        Ok(ParsedProto {
            endpoints: vec![endpoint],
            protocol: ProtocolEssentials {
                proto_kind: ProtocolKind::Vless,
                config_type: ConfigKind::ShareUrl,
                core_type: core_mapping::resolve_core(ProtocolKind::Vless, None, None),
                config: ProtocolConfig::Vless(config),
            },
        })
    }

    /// Rebuild the share URL from this endpoint-free config plus the endpoint
    /// essentials. Endpoint host/port come from `endpoint`; every protocol
    /// parameter (including explicit transport/security host fields) is taken
    /// from the config.
    #[allow(clippy::too_many_lines)]
    pub fn reconstruct_proto(&self, endpoint: &EndpointEssentials) -> Result<String, ParseError> {
        let endpoint_host = endpoint.host.as_str();
        let hostport = if endpoint_host.contains(':') {
            format!("[{endpoint_host}]:{}", endpoint.port)
        } else {
            format!("{endpoint_host}:{}", endpoint.port)
        };

        let userinfo = self.uuid_origin.as_deref().unwrap_or(&self.uuid);
        let mut base = url::Url::parse(format!("vless://{userinfo}@{hostport}").as_str())
            .map_err(|e| ParseError::Unknown(e.into()))?;

        {
            let mut q = base.query_pairs_mut();
            // Security config
            if let Some(ref tls_config) = self.security.tls {
                match tls_config {
                    TlsConfig::Tls(opts) => {
                        q.append_pair("security", "tls");
                        if let Some(ref v) = opts.sni
                            && !should_skip_endpoint_param(endpoint_host, v)
                        {
                            q.append_pair("sni", v);
                        }
                        if let Some(ref v) = opts.alpn {
                            q.append_pair("alpn", v);
                        }
                        if let Some(ref v) = opts.fp {
                            q.append_pair("fp", v);
                        }
                        if opts.insecure == Some(true) {
                            q.append_pair("allowInsecure", "1");
                        }
                    }
                    TlsConfig::Reality(opts) => {
                        q.append_pair("security", "reality");
                        if let Some(ref v) = opts.sni
                            && !should_skip_endpoint_param(endpoint_host, v)
                        {
                            q.append_pair("sni", v);
                        }
                        if let Some(ref v) = opts.fp {
                            q.append_pair("fp", v);
                        }
                        if let Some(ref v) = opts.pbk {
                            q.append_pair("pbk", v);
                        }
                        if let Some(ref v) = opts.sid {
                            q.append_pair("sid", v);
                        }
                        if let Some(ref v) = opts.spx {
                            q.append_pair("spx", v);
                        }
                    }
                }
            }
            if self.transport.type_str() != "tcp" {
                q.append_pair("type", self.transport.type_str());
            }
            match &self.transport {
                TransportConfig::Ws(cfg) => {
                    if let Some(host) = &cfg.host
                        && !should_skip_endpoint_param(endpoint_host, host)
                    {
                        q.append_pair("host", host);
                    }
                }
                TransportConfig::Grpc(cfg) => {
                    if let Some(auth) = &cfg.authority
                        && !should_skip_endpoint_param(endpoint_host, auth)
                    {
                        q.append_pair("host", auth);
                    }
                }
                TransportConfig::Http(cfg) => {
                    if let Some(host) = &cfg.host
                        && !should_skip_endpoint_param(endpoint_host, host)
                    {
                        q.append_pair("host", host);
                    }
                }
                TransportConfig::HttpUpgrade(cfg) => {
                    if let Some(ref host) = cfg.host
                        && !should_skip_endpoint_param(endpoint_host, host)
                    {
                        q.append_pair("host", host);
                    }
                }
                TransportConfig::XHttp(cfg) => {
                    if let Some(ref host) = cfg.host
                        && !should_skip_endpoint_param(endpoint_host, host)
                    {
                        q.append_pair("host", host);
                    }
                    if let Some(ref mode) = cfg.mode {
                        q.append_pair("mode", mode);
                    }
                    if let Some(ref extra) = cfg.extra {
                        q.append_pair("extra", &extra.to_string());
                    }
                }
                _ => {}
            }
            if let Some(ref path) = self.path {
                q.append_pair("path", path);
            }
            if let Some(ref v) = self.encryption {
                q.append_pair("encryption", v);
            }
            if let Some(ref v) = self.flow {
                q.append_pair("flow", v);
            }
            if let Some(v) = self.splice {
                q.append_pair("splice", if v { "true" } else { "false" });
            }
        }
        if let Some(ref remarks) = self.remarks {
            let frag = urlencoding::decode(remarks).unwrap_or(std::borrow::Cow::Borrowed(remarks));
            let frag = frag.trim();
            if !frag.is_empty() {
                base.set_fragment(Some(frag));
            }
        }

        Ok(base.to_string())
    }
}

impl VlessConfig {
    /// Serialize this endpoint-free config plus the endpoint to a Clash proxy
    /// entry. Endpoint host/port are taken from `endpoint` (identity is not
    /// involved in this direction).
    pub fn to_clash_proto(
        &self,
        endpoint: &EndpointEssentials,
    ) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let server = endpoint.host.clone();
        let alpn_str = self.security.alpn();
        // HOST-FREE: `None` — the endpoint host must never fall back into
        // Clash h2 `host` (which re-imports into the identity config).
        let (network, ws_opts, grpc_opts, _, _, _) = transport_to_clash(&self.transport, None);
        let (tls, servername, skip_cert_verify, _, _) = security_to_clash_tls(&self.security);
        let reality_opts = security_to_clash_reality(&self.security);
        Ok(ClashProxy::Vless(ClashVless {
            name,
            server,
            port: endpoint.port,
            uuid: self.uuid.clone(),
            udp: None,
            tfo: None,
            network,
            flow: self.flow.as_ref().map(std::string::ToString::to_string),
            encryption: self
                .encryption
                .as_ref()
                .map(std::string::ToString::to_string),
            tls,
            servername,
            skip_cert_verify,
            alpn: alpn_str.map(|s| vec![s.to_string()]),
            reality_opts,
            ws_opts,
            grpc_opts,
            xhttp_opts: None,
        }))
    }

    /// Parse a Clash proxy entry into the parse boundary: `server`/`port`
    /// become the endpoint essentials; the config payload is endpoint-free.
    /// Explicit Clash TLS `servername` and ws `Host` header fields are
    /// protocol parameters and stay in the config.
    pub fn try_from_clash_proto(proxy: &ClashProxy) -> Result<ParsedProto, ParseError> {
        match proxy {
            ClashProxy::Vless(c) => {
                let security = clash_tls_to_security(
                    c.tls,
                    c.servername.as_deref(),
                    c.skip_cert_verify,
                    clash_alpn_as_str(c.alpn.as_ref()),
                    None,
                    c.reality_opts.as_ref(),
                );
                // HOST-FREE: the endpoint server is never copied into
                // transport host fields (`server = None`).
                let transport = clash_transport_to_transport(
                    c.network.as_deref(),
                    c.ws_opts.as_ref(),
                    c.grpc_opts.as_ref(),
                    None,
                    None,
                    None,
                    None,
                );
                let path = match &transport {
                    TransportConfig::Ws(ws) => ws.path.clone(),
                    TransportConfig::Grpc(g) => g.path.clone(),
                    TransportConfig::Http(h) => h.path.clone(),
                    TransportConfig::HttpUpgrade(cfg) => cfg.path.clone(),
                    TransportConfig::XHttp(cfg) => cfg.path.clone(),
                    _ => None,
                };
                let config = Self {
                    uuid: c.uuid.clone(),
                    uuid_origin: None,
                    security,
                    transport,
                    encryption: c.encryption.clone().map(TinyText::from),
                    flow: c.flow.clone().map(TinyText::from),
                    path,
                    splice: None,
                    remarks: match c.name.as_str() {
                        "" => None,
                        s => Some(TinyText::from(s)),
                    },
                };
                Ok(ParsedProto {
                    endpoints: vec![clash_to_endpoint(&c.server, c.port)],
                    protocol: ProtocolEssentials {
                        proto_kind: ProtocolKind::Vless,
                        config_type: ConfigKind::ShareUrl,
                        core_type: core_mapping::resolve_core(ProtocolKind::Vless, None, None),
                        config: ProtocolConfig::Vless(config),
                    },
                })
            }
            _ => Err(ParseError::Unknown("expected vless clash proxy".into())),
        }
    }
}

/// Legacy [`ProtoSpec`] bridge — kept so `ProtocolConfig` dispatch (and the
/// other 18 configs) compile unchanged.
///
/// DEGRADED PATH (documented): `try_parse`/`try_from_clash` still work by
/// delegating to the `*_proto` variants and discarding the parsed endpoints;
/// `to_clash`/`reconstruct` return errors because the config no longer stores
/// host/port. Import/export rewires to the `*_proto` variants in T11 (phase D
/// builders take the endpoint separately).
impl ProtoSpec for VlessConfig {
    /// # Errors
    ///
    /// If either the URL is invalid or the external configuration is invalid.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let parsed = Self::try_parse_proto(raw)?;
        match parsed.protocol.config {
            ProtocolConfig::Vless(config) => Ok(config),
            // Parser invariant: a vless URL always yields a VlessConfig.
            _ => Err(ParseError::Unknown(
                "vless URL parsed to a non-vless config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::reconstruct_proto`] with the endpoint.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::InvalidHost(
            "vless config no longer stores host/port; use VlessConfig::reconstruct_proto(endpoint)"
                .into(),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::Vless
    }

    /// `None` — the endpoint host moved to [`EndpointEssentials`] (T4).
    fn host(&self) -> Option<&HostSpec> {
        None
    }

    /// `None` — the endpoint port moved to [`EndpointEssentials`] (T4).
    fn port(&self) -> Option<u16> {
        None
    }

    fn remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    fn transport_type(&self) -> Option<&str> {
        Some(self.transport.type_str())
    }

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }

    /// # Errors
    ///
    /// If the Clash proxy doesn't match this protocol type.
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        let parsed = Self::try_from_clash_proto(proxy)?;
        match parsed.protocol.config {
            ProtocolConfig::Vless(config) => Ok(config),
            _ => Err(ParseError::Unknown(
                "vless clash proxy parsed to a non-vless config".into(),
            )),
        }
    }

    /// # Errors
    ///
    /// Always — host/port are no longer stored on the config; use
    /// [`Self::to_clash_proto`] with the endpoint.
    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        Err(ProtoSpecError::Unsupported(
            "vless config no longer stores host/port; use VlessConfig::to_clash_proto(endpoint)"
                .into(),
        ))
    }
}

impl ProtoIdentity for VlessConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"vless");
        let sec_type = self.security.type_str().unwrap_or("none");
        hasher.write(sec_type.as_bytes());
        hasher.write(self.transport.type_str().as_bytes());
        // Endpoint (host/port) intentionally absent from the identity — it
        // lives on the ParsedProto boundary, never in the config payload (T4).
        match &self.transport {
            TransportConfig::HttpUpgrade(cfg) => {
                if let Some(v) = &cfg.host {
                    hasher.write(v.as_bytes());
                }
            }
            TransportConfig::XHttp(cfg) => {
                if let Some(v) = &cfg.host {
                    hasher.write(v.as_bytes());
                }
            }
            _ => {}
        }
        if let Some(path) = &self.path {
            hasher.write(path.as_bytes());
        }
        if let Some(v) = &self.encryption {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.security.sni() {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.flow {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.security.alpn() {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.security.fp() {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = self.splice {
            hasher.write(if v { b"true" } else { b"false" });
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[
            ("uuid", self.uuid.as_str()),
            ("pbk", self.security.pbk().unwrap_or("")),
            ("sid", self.security.sid().unwrap_or("")),
        ])
    }
}

/// Try to recover a valid `XHttp` mode from an unrecognized mode string
/// by matching the longest known valid prefix.
fn recover_xhttp_mode(mode: &str) -> Option<&'static str> {
    const VALID_MODES: &[&str] = &["packet-up", "stream-one", "stream-up", "auto"];
    VALID_MODES
        .iter()
        .find(|&&valid| mode.starts_with(valid))
        .copied()
        .map(|v| v as _)
}

impl InjectToCoreConf for VlessConfig {
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

impl VlessConfig {
    /// xray-core outbound for this config. `endpoint` supplies the server
    /// host/port (the outbound always needs it — there is no orphan form);
    /// transport host/authority left unset by the host-free parse mandate is
    /// filled at build time via `TransportConfig::with_host` (never mutating
    /// the stored config). Ported field-by-field from the old xray builder's
    /// `Protocol::Vless` arm.
    fn inject_xray(
        &self,
        core_conf: &mut Value,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError> {
        let Some(ep) = endpoint else {
            return Err(SupportError::MissingField("server", "vless"));
        };
        // A `reality` security without publicKey/serverName kills the core at
        // startup — refuse to emit such a config (old builder behavior).
        validate_xray_reality(&self.security)?;
        let transport = self
            .transport
            .clone()
            .with_host(Some(ep.host.clone()), None, None);
        let security = security_force_insecure(&self.security, opts.skip_cert_verify);
        let stream = to_xray_stream_settings(&security, &transport);
        let flow = self.flow.as_deref().unwrap_or("");
        *core_conf = json!({
            "tag": "proxy",
            "protocol": "vless",
            "settings": {
                "vnext": [{
                    "address": ep.host,
                    "port": ep.port,
                    "users": [{
                        "id": self.uuid,
                        "encryption": "none",
                        "flow": flow
                    }]
                }]
            },
        });
        if let Some(ss) = stream {
            core_conf["streamSettings"] = ss;
        }
        Ok(())
    }

    /// sing-box outbound for this config, ported field-by-field from the old
    /// builder's `Protocol::Vless` arm (`uuid` + `flow` when non-empty + TLS
    /// via the shared helper) plus the typed `transport` (sing-box
    /// `V2RayTransportOptions`; `kcp`/`xhttp` refuse at build time).
    fn inject_singbox(
        &self,
        core_conf: &mut Value,
        endpoint: Option<&EndpointEssentials>,
        opts: InjectOptions,
    ) -> Result<(), SupportError> {
        let Some(ep) = endpoint else {
            return Err(SupportError::MissingField("server", "vless"));
        };
        // Transport host left unset by the host-free parse mandate is filled
        // at build time (never mutating the stored config) — same rule as the
        // xray arm.
        let transport = self
            .transport
            .clone()
            .with_host(Some(ep.host.clone()), None, None);
        let mut out = json!({
            "tag": "proxy",
            "type": "vless",
            "server": ep.host,
            "server_port": ep.port,
            "uuid": self.uuid,
        });
        if let Some(flow) = self.flow.as_deref().filter(|s| !s.is_empty()) {
            out["flow"] = json!(flow);
        }
        if let Some(tls) = to_singbox_tls(&self.security, ep, opts.skip_cert_verify) {
            out["tls"] = tls;
        }
        if let Some(transport) = to_singbox_transport(&transport)? {
            out["transport"] = transport;
        }
        *core_conf = out;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ConfigKind, CoreType, HostKind, ParsedProto, ProtoSpec, ProtocolConfig, ProtocolKind,
    };
    use super::VlessConfig;
    use crate::proto_spec::common::TransportConfig;
    use crate::urlx::{RawUrlX, SchemeX};

    const UUID: &str = "6202b230-417c-4d8e-b624-0f71afa9c75d";

    fn parse(url: &str) -> ParsedProto {
        VlessConfig::try_parse_proto(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    fn config(parsed: ParsedProto) -> VlessConfig {
        match parsed.protocol.config {
            ProtocolConfig::Vless(c) => c,
            other => panic!("expected VlessConfig, got {other:?}"),
        }
    }

    /// The identity payload must be endpoint-free: no top-level `host`/`port`
    /// keys in the serialized config.
    fn assert_no_top_level_host_port(cfg: &VlessConfig) {
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

    /// Reconstruct round-trip via the endpoint: parse → reconstruct_proto(endpoint)
    /// → re-parse must reproduce the same ParsedProto (endpoints + config).
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
    fn test_vless_basic() {
        let url = format!(
            "vless://{UUID}@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        );
        let parsed = parse(&url);
        assert_eq!(parsed.endpoints.len(), 1);
        let ep = &parsed.endpoints[0];
        assert_eq!(ep.host, "159.223.24.65");
        assert_eq!(ep.host_type, HostKind::Ipv4);
        assert_eq!(ep.port, 443);
        assert_eq!(ep.ports, vec![443]);

        assert_eq!(parsed.protocol.proto_kind, ProtocolKind::Vless);
        assert_eq!(parsed.protocol.config_type, ConfigKind::ShareUrl);
        assert_eq!(parsed.protocol.core_type, CoreType::Xray);
        let cfg = config(parsed);
        assert_eq!(cfg.uuid, UUID);
        assert_eq!(cfg.transport.type_str(), "ws");
        assert_eq!(cfg.security.type_str(), Some("tls"));
        assert_eq!(cfg.security.sni(), Some("test.ir"));
        if let TransportConfig::Ws(ws) = &cfg.transport {
            assert_eq!(ws.path.as_deref(), Some("/?ed=2560"));
        } else {
            panic!("expected ws transport");
        }
        assert_no_top_level_host_port(&cfg);
    }

    #[test]
    fn test_vless_endpoint_host_kinds() {
        let dns = parse("vless://6202b230-417c-4d8e-b624-0f71afa9c75d@example.com:443?type=tcp");
        assert_eq!(dns.endpoints[0].host, "example.com");
        assert_eq!(dns.endpoints[0].host_type, HostKind::Dns);

        let v6 = parse("vless://6202b230-417c-4d8e-b624-0f71afa9c75d@[2001:db8::1]:8443?type=tcp");
        assert_eq!(v6.endpoints[0].host, "2001:db8::1");
        assert_eq!(v6.endpoints[0].host_type, HostKind::Ipv6);
        assert_eq!(v6.endpoints[0].port, 8443);
    }

    #[test]
    fn test_vless_multi_port_endpoint() {
        let parsed = parse(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@example.com:443,8443,9000-9002?type=tcp",
        );
        assert_eq!(parsed.endpoints.len(), 1);
        let ep = &parsed.endpoints[0];
        assert_eq!(ep.port, 443, "primary port first");
        assert_eq!(ep.ports, vec![443, 8443, 9000, 9001, 9002]);
        // The config payload is untouched by the multi-port spec.
        assert_no_top_level_host_port(&config(parsed));
    }

    #[test]
    fn test_vless_reality() {
        let url = format!(
            "vless://{UUID}@159.223.24.65:443?security=reality&encryption=none&type=tcp&flow=xtls-rprx-vision&pbk=abc123"
        );
        let parsed = parse(&url);
        let cfg = config(parsed);
        assert_eq!(cfg.flow.as_deref(), Some("xtls-rprx-vision"));
        if let Some(super::TlsConfig::Reality(ref opts)) = cfg.security.tls {
            assert_eq!(opts.pbk.as_deref(), Some("abc123"));
        } else {
            panic!("expected reality config");
        }
        assert_no_top_level_host_port(&cfg);
    }

    // ── Identity: endpoint-free uid ───────────────────────────────────────

    #[test]
    fn uid_identical_across_servers_different_across_uuids() {
        let url_a = format!(
            "vless://{UUID}@a.example.com:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        );
        let url_b = format!(
            "vless://{UUID}@b.example.com:8443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        );
        let url_c = format!(
            "vless://22222222-3333-4444-5555-666666666666@a.example.com:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        );
        let a = parse(&url_a);
        let b = parse(&url_b);
        let c = parse(&url_c);
        assert_eq!(
            a.uid(),
            b.uid(),
            "same protocol on different servers must dedup to one uid"
        );
        assert_ne!(a.uid(), c.uid(), "different uuid -> different uid");
        assert_ne!(a.sig(), 0);
    }

    #[test]
    fn vless_reality_sig_excludes_pbk_sid_cred_hash_includes_them() {
        use super::super::Proto;
        let url_a = "vless://11111111-2222-3333-4444-555555555555@a.example.com:443?security=reality&pbk=AAAA&sid=1111&spx=%2F&fp=chrome#r";
        let url_b = "vless://11111111-2222-3333-4444-555555555555@a.example.com:443?security=reality&pbk=BBBB&sid=2222&spx=%2F&fp=chrome#r";
        let url_c = "vless://22222222-3333-4444-5555-666666666666@a.example.com:443?security=reality&pbk=AAAA&sid=1111&spx=%2F&fp=chrome#r";
        let a = Proto::new(ProtocolConfig::Vless(
            VlessConfig::try_parse(&RawUrlX::from(url_a)).unwrap(),
        ));
        let b = Proto::new(ProtocolConfig::Vless(
            VlessConfig::try_parse(&RawUrlX::from(url_b)).unwrap(),
        ));
        let c = Proto::new(ProtocolConfig::Vless(
            VlessConfig::try_parse(&RawUrlX::from(url_c)).unwrap(),
        ));
        assert_eq!(
            a.sig(),
            b.sig(),
            "sig is semantic: pbk/sid values must NOT change it"
        );
        assert_ne!(a.cred_hash(), b.cred_hash(), "cred_hash covers pbk/sid");
        assert_ne!(a.uid(), b.uid());
        assert_ne!(a.uid(), c.uid(), "different uuid -> different uid");
        assert_ne!(a.sig(), 0);
    }

    // ── Reconstruct round-trip via endpoint ───────────────────────────────

    #[test]
    fn reconstruct_roundtrip_via_endpoint() {
        assert_reconstruct_roundtrip(&format!(
            "vless://{UUID}@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        ));
        assert_reconstruct_roundtrip(&format!("vless://{UUID}@host:443?type=ws&path=%2F"));
        assert_reconstruct_roundtrip(&format!("vless://{UUID}@host:443"));
        assert_reconstruct_roundtrip(&format!(
            "vless://{UUID}@host.com:443?type=httpupgrade&path=/test&host=myhost.com"
        ));
        assert_reconstruct_roundtrip(&format!(
            "vless://{UUID}@host.com:443?type=xhttp&mode=auto&path=/test&host=myhost.com"
        ));
        assert_reconstruct_roundtrip(&format!(
            "vless://{UUID}@host.com:443?type=grpc&path=svc&security=tls&sni=cdn.example.com"
        ));
        assert_reconstruct_roundtrip(&format!("vless://{UUID}@[2001:db8::1]:443?type=tcp"));
    }

    #[test]
    fn explicit_host_params_survive_roundtrip() {
        for url in [
            format!(
                "vless://{UUID}@example.com:443?security=tls&type=ws&host=cdn.example.com&path=%2Fws#r"
            ),
            format!(
                "vless://{UUID}@example.com:443?security=tls&type=grpc&host=cdn.example.com&path=svc#r"
            ),
            format!(
                "vless://{UUID}@example.com:443?security=tls&type=http&host=cdn.example.com&path=%2Fh#r"
            ),
        ] {
            let parsed = parse(&url);
            let endpoint = parsed.endpoints[0].clone();
            let cfg = config(parsed);
            let out = cfg.reconstruct_proto(&endpoint).expect("reconstruct");
            assert!(
                out.contains("host=cdn.example.com"),
                "roundtrip keeps host for {url}: {out}"
            );
        }
    }

    // ── Clash round-trip via *_proto ──────────────────────────────────────

    #[test]
    fn clash_roundtrip_from_url_via_proto() {
        let url = format!(
            "vless://{UUID}@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        );
        let parsed = parse(&url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        let proxy = cfg.to_clash_proto(&endpoint).expect("to clash");
        let reparsed = VlessConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(
            reparsed.endpoints[0], endpoint,
            "endpoint round-trips through clash"
        );
        assert_eq!(
            reparsed.protocol.config,
            ProtocolConfig::Vless(cfg),
            "config round-trips through clash"
        );
    }

    #[test]
    fn clash_proxy_roundtrip_via_proto() {
        use crate::clash::{ClashProxy, ClashVless, ClashWSOpts};

        let proxy = ClashProxy::Vless(ClashVless {
            name: "test".into(),
            server: "159.223.24.65".into(),
            port: 443,
            uuid: UUID.into(),
            udp: None,
            tfo: None,
            network: Some("ws".into()),
            flow: None,
            encryption: None,
            tls: Some(true),
            servername: Some("test.ir".into()),
            skip_cert_verify: None,
            alpn: None,
            reality_opts: None,
            ws_opts: Some(ClashWSOpts {
                path: Some("/ws".into()),
                headers: Some(std::collections::HashMap::from([(
                    "Host".to_string(),
                    "cdn.example.com".to_string(),
                )])),
                ..Default::default()
            }),
            grpc_opts: None,
            xhttp_opts: None,
        });
        let parsed = VlessConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(parsed.endpoints[0].host, "159.223.24.65");
        assert_eq!(parsed.endpoints[0].host_type, HostKind::Ipv4);
        assert_eq!(parsed.endpoints[0].port, 443);
        let cfg = match &parsed.protocol.config {
            ProtocolConfig::Vless(c) => c,
            other => panic!("expected VlessConfig, got {other:?}"),
        };
        // Explicit Clash ws Host header stays in the config (host-free mandate).
        if let TransportConfig::Ws(ws) = &cfg.transport {
            assert_eq!(ws.host.as_deref(), Some("cdn.example.com"));
            assert_eq!(ws.path.as_deref(), Some("/ws"));
        } else {
            panic!("expected ws transport");
        }
        assert_no_top_level_host_port(cfg);
        let out = cfg.to_clash_proto(&parsed.endpoints[0]).expect("to clash");
        match (out, proxy) {
            (ClashProxy::Vless(out), ClashProxy::Vless(orig)) => assert_eq!(out, orig),
            _ => panic!("expected vless clash proxy"),
        }
    }

    #[test]
    fn clash_roundtrip_via_proto_http_keeps_config_host_free() {
        // Regression (F1): the endpoint host must never leak into the identity
        // config through the clash export/import cycle.
        let url = format!("vless://{UUID}@example.com:443?type=http");
        let parsed = parse(&url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        if let TransportConfig::Http(h) = &cfg.transport {
            assert_eq!(h.host, None, "no explicit host -> config host unset");
        } else {
            panic!("expected http transport");
        }
        let proxy = cfg.to_clash_proto(&endpoint).expect("to clash");
        let reparsed = VlessConfig::try_from_clash_proto(&proxy).expect("clash parse");
        assert_eq!(
            reparsed.endpoints[0], endpoint,
            "endpoint round-trips through clash"
        );
        assert_eq!(
            reparsed.protocol.config,
            ProtocolConfig::Vless(cfg),
            "config must stay endpoint-free through the clash cycle"
        );
    }

    // ── HOST-FREE PARSE MANDATE: no server-address fallback ───────────────

    #[test]
    fn ws_host_param_stored_but_absent_param_stays_unset() {
        // Explicit `host=` is a protocol parameter and IS stored.
        let url = format!(
            "vless://{UUID}@example.com:443?security=tls&type=ws&host=cdn.example.com&path=%2Fws"
        );
        let parsed = parse(&url);
        let cfg = config(parsed);
        if let TransportConfig::Ws(ws) = &cfg.transport {
            assert_eq!(ws.host.as_deref(), Some("cdn.example.com"));
        } else {
            panic!("expected ws transport");
        }

        // Without `host=`, the ws host field must be UNSET — never the server.
        let url = format!("vless://{UUID}@example.com:443?security=tls&type=ws&path=%2Fws");
        let parsed = parse(&url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        if let TransportConfig::Ws(ws) = &cfg.transport {
            assert_eq!(ws.host, None, "ws host must not fall back to the server");
        } else {
            panic!("expected ws transport");
        }
        // Reconstruct must not invent a `host=` param for the absent one.
        let out = cfg.reconstruct_proto(&endpoint).expect("reconstruct");
        assert!(!out.contains("host="), "no host param emitted: {out}");
        assert_no_top_level_host_port(&cfg);
    }

    #[test]
    fn grpc_authority_and_httpupgrade_host_unset_without_param() {
        for url in [
            format!("vless://{UUID}@example.com:443?type=grpc&path=svc"),
            format!("vless://{UUID}@example.com:443?type=httpupgrade&path=%2Fup"),
        ] {
            let parsed = parse(&url);
            let cfg = config(parsed);
            match &cfg.transport {
                TransportConfig::Grpc(g) => {
                    assert_eq!(g.authority, None, "grpc authority must not be the server")
                }
                TransportConfig::HttpUpgrade(c) => {
                    assert_eq!(c.host, None, "httpupgrade host must not be the server")
                }
                other => panic!("unexpected transport for {url}: {other:?}"),
            }
        }
    }

    #[test]
    fn sni_param_stored_but_absent_sni_stays_unset() {
        // Explicit `sni=` is a protocol parameter and IS stored.
        let url =
            format!("vless://{UUID}@example.com:443?security=tls&type=tcp&sni=cdn.example.com");
        let cfg = config(parse(&url));
        assert_eq!(cfg.security.sni(), Some("cdn.example.com"));

        // No `sni=` param: security must not carry the server as sni.
        let url = format!("vless://{UUID}@example.com:443?security=tls&type=tcp");
        let cfg = config(parse(&url));
        assert_eq!(
            cfg.security.sni(),
            None,
            "sni must not fall back to the server"
        );
    }

    // ── Transport edge cases ──────────────────────────────────────────────

    #[test]
    fn test_vless_httpupgrade() {
        let url =
            format!("vless://{UUID}@host.com:443?type=httpupgrade&path=/test&host=myhost.com");
        let parsed = parse(&url);
        let cfg = config(parsed);
        assert_eq!(cfg.transport.type_str(), "httpupgrade");
        assert_reconstruct_roundtrip(&url);
    }

    #[test]
    fn test_vless_xhttp() {
        let url =
            format!("vless://{UUID}@host.com:443?type=xhttp&mode=auto&path=/test&host=myhost.com");
        let parsed = parse(&url);
        let cfg = config(parsed);
        assert_eq!(cfg.transport.type_str(), "xhttp");
        if let TransportConfig::XHttp(x) = &cfg.transport {
            assert_eq!(x.mode.as_deref(), Some("auto"));
            assert_eq!(x.host.as_deref(), Some("myhost.com"));
        } else {
            panic!("expected XHttp transport");
        }
        assert_reconstruct_roundtrip(&url);
    }

    #[test]
    fn test_vless_xhttp_bad_mode() {
        let url = format!("vless://{UUID}@host.com:443?type=xhttp&mode=badmode");
        assert!(VlessConfig::try_parse_proto(&RawUrlX::from(url.as_str())).is_err());
    }

    #[test]
    fn test_vless_xhttp_extra() {
        let url = format!(
            "vless://{UUID}@host.com:443?type=xhttp&mode=auto&path=/test&extra=%7B%22xPaddingBytes%22%3A%22100-1000%22%7D"
        );
        let parsed = parse(&url);
        let cfg = config(parsed);
        assert_eq!(cfg.transport.type_str(), "xhttp");
        if let TransportConfig::XHttp(x) = &cfg.transport {
            assert!(x.extra.is_some());
        } else {
            panic!("expected XHttp transport");
        }
        assert_reconstruct_roundtrip(&url);
    }

    // ── Short-string (uuidv5) handling ────────────────────────────────────

    #[test]
    fn test_vless_short_string_creates_uuidv5() {
        let parsed = parse("vless://somechannel@159.223.24.65:443?security=tls&type=tcp");
        let cfg = config(parsed);
        // uuid must be a valid UUID (generated by UUIDv5)
        assert!(
            uuid::Uuid::parse_str(&cfg.uuid).is_ok(),
            "generated uuid must be a valid UUID: {}",
            cfg.uuid
        );
        // uuid_origin must be the original short string
        assert_eq!(
            cfg.uuid_origin.as_deref(),
            Some("somechannel"),
            "uuid_origin should preserve the original short string"
        );
        // Verify the generated UUID matches UUIDv5 from nil namespace
        let expected = uuid::Uuid::new_v5(&uuid::Uuid::nil(), b"somechannel").to_string();
        assert_eq!(
            cfg.uuid, expected,
            "uuid should be UUIDv5(nil, \"somechannel\")"
        );
    }

    #[test]
    fn test_vless_short_string_roundtrip_preserves_origin() {
        let url = "vless://somechannel@159.223.24.65:443?security=tls&type=tcp";
        let parsed = parse(url);
        let endpoint = parsed.endpoints[0].clone();
        let cfg = config(parsed);
        // Reconstruct: the URL should contain the original short string, not the generated UUID
        let reconstructed = cfg.reconstruct_proto(&endpoint).expect("reconstruct");
        assert!(
            reconstructed.contains("somechannel@"),
            "reconstructed URL should contain the original short string: {reconstructed}"
        );
        // Re-parse the reconstructed URL
        let reparsed = parse(&reconstructed);
        assert_eq!(reparsed.endpoints[0], endpoint, "endpoint round-trip");
        let rcfg = config(reparsed);
        // uuid_origin should be preserved through roundtrip
        assert_eq!(
            rcfg.uuid_origin.as_deref(),
            Some("somechannel"),
            "uuid_origin should survive roundtrip"
        );
        assert_eq!(rcfg.uuid, cfg.uuid, "uuid should match");
    }

    #[test]
    fn test_vless_normal_uuid_has_no_uuid_origin() {
        // Standard UUIDs should have uuid_origin = None
        let url = format!("vless://{UUID}@159.223.24.65:443?type=tcp");
        let cfg = config(parse(&url));
        assert!(
            cfg.uuid_origin.is_none(),
            "normal UUID should not set uuid_origin"
        );
        assert_eq!(cfg.uuid, UUID);
    }

    // ── Legacy trait bridge ───────────────────────────────────────────────

    #[test]
    fn legacy_bridge_parse_works_but_reconstruct_to_clash_error() {
        let url = format!("vless://{UUID}@example.com:443?type=tcp");
        // try_parse still extracts the config (endpoints discarded).
        let bridged = VlessConfig::try_parse(&RawUrlX::from(url.as_str())).expect("bridged parse");
        assert_eq!(bridged.schema(), SchemeX::Vless);
        assert_eq!(bridged.uuid, UUID);
        // host/port accessors are gone — the endpoint lives on ParsedProto.
        assert_eq!(bridged.host(), None);
        assert_eq!(bridged.port(), None);
        // Degraded legacy paths error instead of fabricating a host.
        assert!(bridged.reconstruct().is_err());
        assert!(bridged.to_clash().is_err());
    }

    #[test]
    fn legacy_bridge_try_from_clash_extracts_config() {
        use crate::clash::{ClashProxy, ClashVless};

        let proxy = ClashProxy::Vless(ClashVless {
            name: "test".into(),
            server: "example.com".into(),
            port: 443,
            uuid: UUID.into(),
            udp: None,
            tfo: None,
            network: Some("ws".into()),
            flow: None,
            encryption: None,
            tls: Some(true),
            servername: Some("test.ir".into()),
            skip_cert_verify: None,
            alpn: None,
            reality_opts: None,
            ws_opts: None,
            grpc_opts: None,
            xhttp_opts: None,
        });
        // try_from_clash delegates to try_from_clash_proto and extracts the
        // config (endpoints discarded).
        let bridged = VlessConfig::try_from_clash(&proxy).expect("bridged clash parse");
        assert_eq!(bridged.uuid, UUID);
        assert_eq!(bridged.security.sni(), Some("test.ir"));
        // host/port accessors are gone even via the bridge.
        assert_eq!(bridged.host(), None);
        assert_eq!(bridged.port(), None);
    }

    #[test]
    fn serialized_config_has_no_host_or_port_keys() {
        for url in [
            format!("vless://{UUID}@159.223.24.65:443?type=tcp"),
            format!("vless://{UUID}@example.com:443?type=ws&path=%2F&host=cdn.example.com"),
            format!("vless://{UUID}@example.com:443?type=grpc&path=svc&security=tls&sni=x"),
        ] {
            let cfg = config(parse(&url));
            assert_no_top_level_host_port(&cfg);
        }
    }

    // ── Xray inject_to (Task 14) ──────────────────────────────────────────

    use super::super::{EndpointEssentials, InjectOptions, InjectToCoreConf, SupportError};

    fn vless_ws_tls() -> VlessConfig {
        config(parse(&format!(
            "vless://{UUID}@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws"
        )))
    }

    fn endpoint() -> EndpointEssentials {
        EndpointEssentials::new("159.223.24.65", 443)
    }

    #[test]
    fn xray_inject_writes_proxy_outbound() {
        let cfg = vless_ws_tls();
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::Xray,
            Some(&endpoint()),
            InjectOptions::default(),
        )
        .expect("vless inject");
        assert_eq!(conf["tag"], "proxy");
        assert_eq!(conf["protocol"], "vless");
        let users = &conf["settings"]["vnext"][0]["users"][0];
        assert_eq!(users["id"], UUID);
        assert_eq!(users["encryption"], "none");
        assert_eq!(users["flow"], "");
        assert_eq!(conf["settings"]["vnext"][0]["address"], "159.223.24.65");
        assert_eq!(conf["settings"]["vnext"][0]["port"], 443);
        // streamSettings == to_xray_stream_settings output
        let expected = super::super::common::to_xray_stream_settings(
            &cfg.security,
            &cfg.transport
                .clone()
                .with_host(Some("159.223.24.65".into()), None, None),
        )
        .expect("ws+tls stream settings");
        assert_eq!(conf["streamSettings"], expected);
        assert_eq!(conf["streamSettings"]["network"], "ws");
        assert_eq!(conf["streamSettings"]["security"], "tls");
    }

    #[test]
    fn xray_inject_flow_and_host_free_injection() {
        // flow present; ws host unset → filled from endpoint at build time
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:8443?security=tls&encryption=none&type=ws&path=%2Fws&flow=xtls-rprx-vision"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::Xray,
            Some(&EndpointEssentials::new("example.com", 8443)),
            InjectOptions::default(),
        )
        .expect("vless inject");
        assert_eq!(
            conf["settings"]["vnext"][0]["users"][0]["flow"],
            "xtls-rprx-vision"
        );
        assert_eq!(conf["streamSettings"]["wsSettings"]["host"], "example.com");
    }

    #[test]
    fn xray_inject_skip_cert_verify_forces_allow_insecure() {
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?security=tls&encryption=none&type=tcp"
        )));
        assert_eq!(cfg.security.insecure(), None);
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::Xray,
            Some(&EndpointEssentials::new("example.com", 443)),
            InjectOptions {
                skip_cert_verify: true,
            },
        )
        .expect("vless inject");
        assert_eq!(conf["streamSettings"]["security"], "tls");
        assert_eq!(conf["streamSettings"]["tlsSettings"]["allowInsecure"], true);
    }

    #[test]
    fn xray_inject_reality_without_public_key_is_rejected() {
        // Restores the T13-flipped reality semantics at the proto level:
        // reality without publicKey/serverName must fail at build time.
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?security=reality&encryption=none&type=tcp&flow=xtls-rprx-vision"
        )));
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(
                &mut conf,
                CoreType::Xray,
                Some(&endpoint()),
                InjectOptions::default(),
            )
            .expect_err("reality without pbk/sni must be rejected");
        assert!(
            err.to_string().contains("reality"),
            "error must mention reality: {err}"
        );
    }

    #[test]
    fn xray_inject_without_endpoint_is_rejected() {
        let cfg = vless_ws_tls();
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(&mut conf, CoreType::Xray, None, InjectOptions::default())
            .expect_err("orphan vless must be rejected");
        assert!(matches!(err, SupportError::MissingField("server", "vless")));
    }

    #[test]
    fn singbox_inject_writes_proxy_outbound() {
        let cfg = vless_ws_tls();
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&endpoint()),
            InjectOptions::default(),
        )
        .expect("vless sing-box inject");
        assert_eq!(conf["tag"], "proxy");
        assert_eq!(conf["type"], "vless");
        assert_eq!(conf["server"], "159.223.24.65");
        assert_eq!(conf["server_port"], 443);
        assert_eq!(conf["uuid"], UUID);
        assert!(conf.get("flow").is_none(), "no flow -> key omitted");
        // tls via the shared helper: sni from the config.
        assert_eq!(conf["tls"]["enabled"], true);
        assert_eq!(conf["tls"]["server_name"], "test.ir");
    }

    #[test]
    fn singbox_inject_flow_and_skip_cert_verify() {
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:8443?security=tls&encryption=none&type=tcp&flow=xtls-rprx-vision"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("example.com", 8443)),
            InjectOptions {
                skip_cert_verify: true,
            },
        )
        .expect("vless sing-box inject");
        assert_eq!(conf["flow"], "xtls-rprx-vision");
        // skip_cert_verify forces tls.insecure; server_name falls back to the
        // endpoint host at build time.
        assert_eq!(conf["tls"]["insecure"], true);
        assert_eq!(conf["tls"]["server_name"], "example.com");
    }

    #[test]
    fn singbox_inject_reality_block() {
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?security=reality&encryption=none&type=tcp&flow=xtls-rprx-vision&pbk=REALITY_PUBLIC_KEY&sid=abc123&fp=chrome&spx=%2Fpath"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&endpoint()),
            InjectOptions::default(),
        )
        .expect("vless sing-box inject");
        assert_eq!(conf["tls"]["reality"]["enabled"], true);
        assert_eq!(conf["tls"]["reality"]["public_key"], "REALITY_PUBLIC_KEY");
        assert_eq!(conf["tls"]["reality"]["short_id"], "abc123");
        // NO spider_x: sing-box OutboundRealityOptions has no such field.
        assert!(
            conf["tls"]["reality"].get("spider_x").is_none(),
            "spider_x must never be emitted: {}",
            conf
        );
        assert_eq!(conf["tls"]["utls"]["fingerprint"], "chrome");
    }

    #[test]
    fn singbox_inject_without_endpoint_is_rejected() {
        let cfg = vless_ws_tls();
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(&mut conf, CoreType::SingBox, None, InjectOptions::default())
            .expect_err("orphan vless must be rejected");
        assert!(matches!(err, SupportError::MissingField("server", "vless")));
    }

    #[test]
    fn singbox_inject_ws_transport_with_endpoint_host() {
        // ws host unset by the host-free parse mandate -> endpoint host fills
        // the sing-box headers.Host at build time (V2RayTransportOptions ws).
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:8443?security=tls&encryption=none&type=ws&path=%2Fws"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("example.com", 8443)),
            InjectOptions::default(),
        )
        .expect("vless ws sing-box inject");
        let transport = &conf["transport"];
        assert_eq!(transport["type"], "ws");
        assert_eq!(transport["path"], "/ws");
        assert_eq!(transport["headers"]["Host"], "example.com");
    }

    #[test]
    fn singbox_inject_httpupgrade_transport() {
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?type=httpupgrade&path=%2Fup&host=cdn.example.com"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("example.com", 443)),
            InjectOptions::default(),
        )
        .expect("vless httpupgrade sing-box inject");
        let transport = &conf["transport"];
        assert_eq!(transport["type"], "httpupgrade");
        assert_eq!(transport["path"], "/up");
        // explicit host param is a protocol parameter (not overwritten by the
        // endpoint host).
        assert_eq!(transport["host"], "cdn.example.com");
    }

    #[test]
    fn singbox_inject_quic_transport() {
        let cfg = config(parse(&format!("vless://{UUID}@example.com:443?type=quic")));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("example.com", 443)),
            InjectOptions::default(),
        )
        .expect("vless quic sing-box inject");
        assert_eq!(conf["transport"]["type"], "quic");
    }

    #[test]
    fn singbox_inject_tcp_omits_transport() {
        // Tcp is the sing-box default — no transport key (like the old
        // builder and the v2ray-core "no TCP transport" doc note).
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?security=tls&encryption=none&type=tcp"
        )));
        let mut conf = serde_json::json!({});
        cfg.inject_to(
            &mut conf,
            CoreType::SingBox,
            Some(&EndpointEssentials::new("example.com", 443)),
            InjectOptions::default(),
        )
        .expect("vless tcp sing-box inject");
        assert!(
            conf.get("transport").is_none(),
            "tcp must not emit a transport key: {conf}"
        );
    }

    #[test]
    fn singbox_inject_kcp_is_rejected() {
        // mKCP is not in the vendored sing-box transport set — build-time
        // refusal so the config is never written invalid.
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?type=kcp&path=seed"
        )));
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(
                &mut conf,
                CoreType::SingBox,
                Some(&EndpointEssentials::new("example.com", 443)),
                InjectOptions::default(),
            )
            .expect_err("kcp must be refused by sing-box");
        assert!(
            err.to_string().contains("kcp"),
            "error must mention kcp: {err}"
        );
        assert!(matches!(err, SupportError::Config(_)));
    }

    #[test]
    fn singbox_inject_xhttp_is_rejected() {
        // XHTTP is not in the vendored sing-box transport set — build-time
        // refusal.
        let cfg = config(parse(&format!(
            "vless://{UUID}@example.com:443?type=xhttp&path=%2Fs"
        )));
        let mut conf = serde_json::json!({});
        let err = cfg
            .inject_to(
                &mut conf,
                CoreType::SingBox,
                Some(&EndpointEssentials::new("example.com", 443)),
                InjectOptions::default(),
            )
            .expect_err("xhttp must be refused by sing-box");
        assert!(
            err.to_string().contains("xhttp"),
            "error must mention xhttp: {err}"
        );
        assert!(matches!(err, SupportError::Config(_)));
    }
}
