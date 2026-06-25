use crate::core_type::CoreType;
use crate::protocol::{Protocol, SINGBOX_ONLY_PROTOCOLS};

/// Resolves which core a given protocol should use.
///
/// - `Some(CoreType::Auto)` or `None` → auto-detect from protocol
/// - `Some(core_type)` → forced override (user chose explicitly)
#[must_use]
pub fn resolve_core(protocol: Protocol, profile_override: Option<CoreType>) -> CoreType {
    match profile_override {
        Some(CoreType::Auto) | None => core_for_protocol(protocol),
        Some(core_type) => core_type,
    }
}

fn core_for_protocol(protocol: Protocol) -> CoreType {
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
                resolve_core(*protocol, None),
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
        ] {
            assert_eq!(
                resolve_core(protocol, None),
                CoreType::Xray,
                "{protocol} should resolve to Xray"
            );
        }
    }

    #[test]
    fn override_auto_delegates_to_auto_detect() {
        // Sing-box only protocol + Auto override → SingBox
        assert_eq!(
            resolve_core(Protocol::Tuic, Some(CoreType::Auto)),
            CoreType::SingBox,
        );
        // Xray protocol + Auto override → Xray
        assert_eq!(
            resolve_core(Protocol::Vmess, Some(CoreType::Auto)),
            CoreType::Xray,
        );
    }

    #[test]
    fn override_forced_wins() {
        for protocol in [Protocol::Vmess, Protocol::Tuic] {
            assert_eq!(
                resolve_core(protocol, Some(CoreType::Xray)),
                CoreType::Xray,
                "forced Xray should win for {protocol}"
            );
            assert_eq!(
                resolve_core(protocol, Some(CoreType::SingBox)),
                CoreType::SingBox,
                "forced SingBox should win for {protocol}"
            );
        }
    }
}
