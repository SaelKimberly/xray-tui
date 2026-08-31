//! Hysteria2 QUIC dial + auth + TCP frame handshake.
//!
//! Layering (mirror of the h3 arm + hysteria2 spec §5.2): the QUIC dial
//! replaces dial + security + upgrade entirely (quinn's rustls is internal).
//! Salamander packet obfuscation wraps the UDP socket when configured.

use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::Bytes;
use http::Request;
use ring::rand::{SecureRandom, SystemRandom};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use xray_tui_proto::proto_spec::Hysteria2Config;

use crate::addr::TargetAddr;
use crate::context::LinkContext;
use crate::crypto::salamander::{SALT_LEN, Salamander};
use crate::error::{NativeError, timeouts};

use super::{ALPN_H3, FRAME_TYPE_TCP_REQUEST, MAX_ADDRESS_LEN, TCP_RESPONSE_OK};

// ── randomness ────────────────────────────────────────────────────────────

fn random_bytes(buf: &mut [u8]) {
    SystemRandom::new()
        .fill(buf)
        .expect("system rng is infallible");
}

/// A fresh random salt for one salamander packet.
fn random_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    random_bytes(&mut salt);
    salt
}

/// Random alphanumeric padding in `[min, max)` — the wire value is
/// arbitrary, only its length matters. Fills the whole payload with ONE
/// `getrandom` call, not one per character.
fn random_padding(min: usize, max: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut len_bytes = [0u8; 8];
    random_bytes(&mut len_bytes);
    let span = max - min;
    let len = min + (usize::from_le_bytes(len_bytes) % span);
    let mut buf = vec![0u8; len];
    random_bytes(&mut buf);
    buf.iter()
        .map(|b| char::from(CHARS[usize::from(*b) % CHARS.len()]))
        .collect()
}

// ── QUIC varint (RFC 9000 §16) ───────────────────────────────────────────

/// Encode `value` as a QUIC varint: 1/2/4/8 bytes, 2-bit length prefix.
/// The range guards below guarantee the value fits the cast target width.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
fn varint(value: u64) -> Vec<u8> {
    match value {
        0..=63 => vec![value as u8],
        64..=16_383 => (0x4000_u16 | value as u16).to_be_bytes().to_vec(),
        16_384..=1_073_741_823 => (0x8000_0000_u32 | value as u32).to_be_bytes().to_vec(),
        _ => (0xc000_0000_0000_0000_u64 | value).to_be_bytes().to_vec(),
    }
}

/// Read one QUIC varint from `src`, returning `(value, remaining)`.
fn read_varint(src: &[u8]) -> Option<(u64, &[u8])> {
    let first = *src.first()?;
    let len = match first >> 6 {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 8,
    };
    if src.len() < len {
        return None;
    }
    let mut v = 0u64;
    for b in &src[..len] {
        v = (v << 8) | u64::from(*b);
    }
    let mask = match len {
        1 => 0x3f_u64,
        2 => 0x3fff,
        4 => 0x3fff_ffff,
        _ => 0x3fff_ffff_ffff_ffff,
    };
    Some((v & mask, &src[len..]))
}

/// A destination in the Socksaddr form hysteria2 uses on the wire
/// (`M.Socksaddr.String()`): `host:port` / `[v6]:port`.
fn target_string(target: &TargetAddr) -> String {
    use crate::addr::Host;
    match &target.host {
        Host::Domain(d) => format!("{d}:{}", target.port),
        Host::Ip(std::net::IpAddr::V4(ip)) => format!("{ip}:{}", target.port),
        Host::Ip(std::net::IpAddr::V6(ip)) => format!("[{ip}]:{}", target.port),
    }
}

// ── dial + auth ───────────────────────────────────────────────────────────

