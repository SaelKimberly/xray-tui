use std::net::SocketAddr;

use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
use xray_tui_proto::proto_spec::{ProtoSpec, ProtocolConfig, SecurityConfig, TlsConfig, TlsOpts};

use crate::addr::Host;
use crate::error::{NativeError, timeouts};
use crate::security::reality::HelloProvisionerChoice;
use crate::security::tls_provider::TlsProvider;

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
    /// TLS provider for the security phase: standard rustls or a custom
    /// fingerprint-capable engine. Defaults to [`TlsProvider::Standard`]
    /// (rustls); `wrap()` still routes to the fingerprint engine when the
    /// config carries an `fp` value.
    pub tls_provider: TlsProvider,
    /// REALITY provisioner for the security phase: which fingerprint shapes
    /// the `ClientHello`. Defaults to
    /// [`HelloProvisionerChoice::FixedChrome133`].
    pub reality_provisioner: HelloProvisionerChoice,
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
            tls_provider: TlsProvider::Standard,
            reality_provisioner: HelloProvisionerChoice::FixedChrome133,
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
        self.tls_opts()
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
            .unwrap_or_default()
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

    /// The TLS provider selected for this connect (default: `Standard`).
    #[must_use]
    pub const fn tls_provider(&self) -> &TlsProvider {
        &self.params.tls_provider
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
