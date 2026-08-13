//! XHTTP transport (xray-core `splithttp`, v3 dialect — NO PDU framing, NO
//! X-TP-* headers).
//!
//! Session correlated by a uuid v4 in the URL path; download = a long-lived
//! GET response body; uploads = discrete POSTs of raw body bytes with a
//! decimal `seq` in the path. `auto`/default mode = packet-up.
//!
//! Wire reference: `thirdparty/Xray-core/transport/internet/splithttp/`
//! (dialer.go, client.go, config.go) — byte-identical to mihomo's xhttp
//! client.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context as TaskCx, Poll};
use std::time::{Duration, Instant};

use bytes::{Bytes, BytesMut};
use http::StatusCode;
use http::header::HOST;
use ring::rand::{SecureRandom, SystemRandom};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use xray_tui_proto::proto_spec::SecurityConfig;

use crate::transport::http::{IncomingReader, ReqBody, h1_client, h2_client};

/// packet-up chunk size (bytes) — xray default `scMaxEachPostBytes`.
const MAX_POST_BYTES: usize = 1_000_000;
/// minimum time between upload POSTs (ms) — xray default `scMinPostsIntervalMs`.
const POST_INTERVAL: Duration = Duration::from_millis(30);
/// `x_padding` length range (xray default `xPaddingBytes` 100..1000).
const PAD_MIN: usize = 100;
const PAD_MAX: usize = 1000;
/// Write-side buffering cap before pushing into the upload channel (one
/// packet-up chunk — the server 413s anything larger).
const MAX_PENDING: usize = MAX_POST_BYTES;

/// Normalize a path to start with `/` and ensure a trailing `/` before
/// appending session/seq (xray `appendToPath`).
fn normalize_path(path: &str) -> String {
    let p = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    if p.ends_with('/') { p } else { format!("{p}/") }
}

