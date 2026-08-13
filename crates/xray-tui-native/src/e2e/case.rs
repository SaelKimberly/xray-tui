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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolKind {
    Vless,
    Vmess,
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
}

impl CaseSpec {
    #[must_use]
    pub fn vless() -> Self {
        Self {
            protocol: ProtocolKind::Vless,
            security: None,
            tls: None,
            network: "tcp",
        }
    }

    #[must_use]
    pub fn vmess(variant: impl SecurityVariant + 'static) -> Self {
        Self {
            protocol: ProtocolKind::Vmess,
            security: Some(Box::new(variant)),
            tls: None,
            network: "tcp",
        }
    }

    /// Select the transport ("tcp", "ws", "grpc").
    #[must_use]
    pub const fn with_network(mut self, network: &'static str) -> Self {
        self.network = network;
        self
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
        let tls = self.tls.as_deref().map_or("tls", TlsVariant::name);
        let sec = self
            .security
            .as_ref()
            .map_or(String::new(), |s| format!("/{}", s.name()));
        format!("{proto}/{}/{tls}{sec}", self.network)
    }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        match self.protocol {
            ProtocolKind::Vless => config::vless_inbound(core, env, self.tls(), self.network),
            ProtocolKind::Vmess => {
                let security = self.security.as_ref().and_then(|s| s.server_security(core));
                config::vmess_inbound(core, env, security, self.tls())
            }
        }
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        match self.protocol {
            ProtocolKind::Vless => {
                config::client_params_vless(port, target, self.tls(), self.network)
            }
            ProtocolKind::Vmess => {
                let enc = self
                    .security
                    .as_ref()
                    .expect("vmess case requires a security variant")
                    .client_security();
                config::client_params_vmess(enc, port, target, self.tls())
            }
        }
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
        // A plain client through a REALITY server is transparently proxied
        // and terminates at the server's dest — probe that instead of the
        // plain echo.
        if self.tls().probe_dest() {
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
