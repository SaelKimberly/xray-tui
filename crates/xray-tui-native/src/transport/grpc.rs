//! gRPC transport: HTTP/2 framing (h2 crate) over the engine stream.
//! Single bidirectional stream (gun mode); 5-byte gRPC prefix framing per
//! the xray-core/sing-box wire format.

use std::io;
use std::pin::Pin;
use std::task::{Context as TaskCx, Poll};

use bytes::{Bytes, BytesMut};
use h2::SendStream;
use h2::RecvStream;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use xray_tui_proto::proto_spec::common::GrpcConfig;

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::BoxStream;

/// gRPC service name: config value verbatim (no default). Stream path is
/// `/{service}/Tun`; empty service yields `//Tun` (xray `getServiceName`,
/// sing-box `TunCustomName`).
#[must_use]
pub fn grpc_service_name(cfg: &GrpcConfig) -> String {
    cfg.service_name.as_deref().unwrap_or("").to_string()
}

/// Request path: explicit config path, else `/{service}/Tun` (empty service
/// → `//Tun`, matching both cores).
fn grpc_path(cfg: &GrpcConfig, service: &str) -> String {
    cfg.path
        .as_deref()
        .filter(|p| !p.is_empty())
        .map_or_else(|| format!("/{service}/Tun"), ToString::to_string)
}

/// The xray/sing-box grpc transport carries VLESS bytes inside a protobuf
/// `Hunk` message (`message Hunk { bytes data = 1; }`), which rides inside
/// the standard gRPC 5-byte framing. Wire layout per message:
/// `0x00 | BE32(hunk_len) | 0x0A | varint(data_len) | data`.
/// (Verified: xray-core's `HunkConn.Recv` unmarshals the payload as Hunk —
/// raw bytes fail with "cannot parse invalid wire-format data".
fn varint_len(mut n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(2);
    loop {
        let mut b = u8::try_from(n & 0x7f).unwrap_or(0x7f);
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        out.push(b);
        if n == 0 {
            return out;
        }
    }
}

/// Encode one VLESS byte chunk as a gRPC Hunk message. Pure, unit-tested.
#[must_use]
pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let hunk = {
        let mut h = Vec::with_capacity(1 + 2 + payload.len());
        h.push(0x0A); // field 1, wire type 2 (length-delimited)
        h.extend_from_slice(&varint_len(payload.len()));
        h.extend_from_slice(payload);
        h
    };
    let mut out = Vec::with_capacity(5 + hunk.len());
    out.push(0);
    out.extend_from_slice(&u32::try_from(hunk.len()).unwrap_or(u32::MAX).to_be_bytes());
    out.extend_from_slice(&hunk);
    out
}

/// Decode ONE gRPC Hunk message from the front of `buf`, consuming only
/// complete messages. Returns `None` when fewer than a full message is
/// available (partial prefix/payload stays in `buf`).
pub fn decode_frame(buf: &mut BytesMut) -> Option<Vec<u8>> {
    if buf.len() < 5 {
        return None;
    }
    let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
    if buf.len() < 5 + len {
        return None;
    }
    let msg = buf.split_to(5 + len);
    let hunk = &msg[5..];
    // Hunk: 0x0A tag, then varint data length, then data.
    let tag = *hunk.first()?;
    if tag != 0x0A {
        return None;
    }
    let (dlen, dstart) = varint_decode(hunk, 1)?;
    if 1 + dstart + dlen > hunk.len() {
        return None;
    }
    Some(hunk[1 + dstart..1 + dstart + dlen].to_vec())
}

/// Decode a base-128 varint starting at `start`; returns (value, bytes).
fn varint_decode(buf: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut val = 0usize;
    let mut shift = 0;
    for (i, &b) in buf.iter().enumerate().skip(start) {
        val |= usize::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Some((val, i + 1 - start));
        }
        shift += 7;
        if shift >= usize::BITS {
            return None;
        }
    }
    None
}

