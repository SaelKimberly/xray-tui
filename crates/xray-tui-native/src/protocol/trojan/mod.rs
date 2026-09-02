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
use crate::addr::{
    TargetAddr, addr_port_last_tail_len, decode_addr_port_last, encode_addr_port_last,
};
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};

/// TCP command byte (`protocol.go` `commandTCP`).
const COMMAND_TCP: u8 = 1;
/// UDP command byte (`protocol.go` `commandUDP`).
const COMMAND_UDP: u8 = 3;
/// The protocol's record separator (`protocol.go` `crlf`).
const CRLF: [u8; 2] = [0x0d, 0x0a];
/// xray's UDP payload cap (`protocol.go` `maxLength = 8192`): its
/// `PacketReader` rejects anything longer, so the writer never emits a
/// bigger datagram. The read side has no cap — sing-box's `ReadPacket`
/// (`transport/trojan/protocol.go`) reads the length field's full u16
/// range, and we deliberately follow that permissive peer.
const MAX_UDP_PAYLOAD: usize = 8192;
/// The widest port-last wire address: `ATYP | len | domain(255) | port BE2`
/// — the domain form (IPv4 is 7 bytes, IPv6 19). Every wire address fits,
/// so the reader parses one on the stack and the writer's scratch starts
/// with room for one.
const MAX_WIRE_ADDR: usize = 1 + 1 + 255 + 2;
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

/// trojan has no response header to peel. UDP rides
/// [`connect_udp`] (command 3 + per-datagram address framing).
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

/// Encode the trojan UDP request header: command 3 + the session target
/// (port-last) — xray `writeHeader` with `commandUDP`, sing-box
/// `ClientHandshakePacket`.
fn encode_udp_request(key: &[u8; 56], target: &TargetAddr) -> Result<Vec<u8>, NativeError> {
    let mut out = Vec::with_capacity(56 + 2 + 1 + 1 + 2 + 16 + 2);
    out.extend_from_slice(key);
    out.extend_from_slice(&CRLF);
    out.push(COMMAND_UDP);
    out.extend_from_slice(&encode_addr_port_last(target)?);
    out.extend_from_slice(&CRLF);
    Ok(out)
}

/// Append a per-packet destination in trojan port-last form
/// (`ATYP | addr | port BE2`) — the address half of xray
/// `PacketWriter.writePacket` / sing-box `WritePacket`. This is the IP-only
/// half of [`crate::addr::encode_addr_port_last`]: a `SocketAddr` has no
/// domain form, so it cannot fail, and it writes into the caller's buffer
/// rather than allocating per datagram.
fn push_socket_addr_port_last(out: &mut Vec<u8>, dest: std::net::SocketAddr) {
    match dest.ip() {
        std::net::IpAddr::V4(ip) => {
            out.push(crate::addr::TROJAN_ATYP_IPV4);
            out.extend_from_slice(&ip.octets());
        }
        std::net::IpAddr::V6(ip) => {
            out.push(crate::addr::TROJAN_ATYP_IPV6);
            out.extend_from_slice(&ip.octets());
        }
    }
    out.extend_from_slice(&dest.port().to_be_bytes());
}

