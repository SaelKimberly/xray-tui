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

/// The gRPC service name: explicit config value, else `"gun"` (the xray-core
/// default; sing-box maps an empty `service_name` to `"gun"` too).
#[must_use]
pub fn grpc_service_name(cfg: &GrpcConfig) -> String {
    cfg.service_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("gun")
        .to_string()
}

/// Request path: explicit config path, else `/<service>/Tun`.
fn grpc_path(cfg: &GrpcConfig, service: &str) -> String {
    cfg.path
        .as_deref()
        .filter(|p| !p.is_empty())
        .map_or_else(|| format!("/{service}/Tun"), ToString::to_string)
}

/// gRPC message framing: `0x00` flag + 4-byte big-endian payload length +
/// payload. Pure, unit-tested.
#[must_use]
pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(0);
    out.extend_from_slice(&u32::try_from(payload.len()).unwrap_or(u32::MAX).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

/// Decode ONE gRPC message from the front of `buf`, consuming only complete
/// messages. Returns `None` when fewer than a full message is available
/// (partial prefix or payload stays in `buf`).
pub fn decode_frame(buf: &mut BytesMut) -> Option<Vec<u8>> {
    if buf.len() < 5 {
        return None;
    }
    let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
    if buf.len() < 5 + len {
        return None;
    }
    let msg = buf.split_to(5 + len);
    Some(msg[5..].to_vec())
}

/// Run the gRPC transport: h2 handshake over the (secured) stream, open the
/// single `/<service>/Tun` stream, return the bidirectional byte stream.
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
    let resp = tokio::time::timeout(timeouts::TRANSPORT, resp)
        .await
        .map_err(|_| NativeError::Timeout {
            step: "grpc response headers",
            limit: timeouts::TRANSPORT,
        })?
        .map_err(|e| NativeError::Transport(format!("grpc response: {e}")))?;
    if resp.status() != http::StatusCode::OK {
        return Err(NativeError::Transport(format!(
            "grpc stream rejected: {}",
            resp.status()
        )));
    }
    let recv_stream = resp.into_body();
    Ok(Box::new(GrpcStream::new(send_stream, recv_stream)))
}

/// Single bidirectional gRPC stream: `AsyncWrite` prefixes + sends DATA
/// frames; `AsyncRead` parses the 5-byte prefix and yields payloads.
pub struct GrpcStream {
    send: SendStream<Bytes>,
    recv: RecvStream,
    write_buf: Vec<u8>,
    read_buf: BytesMut,
    payload: BytesMut,
}

impl GrpcStream {
    #[must_use]
    pub fn new(send: SendStream<Bytes>, recv: RecvStream) -> Self {
        Self {
            send,
            recv,
            write_buf: Vec::new(),
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
            match Pin::new(&mut self.recv).poll_data(cx) {
                Poll::Ready(Some(Ok(chunk))) => self.read_buf.extend_from_slice(&chunk),
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::other(e)));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "grpc stream closed",
                    )));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for GrpcStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut TaskCx<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.write_buf.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        if !self.write_buf.is_empty() {
            let framed = encode_frame(&std::mem::take(&mut self.write_buf));
            if let Err(e) = self.send.send_data(Bytes::from(framed), false) {
                return Poll::Ready(Err(io::Error::other(e)));
            }
        }
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
        assert_eq!(&framed[..5], &[0, 0, 0, 0, 11]); // flag 0 + BE len 11
        assert_eq!(&framed[5..], payload);
        let mut buf = BytesMut::from(&framed[..]);
        assert_eq!(decode_frame(&mut buf), Some(payload.to_vec()));
        assert!(buf.is_empty());
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
    fn service_name_defaults_to_gun() {
        assert_eq!(grpc_service_name(&GrpcConfig::default()), "gun");
        let cfg = GrpcConfig {
            service_name: Some("custom".into()),
            ..Default::default()
        };
        assert_eq!(grpc_service_name(&cfg), "custom");
    }

    #[test]
    fn path_defaults_to_service_tun() {
        assert_eq!(grpc_path(&GrpcConfig::default(), "gun"), "/gun/Tun");
        let cfg = GrpcConfig {
            path: Some("/custom/path".into()),
            ..Default::default()
        };
        assert_eq!(grpc_path(&cfg, "gun"), "/custom/path");
    }
}
