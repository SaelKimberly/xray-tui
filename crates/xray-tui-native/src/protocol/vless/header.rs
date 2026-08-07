//! VLESS wire header codec (client side).
//!
//! Request header (v2ray `proxy/vless/encoding/encoding.go`,
//! `EncodeRequestHeader`): `version(1B, 0) | uuid(16B) | addon_len(1B) |
//! addons | command(1B) | [addr+port]`, with the addr encoded
//! PORT-FIRST (`PortThenAddress`). Command: 0x01 TCP, 0x02 UDP, 0x03 MUX.
//! Response header: `version(1B, echoed 0) | addon_len(1B) | addons`.

use xray_tui_proto::proto_spec::ProtocolKind;

use crate::addr::{encode_addr, TargetAddr};
use crate::error::NativeError;

pub const VERSION: u8 = 0;
pub const CMD_TCP: u8 = 1;
pub const CMD_UDP: u8 = 2;
pub const CMD_MUX: u8 = 3;

/// UUID bytes from a canonical UUID string.
pub fn uuid_bytes(uuid_str: &str) -> Result<[u8; 16], NativeError> {
    uuid::Uuid::parse_str(uuid_str)
        .map(|u| *u.as_bytes())
        .map_err(|e| NativeError::Config(format!("invalid uuid {uuid_str:?}: {e}")))
}

/// Encode a VLESS request header (no addons, version 0).
#[must_use]
pub fn encode_request(uuid: &[u8; 16], target: &TargetAddr, command: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.push(VERSION);
    out.extend_from_slice(uuid);
    out.push(0); // addon_len
    out.push(command);
    match command {
        CMD_TCP | CMD_UDP => {
            out.extend_from_slice(&encode_addr(target));
        }
        CMD_MUX => {
            // Mux uses a fixed v1.mux.cool:0 target on the wire.
            out.extend_from_slice(&encode_addr(&TargetAddr::new(
                crate::addr::Host::Domain("v1.mux.cool".into()),
                0,
            )));
        }
        _ => {}
    }
    out
}

/// Validate the first bytes of the response header; returns the addon length.
pub fn check_response_header(buf: &[u8]) -> Result<usize, NativeError> {
    if buf.len() < 2 {
        return Err(NativeError::Protocol {
            kind: ProtocolKind::Vless,
            detail: "response header truncated".into(),
        });
    }
    if buf[0] != VERSION {
        return Err(NativeError::Protocol {
            kind: ProtocolKind::Vless,
            detail: format!("bad response version {}", buf[0]),
        });
    }
    Ok(usize::from(buf[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::Host;

    const UUID_STR: &str = "00010203-0405-0607-0809-0a0b0c0d0e0f";

    fn uuid() -> [u8; 16] {
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ]
    }

    #[test]
    fn tcp_request_header_bytes_exact() {
        let target = TargetAddr::new(Host::new("127.0.0.1"), 8080);
        let got = encode_request(&uuid(), &target, CMD_TCP);
        // version, uuid, addon_len=0, cmd=1, port 0x1f90, type=1, 127.0.0.1
        let mut expected = vec![0x00];
        expected.extend_from_slice(&uuid());
        expected.push(0x00);
        expected.push(0x01);
        expected.extend_from_slice(&[0x1f, 0x90, 0x01, 127, 0, 0, 1]);
        assert_eq!(got, expected);
    }

    #[test]
    fn uuid_parse_match_expected_bytes() {
        assert_eq!(uuid_bytes(UUID_STR).unwrap(), uuid());
        assert!(uuid_bytes("not-a-uuid").is_err());
    }

    #[test]
    fn response_header_ok_and_bad() {
        assert_eq!(check_response_header(&[0x00, 0x00]).unwrap(), 0);
        assert_eq!(check_response_header(&[0x00, 0x02]).unwrap(), 2);
        assert!(check_response_header(&[0x01, 0x00]).is_err());
        assert!(check_response_header(&[0x00]).is_err());
    }
}