/// Read one trojan UDP datagram frame; `Ok(None)` on a clean EOF at a frame
/// boundary (xray `PacketReader` / sing-box `ReadPacket`). A truncated
/// frame — partial address, length, CRLF or short payload — is
/// `UnexpectedEof`.
async fn read_packet_frame<R: tokio::io::AsyncRead + Unpin>(
    r: &mut R,
) -> std::io::Result<Option<(TargetAddr, Vec<u8>)>> {
    use tokio::io::AsyncReadExt;

    // The whole wire address fits `MAX_WIRE_ADDR`, so it is read into one
    // stack buffer and parsed in place. A frame boundary is the only place
    // an EOF is legal, so the ATYP byte is a plain `read`: `read_exact`
    // would turn a clean end-of-stream into `UnexpectedEof`.
    let mut addr_buf = [0u8; MAX_WIRE_ADDR];
    if r.read(&mut addr_buf[..1]).await? == 0 {
        return Ok(None); // clean EOF at a frame boundary
    }
    let atyp = addr_buf[0];
    // The domain family prefixes the name with a length byte; the IP
    // families are fixed-width. `head` is the address bytes already read.
    let (head, domain_len) = if atyp == crate::addr::TROJAN_ATYP_DOMAIN {
        r.read_exact(&mut addr_buf[1..2]).await?;
        (2, addr_buf[1])
    } else {
        (1, 0)
    };
    // How many bytes follow the head — the family layout has one owner.
    let tail = addr_port_last_tail_len(atyp, domain_len).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("trojan udp frame: unknown address type {atyp}"),
        )
    })?;
    r.read_exact(&mut addr_buf[head..head + tail]).await?;

    // `addr_buf[..head + tail]` is now the full `ATYP|addr|port` address.
    let (dest, _) = decode_addr_port_last(&addr_buf[..head + tail]).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "trojan udp frame: malformed address",
        )
    })?;

    // 2-byte BE payload length: every u16 is a legal length. sing-box's
    // `ReadPacket` (`transport/trojan/protocol.go`) reads the field
    // uncapped, xray's `PacketReader` rejects more than `maxLength` = 8192
    // (`proxy/trojan/protocol.go`); we follow sing-box's permissive read
    // and keep xray's cap on the writer (`MAX_UDP_PAYLOAD`).
    let mut len_buf = [0u8; 2];
    r.read_exact(&mut len_buf).await?;
    let payload_len = usize::from(u16::from_be_bytes(len_buf));
    // CRLF separator.
    let mut crlf = [0u8; 2];
    r.read_exact(&mut crlf).await?;
    if crlf != CRLF {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "trojan udp frame: missing CRLF",
        ));
    }
    // Length-exact: a short payload read would deliver a truncated
    // datagram and desynchronize every later frame, so it is an error.
    let mut payload = vec![0u8; payload_len];
    r.read_exact(&mut payload).await?;
    Ok(Some((dest, payload)))
}

/// A trojan UDP connection over the tunnel stream (command 3).
///
/// Each datagram is an address-prefixed frame `ATYP|addr|port || len ||
/// CRLF || payload` in both directions — the address is per-packet (the
/// session target is the default when the caller passes no destination),
/// matching xray `PacketWriter`/`PacketReader` and sing-box `WritePacket`/
/// `ReadPacket`. There is no response header to peel (trojan relays raw).
pub struct PacketConn<S> {
    inner: S,
    /// The session target from the request header, pre-encoded port-last —
    /// the frame address when a `send` carries no per-packet destination
    /// (xray `PacketWriter` defaults `target := &w.Target`). The session
    /// target never changes, so it is encoded once, at construction.
    session_addr: Vec<u8>,
    /// Scratch for the outgoing frame, reused across datagrams: it grows to
    /// the session's largest frame and then stays put, so the steady state
    /// allocates nothing per datagram.
    frame: Vec<u8>,
}

impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin> PacketConn<S> {
    /// Wrap the tunnel stream (the command-3 header already written) and
    /// pre-encode the session target — the default frame address. Fails
    /// only on a target that has no wire form (a domain over 255 bytes).
    pub fn new(inner: S, target: &TargetAddr) -> Result<Self, NativeError> {
        Ok(Self {
            inner,
            session_addr: encode_addr_port_last(target)?,
            frame: Vec::with_capacity(MAX_WIRE_ADDR + 2 + 2),
        })
    }

    /// Send one datagram as `addr (port-last) || len(2B BE) || CRLF ||
    /// payload` — xray `PacketWriter.writePacket`, sing-box `WritePacket`.
    ///
    /// `dest: None` uses the session target (the request-header
    /// destination); `Some(addr)` carries that address in the frame —
    /// trojan frames every datagram with its own destination, so any
    /// per-packet address is legal. The frame is assembled in the reused
    /// scratch buffer and written with a single `write_all`: one datagram
    /// is one record on the wire.
    pub async fn send(
        &mut self,
        dest: Option<std::net::SocketAddr>,
        payload: &[u8],
    ) -> std::io::Result<()> {
        if payload.len() > MAX_UDP_PAYLOAD {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "trojan udp datagram too large ({}, max {MAX_UDP_PAYLOAD})",
                    payload.len()
                ),
            ));
        }
        let len = u16::try_from(payload.len()).expect("payload bounded by MAX_UDP_PAYLOAD");
        self.frame.clear();
        match dest {
            Some(sa) => push_socket_addr_port_last(&mut self.frame, sa),
            None => self.frame.extend_from_slice(&self.session_addr),
        }
        self.frame.extend_from_slice(&len.to_be_bytes());
        self.frame.extend_from_slice(&CRLF);
        self.frame.extend_from_slice(payload);
        self.inner.write_all(&self.frame).await
    }

    /// Receive one datagram. `Ok(None)` on a clean EOF at a frame boundary.
    ///
    /// Returns the frame's per-packet destination for the IP forms, and
    /// `None` for the domain form — no per-packet address. A reply
    /// addressed by domain is ordinary traffic, not an error: for a domain
    /// destination sing-box maps the reply address back to the original
    /// FQDN by default (`route/conn.go` wraps the relay in a
    /// `NATPacketConn` unless `udp_disable_domain_unmapping`), and xray's
    /// `PacketReader` parses the domain family too (`addrParser` carries
    /// `0x03`).
    pub async fn recv(
        &mut self,
    ) -> std::io::Result<Option<(Option<std::net::SocketAddr>, Vec<u8>)>> {
        let Some((dest, payload)) = read_packet_frame(&mut self.inner).await? else {
            return Ok(None);
        };
        let sa = match dest.host {
            crate::addr::Host::Ip(ip) => Some(std::net::SocketAddr::new(ip, dest.port)),
            // A domain has no `SocketAddr` form; the datagram still stands.
            crate::addr::Host::Domain(_) => None,
        };
        Ok(Some((sa, payload)))
    }
}

