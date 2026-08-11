//! Payload-security variants for the e2e pipeline. A variant names itself,
//! gates which cores support it, and supplies the security strings for the
//! server config and client params.

use super::{Certs, CoreKind, config};

/// TLS transport variant: stock rustls, the fingerprint engine, or REALITY.
///
/// Supplies the security-relevant strings for the server config and client
/// params, plus the harness-CA trust hook for the client path. A `CaseSpec`
/// without an explicit variant is `StandardTls`.
pub trait TlsVariant: Sync {
    /// Short variant name used in case labels ("tls-standard", "tls-chrome",
    /// "reality").
    fn name(&self) -> &'static str;
    /// The SNI the client offers (and the core's `serverNames` for REALITY).
    fn sni(&self) -> &'static str;
    /// Fingerprint id for the client TLS config ("chrome", "firefox", ...);
    /// `None` = stock rustls. A fingerprint id routes `wrap()` through the
    /// fingerprint engine AND selects a `Custom` provider in the client
    /// params.
    fn fingerprint(&self) -> Option<&'static str> {
        None
    }
    /// REALITY server private key (base64url); `None` = certificate TLS.
    fn reality_private_key(&self) -> Option<&str> {
        None
    }
    /// REALITY client public key (base64url).
    fn reality_pbk(&self) -> Option<&str> {
        None
    }
    /// REALITY short id (hex).
    fn reality_sid(&self) -> Option<&str> {
        None
    }
    /// Install the harness-CA trust this variant's client path needs. Stock
    /// TLS installs the rustls test config, fingerprint the engine's test CA,
    /// REALITY needs none (its auth key, not a PKI chain, authenticates the
    /// server).
    fn client_trust(&self, _certs: &Certs) {}
}

/// Stock rustls (the tier-1 behavior; no fingerprint, no REALITY).
pub struct StandardTls;

impl TlsVariant for StandardTls {
    fn name(&self) -> &'static str {
        "tls-standard"
    }
    fn sni(&self) -> &'static str {
        "localhost"
    }
    fn client_trust(&self, certs: &Certs) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        crate::security::tls::set_test_config(crate::security::tls::test_client_config(
            &certs.ca_der,
        ));
    }
}

/// The fingerprint engine with a named browser profile (`fp` id).
pub struct FingerprintTls(pub &'static str);

impl TlsVariant for FingerprintTls {
    fn name(&self) -> &'static str {
        match self.0 {
            "chrome" => "tls-chrome",
            "firefox" => "tls-firefox",
            other => other,
        }
    }
    fn sni(&self) -> &'static str {
        "localhost"
    }
    fn fingerprint(&self) -> Option<&'static str> {
        Some(self.0)
    }
    fn client_trust(&self, certs: &Certs) {
        crate::security::fingerprint::set_test_ca(&certs.ca_der);
    }
}

/// REALITY with a fresh keypair + short id (see [`RealityTls::fresh`]).
///
/// The private half is stored alongside `pbk` because the xray/sing-box
/// inbound needs it and a public key cannot be inverted; the brief's visible
/// `pbk`/`sid` pair is what the client consumes.
pub struct RealityTls {
    /// Client public key (base64url).
    pub pbk: String,
    /// Shared short id (8 hex chars).
    pub sid: String,
    /// Server private key (base64url) — the matching private half of `pbk`.
    private_key: String,
}

impl RealityTls {
    /// Generate a fresh REALITY keypair and short id for one e2e run.
    #[must_use]
    pub fn fresh() -> Self {
        let (private_key, pbk) = config::reality_keypair();
        Self {
            pbk,
            sid: config::reality_sid(),
            private_key,
        }
    }
}

impl TlsVariant for RealityTls {
    fn name(&self) -> &'static str {
        "reality"
    }
    fn sni(&self) -> &'static str {
        "localhost"
    }
    fn reality_private_key(&self) -> Option<&str> {
        Some(&self.private_key)
    }
    fn reality_pbk(&self) -> Option<&str> {
        Some(&self.pbk)
    }
    fn reality_sid(&self) -> Option<&str> {
        Some(&self.sid)
    }
    // `client_trust`: none — REALITY authenticates via the auth key.
}

/// Payload-security variant: names itself, gates which cores support it, and
/// supplies the security strings for the server config and client params.
pub trait SecurityVariant: Sync {
    fn name(&self) -> &'static str;
    fn cores(&self) -> &'static [CoreKind];
    fn server_security(&self, core: CoreKind) -> Option<&'static str>;
    fn client_security(&self) -> &'static str;
}

/// `VMess` payload security: AES-128-GCM (xray header security byte 3).
pub struct Aes128GcmVariant;

impl SecurityVariant for Aes128GcmVariant {
    fn name(&self) -> &'static str {
        "aes-128-gcm"
    }
    fn cores(&self) -> &'static [CoreKind] {
        &[CoreKind::Xray, CoreKind::SingBox]
    }
    fn server_security(&self, core: CoreKind) -> Option<&'static str> {
        match core {
            // xray inbound user security mirrors intent; sing-box rejects
            // the field outright (`json: unknown field "security"`).
            CoreKind::Xray => Some("aes-128-gcm"),
            CoreKind::SingBox => None,
        }
    }
    fn client_security(&self) -> &'static str {
        "aes-128-gcm"
    }
}

/// `VMess` payload security: chacha20-poly1305 (header security byte 4).
pub struct Chacha20Poly1305Variant;

impl SecurityVariant for Chacha20Poly1305Variant {
    fn name(&self) -> &'static str {
        "chacha20-poly1305"
    }
    fn cores(&self) -> &'static [CoreKind] {
        &[CoreKind::Xray, CoreKind::SingBox]
    }
    fn server_security(&self, core: CoreKind) -> Option<&'static str> {
        match core {
            CoreKind::Xray => Some("chacha20-poly1305"),
            CoreKind::SingBox => None,
        }
    }
    fn client_security(&self) -> &'static str {
        "chacha20-poly1305"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chacha_variant_supports_both_cores() {
        let v = Chacha20Poly1305Variant;
        assert_eq!(v.name(), "chacha20-poly1305");
        assert_eq!(v.cores(), &[CoreKind::Xray, CoreKind::SingBox]);
        assert_eq!(v.server_security(CoreKind::Xray), Some("chacha20-poly1305"));
        assert_eq!(v.server_security(CoreKind::SingBox), None); // sing-box: no field
        assert_eq!(v.client_security(), "chacha20-poly1305");
    }
}