/// The established hysteria2 session: the quinn connection plus the h3
/// keep-alive that keeps it alive.
///
/// The server runs the h3 `StreamDispatcher` (quic-go `http3.Server`) over
/// the SAME quinn connection that carried the auth request. Dropping the
/// last `SendRequest` clone closes the h3 session, which sends
/// `CONNECTION_CLOSE` (`H3_NO_ERROR`) and tears the whole quinn connection
/// down — so the h3 handle must outlive the tunnel (mirror of the xhttp h3
/// arm's keepalive).
pub(super) struct H3Keepalive(
    #[allow(dead_code)] // RAII guard: only its Drop matters
    h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>,
);

/// Dial the hysteria2 server (QUIC, ALPN `h3`), run the auth exchange and
/// return the live session (quinn connection + h3 keep-alive).
pub(super) async fn dial(
    ctx: &LinkContext,
    cfg: &Hysteria2Config,
) -> Result<(quinn::Connection, H3Keepalive), NativeError> {
    let server_addr = ctx.server_socket().await?;
    let endpoint = client_endpoint(server_addr, cfg.obfs_password.as_deref())?;
    let tls = crate::transport::quic::quic_tls_config(ctx, ALPN_H3)?;
    let quic_config = crate::transport::quic::quinn_client_config(tls);
    let conn = crate::transport::quic::connect(
        &endpoint,
        quic_config,
        server_addr,
        &ctx.server_name(),
        "hysteria2",
    )
    .await?;

    let keepalive = runtime_auth(&conn, &endpoint, &cfg.auth).await?;
    Ok((conn, keepalive))
}

/// The h3 auth exchange: login the connection, verify status 233.
async fn runtime_auth(
    conn: &quinn::Connection,
    endpoint: &quinn::Endpoint,
    auth: &str,
) -> Result<H3Keepalive, NativeError> {
    let h3_quic = h3_quinn::Connection::new(conn.clone());
    let (mut h3_conn, mut sender) = {
        let limit = timeouts::TRANSPORT;
        tokio::time::timeout(limit, h3::client::new(h3_quic))
            .await
            .map_err(|_| NativeError::Timeout {
                step: "hysteria2 h3 open",
                limit,
            })?
            .map_err(|e| NativeError::Transport(format!("hysteria2 h3 open: {e}")))?
    };
    // Drive the h3 connection in the background; keep the endpoint alive for
    // the tunnel's lifetime.
    let endpoint = endpoint.clone();
    tokio::spawn(async move {
        let _ = std::future::poll_fn(|cx| {
            let mut conn = Pin::new(&mut h3_conn);
            conn.as_mut().poll_close(cx)
        })
        .await;
        drop(endpoint);
    });

    let mut req = Request::builder()
        .method(http::method::Method::POST)
        .uri("https://hysteria/auth");
    req = req.header("hysteria-auth", auth);
    // cc-rx: client's max receive bandwidth. Congestion control stays on
    // quinn's defaults (no brutal/BBR port), so advertise 0 = "server runs
    // bandwidth detection".
    req = req.header("hysteria-cc-rx", "0");
    req = req.header("hysteria-padding", &random_padding(256, 2048));
    let request = req
        .body(())
        .map_err(|e| NativeError::Transport(format!("hysteria2 auth request: {e}")))?;

    let mut stream = sender
        .send_request(request)
        .await
        .map_err(|e| NativeError::Transport(format!("hysteria2 auth send: {e}")))?;
    stream
        .finish()
        .await
        .map_err(|e| NativeError::Transport(format!("hysteria2 auth finish: {e}")))?;
    let resp = stream
        .recv_response()
        .await
        .map_err(|e| NativeError::Transport(format!("hysteria2 auth response: {e}")))?;
    drop(stream);

    let status = resp.status();
    if status != http::StatusCode::from_u16(233).expect("233 is a valid status") {
        return Err(NativeError::Protocol {
            kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
            detail: format!("authentication failed, status code: {status}"),
        });
    }
    // The h3 connection stays open only while a `SendRequest` clone is
    // alive: return one so the tunnel's lifetime keeps the session (and the
    // underlying quinn connection) alive.
    Ok(H3Keepalive(sender))
}

