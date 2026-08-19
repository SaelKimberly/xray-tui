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
use std::sync::Arc;
use std::task::{Context as TaskCx, Poll, ready};
use std::time::{Duration, Instant};

use bytes::{Buf, Bytes, BytesMut};
use http::StatusCode;
use http::header::HOST;
use ring::rand::{SecureRandom, SystemRandom};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use xray_tui_proto::proto_spec::{SecurityConfig, TlsConfig};

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

/// HTTP version for xhttp per xray `decideHTTPVersion` (spec §4.1) — a pure
/// mirror of the Go decision over the proto security config:
///
/// | condition | version |
/// |---|---|
/// | reality config present | `"2"` |
/// | no TLS config | `"1.1"` |
/// | TLS, exactly one ALPN `"http/1.1"` | `"1.1"` |
/// | TLS, exactly one ALPN `"h3"` | `"3"` (the QUIC path) |
/// | TLS, exactly one other ALPN | `"2"` |
/// | TLS, zero or 2+ ALPNs | `"2"` |
///
/// ALPN is the comma-separated `TlsOpts::alpn` string, split exactly like
/// the engine builds its ALPN list ([`LinkContext::alpn_vec`]: split on
/// `,`, trim, drop empties).
#[must_use]
pub(crate) fn http_version(security: Option<&SecurityConfig>) -> &'static str {
    let Some(sec) = security else {
        return "1.1";
    };
    let alpn = match &sec.tls {
        None => return "1.1",
        Some(TlsConfig::Reality(_)) => return "2",
        Some(TlsConfig::Tls(opts)) => match &opts.alpn {
            Some(s) => s.as_str(),
            None => return "2",
        },
    };
    let mut protocols = alpn.split(',').map(str::trim).filter(|s| !s.is_empty());
    let Some(first) = protocols.next() else {
        return "2";
    };
    if protocols.next().is_some() {
        return "2";
    }
    match first {
        "http/1.1" => "1.1",
        "h3" => "3",
        _ => "2",
    }
}

/// Build a request with Host + config headers + Referer padding (shared by
/// the h1/h2 hyper arm and the h3 arm — the body is unit; callers map in
/// their transport's body type).
fn build_request(
    ctx: &LinkContext,
    method: &str,
    url_path: &str,
    host: &str,
) -> Result<http::Request<()>, NativeError> {
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
        .body(())
        .map_err(|e| NativeError::Transport(format!("xhttp request build: {e}")))
}

/// Request body for the v3 protocol (transport-agnostic). The hyper arms
/// map it onto `ReqBody`; the h3 arm onto the request stream's send half.
enum V3Body {
    Empty,
    Full(Bytes),
    /// Stream-up: the upload body is a pipe fed by the tunnel writer
    /// (`wrx`); the seam forwards into the transport's body mechanism.
    Pipe(mpsc::Receiver<Bytes>),
}

/// A v3 response: status + the response body as an `AsyncRead`. For POSTs
/// the seam has already started draining the actual body (keepalive
/// X-blobs — xray hub.go) and the reader here is empty; GETs carry the
/// download body.
struct V3Response {
    status: StatusCode,
    reader: Box<dyn AsyncRead + Send + Unpin>,
}

/// One-request sender seam for the splithttp v3 protocol. Both the hyper
/// `SendRequest`s (h1/h2) and the h3 `SendRequest` implement it, so the
/// protocol logic (session open, GET download, POST uploads, pacing,
/// packet-up/stream-up) is written once over this seam.
trait V3Send {
    fn send_one(
        &mut self,
        req: http::Request<V3Body>,
        step: &'static str,
    ) -> impl Future<Output = Result<V3Response, NativeError>> + Send;
}

/// Drain a hyper response body to completion in a spawned task: the
/// stream-up server streams keepalive X-blobs into the upload response when
/// a Referer is present (xray hub.go) — the drain keeps the h1/h2 window
/// from stalling, so it must run concurrently with the tunnel.
fn drain_incoming(reader: IncomingReader) {
    tokio::spawn(async move {
        let mut reader = reader;
        if let Err(e) = tokio::io::copy(&mut reader, &mut tokio::io::sink()).await {
            tracing::debug!(error = %e, "xhttp: drain failed on body read");
        }
    });
}

/// Forward tunnel writes into the stream-up upload pipe.
///
/// `send_data` awaits pipe capacity (hyper drains it into h1/h2 DATA
/// frames), so backpressure propagates without dropping bytes; `SendError`
/// fires only when the body is gone (stream reset) — the upload side is
/// then broken.
async fn forward_pipe(
    mut pipe_tx: http_body_util::channel::Sender<Bytes>,
    mut wrx: mpsc::Receiver<Bytes>,
) {
    while let Some(b) = wrx.recv().await {
        if pipe_tx.send_data(b).await.is_err() {
            return;
        }
    }
}

impl V3Send for hyper::client::conn::http1::SendRequest<ReqBody> {
    async fn send_one(
        &mut self,
        req: http::Request<V3Body>,
        step: &'static str,
    ) -> Result<V3Response, NativeError> {
        let (parts, body) = req.into_parts();
        let (pipe_rx, body) = match body {
            V3Body::Empty => (None, ReqBody::Empty),
            V3Body::Full(b) => (None, ReqBody::Full(b)),
            V3Body::Pipe(rx) => {
                let (tx, body) = ReqBody::channel();
                (Some((tx, rx)), body)
            }
        };
        let req = http::Request::from_parts(parts, body);
        let resp = self
            .send_request(req)
            .await
            .map_err(|e| NativeError::Transport(format!("{step}: {e}")))?;
        let status = resp.status();
        let reader: Box<dyn AsyncRead + Send + Unpin> = match pipe_rx {
            Some((tx, rx)) => {
                // The stream-up server 200s the POST immediately and streams
                // keepalive X-blobs into the response (xray hub.go) — drain
                // concurrently so the h1/h2 window doesn't stall.
                drain_incoming(IncomingReader::new(resp.into_body()));
                tokio::spawn(forward_pipe(tx, rx));
                Box::new(tokio::io::empty())
            }
            None => Box::new(IncomingReader::new(resp.into_body())),
        };
        Ok(V3Response { status, reader })
    }
}

impl V3Send for hyper::client::conn::http2::SendRequest<ReqBody> {
    async fn send_one(
        &mut self,
        req: http::Request<V3Body>,
        step: &'static str,
    ) -> Result<V3Response, NativeError> {
        let (parts, body) = req.into_parts();
        let (pipe_rx, body) = match body {
            V3Body::Empty => (None, ReqBody::Empty),
            V3Body::Full(b) => (None, ReqBody::Full(b)),
            V3Body::Pipe(rx) => {
                let (tx, body) = ReqBody::channel();
                (Some((tx, rx)), body)
            }
        };
        let req = http::Request::from_parts(parts, body);
        let resp = self
            .send_request(req)
            .await
            .map_err(|e| NativeError::Transport(format!("{step}: {e}")))?;
        let status = resp.status();
        let reader: Box<dyn AsyncRead + Send + Unpin> = match pipe_rx {
            Some((tx, rx)) => {
                drain_incoming(IncomingReader::new(resp.into_body()));
                tokio::spawn(forward_pipe(tx, rx));
                Box::new(tokio::io::empty())
            }
            None => Box::new(IncomingReader::new(resp.into_body())),
        };
        Ok(V3Response { status, reader })
    }
}