/// URL-safe session id (uuid v4).
fn session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Random `x_padding` value: 100–1000 `X` characters (X/Z are 8-bit HPACK
/// huffman codes, so byte length == HPACK length — valid under every server
/// validation path, incl. the tokenish huffman-length check).
fn x_padding() -> String {
    let mut buf = [0u8; 2];
    SystemRandom::new()
        .fill(&mut buf)
        .expect("ring CSPRNG fills");
    // `PAD_MAX - PAD_MIN + 1` keeps the inclusive [100, 1000] range (xray's
    // RangeConfig rand is inclusive).
    let len = PAD_MIN + usize::from(u16::from_be_bytes(buf)) % (PAD_MAX - PAD_MIN + 1);
    "X".repeat(len)
}

/// Referer header value: `{url}?x_padding={padding}` (xray query-in-header).
fn referer(url: &str, padding: &str) -> String {
    format!("{url}?x_padding={padding}")
}

/// Host for the HTTP requests: config `host` > endpoint host.
fn http_host(ctx: &LinkContext) -> String {
    ctx.transport_xhttp()
        .and_then(|c| c.host.as_deref())
        .map_or_else(|| ctx.params.server.host.clone(), str::to_string)
}

/// Base path without session (trailing slash).
fn base_path(ctx: &LinkContext) -> String {
    normalize_path(
        ctx.transport_xhttp()
            .and_then(|c| c.path.as_deref())
            .unwrap_or("/"),
    )
}

/// Path with optional session and seq appended (xray `appendToPath`).
fn path_with(ctx: &LinkContext, session: &str, seq: Option<&str>) -> String {
    let base = base_path(ctx);
    let mut p = format!("{base}{session}");
    if let Some(seq) = seq {
        p = format!("{p}/{seq}");
    }
    p
}

/// True when the link actually runs TLS/REALITY (the proto always carries a
/// `SecurityConfig`, even an empty one — `security().is_some()` is not the
/// right test). Drives the HTTP version: no TLS → HTTP/1.1, TLS → HTTP/2.
fn has_tls(ctx: &LinkContext) -> bool {
    ctx.security().and_then(SecurityConfig::type_str).is_some()
}

/// Build a request with Host + config headers + Referer padding.
fn build_request(
    ctx: &LinkContext,
    method: &str,
    url_path: &str,
    body: ReqBody,
    host: &str,
) -> Result<http::Request<ReqBody>, NativeError> {
    let padding = x_padding();
    // xray captures `RawURL` *before* appending session/seq, so the Referer
    // carries the base URL (`{scheme}://{host}{path}`), not the session path.
    let scheme = if has_tls(ctx) { "https" } else { "http" };
    let referer_url = format!("{scheme}://{host}{}", base_path(ctx));
    let mut builder = http::Request::builder()
        .method(method)
        .uri(url_path)
        .header(HOST, host)
        .header("Referer", referer(&referer_url, &padding));
    if let Some(headers) = ctx.transport_xhttp().and_then(|c| c.headers.as_ref()) {
        for (k, v) in headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
    }
    builder
        .body(body)
        .map_err(|e| NativeError::Transport(format!("xhttp request build: {e}")))
}

/// One-request sender seam: both hyper `http1::SendRequest` and
/// `http2::SendRequest` expose an inherent `send_request` (neither implements
/// `hyper::service::Service` in 1.x), so the shared request path is generic
/// over this.
trait SendOne {
    fn send_one(
        &mut self,
        req: http::Request<ReqBody>,
    ) -> impl Future<Output = Result<hyper::Response<hyper::body::Incoming>, hyper::Error>> + Send;
}

impl SendOne for hyper::client::conn::http1::SendRequest<ReqBody> {
    fn send_one(
        &mut self,
        req: http::Request<ReqBody>,
    ) -> impl Future<Output = Result<hyper::Response<hyper::body::Incoming>, hyper::Error>> + Send
    {
        self.send_request(req)
    }
}

impl SendOne for hyper::client::conn::http2::SendRequest<ReqBody> {
    fn send_one(
        &mut self,
        req: http::Request<ReqBody>,
    ) -> impl Future<Output = Result<hyper::Response<hyper::body::Incoming>, hyper::Error>> + Send
    {
        self.send_request(req)
    }
}

/// Send a request, require 200, return the response.
async fn send_200<S: SendOne>(
    sender: &mut S,
    req: http::Request<ReqBody>,
    step: &'static str,
) -> Result<hyper::Response<hyper::body::Incoming>, NativeError> {
    let limit = timeouts::TRANSPORT;
    let resp = tokio::time::timeout(limit, sender.send_one(req))
        .await
        .map_err(|_| NativeError::Timeout { step, limit })?
        .map_err(|e| NativeError::Transport(format!("{step}: {e}")))?;
    if resp.status() != StatusCode::OK {
        return Err(NativeError::Transport(format!(
            "{step}: expected 200, got {}",
            resp.status()
        )));
    }
    Ok(resp)
}

/// Send ONE packet-up POST: path `/x/{session}/{seq}`, raw body, 200
/// required. Shared by the h1 and h2 upload tasks (the sender type is the
/// only difference).
async fn post_packet<S: SendOne>(
    sender: &mut S,
    ctx: &LinkContext,
    session: &str,
    host: &str,
    seq: u64,
    payload: Bytes,
) -> Result<(), NativeError> {
    let seq_str = seq.to_string();
    let url_path = path_with(ctx, session, Some(&seq_str));
    let req = build_request(ctx, "POST", &url_path, ReqBody::Full(payload), host)?;
    let _resp = send_200(sender, req, "xhttp upload").await?;
    Ok(())
}

/// Shared packet-up pacing/batching core. Drains `rx`, accumulates app
/// bytes, and POSTs chunks (≤ [`MAX_POST_BYTES`], ≥ [`POST_INTERVAL`] apart,
/// one in flight, via [`post_packet`]) in seq order. The h1/h2 upload tasks
/// are thin wrappers over this core — they differ only in the sender type.
async fn upload_loop_core<S: SendOne>(
    sender: &mut S,
    ctx: &LinkContext,
    session: &str,
    host: &str,
    mut rx: mpsc::Receiver<Bytes>,
) {
    let mut chunk = BytesMut::new();
    let mut seq: u64 = 0;
    let mut last = Instant::now();
    loop {
        // Idle window: flush partial chunks on the timeout, or keep
        // accumulating until the size cap (xray `time.Sleep` before each
        // POST + `WithSizeLimit` batching).
        let mut closed = false;
        let mut filled = false;
        match tokio::time::timeout(POST_INTERVAL, rx.recv()).await {
            Ok(Some(data)) => {
                filled = true;
                chunk.extend_from_slice(&data);
            }
            Ok(None) => closed = true,
            Err(_) => {}
        }
        if filled && chunk.len() < MAX_POST_BYTES {
            continue;
        }
        if chunk.is_empty() {
            // Channel closed with nothing pending — done.
            if closed {
                break;
            }
            continue;
        }
        // Split at the size cap (xray `buf.SplitSize`): a single channel
        // item may push the accumulated chunk past `MAX_POST_BYTES`, and the
        // server 413s anything over `scMaxEachPostBytes`.
        loop {
            // The first POST is exempt from pacing: the 30 ms idle window
            // before it already spaced it from the GET.
            if seq > 0 {
                let since = last.elapsed();
                if since < POST_INTERVAL {
                    tokio::time::sleep(POST_INTERVAL.saturating_sub(since)).await;
                }
            }
            last = Instant::now();
            let take = chunk.len().min(MAX_POST_BYTES);
            let payload = chunk.split_to(take).freeze();
            if post_packet(sender, ctx, session, host, seq, payload)
                .await
                .is_err()
            {
                return;
            }
            seq += 1;
            if chunk.is_empty() {
                break;
            }
        }
        if closed {
            // Channel closed: the final chunk was flushed above — stop.
            break;
        }
    }
}

/// packet-up uploads over the HTTP/1.1 POST connection (a second raw dial).
async fn upload_loop_h1(
    mut sender: hyper::client::conn::http1::SendRequest<ReqBody>,
    ctx: LinkContext,
    session: String,
    host: String,
    rx: mpsc::Receiver<Bytes>,
) {
    upload_loop_core(&mut sender, &ctx, &session, &host, rx).await;
}

/// packet-up uploads over the shared HTTP/2 connection (same conn as GET).
async fn upload_loop_h2(
    mut sender: hyper::client::conn::http2::SendRequest<ReqBody>,
    ctx: LinkContext,
    session: String,
    host: String,
    rx: mpsc::Receiver<Bytes>,
) {
    upload_loop_core(&mut sender, &ctx, &session, &host, rx).await;
}

/// Run the XHTTP transport over the established (secured) stream.
///
/// `auto`/default = packet-up. HTTP version matches xray `decideHTTPVersion`:
/// no TLS → HTTP/1.1 (GET on the primary stream, POSTs on a second raw TCP
/// dial — Go h1 serializes responses in request order, so a long-lived GET
/// body would block POST responses on its own conn); TLS → HTTP/2 (one conn,
/// multiplexed streams).
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let mode = ctx
        .transport_xhttp()
        .and_then(|c| c.mode.as_deref())
        .unwrap_or("auto");
    match mode {
        "auto" | "packet-up" => {
            if has_tls(ctx) {
                packet_up_h2(ctx, stream).await
            } else {
                packet_up_h1(ctx, stream).await
            }
        }
        "stream-up" => Err(NativeError::NotImplemented {
            feature: "xhttp stream-up (Task 4)".into(),
        }),
        // "stream-one" (legacy XHTTP v1) is out of scope for this plan.
        other => Err(NativeError::NotImplemented {
            feature: format!("xhttp mode {other}"),
        }),
    }
}

