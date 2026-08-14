//! Data-driven e2e case: protocol + optional payload-security variant.
//!
//! The `E2eCase` impl derives the server config, client params, expectations
//! and label from these parts — adding a variant is a new `SecurityVariant`
//! impl, not a new case struct.

use std::net::SocketAddr;

use super::{
    Certs, ConnectExpect, CoreKind, E2eCase, E2eExpect, SecurityVariant, ServerEnv, StandardTls,
    TlsVariant, config,
};
use crate::NativeConnectParams;
use crate::protocol::vless::PacketMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolKind {
    Vless,
    Vmess,
}

/// VLESS flow control; only `xtls-rprx-vision` is implemented (None = none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Vision,
}

impl Flow {
    /// The wire flow name both cores and the native client emit.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        "xtls-rprx-vision"
    }
}

/// The app-side probe kind: how the client's application data is carried
/// through the tunnel (spec §7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    /// Plain HTTP over the raw tunnel (the End path for non-TLS traffic).
    Plain,
    /// A real TLS 1.3 session THROUGH the tunnel: the app wraps the tunnel
    /// in the engine TLS client to a rustls echo target (the vision
    /// Direct-splice path).
    InnerTls,
    /// UDP datagrams through the tunnel (VLESS command 0x02): the probe
    /// sends datagrams to the UDP echo target and matches echoes by
    /// payload. The packet mode (`Raw` / `PacketAddr`) lives in
    /// [`CaseSpec::udp`].
    Udp,
}

/// One e2e scenario described as data.
///
/// Construct via [`CaseSpec::vless`] / [`CaseSpec::vmess`]; `protocol` and
/// `security` are private invariants (a vmess case always carries a variant).
/// `tls` defaults to [`StandardTls`]; add fingerprint/REALITY variants with
/// [`CaseSpec::with_tls`].
pub struct CaseSpec {
    protocol: ProtocolKind,
    security: Option<Box<dyn SecurityVariant>>,
    tls: Option<Box<dyn TlsVariant>>,
    network: &'static str,
    /// Client-side xhttp dialect ("packet-up" default, "stream-up"). The
    /// client's mode selects the dialect on the wire; `None` for non-xhttp
    /// networks.
    xhttp_mode: Option<&'static str>,
    /// VLESS flow control; `None` = no flow. Only `Flow::Vision`
    /// (`xtls-rprx-vision`) exists; it is emitted in the server's
    /// clients/users and the client outbound (vmess cases never carry one).
    flow: Option<Flow>,
    /// App-side probe kind: plain HTTP over the tunnel, an inner TLS
    /// session THROUGH the tunnel to a rustls echo target, or UDP
    /// datagrams to the UDP echo target.
    app: AppKind,
    /// UDP packet mode for UDP rows: `None` = the TCP path (default);
    /// `Some(mode)` selects the VLESS UDP datagram tunnel ([`PacketMode`]
    /// `Raw` = header-dest, `PacketAddr` = per-packet address header
    /// (`atyp|addr|port`) — the magic fqdn is header-dest only, never in
    /// the frame). Ignored unless `app` is [`AppKind::Udp`]; drives
    /// `NativeConnectParams.udp` for the probe's `connect_udp`.
    udp: Option<PacketMode>,
}

impl CaseSpec {
    #[must_use]
    pub fn vless() -> Self {
        Self {
            protocol: ProtocolKind::Vless,
            security: None,
            tls: None,
            network: "tcp",
            xhttp_mode: None,
            flow: None,
            app: AppKind::Plain,
            udp: None,
        }
    }

    #[must_use]
    pub fn vmess(variant: impl SecurityVariant + 'static) -> Self {
        Self {
            protocol: ProtocolKind::Vmess,
            security: Some(Box::new(variant)),
            tls: None,
            network: "tcp",
            xhttp_mode: None,
            flow: None,
            app: AppKind::Plain,
            udp: None,
        }
    }

    /// Select the transport ("tcp", "ws", "grpc").
    #[must_use]
    pub const fn with_network(mut self, network: &'static str) -> Self {
        self.network = network;
        self
    }

    /// Select the client-side xhttp dialect ("stream-up"; packet-up is the
    /// default). Ignored for non-xhttp networks.
    #[must_use]
    pub const fn with_xhttp_mode(mut self, mode: &'static str) -> Self {
        self.xhttp_mode = Some(mode);
        self
    }

    /// Select the VLESS flow control ("vision"; the default is no flow).
    #[must_use]
    pub const fn with_flow(mut self, flow: Flow) -> Self {
        self.flow = Some(flow);
        self
    }

