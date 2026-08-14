//! VLESS UDP `PacketConn`: the tunnel byte stream wrapped in `[2B BE len]`
//! datagram framing (spec §4.2), with the packetaddr destination codec
//! (§4.3) folded in per mode.

use std::io;
use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::vless::mux::UdpSession;
use crate::protocol::vless::packetaddr;
use crate::protocol::vless::stream::Peel;
use crate::protocol::vless::udp::{read_packet, write_packet};

/// Maximum frame payload: the 2-byte length field is u16.
const MAX_FRAME: usize = 65_535;

/// Datagram mode: `Raw` (header-dest, xray-style), `PacketAddr`
/// (per-packet address header `atyp|addr|port`, sing-box-style; spec §4.3),
/// or `XUdp` (datagrams over a mux UDP session — spec §4.1).
///
/// The packetaddr magic fqdn appears only in the header dest, never in the
/// per-packet frame. `XUdp` is not constructible via [`PacketConn::new`]
/// (a stream cannot carry it) — use [`PacketConn::xudp`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketMode {
    Raw,
    PacketAddr,
    XUdp,
}

/// A VLESS UDP connection.
///
/// Datagrams ride `[2B BE len][payload]` frames over the tunnel stream
/// (Raw / `PacketAddr`), or delegate entirely to a mux UDP session
/// (`XUdp` — the mux frames carry the length and the per-packet
/// destination).
///
/// The stream modes own the response-header peel: the server sends
/// `[0x00, 0x00]` before the first downlink frame (spec §4.4), consumed on
/// the first `recv`. `XUdp` needs no peel — the mux tunnel's connect
/// already consumed the header before the session opened.
pub struct PacketConn<S> {
    inner: PacketInner<S>,
    mode: PacketMode,
    peel: Peel,
}

/// The datagram carrier, selected by the mode: the tunnel byte stream
/// (Raw / `PacketAddr`) or the mux UDP session (`XUdp`).
enum PacketInner<S> {
    Stream(S),
    XUdp(UdpSession),
}

impl<S: AsyncRead + AsyncWrite + Unpin> PacketConn<S> {
    /// Wraps a tunnel stream in datagram framing for the given mode
    /// (`Raw` or `PacketAddr`). `XUdp` mode is not constructible here —
    /// the `XUdp` carrier is a [`UdpSession`], use [`PacketConn::xudp`].
    pub const fn new(inner: S, mode: PacketMode) -> Self {
        debug_assert!(
            !matches!(mode, PacketMode::XUdp),
            "vless: XUdp mode requires PacketConn::xudp (a UdpSession)"
        );
        Self {
            inner: PacketInner::Stream(inner),
            mode,
            peel: Peel::new(),
        }
    }

    /// Wraps a mux UDP session in the `XUdp` datagram API: `send`/`recv`
    /// delegate to the session — no 2-byte framing, the mux frames carry
    /// the length and the per-packet destination (spec §4.1, §5.2).
    pub(crate) const fn xudp(session: UdpSession) -> Self {
        Self {
            inner: PacketInner::XUdp(session),
            mode: PacketMode::XUdp,
            peel: Peel::new(),
        }
    }