/// packet-up over HTTP/1.1 (no TLS): GET on the primary stream, upload POSTs
/// on a second raw TCP dial.
async fn packet_up_h1(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let get_path = path_with(ctx, &session, None);
    let mut get_sender = h1_client(stream).await?;
    let get_req = build_request(ctx, "GET", &get_path, ReqBody::Empty, &host)?;
    let resp = send_200(&mut get_sender, get_req, "xhttp download GET").await?;
    let reader = IncomingReader::new(resp.into_body());

    // Second raw TCP dial for uploads.
    let post_stream = crate::transport::tcp::connect(ctx, None).await?;
    let post_sender = h1_client(post_stream).await?;

    let (tx, rx) = mpsc::channel::<Bytes>(4);
    let ctx = ctx.clone();
    let session_owned = session.clone();
    let host_owned = host.clone();
    tokio::spawn(upload_loop_h1(
        post_sender,
        ctx,
        session_owned,
        host_owned,
        rx,
    ));
    Ok(Box::new(XhttpStream::new(reader, tx)))
}

/// packet-up over HTTP/2 (TLS): one conn, GET stream + POST streams.
async fn packet_up_h2(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let get_path = path_with(ctx, &session, None);
    let mut sender = h2_client(stream).await?;
    let get_req = build_request(ctx, "GET", &get_path, ReqBody::Empty, &host)?;
    let resp = send_200(&mut sender, get_req, "xhttp download GET").await?;
    let reader = IncomingReader::new(resp.into_body());

    let (tx, rx) = mpsc::channel::<Bytes>(4);
    let ctx = ctx.clone();
    let session_owned = session.clone();
    let host_owned = host.clone();
    tokio::spawn(upload_loop_h2(sender, ctx, session_owned, host_owned, rx));
    Ok(Box::new(XhttpStream::new(reader, tx)))
}

