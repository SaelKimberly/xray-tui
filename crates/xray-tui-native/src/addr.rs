use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use xray_tui_proto::proto_spec::HostKind;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

use crate::error::NativeError;

/// Remote host: parsed IP or DNS name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    Ip(IpAddr),
    Domain(String),
}

impl Host {
    /// Parse a host string: `IpAddr` when it parses, else `Domain`.
    #[must_use]
    pub fn new(host: &str) -> Self {
        host.parse::<IpAddr>()
            .map_or_else(|_| Self::Domain(host.to_string()), Self::Ip)
    }

    #[must_use]
    pub const fn kind(&self) -> HostKind {
        match self {
            Self::Ip(IpAddr::V4(_)) => HostKind::Ipv4,
            Self::Ip(IpAddr::V6(_)) => HostKind::Ipv6,
            Self::Domain(_) => HostKind::Dns,
        }
    }
}

impl From<&str> for Host {
    fn from(host: &str) -> Self {
        Self::new(host)
    }
}

/// A destination address on the wire: host + port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAddr {
    pub host: Host,
    pub port: u16,
}

impl TargetAddr {
    #[must_use]
    pub fn new(host: impl Into<Host>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Build from an endpoint's host/port (resolve later; IP hosts stay IP).
    #[must_use]
    pub fn from_endpoint(e: &EndpointEssentials) -> Self {
        Self {
            host: Host::new(&e.host),
            port: e.port,
        }
    }
}

// v2ray wire address types (common/protocol/payload.go AddressType).
pub const ADDR_TYPE_IPV4: u8 = 1;
pub const ADDR_TYPE_DOMAIN: u8 = 2;
pub const ADDR_TYPE_IPV6: u8 = 3;

/// Encode a destination in v2ray wire format: **port first, then address**
/// (`PortThenAddress`), address = type byte + payload.
pub fn encode_addr(target: &TargetAddr) -> Result<Vec<u8>, NativeError> {
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(&target.port.to_be_bytes());
    match &target.host {
        Host::Ip(IpAddr::V4(ip)) => {
            out.push(ADDR_TYPE_IPV4);
            out.extend_from_slice(&ip.octets());
        }
        Host::Ip(IpAddr::V6(ip)) => {
            out.push(ADDR_TYPE_IPV6);
            out.extend_from_slice(&ip.octets());
        }
        Host::Domain(domain) => {
            out.push(ADDR_TYPE_DOMAIN);
            let len = u8::try_from(domain.len()).map_err(|_| {
                NativeError::Config(format!("domain longer than 255 bytes: {domain}"))
            })?;
            out.push(len);
            out.extend_from_slice(domain.as_bytes());
        }
    }
    Ok(out)
}

/// Decode one wire address; returns the address plus the unconsumed tail.
#[must_use]
pub fn decode_addr(bytes: &[u8]) -> Option<(TargetAddr, &[u8])> {
    if bytes.len() < 3 {
        return None;
    }
    let port = u16::from_be_bytes([bytes[0], bytes[1]]);
    match bytes[2] {
        ADDR_TYPE_IPV4 => {
            if bytes.len() < 3 + 4 {
                return None;
            }
            let ip = Ipv4Addr::new(bytes[3], bytes[4], bytes[5], bytes[6]);
            Some((TargetAddr::new(Host::Ip(IpAddr::V4(ip)), port), &bytes[7..]))
        }
        ADDR_TYPE_DOMAIN => {
            if bytes.len() < 4 {
                return None;
            }
            let len = usize::from(bytes[3]);
            if bytes.len() < 4 + len {
                return None;
            }
            let domain = std::str::from_utf8(&bytes[4..4 + len]).ok()?.to_string();
            Some((
                TargetAddr::new(Host::Domain(domain), port),
                &bytes[4 + len..],
            ))
        }
        ADDR_TYPE_IPV6 => {
            if bytes.len() < 3 + 16 {
                return None;
            }
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&bytes[3..19]);
            Some((
                TargetAddr::new(Host::Ip(IpAddr::V6(Ipv6Addr::from(octets))), port),
                &bytes[19..],
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_ipv4_is_port_first() {
        let t = TargetAddr::new(Host::new("127.0.0.1"), 8080);
        assert_eq!(
            encode_addr(&t).unwrap(),
            vec![0x1f, 0x90, 0x01, 127, 0, 0, 1]
        );
    }

    #[test]
    fn encode_domain() {
        let t = TargetAddr::new(Host::Domain("example.com".into()), 443);
        assert_eq!(
            encode_addr(&t).unwrap(),
            vec![
                0x01, 0xbb, 0x02, 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o',
                b'm'
            ]
        );
    }

    #[test]
    fn encode_domain_too_long_is_config_error() {
        let t = TargetAddr::new(Host::Domain("a".repeat(256)), 443);
        let err = encode_addr(&t).unwrap_err();
        assert!(err.to_string().contains("domain longer than 255 bytes"));
    }

    #[test]
    fn encode_ipv6() {
        let t = TargetAddr::new(Host::new("::1"), 1234);
        let mut expected = vec![0x04, 0xd2, 0x03];
        expected.extend_from_slice(&[0u8; 15]);
        expected.push(1);
        assert_eq!(encode_addr(&t).unwrap(), expected);
    }

    #[test]
    fn decode_roundtrip_all_types() {
        for t in [
            TargetAddr::new(Host::new("1.2.3.4"), 80),
            TargetAddr::new(Host::Domain("sub.example.org".into()), 255),
            TargetAddr::new(Host::new("2001:db8::1"), 443),
        ] {
            let encoded = encode_addr(&t).unwrap();
            let (decoded, tail) = decode_addr(&encoded).expect("decode");
            assert_eq!(decoded, t);
            assert!(tail.is_empty());
        }
    }

    #[test]
    fn decode_domain_with_tail_keeps_rest() {
        let t = TargetAddr::new(Host::Domain("x.io".into()), 9);
        let mut bytes = encode_addr(&t).unwrap();
        bytes.push(0xAA);
        let (decoded, tail) = decode_addr(&bytes).expect("decode");
        assert_eq!(decoded, t);
        assert_eq!(tail, &[0xAA]);
    }

    #[test]
    fn decode_truncated_inputs_return_none() {
        assert!(decode_addr(&[]).is_none());
        assert!(decode_addr(&[0x00, 0x50]).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x01, 0x7f]).is_none());
        // Domain type byte present but the length byte is missing — used to
        // panic OOB on `bytes[3]`.
        assert!(decode_addr(&[0x00, 0x50, 0x02]).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x02, 0x05, b'a']).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x09, 0x00]).is_none());
    }
}