    /// Sends one datagram.
    ///
    /// Raw: one `[len][payload]` frame. `PacketAddr`: prepends the
    /// per-packet address header (`atyp|addr|port`, spec §4.3) — header and
    /// payload go
    /// in ONE frame, mirroring the sing encoder which writes
    /// `AddrPortLen + payload` in a single buffer; the destination is
    /// required and the combined length must fit a u16 frame. Oversized
    /// datagrams are rejected before any byte is written. `XUdp`:
    /// `session.send_to(dest, payload)` — the destination is required (the
    /// mux frame carries it per packet).
    pub async fn send(&mut self, target: Option<SocketAddr>, payload: &[u8]) -> io::Result<()> {
        match self.mode {
            PacketMode::Raw => {
                reject_oversized(payload.len())?;
                write_packet(self.stream_mut()?, payload).await
            }
            PacketMode::PacketAddr => {
                let target = target.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "vless packetaddr send requires a destination",
                    )
                })?;
                let dest = packetaddr::encode_dest(target);
                let total = dest.len() + payload.len();
                reject_oversized(total)?;
                let mut frame = Vec::with_capacity(total);
                frame.extend_from_slice(&dest);
                frame.extend_from_slice(payload);
                write_packet(self.stream_mut()?, &frame).await
            }
            PacketMode::XUdp => {
                let target = target.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "vless xudp send requires a destination",
                    )
                })?;
                self.session_mut()?.send_to(target, payload).await
            }
        }
    }

    /// Receives one datagram.
    ///
    /// The stream modes run the response-header peel before the first
    /// frame read. Returns `Ok(None)` on a clean EOF at a frame boundary.
    /// Raw: `(None, payload)`. `PacketAddr`: `(Some(dest), payload)` after
    /// validating the address family (atyp) — a malformed destination
    /// header is an error, never delivered as garbage. `XUdp`:
    /// `session.recv_from()` → `(Some(dest), payload)` — the destination
    /// the server dispatched the packet to.
    pub async fn recv(&mut self) -> io::Result<Option<(Option<SocketAddr>, Vec<u8>)>> {
        // XUdp delegates entirely to the mux UDP session.
        if self.mode == PacketMode::XUdp {
            let session = self.session_mut()?;
            return Ok(session
                .recv_from()
                .await?
                .map(|(dest, payload)| (Some(dest), payload)));
        }
        let inner = match &mut self.inner {
            PacketInner::Stream(stream) => stream,
            PacketInner::XUdp(_) => unreachable!("XUdp handled above"),
        };
        if !self.peel.is_peeled() {
            self.peel.ensure_peeled(inner).await?;
        }
        // A non-vless first byte (REALITY transparent-proxy fallback) has
        // no valid UDP framing; error rather than drop or mis-deliver it.
        if self.peel.take_passthrough().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "vless udp: non-vless response byte (REALITY fallback not supported for UDP)",
            ));
        }
        let Some(frame) = read_packet(inner).await? else {
            return Ok(None);
        };
        match self.mode {
            PacketMode::Raw => Ok(Some((None, frame))),
            PacketMode::PacketAddr => {
                let (dest, payload) = packetaddr::decode_dest(&frame)?;
                Ok(Some((Some(dest), payload.to_vec())))
            }
            PacketMode::XUdp => unreachable!("XUdp handled above"),
        }
    }

    /// The tunnel stream in the Raw / `PacketAddr` modes — an error in
    /// `XUdp` mode (the state invariant: `XUdp` always carries a
    /// [`UdpSession`], built by [`PacketConn::xudp`]).
    fn stream_mut(&mut self) -> io::Result<&mut S> {
        match &mut self.inner {
            PacketInner::Stream(stream) => Ok(stream),
            PacketInner::XUdp(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "vless packet conn: stream modes require a tunnel stream",
            )),
        }
    }

    /// The mux UDP session in `XUdp` mode — an error in the stream modes.
    fn session_mut(&mut self) -> io::Result<&mut UdpSession> {
        match &mut self.inner {
            PacketInner::XUdp(session) => Ok(session),
            PacketInner::Stream(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "vless packet conn: XUdp mode requires a UdpSession",
            )),
        }
    }
}