/// Bind the QUIC client endpoint. With `obfs_password` the UDP socket is
/// wrapped in a Salamander obfuscator; otherwise a plain socket.
fn client_endpoint(
    server_addr: SocketAddr,
    obfs_password: Option<&str>,
) -> Result<quinn::Endpoint, NativeError> {
    let bind: SocketAddr = if server_addr.is_ipv6() {
        "[::]:0".parse().expect("static ipv6 bind addr")
    } else {
        "0.0.0.0:0".parse().expect("static ipv4 bind addr")
    };

    let Some(obfs) = obfs_password else {
        return Ok(crate::transport::quic::client_endpoint(server_addr));
    };
    let sal = Salamander::new(obfs.as_bytes())?;
    let runtime = quinn::default_runtime()
        .ok_or_else(|| NativeError::Dial("no quinn runtime configured".into()))?;
    let inner_u8 = std::net::UdpSocket::bind(bind)?;
    let inner = runtime.wrap_udp_socket(inner_u8)?;
    let sock = Arc::new(SalamanderSocket { inner, sal });
    Ok(quinn::Endpoint::new_with_abstract_socket(
        quinn::EndpointConfig::default(),
        None,
        sock,
        runtime,
    )?)
}

/// A UDP socket that obfuscates every QUIC datagram with Salamander.
struct SalamanderSocket {
    inner: Arc<dyn quinn::AsyncUdpSocket>,
    sal: Salamander,
}

impl std::fmt::Debug for SalamanderSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SalamanderSocket")
            .field("sal", &"<salamander obfuscator>")
            .finish_non_exhaustive()
    }
}

impl quinn::AsyncUdpSocket for SalamanderSocket {
    fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn quinn::UdpPoller>> {
        self.inner.clone().create_io_poller()
    }

