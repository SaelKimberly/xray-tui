use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    // Xray-core native
    Vmess,
    Vless,
    Shadowsocks,
    Shadowsocks2022,
    Socks,
    Http,
    Trojan,
    WireGuard,
    Hysteria2,
    DokodemoDoor,
    Freedom,
    Blackhole,
    Dns,
    Loopback,
    Custom,

    // Sing-box only
    Tuic,
    Hysteria,
    Naive,
    AnyTls,
    ShadowTls,
    Tor,
    Ssh,
    Tailscale,
    ShadowsocksR,
    Redirect,
    TProxy,
    Mixed,
}

impl Protocol {
    #[must_use]
    pub fn is_singbox_only(&self) -> bool {
        SINGBOX_ONLY_PROTOCOLS.contains(self)
    }

    /// Maps v2rayN `EConfigType` integer values to Protocol variants.
    #[must_use]
    pub const fn try_from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Vmess),
            2 => Some(Self::Custom),
            3 => Some(Self::Shadowsocks),
            4 => Some(Self::Socks),
            5 => Some(Self::Vless),
            6 => Some(Self::Trojan),
            7 => Some(Self::Hysteria2),
            8 => Some(Self::Tuic),
            9 => Some(Self::WireGuard),
            10 => Some(Self::Http),
            11 => Some(Self::AnyTls),
            12 => Some(Self::Naive),
            13 => Some(Self::ShadowsocksR),
            14 => Some(Self::Hysteria),
            15 => Some(Self::ShadowTls),
            16 => Some(Self::Tor),
            17 => Some(Self::Ssh),
            18 => Some(Self::Tailscale),
            19 => Some(Self::Redirect),
            20 => Some(Self::TProxy),
            21 => Some(Self::Mixed),
            22 => Some(Self::DokodemoDoor),
            23 => Some(Self::Freedom),
            24 => Some(Self::Blackhole),
            25 => Some(Self::Dns),
            26 => Some(Self::Loopback),
            27 => Some(Self::Shadowsocks2022),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_i32(self) -> i32 {
        match self {
            Self::Vmess => 1,
            Self::Custom => 2,
            Self::Shadowsocks => 3,
            Self::Socks => 4,
            Self::Vless => 5,
            Self::Trojan => 6,
            Self::Hysteria2 => 7,
            Self::Tuic => 8,
            Self::WireGuard => 9,
            Self::Http => 10,
            Self::AnyTls => 11,
            Self::Naive => 12,
            Self::ShadowsocksR => 13,
            Self::Hysteria => 14,
            Self::ShadowTls => 15,
            Self::Tor => 16,
            Self::Ssh => 17,
            Self::Tailscale => 18,
            Self::Redirect => 19,
            Self::TProxy => 20,
            Self::Mixed => 21,
            Self::DokodemoDoor => 22,
            Self::Freedom => 23,
            Self::Blackhole => 24,
            Self::Dns => 25,
            Self::Loopback => 26,
            Self::Shadowsocks2022 => 27,
        }
    }
}

pub const SINGBOX_ONLY_PROTOCOLS: &[Protocol] = &[
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
];

fn kebab_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_lower = false;
    for ch in s.chars() {
        if ch.is_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            prev_lower = false;
        } else if ch.is_ascii_digit() {
            if prev_lower && !out.is_empty() {
                out.push('-');
            }
            out.push(ch);
            prev_lower = false;
        } else {
            out.push(ch);
            prev_lower = true;
        }
    }
    out
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Ssh => "ssh".to_owned(),
            Self::TProxy => "t-proxy".to_owned(),
            Self::Shadowsocks2022 => "shadowsocks-2022".to_owned(),
            Self::Shadowsocks => "ss".to_owned(),
            Self::ShadowsocksR => "ssr".to_owned(),
            Self::Hysteria => "hy".to_owned(),
            Self::Hysteria2 => "hy2".to_owned(),
            Self::AnyTls => "any-tls".to_owned(),
            Self::ShadowTls => "shadow-tls".to_owned(),
            // All others: camelCase → kebab-case
            other => kebab_case(&format!("{other:?}")),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct ParseProtocolError(String);

impl fmt::Display for ParseProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid protocol: '{}'", self.0)
    }
}

