//! Core-resolution mapping — canonical logic lives in the proto crate.
//!
//! This module is an adapter shim over
//! `xray_tui_proto::proto_spec::core_mapping`: the shared SS-method
//! tables/helpers are re-exported verbatim, and [`resolve_core`] adapts the
//! proto `ProtocolKind`/`CoreType` signature to the core's `Protocol`/
//! `CoreType` (which carries `Auto`) so all existing callers compile
//! unchanged.

use crate::core_type::CoreType;
use crate::protocol::Protocol;
use xray_tui_proto::ProtocolKind;
use xray_tui_proto::proto_spec::CoreType as ProtoCoreType;
use xray_tui_proto::proto_spec::core_mapping;

#[cfg(test)]
use crate::protocol::SINGBOX_ONLY_PROTOCOLS;
pub use xray_tui_proto::proto_spec::core_mapping::{
    SINGBOX_SS_METHODS, XRAY_SS_METHODS, singbox_supports_ss_method, ss_method_supported,
    xray_supports_ss_method,
};

/// Core-side conversion: [`Protocol`] → [`ProtocolKind`].
impl From<Protocol> for ProtocolKind {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Vmess => Self::Vmess,
            Protocol::Vless => Self::Vless,
            Protocol::Shadowsocks => Self::Shadowsocks,
            Protocol::Shadowsocks2022 => Self::Shadowsocks2022,
            Protocol::Socks => Self::Socks,
            Protocol::Http => Self::Http,
            Protocol::Trojan => Self::Trojan,
            Protocol::WireGuard => Self::WireGuard,
            Protocol::Hysteria2 => Self::Hysteria2,
            Protocol::DokodemoDoor => Self::DokodemoDoor,
            Protocol::Freedom => Self::Freedom,
            Protocol::Blackhole => Self::Blackhole,
            Protocol::Dns => Self::Dns,
            Protocol::Loopback => Self::Loopback,
            Protocol::Custom => Self::Custom,
            Protocol::Tuic => Self::Tuic,
            Protocol::Hysteria => Self::Hysteria,
            Protocol::Naive => Self::Naive,
            Protocol::AnyTls => Self::AnyTls,
            Protocol::ShadowTls => Self::ShadowTls,
            Protocol::Tor => Self::Tor,
            Protocol::Ssh => Self::Ssh,
            Protocol::Tailscale => Self::Tailscale,
            Protocol::ShadowsocksR => Self::ShadowsocksR,
            Protocol::Redirect => Self::Redirect,
            Protocol::TProxy => Self::TProxy,
            Protocol::Mixed => Self::Mixed,
        }
    }
}

/// Core-side conversion: [`ProtocolKind`] → [`Protocol`] — inverse of the
/// above. Used by the typed ping engine to derive the legacy `config_type`
/// integer (via `Protocol::to_i32`) for `ProfileKey`.
impl From<ProtocolKind> for Protocol {
    fn from(kind: ProtocolKind) -> Self {
        match kind {
            ProtocolKind::Vmess => Self::Vmess,
            ProtocolKind::Vless => Self::Vless,
            ProtocolKind::Shadowsocks => Self::Shadowsocks,
            ProtocolKind::Shadowsocks2022 => Self::Shadowsocks2022,
            ProtocolKind::Socks => Self::Socks,
            ProtocolKind::Http => Self::Http,
            ProtocolKind::Trojan => Self::Trojan,
            ProtocolKind::WireGuard => Self::WireGuard,
            ProtocolKind::Hysteria2 => Self::Hysteria2,
            ProtocolKind::DokodemoDoor => Self::DokodemoDoor,
            ProtocolKind::Freedom => Self::Freedom,
            ProtocolKind::Blackhole => Self::Blackhole,
            ProtocolKind::Dns => Self::Dns,
            ProtocolKind::Loopback => Self::Loopback,
            ProtocolKind::Custom => Self::Custom,
            ProtocolKind::Tuic => Self::Tuic,
            ProtocolKind::Hysteria => Self::Hysteria,
            ProtocolKind::Naive => Self::Naive,
            ProtocolKind::AnyTls => Self::AnyTls,
            ProtocolKind::ShadowTls => Self::ShadowTls,
            ProtocolKind::Tor => Self::Tor,
            ProtocolKind::Ssh => Self::Ssh,
            ProtocolKind::Tailscale => Self::Tailscale,
            ProtocolKind::ShadowsocksR => Self::ShadowsocksR,
            ProtocolKind::Redirect => Self::Redirect,
            ProtocolKind::TProxy => Self::TProxy,
            ProtocolKind::Mixed => Self::Mixed,
        }
    }
}

