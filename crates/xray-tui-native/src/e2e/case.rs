//! Data-driven e2e case: protocol + optional payload-security variant.
//!
//! The `E2eCase` impl derives the server config, client params, expectations
//! and label from these parts — adding a variant is a new `SecurityVariant`
//! impl, not a new case struct.

use std::net::SocketAddr;

use super::{Certs, CoreKind, E2eCase, E2eExpect, SecurityVariant, ServerEnv, config};
use crate::NativeConnectParams;
use crate::security;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolKind {
    Vless,
    Vmess,
}

/// One e2e scenario described as data.
pub struct CaseSpec {
    pub protocol: ProtocolKind,
    pub security: Option<Box<dyn SecurityVariant>>,
}

impl CaseSpec {
    #[must_use]
    pub fn vless() -> Self {
        Self {
            protocol: ProtocolKind::Vless,
            security: None,
        }
    }

    #[must_use]
    pub fn vmess(variant: impl SecurityVariant + 'static) -> Self {
        Self {
            protocol: ProtocolKind::Vmess,
            security: Some(Box::new(variant)),
        }
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
        match (self.protocol, self.security.as_deref()) {
            (ProtocolKind::Vless, _) => "vless/tcp/tls".to_string(),
            (ProtocolKind::Vmess, Some(s)) => format!("vmess/tcp/tls/{}", s.name()),
            (ProtocolKind::Vmess, None) => "vmess/tcp/tls".to_string(),
        }
    }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        match self.protocol {
            ProtocolKind::Vless => config::vless_inbound(core, env),
            ProtocolKind::Vmess => {
                let security = self.security.as_ref().and_then(|s| s.server_security(core));
                config::vmess_inbound(core, env, security)
            }
        }
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        match self.protocol {
            ProtocolKind::Vless => config::client_params_vless(port, target),
            ProtocolKind::Vmess => {
                let enc = self
                    .security
                    .as_ref()
                    .expect("vmess case requires a security variant")
                    .client_security();
                config::client_params_vmess(enc, port, target)
            }
        }
    }

    fn expected(&self) -> E2eExpect {
        E2eExpect {
            status: 200,
            body: config::BODY.into(),
        }
    }

    fn client_trust(&self, certs: &Certs) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));
    }
}

#[cfg(test)]
mod tests {
    use super::super::variant::{Aes128GcmVariant, Chacha20Poly1305Variant};
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
