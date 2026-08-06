use crate::clash::{
    ClashGrpcOpts, ClashH2Opts, ClashHttpOpts, ClashKcpOpts, ClashRealityOpts, ClashWSOpts,
};
use crate::proto_spec::{EndpointEssentials, HostKind, ParseError, SupportError};
use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::urlx::{HostSpec, TinyText};

/// Clash TLS fields as returned by `security_to_clash_tls`.
type SecurityClashTls = (
    Option<bool>,
    Option<String>,
    Option<bool>,
    Option<String>,
    Option<String>,
);

/// Clash transport fields as returned by `transport_to_clash`.
type TransportClash = (
    Option<String>,
    Option<ClashWSOpts>,
    Option<ClashGrpcOpts>,
    Option<ClashH2Opts>,
    Option<ClashHttpOpts>,
    Option<ClashKcpOpts>,
);

// ========================================
// Transport Configurations
// ========================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransportConfig {
    Tcp,
    Ws(WebSocketConfig),
    Grpc(GrpcConfig),
    Http(HttpConfig),
    Quic,
    Kcp(KcpConfig),
    HttpUpgrade(HttpUpgradeConfig),
    XHttp(XHttpConfig),
}

impl TransportConfig {
    #[must_use]
    pub const fn type_str(&self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Ws(_) => "ws",
            Self::Grpc(_) => "grpc",
            Self::Http(_) => "http",
            Self::Quic => "quic",
            Self::Kcp(_) => "kcp",
            Self::HttpUpgrade(_) => "httpupgrade",
            Self::XHttp(_) => "xhttp",
        }
    }
    fn recover_transport_type(input: &str) -> Option<&'static str> {
        // Sorted by length descending, so longest prefix matches first
        const KNOWN: &[&str] = &[
            "httpupgrade",
            "splithttp",
            "websocket",
            "https",
            "xhttp",
            "grpc",
            "http",
            "mkcp",
            "quic",
            "kcp",
            "raw",
            "tcp",
            "h2",
            "ws",
        ];
        let lower = input.to_ascii_lowercase();
        // Exact match (case-insensitive) via eq_ignore_ascii_case
        for &known in KNOWN {
            if known.eq_ignore_ascii_case(input) {
                return Some(known);
            }
        }
        // Longest prefix match
        KNOWN
            .iter()
            .find(|&&known| lower.starts_with(known))
            .copied()
            .map(|v| v as _)
    }

    /// # Errors
    ///
    /// Return `ParseError`, if `protocol_type` is invalid and could not be recovered.
    pub fn from_type_and_path(
        protocol_type: Option<&str>,
        path: Option<&str>,
    ) -> Result<Option<Self>, ParseError> {
        match protocol_type {
            None | Some("") => Ok(None),
            // "auto"/"none" on `net` field are common mistakes from share link
            // generators confusing `net` (transport) with `scy` (security) or
            // `type` (header type). Default to TCP like mihomo does.
            Some("tcp" | "raw" | "auto" | "none") => Ok(Some(Self::Tcp)),
            Some("ws" | "websocket") => Ok(Some(Self::Ws(WebSocketConfig {
                path: path.map(TinyText::from),
                ..WebSocketConfig::default()
            }))),
            Some("grpc") => Ok(Some(Self::Grpc(GrpcConfig {
                path: path.map(TinyText::from),
                ..GrpcConfig::default()
            }))),
            Some("http" | "h2" | "https") => Ok(Some(Self::Http(HttpConfig {
                path: path.map(TinyText::from),
                ..HttpConfig::default()
            }))),
            Some("quic") => Ok(Some(Self::Quic)),
            Some("kcp" | "mkcp") => Ok(Some(Self::Kcp(KcpConfig::default()))),
            Some("httpupgrade") => Ok(Some(Self::HttpUpgrade(HttpUpgradeConfig {
                path: Some(TinyText::from(path.unwrap_or("/"))),
                ..HttpUpgradeConfig::default()
            }))),
            Some("xhttp" | "splithttp") => Ok(Some(Self::XHttp(XHttpConfig {
                path: Some(TinyText::from(path.unwrap_or("/"))),
                mode: Some(TinyText::from("auto")),
                ..XHttpConfig::default()
            }))),

            Some(other) =>  Self::recover_transport_type(other).map_or_else(
                ||Err(ParseError::InvalidConf(
                    "type".into(),
                    other.to_string().into(),
                )), |recovered| {
                    tracing::warn!(target: "proto_spec::common", transport = %other, recovered = %recovered, "Recovered transport type");
                    Self::from_type_and_path(Some(recovered), path)
                })

        }
    }

    #[must_use]
    pub fn with_host(
        self,
        host: Option<String>,
        sni: Option<String>,
        server_addr: Option<String>,
    ) -> Self {
        let resolved: Option<TinyText> = host.or(sni).or(server_addr).map(TinyText::from);
        match self {
            Self::Ws(cfg) => Self::Ws(WebSocketConfig {
                host: cfg.host.or(resolved),
                ..cfg
            }),
            Self::Grpc(cfg) => Self::Grpc(GrpcConfig {
                authority: cfg.authority.or(resolved),
                ..cfg
            }),
            Self::Http(cfg) => Self::Http(HttpConfig {
                host: cfg.host.or(resolved),
                ..cfg
            }),
            Self::HttpUpgrade(cfg) => Self::HttpUpgrade(HttpUpgradeConfig {
                host: cfg.host.or(resolved),
                ..cfg
            }),
            Self::XHttp(cfg) => Self::XHttp(XHttpConfig {
                host: cfg.host.or(resolved),
                ..cfg
            }),
            other => other,
        }
    }
}

