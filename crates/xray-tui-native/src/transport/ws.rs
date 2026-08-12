//! WebSocket transport: RFC 6455 framing (tokio-tungstenite) over an
//! established stream — the engine TLS/REALITY session or raw TCP. The
//! transport never owns TLS; it consumes `AsyncRead + AsyncWrite`.

use std::io;
use std::pin::Pin;
use std::task::{Context as TaskCx, Poll};

use bytes::BytesMut;
use futures_core::Stream;
use futures_sink::Sink;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{CONNECTION, HOST, UPGRADE};
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue, Request, StatusCode};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use xray_tui_proto::proto_spec::common::WebSocketConfig;

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::BoxStream;

/// Build the WS upgrade request. Pure, unit-testable.
pub fn ws_request(cfg: &WebSocketConfig, server_host: &str) -> Result<Request<()>, NativeError> {
    let path = cfg.path.as_deref().unwrap_or("/");
    let host = cfg.host.as_deref().unwrap_or(server_host);
    let mut req = format!("ws://{host}{path}")
        .into_client_request()
        .map_err(|e| NativeError::Transport(format!("ws request: {e}")))?;
    req.headers_mut().insert(
        HOST,
        HeaderValue::from_str(host).map_err(|e| NativeError::Transport(format!("ws host: {e}")))?,
    );
    if let Some(headers) = &cfg.headers {
        for (k, v) in headers {
            let name = http_name(k);
            if let Ok(vv) = HeaderValue::from_str(v) {
                req.headers_mut().append(name, vv);
            }
        }
    }
    req.headers_mut()
        .insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    req.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("websocket"));
    Ok(req)
}

fn http_name(k: &str) -> HeaderName {
    HeaderName::from_bytes(k.as_bytes())
        .unwrap_or_else(|_| HeaderName::from_static("x-v2ray-unknown"))
}

/// Run the WS upgrade over the established (secured) stream.
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let cfg = ctx.transport_ws().ok_or_else(|| {
        NativeError::Config("ws transport requested but config has no ws settings".into())
    })?;
    let server_host = ctx.params.server.host.as_str();
    let req = ws_request(cfg, server_host)?;
    let handshake = tokio_tungstenite::client_async(req, stream);
    let (ws, resp) = tokio::time::timeout(timeouts::TRANSPORT, handshake)
        .await
        .map_err(|_| NativeError::Timeout {
            step: "ws upgrade",
            limit: timeouts::TRANSPORT,
        })?
        .map_err(|e| NativeError::Transport(format!("ws handshake: {e}")))?;
    if resp.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(NativeError::Transport(format!(
            "ws upgrade rejected: {}",
            resp.status()
        )));
    }
    Ok(Box::new(WsStream::new(ws)))
}

/// Binary-message byte stream: writes buffer into Binary frames (flushed on
/// `poll_flush`); reads drain Binary payloads (pong auto-replied by
/// tungstenite); Close/EOF surfaces as `UnexpectedEof`.
pub struct WsStream {
    inner: WebSocketStream<BoxStream>,
    read_buf: BytesMut,
}

impl WsStream {
    #[must_use]
    pub fn new(inner: WebSocketStream<BoxStream>) -> Self {
        Self {
            inner,
            read_buf: BytesMut::new(),
        }
    }
}

impl AsyncRead for WsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            if !self.read_buf.is_empty() {
                let n = std::cmp::min(self.read_buf.len(), buf.remaining());
                buf.put_slice(&self.read_buf.split_to(n));
                return Poll::Ready(Ok(()));
            }
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Binary(b)))) => {
                    self.read_buf.extend_from_slice(&b);
                }
                Poll::Ready(Some(Ok(Message::Close(_))) | None) => {
                    // Clean WS close: report EOF so `read_to_end` completes.
                    return Poll::Ready(Ok(()));
                }
                // Ping (pong auto-sent by tungstenite), Pong, Text, Frame: ignored.
                Poll::Ready(Some(Ok(_))) => {}
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::other(e)));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for WsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Write-through (leaf's pattern): one Binary frame per write, best
        // effort flush so flush-less consumers (`write_all`-only) deliver.
        match Pin::new(&mut self.inner).poll_ready(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(io::Error::other(e))),
            Poll::Ready(Ok(())) => {}
        }
        let msg = Message::Binary(buf.to_vec().into());
        if let Err(e) = Pin::new(&mut self.inner).start_send(msg) {
            return Poll::Ready(Err(io::Error::other(e)));
        }
        let _ = Pin::new(&mut self.inner).poll_flush(cx);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner)
            .poll_flush(cx)
            .map_err(io::Error::other)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner)
            .poll_close(cx)
            .map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_proto::proto_spec::common::WebSocketConfig;

    #[test]
    fn request_headers_include_host_path_and_custom() {
        let cfg = WebSocketConfig {
            path: Some("/ws?ed=2048".into()),
            host: Some("cdn.example.com".into()),
            headers: Some([("X-Custom".to_string(), "v1".to_string())].into()),
            ..Default::default()
        };
        let req = ws_request(&cfg, "real-server.example").unwrap();
        assert_eq!(
            req.uri().path_and_query().unwrap().as_str(),
            "/ws?ed=2048"
        );
        assert_eq!(req.headers().get("host").unwrap(), "cdn.example.com");
        assert_eq!(req.headers().get("x-custom").unwrap(), "v1");
        assert_eq!(req.headers().get("connection").unwrap(), "Upgrade");
        assert_eq!(req.headers().get("upgrade").unwrap(), "websocket");
    }

    #[test]
    fn host_defaults_to_server_host() {
        let cfg = WebSocketConfig {
            path: Some("/".into()),
            ..Default::default()
        };
        let req = ws_request(&cfg, "proxy.example.net").unwrap();
        assert_eq!(req.headers().get("host").unwrap(), "proxy.example.net");
        assert_eq!(req.uri().path(), "/");
    }
}
