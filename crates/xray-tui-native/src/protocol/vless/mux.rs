//! VLESS v1.mux.cool frame codec (SP2 of the parity program).
//!
//! Wire format (xray-core `common/mux/frame.go` + `writer.go`):
//!
//! ```text
//! [2B meta_len BE][2B session_id BE][1B status][1B option]
//! [New: 1B network + port-first addr][2B data_len][payload]
//! ```
//!
//! The 2-byte `meta_len` covers ONLY the metadata (session id, status,
//! option, and the New-frame target); `writeMetaWithFrame` appends
//! `[2B data_len][data]` after it. Status: `New` `0x01` / `Keep` `0x02` /
//! `End` `0x03` / `KeepAlive` `0x04`. Option: `Data` `0x01` / `Error`
//! `0x02`. Addresses are port-first (`[2B port][atyp][addr]`, atyp IPv4
//! `0x01` / Domain `0x02` / IPv6 `0x03`) exactly like the VLESS request
//! header. `meta_len` is capped at 512 on the server side (mirrored on
//! read); application data is chunked at 8 KiB by the client. The mux
//! connection itself targets the fixed `v1.mux.cool:9527` destination.
//!
//! Scope: TCP targets only — `MuxTarget` has no UDP variant (UDP mux /
//! XUDP is a later plan), and a New frame with a non-TCP network byte is
//! a protocol error. The server's response writer starts at Keep
//! (`NewResponseWriter` sets `followup: true`), so `target` is only ever
//! `Some` on the New frames the client writes.
//!
//! Items are `pub` inside the `pub(crate)` module (effective
//! `pub(crate)`), mirroring `udp.rs`.

// Temporary: the MuxClient (next task) consumes every codec item; this is
// removed when the multiplexer is wired (same lifecycle as the SP1
// udp.rs `#[allow(dead_code)]`, dropped by its Task 2).
#![allow(dead_code)]

use std::io;
use std::net::{IpAddr, SocketAddr};

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::addr::{ADDR_TYPE_DOMAIN, ADDR_TYPE_IPV4, ADDR_TYPE_IPV6, Host, decode_addr};

/// The mux connection's fixed destination inside the VLESS header
/// (`v1.mux.cool:9527` — the VLESS Mux command target).
pub const MUX_DEST: &str = "v1.mux.cool";
pub const MUX_PORT: u16 = 9527;

pub const STATUS_NEW: u8 = 0x01;
pub const STATUS_KEEP: u8 = 0x02;
pub const STATUS_END: u8 = 0x03;
pub const STATUS_KEEPALIVE: u8 = 0x04;

pub const OPT_DATA: u8 = 0x01;
pub const OPT_ERROR: u8 = 0x02;

/// Application data chunk size written per Keep frame (xray
/// `SplitSize(mb, 8*1024)` for stream transfers).
pub const CHUNK_SIZE: usize = 8 * 1024;

/// Server-side `meta_len` rejection cap (`FrameMetadata.Unmarshal`), used
/// to bound the metadata read on the client too.
pub const MAX_META: usize = 512;

/// Network byte for New frames. Always TCP in this codec — `MuxTarget` is
/// TCP-only (see the module doc).
const NETWORK_TCP: u8 = 0x01;

/// A mux session target (the New frame's `network + port-first addr`).
/// TCP-only: UDP mux is out of scope (a later plan).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MuxTarget {
    /// IP target.
    Tcp(SocketAddr),
    /// Domain target: name + port.
    TcpDomain(String, u16),
}

/// One decoded v1.mux.cool frame.
#[derive(Debug)]
pub struct Frame {
    pub session_id: u16,
    pub status: u8,
    pub option: u8,
    /// `Some` on New frames (the client's first frame per session); the
    /// server's response writer starts at Keep, so `None` otherwise.
    pub target: Option<MuxTarget>,
    /// Empty unless the Data option is set.
    pub payload: Bytes,
}