/// Drain an h3 response body to completion in a spawned task: the stream-up
/// server streams keepalive X-blobs into the upload response (xray hub.go)
/// — Go's client does `io.Copy(io.Discard, resp.Body)`; the drain must run
/// concurrently with the tunnel.
fn drain_h3<S>(stream: h3::client::RequestStream<S, Bytes>)
where
    S: h3::quic::RecvStream + Send + 'static,
{
    tokio::spawn(async move {
        let mut stream = stream;
        while let Ok(Some(_)) = stream.recv_data().await {}
    });
}

/// Forward tunnel writes into the stream-up h3 upload pipe: each channel
/// item becomes a request-body DATA chunk; the body ends on channel close.
async fn forward_pipe_h3(
    mut send: h3::client::RequestStream<h3_quinn::SendStream<Bytes>, Bytes>,
    mut wrx: mpsc::Receiver<Bytes>,
) {
    while let Some(b) = wrx.recv().await {
        if send.send_data(b).await.is_err() {
            return;
        }
    }
    let _ = send.finish().await;
}

/// The h3 request seam: one h3 request per call, over the shared QUIC
/// connection (h3 multiplexes bidirectional streams like h2).
impl V3Send for h3::client::SendRequest<h3_quinn::OpenStreams, Bytes> {
    async fn send_one(
        &mut self,
        req: http::Request<V3Body>,
        step: &'static str,
    ) -> Result<V3Response, NativeError> {
        let (parts, body) = req.into_parts();
        let req = http::Request::from_parts(parts, ());
        let err = |e: h3::error::StreamError| NativeError::Transport(format!("{step}: {e}"));
        let mut stream = self.send_request(req).await.map_err(err)?;
        match body {
            V3Body::Empty => {
                stream.finish().await.map_err(err)?;
                let resp = stream.recv_response().await.map_err(err)?;
                let status = resp.status();
                Ok(V3Response {
                    status,
                    reader: Box::new(H3Reader::new(stream)),
                })
            }
            V3Body::Full(payload) => {
                stream.send_data(payload).await.map_err(err)?;
                stream.finish().await.map_err(err)?;
                let resp = stream.recv_response().await.map_err(err)?;
                let status = resp.status();
                drain_h3(stream);
                Ok(V3Response {
                    status,
                    reader: Box::new(tokio::io::empty()),
                })
            }
            V3Body::Pipe(rx) => {
                let (send, mut recv) = stream.split();
                let resp = recv.recv_response().await.map_err(err)?;
                let status = resp.status();
                // The forward task outlives this call: it must keep a
                // `SendRequest` clone so the h3 connection stays open for
                // the tunnel's lifetime (dropping the last clone closes it
                // with H3_NO_ERROR).
                let keepalive = self.clone();
                tokio::spawn(async move {
                    let _keepalive = keepalive;
                    forward_pipe_h3(send, rx).await;
                });
                drain_h3(recv);
                Ok(V3Response {
                    status,
                    reader: Box::new(tokio::io::empty()),
                })
            }
        }
    }
}

/// `AsyncRead` over an h3 response body: `recv_data` yields `Bytes` chunks
/// (h3-quinn's read-chunk adapter), copied into the pending buffer.
struct H3Reader {
    stream: Option<h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>>,
    buf: Bytes,
}

impl H3Reader {
    const fn new(stream: h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>) -> Self {
        Self {
            stream: Some(stream),
            buf: Bytes::new(),
        }
    }
}

impl AsyncRead for H3Reader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            if !self.buf.is_empty() {
                let n = self.buf.len().min(buf.remaining());
                buf.put_slice(&self.buf[..n]);
                self.buf.advance(n);
                return Poll::Ready(Ok(()));
            }
            let Some(stream) = self.stream.as_mut() else {
                return Poll::Ready(Ok(()));
            };
            match ready!(stream.poll_recv_data(cx)) {
                Ok(Some(mut data)) => {
                    self.buf = data.copy_to_bytes(data.remaining());
                }
                Ok(None) => {
                    self.stream = None;
                    return Poll::Ready(Ok(()));
                }
                Err(e) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        format!("xhttp h3 download: {e:?}"),
                    )));
                }
            }
        }
    }
}

/// Send a request, require 200, return the response.
async fn send_200<S: V3Send>(
    sender: &mut S,
    req: http::Request<V3Body>,
    step: &'static str,
) -> Result<V3Response, NativeError> {
    let limit = timeouts::TRANSPORT;
    let resp = tokio::time::timeout(limit, sender.send_one(req, step))
        .await
        .map_err(|_| NativeError::Timeout { step, limit })??;
    if resp.status != StatusCode::OK {
        return Err(NativeError::Transport(format!(
            "{step}: expected 200, got {}",
            resp.status
        )));
    }
    Ok(resp)
}

/// Send ONE packet-up POST: path `/x/{session}/{seq}`, raw body, 200
/// required. Shared by the h1, h2 and h3 upload tasks (the sender type is
/// the only difference).
async fn post_packet<S: V3Send>(
    sender: &mut S,
    ctx: &LinkContext,
    session: &str,
    host: &str,
    seq: u64,
    payload: Bytes,
) -> Result<(), NativeError> {
    let seq_str = seq.to_string();
    let url_path = path_with(ctx, session, Some(&seq_str));
    let req = build_request(ctx, "POST", &url_path, host)?.map(|()| V3Body::Full(payload));
    let _resp = send_200(sender, req, "xhttp upload").await?;
    Ok(())
}

/// Shared packet-up pacing/batching core. Drains `rx`, accumulates app
/// bytes, and POSTs chunks (≤ [`MAX_POST_BYTES`], ≥ [`POST_INTERVAL`] apart,
/// one in flight, via [`post_packet`]) in seq order. The h1/h2/h3 upload
/// tasks are thin wrappers over this core — they differ only in the sender
/// type.
async fn upload_loop_core<S: V3Send>(
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

/// packet-up uploads over the shared HTTP/3 connection (same QUIC conn as
/// the GET — h3 multiplexes streams like h2).
async fn upload_loop_h3(
    mut sender: h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>,
    ctx: LinkContext,
    session: String,
    host: String,
    rx: mpsc::Receiver<Bytes>,
) {
    upload_loop_core(&mut sender, &ctx, &session, &host, rx).await;
}

/// The client-side xhttp dialect (config `mode`): `auto`/empty →
/// packet-up. Shared by the h1/h2 [`connect`] dispatch and the h3 arm.
fn xhttp_mode(ctx: &LinkContext) -> Result<&'static str, NativeError> {
    let mode = ctx
        .transport_xhttp()
        .and_then(|c| c.mode.as_deref())
        .unwrap_or("auto");
    match mode {
        "stream-up" => Ok("stream-up"),
        "packet-up" | "auto" | "" => Ok("packet-up"),
        "stream-one" => Err(NativeError::NotImplemented {
            feature: "xhttp stream-one mode".into(),
        }),
        other => Err(NativeError::Config(format!("unknown xhttp mode: {other}"))),
    }
}