/// Rejects a frame payload that does not fit the 2-byte length field.
fn reject_oversized(total: usize) -> io::Result<()> {
    if total > MAX_FRAME {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("vless udp datagram too large ({total} bytes, max {MAX_FRAME})"),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::vless::MuxTarget;
    use crate::protocol::vless::mux::{
        Frame, MuxClient, OPT_DATA, STATUS_END, STATUS_KEEP, STATUS_NEW, read_frame, write_frame,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn raw_send_recv() {
        let (client, mut server) = tokio::io::duplex(1024);
        let mut conn = PacketConn::new(client, PacketMode::Raw);

        // send(b"hi") → peer sees the exact frame [0x00,0x02,'h','i'].
        conn.send(None, b"hi").await.unwrap();
        let mut wire = [0u8; 4];
        server.read_exact(&mut wire).await.unwrap();
        assert_eq!(wire, [0x00, 0x02, b'h', b'i']);

        // Peer replies with the response header then a frame.
        server.write_all(&[0x00, 0x00]).await.unwrap();
        write_packet(&mut server, b"yo").await.unwrap();

        let (dest, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(dest, None);
        assert_eq!(payload, b"yo");
    }

    #[tokio::test]
    async fn packetaddr_send_recv() {
        let (client, mut server) = tokio::io::duplex(1024);
        let mut conn = PacketConn::new(client, PacketMode::PacketAddr);

        // send(Some(127.0.0.1:8080), b"p") → atyp(0x01) + addr + port + 'p'
        // (the per-packet address header; no magic in the frame).
        let target = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        conn.send(Some(target), b"p").await.unwrap();
        let frame = read_packet(&mut server).await.unwrap().unwrap();
        let mut expected = packetaddr::encode_dest(target);
        expected.push(b'p');
        assert_eq!(frame, expected);

        // Peer replies with a packetaddr address-header frame for a
        // different dest.
        let reply_dest = "[::1]:53".parse::<SocketAddr>().unwrap();
        server.write_all(&[0x00, 0x00]).await.unwrap();
        let mut reply = packetaddr::encode_dest(reply_dest);
        reply.extend_from_slice(b"ok");
        write_packet(&mut server, &reply).await.unwrap();

        let (dest, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(dest, Some(reply_dest));
        assert_eq!(payload, b"ok");
    }

    #[tokio::test]
    async fn peel_before_first_recv() {
        // Peer sends [0,0] (response header) then frames: the first recv()
        // consumes the header, both recvs return the frame payloads.
        let (client, mut server) = tokio::io::duplex(1024);
        let mut conn = PacketConn::new(client, PacketMode::Raw);

        server.write_all(&[0x00, 0x00]).await.unwrap();
        write_packet(&mut server, b"first").await.unwrap();
        write_packet(&mut server, b"second").await.unwrap();

        let (dest, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(dest, None);
        assert_eq!(payload, b"first");
        let (_, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(payload, b"second");
    }

    #[tokio::test]
    async fn eof_returns_none() {
        // Peer sends the response header, then closes at a frame boundary.
        let (client, server) = tokio::io::duplex(1024);
        let mut conn = PacketConn::new(client, PacketMode::Raw);
        let mut server = server;
        server.write_all(&[0x00, 0x00]).await.unwrap();
        drop(server);

        assert!(conn.recv().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn oversized_send_is_error() {
        // 70000-byte datagram: rejected in both modes before any write.
        let big = vec![0u8; 70_000];

        let (client, mut server) = tokio::io::duplex(65536);
        let mut raw = PacketConn::new(client, PacketMode::Raw);
        let err = raw.send(None, &big).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(raw);
        // No partial frame reached the peer.
        assert!(read_packet(&mut server).await.unwrap().is_none());

        let (client, mut server) = tokio::io::duplex(65536);
        let mut addr = PacketConn::new(client, PacketMode::PacketAddr);
        let err = addr
            .send(Some("127.0.0.1:1".parse().unwrap()), &big)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(addr);
        assert!(read_packet(&mut server).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn packetaddr_send_requires_dest() {
        // PacketAddr mode needs a per-datagram destination (spec §4.3);
        // send(None) must fail before any byte is written.
        let (client, mut server) = tokio::io::duplex(1024);
        let mut conn = PacketConn::new(client, PacketMode::PacketAddr);

        let err = conn.send(None, b"p").await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(conn);
        // No partial frame reached the peer.
        assert!(read_packet(&mut server).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn max_frame_boundary() {
        // The 2-byte length field caps a frame at 65535 payload bytes: that
        // exact size is accepted and delivered; 65536 is rejected before
        // any write.
        let (client, mut server) = tokio::io::duplex(1 << 17);
        let mut conn = PacketConn::new(client, PacketMode::Raw);

        let max = vec![0u8; MAX_FRAME];
        conn.send(None, &max).await.unwrap();
        let frame = read_packet(&mut server).await.unwrap().unwrap();
        assert_eq!(frame, max);

        let over = vec![0u8; MAX_FRAME + 1];
        let err = conn.send(None, &over).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(conn);
        // Nothing more reached the peer.
        assert!(read_packet(&mut server).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn xudp_send_recv_roundtrip() {
        // PacketConn in XUdp mode over a real UdpSession (a MuxClient over
        // a duplex): send(dest, payload) → the New frame (network UDP +
        // dest + GlobalID + payload, no 2-byte framing — the mux frames
        // carry the length); the peer's Keep reply with a per-packet dest
        // → recv() returns (Some(dest), payload).
        let (client, mut server) = tokio::io::duplex(1024);
        let mux = MuxClient::new(client);
        let session = mux.open_udp_session([0xAA; 8]).await.unwrap();
        // XUdp holds no stream, so S is unconstrained — pin it for the
        // test (production pins it via connect_udp's return type).
        let mut conn: PacketConn<crate::BoxStream> = PacketConn::xudp(session);

        let dest: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        conn.send(Some(dest), b"hi").await.unwrap();
        let frame = read_frame(&mut server).await.unwrap().unwrap();
        assert_eq!(frame.status, STATUS_NEW);
        assert_eq!(frame.target, Some(MuxTarget::Udp(dest)));
        assert_eq!(frame.global_id, Some([0xAA; 8]));
        assert_eq!(frame.payload.as_ref(), b"hi");

        // The peer replies with a Keep data frame carrying a per-packet dest.
        let reply_dest: SocketAddr = "127.0.0.1:5353".parse().unwrap();
        write_frame(
            &mut server,
            &Frame {
                session_id: 1,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: Some(MuxTarget::Udp(reply_dest)),
                global_id: None,
                payload: bytes::Bytes::from_static(b"yo"),
            },
        )
        .await
        .unwrap();

        let (dest, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(dest, Some(reply_dest));
        assert_eq!(payload, b"yo");
    }

    #[tokio::test]
    async fn xudp_send_requires_dest() {
        // XUdp sends carry the destination per datagram (the mux frame's
        // per-packet dest); send(None) must fail before any frame is queued.
        let (client, mut server) = tokio::io::duplex(1024);
        let mux = MuxClient::new(client);
        let session = mux.open_udp_session([0xAA; 8]).await.unwrap();
        let mut conn: PacketConn<crate::BoxStream> = PacketConn::xudp(session);

        let err = conn.send(None, b"p").await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(conn);
        // The failed send queued nothing (no eager New frame); dropping
        // the session fires the clean close — the peer sees only the End.
        let frame = read_frame(&mut server).await.unwrap().unwrap();
        assert_eq!(frame.status, STATUS_END);
        assert_eq!(frame.payload.as_ref(), b"");
    }
}