    /// Select the app-side probe kind: plain HTTP over the tunnel (default),
    /// an inner TLS session through the tunnel to a rustls echo target, or
    /// UDP datagrams to the UDP echo target.
    #[must_use]
    pub const fn with_app(mut self, app: AppKind) -> Self {
        self.app = app;
        self
    }

    /// Select the UDP packet mode (`Raw` / `PacketAddr`); the row's `app`
    /// must be [`AppKind::Udp`] (the mode drives the probe's `connect_udp`).
    #[must_use]
    pub const fn with_udp(mut self, mode: PacketMode) -> Self {
        self.udp = Some(mode);
        self
    }

    /// The app-side probe kind.
    #[must_use]
    pub const fn app(&self) -> AppKind {
        self.app
    }

    /// Select the TLS transport variant (fingerprint engine or REALITY).
    #[must_use]
    pub fn with_tls(mut self, tls: Box<dyn TlsVariant>) -> Self {
        self.tls = Some(tls);
        self
    }

    /// The TLS variant, defaulting to stock rustls.
    fn tls(&self) -> &dyn TlsVariant {
        self.tls.as_deref().unwrap_or(&StandardTls)
    }

    /// Cores this case runs against (variant gate; no security → both).
    #[must_use]
    pub fn cores(&self) -> &'static [CoreKind] {
        self.security
            .as_ref()
            .map_or(&[CoreKind::Xray, CoreKind::SingBox], |s| s.cores())
    }
}

impl E2eCase for CaseSpec {
    fn label(&self) -> String {
        let proto = match self.protocol {
            ProtocolKind::Vless => "vless",
            ProtocolKind::Vmess => "vmess",
        };
        let flow = self
            .flow
            .map_or_else(String::new, |flow| format!("{}/", flow.as_str()));
        let tls = self.tls.as_deref().map_or("tls", TlsVariant::name);
        let sec = self
            .security
            .as_ref()
            .map_or(String::new(), |s| format!("/{}", s.name()));
        let app = match self.app {
            AppKind::Plain => String::new(),
            AppKind::InnerTls => "/inner-tls".to_string(),
            AppKind::Udp => "/udp".to_string(),
        };
        format!("{proto}/{flow}{}/{tls}{sec}{app}", self.network)
    }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        match self.protocol {
            ProtocolKind::Vless => {
                config::vless_inbound(core, env, self.flow, self.tls(), self.network)
            }
            ProtocolKind::Vmess => {
                let security = self.security.as_ref().and_then(|s| s.server_security(core));
                config::vmess_inbound(core, env, security, self.tls(), self.network)
            }
        }
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        let mut params = match self.protocol {
            ProtocolKind::Vless => config::client_params_vless(
                port,
                target,
                self.flow,
                self.tls(),
                self.network,
                self.xhttp_mode,
            ),
            ProtocolKind::Vmess => {
                let enc = self
                    .security
                    .as_ref()
                    .expect("vmess case requires a security variant")
                    .client_security();
                config::client_params_vmess(
                    enc,
                    port,
                    target,
                    self.tls(),
                    self.network,
                    self.xhttp_mode,
                )
            }
        };
        // UDP rows: the VLESS UDP datagram tunnel dispatches on the params'
        // packet mode (`connect_udp` requires it; the proto has no
        // `packet_encoding` field — spec §4.3, no proto changes allowed).
        params.udp = self.udp;
        params
    }

    fn expected(&self) -> E2eExpect {
        let connect = if self.tls().expect_fallback() {
            ConnectExpect::ErrRealityFallback
        } else {
            ConnectExpect::Ok
        };
        E2eExpect {
            connect,
            ..Default::default()
        }
    }

    fn probe_target(&self, env: &ServerEnv) -> SocketAddr {
        // UDP rows echo datagrams to the dedicated UDP target (the probe's
        // `connect_udp` header destination). Inner-TLS rows splice to a
        // rustls echo target (the vision Direct path); a plain client
        // through a REALITY server is transparently proxied and terminates
        // at the server's dest — probe that instead of the plain echo.
        if self.app == AppKind::Udp {
            env.udp_echo.expect("udp rows spawn the UDP echo target")
        } else if self.app == AppKind::InnerTls {
            env.inner_tls_echo
                .expect("inner-tls rows spawn the inner TLS echo target")
        } else if self.tls().probe_dest() {
            env.tls_echo
        } else {
            env.echo
        }
    }

    fn spider_reaches_dest(&self) -> bool {
        self.tls().spider_reaches_dest()
    }

    fn client_trust(&self, certs: &Certs) {
        self.tls().client_trust(certs);
    }
}