/// Resolves which core a given protocol should use.
///
/// - `Some(CoreType::Auto)` or `None` → auto-detect from protocol
/// - `Some(core_type)` → forced override (user chose explicitly)
///
/// Shadowsocks core depends on the cipher: xray-core supports only AEAD +
/// 2022-blake3, so legacy ciphers (cfb/ctr/rc4-md5/none/...) route to
/// sing-box, which implements the full legacy set. `ss_method` is the
/// profile's method string for Shadowsocks/Shadowsocks2022 (other protocols
/// ignore it). A forced override always wins — the config builder then
/// validates the cipher against that core.
#[must_use]
pub fn resolve_core(
    protocol: Protocol,
    profile_override: Option<CoreType>,
    ss_method: Option<&str>,
) -> CoreType {
    let kind = ProtocolKind::from(protocol);
    let override_ = match profile_override {
        Some(CoreType::Auto) | None => None,
        Some(CoreType::Xray) => Some(ProtoCoreType::Xray),
        Some(CoreType::SingBox) => Some(ProtoCoreType::SingBox),
    };
    match core_mapping::resolve_core(kind, override_, ss_method) {
        ProtoCoreType::Xray => CoreType::Xray,
        ProtoCoreType::SingBox => CoreType::SingBox,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Protocol;

    #[test]
    fn sing_box_only_protocols_resolve_to_singbox() {
        for protocol in SINGBOX_ONLY_PROTOCOLS {
            assert_eq!(
                resolve_core(*protocol, None, None),
                CoreType::SingBox,
                "{protocol} should resolve to SingBox"
            );
        }
    }

    #[test]
    fn xray_protocols_resolve_to_xray() {
        for protocol in [
            Protocol::Vmess,
            Protocol::Vless,
            Protocol::Socks,
            Protocol::Http,
            Protocol::Trojan,
            Protocol::WireGuard,
            Protocol::Hysteria2,
            Protocol::DokodemoDoor,
            Protocol::Freedom,
            Protocol::Blackhole,
            Protocol::Dns,
            Protocol::Loopback,
            Protocol::Custom,
        ] {
            assert_eq!(
                resolve_core(protocol, None, None),
                CoreType::Xray,
                "{protocol} should resolve to Xray"
            );
        }
    }

    #[test]
    fn shadowsocks_aead_ciphers_resolve_to_xray() {
        for method in [
            "aes-128-gcm",
            "aes-256-gcm",
            "chacha20-poly1305",
            "chacha20-ietf-poly1305",
            "xchacha20-poly1305",
            "xchacha20-ietf-poly1305",
            "AES-256-GCM", // case-insensitive, like xray's cipherFromString
            "2022-blake3-aes-128-gcm",
            "2022-blake3-aes-256-gcm",
            "2022-blake3-chacha20-poly1305",
        ] {
            assert_eq!(
                resolve_core(Protocol::Shadowsocks, None, Some(method)),
                CoreType::Xray,
                "method {method} should resolve to Xray"
            );
        }
    }

    #[test]
    fn shadowsocks_legacy_ciphers_resolve_to_singbox() {
        // xray-core's CipherType enum has no CFB/CTR/rc4/none — sing-box does.
        for method in [
            "aes-128-cfb",
            "aes-192-cfb",
            "aes-256-cfb",
            "aes-128-ctr",
            "aes-192-ctr",
            "aes-256-ctr",
            "rc4-md5",
            "chacha20-ietf",
            "xchacha20",
            "none",
        ] {
            assert_eq!(
                resolve_core(Protocol::Shadowsocks, None, Some(method)),
                CoreType::SingBox,
                "method {method} should resolve to SingBox"
            );
        }
    }

    #[test]
    fn shadowsocks_unknown_cipher_resolves_to_singbox() {
        // Unsupported everywhere: sing-box chosen so the builder's clearer
        // error ("not supported by sing-box") surfaces, never a core crash.
        assert_eq!(
            resolve_core(Protocol::Shadowsocks, None, Some("salsa20")),
            CoreType::SingBox,
        );
    }

    #[test]
    fn shadowsocks_2022_method_resolves_to_xray() {
        assert_eq!(
            resolve_core(
                Protocol::Shadowsocks2022,
                None,
                Some("2022-blake3-aes-128-gcm")
            ),
            CoreType::Xray,
        );
    }

    #[test]
    fn shadowsocks_without_method_keeps_xray_default() {
        assert_eq!(
            resolve_core(Protocol::Shadowsocks, None, None),
            CoreType::Xray,
        );
    }

    #[test]
    fn forced_override_wins_over_cipher_routing() {
        // Explicit user choice beats auto-routing; the builder then validates
        // the cipher against that core and errors clearly.
        assert_eq!(
            resolve_core(
                Protocol::Shadowsocks,
                Some(CoreType::Xray),
                Some("aes-256-cfb")
            ),
            CoreType::Xray,
        );
        assert_eq!(
            resolve_core(
                Protocol::Shadowsocks,
                Some(CoreType::SingBox),
                Some("aes-256-gcm")
            ),
            CoreType::SingBox,
        );
    }

    #[test]
    fn xray_supports_only_aead_and_2022() {
        assert!(xray_supports_ss_method("aes-256-gcm"));
        assert!(xray_supports_ss_method("aead_aes_128_gcm"));
        assert!(xray_supports_ss_method("2022-blake3-chacha20-poly1305"));
        assert!(!xray_supports_ss_method("aes-256-cfb"));
        assert!(!xray_supports_ss_method("none"));
        assert!(!xray_supports_ss_method("salsa20"));
    }

    #[test]
    fn singbox_supports_modern_and_legacy() {
        assert!(singbox_supports_ss_method("aes-256-cfb"));
        assert!(singbox_supports_ss_method("none"));
        assert!(singbox_supports_ss_method("aes-192-gcm"));
        assert!(singbox_supports_ss_method("2022-blake3-aes-256-gcm"));
        assert!(!singbox_supports_ss_method("salsa20"));
        assert!(!singbox_supports_ss_method("chacha20")); // bare, non-ietf
    }

    #[test]
    fn override_auto_delegates_to_auto_detect() {
        // Sing-box only protocol + Auto override → SingBox
        assert_eq!(
            resolve_core(Protocol::Tuic, Some(CoreType::Auto), None),
            CoreType::SingBox,
        );
        // Xray protocol + Auto override → Xray
        assert_eq!(
            resolve_core(Protocol::Vmess, Some(CoreType::Auto), None),
            CoreType::Xray,
        );
    }

    #[test]
    fn override_forced_wins() {
        for protocol in [Protocol::Vmess, Protocol::Tuic] {
            assert_eq!(
                resolve_core(protocol, Some(CoreType::Xray), None),
                CoreType::Xray,
                "forced Xray should win for {protocol}"
            );
            assert_eq!(
                resolve_core(protocol, Some(CoreType::SingBox), None),
                CoreType::SingBox,
                "forced SingBox should win for {protocol}"
            );
        }
    }

    #[test]
    fn protocol_to_kind_conversion_is_total() {
        for protocol in [
            Protocol::Vmess,
            Protocol::Vless,
            Protocol::Shadowsocks,
            Protocol::Shadowsocks2022,
            Protocol::Socks,
            Protocol::Http,
            Protocol::Trojan,
            Protocol::WireGuard,
            Protocol::Hysteria2,
            Protocol::DokodemoDoor,
            Protocol::Freedom,
            Protocol::Blackhole,
            Protocol::Dns,
            Protocol::Loopback,
            Protocol::Custom,
            Protocol::Tuic,
            Protocol::Hysteria,
            Protocol::Naive,
            Protocol::AnyTls,
            Protocol::ShadowTls,
            Protocol::Tor,
            Protocol::Ssh,
            Protocol::Tailscale,
            Protocol::ShadowsocksR,
            Protocol::Redirect,
            Protocol::TProxy,
            Protocol::Mixed,
        ] {
            let kind = ProtocolKind::from(protocol);
            // Every core Protocol maps to a kind; resolve_core must accept it.
            let _ = core_mapping::resolve_core(kind, None, None);
        }
    }
}
