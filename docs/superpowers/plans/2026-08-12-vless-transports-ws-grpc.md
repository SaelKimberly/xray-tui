# VLESS Transports WS + gRPC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement WebSocket + gRPC transports for the native VLESS client, running over the engine TLS/REALITY stream (TLS outer, transport inner), e2e-proven against real xray-core + sing-box.

**Architecture:** Per link: dial (tcp) → engine security (TLS/REALITY) → transport upgrade (ws/grpc framing) → protocol (VLESS). Transport framing crates (tokio-tungstenite, h2) receive the established `BoxStream`; they never own TLS.

**Tech Stack:** tokio-tungstenite 0.30 (WS), h2 0.4.15 (gRPC framing), bytes, http, futures-core; engine xray-tui-tls for the outer TLS; xray-tui-proto configs (`TransportConfig::Ws/Grpc`) untouched.

## Global Constraints

- ring-only in xray-tui-tls; native adds tokio-tungstenite/h2 as NON-optional deps (transport framing, not TLS).
- Clippy pedantic+nursery ZERO warnings; edition 2024; rustfmt.
- xray-tui-proto UNTOUCHED. Binary crate untouched.
- Behavior-preserving: tcp-only paths byte-identical (existing 11 e2e cases stay green).
- e2e: feature `native-e2e` + `XRAY_TUI_CORE_BIN_DIR` (hard-fail absent); cores at /tmp/core-bin (xray 26.3.27, sing-box 1.13.16).
- Every task compiles + its tests pass before commit.

---
## Task 1: Chain rework — `transport::upgrade` + fold flip

**Files:**
- Modify: `crates/xray-tui-native/src/transport/mod.rs`
- Modify: `crates/xray-tui-native/src/chain.rs:37-38`
- Test: `crates/xray-tui-native/src/chain.rs` (existing tests stay green)

**Interfaces:**
- Consumes: `LinkContext::transport_type() -> Option<&str>` (context.rs:141), `BoxStream = Box<dyn Stream>` (lib.rs:31), `NativeError::NotImplemented`
- Produces: `transport::upgrade(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError>` — ws/grpc dispatch (stubbed NotImplemented until T2/T3), tcp/None = passthrough.

- [ ] **Step 1: Add `upgrade` to transport/mod.rs**

```rust
/// Run the transport-upgrade step over an ALREADY-established stream
/// (typically the secured engine stream). TCP = passthrough; ws/grpc =
/// framing handshake over the stream (T2/T3).
pub async fn upgrade(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        Some("ws") => Err(NativeError::NotImplemented {
            feature: "ws transport".into(),
        }),
        Some("grpc") => Err(NativeError::NotImplemented {
            feature: "grpc transport".into(),
        }),
        None | Some("tcp") => Ok(stream),
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}
```

- [ ] **Step 2: Flip the fold in chain.rs**

Replace the loop body (chain.rs:36-40):
```rust
        let transported = transport::connect(&ctx, base).await?;
        let secured = security::wrap(&ctx, transported).await?;
        base = Some(protocol::connect(&ctx, secured).await?);
```
with:
```rust
        let dialed = transport::connect(&ctx, base).await?;
        let secured = security::wrap(&ctx, dialed).await?;
        let upgraded = transport::upgrade(&ctx, secured).await?;
        base = Some(protocol::connect(&ctx, upgraded).await?);
```

- [ ] **Step 3: Update the module doc comment** (chain.rs:1-10 and transport/mod.rs:1-10) to the new order: `dial → security → transport upgrade → protocol` for ws/grpc links.

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-native --lib`
Expected: PASS (chain tests + all unit tests; tcp path identical).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/transport/mod.rs crates/xray-tui-native/src/chain.rs
git commit -m "refactor(native): chain fold dial→security→transport-upgrade→protocol"
```

---
## Task 2: WebSocket transport

**Files:**
- Create: `crates/xray-tui-native/src/transport/ws.rs`
- Modify: `crates/xray-tui-native/src/transport/mod.rs` (dispatch arm), `crates/xray-tui-native/Cargo.toml` (deps)
- Test: inline `#[cfg(test)]` in ws.rs