#[cfg(test)]
mod tests {
    use super::super::variant::{
        Aes128GcmVariant, Chacha20Poly1305Variant, FingerprintTls, RealityTls, StandardTls,
    };
    use super::*;

    #[test]
    fn derived_labels() {
        assert_eq!(CaseSpec::vless().label(), "vless/tcp/tls");
        assert_eq!(
            CaseSpec::vmess(Aes128GcmVariant).label(),
            "vmess/tcp/tls/aes-128-gcm"
        );
        assert_eq!(
            CaseSpec::vmess(Chacha20Poly1305Variant).label(),
            "vmess/tcp/tls/chacha20-poly1305"
        );
        assert_eq!(
            CaseSpec::vless().with_tls(Box::new(StandardTls)).label(),
            "vless/tcp/tls-standard"
        );
        assert_eq!(
            CaseSpec::vless()
                .with_tls(Box::new(FingerprintTls("chrome")))
                .label(),
            "vless/tcp/tls-chrome"
        );
        assert_eq!(
            CaseSpec::vmess(Aes128GcmVariant)
                .with_tls(Box::new(RealityTls::fresh()))
                .label(),
            "vmess/tcp/reality/aes-128-gcm"
        );
        assert_eq!(
            CaseSpec::vless().with_flow(Flow::Vision).label(),
            "vless/xtls-rprx-vision/tcp/tls"
        );
        assert_eq!(
            CaseSpec::vless()
                .with_flow(Flow::Vision)
                .with_tls(Box::new(RealityTls::fresh()))
                .label(),
            "vless/xtls-rprx-vision/tcp/reality"
        );
        assert_eq!(
            CaseSpec::vless()
                .with_flow(Flow::Vision)
                .with_tls(Box::new(RealityTls::fresh()))
                .with_app(AppKind::InnerTls)
                .label(),
            "vless/xtls-rprx-vision/tcp/reality/inner-tls"
        );
        assert_eq!(
            CaseSpec::vless().with_app(AppKind::Udp).label(),
            "vless/tcp/tls/udp"
        );
        assert_eq!(
            CaseSpec::vless()
                .with_tls(Box::new(RealityTls::fresh()))
                .with_app(AppKind::Udp)
                .label(),
            "vless/tcp/reality/udp"
        );
    }

    #[test]
    fn udp_mode_plumbs_into_client_params() {
        use xray_tui_proto::proto_spec::ProtoSpec as _;

        // Default (TCP) rows keep `params.udp` unset.
        let tcp = CaseSpec::vless();
        let params = tcp.client_params(12345, "127.0.0.1:9999".parse().unwrap());
        assert_eq!(params.udp, None);

        // Raw mode: plumbed for the probe's connect_udp.
        let raw = CaseSpec::vless()
            .with_app(AppKind::Udp)
            .with_udp(PacketMode::Raw);
        let params = raw.client_params(12345, "127.0.0.1:9999".parse().unwrap());
        assert_eq!(params.udp, Some(PacketMode::Raw));

        // PacketAddr mode: same plumbing.
        let addr = CaseSpec::vless()
            .with_app(AppKind::Udp)
            .with_udp(PacketMode::PacketAddr);
        let params = addr.client_params(12345, "127.0.0.1:9999".parse().unwrap());
        assert_eq!(params.udp, Some(PacketMode::PacketAddr));

        // The vless client protocol payload still parses as a plain vless
        // config — the packet mode lives in the params, NOT the proto
        // (spec §4.3; xray-tui-proto is never modified).
        let protocol: xray_tui_proto::proto_spec::ProtocolConfig =
            serde_json::from_value(serde_json::to_value(params.protocol).unwrap()).unwrap();
        match protocol {
            xray_tui_proto::proto_spec::ProtocolConfig::Vless(cfg) => {
                assert_eq!(cfg.transport_type(), Some("tcp"));
            }
            other => panic!("expected vless config, got {other:?}"),
        }
    }

    #[test]
    fn cores_gate_defaults_to_both() {
        assert_eq!(
            CaseSpec::vless().cores(),
            &[CoreKind::Xray, CoreKind::SingBox]
        );
        assert_eq!(
            CaseSpec::vmess(Chacha20Poly1305Variant).cores(),
            &[CoreKind::Xray, CoreKind::SingBox]
        );
    }
}