/// Reads one frame. `Ok(None)` on a clean EOF at a frame boundary; a
/// truncated length/metadata/payload at EOF is `UnexpectedEof`, a
/// `meta_len` over [`MAX_META`] or an unparseable/unsupported New target
/// is `InvalidData`.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Frame>> {
    let mut len_buf = [0u8; 2];
    // Byte-by-byte so a clean EOF (0 bytes) is distinguishable from a
    // truncated length (1 byte then EOF).
    let mut filled = 0;
    while filled < len_buf.len() {
        match r.read(&mut len_buf[filled..]).await {
            Ok(0) if filled == 0 => return Ok(None),
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "vless mux frame truncated (partial meta length)",
                ));
            }
            Ok(n) => filled += n,
            Err(e) => return Err(e),
        }
    }
    let meta_len = usize::from(u16::from_be_bytes(len_buf));
    if meta_len > MAX_META {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("vless mux invalid metalen {meta_len}"),
        ));
    }
    // The allocation is capped at meta_len (≤ MAX_META) by the check above.
    let mut meta = vec![0u8; meta_len];
    r.read_exact(&mut meta).await?;

    let mut frame = parse_meta(&meta)?;
    if frame.option & OPT_DATA != 0 {
        let mut dlen = [0u8; 2];
        r.read_exact(&mut dlen).await?;
        // Cap the payload allocation to data_len (defensive — the
        // server's frames are ≤ 8 KiB, so 64 KiB is the absolute worst
        // case; a truncated payload then fails the read_exact).
        let n = usize::from(u16::from_be_bytes(dlen));
        let mut payload = vec![0u8; n];
        r.read_exact(&mut payload).await?;
        frame.payload = Bytes::from(payload);
    }
    Ok(Some(frame))
}

/// Parses the metadata body: `[sid 2B][status 1B][option 1B]`, plus
/// `[network 1B][port-first addr]` for New frames.
fn parse_meta(meta: &[u8]) -> io::Result<Frame> {
    if meta.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "vless mux metadata too short",
        ));
    }
    let session_id = u16::from_be_bytes([meta[0], meta[1]]);
    let status = meta[2];
    let option = meta[3];
    let mut rest = &meta[4..];
    let mut target = None;
    if status == STATUS_NEW {
        let (network, tail) = rest.split_first().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "vless mux new frame missing network byte",
            )
        })?;
        if *network != NETWORK_TCP {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("vless mux unsupported network type {network}"),
            ));
        }
        // port-first: [2B port][atyp][addr] — same wire layout as the
        // VLESS request header (addr.rs `decode_addr`).
        let (addr, tail) = decode_addr(tail).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "vless mux truncated target address",
            )
        })?;
        target = Some(match addr.host {
            Host::Ip(ip) => MuxTarget::Tcp(SocketAddr::new(ip, addr.port)),
            Host::Domain(domain) => MuxTarget::TcpDomain(domain, addr.port),
        });
        rest = tail;
    }
    // The metadata is exactly what we parsed; anything left over is a
    // frame format we don't understand (e.g. source/local metadata).
    if !rest.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "vless mux trailing metadata bytes",
        ));
    }
    Ok(Frame {
        session_id,
        status,
        option,
        target,
        payload: Bytes::new(),
    })
}

/// Writes one frame in a single `write_all`: `[2B meta_len][metadata]`
/// plus `[2B data_len][payload]` when the Data option is set. The
/// metadata (sid, status, option, New target) is what `meta_len` covers;
/// `writeMetaWithFrame` appends the data after it.
pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, f: &Frame) -> io::Result<()> {
    // sid(2) + status(1) + opt(1) + [net(1) + port(2) + atyp(1) + IPv6(16)]
    let mut meta = Vec::with_capacity(2 + 1 + 1 + 1 + 2 + 1 + 16);
    meta.extend_from_slice(&f.session_id.to_be_bytes());
    meta.push(f.status);
    meta.push(f.option);
    if f.status == STATUS_NEW {
        meta.push(NETWORK_TCP);
        let target = f.target.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vless mux new frame requires a target",
            )
        })?;
        meta.extend_from_slice(&encode_new_target(target));
    }
    let meta_len = u16::try_from(meta.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "vless mux metadata exceeds the 2-byte meta length",
        )
    })?;
    if usize::from(meta_len) > MAX_META {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("vless mux metadata exceeds the {MAX_META}-byte limit"),
        ));
    }

    let mut out = Vec::with_capacity(2 + meta.len() + 2 + f.payload.len());
    out.extend_from_slice(&meta_len.to_be_bytes());
    out.extend_from_slice(&meta);
    if f.option & OPT_DATA != 0 {
        let data_len = u16::try_from(f.payload.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vless mux frame payload exceeds the 2-byte data length (65535)",
            )
        })?;
        out.extend_from_slice(&data_len.to_be_bytes());
        out.extend_from_slice(&f.payload);
    } else if !f.payload.is_empty() {
        // Without the Data option the reader never consumes a data_len —
        // the bytes would be misparsed as the next frame's meta_len.
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "vless mux frame with payload requires the data option",
        ));
    }
    w.write_all(&out).await
}

