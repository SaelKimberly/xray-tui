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

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::common::{
    RealityOpts, SecurityConfig, TlsConfig, TlsOpts, TransportConfig, should_skip_param,
};
use super::impl_sig_cache;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashProxy, ClashVless};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{
    clash_alpn_as_str, clash_server_to_host, clash_tls_to_security, clash_transport_to_transport,
    host_spec_to_string, security_to_clash_reality, security_to_clash_tls, transport_to_clash,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct VlessConfig {
    #[serde(skip)]
    sig_cache: std::sync::OnceLock<NonZeroU64>,
    #[serde(skip)]
    cred_hash_cache: std::sync::OnceLock<u64>,

    pub uuid: String,
    pub uuid_origin: Option<TinyText>,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub transport: TransportConfig,
    pub encryption: Option<TinyText>,
    pub flow: Option<TinyText>,
    pub path: Option<TinyText>,
    pub splice: Option<bool>,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for VlessConfig {
    /// Parse a VLESS URL (standard URI format).
    ///
    /// UUID is extracted from userinfo, server address from host:port,
    /// all configuration from query parameters, remarks from fragment.
    ///
    /// Supports combined `userinfo@hostport` or separate hostport components.
    /// UUID validated via `uuid::Uuid::parse_str`.
    #[allow(clippy::too_many_lines)]
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let (username, hostport) = if let Some(hostport) = raw.hostport {
            (raw.userinfo, hostport)
        } else {
            let userinfo = raw.userinfo;
            let (username, hostport) = userinfo.split_once('@').ok_or_else(|| {
                ParseError::InvalidUserInfo(format!("{userinfo}: missing hostport").into())
            })?;
            (username, hostport)
        };

        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;
        let (uuid, uuid_origin) = if uuid::Uuid::parse_str(username).is_ok() {
            (username.to_string(), None)
        } else {
            let generated = uuid::Uuid::new_v5(&uuid::Uuid::nil(), username.as_bytes()).to_string();
            (generated, Some(TinyText::from(username)))
        };

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

        let host = utils::query_get(&query, "host").map(str::to_string);
        let sni_from_query = utils::query_get(&query, "sni").map(str::to_string);
        let server_addr = Some(parsed_host.to_str().into_owned());

        let mut transport =
            TransportConfig::from_type_and_path(Some(&transport_type), path.as_deref())?
                .unwrap_or(TransportConfig::Tcp);
        transport = transport.with_host(host, sni_from_query, server_addr);

        // Extract mode and extra for XHttp, validate mode
        if let TransportConfig::XHttp(ref mut xcfg) = transport {
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

        let path = match transport {
            TransportConfig::Ws(ref ws) => ws.path.clone(),
            TransportConfig::Grpc(ref g) => g.path.clone(),
            TransportConfig::Http(ref h) => h.path.clone(),
            TransportConfig::HttpUpgrade(ref cfg) => cfg.path.clone(),
            TransportConfig::XHttp(ref cfg) => cfg.path.clone(),
            _ => path,
        };

        Ok(Self {
            sig_cache: std::sync::OnceLock::new(),
            cred_hash_cache: std::sync::OnceLock::new(),
            uuid,
            uuid_origin,
            host: parsed_host,
            port: parsed_port,
            transport,
            security,
            encryption,
            flow,
            path,
            splice,
            remarks,
        })
    }

    #[allow(clippy::too_many_lines)]
    fn reconstruct(&self) -> Result<String, ParseError> {
        let host = self.host.to_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", self.port)
        } else {
            format!("{host}:{}", self.port)
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
                            && !should_skip_param(&self.host, v)
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
                            && !should_skip_param(&self.host, v)
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
                TransportConfig::HttpUpgrade(cfg) => {
                    if let Some(ref host) = cfg.host
                        && !should_skip_param(&self.host, host)
                    {
                        q.append_pair("host", host);
                    }
                }
                TransportConfig::XHttp(cfg) => {
                    if let Some(ref host) = cfg.host
                        && !should_skip_param(&self.host, host)
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

    fn schema(&self) -> SchemeX {
        SchemeX::Vless
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
        *self.cred_hash_cache.get_or_init(|| {
            utils::compute_cred_hash(&[
                ("uuid", self.uuid.as_str()),
                ("pbk", self.security.pbk().unwrap_or("")),
                ("sid", self.security.sid().unwrap_or("")),
            ])
        })
    }

    fn set_cred_hash_cache(&self, v: u64) {
        _ = self.cred_hash_cache.set(v);
    }

    impl_sig_cache!();

    fn transport_type(&self) -> Option<&str> {
        Some(self.transport.type_str())
    }

    fn security(&self) -> Option<&SecurityConfig> {
        Some(&self.security)
    }
    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Vless(c) => {
                let security = clash_tls_to_security(
                    c.tls,
                    c.servername.as_deref(),
                    c.skip_cert_verify,
                    clash_alpn_as_str(&c.alpn),
                    None,
                    c.reality_opts.as_ref(),
                );
                let transport = clash_transport_to_transport(
                    c.network.as_deref(),
                    &c.ws_opts,
                    &c.grpc_opts,
                    &None,
                    &None,
                    &None,
                    Some(&c.server),
                );
                let path = match &transport {
                    TransportConfig::Ws(ws) => ws.path.clone(),
                    TransportConfig::Grpc(g) => g.path.clone(),
                    TransportConfig::Http(h) => h.path.clone(),
                    TransportConfig::HttpUpgrade(cfg) => cfg.path.clone(),
                    TransportConfig::XHttp(cfg) => cfg.path.clone(),
                    _ => None,
                };
                Ok(Self {
                    sig_cache: std::sync::OnceLock::new(),
                    cred_hash_cache: std::sync::OnceLock::new(),
                    uuid: c.uuid.clone(),
                    uuid_origin: None,
                    host: clash_server_to_host(&c.server)?,
                    port: c.port,
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
                })
            }
            _ => Err(ParseError::Unknown("expected vless clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        let server = host_spec_to_string(&self.host);
        let alpn_str = self.security.alpn();
        let (network, ws_opts, grpc_opts, _, _, _) = transport_to_clash(&self.transport, &server);
        let (tls, servername, skip_cert_verify, _, _) = security_to_clash_tls(&self.security);
        let reality_opts = security_to_clash_reality(&self.security);
        Ok(ClashProxy::Vless(ClashVless {
            name,
            server,
            port: self.port,
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
}

impl VlessConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"vless");
        let sec_type = self.security.type_str().unwrap_or("none");
        hasher.write(sec_type.as_bytes());
        hasher.write(self.transport.type_str().as_bytes());
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
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

#[cfg(test)]
mod tests {
    use super::super::ProtoSpec;
    use crate::urlx::PortSpec;
    use crate::urlx::SchemeX;

    #[test]
    fn test_vless_basic() {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Vless);
        assert_eq!(
            config.host().map(|h| h.to_str()),
            Some("159.223.24.65".into())
        );
        assert_eq!(config.uuid, "6202b230-417c-4d8e-b624-0f71afa9c75d");
    }

    #[test]
    fn test_vless_reality() {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?security=reality&encryption=none&type=tcp&flow=xtls-rprx-vision&pbk=abc123";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::Vless);
        assert_eq!(config.flow.as_deref(), Some("xtls-rprx-vision"));
        if let Some(super::TlsConfig::Reality(ref opts)) = config.security.tls {
            assert_eq!(opts.pbk.as_deref(), Some("abc123"));
        } else {
            panic!("expected reality config");
        }
    }

    #[test]
    fn test_reconstruct_vless_roundtrip() {
        let input = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = VlessConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = VlessConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(parsed.uuid, reparsed.uuid, "uuid mismatch");
    }

    #[test]
    fn test_vless_serde_roundtrip() {
        let input = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = VlessConfig::try_parse(&raw).expect("failed to parse");

        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: VlessConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.host, deserialized.host, "host mismatch");
        assert_eq!(parsed.port, deserialized.port, "port mismatch");
        assert_eq!(parsed.uuid, deserialized.uuid, "uuid mismatch");
    }

    use super::super::test_helpers::check_roundtrip;
    use super::VlessConfig;

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<VlessConfig>(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws",
        );
        check_roundtrip::<VlessConfig>(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host:443?type=ws&path=%2F",
        );
        check_roundtrip::<VlessConfig>("vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host:443");
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<VlessConfig>(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws",
        );
    }

    #[test]
    fn vless_reality_sig_excludes_pbk_sid_cred_hash_includes_them() {
        let url_a = "vless://11111111-2222-3333-4444-555555555555@a.example.com:443?security=reality&pbk=AAAA&sid=1111&spx=%2F&fp=chrome#r";
        let url_b = "vless://11111111-2222-3333-4444-555555555555@a.example.com:443?security=reality&pbk=BBBB&sid=2222&spx=%2F&fp=chrome#r";
        let url_c = "vless://22222222-3333-4444-5555-666666666666@a.example.com:443?security=reality&pbk=AAAA&sid=1111&spx=%2F&fp=chrome#r";
        let a = VlessConfig::try_parse(&crate::urlx::RawUrlX::from(url_a)).unwrap();
        let b = VlessConfig::try_parse(&crate::urlx::RawUrlX::from(url_b)).unwrap();
        let c = VlessConfig::try_parse(&crate::urlx::RawUrlX::from(url_c)).unwrap();
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

    #[test]
    fn test_vless_httpupgrade() {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host.com:443?type=httpupgrade&path=/test&host=myhost.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("httpupgrade parse failed");
        assert_eq!(config.transport.type_str(), "httpupgrade");
        check_roundtrip::<VlessConfig>(url);
    }

    #[test]
    fn test_vless_xhttp() {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host.com:443?type=xhttp&mode=auto&path=/test&host=myhost.com";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("xhttp parse failed");
        assert_eq!(config.transport.type_str(), "xhttp");
        check_roundtrip::<VlessConfig>(url);
    }

    #[test]
    fn test_vless_xhttp_bad_mode() {
        let url =
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host.com:443?type=xhttp&mode=badmode";
        let raw = crate::urlx::RawUrlX::from(url);
        assert!(VlessConfig::try_parse(&raw).is_err());
    }

    #[test]
    fn test_vless_xhttp_extra() {
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host.com:443?type=xhttp&mode=auto&path=/test&extra=%7B%22xPaddingBytes%22%3A%22100-1000%22%7D";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("xhttp+extra parse failed");
        assert_eq!(config.transport.type_str(), "xhttp");
        if let super::TransportConfig::XHttp(ref xcfg) = config.transport {
            assert!(xcfg.extra.is_some());
        } else {
            panic!("expected XHttp transport");
        }
    }

    #[test]
    fn test_vless_httpupgrade_host_fallback() {
        let url =
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@host.com:443?type=httpupgrade&path=/test";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("httpupgrade no host parse failed");
        assert_eq!(config.transport.type_str(), "httpupgrade");
        if let super::TransportConfig::HttpUpgrade(ref cfg) = config.transport {
            assert_eq!(cfg.host.as_deref(), Some("host.com"));
        } else {
            panic!("expected HttpUpgrade transport");
        }
    }

    #[test]
    fn test_vless_short_string_creates_uuidv5() {
        let url = "vless://somechannel@159.223.24.65:443?security=tls&type=tcp";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("short string should parse successfully");
        // uuid must be a valid UUID (generated by UUIDv5)
        assert!(
            uuid::Uuid::parse_str(&config.uuid).is_ok(),
            "generated uuid must be a valid UUID: {}",
            config.uuid
        );
        // uuid_origin must be the original short string
        assert_eq!(
            config.uuid_origin.as_deref(),
            Some("somechannel"),
            "uuid_origin should preserve the original short string"
        );
        // Verify the generated UUID matches UUIDv5 from nil namespace
        let expected = uuid::Uuid::new_v5(&uuid::Uuid::nil(), b"somechannel").to_string();
        assert_eq!(
            config.uuid, expected,
            "uuid should be UUIDv5(nil, \"somechannel\")"
        );
    }

    #[test]
    fn test_vless_short_string_roundtrip_preserves_origin() {
        let url = "vless://somechannel@159.223.24.65:443?security=tls&type=tcp";
        let raw = crate::urlx::RawUrlX::from(url);
        let parsed = VlessConfig::try_parse(&raw).expect("parse short string");
        // Reconstruct: the URL should contain the original short string, not the generated UUID
        let reconstructed = parsed.reconstruct().expect("reconstruct");
        assert!(
            reconstructed.contains("somechannel@"),
            "reconstructed URL should contain the original short string: {reconstructed}"
        );
        // Re-parse the reconstructed URL
        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = VlessConfig::try_parse(&raw2).expect("reparse");
        // uuid_origin should be preserved through roundtrip
        assert_eq!(
            reparsed.uuid_origin.as_deref(),
            Some("somechannel"),
            "uuid_origin should survive roundtrip"
        );
        assert_eq!(reparsed.uuid, parsed.uuid, "uuid should match");
    }

    #[test]
    fn test_vless_short_string_serde_roundtrip() {
        let url = "vless://somechannel@159.223.24.65:443?security=tls&type=tcp";
        let raw = crate::urlx::RawUrlX::from(url);
        let parsed = VlessConfig::try_parse(&raw).expect("parse short string");

        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: VlessConfig = serde_json::from_str(&json).expect("deserialize");

        // uuid_origin should survive serde
        assert_eq!(
            deserialized.uuid_origin.as_deref(),
            Some("somechannel"),
            "uuid_origin should survive serde roundtrip"
        );
        assert_eq!(deserialized.uuid, parsed.uuid, "uuid should match");

        // Reconstruct from deserialized should also use the original short string
        let reconstructed = deserialized.reconstruct().expect("reconstruct after serde");
        assert!(
            reconstructed.contains("somechannel@"),
            "reconstructed URL after serde should contain original string: {reconstructed}"
        );
    }

    #[test]
    fn test_vless_normal_uuid_has_no_uuid_origin() {
        // Standard UUIDs should have uuid_origin = None
        let url = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?type=tcp";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = VlessConfig::try_parse(&raw).expect("normal UUID parse");
        assert!(
            config.uuid_origin.is_none(),
            "normal UUID should not set uuid_origin"
        );
        assert_eq!(config.uuid, "6202b230-417c-4d8e-b624-0f71afa9c75d");
        // Roundtrip should work as before
        let reconstructed = config.reconstruct().expect("reconstruct");
        assert!(
            reconstructed.contains("6202b230-417c-4d8e-b624-0f71afa9c75d@"),
            "reconstructed URL should contain the UUID"
        );
    }
}