/// Connect through a Trojan outbound with a UDP datagram tunnel (command 3).
///
/// Writes the UDP request header (`key || CRLF || 3 || session-target
/// port-last || CRLF`) then wraps the stream in the address-framed
/// [`PacketConn`]. No server response header — the server relays the
/// target's UDP datagrams raw (xray `server.go` `handleUDP` / sing-box
/// `service.go`).
pub async fn connect_udp(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &TrojanConfig,
) -> Result<PacketConn<BoxStream>, NativeError> {
    let request = encode_udp_request(&auth_key(&cfg.password), &ctx.target)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "trojan udp request write",
            limit: timeout,
        })??;
    PacketConn::new(stream, &ctx.target)
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

    #[test]
    fn udp_request_header_uses_command_3() {
        let key = auth_key("pw");
        let req = encode_udp_request(
            &key,
            &TargetAddr::new(Host::Domain("example.com".into()), 443),
        )
        .unwrap();
        let mut expect = Vec::new();
        expect.extend_from_slice(&key);
        expect.extend_from_slice(&CRLF);
        expect.push(3); // command UDP
        expect.push(crate::addr::TROJAN_ATYP_DOMAIN);
        expect.push(11);
        expect.extend_from_slice(b"example.com");
        expect.extend_from_slice(&443u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        assert_eq!(req, expect);
    }

    #[tokio::test]
    async fn packet_conn_wire_frame_exact() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        // send(Some(v4:5353), b"hi") → the exact address-prefixed frame.
        conn.send(Some("9.9.9.9:5353".parse().unwrap()), b"hi")
            .await
            .unwrap();
        let mut got = vec![0u8; 64];
        let n = server.read(&mut got).await.unwrap();
        let mut expect = vec![0x01, 9, 9, 9, 9, 0x14, 0xe9]; // ATYP IPv4 + 9.9.9.9 + port 5353
        expect.extend_from_slice(&2u16.to_be_bytes()); // len
        expect.extend_from_slice(&CRLF);
        expect.extend_from_slice(b"hi");
        assert_eq!(&got[..n], &expect[..]);
    }

    #[tokio::test]
    async fn packet_conn_send_none_uses_session_target() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        conn.send(None, b"x").await.unwrap();
        let mut got = vec![0u8; 32];
        let n = server.read(&mut got).await.unwrap();
        let mut expect = vec![0x01, 1, 2, 3, 4, 0x00, 0x35]; // session target 1.2.3.4:53
        expect.extend_from_slice(&1u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        expect.push(b'x');
        assert_eq!(&got[..n], &expect[..]);
    }

    #[tokio::test]
    async fn packet_conn_recv_parses_frame() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        // Peer writes a reply frame: addr + len + CRLF + payload.
        let mut frame = vec![0x01, 8, 8, 8, 8, 0x00, 0x35];
        frame.extend_from_slice(&3u16.to_be_bytes());
        frame.extend_from_slice(&CRLF);
        frame.extend_from_slice(b"hey");
        server.write_all(&frame).await.unwrap();
        let (dest, payload) = conn.recv().await.unwrap().expect("a datagram");
        assert_eq!(dest, Some("8.8.8.8:53".parse().unwrap()));
        assert_eq!(payload, b"hey");
    }

    #[tokio::test]
    async fn packet_conn_recv_clean_eof_is_none() {
        let (client, server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        drop(server);
        assert!(conn.recv().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn packet_conn_recv_truncated_frame_is_error() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        server.write_all(&[0x01, 8, 8]).await.unwrap(); // ATYP IPv4 + 2 addr bytes only
        drop(server); // close the write half so read_exact hits EOF
        let err = conn.recv().await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn packet_conn_send_oversized_rejected() {
        let (client, _server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let big = vec![0u8; MAX_UDP_PAYLOAD + 1];
        let err = conn.send(None, &big).await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn connect_udp_writes_command_3_header() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let cfg: xray_tui_proto::proto_spec::TrojanConfig =
            serde_json::from_value(serde_json::json!({
                "schema": "Trojan",
                "password": "pw",
                "transport": { "type": "tcp" }
            }))
            .expect("trojan config parses");
        let params = crate::context::NativeConnectParams::new(
            xray_tui_proto::proto_spec::ProtocolConfig::Trojan(cfg.clone()),
            xray_tui_proto::proto_spec::endpoint::EndpointEssentials::new("1.2.3.4", 443),
            TargetAddr::new(Host::Domain("example.com".into()), 443),
        );
        let ctx = crate::context::LinkContext::new(
            params,
            TargetAddr::new(Host::Domain("example.com".into()), 443),
        );
        let conn = connect_udp(&ctx, Box::new(client), &cfg).await.unwrap();
        drop(conn);
        let mut got = vec![0u8; 128];
        let n = server.read(&mut got).await.unwrap();
        let key = auth_key("pw");
        let mut expect = Vec::new();
        expect.extend_from_slice(&key);
        expect.extend_from_slice(&CRLF);
        expect.push(3);
        expect.push(crate::addr::TROJAN_ATYP_DOMAIN);
        expect.push(11);
        expect.extend_from_slice(b"example.com");
        expect.extend_from_slice(&443u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        assert_eq!(&got[..n], &expect[..]);
    }

    /// A domain session target is pre-encoded once at construction, so a
    /// `send(None, ..)` emits the port-last domain form (`0x03 | len |
    /// name | port`) — the common case, and the one that used to clone a
    /// `String` per datagram.
    #[tokio::test]
    async fn packet_conn_send_none_uses_domain_session_target() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::Domain("example.com".into()), 443);
        let mut conn = PacketConn::new(client, &target).unwrap();
        conn.send(None, b"q").await.unwrap();
        let mut got = vec![0u8; 64];
        let n = server.read(&mut got).await.unwrap();
        let mut expect = vec![crate::addr::TROJAN_ATYP_DOMAIN, 11];
        expect.extend_from_slice(b"example.com");
        expect.extend_from_slice(&443u16.to_be_bytes()); // port LAST
        expect.extend_from_slice(&1u16.to_be_bytes()); // payload len
        expect.extend_from_slice(&CRLF);
        expect.push(b'q');
        assert_eq!(&got[..n], &expect[..]);
    }

    /// The IPv6 per-packet form: the SOCKS5 family byte `0x04` (NOT the
    /// VLESS/VMess `3`), the 16 octets, then the port — port-last.
    #[tokio::test]
    async fn packet_conn_send_ipv6_per_packet_address() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let ip: std::net::Ipv6Addr = "2001:db8::1".parse().unwrap();
        conn.send(Some(std::net::SocketAddr::from((ip, 443))), b"z")
            .await
            .unwrap();
        let mut got = [0u8; 1 + 16 + 2 + 2 + 2 + 1];
        server.read_exact(&mut got).await.unwrap();
        let mut expect = vec![crate::addr::TROJAN_ATYP_IPV6];
        expect.extend_from_slice(&ip.octets());
        expect.extend_from_slice(&443u16.to_be_bytes());
        expect.extend_from_slice(&1u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        expect.push(b'z');
        assert_eq!(&got[..], &expect[..]);
    }

    /// The scratch frame buffer is reset per datagram: two sends produce
    /// two independent frames, never the second appended to the first.
    #[tokio::test]
    async fn packet_conn_send_reuses_scratch_per_datagram() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        conn.send(None, b"one").await.unwrap();
        conn.send(Some("9.9.9.9:5353".parse().unwrap()), b"two")
            .await
            .unwrap();
        let mut got = [0u8; 2 * (7 + 2 + 2 + 3)];
        server.read_exact(&mut got).await.unwrap();
        let mut expect = vec![0x01, 1, 2, 3, 4, 0x00, 0x35]; // session target
        expect.extend_from_slice(&3u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        expect.extend_from_slice(b"one");
        expect.extend_from_slice(&[0x01, 9, 9, 9, 9, 0x14, 0xe9]); // 9.9.9.9:5353
        expect.extend_from_slice(&3u16.to_be_bytes());
        expect.extend_from_slice(&CRLF);
        expect.extend_from_slice(b"two");
        assert_eq!(&got[..], &expect[..]);
    }

    /// sing-box maps a UDP reply's address back to the original FQDN by
    /// default for a domain destination (`route/conn.go`;
    /// `udp_disable_domain_unmapping` opts out), so a domain-addressed
    /// reply frame is ordinary traffic: the datagram is delivered with no
    /// per-packet address instead of killing the tunnel.
    #[tokio::test]
    async fn packet_conn_recv_domain_reply_has_no_address() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::Domain("example.com".into()), 443);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let mut frame = vec![crate::addr::TROJAN_ATYP_DOMAIN, 11];
        frame.extend_from_slice(b"example.com");
        frame.extend_from_slice(&443u16.to_be_bytes());
        frame.extend_from_slice(&2u16.to_be_bytes());
        frame.extend_from_slice(&CRLF);
        frame.extend_from_slice(b"ok");
        server.write_all(&frame).await.unwrap();
        let (dest, payload) = conn.recv().await.unwrap().expect("a datagram");
        assert_eq!(dest, None);
        assert_eq!(payload, b"ok");
    }

    /// The payload read is length-exact: a frame that promises 5 bytes and
    /// delivers 3 is `UnexpectedEof`, never a short datagram. A single
    /// `read` here would hand back `abc` and desynchronize every later
    /// frame.
    #[tokio::test]
    async fn packet_conn_recv_short_payload_is_error() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let mut frame = vec![0x01, 8, 8, 8, 8, 0x00, 0x35];
        frame.extend_from_slice(&5u16.to_be_bytes()); // promises 5
        frame.extend_from_slice(&CRLF);
        frame.extend_from_slice(b"abc"); // delivers 3
        server.write_all(&frame).await.unwrap();
        drop(server); // no more bytes are coming
        let err = conn.recv().await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    /// One datagram split across two writer flights is still one datagram:
    /// the reader accumulates to the frame length instead of returning the
    /// first flight.
    #[tokio::test]
    async fn packet_conn_recv_reassembles_split_payload() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let mut first = vec![0x01, 8, 8, 8, 8, 0x00, 0x35];
        first.extend_from_slice(&6u16.to_be_bytes());
        first.extend_from_slice(&CRLF);
        first.extend_from_slice(b"abc"); // half the payload
        // The reader blocks on the rest before the second flight lands.
        let (received, ()) = tokio::join!(conn.recv(), async {
            server.write_all(&first).await.unwrap();
            tokio::task::yield_now().await;
            server.write_all(b"def").await.unwrap();
        });
        let (dest, payload) = received.unwrap().expect("a datagram");
        assert_eq!(dest, Some("8.8.8.8:53".parse().unwrap()));
        assert_eq!(payload, b"abcdef");
    }

    /// A separator that is not CRLF is malformed data, not a short read.
    #[tokio::test]
    async fn packet_conn_recv_bad_crlf_is_invalid_data() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let mut frame = vec![0x01, 8, 8, 8, 8, 0x00, 0x35];
        frame.extend_from_slice(&1u16.to_be_bytes());
        frame.extend_from_slice(&[0x0d, 0x00]); // CR without LF
        frame.push(b'x');
        server.write_all(&frame).await.unwrap();
        drop(server);
        let err = conn.recv().await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    /// `ADDR_TYPE_DOMAIN` (0x02) is the VLESS/VMess family byte, not one of
    /// trojan's SOCKS5 set: an unknown ATYP is rejected, never parsed as an
    /// address.
    #[tokio::test]
    async fn packet_conn_recv_unknown_atyp_is_invalid_data() {
        use tokio::io::AsyncWriteExt;
        let (client, mut server) = tokio::io::duplex(1024);
        let target = TargetAddr::new(Host::new("1.2.3.4"), 53);
        let mut conn = PacketConn::new(client, &target).unwrap();
        let frame = [
            crate::addr::ADDR_TYPE_DOMAIN,
            4,
            b'x',
            b'.',
            b'i',
            b'o',
            0,
            9,
        ];
        server.write_all(&frame).await.unwrap();
        drop(server); // a reader that keeps reading fails, never hangs
        let err = conn.recv().await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