/// Run the XHTTP transport over the established (secured) stream.
///
/// The client-side mode (config `mode`) selects the dialect: `auto`/empty →
/// packet-up; `stream-up` supported; `stream-one` (legacy XHTTP v1) →
/// `NotImplemented`. HTTP version matches xray `decideHTTPVersion`: no
/// TLS → HTTP/1.1 (GET on the primary stream, POSTs on a second raw TCP
/// dial — Go h1 serializes responses in request order, so a long-lived GET
/// body would block POST responses on its own conn); TLS → HTTP/2 (one conn,
/// multiplexed streams). The h3 mode is NOT reached here: it is a different
/// dial ([`connect_quic`]) that replaces this upgrade step (spec §5.2).
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match xhttp_mode(ctx)? {
        "stream-up" => {
            if has_tls(ctx) {
                stream_up_h2(ctx, stream).await
            } else {
                stream_up_h1(ctx, stream).await
            }
        }
        _ => {
            if has_tls(ctx) {
                packet_up_h2(ctx, stream).await
            } else {
                packet_up_h1(ctx, stream).await
            }
        }
    }
}

/// rustls `ServerCertVerifier` for the `insecure` TLS option: skips the
/// certificate chain walk but still verifies the handshake signatures (xray
/// `allowInsecure` semantics — the QUIC session stays sound, only the
/// identity check is dropped).
#[derive(Debug)]
struct SkipServerVerify {
    algorithms: rustls::crypto::WebPkiSupportedAlgorithms,
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.algorithms.supported_schemes()
    }
}

/// rustls client config for the QUIC dial: ALPN `h3` (the trigger — spec
/// §4.1), early data enabled (0-RTT), and cert verification mirroring the
/// TLS opts (`insecure` → skip the chain walk; default → webpki-roots). The
/// engine TLS + fingerprint machinery does not apply to QUIC (spec §5.2) —
/// quinn's rustls is internal.
fn quic_tls_config(ctx: &LinkContext) -> Result<rustls::ClientConfig, NativeError> {
    let insecure = ctx.tls_opts()?.and_then(|o| o.insecure).unwrap_or(false);
    let mut tls = if insecure {
        let algorithms = rustls::crypto::ring::default_provider().signature_verification_algorithms;
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerify { algorithms }))
            .with_no_client_auth()
    } else {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth()
    };
    tls.alpn_protocols = vec![b"h3".to_vec()];
    tls.enable_early_data = true;
    Ok(tls)
}

/// The HTTP/3 arm (xhttp + [`http_version`] == `"3"` → QUIC dial).
///
/// A quinn Endpoint over UDP with ALPN `h3`, replacing the dial + security +
/// upgrade chain (spec §5.2 — quinn's TLS is internal; the engine TLS never
/// wraps QUIC). The splithttp v3 protocol then runs over the h3 connection
/// (same session/GET/POST logic as the h1/h2 arms, via the [`V3Send`] seam).
/// 0-RTT: quinn early data is enabled; a fresh connection has no session
/// ticket and does the full handshake (xray `DialEarly` without a ticket —
/// spec §8.3).
pub async fn connect_quic(ctx: &LinkContext) -> Result<BoxStream, NativeError> {
    let mode = xhttp_mode(ctx)?;
    let server_addr = ctx.server_socket().await?;
    // Bind the same address family as the server (quinn needs one socket).
    let bind: std::net::SocketAddr = if server_addr.is_ipv6() {
        "[::]:0".parse().expect("static ipv6 bind addr")
    } else {
        "0.0.0.0:0".parse().expect("static ipv4 bind addr")
    };
    let endpoint = quinn::Endpoint::client(bind)?;
    let tls = quic_tls_config(ctx)?;
    let mut quic_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(tls)
            .map_err(|e| NativeError::Config(format!("xhttp h3 tls config: {e}")))?,
    ));
    // Spec §4.2: xray `ConnIdleTimeout` = 300s + `QuicgoH3KeepAlivePeriod` =
    // 10s. quinn's defaults (30s idle, no keepalive) would kill an idle h3
    // tunnel mid-session where xray survives (the e2e wouldn't catch it:
    // xray's server keepalives reset the client's idle timer).
    let mut transport = quinn::TransportConfig::default();
    transport
        .max_idle_timeout(Some(
            quinn::IdleTimeout::try_from(Duration::from_mins(5))
                .expect("300s idle timeout fits a VarInt"),
        ))
        .keep_alive_interval(Some(Duration::from_secs(10)));
    quic_config.transport_config(Arc::new(transport));
    let connecting = endpoint
        .connect_with(quic_config, server_addr, &ctx.sni())
        .map_err(|e| NativeError::Dial(format!("xhttp h3 connect: {e}")))?;
    let conn = match connecting.into_0rtt() {
        Ok((conn, _accepted)) => conn,
        Err(connecting) => {
            let limit = timeouts::DIAL;
            tokio::time::timeout(limit, connecting)
                .await
                .map_err(|_| NativeError::Timeout {
                    step: "xhttp h3 handshake",
                    limit,
                })?
                .map_err(|e| NativeError::Dial(format!("xhttp h3 handshake: {e}")))?
        }
    };

    // The h3 connection driver owns the endpoint + quinn connection: it
    // polls the connection-level state (control frames, settings) and lives
    // exactly as long as the tunnel — when the last `SendRequest` clone
    // drops, the connection closes with H3_NO_ERROR, `poll_close` resolves,
    // and the endpoint goes with it.
    let h3_quic = h3_quinn::Connection::new(conn);
    let (mut h3_conn, send_request) = {
        let limit = timeouts::TRANSPORT;
        tokio::time::timeout(limit, h3::client::new(h3_quic))
            .await
            .map_err(|_| NativeError::Timeout {
                step: "xhttp h3 open",
                limit,
            })?
            .map_err(|e| NativeError::Transport(format!("xhttp h3 open: {e}")))?
    };
    tokio::spawn(async move {
        let _ = std::future::poll_fn(|cx| h3_conn.poll_close(cx)).await;
        drop(endpoint);
    });

    match mode {
        "packet-up" => packet_up_h3(ctx, send_request).await,
        _ => stream_up_h3(ctx, send_request).await,
    }
}

/// packet-up over HTTP/1.1 (no TLS): GET on the primary stream, upload POSTs
/// on a second raw TCP dial.
async fn packet_up_h1(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let get_path = path_with(ctx, &session, None);
    let mut get_sender = h1_client(stream).await?;
    let get_req = build_request(ctx, "GET", &get_path, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut get_sender, get_req, "xhttp download GET").await?;
    let reader = resp.reader;

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
    let get_req = build_request(ctx, "GET", &get_path, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut sender, get_req, "xhttp download GET").await?;
    let reader = resp.reader;

    let (tx, rx) = mpsc::channel::<Bytes>(4);
    let ctx = ctx.clone();
    let session_owned = session.clone();
    let host_owned = host.clone();
    tokio::spawn(upload_loop_h2(sender, ctx, session_owned, host_owned, rx));
    Ok(Box::new(XhttpStream::new(reader, tx)))
}