/// Returns `true` when `value` equals the endpoint host's DNS name
/// (case-insensitive). When the endpoint is an IP address, always returns
/// `false` — sni/host must be emitted because the IP address cannot serve as
/// a TLS SNI/HTTP Host value. Takes the endpoint host *string*, which is all
/// the `*_proto` reconstruct path has (`EndpointEssentials::host`).
pub(crate) fn should_skip_endpoint_param(endpoint_host: &str, value: &str) -> bool {
    if endpoint_host.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }
    endpoint_host.eq_ignore_ascii_case(value)
}

/// Endpoint host kind for a parsed [`HostSpec`] — the rule every URL parser
/// uses: Ipv4/Ipv6 when the host parses as an IP address, Dns otherwise
/// (`Undefined` only when there is no host at all, which vless/vmess never
/// hit).
pub(crate) fn host_kind_for(host: &HostSpec) -> HostKind {
    match host {
        HostSpec::IpAddress(rustls::pki_types::IpAddr::V4(_)) => HostKind::Ipv4,
        HostSpec::IpAddress(rustls::pki_types::IpAddr::V6(_)) => HostKind::Ipv6,
        _ => HostKind::Dns,
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketConfig {
    pub path: Option<TinyText>,
    pub host: Option<TinyText>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub max_early_data: Option<u32>,
    pub early_data_header_name: Option<TinyText>,
    pub v2ray_http_upgrade: Option<bool>,
    pub v2ray_http_upgrade_fast_open: Option<bool>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct GrpcConfig {
    pub path: Option<TinyText>,
    pub authority: Option<TinyText>,
    pub service_name: Option<TinyText>,
    pub mode: Option<TinyText>,
    pub user_agent: Option<TinyText>,
    pub ping_interval: Option<u32>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct HttpConfig {
    pub path: Option<TinyText>,
    pub host: Option<TinyText>,
    pub method: Option<TinyText>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub idle_timeout: Option<u32>,
    pub ping_timeout: Option<u32>,
}
/// `HTTPUpgrade` transport config (fake WebSocket upgrade).
///
/// Sends HTTP GET with `Upgrade: websocket` → `101 Switching Protocols`,
/// then pipes raw bytes. No actual WebSocket framing.
///
/// Reference: `thirdparty/Xray-core/transport/internet/httpupgrade/config.proto`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct HttpUpgradeConfig {
    pub path: Option<TinyText>,
    pub host: Option<TinyText>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub ed: Option<u32>,
}
/// SplitHTTP/XHTTP transport config — full HTTP-based transport.
///
/// Supports 4 modes (`auto`, `packet-up`, `stream-up`, `stream-one`),
/// session-based multiplexing, `XPadding` obfuscation, separate download paths.
/// Extra fields from share link `extra=` JSON blob are stored raw.
///
/// Reference config proto: `thirdparty/Xray-core/transport/internet/splithttp/config.proto`
/// Reference client config: `thirdparty/mihomo/transport/xhttp/config.go`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct XHttpConfig {
    pub path: Option<TinyText>,
    pub host: Option<TinyText>,
    pub mode: Option<TinyText>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub extra: Option<Value>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct KcpConfig {
    pub mtu: Option<u32>,
    pub tti: Option<u32>,
    pub uplink_capacity: Option<u32>,
    pub downlink_capacity: Option<u32>,
    pub congestion: Option<bool>,
    pub read_buffer: Option<u32>,
    pub write_buffer: Option<u32>,
    pub seed: Option<TinyText>,
    pub header_type: Option<TinyText>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct QuicOpts {
    pub security: Option<TinyText>,
    pub key: Option<TinyText>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    #[serde(flatten)]
    pub tls: Option<TlsConfig>,
    pub enc: Option<TinyText>,
}

impl SecurityConfig {
    #[must_use]
    pub const fn type_str(&self) -> Option<&'static str> {
        match self.tls {
            None => None,
            Some(ref c @ (TlsConfig::Reality(_) | TlsConfig::Tls(_))) => Some(c.type_str()),
        }
    }
    #[must_use]
    pub fn sni(&self) -> Option<&str> {
        match self.tls {
            Some(
                TlsConfig::Tls(TlsOpts {
                    sni: Some(ref sni), ..
                })
                | TlsConfig::Reality(RealityOpts {
                    sni: Some(ref sni), ..
                }),
            ) => Some(sni.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn alpn(&self) -> Option<&str> {
        if let Some(TlsConfig::Tls(TlsOpts {
            alpn: Some(ref alpn),
            ..
        })) = self.tls
        {
            Some(alpn.as_str())
        } else {
            None
        }
    }

    #[must_use]
    pub fn fp(&self) -> Option<&str> {
        match self.tls {
            Some(
                TlsConfig::Tls(TlsOpts {
                    fp: Some(ref fp), ..
                })
                | TlsConfig::Reality(RealityOpts {
                    fp: Some(ref fp), ..
                }),
            ) => Some(fp.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub const fn insecure(&self) -> Option<bool> {
        if let Some(TlsConfig::Tls(TlsOpts { insecure, .. })) = self.tls {
            insecure
        } else {
            None
        }
    }

    #[must_use]
    pub const fn pbk(&self) -> Option<&str> {
        if let Some(TlsConfig::Reality(RealityOpts {
            pbk: Some(ref pbk), ..
        })) = self.tls
        {
            Some(pbk.as_str())
        } else {
            None
        }
    }

    #[must_use]
    pub fn sid(&self) -> Option<&str> {
        if let Some(TlsConfig::Reality(RealityOpts {
            sid: Some(ref sid), ..
        })) = self.tls
        {
            Some(sid.as_str())
        } else {
            None
        }
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.tls.is_none() && self.enc.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TlsConfig {
    Tls(TlsOpts),
    Reality(RealityOpts),
}

impl TlsConfig {
    #[must_use]
    pub const fn type_str(&self) -> &'static str {
        match self {
            Self::Tls(_) => "tls",
            Self::Reality(_) => "reality",
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct TlsOpts {
    pub sni: Option<TinyText>,
    pub alpn: Option<TinyText>,
    pub fp: Option<TinyText>,
    pub insecure: Option<bool>,
    pub pqv: Option<TinyText>,
    pub ech: Option<TinyText>,
    pub vcn: Option<bool>,
    pub pcs: Option<TinyText>,
    pub pin_sha256: Option<TinyText>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct RealityOpts {
    pub sni: Option<TinyText>,
    pub fp: Option<TinyText>,
    pub pbk: Option<String>,
    pub sid: Option<TinyText>,
    pub spx: Option<TinyText>,
}

/// Convert Clash TLS fields to a `SecurityConfig`.
pub(crate) fn clash_tls_to_security(
    tls: Option<bool>,
    servername: Option<&str>,
    skip_cert_verify: Option<bool>,
    alpn: Option<&str>,
    fingerprint: Option<&str>,
    reality_opts: Option<&ClashRealityOpts>,
) -> SecurityConfig {
    if tls.is_none() || tls == Some(false) {
        return SecurityConfig::default();
    }
    if let Some(r) = reality_opts {
        return SecurityConfig {
            tls: Some(TlsConfig::Reality(RealityOpts {
                sni: servername.map(TinyText::from),
                pbk: r.public_key.clone(),
                sid: r.short_id.clone().map(TinyText::from),
                fp: fingerprint.map(TinyText::from),
                ..RealityOpts::default()
            })),
            enc: None,
        };
    }
    SecurityConfig {
        tls: Some(TlsConfig::Tls(TlsOpts {
            sni: servername.map(TinyText::from),
            insecure: skip_cert_verify,
            alpn: alpn.map(TinyText::from),
            fp: fingerprint.map(TinyText::from),
            ..TlsOpts::default()
        })),
        enc: None,
    }
}

/// Convert a `SecurityConfig` back to Clash TLS fields.
pub(crate) fn security_to_clash_tls(security: &SecurityConfig) -> SecurityClashTls {
    match &security.tls {
        Some(TlsConfig::Tls(opts)) => (
            Some(true),
            opts.sni.as_ref().map(std::string::ToString::to_string),
            opts.insecure,
            opts.alpn.as_ref().map(std::string::ToString::to_string),
            opts.fp.as_ref().map(std::string::ToString::to_string),
        ),
        Some(TlsConfig::Reality(opts)) => (
            None,
            opts.sni.as_ref().map(std::string::ToString::to_string),
            None,
            None,
            opts.fp.as_ref().map(std::string::ToString::to_string),
        ),
        None => (None, None, None, None, None),
    }
}

/// Convert a `SecurityConfig` to Clash `reality_opts`.
pub(crate) fn security_to_clash_reality(security: &SecurityConfig) -> Option<ClashRealityOpts> {
    match &security.tls {
        Some(TlsConfig::Reality(opts)) => Some(ClashRealityOpts {
            public_key: opts.pbk.clone(),
            short_id: opts.sid.as_ref().map(std::string::ToString::to_string),
            support_x25519mlkem768: None,
        }),
        _ => None,
    }
}

/// Clash `server` string + `port` → [`EndpointEssentials`], using the same
/// host-kind rule as the URL parsers ([`host_kind_for`]). The raw server
/// string is stored verbatim so `to_clash_proto` round-trips it unchanged.
pub(crate) fn clash_to_endpoint(server: &str, port: u16) -> EndpointEssentials {
    let host_type = match server.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(_)) => HostKind::Ipv4,
        Ok(std::net::IpAddr::V6(_)) => HostKind::Ipv6,
        Err(_) => HostKind::Dns,
    };
    EndpointEssentials {
        host: server.to_string(),
        host_type,
        port,
        ports: vec![port],
    }
}

/// Convert Clash transport fields to a `TransportConfig`.
pub(crate) fn clash_transport_to_transport(
    network: Option<&str>,
    ws_opts: Option<&ClashWSOpts>,
    grpc_opts: Option<&ClashGrpcOpts>,
    h2_opts: Option<&ClashH2Opts>,
    _http_opts: Option<&ClashHttpOpts>,
    mkcp_opts: Option<&ClashKcpOpts>,
    server: Option<&str>,
) -> TransportConfig {
    match network {
        Some("ws" | "websocket") => {
            let Some(w) = ws_opts else {
                return TransportConfig::Tcp;
            };
            // The vhost lives in the dedicated `host` field, not in `headers`;
            // drop `Host` from the header map so the two never double-represent it.
            let headers = w.headers.as_ref().map(|h| {
                h.iter()
                    .filter(|(key, _)| !key.eq_ignore_ascii_case("host"))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<std::collections::HashMap<_, _>>()
            });
            let headers = headers.filter(|h| !h.is_empty());
            TransportConfig::Ws(WebSocketConfig {
                path: w.path.clone().map(TinyText::from),
                // Match the strip above case-insensitively, so a lowercase
                // `host` header key still restores the vhost.
                host: w
                    .headers
                    .as_ref()
                    .and_then(|h| {
                        h.iter()
                            .find(|(key, _)| key.eq_ignore_ascii_case("host"))
                            .map(|(_, value)| value.clone())
                    })
                    .map(TinyText::from),
                headers,
                max_early_data: w.max_early_data,
                early_data_header_name: w.early_data_header_name.clone().map(TinyText::from),
                v2ray_http_upgrade: w.v2ray_http_upgrade,
                v2ray_http_upgrade_fast_open: w.v2ray_http_upgrade_fast_open,
            })
        }
        Some("grpc") => TransportConfig::Grpc(GrpcConfig {
            path: grpc_opts
                .as_ref()
                .and_then(|g| g.grpc_service_name.clone())
                .map(TinyText::from),
            authority: None,
            service_name: grpc_opts
                .as_ref()
                .and_then(|g| g.grpc_service_name.clone())
                .map(TinyText::from),
            mode: None,
            user_agent: grpc_opts
                .as_ref()
                .and_then(|g| g.grpc_user_agent.clone())
                .map(TinyText::from),
            ping_interval: grpc_opts.and_then(|g| g.ping_interval),
        }),
        Some("h2" | "http") => {
            let h = h2_opts;
            TransportConfig::Http(HttpConfig {
                path: h.and_then(|h| h.path.clone()).map(TinyText::from),
                host: h
                    .and_then(|h| {
                        h.host
                            .as_ref()
                            .and_then(|v| v.first())
                            .map(std::string::String::as_str)
                    })
                    .or(server)
                    .map(|s| TinyText::from(s.to_string())),
                method: None,
                headers: None,
                idle_timeout: None,
                ping_timeout: None,
            })
        }
        Some("kcp" | "mkcp") => {
            let Some(k) = mkcp_opts else {
                return TransportConfig::Tcp;
            };
            TransportConfig::Kcp(KcpConfig {
                mtu: k.mtu,
                tti: k.tti,
                uplink_capacity: k.uplink_capacity,
                downlink_capacity: k.downlink_capacity,
                congestion: k.congestion,
                read_buffer: k.read_buffer,
                write_buffer: k.write_buffer,
                seed: k.seed.clone().map(TinyText::from),
                header_type: k.header.clone().map(TinyText::from),
            })
        }
        _ => TransportConfig::Tcp,
    }
}

/// Convert a `TransportConfig` back to Clash transport fields.
///
/// `server` is the OPT-IN fallback for the HTTP/h2 transport's `host` when the
/// config carries no explicit host. The `*_proto` export path passes `None` —
/// the endpoint host must never leak into Clash fields that re-import into the
/// identity-hashed config (host-free parse mandate). Legacy `to_clash` impls
/// (config still stores its host) pass `Some(config_host)` to preserve the
/// historical fallback.
pub(crate) fn transport_to_clash(
    transport: &TransportConfig,
    server: Option<&str>,
) -> TransportClash {
    match transport {
        TransportConfig::Ws(w) => {
            // Clash carries the WS vhost in `headers.Host`; forward `host` so the
            // Clash -> TransportConfig conversion can restore it.
            let mut headers = w.headers.clone().unwrap_or_default();
            if let Some(host) = &w.host {
                headers.insert("Host".to_string(), host.to_string());
            }
            let headers = (!headers.is_empty()).then_some(headers);
            (
                Some("ws".to_string()),
                Some(ClashWSOpts {
                    path: w.path.as_ref().map(std::string::ToString::to_string),
                    headers,
                    max_early_data: w.max_early_data,
                    early_data_header_name: w
                        .early_data_header_name
                        .as_ref()
                        .map(std::string::ToString::to_string),
                    v2ray_http_upgrade: w.v2ray_http_upgrade,
                    v2ray_http_upgrade_fast_open: w.v2ray_http_upgrade_fast_open,
                }),
                None,
                None,
                None,
                None,
            )
        }
        TransportConfig::Grpc(g) => (
            Some("grpc".to_string()),
            None,
            Some(ClashGrpcOpts {
                grpc_service_name: g
                    .service_name
                    .as_ref()
                    .or(g.path.as_ref())
                    .map(std::string::ToString::to_string),
                grpc_user_agent: g.user_agent.as_ref().map(std::string::ToString::to_string),
                ping_interval: g.ping_interval,
                max_concurrent_streams: None,
            }),
            None,
            None,
            None,
        ),
        TransportConfig::Http(h) => {
            // h2 host is emitted ONLY from the config's explicit host, or from
            // the opt-in `server` fallback when the config host is unset. The
            // `*_proto` export path passes `None`, so an unset host stays unset
            // (never the endpoint). The opts block is emitted when either host
            // or path is present, so a host-free h2 config keeps its path.
            let host = h
                .host
                .as_ref()
                .map(std::string::ToString::to_string)
                .or_else(|| server.map(str::to_string));
            let h2_opts = (h.host.is_some() || h.path.is_some()).then(|| ClashH2Opts {
                host: host.map(|host| vec![host]),
                path: h.path.as_ref().map(std::string::ToString::to_string),
            });
            (Some("http".to_string()), None, None, h2_opts, None, None)
        }
        TransportConfig::Kcp(k) => (
            Some("kcp".to_string()),
            None,
            None,
            None,
            None,
            Some(ClashKcpOpts {
                mtu: k.mtu,
                tti: k.tti,
                uplink_capacity: k.uplink_capacity,
                downlink_capacity: k.downlink_capacity,
                congestion: k.congestion,
                write_buffer: k.write_buffer,
                read_buffer: k.read_buffer,
                seed: k.seed.as_ref().map(std::string::ToString::to_string),
                header: k.header_type.as_ref().map(std::string::ToString::to_string),
            }),
        ),
        _ => (None, None, None, None, None, None),
    }
}

/// Convert a Clash ALPN (Option<Vec<String>>) to Option<&str> for `clash_tls_to_security`.
pub(crate) fn clash_alpn_as_str(alpn: Option<&Vec<String>>) -> Option<&str> {
    alpn.and_then(|v| v.first())
        .map(std::string::String::as_str)
}

/// Build xray-core `streamSettings` JSON from typed security + transport.
/// Returns `None` when there is nothing to emit (tcp + no TLS).
pub fn to_xray_stream_settings(
    security: &SecurityConfig,
    transport: &TransportConfig,
) -> Option<serde_json::Value> {
    let mut ss = serde_json::Map::new();
    // Xray-core calls the XHttp transport "splithttp" (its proto/config name);
    // `type_str()` returns "xhttp" which xray-core does not recognize as a
    // network.
    let network = if matches!(transport, TransportConfig::XHttp(_)) {
        "splithttp"
    } else {
        transport.type_str()
    };
    if network != "tcp" {
        ss.insert(
            "network".into(),
            serde_json::Value::String(network.to_string()),
        );
    }
    match &security.tls {
        Some(TlsConfig::Tls(opts)) => {
            ss.insert("security".into(), serde_json::json!("tls"));
            let mut t = serde_json::Map::new();
            if let Some(sni) = &opts.sni {
                t.insert("serverName".into(), serde_json::json!(sni.as_str()));
            }
            if let Some(insecure) = opts.insecure {
                t.insert("allowInsecure".into(), serde_json::json!(insecure));
            }
            if let Some(fp) = &opts.fp {
                t.insert("fingerprint".into(), serde_json::json!(fp.as_str()));
            }
            if let Some(alpn) = &opts.alpn {
                let list: Vec<&str> = alpn.split(',').map(str::trim).collect();
                t.insert("alpn".into(), serde_json::json!(list));
            }
            if !t.is_empty() {
                ss.insert("tlsSettings".into(), serde_json::Value::Object(t));
            }
        }
        Some(TlsConfig::Reality(opts)) => {
            ss.insert("security".into(), serde_json::json!("reality"));
            let mut r = serde_json::Map::new();
            if let Some(sni) = &opts.sni {
                r.insert("serverName".into(), serde_json::json!(sni.as_str()));
            }
            if let Some(pbk) = &opts.pbk {
                r.insert("publicKey".into(), serde_json::json!(pbk));
            }
            if let Some(sid) = &opts.sid {
                r.insert("shortId".into(), serde_json::json!(sid.as_str()));
            }
            if let Some(spx) = &opts.spx {
                r.insert("spiderX".into(), serde_json::json!(spx.as_str()));
            }
            if let Some(fp) = &opts.fp {
                r.insert("fingerprint".into(), serde_json::json!(fp.as_str()));
            }
            ss.insert("realitySettings".into(), serde_json::Value::Object(r));
        }
        None => {}
    }
    match transport {
        TransportConfig::Ws(cfg) => {
            let mut w = serde_json::Map::new();
            if let Some(p) = &cfg.path {
                w.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(h) = &cfg.host {
                // Top-level `host` — xray-core deprecates `headers.Host`
                // ("will be removed soon").
                w.insert("host".into(), serde_json::json!(h.as_str()));
            }
            if !w.is_empty() {
                ss.insert("wsSettings".into(), serde_json::Value::Object(w));
            }
        }
        TransportConfig::Grpc(cfg) => {
            let mut g = serde_json::Map::new();
            if let Some(sn) = &cfg.service_name {
                g.insert("serviceName".into(), serde_json::json!(sn.as_str()));
            }
            if !g.is_empty() {
                ss.insert("grpcSettings".into(), serde_json::Value::Object(g));
            }
        }
        TransportConfig::Http(cfg) => {
            let mut h = serde_json::Map::new();
            if let Some(p) = &cfg.path {
                h.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(host) = &cfg.host {
                h.insert("host".into(), serde_json::json!([host.as_str()]));
            }
            if !h.is_empty() {
                ss.insert("httpSettings".into(), serde_json::Value::Object(h));
            }
        }
        TransportConfig::HttpUpgrade(cfg) => {
            let mut u = serde_json::Map::new();
            if let Some(p) = &cfg.path {
                u.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(host) = &cfg.host {
                // httpupgradeSettings.host is a single string (unlike
                // httpSettings.host which is an array).
                u.insert("host".into(), serde_json::json!(host.as_str()));
            }
            if !u.is_empty() {
                ss.insert("httpupgradeSettings".into(), serde_json::Value::Object(u));
            }
        }
        TransportConfig::XHttp(cfg) => {
            let mut x = serde_json::Map::new();
            if let Some(p) = &cfg.path {
                x.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(host) = &cfg.host {
                x.insert("host".into(), serde_json::json!(host.as_str()));
            }
            if !x.is_empty() {
                ss.insert("splithttpSettings".into(), serde_json::Value::Object(x));
            }
        }
        TransportConfig::Tcp | TransportConfig::Quic | TransportConfig::Kcp(_) => {}
    }
    if ss.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(ss))
    }
}

/// Build-time override copy of `security` for the xray `inject_to` impls:
/// when `skip_cert_verify` is set, TLS `insecure` is forced on (the TUI's
/// "skip cert verify" toggle → `tlsSettings.allowInsecure`). Reality has no
/// insecure knob and is returned unchanged. Never mutates the stored config.
#[must_use]
pub(crate) fn security_force_insecure(
    security: &SecurityConfig,
    skip_cert_verify: bool,
) -> SecurityConfig {
    if !skip_cert_verify {
        return security.clone();
    }
    let mut sec = security.clone();
    if let Some(TlsConfig::Tls(opts)) = &mut sec.tls {
        opts.insecure = Some(true);
    }
    sec
}

/// Build-time reality validation, ported from the old xray builder: a
/// `security: "reality"` stream setting without a usable realitySettings
/// kills the core at startup ("REALITY: Empty \"realitySettings\"" when the
/// object is absent, `empty "password"` when publicKey is missing). Realities
/// need at least the server's public key and an SNI; both are unrecoverable
/// if absent from the profile.
///
/// # Errors
///
/// Returns [`SupportError::Config`] when the security is reality but the
/// reality settings are incomplete.
pub(crate) fn validate_xray_reality(security: &SecurityConfig) -> Result<(), SupportError> {
    if let Some(TlsConfig::Reality(opts)) = &security.tls {
        let has = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
        if !has(opts.pbk.as_deref()) || !has(opts.sni.as_deref()) {
            return Err(SupportError::Config(
                "REALITY profile is missing required stream settings \
                 (realitySettings.publicKey / serverName); \
                 security is 'reality' but the reality settings are incomplete"
                    .to_string(),
            ));
        }
    }
    Ok(())
}

/// Build the sing-box `tls` block shared by every TLS-capable outbound,
/// ported field-by-field from the old sing-box builder's `build_tls` onto the
/// typed configs. Returns `None` when the config carries no TLS at all (e.g.
/// `security=none` vless/vmess/trojan, empty shadowtls) — the caller then
/// emits no `tls` key, exactly like the old builder.
///
/// Shape: `{ "enabled": true, "server_name", "insecure"?, "alpn"?,
/// "utls"?, "reality"? }` (sing-box `OutboundTLSOptions`).
///
/// - `server_name` = explicit config sni, falling back to the endpoint host
///   at BUILD time — never stored in the config (host-free parse mandate).
/// - `skip_cert_verify` forces `insecure: true` (old builder:
///   `params.skip_cert_verify || p_settings.insecure`).
/// - `alpn` is the comma-separated list string → array.
/// - `utls` from the config fingerprint.
/// - reality emits `public_key`/`short_id` only when present; the block is
///   dropped when it would carry nothing beyond `enabled`. `spider_x` is
///   deliberately NOT emitted — sing-box's `OutboundRealityOptions` has no
///   such field (xray-only; it must never leak into `short_id`).
#[must_use]
pub(crate) fn to_singbox_tls(
    security: &SecurityConfig,
    endpoint: &EndpointEssentials,
    skip_cert_verify: bool,
) -> Option<serde_json::Value> {
    let tls = security.tls.as_ref()?;
    let mut tls_json = serde_json::Map::new();
    tls_json.insert("enabled".into(), serde_json::json!(true));

    let sni = security.sni().filter(|s| !s.is_empty());
    tls_json.insert(
        "server_name".into(),
        serde_json::json!(sni.unwrap_or(endpoint.host.as_str())),
    );

    let insecure = skip_cert_verify || security.insecure() == Some(true);
    if insecure {
        tls_json.insert("insecure".into(), serde_json::json!(true));
    }

    if let Some(alpn) = security.alpn().filter(|s| !s.is_empty()) {
        let parts: Vec<&str> = alpn
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if !parts.is_empty() {
            tls_json.insert("alpn".into(), serde_json::json!(parts));
        }
    }

    if let Some(fp) = security.fp().filter(|s| !s.is_empty()) {
        tls_json.insert(
            "utls".into(),
            serde_json::json!({ "enabled": true, "fingerprint": fp }),
        );
    }

    if let TlsConfig::Reality(reality) = tls {
        let mut r = serde_json::Map::new();
        r.insert("enabled".into(), serde_json::json!(true));
        if let Some(pbk) = reality.pbk.as_deref().filter(|s| !s.is_empty()) {
            r.insert("public_key".into(), serde_json::json!(pbk));
        }
        if let Some(sid) = reality.sid.as_deref().filter(|s| !s.is_empty()) {
            r.insert("short_id".into(), serde_json::json!(sid));
        }
        // Only add the reality block if it has meaningful content beyond
        // "enabled" (old builder: `if reality.len() > 1`).
        if r.len() > 1 {
            tls_json.insert("reality".into(), serde_json::json!(r));
        }
    }

    Some(serde_json::Value::Object(tls_json))
}

/// Like [`to_singbox_tls`] but always returns a block: for protocols where
/// TLS is mandatory (tuic/hysteria1/hysteria2/naive/anytls) the old builder
/// unconditionally emitted `tls.enabled`, so an empty typed security still
/// yields `{ enabled: true, server_name: <sni or endpoint host> }` (plus any
/// insecure/alpn/utls the config does carry).
#[must_use]
pub(crate) fn to_singbox_tls_or_default(
    security: &SecurityConfig,
    endpoint: &EndpointEssentials,
    skip_cert_verify: bool,
) -> serde_json::Value {
    to_singbox_tls(security, endpoint, skip_cert_verify).unwrap_or_else(|| {
        serde_json::json!({
            "enabled": true,
            "server_name": security
                .sni()
                .filter(|s| !s.is_empty())
                .unwrap_or(endpoint.host.as_str()),
        })
    })
}

/// Build the sing-box `transport` block (`V2RayTransportOptions`) for the
/// vless/vmess/trojan outbounds.
///
/// Shape per `thirdparty/sing-box/option/v2ray_transport.go` +
/// `docs/configuration/shared/v2ray-transport.md`:
///
/// | Typed transport | sing-box emission |
/// |-----------------|--------------------|
/// | `Tcp`           | `None` — tcp is the sing-box default, no `transport` key |
/// | `Ws`            | `{ type: "ws", path, headers: { Host, … }, max_early_data, early_data_header_name }` |
/// | `Grpc`          | `{ type: "grpc", service_name }` (share-link convention: the typed `path` doubles as the service name) |
/// | `Http`          | `{ type: "http", host: [...], path, method, headers }` |
/// | `HttpUpgrade`   | `{ type: "httpupgrade", host, path, headers }` |
/// | `Quic`          | `{ type: "quic" }` |
/// | `Kcp`/`XHttp`   | build-time refusal — not in the vendored sing-box transport set |
///
/// Host-bearing fields come from the (host-injected) transport; callers apply
/// `TransportConfig::with_host(endpoint.host, …)` first so unset host fields
/// filled from the endpoint at build time (host-free parse mandate).
///
/// # Errors
///
/// Returns [`SupportError::Config`] when the transport cannot be mapped to
/// the vendored sing-box transport set (`kcp`/`xhttp`).
pub(crate) fn to_singbox_transport(
    transport: &TransportConfig,
) -> Result<Option<serde_json::Value>, SupportError> {
    let mut obj = serde_json::Map::new();
    match transport {
        TransportConfig::Tcp => return Ok(None),
        TransportConfig::Ws(cfg) => {
            obj.insert("type".into(), serde_json::json!("ws"));
            if let Some(p) = &cfg.path {
                obj.insert("path".into(), serde_json::json!(p.as_str()));
            }
            // sing-box has no top-level ws host field — the vhost goes into
            // the `headers` map (Host header), merged over any extra headers.
            let mut headers = cfg.headers.clone().unwrap_or_default();
            if let Some(h) = &cfg.host {
                headers.insert("Host".to_string(), h.to_string());
            }
            if !headers.is_empty() {
                obj.insert("headers".into(), serde_json::json!(headers));
            }
            if let Some(med) = cfg.max_early_data {
                obj.insert("max_early_data".into(), serde_json::json!(med));
            }
            if let Some(name) = &cfg.early_data_header_name {
                obj.insert(
                    "early_data_header_name".into(),
                    serde_json::json!(name.as_str()),
                );
            }
        }
        TransportConfig::Grpc(cfg) => {
            obj.insert("type".into(), serde_json::json!("grpc"));
            // `service_name` for forms-built configs; URL share links carry
            // the service name in `path` (share-link convention).
            if let Some(sn) = cfg.service_name.as_ref().or(cfg.path.as_ref()) {
                obj.insert("service_name".into(), serde_json::json!(sn.as_str()));
            }
        }
        TransportConfig::Http(cfg) => {
            obj.insert("type".into(), serde_json::json!("http"));
            if let Some(host) = &cfg.host {
                obj.insert("host".into(), serde_json::json!([host.as_str()]));
            }
            if let Some(p) = &cfg.path {
                obj.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(m) = &cfg.method {
                obj.insert("method".into(), serde_json::json!(m.as_str()));
            }
            if let Some(headers) = &cfg.headers
                && !headers.is_empty()
            {
                obj.insert("headers".into(), serde_json::json!(headers));
            }
        }
        TransportConfig::HttpUpgrade(cfg) => {
            obj.insert("type".into(), serde_json::json!("httpupgrade"));
            if let Some(host) = &cfg.host {
                obj.insert("host".into(), serde_json::json!(host.as_str()));
            }
            if let Some(p) = &cfg.path {
                obj.insert("path".into(), serde_json::json!(p.as_str()));
            }
            if let Some(headers) = &cfg.headers
                && !headers.is_empty()
            {
                obj.insert("headers".into(), serde_json::json!(headers));
            }
        }
        TransportConfig::Quic => {
            obj.insert("type".into(), serde_json::json!("quic"));
        }
        TransportConfig::Kcp(_) => {
            return Err(SupportError::Config(
                "kcp transport is not supported by sing-box (no mKCP in the vendored transport set)"
                    .into(),
            ));
        }
        TransportConfig::XHttp(_) => {
            return Err(SupportError::Config(
                "xhttp/splithttp transport is not supported by sing-box (no XHTTP in the vendored transport set)"
                    .into(),
            ));
        }
    }
    Ok(Some(serde_json::Value::Object(obj)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto_spec::{ProtoSpec, ProtocolConfig, VlessConfig};
    use crate::urlx::RawUrlX;

    #[test]
    fn vless_stream_settings_emits_xray_stream_settings() {
        let raw = RawUrlX::from(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@cdn.example.com:443?security=tls&type=ws&path=%2Fws&host=cdn.example.com#r",
        );
        let config = VlessConfig::try_parse(&raw).expect("parse vless URL");
        let ss = to_xray_stream_settings(&config.security, &config.transport)
            .expect("streamSettings present");
        assert_eq!(ss["network"], "ws");
        assert_eq!(ss["security"], "tls");
        assert_eq!(ss["wsSettings"]["path"], "/ws");
        // The WS vhost goes in the top-level `host` field — xray-core
        // deprecates `headers.Host` ("will be removed soon").
        assert_eq!(ss["wsSettings"]["host"], "cdn.example.com");
        assert!(
            ss["wsSettings"].get("headers").is_none(),
            "headers.Host must not be emitted"
        );
    }

    #[test]
    fn vless_stream_settings_emits_splithttp_network() {
        let raw = RawUrlX::from(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@cdn.example.com:443?security=tls&type=splithttp&path=%2Fs#r",
        );
        let config = VlessConfig::try_parse(&raw).expect("parse vless URL");
        let ss = to_xray_stream_settings(&config.security, &config.transport)
            .expect("streamSettings present");
        // xray-core only recognizes "splithttp" as the network name.
        assert_eq!(ss["network"], "splithttp");
        assert_eq!(ss["splithttpSettings"]["path"], "/s");
    }

    #[test]
    fn vless_stream_settings_emits_httpupgrade_host_string() {
        let raw = RawUrlX::from(
            "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@cdn.example.com:443?type=httpupgrade&path=%2Fup&host=cdn.example.com#r",
        );
        let config = VlessConfig::try_parse(&raw).expect("parse vless URL");
        let ss = to_xray_stream_settings(&config.security, &config.transport)
            .expect("streamSettings present");
        assert_eq!(ss["network"], "httpupgrade");
        let host = &ss["httpupgradeSettings"]["host"];
        assert!(
            host.is_string(),
            "httpupgradeSettings.host must be a string, got {host}"
        );
        assert_eq!(host, "cdn.example.com");
    }

    #[test]
    fn security_config_default_is_empty() {
        let sc = SecurityConfig::default();
        assert!(sc.tls.is_none());
        assert!(sc.enc.is_none());
    }

    #[test]
    fn security_config_type_str() {
        let tls = SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts::default())),
            enc: None,
        };
        assert_eq!(tls.type_str(), Some("tls"));

        let reality = SecurityConfig {
            tls: Some(TlsConfig::Reality(RealityOpts::default())),
            enc: None,
        };
        assert_eq!(reality.type_str(), Some("reality"));

        let none = SecurityConfig::default();
        assert_eq!(none.type_str(), None);
    }

    #[test]
    fn security_config_serde_empty() {
        let sc = SecurityConfig::default();
        let json = serde_json::to_string(&sc).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn security_config_serde_tls() {
        let sc = SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                sni: Some("example.com".into()),
                ..TlsOpts::default()
            })),
            enc: None,
        };
        let json = serde_json::to_string(&sc).unwrap();
        assert!(json.contains("\"tls\""));
        assert!(json.contains("\"sni\""));
        assert!(json.contains("\"example.com\""));
        assert!(!json.contains("\"enc\""));
    }

    #[test]
    fn should_skip_endpoint_param_skips_matching_dns() {
        assert!(should_skip_endpoint_param("example.com", "example.com"));
        assert!(should_skip_endpoint_param("example.com", "EXAMPLE.COM"));
        assert!(!should_skip_endpoint_param("example.com", "other.com"));
        assert!(!should_skip_endpoint_param("example.com", "example.org"));
    }

    #[test]
    fn should_skip_endpoint_param_never_skips_for_ip() {
        assert!(!should_skip_endpoint_param("1.2.3.4", "anything"));
        assert!(!should_skip_endpoint_param("1.2.3.4", "1.2.3.4"));

        assert!(!should_skip_endpoint_param("::1", "::1"));
        assert!(!should_skip_endpoint_param("::1", "anything"));
    }

    #[test]
    fn clash_ws_lowercase_host_header_restores_vhost() {
        let ws = ClashWSOpts {
            path: Some("/ws".into()),
            headers: Some(std::collections::HashMap::from([
                ("host".to_string(), "cdn.example.com".to_string()),
                ("X-Test".to_string(), "1".to_string()),
            ])),
            ..ClashWSOpts::default()
        };
        let t = clash_transport_to_transport(Some("ws"), Some(&ws), None, None, None, None, None);
        match t {
            TransportConfig::Ws(w) => {
                assert_eq!(w.host.as_deref(), Some("cdn.example.com"));
                let headers = w.headers.expect("non-host headers survive");
                assert!(
                    !headers.keys().any(|k| k.eq_ignore_ascii_case("host")),
                    "host must not be double-represented: {headers:?}"
                );
                assert_eq!(headers.get("X-Test").map(String::as_str), Some("1"));
            }
            other => panic!("expected Ws transport, got {other:?}"),
        }
    }

    #[test]
    fn with_host_forwards_host_to_ws_grpc_http() {
        let cases = vec![
            TransportConfig::Ws(WebSocketConfig::default()),
            TransportConfig::Grpc(GrpcConfig::default()),
            TransportConfig::Http(HttpConfig::default()),
        ];
        for t in cases {
            let t = t.with_host(Some("cdn.example.com".into()), None, None);
            let host = match &t {
                TransportConfig::Ws(c) => c.host.as_deref(),
                TransportConfig::Grpc(c) => c.authority.as_deref(),
                TransportConfig::Http(c) => c.host.as_deref(),
                _ => None,
            };
            assert_eq!(host, Some("cdn.example.com"), "{t:?} keeps host");
        }
    }
}
