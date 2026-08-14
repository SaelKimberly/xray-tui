//! VLESS UDP `PacketConn`: the tunnel byte stream wrapped in `[2B BE len]`
//! datagram framing (spec §4.2), with the packetaddr destination codec
//! (§4.3) folded in per mode.

use std::io;
use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::vless::packetaddr;
use crate::protocol::vless::stream::Peel;
use crate::protocol::vless::udp::{read_packet, write_packet};

/// Maximum frame payload: the 2-byte length field is u16.
const MAX_FRAME: usize = 65_535;

/// Datagram mode: `Raw` (header-dest, xray-style) or `PacketAddr`
/// (per-packet magic-address destination, sing-box-style; spec §4.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PacketMode {
    Raw,
    PacketAddr,
}

/// A VLESS UDP connection: datagrams carried as `[2B BE len][payload]`
/// frames over the tunnel stream.
///
/// Owns the response-header peel: the server sends `[0x00, 0x00]` before
/// the first downlink frame (spec §4.4), consumed on the first `recv`.
pub struct PacketConn<S> {
    inner: S,
    mode: PacketMode,
    peel: Peel,
}

impl<S: AsyncRead + AsyncWrite + Unpin> PacketConn<S> {
    /// Wraps a tunnel stream in datagram framing for the given mode.
    pub const fn new(inner: S, mode: PacketMode) -> Self {
        Self {
            inner,
            mode,
            peel: Peel::new(),
        }
    }

    /// Sends one datagram.
    ///
    /// Raw: one `[len][payload]` frame. PacketAddr: prepends the
    /// magic-address destination header (spec §4.3) — header and payload go
    /// in ONE frame, mirroring the sing encoder which writes
    /// `AddrPortLen + payload` in a single buffer; the destination is
    /// required and the combined length must fit a u16 frame. Oversized
    /// datagrams are rejected before any byte is written.
    pub async fn send(&mut self, target: Option<SocketAddr>, payload: &[u8]) -> io::Result<()> {
        match self.mode {
            PacketMode::Raw => {
                reject_oversized(payload.len())?;
                write_packet(&mut self.inner, payload).await
            }
            PacketMode::PacketAddr => {
                let target = target.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "vless packetaddr send requires a destination",
                    )
                })?;
                let dest = packetaddr::encode_dest(target)?;
                let total = dest.len() + payload.len();
                reject_oversized(total)?;
                let mut frame = Vec::with_capacity(total);
                frame.extend_from_slice(&dest);
                frame.extend_from_slice(payload);
                write_packet(&mut self.inner, &frame).await
            }
        }
    }

    /// Receives one datagram.
    ///
    /// The response header precedes ALL downlink frames, so the peel runs
    /// before the first frame read. Returns `Ok(None)` on a clean EOF at a
    /// frame boundary. Raw: `(None, payload)`. PacketAddr: `(Some(dest),
    /// payload)` after validating the magic — a malformed destination
    /// header is an error, never delivered as garbage.
    pub async fn recv(&mut self) -> io::Result<Option<(Option<SocketAddr>, Vec<u8>)>> {
        if !self.peel.is_peeled() {
            self.peel.ensure_peeled(&mut self.inner).await?;
        }
        // A non-vless first byte (REALITY transparent-proxy fallback) has
        // no valid UDP framing; error rather than drop or mis-deliver it.
        if self.peel.take_passthrough().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "vless udp: non-vless response byte (REALITY fallback not supported for UDP)",
            ));
        }
        let Some(frame) = read_packet(&mut self.inner).await? else {
            return Ok(None);
        };
        match self.mode {
            PacketMode::Raw => Ok(Some((None, frame))),
            PacketMode::PacketAddr => {
                let (dest, payload) = packetaddr::decode_dest(&frame)?;
                Ok(Some((Some(dest), payload.to_vec())))
            }
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

        // send(Some(127.0.0.1:8080), b"p") → magic + 0x01 + addr + port + 'p'.
        let target = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        conn.send(Some(target), b"p").await.unwrap();
        let frame = read_packet(&mut server).await.unwrap().unwrap();
        let mut expected = packetaddr::encode_dest(target).unwrap();
        expected.push(b'p');
        assert_eq!(frame, expected);

        // Peer replies with a magic-address frame for a different dest.
        let reply_dest = "[::1]:53".parse::<SocketAddr>().unwrap();
        server.write_all(&[0x00, 0x00]).await.unwrap();
        let mut reply = packetaddr::encode_dest(reply_dest).unwrap();
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
}