/// Run the gRPC transport: h2 handshake over the (secured) stream, open the
/// single `/{service}/Tun` stream, return the bidirectional byte stream.
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let cfg = ctx.transport_grpc().ok_or_else(|| {
        NativeError::Config("grpc transport requested but config has no grpc settings".into())
    })?;
    let service = grpc_service_name(cfg);
    let path = grpc_path(cfg, &service);
    let authority = cfg
        .authority
        .as_deref()
        .filter(|a| !a.is_empty())
        .map_or_else(|| ctx.params.server.host.clone(), ToString::to_string);

    let handshake = h2::client::handshake(stream);
    let (mut send_req, conn) = tokio::time::timeout(timeouts::TRANSPORT, handshake)
        .await
        .map_err(|_| NativeError::Timeout {
            step: "grpc h2 handshake",
            limit: timeouts::TRANSPORT,
        })?
        .map_err(|e| NativeError::Transport(format!("grpc h2 handshake: {e}")))?;
    // Drive the h2 connection in the background for the stream's lifetime.
    tokio::spawn(async move {
        let _ = conn.await;
    });

    let req = http::Request::builder()
        .method("POST")
        .uri(format!("https://{authority}{path}"))
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .header("grpc-accept-encoding", "identity")
        .header(
            "user-agent",
            cfg.user_agent
                .as_deref()
                .filter(|ua| !ua.is_empty())
                .unwrap_or("grpc-go/1.64.0"),
        )
        .body(())
        .map_err(|e| NativeError::Transport(format!("grpc request: {e}")))?;
    let (resp, send_stream) = send_req
        .send_request(req, false)
        .map_err(|e| NativeError::Transport(format!("grpc stream open: {e}")))?;
    // Do NOT block on the response headers here: xray's grpc `Tun` handler
    // defers its 200 until the first client DATA message (verified with
    // curl — no body hangs, body → 200). The protocol writes the VLESS
    // request as the first message; a background task hands the `RecvStream`
    // over once the server responds.
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let out = match tokio::time::timeout(timeouts::TRANSPORT, resp).await {
            Ok(Ok(r)) if r.status() == http::StatusCode::OK => Ok(r.into_body()),
            Ok(Ok(r)) => Err(NativeError::Transport(format!(
                "grpc stream rejected: {}",
                r.status()
            ))),
            Ok(Err(e)) => Err(NativeError::Transport(format!("grpc response: {e}"))),
            Err(_) => Err(NativeError::Timeout {
                step: "grpc response headers",
                limit: timeouts::TRANSPORT,
            }),
        };
        let _ = tx.send(out);
    });
    Ok(Box::new(GrpcStream::new(send_stream, rx)))
}

/// Single bidirectional gRPC stream: `AsyncWrite` prefixes + sends DATA
/// frames; `AsyncRead` parses the 5-byte prefix and yields payloads.
pub struct GrpcStream {
    send: SendStream<Bytes>,
    recv: Option<RecvStream>,
    response: Option<tokio::sync::oneshot::Receiver<Result<RecvStream, NativeError>>>,
    read_buf: BytesMut,
    payload: BytesMut,
}

impl GrpcStream {
    /// `response` resolves once the server replies to the first message;
    /// the read side starts pulling from it (the write side works at once).
    #[must_use]
    pub fn new(
        send: SendStream<Bytes>,
        response: tokio::sync::oneshot::Receiver<Result<RecvStream, NativeError>>,
    ) -> Self {
        Self {
            send,
            recv: None,
            response: Some(response),
            read_buf: BytesMut::new(),
            payload: BytesMut::new(),
        }
    }
}

