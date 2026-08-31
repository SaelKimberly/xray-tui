//! Trojan — native client.
//!
//! Wire contract: xray-core / sing-box `proxy/trojan` (MIT; sing-box
//! `transport/trojan/protocol.go`). The client writes the request header
//! `hex(sha224(password)) (56 ASCII bytes) || CRLF || command || address ||
//! CRLF` where the address is **port-last** (`ATYP | addr | port BE2` —
//! trojan's `NewAddressParser` has no `PortFirst()` option, and sing-box's
//! `SocksaddrSerializer` is port-last too), command `1` = TCP / `3` = UDP.
//! There is **no server response header**: after the request the server
//! relays the target's bytes raw in both directions (xray `server.go`
//! `handleConnection` and `<sing>/protocol/trojan/client.go` both relay
//! without a response frame). The tunnel is pure passthrough.
//!
//! The trojan address uses the SOCKS5-ATYP family bytes (`0x01` IPv4 /
//! `0x03` domain / `0x04` IPv6) in port-last order —
//! [`crate::addr::encode_addr_port_last`] — NOT the VLESS/VMess
//! `ADDR_TYPE_*` (1/2/3) set, which those protocols' parsers expect.

use sha2::{Digest, Sha224};
use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::TrojanConfig;

use crate::BoxStream;
use crate::addr::{TargetAddr, encode_addr_port_last};
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};

/// TCP command byte (`protocol.go` `commandTCP`).
const COMMAND_TCP: u8 = 1;
/// The protocol's record separator (`protocol.go` `crlf`).
const CRLF: [u8; 2] = [0x0d, 0x0a];

/// The 56-byte lowercase hex encoding of `sha224(password)` — the wire auth
/// hash (`config.go` `hexSha224`).
#[must_use]
pub fn auth_key(password: &str) -> [u8; 56] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha224::digest(password.as_bytes());
    let mut out = [0u8; 56];
    for (i, byte) in digest.iter().enumerate() {
        out[i * 2] = HEX[usize::from(byte >> 4)];
        out[i * 2 + 1] = HEX[usize::from(byte & 0x0f)];
    }
    out
}

/// Encode the trojan TCP request header (`protocol.go` `writeHeader` /
/// `sing-box` `ClientHandshake`): `key || CRLF || command || addr || CRLF`.
fn encode_request(key: &[u8; 56], target: &TargetAddr) -> Result<Vec<u8>, NativeError> {
    let mut out = Vec::with_capacity(56 + 2 + 1 + 1 + 2 + 16 + 2);
    out.extend_from_slice(key);
    out.extend_from_slice(&CRLF);
    out.push(COMMAND_TCP);
    out.extend_from_slice(&encode_addr_port_last(target)?);
    out.extend_from_slice(&CRLF);
    Ok(out)
}

/// Connect through a Trojan outbound over an already-secured stream.
///
/// Writes the request header then returns the stream as the raw tunnel —
/// trojan has no response header to peel. UDP (command 3) stays
/// `NotImplemented` — the crate serves only VLESS over its UDP datagram
/// path ([`crate::protocol::connect_udp`]).
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &TrojanConfig,
) -> Result<BoxStream, NativeError> {
    let request = encode_request(&auth_key(&cfg.password), &ctx.target)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "trojan request write",
            limit: timeout,
        })??;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{Host, TargetAddr};

    /// NIST SHA-224("") = d14a028c...e42f — the digest the raw hash must match.
    #[test]
    fn auth_key_nist_empty_vector() {
        let key = auth_key("");
        let hex = key.iter().map(|b| char::from(*b)).collect::<String>();
        assert_eq!(
            hex,
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f"
        );
    }

    /// `auth_key` is genuinely the hex of `sha224(password)` (not a stale
    /// constant): recompute with the `sha2` crate and compare.
    #[test]
    fn auth_key_matches_sha224_wiring() {
        let password = "secret-token";
        let key = auth_key(password);
        let expect: Vec<u8> = Sha224::digest(password.as_bytes())
            .iter()
            .flat_map(|b| format!("{b:02x}").into_bytes())
            .collect();
        assert_eq!(&key[..], &expect[..]);
    }

    #[test]
    fn request_header_wire_order() {
        let key = auth_key("pw");
        // Domain target: ATYP(1) len(11) "example.com" port(443 BE) — the
        // address comes BEFORE the port (port-last), and there is a CRLF at
        // both the start and the end.
        let req = encode_request(
            &key,
            &TargetAddr::new(Host::Domain("example.com".into()), 443),
        )
        .unwrap();
        let mut expect = Vec::new();
        expect.extend_from_slice(&key);
        expect.extend_from_slice(&CRLF);
        expect.push(1); // command TCP
        expect.push(crate::addr::TROJAN_ATYP_DOMAIN); // ATYP (SOCKS5 0x03)
        expect.push(11); // domain len ("example.com")
        expect.extend_from_slice(b"example.com");
        expect.extend_from_slice(&443u16.to_be_bytes()); // port LAST
        expect.extend_from_slice(&CRLF);
        assert_eq!(req, expect);
    }
}