/// An in-flight channel send (the channel was full when kicked) — a boxed
/// future keeps the type small.
type PendingSend = Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;

/// packet-up tunnel: read side = the GET download body; write side = a
/// buffered channel to the upload task.
///
/// The write path mirrors [`crate::transport::grpc::GrpcStream`]: accepted
/// bytes sit in `pending` until delivered — `poll_write` never returns
/// `WouldBlock` (the protocol write path treats it as fatal).
pub struct XhttpStream {
    /// The GET download body (read side). Concrete type: `IncomingReader` is
    /// `AsyncRead`-only, so it cannot live in a `BoxStream`.
    reader: IncomingReader,
    tx: mpsc::Sender<Bytes>,
    /// Accepted-but-unsent bytes.
    pending: BytesMut,
    /// An in-flight channel send (the channel was full when kicked).
    flushing: Option<PendingSend>,
}

impl XhttpStream {
    #[must_use]
    pub fn new(reader: IncomingReader, tx: mpsc::Sender<Bytes>) -> Self {
        Self {
            reader,
            tx,
            pending: BytesMut::new(),
            flushing: None,
        }
    }

    /// Push `pending` into the channel. Returns `false` when the uploader
    /// has gone (channel closed) — the write side is then broken.
    fn kick(&mut self) -> bool {
        if self.pending.is_empty() {
            return true;
        }
        let bytes = std::mem::take(&mut self.pending).freeze();
        match self.tx.try_send(bytes) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(bytes)) => {
                let tx = self.tx.clone();
                self.flushing = Some(Box::pin(
                    async move { tx.send(bytes).await.map_err(|_| ()) },
                ));
                true
            }
            Err(mpsc::error::TrySendError::Closed(_)) => false,
        }
    }

    /// Drive the in-flight send to completion. `Pending` = the channel is
    /// still full — the caller must not accept more bytes.
    fn poll_flushing(&mut self, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        while let Some(fut) = self.flushing.as_mut() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(())) => self.flushing = None,
                Poll::Ready(Err(())) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "xhttp uploader closed",
                    )));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for XhttpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl AsyncWrite for XhttpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Backpressure: an in-flight channel send must complete before we
        // accept more bytes (Pending = nothing consumed, per the AsyncWrite
        // contract — the protocol retries the write).
        match self.poll_flushing(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
        let take = buf
            .len()
            .min(MAX_PENDING.saturating_sub(self.pending.len()));
        self.pending.extend_from_slice(&buf[..take]);
        // Deliver on every write: `AsyncWriteExt::write_all` never calls
        // `poll_flush`, so flush-only delivery would strand bytes (ws.rs
        // mirrors this write-through choice). The upload task batches the
        // channel items and paces the POSTs itself.
        if !self.kick() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "xhttp uploader closed",
            )));
        }
        Poll::Ready(Ok(take))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        if !self.kick() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "xhttp uploader closed",
            )));
        }
        self.poll_flushing(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        // Flush the tail; dropping the stream then closes the channel (the
        // upload task POSTs the tail and stops).
        if !self.kick() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "xhttp uploader closed",
            )));
        }
        self.poll_flushing(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use xray_tui_proto::proto_spec::ProtocolConfig;
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

    use super::*;
    use crate::addr::{Host, TargetAddr};
    use crate::context::{LinkContext, NativeConnectParams};

    fn vless_xhttp(mode: &str) -> ProtocolConfig {
        // The proto's `TransportConfig` serde tag is snake_case: the variant
        // serializes/parses as `x_http` (the wire/type_str name is `xhttp`).
        serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "x_http", "path": "/x", "host": "example.com", "mode": mode }
        }))
        .expect("config parses")
    }

    fn ctx_at(addr: SocketAddr, mode: &str) -> LinkContext {
        LinkContext::new(
            NativeConnectParams::new(
                vless_xhttp(mode),
                EndpointEssentials::new(addr.ip().to_string(), addr.port()),
                TargetAddr::new(Host::Domain("dest.test".into()), 80),
            ),
            TargetAddr::new(Host::Domain("dest.test".into()), 80),
        )
    }

    async fn read_head(sock: &mut tokio::net::TcpStream) -> String {
        let mut head = Vec::new();
        let mut buf = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            sock.read_exact(&mut buf).await.unwrap();
            head.push(buf[0]);
        }
        String::from_utf8(head).unwrap()
    }

    /// Two connections: GET (download, session in path) + POSTs (upload, seq
    /// in path). Server responds 200 to both; GET body streams echoes.
    #[tokio::test]
    async fn packet_up_h1_get_and_post() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut get_sock, _) = listener.accept().await.unwrap();
            let get_head = read_head(&mut get_sock).await;
            assert!(get_head.starts_with("GET /x/"), "{get_head}");
            assert!(
                get_head
                    .to_lowercase()
                    .contains("referer: http://example.com/x/"),
                "{get_head}"
            );
            assert!(get_head.contains("x_padding="), "{get_head}");
            // 200 + SSE headers; body is the download stream.
            get_sock
                .write_all(
                    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/event-stream\r\nX-Accel-Buffering: no\r\nCache-Control: no-store\r\n\r\n",
                )
                .await
                .unwrap();
            // (upload comes on a second connection)
            let (mut up_sock, _) = listener.accept().await.unwrap();
            let up_head = read_head(&mut up_sock).await;
            assert!(up_head.starts_with("POST /x/"), "{up_head}");
            // hyper writes header names lowercase on the wire.
            assert!(
                up_head.to_lowercase().contains("content-length: 5"),
                "{up_head}"
            );
            // seq 0: the request-target is `/x/{session}/{seq}`.
            let target = up_head.split_whitespace().nth(1).unwrap();
            assert!(target.starts_with("/x/"), "{target}");
            assert!(target.ends_with("/0"), "{target}");
            let mut body = [0u8; 5];
            up_sock.read_exact(&mut body).await.unwrap();
            assert_eq!(&body, b"hello");
            up_sock
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .await
                .unwrap();
            // echo the downloaded bytes back through the GET chunked body
            get_sock
                .write_all(b"5\r\nworld\r\n0\r\n\r\n")
                .await
                .unwrap();
        });
        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let ctx = ctx_at(addr, "packet-up");
        let mut t = connect(&ctx, Box::new(stream)).await.unwrap();
        t.write_all(b"hello").await.unwrap();
        let mut out = [0u8; 5];
        t.read_exact(&mut out).await.unwrap();
        assert_eq!(&out, b"world");
        server.await.unwrap();
    }

    #[test]
    fn padding_is_repeat_x_within_range() {
        for _ in 0..50 {
            let pad = x_padding();
            assert!(
                pad.len() >= PAD_MIN && pad.len() <= PAD_MAX,
                "len {}",
                pad.len()
            );
            assert!(pad.bytes().all(|b| b == b'X'), "repeat-X only");
        }
    }

    #[test]
    fn referer_appends_padding_query() {
        assert_eq!(
            referer("http://example.com/x/", "XXX"),
            "http://example.com/x/?x_padding=XXX"
        );
    }

    #[test]
    fn paths_append_session_and_seq() {
        let ctx = ctx_at("127.0.0.1:1".parse().unwrap(), "packet-up");
        assert_eq!(path_with(&ctx, "sess", None), "/x/sess");
        assert_eq!(path_with(&ctx, "sess", Some("0")), "/x/sess/0");
        let no_slash = serde_json::from_value::<ProtocolConfig>(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "x_http", "path": "no-slash" }
        }))
        .expect("config parses");
        let ctx2 = LinkContext::new(
            NativeConnectParams::new(
                no_slash,
                EndpointEssentials::new("127.0.0.1", 1),
                TargetAddr::new(Host::Domain("dest.test".into()), 80),
            ),
            TargetAddr::new(Host::Domain("dest.test".into()), 80),
        );
        assert_eq!(base_path(&ctx2), "/no-slash/");
    }
}