impl AsyncRead for GrpcStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            if !self.payload.is_empty() {
                let n = std::cmp::min(self.payload.len(), buf.remaining());
                buf.put_slice(&self.payload.split_to(n));
                return Poll::Ready(Ok(()));
            }
            if let Some(msg) = decode_frame(&mut self.read_buf) {
                self.payload.extend_from_slice(&msg);
                continue;
            }
            if self.recv.is_none() {
                let rx = self.response.as_mut().expect("response channel present while recv unset");
                match Pin::new(rx).poll(cx) {
                    Poll::Ready(Ok(Ok(recv))) => {
                        self.recv = Some(recv);
                        self.response = None;
                        continue;
                    }
                    Poll::Ready(Ok(Err(e))) => {
                        return Poll::Ready(Err(io::Error::other(e)));
                    }
                    Poll::Ready(Err(_)) => {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "grpc response task dropped",
                        )));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
            let recv = self.recv.as_mut().expect("recv set above");
            match Pin::new(recv).poll_data(cx) {
                Poll::Ready(Some(Ok(chunk))) => self.read_buf.extend_from_slice(&chunk),
                Poll::Ready(Some(Err(e))) => {
                    // A reset with NO_ERROR (grpc-go closes streams that
                    // way) is a clean end: report EOF so `read_to_end`
                    // completes. Any other reset is a real error.
                    if e.reason() == Some(h2::Reason::NO_ERROR) {
                        return Poll::Ready(Ok(()));
                    }
                    return Poll::Ready(Err(io::Error::other(e)));
                }
                Poll::Ready(None) => {
                    // Clean end-of-stream (trailers): report EOF so
                    // `read_to_end` completes instead of erroring.
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for GrpcStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Write-through: one gRPC message per write, chunked at 16 KiB so a
        // single message never exceeds the h2 flow window. `reserve` +
        // `poll_capacity` before consuming keeps the AsyncWrite contract
        // (Pending = nothing consumed).
        let take = buf.len().min(16384);
        let framed = encode_frame(&buf[..take]);
        self.send.reserve_capacity(framed.len());
        match Pin::new(&mut self.send).poll_capacity(cx) {
            Poll::Ready(Some(Ok(_))) => {}
            Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(io::Error::other(e))),
            Poll::Ready(None) => {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "grpc send stream closed",
                )));
            }
            Poll::Pending => return Poll::Pending,
        }
        if let Err(e) = self.send.send_data(Bytes::from(framed), false) {
            return Poll::Ready(Err(io::Error::other(e)));
        }
        Poll::Ready(Ok(take))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        match Pin::new(&mut self.send).poll_capacity(cx) {
            Poll::Ready(Some(Ok(_))) => Poll::Ready(Ok(())),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Err(io::Error::other(e))),
            Poll::Ready(None) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "grpc send stream closed",
            ))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        // End-of-stream: empty DATA frame with END_STREAM (SendStream has no
        // poll_close; send_data with eos=true is the shutdown signal).
        match self.send.send_data(Bytes::new(), true) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(io::Error::other(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_proto::proto_spec::common::GrpcConfig;

    #[test]
    fn frame_roundtrip() {
        let payload = b"hello vless";
        let framed = encode_frame(payload);
        // gRPC prefix (flag 0 + BE hunk len 13) + Hunk protobuf wrapper.
        assert_eq!(&framed[..5], &[0, 0, 0, 0, 13]);
        assert_eq!(&framed[5..7], &[0x0A, 11]);
        assert_eq!(&framed[7..], payload);
        let mut buf = BytesMut::from(&framed[..]);
        assert_eq!(decode_frame(&mut buf), Some(payload.to_vec()));
        assert!(buf.is_empty());
    }

    #[test]
    fn frame_hunk_wraps_long_payloads_with_varint() {
        let payload = vec![0x42u8; 200]; // varint length needs 2 bytes
        let framed = encode_frame(&payload);
        assert_eq!(&framed[..5], &[0, 0, 0, 0, 1 + 2 + 200]);
        assert_eq!(&framed[5..8], &[0x0A, 0xC8, 0x01]);
        let mut buf = BytesMut::from(&framed[..]);
        assert_eq!(decode_frame(&mut buf), Some(payload));
    }

    #[test]
    fn frame_parse_splits_across_chunks() {
        let framed = encode_frame(b"abcdef");
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&framed[..3]);
        assert_eq!(decode_frame(&mut buf), None); // partial prefix
        buf.extend_from_slice(&framed[3..]);
        assert_eq!(decode_frame(&mut buf), Some(b"abcdef".to_vec()));
    }

    #[test]
    fn frame_parse_handles_multiple_messages() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&encode_frame(b"one"));
        buf.extend_from_slice(&encode_frame(b"two"));
        assert_eq!(decode_frame(&mut buf), Some(b"one".to_vec()));
        assert_eq!(decode_frame(&mut buf), Some(b"two".to_vec()));
        assert!(buf.is_empty());
    }

    #[test]
    fn service_name_is_verbatim_no_default() {
        assert_eq!(grpc_service_name(&GrpcConfig::default()), "");
        let cfg = GrpcConfig {
            service_name: Some("gun".into()),
            ..Default::default()
        };
        assert_eq!(grpc_service_name(&cfg), "gun");
    }

    #[test]
    fn path_builds_slash_service_tun() {
        // Empty service → "//Tun", matching xray + sing-box path derivation.
        assert_eq!(grpc_path(&GrpcConfig::default(), ""), "//Tun");
        assert_eq!(grpc_path(&GrpcConfig::default(), "gun"), "/gun/Tun");
        let cfg = GrpcConfig {
            path: Some("/custom/path".into()),
            ..Default::default()
        };
        assert_eq!(grpc_path(&cfg, "gun"), "/custom/path");
    }
}