/// Encodes a New-frame target as the port-first address bytes:
/// `[2B port][atyp][addr]` (IPv4 4 / Domain 1+len / IPv6 16). The network
/// byte (`NETWORK_TCP`) is written separately by [`write_frame`].
#[must_use]
pub fn encode_new_target(t: &MuxTarget) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 1 + 16);
    match t {
        MuxTarget::Tcp(sa) => {
            out.extend_from_slice(&sa.port().to_be_bytes());
            match sa.ip() {
                IpAddr::V4(v4) => {
                    out.push(ADDR_TYPE_IPV4);
                    out.extend_from_slice(&v4.octets());
                }
                IpAddr::V6(v6) => {
                    out.push(ADDR_TYPE_IPV6);
                    out.extend_from_slice(&v6.octets());
                }
            }
        }
        MuxTarget::TcpDomain(domain, port) => {
            out.extend_from_slice(&port.to_be_bytes());
            out.push(ADDR_TYPE_DOMAIN);
            // The wire address caps domain length at 255; the VLESS header
            // encode rejects longer domains before a mux target is ever
            // built (addr.rs `encode_addr` → Config error).
            let len =
                u8::try_from(domain.len()).expect("mux target domain fits in one length byte");
            out.push(len);
            out.extend_from_slice(domain.as_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use bytes::Bytes;
    use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

    use super::*;

    /// Feeds a byte buffer in fixed-size pieces so the codec must
    /// reassemble frames across partial reads (no single read returns a
    /// whole field). Deterministic and non-blocking.
    struct PieceReader {
        data: Vec<u8>,
        pos: usize,
        piece: usize,
    }

    impl AsyncRead for PieceReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let n = self
                .piece
                .min(self.data.len() - self.pos)
                .min(buf.remaining());
            buf.put_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn new_frame_exact_bytes() {
        // Frame { session_id: 1, status: New, option: 0,
        //         target: Some(Tcp(127.0.0.1:8080)), payload: [] }
        // wire: [0x00,0x0C][0x00,0x01][0x01][0x00][0x01][0x1F,0x90][0x01][127,0,0,1]
        //   meta_len = 0x000C = 12 (2B sid + 1B status + 1B opt + 1B net
        //   + 2B port + 1B atyp + 4B addr)
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: Some(MuxTarget::Tcp("127.0.0.1:8080".parse().unwrap())),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x0C, 0x00, 0x01, 0x01, 0x00, 0x01, 0x1F, 0x90, 0x01, 127, 0, 0, 1
            ]
        );
    }

    #[tokio::test]
    async fn keep_data_frame_roundtrip() {
        // Keep frame with payload b"hello":
        // [meta_len 0x0004][sid 7][0x02][0x01][2B data_len 5][payload]
        let frame = Frame {
            session_id: 7,
            status: STATUS_KEEP,
            option: OPT_DATA,
            target: None,
            payload: Bytes::from_static(b"hello"),
        };
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(&mut a, &frame).await.unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o'
            ]
        );
        // and it reads back to the same frame
        let mut c = std::io::Cursor::new(raw);
        let got = read_frame(&mut c).await.unwrap().unwrap();
        assert_eq!(got.session_id, frame.session_id);
        assert_eq!(got.status, frame.status);
        assert_eq!(got.option, frame.option);
        assert_eq!(got.target, frame.target);
        assert_eq!(&got.payload[..], &frame.payload[..]);
    }

    #[tokio::test]
    async fn end_frame_meta_only() {
        // status End, no data: [meta_len 0x0004][sid 3][0x03][0x00]
        // — no data_len on the wire (xray Close writes meta only).
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 3,
                status: STATUS_END,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x03, 0x03, 0x00]);
    }

    #[test]
    fn domain_target_encode() {
        // port-first: [port 0x01BB][atyp 0x02][len 0x0B][b"example.com"]
        let bytes = encode_new_target(&MuxTarget::TcpDomain("example.com".into(), 443));
        let mut expected = vec![0x01, 0xBB, 0x02, 0x0B];
        expected.extend_from_slice(b"example.com");
        assert_eq!(bytes, expected);
    }

    #[tokio::test]
    async fn split_frame_reads() {
        // One Keep+Data frame fed two bytes at a time must reassemble.
        let wire = [
            0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o',
        ];
        let mut r = PieceReader {
            data: wire.to_vec(),
            pos: 0,
            piece: 2,
        };
        let frame = read_frame(&mut r).await.unwrap().unwrap();
        assert_eq!(frame.session_id, 7);
        assert_eq!(frame.status, STATUS_KEEP);
        assert_eq!(frame.option, OPT_DATA);
        assert_eq!(frame.target, None);
        assert_eq!(&frame.payload[..], b"hello");
        // clean EOF after the frame
        assert!(read_frame(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn eof_at_boundary_none() {
        let mut r = PieceReader {
            data: vec![],
            pos: 0,
            piece: 2,
        };
        assert!(read_frame(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn oversized_meta_is_error() {
        // meta_len 0x0201 = 513 > 512 cap → InvalidData before any meta
        // body is read (mirrors xray's "invalid metalen").
        let mut r = PieceReader {
            data: vec![0x02, 0x01],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("metalen"));
    }

    #[tokio::test]
    async fn error_option_frame() {
        // option Error, status Keep, no data:
        // [meta_len 0x0004][sid 5][0x02][0x02]
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 5,
                status: STATUS_KEEP,
                option: OPT_ERROR,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x05, 0x02, 0x02]);
    }

    #[tokio::test]
    async fn new_frame_with_first_data_chunk() {
        // The client's first write is New + Data (xray writeData with
        // getNextFrameMeta New): target metadata, then [2B data_len][data].
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 9,
                status: STATUS_NEW,
                option: OPT_DATA,
                target: Some(MuxTarget::Tcp("192.0.2.1:443".parse().unwrap())),
                payload: Bytes::from_static(b"GET /"),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x0C, 0x00, 0x09, 0x01, 0x01, 0x01, 0x01, 0xBB, 0x01, 192, 0, 2, 1, 0x00,
                0x05, b'G', b'E', b'T', b' ', b'/'
            ]
        );
        // reads back to the same frame
        let mut c = std::io::Cursor::new(raw);
        let got = read_frame(&mut c).await.unwrap().unwrap();
        assert_eq!(got.session_id, 9);
        assert_eq!(got.status, STATUS_NEW);
        assert_eq!(got.option, OPT_DATA);
        assert_eq!(
            got.target,
            Some(MuxTarget::Tcp("192.0.2.1:443".parse().unwrap()))
        );
        assert_eq!(&got.payload[..], b"GET /");
    }

    #[tokio::test]
    async fn domain_target_full_frame() {
        // New + Domain: meta_len = 2+1+1+1+2+1+1+11 = 20 = 0x0014.
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 2,
                status: STATUS_NEW,
                option: 0,
                target: Some(MuxTarget::TcpDomain("example.com".into(), 443)),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        let mut expected = vec![
            0x00, 0x14, 0x00, 0x02, 0x01, 0x00, 0x01, 0x01, 0xBB, 0x02, 0x0B,
        ];
        expected.extend_from_slice(b"example.com");
        assert_eq!(raw, expected);
    }

    #[tokio::test]
    async fn ipv6_target_roundtrip() {
        let target = MuxTarget::Tcp("[2001:db8::1]:443".parse().unwrap());
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: Some(target.clone()),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let got = read_frame(&mut b).await.unwrap().unwrap();
        assert_eq!(got.session_id, 1);
        assert_eq!(got.status, STATUS_NEW);
        assert_eq!(got.target, Some(target));
        assert!(got.payload.is_empty());
    }

    #[tokio::test]
    async fn keepalive_frame_meta_only() {
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 0,
                status: STATUS_KEEPALIVE,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x00, 0x04, 0x00]);
    }

    #[tokio::test]
    async fn partial_length_eof_is_error() {
        // One length byte then EOF → UnexpectedEof (a clean boundary is 0).
        let mut r = PieceReader {
            data: vec![0x00],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn truncated_metadata_is_error() {
        // meta_len 10 but only 3 bytes follow.
        let mut r = PieceReader {
            data: vec![0x00, 0x0A, 0x00, 0x07, 0x02],
            pos: 0,
            piece: 3,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn truncated_payload_is_error() {
        // meta fine (Keep+Data), data_len 5 but only 2 payload bytes.
        let mut r = PieceReader {
            data: vec![0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'i'],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn unknown_network_is_error() {
        // New frame with network byte 0x02 (UDP): the codec is TCP-only
        // (mirrors xray's "unknown network type" — the target cannot be
        // represented, so the whole connection is unrecoverable).
        let mut r = PieceReader {
            data: vec![
                0x00, 0x0C, 0x00, 0x01, 0x01, 0x00, 0x02, 0x1F, 0x90, 0x01, 127, 0, 0, 1,
            ],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("network"));
    }

    #[tokio::test]
    async fn new_frame_without_target_is_invalid() {
        // A New frame must carry a target on the wire — writing one
        // without it is a client bug (InvalidInput), nothing written.
        let (mut a, mut b) = tokio::io::duplex(64);
        let err = write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert!(raw.is_empty());
    }

    #[tokio::test]
    async fn payload_without_data_option_is_invalid() {
        // A payload needs the Data option — otherwise the reader would
        // misparse it as the next frame's meta_len.
        let (mut a, mut b) = tokio::io::duplex(64);
        let err = write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_KEEP,
                option: 0,
                target: None,
                payload: Bytes::from_static(b"x"),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert!(raw.is_empty());
    }
}
