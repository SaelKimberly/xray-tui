use std::net::SocketAddr;

use xray_tui_proto::proto_spec::common::{
    GrpcConfig, HttpConfig, HttpUpgradeConfig, KcpConfig, TransportConfig, WebSocketConfig,
    XHttpConfig,
};
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
use xray_tui_proto::proto_spec::{
    ProtoSpec, ProtocolConfig, SecurityConfig, TlsConfig, TlsOpts, parse_curve_names,
};

use crate::addr::Host;
use crate::error::{NativeError, timeouts};
use crate::protocol::vless::PacketMode;
use crate::security::reality::HelloProvisionerChoice;

/// Per-connect parameters: the typed proto config plus the dial address.
///
/// `server` is the proxy server (host/port); `target` is the destination the
/// proxy should reach. `resolved_ip` lets callers inject a pre-resolved
/// address (the enrichment pipeline's `resolved_as`) and skip DNS.
#[derive(Debug, Clone)]
pub struct NativeConnectParams {
    pub protocol: ProtocolConfig,
    pub server: EndpointEssentials,
    pub target: crate::addr::TargetAddr,
    pub resolved_ip: Option<SocketAddr>,
    /// REALITY provisioner for the security phase: which fingerprint shapes
    /// the `ClientHello`. Defaults to
    /// [`HelloProvisionerChoice::FixedChrome133`].
    pub reality_provisioner: HelloProvisionerChoice,
    /// UDP packet mode: `None` = the TCP path (default; [`crate::connect`]);
    /// `Some(mode)` selects the VLESS UDP datagram tunnel ([`crate::connect_udp`],
    /// command 0x02) with `Raw` (header-dest) or `PacketAddr` (per-packet
    /// magic-address destinations) framing.
    pub udp: Option<PacketMode>,
    /// Mux tunnel for UDP: `true` routes [`crate::connect_udp`] through
    /// the VLESS mux tunnel (XUDP — `connect_mux` → `open_udp_session` →
    /// [`PacketMode::XUdp`]) instead of the raw `command=0x02` tunnel.
    /// The `xtls-rprx-vision-udp443` flow forces the mux path regardless
    /// (spec §4.3). Ignored by the TCP path ([`crate::connect`]).
    pub mux: bool,
}

impl NativeConnectParams {
    #[must_use]
    pub const fn new(
        protocol: ProtocolConfig,
        server: EndpointEssentials,
        target: crate::addr::TargetAddr,
    ) -> Self {
        Self {
            protocol,
            server,
            target,
            resolved_ip: None,
            reality_provisioner: HelloProvisionerChoice::FixedChrome133,
            udp: None,
            mux: false,
        }
    }
}

/// Per-link state in the connect pipeline (a chain has one context per link).
#[derive(Debug, Clone)]
pub struct LinkContext {
    pub params: NativeConnectParams,
    pub target: crate::addr::TargetAddr,
}

impl LinkContext {
    #[must_use]
    pub const fn new(params: NativeConnectParams, target: crate::addr::TargetAddr) -> Self {
        Self { params, target }
    }

    /// The proxy server's socket address: `resolved_ip` override, else the
    /// parsed host IP, else DNS resolution via `tokio::net::lookup_host`.
    ///
    /// DNS lookup is a network step: bounded by [`timeouts::DIAL`].
    pub async fn server_socket(&self) -> Result<SocketAddr, NativeError> {
        if let Some(addr) = self.params.resolved_ip {
            return Ok(addr);
        }
        match Host::new(&self.params.server.host) {
            Host::Ip(ip) => Ok(SocketAddr::new(ip, self.params.server.port)),
            Host::Domain(domain) => {
                let (host, port) = (domain.as_str(), self.params.server.port);
                let timeout = timeouts::DIAL;
                let mut addrs =
                    tokio::time::timeout(timeout, tokio::net::lookup_host((host, port)))
                        .await
                        .map_err(|_| NativeError::Timeout {
                            step: "dns lookup",
                            limit: timeout,
                        })?
                        .map_err(|e| NativeError::Dial(format!("{host}: {e}")))?;
                addrs
                    .next()
                    .ok_or_else(|| NativeError::Dial(format!("{host}: no addresses found")))
            }
        }
    }

