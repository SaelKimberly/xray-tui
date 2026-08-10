//! Payload-security variants for the e2e pipeline. A variant names itself,
//! gates which cores support it, and supplies the security strings for the
//! server config and client params.

use super::{CoreKind, SecurityVariant};

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
