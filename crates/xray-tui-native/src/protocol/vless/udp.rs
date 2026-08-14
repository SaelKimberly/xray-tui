//! VLESS UDP packet framing codec (spec §4.2).
//!
//! After the request header the tunnel stream carries length-prefixed
//! packets in both directions: `[2 bytes big-endian length][payload]`.
//! The framing mirrors xray's `LengthPacketReader`/`LengthPacketWriter`
//! semantics: empty (len 0) frames are skipped, a clean EOF at a frame
//! boundary is `Ok(None)`, and a truncated frame (partial length or short
//! payload) is an `UnexpectedEof` error (spec §5.1/§6).

use std::io;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Reads one `[2B BE len][payload]` frame.
///
/// Returns `Ok(None)` on a clean EOF at a frame boundary (zero bytes read
/// for the length). Empty frames (len 0) are skipped. A truncated frame —
/// a partial length byte or a short payload at EOF — is `UnexpectedEof`.
pub async fn read_packet<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Vec<u8>>> {
    loop {
        let mut len = [0u8; 2];
        // Read the length byte-by-byte so a clean EOF (0 bytes) is
        // distinguishable from a truncated length (1 byte then EOF).
        let mut filled = 0;
        while filled < len.len() {
            match r.read(&mut len[filled..]).await {
                Ok(0) if filled == 0 => return Ok(None),
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "vless udp frame truncated (partial length)",
                    ));
                }
                Ok(n) => filled += n,
                Err(e) => return Err(e),
            }
        }
        let n = usize::from(u16::from_be_bytes(len));
        if n == 0 {
            continue; // skip empty frames
        }
        let mut payload = vec![0u8; n];
        r.read_exact(&mut payload).await?;
        return Ok(Some(payload));
    }
}

/// Writes one `[2B BE len][payload]` frame.
///
/// The payload must fit a u16 length (<= 65535); the caller (the
/// `PacketConn`) rejects larger datagrams before reaching the codec.
pub async fn write_packet<W: AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> io::Result<()> {
    let n = u16::try_from(payload.len()).expect("vless udp frame payload fits u16");
    let mut frame = Vec::with_capacity(payload.len() + 2);
    frame.extend_from_slice(&n.to_be_bytes());
    frame.extend_from_slice(payload);
    w.write_all(&frame).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn write_then_read_roundtrip() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        write_packet(&mut a, b"hello").await.unwrap();
        drop(a);
        assert_eq!(read_packet(&mut b).await.unwrap().unwrap(), b"hello");
    }

    #[tokio::test]
    async fn exact_wire_bytes() {
        let (mut a, mut b) = tokio::io::duplex(64);
        write_packet(&mut a, b"hi").await.unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x02, b'h', b'i']);
    }

    #[tokio::test]
    async fn split_frame_reads() {
        // One frame split across two writes: the length + first payload
        // byte, then the remainder; read_packet must reassemble across
        // partial reads.
        let (mut a, mut b) = tokio::io::duplex(4);
        a.write_all(&[0x00, 0x04, 0xAA]).await.unwrap();
        let reader = tokio::spawn(async move { read_packet(&mut b).await.unwrap().unwrap() });
        tokio::task::yield_now().await;
        a.write_all(&[0xBB, 0xCC, 0xDD]).await.unwrap();
        drop(a);
        assert_eq!(reader.await.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[tokio::test]
    async fn eof_at_boundary_is_none() {
        // Peer closes cleanly (no partial frame): clean end of the tunnel.
        let (a, mut b) = tokio::io::duplex(64);
        drop(a);
        assert!(read_packet(&mut b).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn empty_frame_skipped() {
        // A len=0 frame is skipped; the next frame's payload is returned.
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&[0x00, 0x00, 0x00, 0x02, b'h', b'i'])
            .await
            .unwrap();
        drop(a);
        assert_eq!(read_packet(&mut b).await.unwrap().unwrap(), b"hi");
    }

    #[tokio::test]
    async fn truncated_frame_is_error() {
        // Full length but a short payload at EOF: truncated frame.
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&[0x00, 0x05, b'a']).await.unwrap();
        drop(a);
        let err = read_packet(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn partial_length_then_eof_is_error() {
        // One length byte then EOF is a truncated frame, not a clean close
        // (spec §6): only a boundary-aligned EOF yields Ok(None).
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&[0x00]).await.unwrap();
        drop(a);
        let err = read_packet(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }
}