impl FromStr for Protocol {
    type Err = ParseProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase().replace('-', "_");
        match normalized.as_str() {
            "vmess" => Ok(Self::Vmess),
            "vless" => Ok(Self::Vless),
            "shadowsocks" | "ss" => Ok(Self::Shadowsocks),
            "shadowsocks_2022" => Ok(Self::Shadowsocks2022),
            "socks" => Ok(Self::Socks),
            "http" => Ok(Self::Http),
            "trojan" => Ok(Self::Trojan),
            "wire_guard" | "wireguard" => Ok(Self::WireGuard),
            "hysteria_2" | "hysteria2" | "hy2" => Ok(Self::Hysteria2),
            "dokodemo_door" | "dokodemo-door" => Ok(Self::DokodemoDoor),
            "freedom" => Ok(Self::Freedom),
            "blackhole" => Ok(Self::Blackhole),
            "dns" => Ok(Self::Dns),
            "loopback" => Ok(Self::Loopback),
            "custom" => Ok(Self::Custom),
            "tuic" => Ok(Self::Tuic),
            "hysteria" | "hy" => Ok(Self::Hysteria),
            "naive" => Ok(Self::Naive),
            "any_tls" | "anytls" => Ok(Self::AnyTls),
            "shadow_tls" | "shadowtls" => Ok(Self::ShadowTls),
            "tor" => Ok(Self::Tor),
            "ssh" => Ok(Self::Ssh),
            "tailscale" => Ok(Self::Tailscale),
            "shadowsocks_r" | "shadowsocksr" | "ssr" => Ok(Self::ShadowsocksR),
            "redirect" => Ok(Self::Redirect),
            "t_proxy" | "tproxy" => Ok(Self::TProxy),
            "mixed" => Ok(Self::Mixed),
            _ => Err(ParseProtocolError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_kebab_case() {
        let cases = [
            (Protocol::Vmess, "vmess"),
            (Protocol::Vless, "vless"),
            (Protocol::Shadowsocks, "ss"),
            (Protocol::Shadowsocks2022, "shadowsocks-2022"),
            (Protocol::Socks, "socks"),
            (Protocol::Http, "http"),
            (Protocol::Trojan, "trojan"),
            (Protocol::WireGuard, "wire-guard"),
            (Protocol::Hysteria2, "hy2"),
            (Protocol::DokodemoDoor, "dokodemo-door"),
            (Protocol::Freedom, "freedom"),
            (Protocol::Blackhole, "blackhole"),
            (Protocol::Dns, "dns"),
            (Protocol::Loopback, "loopback"),
            (Protocol::Custom, "custom"),
            (Protocol::Tuic, "tuic"),
            (Protocol::Hysteria, "hy"),
            (Protocol::Naive, "naive"),
            (Protocol::AnyTls, "any-tls"),
            (Protocol::ShadowTls, "shadow-tls"),
            (Protocol::Tor, "tor"),
            (Protocol::Ssh, "ssh"),
            (Protocol::Tailscale, "tailscale"),
            (Protocol::ShadowsocksR, "ssr"),
            (Protocol::Redirect, "redirect"),
            (Protocol::TProxy, "t-proxy"),
            (Protocol::Mixed, "mixed"),
        ];
        for (p, expected) in &cases {
            assert_eq!(p.to_string(), *expected, "mismatch for {p:?}");
        }
    }

    #[test]
    fn from_str_round_trip_all_variants() {
        for variant in [
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
            let s = variant.to_string();
            let parsed: Protocol = s
                .parse()
                .unwrap_or_else(|_| panic!("failed to parse protocol string: {s}"));
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn is_singbox_only_consistency() {
        for variant in SINGBOX_ONLY_PROTOCOLS {
            assert!(
                variant.is_singbox_only(),
                "{variant:?} should be sing-box only"
            );
        }
        // Check a few xray-native ones are NOT singbox only
        assert!(!Protocol::Vmess.is_singbox_only());
        assert!(!Protocol::Trojan.is_singbox_only());
        assert!(!Protocol::Hysteria2.is_singbox_only());
    }

    #[test]
    fn from_str_accepts_compact_variants() {
        // Many sing-box only protocols are written without hyphen in their configs
        for (input, expected) in [
            ("wireguard", Protocol::WireGuard),
            ("hysteria2", Protocol::Hysteria2),
            ("anytls", Protocol::AnyTls),
            ("shadowtls", Protocol::ShadowTls),
            ("shadowsocksr", Protocol::ShadowsocksR),
            ("tproxy", Protocol::TProxy),
        ] {
            let parsed: Protocol = input
                .parse()
                .unwrap_or_else(|_| panic!("failed to parse {input}"));
            assert_eq!(parsed, expected, "compact variant {input}");
        }
    }

    #[test]
    fn from_str_rejects_invalid() {
        assert!("not-a-protocol".parse::<Protocol>().is_err());
    }

    #[test]
    fn serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_value(Protocol::DokodemoDoor).unwrap(),
            serde_json::json!("dokodemo_door")
        );
        assert_eq!(
            serde_json::to_value(Protocol::Shadowsocks2022).unwrap(),
            serde_json::json!("shadowsocks2022")
        );
    }
}