    fn try_send(&self, transmit: &quinn::udp::Transmit) -> io::Result<()> {
        // Reuse a thread-local scratch buffer to avoid per-datagram
        // allocation on the hot path (QUIC datagrams are sent frequently).
        thread_local! {
            static SEND_SCRATCH: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
        }
        let len = SALT_LEN + transmit.contents.len();
        SEND_SCRATCH.with(|scratch| {
            let mut buf = scratch.borrow_mut();
            buf.resize(len, 0);
            let salt = random_salt();
            self.sal.obfuscate_into(&salt, transmit.contents, &mut buf);
            let ob = quinn::udp::Transmit {
                destination: transmit.destination,
                ecn: transmit.ecn,
                contents: &buf[..len],
                segment_size: transmit.segment_size,
                src_ip: transmit.src_ip,
            };
            self.inner.try_send(&ob)
        })
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [io::IoSliceMut<'_>],
        meta: &mut [quinn::udp::RecvMeta],
    ) -> Poll<io::Result<usize>> {
        let ready = self.inner.poll_recv(cx, bufs, meta);
        if let Poll::Ready(Ok(n)) = ready {
            for i in 0..n {
                let len = meta[i].len;
                if len <= SALT_LEN {
                    meta[i].len = 0;
                    continue;
                }
                // Copy the datagram out first: source and destination alias
                // the same `IoSliceMut`.
                let mut scratch = vec![0u8; len];
                scratch.copy_from_slice(&bufs[i][..len]);
                let plain = self.sal.deobfuscate_into(&scratch, &mut bufs[i][..]);
                meta[i].len = plain;
            }
        }
        ready
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    fn max_transmit_segments(&self) -> usize {
        1
    }

    fn max_receive_segments(&self) -> usize {
        1
    }

    fn may_fragment(&self) -> bool {
        false
    }
}

// ── TCP tunnel ────────────────────────────────────────────────────────────

/// Open one TCP tunnel over a fresh QUIC bidi stream: write the `TCPRequest`
/// frame, read + validate the `TCPResponse`, then return the relay stream.
pub(super) async fn open_tcp_tunnel(
    conn: &quinn::Connection,
    keepalive: H3Keepalive,
    target: &TargetAddr,
) -> Result<H2Stream, NativeError> {
    let (send, recv) = {
        let limit = timeouts::PROTOCOL;
        tokio::time::timeout(limit, conn.open_bi())
            .await
            .map_err(|_| NativeError::Timeout {
                step: "hysteria2 open stream",
                limit,
            })?
            .map_err(|e| NativeError::Protocol {
                kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
                detail: format!("open stream: {e}"),
            })?
    };

    // TCPRequest: `0x401 | addrLen | addr | padLen | padding`.
    let addr = target_string(target);
    let padding = random_padding(64, 512);
    let mut frame = varint(FRAME_TYPE_TCP_REQUEST);
    frame.extend_from_slice(&varint(addr.len() as u64));
    frame.extend_from_slice(addr.as_bytes());
    frame.extend_from_slice(&varint(padding.len() as u64));
    frame.extend_from_slice(padding.as_bytes());

    let mut send = send;
    let limit = timeouts::PROTOCOL;
    tokio::time::timeout(
        limit,
        tokio::io::AsyncWriteExt::write_all(&mut send, &frame),
    )
    .await
    .map_err(|_| NativeError::Timeout {
        step: "hysteria2 tcp request write",
        limit,
    })?
    .map_err(|e| NativeError::Protocol {
        kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
        detail: format!("write tcp request: {e}"),
    })?;

    // TCPResponse: `status byte | msgLen | msg | padLen | padding`.
    let mut recv = recv;
    let mut head = [0u8; 1];
    let limit = timeouts::PROTOCOL;
    tokio::time::timeout(
        limit,
        tokio::io::AsyncReadExt::read_exact(&mut recv, &mut head),
    )
    .await
    .map_err(|_| NativeError::Timeout {
        step: "hysteria2 tcp response read",
        limit,
    })?
    .map_err(|e| NativeError::Protocol {
        kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
        detail: format!("read tcp response status: {e}"),
    })?;

    // Message + padding are always present, on both the OK and error paths
    // (`WriteTCPResponse` emits `status | msgLen | msg | padLen | padding`).
    let msg = read_vstring(&mut recv, "hysteria2 tcp response").await?;
    read_vstring(&mut recv, "hysteria2 tcp response padding").await?;

    if head[0] != TCP_RESPONSE_OK {
        return Err(NativeError::Protocol {
            kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
            detail: format!("remote error: {msg}"),
        });
    }

    Ok(H2Stream::new(conn.clone(), keepalive, send, recv))
}

/// Read a single byte from the stream, bounded and EOF-mapped.
async fn read_exact_u8(
    recv: &mut quinn::RecvStream,
    step: &'static str,
) -> Result<u8, NativeError> {
    let limit = timeouts::PROTOCOL;
    let mut b = [0u8; 1];
    tokio::time::timeout(limit, tokio::io::AsyncReadExt::read_exact(recv, &mut b))
        .await
        .map_err(|_| NativeError::Timeout { step, limit })?
        .map_err(|e| NativeError::Protocol {
            kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
            detail: format!("{step}: {e}"),
        })?;
    Ok(b[0])
}

/// Read a QUIC varint from the stream: one leading byte selects the width
/// (1/2/4/8 by the top two bits), then the remaining bytes complete it.
async fn read_varint_stream(
    recv: &mut quinn::RecvStream,
    step: &'static str,
) -> Result<u64, NativeError> {
    let first = read_exact_u8(recv, step).await?;
    let width = match first >> 6 {
        0 => 1usize,
        1 => 2,
        2 => 4,
        _ => 8,
    };
    let mut raw = vec![first];
    for _ in 1..width {
        raw.push(read_exact_u8(recv, step).await?);
    }
    read_varint(&raw)
        .map(|(v, _)| v)
        .ok_or_else(|| NativeError::Protocol {
            kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
            detail: format!("{step}: malformed length"),
        })
}

/// Read a varint-length-prefixed string (message or padding) and return it
/// as owned bytes. Bounded by the protocol overflow guards.
async fn read_vstring(
    recv: &mut quinn::RecvStream,
    step: &'static str,
) -> Result<String, NativeError> {
    let limit = timeouts::PROTOCOL;
    let len_u64 = read_varint_stream(recv, step).await?;
    let len = usize::try_from(len_u64).map_err(|_| NativeError::Protocol {
        kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
        detail: format!("{step}: length overflow"),
    })?;
    if len > MAX_ADDRESS_LEN {
        return Err(NativeError::Protocol {
            kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
            detail: format!("{step}: invalid length {len}"),
        });
    }
    let mut buf = vec![0u8; len];
    if len > 0 {
        tokio::time::timeout(limit, tokio::io::AsyncReadExt::read_exact(recv, &mut buf))
            .await
            .map_err(|_| NativeError::Timeout { step, limit })?
            .map_err(|e| NativeError::Protocol {
                kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
                detail: format!("{step}: {e}"),
            })?;
    }
    String::from_utf8(buf).map_err(|_| NativeError::Protocol {
        kind: xray_tui_proto::proto_spec::ProtocolKind::Hysteria2,
        detail: format!("{step}: non-utf8 payload"),
    })
}

/// The hysteria2 TCP tunnel: a raw quinn bidi stream, kept alive by the
/// connection and h3-session handles it carries (dropping either closes the
/// underlying quinn connection).
pub(super) struct H2Stream {
    _conn: quinn::Connection,
    /// Keeps the h3 session alive (dropping it closes the quinn connection).
    _h3: H3Keepalive,
    send: quinn::SendStream,
    recv: quinn::RecvStream,
}

impl H2Stream {
    const fn new(
        conn: quinn::Connection,
        keepalive: H3Keepalive,
        send: quinn::SendStream,
        recv: quinn::RecvStream,
    ) -> Self {
        Self {
            _conn: conn,
            _h3: keepalive,
            send,
            recv,
        }
    }
}

impl AsyncRead for H2Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        tokio::io::AsyncRead::poll_read(Pin::new(&mut self.recv), cx, buf)
    }
}