/// packet-up over HTTP/3: one QUIC conn, GET stream + POST streams (h3
/// multiplexes like h2 — the upload task reuses the same [`V3Send`] loop).
async fn packet_up_h3(
    ctx: &LinkContext,
    mut sender: h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>,
) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let get_path = path_with(ctx, &session, None);
    let get_req = build_request(ctx, "GET", &get_path, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut sender, get_req, "xhttp h3 download GET").await?;
    let reader = resp.reader;

    let (tx, rx) = mpsc::channel::<Bytes>(4);
    let ctx = ctx.clone();
    let session_owned = session.clone();
    let host_owned = host.clone();
    tokio::spawn(upload_loop_h3(sender, ctx, session_owned, host_owned, rx));
    Ok(Box::new(XhttpStream::new(reader, tx)))
}

/// stream-up: GET (download) + one long-lived POST (upload, pipe body,
/// `Content-Type: application/grpc`) on a second raw TCP dial. The server
/// 200s the POST immediately and streams keepalive X-blobs into the response
/// (drained by the seam).
async fn stream_up_h1(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let url = path_with(ctx, &session, None);
    let mut get_sender = h1_client(stream).await?;
    let get_req = build_request(ctx, "GET", &url, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut get_sender, get_req, "xhttp stream-up GET").await?;
    let reader = resp.reader;

    let post_stream = crate::transport::tcp::connect(ctx, None).await?;
    let mut post_sender = h1_client(post_stream).await?;
    let (wt, wrx) = mpsc::channel::<Bytes>(4);
    let mut req = build_request(ctx, "POST", &url, &host)?.map(|()| V3Body::Pipe(wrx));
    req.headers_mut().insert(
        "Content-Type",
        http::HeaderValue::from_static("application/grpc"),
    );
    let _resp = send_200(&mut post_sender, req, "xhttp stream-up POST").await?;
    Ok(Box::new(XhttpStream::new(reader, wt)))
}

/// stream-up over h2 (TLS): GET + POST streams on one h2 conn.
async fn stream_up_h2(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let url = path_with(ctx, &session, None);
    let mut sender = h2_client(stream).await?;
    let get_req = build_request(ctx, "GET", &url, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut sender, get_req, "xhttp stream-up GET").await?;
    let reader = resp.reader;

    let (wt, wrx) = mpsc::channel::<Bytes>(4);
    let mut req = build_request(ctx, "POST", &url, &host)?.map(|()| V3Body::Pipe(wrx));
    req.headers_mut().insert(
        "Content-Type",
        http::HeaderValue::from_static("application/grpc"),
    );
    let _resp = send_200(&mut sender, req, "xhttp stream-up POST").await?;
    Ok(Box::new(XhttpStream::new(reader, wt)))
}

/// stream-up over h3: GET + POST streams on the one QUIC conn (the pipe
/// body maps onto the h3 request stream's send half).
async fn stream_up_h3(
    ctx: &LinkContext,
    mut sender: h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>,
) -> Result<BoxStream, NativeError> {
    let host = http_host(ctx);
    let session = session_id();
    let url = path_with(ctx, &session, None);
    let get_req = build_request(ctx, "GET", &url, &host)?.map(|()| V3Body::Empty);
    let resp = send_200(&mut sender, get_req, "xhttp h3 stream-up GET").await?;
    let reader = resp.reader;

    let (wt, wrx) = mpsc::channel::<Bytes>(4);
    let mut req = build_request(ctx, "POST", &url, &host)?.map(|()| V3Body::Pipe(wrx));
    req.headers_mut().insert(
        "Content-Type",
        http::HeaderValue::from_static("application/grpc"),
    );
    let _resp = send_200(&mut sender, req, "xhttp h3 stream-up POST").await?;
    Ok(Box::new(XhttpStream::new(reader, wt)))
}

/// An in-flight channel send (the channel was full when kicked) — a boxed
/// future keeps the type small.
type PendingSend = Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;

/// packet-up tunnel: read side = the GET download body; write side = a
/// buffered channel to the upload task.
///
/// The write path mirrors [`crate::transport::grpc::GrpcStream`]: accepted
/// bytes sit in `pending` until delivered — `poll_write` never returns
/// `WouldBlock` (the protocol write path treats it as fatal). `R` is the
/// download reader: `IncomingReader` on the h1/h2 arms, an h3 body reader
/// on the QUIC arm.
pub struct XhttpStream<R> {
    /// The GET download body (read side).
    reader: R,
    tx: mpsc::Sender<Bytes>,
    /// Accepted-but-unsent bytes.
    pending: BytesMut,
    /// An in-flight channel send (the channel was full when kicked).
    flushing: Option<PendingSend>,
}

impl<R> XhttpStream<R> {
    #[must_use]
    pub fn new(reader: R, tx: mpsc::Sender<Bytes>) -> Self {
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

impl<R: AsyncRead + Unpin + Send> AsyncRead for XhttpStream<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl<R: AsyncRead + Unpin + Send> AsyncWrite for XhttpStream<R> {
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
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::Notify;
    use tokio::task::JoinHandle;
    use xray_tui_proto::proto_spec::ProtocolConfig;
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
    use xray_tui_proto::proto_spec::{RealityOpts, TlsOpts};
    use xray_tui_proto::urlx::TinyText;

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

    /// Decode the chunked POST body off the raw socket: parse size lines and
    /// payloads until `want` has arrived (the stream-up tunnel POST stays
    /// open, so the caller stops early) or the peer closes. Returns the
    /// de-chunked payload read so far.
    async fn read_chunked_until(sock: &mut tokio::net::TcpStream, want: &[u8]) -> Vec<u8> {
        let mut raw: Vec<u8> = Vec::new();
        let mut body: Vec<u8> = Vec::new();
        let mut tmp = [0u8; 4096];
        let mut expect = 0usize; // payload bytes remaining in the current chunk
        loop {
            if expect == 0 {
                if let Some(pos) = raw.windows(2).position(|w| w == b"\r\n") {
                    let size =
                        usize::from_str_radix(std::str::from_utf8(&raw[..pos]).unwrap().trim(), 16)
                            .unwrap();
                    raw.drain(..pos + 2);
                    if size == 0 {
                        return body; // terminating last-chunk
                    }
                    expect = size;
                } else {
                    let n = sock.read(&mut tmp).await.unwrap();
                    if n == 0 {
                        return body;
                    }
                    raw.extend_from_slice(&tmp[..n]);
                    continue;
                }
            }
            if raw.len() < expect + 2 {
                let n = sock.read(&mut tmp).await.unwrap();
                if n == 0 {
                    return body;
                }
                raw.extend_from_slice(&tmp[..n]);
                continue;
            }
            body.extend_from_slice(&raw[..expect]);
            raw.drain(..expect + 2); // payload + trailing CRLF
            expect = 0;
            if body == want {
                return body;
            }
        }
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

    /// stream-up: GET (download, 200 + streaming body) + POST (upload, pipe
    /// body with application/grpc). The server 200s the POST immediately
    /// (xray hub.go) and streams echoes on the GET; the tunnel POST stays
    /// open for the connection's lifetime, so the server echoes as soon as
    /// the payload arrived (waiting for EOF would deadlock — xray streams
    /// continuously).
    #[tokio::test]
    async fn stream_up_h1_get_and_post() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut get_sock, _) = listener.accept().await.unwrap();
            let get_head = read_head(&mut get_sock).await;
            assert!(get_head.starts_with("GET /x/"), "{get_head}");
            get_sock
                .write_all(
                    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/event-stream\r\n\r\n",
                )
                .await
                .unwrap();
            let (mut up_sock, _) = listener.accept().await.unwrap();
            let up_head = read_head(&mut up_sock).await;
            assert!(up_head.starts_with("POST /x/"), "{up_head}");
            // hyper writes header names lowercase on the wire.
            assert!(
                up_head
                    .to_lowercase()
                    .contains("content-type: application/grpc"),
                "{up_head}"
            );
            // xray hub.go 200s the upload POST immediately, then streams
            // keepalive X-blobs into the response body (drained client-side).
            up_sock
                .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n")
                .await
                .unwrap();
            // POST body is chunked (unknown-length pipe); the tunnel stays
            // open — decode the framing and echo as soon as the payload
            // arrived (waiting for EOF would deadlock; xray streams echoes
            // continuously).
            let body = read_chunked_until(&mut up_sock, b"hello").await;
            assert_eq!(&body, b"hello");
            get_sock
                .write_all(b"6\r\nworld!\r\n0\r\n\r\n")
                .await
                .unwrap();
        });
        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let ctx = ctx_at(addr, "stream-up");
        let mut t = connect(&ctx, Box::new(stream)).await.unwrap();
        t.write_all(b"hello").await.unwrap();
        let mut out = [0u8; 6];
        t.read_exact(&mut out).await.unwrap();
        assert_eq!(&out, b"world!");
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