    /// TLS SNI: explicit `sni` option, else the endpoint host.
    #[must_use]
    pub fn sni(&self) -> String {
        self.tls_opts()
            .ok()
            .flatten()
            .and_then(|o| o.sni.as_ref())
            .map_or_else(
                || self.params.server.host.clone(),
                std::string::ToString::to_string,
            )
    }

    /// TLS ALPN list: comma-separated `alpn` option, else empty (no ALPN).
    #[must_use]
    pub fn alpn_vec(&self) -> Vec<Vec<u8>> {
        let explicit: Vec<Vec<u8>> = self
            .tls_opts()
            .ok()
            .flatten()
            .and_then(|o| o.alpn.as_ref())
            .map(|a| {
                a.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::as_bytes)
                    .map(<[u8]>::to_vec)
                    .collect()
            })
            .unwrap_or_default();
        if !explicit.is_empty() {
            return explicit;
        }
        // Transport-implied ALPN: the grpc, xhttp and v2rayhttp (`http`)
        // transports are HTTP/2 (xhttp falls back to HTTP/1.1 only when
        // there is no TLS at all — no ALPN then), the ws and httpupgrade
        // upgrades are HTTP/1.1 exchanges. (Reality forces h2+http/1.1
        // server-side; an explicit `alpn` option wins above.)
        match self.transport_type() {
            Some("grpc" | "xhttp" | "http") => vec![b"h2".to_vec()],
            Some("ws" | "httpupgrade") => vec![b"http/1.1".to_vec()],
            _ => vec![],
        }
    }

    /// TLS curve preferences: the `curves` option parsed to wire curve IDs
    /// via the proto's `parse_curve_names` (xray `ParseCurveName` mirror —
    /// unknown names skipped). Empty when unset or security is REALITY.
    #[must_use]
    pub fn curve_ids(&self) -> Vec<u16> {
        self.tls_opts()
            .ok()
            .flatten()
            .and_then(|o| o.curves.as_ref())
            .map_or_else(Vec::new, |c| parse_curve_names(c.as_str()))
    }

    /// The typed TLS options when security is plain TLS (not Reality).
    pub fn tls_opts(&self) -> Result<Option<&TlsOpts>, NativeError> {
        match self.security() {
            Some(SecurityConfig {
                tls: Some(TlsConfig::Tls(opts)),
                ..
            }) => Ok(Some(opts)),
            Some(SecurityConfig {
                tls: Some(TlsConfig::Reality(_)),
                ..
            }) => Err(NativeError::Reality(
                "plain-TLS opts requested for a REALITY config".into(),
            )),
            _ => Ok(None),
        }
    }

    /// Security config from the protocol payload (typed, via the proto trait).
    #[must_use]
    pub fn security(&self) -> Option<&SecurityConfig> {
        self.params.protocol.security()
    }

    /// Transport name ("tcp", "ws", ...) from the protocol payload.
    #[must_use]
    pub fn transport_type(&self) -> Option<&str> {
        self.params.protocol.transport_type()
    }

    /// Typed transport config (vless/vmess/trojan carry `transport`).
    #[must_use]
    pub const fn transport_config(&self) -> Option<&TransportConfig> {
        use xray_tui_proto::proto_spec::ProtocolConfig as PC;
        match &self.params.protocol {
            PC::Vless(c) => Some(&c.transport),
            PC::Vmess(c) => Some(&c.transport),
            PC::Trojan(c) => Some(&c.transport),
            _ => None,
        }
    }

    /// WebSocket transport config, when the link uses `ws`.
    #[must_use]
    pub fn transport_ws(&self) -> Option<&WebSocketConfig> {
        match self.transport_config()? {
            TransportConfig::Ws(c) => Some(c),
            _ => None,
        }
    }

    /// gRPC transport config, when the link uses `grpc`.
    #[must_use]
    pub fn transport_grpc(&self) -> Option<&GrpcConfig> {
        match self.transport_config()? {
            TransportConfig::Grpc(c) => Some(c),
            _ => None,
        }
    }

    /// `HTTPUpgrade` transport config, when the link uses `httpupgrade`.
    #[must_use]
    pub fn transport_httpupgrade(&self) -> Option<&HttpUpgradeConfig> {
        match self.transport_config()? {
            TransportConfig::HttpUpgrade(c) => Some(c),
            _ => None,
        }
    }

    /// XHTTP transport config, when the link uses `xhttp`.
    #[must_use]
    pub fn transport_xhttp(&self) -> Option<&XHttpConfig> {
        match self.transport_config()? {
            TransportConfig::XHttp(c) => Some(c),
            _ => None,
        }
    }

    /// h2 (v2rayhttp) transport config, when the link uses `http`.
    #[must_use]
    pub fn transport_http(&self) -> Option<&HttpConfig> {
        match self.transport_config()? {
            TransportConfig::Http(c) => Some(c),
            _ => None,
        }
    }

    /// mKCP transport config, when the link uses `kcp` (the native params
    /// expose only mtu/tti; everything else derives, spec §4.5).
    #[must_use]
    pub fn transport_kcp(&self) -> Option<&KcpConfig> {
        match self.transport_config()? {
            TransportConfig::Kcp(c) => Some(c),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{Host, TargetAddr};

    fn vless_empty() -> ProtocolConfig {
        // VlessConfig has no Default; build through the serde schema.
        // ProtocolConfig is `#[serde(tag = "schema")]` with no rename: the
        // tag value is the PascalCase variant name ("Vless").
        serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        }))
        .expect("vless config parses")
    }

    fn target(host: &str) -> TargetAddr {
        TargetAddr::new(Host::Domain(host.into()), 80)
    }

    #[test]
    fn server_socket_uses_ip_host_without_dns() {
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("127.0.0.1", 4430),
            target("example.com"),
        );
        let ctx = LinkContext::new(params, target("example.com"));
        let sock = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ctx.server_socket())
            .unwrap();
        assert_eq!(sock, SocketAddr::from(([127, 0, 0, 1], 4430)));
    }

    #[test]
    fn resolved_ip_overrides_host() {
        let mut params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("server.example", 4430),
            target("example.com"),
        );
        params.resolved_ip = Some(SocketAddr::from(([127, 0, 0, 1], 9999)));
        let ctx = LinkContext::new(params, target("example.com"));
        let sock = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ctx.server_socket())
            .unwrap();
        assert_eq!(sock, SocketAddr::from(([127, 0, 0, 1], 9999)));
    }

    #[test]
    fn sni_falls_back_to_endpoint_host() {
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("my.server.test", 4430),
            target("example.com"),
        );
        let ctx = LinkContext::new(params, target("example.com"));
        assert_eq!(ctx.sni(), "my.server.test");
    }

    #[test]
    fn alpn_splits_on_comma() {
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("127.0.0.1", 4430),
            target("example.com"),
        );
        let ctx = LinkContext::new(params, target("example.com"));
        assert!(ctx.alpn_vec().is_empty());
    }

    #[test]
    fn curve_ids_parse_curves_option() {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": {
                "type": "tls", "sni": "sni.example",
                "curves": "X25519MLKEM768, x25519, bogus"
            },
            "transport": { "type": "tcp" }
        }))
        .expect("vless tls+curves config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                protocol,
                EndpointEssentials::new("127.0.0.1", 4430),
                target("example.com"),
            ),
            target("example.com"),
        );
        // Case-insensitive; unknown names skipped (ParseCurveName mirror).
        assert_eq!(ctx.curve_ids(), vec![4588, 29]);
    }

    #[test]
    fn curve_ids_empty_without_curves_option() {
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                vless_empty(),
                EndpointEssentials::new("127.0.0.1", 4430),
                target("example.com"),
            ),
            target("example.com"),
        );
        assert!(ctx.curve_ids().is_empty());
    }

    /// The full plumbing: config string → proto parse → native params →
    /// the TLS spec's `supported_groups`/`key_share` extensions.
    #[test]
    fn curves_config_flows_to_tls_spec_extensions() {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": {
                "type": "tls", "sni": "sni.example",
                "curves": "x25519mlkem768,x25519"
            },
            "transport": { "type": "tcp" }
        }))
        .expect("vless tls+curves config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                protocol,
                EndpointEssentials::new("127.0.0.1", 4430),
                target("example.com"),
            ),
            target("example.com"),
        );
        let ids = ctx.curve_ids();
        assert_eq!(ids, vec![4588, 29]);

        let spec = xray_tui_tls::spec::apply_curve_preferences(
            &xray_tui_tls::profiles::chrome133::spec(),
            &ids,
        );
        let groups = spec.extensions.iter().find_map(|e| match e {
            xray_tui_tls::spec::ExtensionSpec::SupportedGroups(g) => Some(g.clone()),
            _ => None,
        });
        assert_eq!(groups, Some(vec![4588, 29]));
        let key_shares = spec.extensions.iter().find_map(|e| match e {
            xray_tui_tls::spec::ExtensionSpec::KeyShare(g) => Some(g.clone()),
            _ => None,
        });
        assert!(matches!(
            key_shares.as_deref(),
            Some([
                xray_tui_tls::spec::KeyShareGroup::Grease,
                xray_tui_tls::spec::KeyShareGroup::X25519Mlkem768,
                xray_tui_tls::spec::KeyShareGroup::X25519
            ])
        ));
    }

    #[test]
    fn alpn_defaults_per_transport() {
        let grpc: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "grpc", "service_name": "gun" }
        }))
        .expect("vless grpc config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                grpc,
                EndpointEssentials::new("127.0.0.1", 4430),
                target("x"),
            ),
            target("x"),
        );
        assert_eq!(ctx.alpn_vec(), vec![b"h2".to_vec()]);

        let ws: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "ws", "path": "/ws" }
        }))
        .expect("vless ws config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(ws, EndpointEssentials::new("127.0.0.1", 4430), target("x")),
            target("x"),
        );
        assert_eq!(ctx.alpn_vec(), vec![b"http/1.1".to_vec()]);

        // v2rayhttp rides HTTP/2 like grpc.
        let http: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "http", "path": "/h2" }
        }))
        .expect("vless http config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                http,
                EndpointEssentials::new("127.0.0.1", 4430),
                target("x"),
            ),
            target("x"),
        );
        assert_eq!(ctx.alpn_vec(), vec![b"h2".to_vec()]);
    }

    #[test]
    fn explicit_alpn_wins_over_transport() {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": { "type": "tls", "sni": "sni.example", "alpn": "h2,http/1.1" },
            "transport": { "type": "grpc", "service_name": "gun" }
        }))
        .expect("vless grpc+tls config parses");
        let ctx = LinkContext::new(
            NativeConnectParams::new(
                protocol,
                EndpointEssentials::new("127.0.0.1", 4430),
                target("x"),
            ),
            target("x"),
        );
        assert_eq!(ctx.alpn_vec(), vec![b"h2".to_vec(), b"http/1.1".to_vec()]);
    }

    #[test]
    fn tls_config_yields_sni_and_alpn() {
        // `SecurityConfig.tls` is `#[serde(flatten)]` — the TlsConfig internal
        // tag sits directly under `security`.
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": { "type": "tls", "sni": "sni.example", "alpn": "h2,http/1.1" },
            "transport": { "type": "tcp" }
        }))
        .expect("vless tls config parses");
        let params = NativeConnectParams::new(
            protocol,
            EndpointEssentials::new("127.0.0.1", 4430),
            target("example.com"),
        );
        let ctx = LinkContext::new(params, target("example.com"));
        assert_eq!(ctx.sni(), "sni.example");
        assert_eq!(ctx.alpn_vec(), vec![b"h2".to_vec(), b"http/1.1".to_vec()]);
    }
}