impl AsyncWrite for H2Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(Pin::new(&mut self.send), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(Pin::new(&mut self.send), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(Pin::new(&mut self.send), cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::Host;

    #[test]
    fn varint_encoding() {
        assert_eq!(varint(0), vec![0]);
        assert_eq!(varint(63), vec![63]);
        assert_eq!(varint(64), vec![0x40, 0x40]);
        assert_eq!(varint(16_383), vec![0x7f, 0xff]);
        assert_eq!(varint(16_384), vec![0x80, 0x00, 0x40, 0x00]);
        // 0x401 = 1025: 2-byte form, high byte 0x40 | (1025 >> 8) = 0x44.
        assert_eq!(varint(FRAME_TYPE_TCP_REQUEST), vec![0x44, 0x01]);
    }

    #[test]
    fn varint_roundtrip() {
        for v in [
            0u64,
            1,
            63,
            64,
            16_383,
            16_384,
            1_073_741_823,
            1_073_741_824,
            0x3fff_ffff_ffff_ffff, // 2^62 - 1, the varint max
        ] {
            let enc = varint(v);
            let (got, rest) = read_varint(&enc).expect("decodes");
            assert_eq!(got, v);
            assert!(rest.is_empty());
        }
    }

    #[test]
    fn target_string_formats() {
        assert_eq!(
            target_string(&TargetAddr::new(Host::Domain("example.com".into()), 443)),
            "example.com:443"
        );
        assert_eq!(
            target_string(&TargetAddr::new(Host::Ip("1.2.3.4".parse().unwrap()), 80)),
            "1.2.3.4:80"
        );
        assert_eq!(
            target_string(&TargetAddr::new(Host::Ip("::1".parse().unwrap()), 443)),
            "[::1]:443"
        );
    }

    #[test]
    fn padding_len_in_range() {
        let p = random_padding(256, 2048);
        assert!((256..2048).contains(&p.len()), "len {}", p.len());
        assert!(p.bytes().all(|b| b.is_ascii_alphanumeric()));
        let p = random_padding(64, 512);
        assert!((64..512).contains(&p.len()));
    }
}