    // --- decideHTTPVersion mirror (spec §4.1) -----------------------------

    fn tls(alpn: Option<&str>) -> SecurityConfig {
        SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                alpn: alpn.map(TinyText::from),
                ..TlsOpts::default()
            })),
            ..SecurityConfig::default()
        }
    }

    fn reality() -> SecurityConfig {
        SecurityConfig {
            tls: Some(TlsConfig::Reality(RealityOpts::default())),
            ..SecurityConfig::default()
        }
    }

    #[test]
    fn dispatch_no_security_is_http_1_1() {
        assert_eq!(http_version(None), "1.1");
        assert_eq!(http_version(Some(&SecurityConfig::default())), "1.1");
    }

    #[test]
    fn dispatch_reality_is_http_2() {
        // reality config present → "2", even with a single h3 ALPN
        // (reality is h2-only in splithttp).
        assert_eq!(http_version(Some(&reality())), "2");
    }

    #[test]
    fn dispatch_single_http_1_1_alpn_is_http_1_1() {
        assert_eq!(http_version(Some(&tls(Some("http/1.1")))), "1.1");
        assert_eq!(http_version(Some(&tls(Some(" http/1.1 ")))), "1.1");
    }

    #[test]
    fn dispatch_single_h3_alpn_is_http_3() {
        assert_eq!(http_version(Some(&tls(Some("h3")))), "3");
        assert_eq!(http_version(Some(&tls(Some(" h3 ")))), "3");
    }

    #[test]
    fn dispatch_other_single_alpn_is_http_2() {
        assert_eq!(http_version(Some(&tls(Some("h2")))), "2");
        assert_eq!(http_version(Some(&tls(Some("silly")))), "2");
    }

    #[test]
    fn dispatch_zero_alpn_is_http_2() {
        assert_eq!(http_version(Some(&tls(None))), "2");
        assert_eq!(http_version(Some(&tls(Some("")))), "2");
        assert_eq!(http_version(Some(&tls(Some(" , ")))), "2");
    }

    #[test]
    fn dispatch_two_plus_alpn_is_http_2() {
        assert_eq!(http_version(Some(&tls(Some("h2,http/1.1")))), "2");
        assert_eq!(http_version(Some(&tls(Some("h3,h2")))), "2");
        assert_eq!(http_version(Some(&tls(Some("h2,http/1.1,h3")))), "2");
    }

    #[test]
    fn dispatch_mirrors_decide_http_version_table() {
        // the full xray table in one sweep (dialer.go decideHTTPVersion)
        let cases: &[(Option<&SecurityConfig>, &str)] = &[
            (None, "1.1"),
            (Some(&reality()), "2"),
            (Some(&tls(None)), "2"),
            (Some(&tls(Some("http/1.1"))), "1.1"),
            (Some(&tls(Some("h3"))), "3"),
            (Some(&tls(Some("h2"))), "2"),
            (Some(&tls(Some("h2,http/1.1"))), "2"),
        ];
        for (sec, want) in cases {
            assert_eq!(http_version(*sec), *want);
        }
    }

    // --- SP5 T1 smoke: the quinn + h3 dep stack ---------------------------

    /// quinn server + client over loopback UDP, self-signed cert, the `h3`
    /// ALPN (spec §4.1): proves the stack compiles, negotiates TLS 1.3 +
    /// ALPN, and streams bytes. `h3`/`h3-quinn` compile as deps (the client
    /// arm consumes them in T2); this exercises the quinn transport their
    /// `h3-quinn` adapter wraps.
    #[tokio::test]
    async fn quic_loopback_smoke() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified =
            rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("cert");
        let cert_der = rustls::pki_types::CertificateDer::from(certified.cert.der().to_vec());
        let key_der =
            rustls::pki_types::PrivateKeyDer::try_from(certified.signing_key.serialize_der())
                .expect("key der");

        let mut server_tls = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der.clone()], key_der)
            .expect("server tls");
        server_tls.alpn_protocols = vec![b"h3".to_vec()];
        let server = quinn::Endpoint::server(
            quinn::ServerConfig::with_crypto(Arc::new(
                quinn::crypto::rustls::QuicServerConfig::try_from(server_tls)
                    .expect("quic server config"),
            )),
            "127.0.0.1:0".parse().expect("bind addr"),
        )
        .expect("quic server endpoint");
        let server_addr = server.local_addr().expect("server addr");

        let server_task = {
            let server = server.clone();
            tokio::spawn(async move {
                let conn = server
                    .accept()
                    .await
                    .expect("incoming conn")
                    .await
                    .expect("server handshake");
                let (mut send, mut recv) = conn.accept_bi().await.expect("accept_bi");
                let data = recv.read_to_end(usize::MAX).await.expect("read");
                send.write_all(&data).await.expect("echo write");
                send.finish().expect("finish");
                // Hold the connection until the client closes it: dropping it
                // here would abort the echoed stream before the client reads.
                conn.closed().await;
            })
        };

        let mut client_tls = rustls::ClientConfig::builder()
            .with_root_certificates({
                let mut roots = rustls::RootCertStore::empty();
                roots.add(cert_der).expect("root add");
                roots
            })
            .with_no_client_auth();
        client_tls.alpn_protocols = vec![b"h3".to_vec()];
        let client = quinn::Endpoint::client("127.0.0.1:0".parse().expect("bind addr"))
            .expect("quic client endpoint");
        let conn = client
            .connect_with(
                quinn::ClientConfig::new(Arc::new(
                    quinn::crypto::rustls::QuicClientConfig::try_from(client_tls)
                        .expect("quic client config"),
                )),
                server_addr,
                "localhost",
            )
            .expect("connect")
            .await
            .expect("client handshake");

        let (mut send, mut recv) = conn.open_bi().await.expect("open_bi");
        send.write_all(b"hello quinn h3").await.expect("write");
        send.finish().expect("finish");
        let echoed = recv.read_to_end(usize::MAX).await.expect("echo read");
        assert_eq!(echoed, b"hello quinn h3");

        conn.close(0u32.into(), b"done");
        client.wait_idle().await;
        server_task.await.expect("server task");
        server.wait_idle().await;
    }

    // --- SP5 T2: the h3 client arm ----------------------------------------

    /// An xhttp ctx with the h3 ALPN (spec §4.1 trigger) + insecure TLS
    /// (the loopback server's cert is self-signed — verification is T3's
    /// e2e concern).
    fn ctx_h3(addr: SocketAddr, mode: &str) -> LinkContext {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "x_http", "path": "/x", "host": "localhost", "mode": mode },
            "security": { "type": "tls", "alpn": "h3", "insecure": true }
        }))
        .expect("config parses");
        LinkContext::new(
            NativeConnectParams::new(
                protocol,
                EndpointEssentials::new(addr.ip().to_string(), addr.port()),
                TargetAddr::new(Host::Domain("dest.test".into()), 80),
            ),
            TargetAddr::new(Host::Domain("dest.test".into()), 80),
        )
    }

    // --- SP5 T3: the hermetic h3 server double (spec §7.2) ----------------

    /// Bounded server-side await: a hung client must fail the server task
    /// (and thus the test) instead of hanging forever.
    async fn bounded<T>(fut: impl Future<Output = T>, what: &'static str) -> T {
        tokio::time::timeout(H3_DEADLINE, fut)
            .await
            .unwrap_or_else(|_| panic!("fake h3 server: {what} timed out after {H3_DEADLINE:?}"))
    }

    /// Upper bound on every server-side await and the test-side observation
    /// wait (loopback QUIC round trips are sub-ms; 10 s is generous headroom
    /// for slow CI without letting a hung client hang the suite).
    const H3_DEADLINE: Duration = Duration::from_secs(10);

    /// The download body the fake streams on the session-open GET.
    const H3_DOWNLOAD: &[u8] = b"world";

    /// What the fake server observed, asserted on by the tests. Event-based:
    /// every record bumps [`H3Handle::changed`], and the tests wait on it
    /// under a deadline — never sleep-then-assert.
    #[derive(Debug, Default)]
    struct H3Obs {
        /// Session-open GETs: `(path, referer)` in arrival order.
        gets: Vec<(String, String)>,
        /// Packet-up POSTs: `(seq, body, referer, arrival)` in arrival order.
        posts: Vec<(u64, Bytes, String, Instant)>,
        /// Stream-up POSTs: `(referer, content-type)` in arrival order.
        stream_reqs: Vec<(String, String)>,
        /// Stream-up upload bodies (EOF is always observed — the fake reads
        /// each body to completion).
        stream_uploads: Vec<Bytes>,
        /// The h3 connection closed (the client is gone).
        conn_closed: bool,
    }

    /// Shared handle: the observations + the event-based wait primitive.
    /// The Notify lives OUTSIDE the mutex so a waiter never holds the lock
    /// across an await — holding it there would deadlock the server's
    /// `record` against the test's wait.
    struct H3Handle {
        data: Mutex<H3Obs>,
        changed: Notify,
    }

    /// Record one observation and wake the test waiter.
    fn record(obs: &Arc<H3Handle>, f: impl FnOnce(&mut H3Obs)) {
        let mut o = obs.data.lock().expect("obs lock");
        f(&mut o);
        obs.changed.notify_one();
    }

    /// Wait until `pred` holds over the observations, under [`H3_DEADLINE`]
    /// (event-driven via the Notify — the permit model makes a record
    /// between the check and the wait unmissable).
    async fn wait_obs(obs: &Arc<H3Handle>, pred: impl Fn(&H3Obs) -> bool) {
        loop {
            if pred(&obs.data.lock().expect("obs lock")) {
                return;
            }
            let notified = obs.changed.notified();
            assert!(
                tokio::time::timeout(H3_DEADLINE, notified).await.is_ok(),
                "fake h3 server: expected observation missed: {:#?}",
                *obs.data.lock().expect("obs lock")
            );
        }
    }

    /// Referer = `{scheme}://{host}{base}?x_padding={X..}` — `X`s in the
    /// xray `xPaddingBytes` [100, 1000] range (the v3 padding the client
    /// sends on every request).
    fn assert_referer(referer: &str) {
        let (url, pad) = referer
            .split_once("?x_padding=")
            .expect("referer carries x_padding");
        assert_eq!(url, "https://localhost/x/", "{referer}");
        assert!(pad.bytes().all(|b| b == b'X'), "{referer}");
        assert!((PAD_MIN..=PAD_MAX).contains(&pad.len()), "{referer}");
    }

    /// Read an h3 request body to EOF, concatenated. The client always
    /// finishes its upload bodies (packet POSTs are finite; the stream-up
    /// pipe ends when the tunnel writer drops), so EOF is the expected
    /// outcome — a stalled peer trips the outer `bounded` deadline.
    async fn read_h3_body(
        stream: &mut h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    ) -> Bytes {
        let mut body = Vec::new();
        while let Some(mut chunk) = stream.recv_data().await.expect("read request body") {
            body.extend_from_slice(chunk.chunk());
            chunk.advance(chunk.remaining());
        }
        Bytes::from(body)
    }

    /// Serve the v3 protocol over one h3 connection until the client closes
    /// it: session-open GETs, packet-up POSTs (seq in the path), stream-up
    /// POSTs (200 immediately — xray hub.go — then the piped body to EOF).
    async fn handle_h3_request(
        req: http::Request<()>,
        mut stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
        obs: &Arc<H3Handle>,
        reject_get: bool,
    ) {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let referer = req
            .headers()
            .get("referer")
            .map(|v| v.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        if method == http::Method::GET {
            // Session open: `{base}/{uuid}` — never a seq segment.
            let session = path.strip_prefix("/x/").expect("GET on session path");
            assert!(
                !session.contains('/'),
                "session-open GET must not carry a seq: {path}"
            );
            uuid::Uuid::parse_str(session).expect("session is a uuid v4");
            record(obs, |o| o.gets.push((path, referer)));
            let status = if reject_get {
                http::StatusCode::INTERNAL_SERVER_ERROR
            } else {
                http::StatusCode::OK
            };
            let resp = http::Response::builder().status(status).body(()).unwrap();
            bounded(stream.send_response(resp), "GET response")
                .await
                .expect("GET response");
            if !reject_get {
                bounded(
                    stream.send_data(Bytes::from_static(H3_DOWNLOAD)),
                    "GET body",
                )
                .await
                .expect("GET body");
            }
            bounded(stream.finish(), "GET finish")
                .await
                .expect("GET finish");
        } else if let Ok(seq) = path
            .rsplit('/')
            .next()
            .expect("POST path has a last segment")
            .parse::<u64>()
        {
            // Packet-up upload: `{base}/{uuid}/{seq}` — the raw body, 200.
            let body = bounded(read_h3_body(&mut stream), "upload body").await;
            record(obs, |o| o.posts.push((seq, body, referer, Instant::now())));
            let resp = http::Response::builder()
                .status(http::StatusCode::OK)
                .body(())
                .unwrap();
            bounded(stream.send_response(resp), "POST response")
                .await
                .expect("POST response");
            bounded(stream.finish(), "POST finish")
                .await
                .expect("POST finish");
        } else {
            // Stream-up upload: `{base}/{uuid}` — 200 immediately (xray
            // hub.go), then the piped body to EOF (the tunnel writer's
            // close ends it).
            let content_type = req
                .headers()
                .get("content-type")
                .map(|v| v.to_str().unwrap_or_default().to_string())
                .unwrap_or_default();
            record(obs, |o| o.stream_reqs.push((referer, content_type)));
            let resp = http::Response::builder()
                .status(http::StatusCode::OK)
                .body(())
                .unwrap();
            bounded(stream.send_response(resp), "stream-up 200")
                .await
                .expect("stream-up 200");
            let body = bounded(read_h3_body(&mut stream), "stream-up body").await;
            record(obs, |o| o.stream_uploads.push(body));
            bounded(stream.finish(), "stream-up finish")
                .await
                .expect("stream-up finish");
        }
    }

    /// The fake server loop: one quinn connection (the tests open exactly
    /// one), the h3 server connection over it, and the request loop until
    /// the client closes. Every await is bounded.
    async fn serve_h3(server: quinn::Endpoint, obs: Arc<H3Handle>, reject_get: bool) {
        let incoming = bounded(server.accept(), "accept conn")
            .await
            .expect("incoming quinn conn");
        let conn = bounded(incoming.into_future(), "conn handshake")
            .await
            .expect("server handshake");
        let h3_quic = h3_quinn::Connection::new(conn);
        let mut h3_conn = bounded(h3::server::Connection::new(h3_quic), "h3 open")
            .await
            .expect("h3 server open");
        loop {
            // The client closes the h3 connection (H3_NO_ERROR after the
            // last request stream dropped) → accept yields `None`/`Err`.
            let Some(resolver) = bounded(h3_conn.accept(), "accept request")
                .await
                .ok()
                .flatten()
            else {
                record(&obs, |o| o.conn_closed = true);
                break;
            };
            let (req, stream) = bounded(resolver.resolve_request(), "resolve request")
                .await
                .expect("resolve request");
            handle_h3_request(req, stream, &obs, reject_get).await;
        }
    }

    /// Bind + spawn the fake h3 server (self-signed cert, `h3` ALPN —
    /// spec §4.1) over loopback QUIC, returning the socket address + the
    /// shared observations + the server task.
    ///
    /// Cert story: the client's `insecure` TLS mode (the `SkipServerVerify`
    /// seam — chain walk skipped, handshake signatures still verified)
    /// accepts the self-signed loopback cert in the hermetic tests; the
    /// webpki-roots path is the T4 e2e's job (the real xray server's own
    /// cert — there is no CA-injection hook in `quic_tls_config`).
    ///
    /// `reject_get` makes the server 500 the session-open GET (the error
    /// path test).
    fn spawn_h3_fake(reject_get: bool) -> (SocketAddr, Arc<H3Handle>, JoinHandle<()>) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified =
            rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("cert");
        let cert_der = rustls::pki_types::CertificateDer::from(certified.cert.der().to_vec());
        let key_der =
            rustls::pki_types::PrivateKeyDer::try_from(certified.signing_key.serialize_der())
                .expect("key der");
        let mut server_tls = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .expect("server tls");
        server_tls.alpn_protocols = vec![b"h3".to_vec()];
        let server = quinn::Endpoint::server(
            quinn::ServerConfig::with_crypto(Arc::new(
                quinn::crypto::rustls::QuicServerConfig::try_from(server_tls)
                    .expect("quic server config"),
            )),
            "127.0.0.1:0".parse().expect("bind addr"),
        )
        .expect("quic server endpoint");
        let addr = server.local_addr().expect("server addr");
        let obs = Arc::new(H3Handle {
            data: Mutex::new(H3Obs::default()),
            changed: Notify::new(),
        });
        let handle = tokio::spawn(serve_h3(server, obs.clone(), reject_get));
        (addr, obs, handle)
    }

    /// Assert the fake's observations for one full packet-up round trip:
    /// one session-open GET (uuid path, no seq, Referer) + one packet POST
    /// (seq 0, raw body, Referer). Sync helper — keeps the `MutexGuard` out
    /// of the async test bodies.
    fn assert_packet_roundtrip(o: &H3Obs) {
        assert_eq!(o.gets.len(), 1, "{o:#?}");
        let (get_path, get_referer) = &o.gets[0];
        assert!(get_path.starts_with("/x/"), "{get_path}");
        uuid::Uuid::parse_str(&get_path["/x/".len()..]).expect("GET session uuid");
        assert_referer(get_referer);
        assert_eq!(o.posts.len(), 1, "{o:#?}");
        let (seq, body, post_referer, _arrival) = &o.posts[0];
        assert_eq!(*seq, 0);
        assert_eq!(&body[..], &b"hello"[..]);
        assert_referer(post_referer);
    }

    /// Assert the chunked-upload observations: two packet POSTs, seqs 0 + 1
    /// in arrival order, each ≤ the 1 MB cap, concatenated == the payload,
    /// paced ≥ 25 ms apart, both carrying the Referer.
    fn assert_chunked_uploads(o: &H3Obs, payload: &[u8]) {
        assert_eq!(o.posts.len(), 2, "{o:#?}");
        let (s0, b0, r0, t0) = &o.posts[0];
        let (s1, b1, r1, t1) = &o.posts[1];
        assert_eq!(*s0, 0, "seq order");
        assert_eq!(*s1, 1, "seq order");
        assert_eq!(b0.len(), MAX_POST_BYTES, "first chunk = the size cap");
        assert_eq!(b1.len(), 200_000, "tail chunk");
        let mut joined = Vec::with_capacity(b0.len() + b1.len());
        joined.extend_from_slice(b0);
        joined.extend_from_slice(b1);
        assert_eq!(joined, payload, "no byte loss across the split");
        assert_referer(r0);
        assert_referer(r1);
        assert!(
            t1.duration_since(*t0) >= POST_INTERVAL.saturating_sub(Duration::from_millis(5)),
            "posts must be paced ≥ 25 ms apart: {t0:?} -> {t1:?}"
        );
    }

    /// Assert the stream-up observations: session-open GET with Referer, one
    /// pipe POST (application/grpc + Referer, body delivered intact).
    fn assert_stream_up_roundtrip(o: &H3Obs) {
        assert_eq!(o.gets.len(), 1, "{o:#?}");
        assert_referer(&o.gets[0].1);
        assert_eq!(o.stream_reqs.len(), 1, "{o:#?}");
        let (referer, content_type) = &o.stream_reqs[0];
        assert_eq!(content_type, "application/grpc");
        assert_referer(referer);
        assert_eq!(o.stream_uploads.len(), 1, "{o:#?}");
        assert_eq!(
            &o.stream_uploads[0][..],
            &b"hello"[..],
            "piped body delivered intact"
        );
    }

    /// Full packet-up round trip: `connect_quic` dials the loopback quinn
    /// server, negotiates TLS 1.3 + the h3 ALPN, opens the v3 session (GET
    /// `/{uuid}` → 200), uploads via the first packet POST (seq 0, raw
    /// body), and downloads from the GET body. The fake's observations pin
    /// the wire details: the session path is a uuid (no seq on the GET), the
    /// Referer carries the base URL + `x_padding`, and the client closes the
    /// h3 connection cleanly afterwards.
    #[tokio::test]
    async fn h3_loopback_get_and_post() {
        let (addr, obs, server_task) = spawn_h3_fake(false);
        let ctx = ctx_h3(addr, "packet-up");
        let mut t = connect_quic(&ctx).await.expect("h3 connect");
        t.write_all(b"hello").await.expect("upload write");
        let mut out = [0u8; H3_DOWNLOAD.len()];
        t.read_exact(&mut out).await.expect("download read");
        assert_eq!(&out[..], H3_DOWNLOAD);
        drop(t);

        wait_obs(&obs, |o| o.conn_closed).await;
        assert_packet_roundtrip(&obs.data.lock().expect("obs lock"));
        server_task.await.expect("server task");
    }

    /// Packet-up chunking + seq order + pacing: a > 1 MB tunnel write is
    /// split at `scMaxEachPostBytes` into consecutive seqs, each POST ≤
    /// 1 MB, paced ≥ `scMinPostsIntervalMs` apart (the first rides the idle
    /// window), in arrival order, each carrying the Referer. The split and
    /// the 30 ms sleep are client-side guarantees, so the assertions are
    /// lower bounds — deterministic, no flake.
    #[tokio::test]
    async fn h3_packet_up_seq_order_and_chunking() {
        let (addr, obs, server_task) = spawn_h3_fake(false);
        let ctx = ctx_h3(addr, "packet-up");
        let mut t = connect_quic(&ctx).await.expect("h3 connect");

        // 1.2 MB in one write: the upload loop splits it into a 1 MB +
        // 200 KB pair (the server 413s anything over the size cap).
        let payload = vec![0xABu8; MAX_POST_BYTES + 200_000];
        t.write_all(&payload).await.expect("upload write");
        let mut out = [0u8; H3_DOWNLOAD.len()];
        t.read_exact(&mut out).await.expect("download read");
        assert_eq!(&out[..], H3_DOWNLOAD);
        drop(t);

        // Both POSTs arrive (the channel-close flushes the tail after the
        // tunnel dropped), then the h3 connection closes.
        wait_obs(&obs, |o| o.posts.len() == 2 && o.conn_closed).await;
        assert_chunked_uploads(&obs.data.lock().expect("obs lock"), &payload);
        server_task.await.expect("server task");
    }

    /// The h3 stream-up path: session open (GET), one long-lived POST with
    /// a piped body — 200 immediately (xray hub.go), `Content-Type:
    /// application/grpc`, Referer present. The tunnel writer's drop ends
    /// the pipe: the fake reads the body to EOF, then the h3 connection
    /// closes.
    #[tokio::test]
    async fn h3_loopback_stream_up() {
        let (addr, obs, server_task) = spawn_h3_fake(false);
        let ctx = ctx_h3(addr, "stream-up");
        let mut t = connect_quic(&ctx).await.expect("h3 stream-up connect");
        t.write_all(b"hello").await.expect("upload write");
        let mut out = [0u8; H3_DOWNLOAD.len()];
        t.read_exact(&mut out).await.expect("download read");
        assert_eq!(&out[..], H3_DOWNLOAD);
        drop(t);

        wait_obs(&obs, |o| !o.stream_uploads.is_empty() && o.conn_closed).await;
        assert_stream_up_roundtrip(&obs.data.lock().expect("obs lock"));
        server_task.await.expect("server task");
    }

    /// Error path: a non-200 session-open GET must surface as a transport
    /// error from `connect_quic` (the v3 contract requires 200 everywhere),
    /// and the aborted session still closes the h3 connection.
    #[tokio::test]
    async fn h3_session_open_reject_surfaces_error() {
        let (addr, obs, server_task) = spawn_h3_fake(true);
        let ctx = ctx_h3(addr, "packet-up");
        let Err(err) = connect_quic(&ctx).await else {
            panic!("a 500 session open must fail connect_quic")
        };
        assert!(
            matches!(err, NativeError::Transport(ref m) if m.contains("expected 200, got 500")),
            "{err}"
        );
        wait_obs(&obs, |o| o.conn_closed).await;
        server_task.await.expect("server task");
    }

    #[tokio::test]
    async fn connect_quic_rejects_unsupported_modes_before_dialing() {
        // The mode check precedes the quinn dial: stream-one (legacy XHTTP
        // v1) and unknown modes error without any network step.
        let ctx = ctx_h3("127.0.0.1:1".parse().unwrap(), "stream-one");
        let Err(err) = connect_quic(&ctx).await else {
            panic!("stream-one must be rejected")
        };
        assert!(matches!(err, NativeError::NotImplemented { .. }), "{err}");
        let ctx = ctx_h3("127.0.0.1:1".parse().unwrap(), "bogus");
        let Err(err) = connect_quic(&ctx).await else {
            panic!("unknown mode must be rejected")
        };
        assert!(matches!(err, NativeError::Config(_)), "{err}");
    }

    #[tokio::test]
    async fn transport_connect_h3_with_base_is_config_error() {
        // xhttp+h3 as a later chain hop (base = Some) is forbidden: the
        // QUIC dial is a fresh connection that never reuses a base tunnel.
        let ctx = ctx_h3("127.0.0.1:1".parse().unwrap(), "packet-up");
        let Err(err) = crate::transport::connect(&ctx, Some(Box::new(tokio::io::empty()))).await
        else {
            panic!("base reuse must be rejected")
        };
        assert!(matches!(err, NativeError::Config(_)), "{err}");
    }

    #[test]
    fn self_contained_only_for_the_h3_arm() {
        let h3 = ctx_h3("127.0.0.1:1".parse().unwrap(), "packet-up");
        assert!(crate::transport::is_self_contained(&h3));
        let h1 = ctx_at("127.0.0.1:1".parse().unwrap(), "packet-up");
        assert!(!crate::transport::is_self_contained(&h1));
    }
}
