use crate::core_type::CoreType;
use crate::protocol::{Protocol, SINGBOX_ONLY_PROTOCOLS};

/// Shadowsocks ciphers xray-core accepts, case-insensitive: the AEAD set from
/// `CipherType` (proxy/shadowsocks/config.proto) plus the `aead_*` and
/// `chacha20-poly1305` name aliases of `cipherFromString`
/// (infra/conf/shadowsocks.go), plus the 2022-blake3 family
/// (shadowaead_2022.List).
pub const XRAY_SS_METHODS: &[&str] = &[
    "aes-128-gcm",
    "aes-256-gcm",
    "chacha20-poly1305",
    "chacha20-ietf-poly1305",
    "xchacha20-poly1305",
    "xchacha20-ietf-poly1305",
    "aead_aes_128_gcm",
    "aead_aes_256_gcm",
    "aead_chacha20_poly1305",
    "aead_xchacha20_poly1305",
    "2022-blake3-aes-128-gcm",
    "2022-blake3-aes-256-gcm",
    "2022-blake3-chacha20-poly1305",
];

/// Shadowsocks methods sing-box accepts (docs/configuration/outbound/
/// shadowsocks.md — modern + legacy).
pub const SINGBOX_SS_METHODS: &[&str] = &[
    "2022-blake3-aes-128-gcm",
    "2022-blake3-aes-256-gcm",
    "2022-blake3-chacha20-poly1305",
    "none",
    "aes-128-gcm",
    "aes-192-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "xchacha20-ietf-poly1305",
    "aes-128-ctr",
    "aes-192-ctr",
    "aes-256-ctr",
    "aes-128-cfb",
    "aes-192-cfb",
    "aes-256-cfb",
    "rc4-md5",
    "chacha20-ietf",
    "xchacha20",
];

/// True when xray-core can build a shadowsocks outbound for `method`.
#[must_use]
pub fn xray_supports_ss_method(method: &str) -> bool {
    XRAY_SS_METHODS
        .iter()
        .any(|m| m.eq_ignore_ascii_case(method))
}

/// True when sing-box can build a shadowsocks outbound for `method`.
#[must_use]
pub fn singbox_supports_ss_method(method: &str) -> bool {
    SINGBOX_SS_METHODS
        .iter()
        .any(|m| m.eq_ignore_ascii_case(method))
}

/// True when at least one core can build a shadowsocks outbound for `method`.
#[must_use]
pub fn ss_method_supported(method: &str) -> bool {
    xray_supports_ss_method(method) || singbox_supports_ss_method(method)
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
    match profile_override {
        Some(CoreType::Auto) | None => core_for_protocol(protocol, ss_method),
        Some(core_type) => core_type,
    }
}

fn core_for_protocol(protocol: Protocol, ss_method: Option<&str>) -> CoreType {
    if matches!(protocol, Protocol::Shadowsocks | Protocol::Shadowsocks2022) {
        return match ss_method {
            Some(method) if xray_supports_ss_method(method) => CoreType::Xray,
            // Legacy/unknown ciphers: sing-box covers the legacy set; the
            // builder rejects what neither core can build.
            Some(_) => CoreType::SingBox,
            // Method unknown: keep the historical default; the config
            // builder validates before any core is launched.
            None => CoreType::Xray,
        };
    }
    if SINGBOX_ONLY_PROTOCOLS.contains(&protocol) {
        CoreType::SingBox
    } else {
        CoreType::Xray
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
}