**Interfaces:**
- Consumes: `transport::upgrade` dispatch (T1), `LinkContext` + `ctx.transport_ws() -> Option<&WebSocketConfig>` (new accessor), `WebSocketConfig { path, host, headers }` (proto common.rs:204), `timeouts::TRANSPORT` (error.rs:34), `NativeError`
- Produces: `ws::connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError>`, `WsStream` (AsyncRead+AsyncWrite wrapper), `fn ws_request_headers(cfg: &WebSocketConfig, host: &str) -> Vec<(String, String)>` (unit-testable pure fn)

- [ ] **Step 1: Add deps to native Cargo.toml**

```toml
# WebSocket transport framing (RFC 6455) over the engine stream — no TLS
tokio-tungstenite = "0.30"
futures-core = "0.3"
```

- [ ] **Step 2: Add `NativeError::Transport` variant + `Host` string helper**

error.rs (after `Reality`):
```rust
    #[error("transport error: {0}")]
    Transport(String),
```
addr.rs — add a string accessor (no Display today):
```rust
    /// Raw host string (domain or IP), no brackets for IPv6.
    #[must_use]
    pub fn as_str(&self) -> String {
        match self {
            Self::Ip(ip) => ip.to_string(),
            Self::Domain(d) => d.clone(),
        }
    }
```
and a `transport_ws` accessor + `set_alpn` test helper where needed:
```rust
    pub fn transport_ws(&self) -> Option<&WebSocketConfig> {
        match &self.params.protocol.transport_config()? {
            xray_tui_proto::proto_spec::TransportConfig::Ws(c) => Some(c),
            _ => None,
        }
    }
```
(If `transport_config()` doesn't exist, add it on ProtocolConfig — it returns `Option<&TransportConfig>` mirroring `transport_type()`.)

- [ ] **Step 3: Write the failing request-header test**

```rust
#[test]
fn request_headers_include_host_path_and_custom() {
    let cfg = WebSocketConfig {
        path: Some("/ws?ed=2048".into()),
        host: Some("cdn.example.com".into()),
        headers: Some([("X-Custom".into(), "v1".into())].into()),
        ..Default::default()
    };
    let req = ws_request(&cfg, "real-server.example").unwrap();
    assert_eq!(req.uri().path_and_query().unwrap().as_str(), "/ws?ed=2048");
    assert_eq!(req.headers().get("host").unwrap(), "cdn.example.com");
    assert_eq!(req.headers().get("x-custom").unwrap(), "v1");
    assert_eq!(req.headers().get("connection").unwrap(), "Upgrade");
    assert_eq!(req.headers().get("upgrade").unwrap(), "websocket");
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --lib ws::tests::request_headers`
Expected: FAIL (module missing).

- [ ] **Step 5: Implement ws.rs**

```rust
//! WebSocket transport: RFC 6455 framing (tokio-tungstenite) over an
//! established stream (the engine TLS/REALITY session or raw TCP).

use bytes::BytesMut;
use std::io;
use std::pin::Pin;
use std::task::{Context as TaskCx, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{
    CONNECTION, HOST, UPGRADE,
};
use tokio_tungstenite::tungstenite::http::{HeaderValue, Request, StatusCode};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use xray_tui_proto::proto_spec::common::WebSocketConfig;

use crate::error::{NativeError, timeouts};
use crate::{BoxStream, context::LinkContext};

/// Build the WS upgrade request (pure, unit-testable).
pub fn ws_request(cfg: &WebSocketConfig, server_host: &str) -> Result<Request<()>, NativeError> {
    let path = cfg.path.as_deref().unwrap_or("/");
    let host = cfg.host.as_deref().unwrap_or(server_host);
    let mut req = format!("ws://{host}{path}").into_client_request()
        .map_err(|e| NativeError::Transport(format!("ws request: {e}")))?;
    req.headers_mut().insert(HOST, HeaderValue::from_str(host)
        .map_err(|e| NativeError::Transport(format!("ws host header: {e}")))?);
    if let Some(headers) = &cfg.headers {
        for (k, v) in headers {
            if let (Ok(kv), Ok(vv)) = (HeaderValue::from_str(k), HeaderValue::from_str(v)) {
                req.headers_mut().append(k, kv);
                req.headers_mut().append(k, vv);
            }
        }
    }
    req.headers_mut().insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    req.headers_mut().insert(UPGRADE, HeaderValue::from_static("websocket"));
    Ok(req)
}

pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let cfg = ctx.transport_ws()
        .ok_or_else(|| NativeError::Config("ws transport requested but config is tcp".into()))?;
    let server_host = ctx.params.server.host.to_string();
    let req = ws_request(cfg, &server_host)?;
    let handshake = tokio_tungstenite::client_async(req, stream);
    let (ws, resp) = tokio::time::timeout(timeouts::TRANSPORT, handshake)
        .await
        .map_err(|_| NativeError::Timeout { step: "ws upgrade", limit: timeouts::TRANSPORT })?
        .map_err(|e| NativeError::Transport(format!("ws handshake: {e}")))?;
    if resp.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(NativeError::Transport(format!(
            "ws upgrade rejected: {}", resp.status()
        )));
    }
    Ok(Box::new(WsStream::new(ws)))
}

/// Binary-message byte stream: AsyncWrite buffers → Binary frames; AsyncRead
/// drains Binary payloads, auto-pongs (tungstenite read), errors on Close.
pub struct WsStream {
    inner: WebSocketStream<BoxStream>,
    read_buf: BytesMut,
    write_buf: Vec<u8>,
}

impl WsStream {
    pub fn new(inner: WebSocketStream<BoxStream>) -> Self {
        Self { inner, read_buf: BytesMut::new(), write_buf: Vec::new() }
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
                let n = self.read_buf.as_ref().read(buf).unwrap_or(0);
                self.read_buf.advance(n);
                return Poll::Ready(Ok(()));
            }
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Binary(b)))) => self.read_buf.extend_from_slice(&b),
                Poll::Ready(Some(Ok(Message::Close(_)))) | Poll::Ready(None) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "ws closed")))
                }
                Poll::Ready(Some(Ok(_))) => {} // Text/Ping/Pong — ignore (pong auto-sent)
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)))
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for WsStream {
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
            let msg = Message::Binary(std::mem::take(&mut self.write_buf));
            if let Err(e) = Pin::new(&mut self.inner).start_send(msg) {
                return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)));
            }
        }
        Pin::new(&mut self.inner).poll_flush(cx).map_err(io::Error::from)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx).map_err(io::Error::from)
    }
}
```
(Note: `WebSocketStream` implements `Stream` (futures-core) and `Sink<Message>` (futures-util). Add `futures_util = { version = "0.3", default-features = false, features = ["sink"] }` if `start_send`/`poll_close` need it — tungstenite re-exports via its own `Sink` bound; use `tokio_tungstenite::tungstenite::utilities::Sink`-compatible calls, whichever compiles.)

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p xray-tui-native --lib ws::tests`
Expected: PASS.

- [ ] **Step 7: Wire the dispatch arm** (transport/mod.rs `upgrade`)

```rust
        Some("ws") => ws::connect(ctx, stream).await,
```
and `pub mod ws;`.

- [ ] **Step 8: Compile + clippy + commit**

Run: `cargo test -p xray-tui-native --lib && cargo clippy -p xray-tui-native --all-targets` (zero warnings)
```bash
git add crates/xray-tui-native/src/transport/ws.rs crates/xray-tui-native/src/transport/mod.rs crates/xray-tui-native/src/context.rs crates/xray-tui-native/Cargo.toml Cargo.lock
git commit -m "feat(native): WebSocket transport over engine stream"
```

---
## Task 3: gRPC transport

**Files:**
- Create: `crates/xray-tui-native/src/transport/grpc.rs`
- Modify: `crates/xray-tui-native/src/transport/mod.rs` (dispatch arm), `crates/xray-tui-native/Cargo.toml`
- Test: inline `#[cfg(test)]` in grpc.rs

**Interfaces:**
- Consumes: `transport::upgrade` dispatch, `ctx.transport_grpc() -> Option<&GrpcConfig>`, `GrpcConfig { path, authority, service_name, mode, user_agent }` (proto common.rs:216), `timeouts::TRANSPORT`
- Produces: `grpc::connect(ctx, stream) -> Result<BoxStream, NativeError>`, `GrpcStream` (AsyncRead+AsyncWrite over `SendStream<Bytes>` + `RecvStream`), `fn encode_frame(payload: &[u8]) -> Vec<u8>` + `fn decode_frame(buf: &mut BytesMut) -> Option<Vec<u8>>` (pure, unit-tested), `fn grpc_service_name(cfg) -> String` (default "gun")

- [ ] **Step 1: Add deps**

```toml
# gRPC transport framing (HTTP/2, RFC 7540) over the engine stream
h2 = "0.4.15"
http = "1"
bytes = "1"
```

- [ ] **Step 2: Add `transport_grpc` accessor** (context.rs, mirror of T2 Step 2).

- [ ] **Step 3: Write the failing framing tests**

```rust
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
    assert_eq!(decode_frame(&mut buf), None); // incomplete prefix
    buf.extend_from_slice(&framed[3..]);
    assert_eq!(decode_frame(&mut buf), Some(b"abcdef".to_vec()));
}

#[test]
fn service_name_defaults_to_gun() {
    assert_eq!(grpc_service_name(&GrpcConfig::default()), "gun");
}
```

- [ ] **Step 4: Run to verify they fail**

Run: `cargo test -p xray-tui-native --lib grpc::tests`
Expected: FAIL (module missing).

- [ ] **Step 5: Implement grpc.rs**

```rust
//! gRPC transport: HTTP/2 framing (h2 crate) over an established stream.
//! One bidirectional stream (gun mode); VLESS bytes are gRPC messages:
//! 5-byte prefix (1 compression flag + 4-byte big-endian length) + payload.
//! Matches xray-core/sing-box grpc transport wire format.

use bytes::{Bytes, BytesMut};
use h2::client::{SendRequest, SendStream};
use h2::RecvStream;
use std::io;
use std::pin::Pin;
use std::task::{Context as TaskCx, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use xray_tui_proto::proto_spec::common::GrpcConfig;

use crate::error::{NativeError, timeouts};
use crate::{BoxStream, context::LinkContext};

pub fn grpc_service_name(cfg: &GrpcConfig) -> String {
    cfg.service_name.as_deref().filter(|s| !s.is_empty()).unwrap_or("gun").to_string()
}

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(0);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

/// Try to decode one gRPC message from the front of `buf`. Consumes only
/// complete messages; leaves partial data in place. Returns `None` when a
/// full message is not yet available.
pub fn decode_frame(buf: &mut BytesMut) -> Option<Vec<u8>> {
    if buf.len() < 5 { return None; }
    let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
    if buf.len() < 5 + len { return None; }
    let payload = buf.split_to(5 + len).to_vec();
    Some(payload[5..].to_vec())
}

pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let cfg = ctx.transport_grpc()
        .ok_or_else(|| NativeError::Config("grpc transport requested but config is tcp".into()))?;
    let service = grpc_service_name(cfg);
    let authority = cfg.authority.as_deref()
        .unwrap_or(ctx.params.server.host.as_str())
        .to_string();
    let handshake = h2::client::handshake(stream);
    let (send_req, conn) = tokio::time::timeout(timeouts::TRANSPORT, handshake)
        .await
        .map_err(|_| NativeError::Timeout { step: "grpc h2 handshake", limit: timeouts::TRANSPORT })?
        .map_err(|e| NativeError::Transport(format!("grpc h2 handshake: {e}")))?;
    tokio::spawn(async move { let _ = conn.await; });

    let mut req = http::Request::builder()
        .method("POST")
        .uri(format!("https://{authority}/{service}/Tun"))
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .header("grpc-accept-encoding", "identity")
        .header("user-agent", cfg.user_agent.as_deref().unwrap_or("grpc-go/1.64.0"))
        .body(())
        .map_err(|e| NativeError::Transport(format!("grpc request: {e}")))?;
    if let Some(ua) = cfg.user_agent.as_deref() {
        req.headers_mut().insert("user-agent", ua.parse().unwrap());
    }
    let (resp, send_stream) = send_req.send_request(req, false)
        .map_err(|e| NativeError::Transport(format!("grpc stream open: {e}")))?;
    let resp = tokio::time::timeout(timeouts::TRANSPORT, resp)
        .await
        .map_err(|_| NativeError::Timeout { step: "grpc response headers", limit: timeouts::TRANSPORT })?
        .map_err(|e| NativeError::Transport(format!("grpc response: {e}")))?;
    if resp.status() != http::StatusCode::OK {
        return Err(NativeError::Transport(format!("grpc stream rejected: {}", resp.status())));
    }
    let recv_stream = resp.into_body();
    Ok(Box::new(GrpcStream::new(send_stream, recv_stream)))
}

/// Single bidirectional gRPC stream: AsyncWrite prefixes + sends DATA frames;
/// AsyncRead parses the 5-byte prefix and yields message payloads.
pub struct GrpcStream {
    send: SendStream<Bytes>,
    recv: RecvStream,
    write_buf: Vec<u8>,
    read_buf: BytesMut,
    payload: BytesMut,
}

impl GrpcStream {
    pub fn new(send: SendStream<Bytes>, recv: RecvStream) -> Self {
        Self { send, recv, write_buf: Vec::new(), read_buf: BytesMut::new(), payload: BytesMut::new() }
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
                let n = self.payload.as_ref().read(buf).unwrap_or(0);
                self.payload.advance(n);
                return Poll::Ready(Ok(()));
            }
            if let Some(msg) = decode_frame(&mut self.read_buf) {
                self.payload.extend_from_slice(&msg);
                continue;
            }
            match Pin::new(&mut self.recv).poll_data(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.read_buf.extend_from_slice(&chunk);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)))
                }
                Poll::Ready(None) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "grpc stream closed")))
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
                return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)));
            }
        }
        self.send.poll_capacity(cx).map_ok(|_| ()).map_err(io::Error::from)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        match self.send.poll_close(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p xray-tui-native --lib grpc::tests`
Expected: PASS.

- [ ] **Step 7: Wire the dispatch arm** (transport/mod.rs `upgrade`): `Some("grpc") => grpc::connect(ctx, stream).await,` + `pub mod grpc;`

- [ ] **Step 8: Compile + clippy + commit**

Run: `cargo test -p xray-tui-native --lib && cargo clippy -p xray-tui-native --all-targets` (zero warnings)
```bash
git add crates/xray-tui-native/src/transport/grpc.rs crates/xray-tui-native/src/transport/mod.rs crates/xray-tui-native/src/context.rs crates/xray-tui-native/Cargo.toml Cargo.lock
git commit -m "feat(native): gRPC transport over engine stream"
```

---
## Task 4: ALPN forcing per transport

**Files:**
- Modify: `crates/xray-tui-native/src/context.rs` (`alpn_vec`, ~line 100)
- Test: context.rs unit tests

**Interfaces:**
- Consumes: `TlsOpts.alpn: Option<Vec<Vec<u8>>>` (existing), `ctx.transport_type()`
- Produces: transport-aware `alpn_vec()` — explicit user ALPN wins; else grpc → `["h2"]`, ws → `["http/1.1"]`, tcp → `[]`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn alpn_defaults_per_transport() {
    // grpc without explicit alpn → h2
    let mut p = params_vless_ws_grpc("grpc");
    assert_eq!(LinkContext::new(p, target("x")).alpn_vec(), vec![b"h2".to_vec()]);
    // ws → http/1.1
    let mut p = params_vless_ws_grpc("ws");
    assert_eq!(LinkContext::new(p, target("x")).alpn_vec(), vec![b"http/1.1".to_vec()]);
    // explicit alpn wins
    let mut p = params_vless_ws_grpc("grpc");
    p.protocol.set_alpn(Some(vec![b"h2,http/1.1".to_vec()]));
    assert_eq!(LinkContext::new(p, target("x")).alpn_vec(), vec![b"h2,http/1.1".to_vec()]);
}
```
(Adapt to the actual `alpn_vec` signature: it may already split on comma — read context.rs:100-116 first and match its exact semantics.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p xray-tui-native --lib alpn_defaults_per_transport`
Expected: FAIL (no transport-aware logic).

- [ ] **Step 3: Implement** — in `alpn_vec()`, after the explicit-ALPN branch returns:

```rust
        // Transport-implied ALPN: grpc needs h2, ws upgrade is HTTP/1.1.
        // (Reality forces h2+http/1.1 server-side; explicit alpn wins above.)
        match self.transport_type() {
            Some("grpc") => vec![b"h2".to_vec()],
            Some("ws") => vec![b"http/1.1".to_vec()],
            _ => vec![],
        }
```

- [ ] **Step 4: Run tests + commit**

Run: `cargo test -p xray-tui-native --lib`
```bash
git add crates/xray-tui-native/src/context.rs
git commit -m "feat(native): transport-implied ALPN (grpc→h2, ws→http/1.1)"
```

---
## Task 5: E2E — ws/grpc against real cores

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/config.rs` (`vless_inbound` + `client_params_vless` gain transport), `crates/xray-tui-native/src/e2e/variant.rs` (new `TransportVariant` or transport field), `crates/xray-tui-native/src/e2e/case.rs` (`E2eCase` transports), the sweep runner (`tests/vless.rs` or e2e/mod.rs case list)
- Test: e2e sweep `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`

**Interfaces:**
- Consumes: `TlsVariant` (variant.rs), `vless_inbound(core, env, tls)` (config.rs:107), `client_params_vless(port, target, tls)` (config.rs:99 dispatch), core dialects
- Produces: 4 new cases — `VlessWsPlain`, `VlessWsTls`, `VlessGrpcPlain`, `VlessGrpcTls` (each runs × xray + sing-box)

- [ ] **Step 1: Verify wire-format ground truth** — read `thirdparty/Xray-core/transport/internet/grpc/` (serviceName default, `/Tun` path, headers) + `thirdparty/sing-box/transport/v2raygrpc/` (default service_name) + ws settings defaults in `thirdparty/Xray-core/transport/internet/ws/config.proto`. Adjust `grpc_service_name` default + ws path/host defaults if the sources disagree with this plan.

- [ ] **Step 2: Extend `vless_inbound`** with a `network: &str` param. For `CoreKind::Xray`:
```json
"streamSettings": { "network": "ws", "security": "tls", "tlsSettings": { "serverName": "<sni>", "allowInsecure": true, "certificates": [...] },
  "wsSettings": { "path": "/ws", "headers": { "Host": "xray.test" } } }
```
or `"network": "grpc", "grpcSettings": { "serviceName": "gun", "path": "/gun/Tun" }` — the exact shape per the existing tlsSettings block in vless_inbound (config.rs:85-130). Sing-box dialect: `"transport": { "type": "ws", "path": "/ws", "headers": { "Host": "xray.test" } }` / `{ "type": "grpc", "service_name": "gun" }` at the outbound/inbound level per sing-box schema.

- [ ] **Step 3: Extend `client_params_vless`** — add `transport: Option<&str>`; when set, emit `"transport": { "type": "ws", "path": "/ws", "host": "xray.test" }` or `{ "type": "grpc", "service_name": "gun" }` in the client ProtocolConfig JSON. The client params flow into `NativeConnectParams.protocol`.

- [ ] **Step 4: Add the 4 cases** — extend `E2eCase` (or add `TransportVariant` alongside `TlsVariant`) with `network: &'static str`; register `VlessWsPlain`, `VlessWsTls`, `VlessGrpcPlain`, `VlessGrpcTls` in the sweep list. TLS variants reuse `StandardTls`/`FingerprintTls` (chrome); REALITY excluded (tcp-only on both cores).

- [ ] **Step 5: Run the sweep**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: 11 existing + 8 new cases green × both cores.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/e2e/
git commit -m "test(e2e): ws+grpc transports vs xray + sing-box"
```

---
## Task 6: Docs

**Files:**
- Modify: `NATIVE_CORE.md` (transport matrix + layering), `AGENTS.md` (native transport section)

- [ ] **Step 1: NATIVE_CORE.md** — document: fold order `dial → security → transport upgrade → protocol`; WS (tokio-tungstenite over engine stream, Host/path/headers, Binary framing); gRPC (h2 over engine stream, 5-byte prefix, gun mode, `/Tun` path); ALPN forcing (grpc→h2, ws→http/1.1); out-of-scope transports (xhttp/httpupgrade/kcp/quic/splithttp, ws early data, grpc multi).
- [ ] **Step 2: AGENTS.md** — update the native section (transport module: tcp/ws/grpc) + the transport note in the architecture bullet.
- [ ] **Step 3: Full verification**

Run: `cargo test` (workspace) + `cargo clippy --workspace --all-targets` (zero warnings) + `cargo fmt --check`
```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs: ws+grpc transports in native layering"
```
