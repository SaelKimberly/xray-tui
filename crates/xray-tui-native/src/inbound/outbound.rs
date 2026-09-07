//! Outbounds for the SOCKS5 inbound: tagged destinations the router may
//! select (`Decision::Route { tag }` → [`Outbound`]).
//!
//! Composition mirrors xray/sing-box: the "proxy" outbound reuses
//! [`crate::connect`] — the native client tunnel — so a SOCKS5 inbound in
//! front of a VLESS/VMess/... proxy is the in-process equivalent of a local
//! SOCKS5 listener wired to a remote proxy.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use xray_tui_proto::proto_spec::ProtocolConfig;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

use crate::BoxStream;
use crate::addr::{Host, TargetAddr};
use crate::context::NativeConnectParams;
use crate::error::{NativeError, timeouts};

/// A tagged outbound the router may forward to.
#[derive(Debug, Clone)]
pub struct Outbound {
    pub tag: String,
    pub kind: OutboundKind,
}

/// How an outbound reaches the destination.
#[derive(Debug, Clone)]
pub enum OutboundKind {
    /// Dial the destination directly (no proxy).
    Direct,
    /// Refuse connections routed here (the handler answers SOCKS5 0x02).
    Block,
    /// Tunnel through a native proxy client ([`crate::connect`]).
    Proxy(Box<ProxyOutbound>),
}

/// A proxy outbound: the typed protocol config plus the proxy server.
#[derive(Debug, Clone)]
pub struct ProxyOutbound {
    pub protocol: ProtocolConfig,
    pub server: EndpointEssentials,
    /// Pre-resolved proxy server address; `None` = resolve/DNS in
    /// [`crate::connect`].
    pub resolved_ip: Option<SocketAddr>,
}

/// Build the per-connection [`NativeConnectParams`] for a proxy outbound.
///
/// Pure construction (no I/O) — unit-tested independently of [`dial`].
#[must_use]
pub fn proxy_params(proxy: &ProxyOutbound, target: &TargetAddr) -> NativeConnectParams {
    let mut params =
        NativeConnectParams::new(proxy.protocol.clone(), proxy.server.clone(), target.clone());
    params.resolved_ip = proxy.resolved_ip;
    params
}

/// Open the outbound byte stream to `target`.
pub(crate) async fn dial(
    kind: &OutboundKind,
    target: &TargetAddr,
) -> Result<BoxStream, NativeError> {
    match kind {
        OutboundKind::Direct => Ok(Box::new(dial_direct(target).await?)),
        OutboundKind::Block => Err(NativeError::Config("block outbound cannot dial".into())),
        OutboundKind::Proxy(proxy) => {
            Ok(Box::new(crate::connect(proxy_params(proxy, target)).await?))
        }
    }
}

/// Dial `target` directly over TCP, resolving domains locally.
///
/// Failures keep BOTH halves of the diagnosis: the `io::ErrorKind` (which
/// [`crate::inbound::reply_for`] maps to an RFC 1928 reply code) and the
/// destination in the message. A bare `io::Error` would drop the address;
/// [`NativeError::Dial`] would drop the kind.
async fn dial_direct(target: &TargetAddr) -> Result<TcpStream, NativeError> {
    let addr = match &target.host {
        Host::Ip(ip) => SocketAddr::new(*ip, target.port),
        Host::Domain(domain) => {
            let mut addrs = tokio::time::timeout(
                timeouts::DIAL,
                tokio::net::lookup_host((domain.as_str(), target.port)),
            )
            .await
            .map_err(|_| NativeError::Timeout {
                step: "direct dns lookup",
                limit: timeouts::DIAL,
            })?
            // `getaddrinfo` failures surface as `ErrorKind::Uncategorized`,
            // which would answer `0x01 General failure`; an unresolvable name
            // is `0x04 Host unreachable` (RFC 1928 §6).
            .map_err(|e| {
                NativeError::Io(std::io::Error::new(
                    std::io::ErrorKind::HostUnreachable,
                    format!("{domain}: {e}"),
                ))
            })?;
            addrs.next().ok_or_else(|| {
                NativeError::Io(std::io::Error::new(
                    std::io::ErrorKind::HostUnreachable,
                    format!("{domain}: no addresses found"),
                ))
            })?
        }
    };
    let stream = tokio::time::timeout(timeouts::DIAL, TcpStream::connect(addr))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "direct connect",
            limit: timeouts::DIAL,
        })?
        .map_err(|e| NativeError::Io(std::io::Error::new(e.kind(), format!("{addr}: {e}"))))?;
    // Disable Nagle (Go cores set TCP_NODELAY by default); direct relays must
    // not add ~200ms buffering latency to interactive traffic. A socket-option
    // failure never invalidates a connection that is already up.
    if let Err(error) = stream.set_nodelay(true) {
        tracing::debug!(%addr, %error, "direct outbound: set_nodelay failed");
    }
    Ok(stream)
}

/// One 16 KiB buffer per relay direction, reused for every chunk.
const RELAY_BUF: usize = 16 * 1024;

/// The tunnel's shared idle deadline.
///
/// [`timeouts::TUNNEL_READ`] bounds inactivity of the TUNNEL, not of one
/// direction: a download goes quiet on the uplink for as long as it lasts.
/// The previous `select!`-per-iteration shape got this for free (it rebuilt
/// both reads, and therefore both timeouts, after activity in either
/// direction); with one future per direction the stamp has to be explicit.
struct Idle {
    base: tokio::time::Instant,
    /// Nanoseconds since `base` at the last byte moved in either direction.
    last_ns: AtomicU64,
}

impl Idle {
    fn new() -> Self {
        Self {
            base: tokio::time::Instant::now(),
            last_ns: AtomicU64::new(0),
        }
    }

    /// Record activity now.
    fn touch(&self) {
        let ns = u64::try_from(self.base.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.last_ns.store(ns, Ordering::Relaxed);
    }

    /// The instant the tunnel becomes idle, given the last activity seen.
    fn deadline(&self) -> tokio::time::Instant {
        self.base
            + Duration::from_nanos(self.last_ns.load(Ordering::Relaxed))
            + timeouts::TUNNEL_READ
    }
}

/// Bidirectionally copy bytes between the client and the outbound stream,
/// half-closing each direction on EOF and failing the tunnel once NEITHER
/// direction has moved bytes for [`timeouts::TUNNEL_READ`].
pub(crate) async fn relay(client: TcpStream, upstream: BoxStream) -> Result<(), NativeError> {
    // Owned halves for the client: `tokio::io::split` puts the stream behind
    // an `Arc<Mutex<..>>` that is locked on every poll. The upstream is a
    // trait object with no owned split, so it keeps the locking form.
    let (client_r, client_w) = client.into_split();
    let (up_r, up_w) = tokio::io::split(upstream);
    let idle = Arc::new(Idle::new());
    // One future per direction instead of one `select!` over both: an error
    // still tears the peer direction down, but a write in flight is never
    // cancelled by the *other* direction completing.
    tokio::try_join!(
        copy_half(client_r, up_w, &idle),
        copy_half(up_r, client_w, &idle)
    )?;
    Ok(())
}

/// Copy one direction until EOF, then half-close the writer.
///
/// One buffer and one timer per direction, both reused across chunks: the
/// previous shape allocated a fresh 16 KiB stack buffer and built a fresh
/// `tokio::time::timeout` future per chunk.
async fn copy_half<R, W>(mut r: R, mut w: W, idle: &Idle) -> Result<(), std::io::Error>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; RELAY_BUF];
    let sleep = tokio::time::sleep_until(idle.deadline());
    tokio::pin!(sleep);
    loop {
        let deadline = idle.deadline();
        sleep.as_mut().reset(deadline);
        let n = tokio::select! {
            // `AsyncReadExt::read` is cancel-safe, so the timeout branch can
            // only ever discard a read that produced no bytes.
            res = r.read(&mut buf) => res?,
            () = sleep.as_mut() => {
                // The peer direction moved bytes inside the window: the
                // tunnel is alive, re-arm from the shared stamp.
                if idle.deadline() > deadline {
                    continue;
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "relay read timed out",
                ));
            }
        };
        if n == 0 {
            break;
        }
        // Stamp before the write: while this direction blocks on a slow peer,
        // the other direction must not read the tunnel as idle.
        idle.touch();
        // Outside the `select!`: bytes that were read are always written.
        w.write_all(&buf[..n]).await?;
    }
    // EOF: send FIN and let the peer direction keep draining.
    let _ = w.shutdown().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_proto::proto_spec::{HostKind, SecurityConfig, Socks5Config};

    #[test]
    fn proxy_params_carries_protocol_server_target_and_resolved_ip() {
        let proxy = Box::new(ProxyOutbound {
            protocol: ProtocolConfig::Socks(Socks5Config {
                username: Some("u".into()),
                password: Some("p".into()),
                security: SecurityConfig::default(),
                remarks: None,
            }),
            server: EndpointEssentials {
                host: "example.com".into(),
                host_type: HostKind::Dns,
                port: 1080,
                ports: vec![1080],
            },
            resolved_ip: Some("127.0.0.1:1080".parse().unwrap()),
        });
        let target = TargetAddr::new(Host::new("example.org"), 443);

        let params = proxy_params(&proxy, &target);

        assert_eq!(params.target, target);
        assert_eq!(params.server.host, "example.com");
        assert_eq!(params.server.port, 1080);
        assert_eq!(params.resolved_ip, Some("127.0.0.1:1080".parse().unwrap()));
        // proxy_params builds a TCP link; the UDP relay sets its own mode.
        assert!(params.udp.is_none());
        assert!(!params.mux);
        let ProtocolConfig::Socks(config) = &params.protocol else {
            panic!("protocol must round-trip");
        };
        assert_eq!(config.username.as_deref(), Some("u"));
    }

    /// The old relay `select!`ed two futures that each did `read` then
    /// `write_all`; when one completed, the other was dropped — and if it sat
    /// inside `write_all`, the bytes it had already read were gone. Force that
    /// shape: a 1 KiB upstream pipe with a slow reader keeps the forward
    /// `write_all` blocked while reverse traffic keeps completing.
    #[tokio::test]
    async fn relay_does_not_lose_bytes_under_write_backpressure() {
        const TOTAL: usize = 64 * 1024;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (upstream_side, peer) = tokio::io::duplex(1024);
        let relayed = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            relay(sock, Box::new(upstream_side)).await
        });

        let (mut peer_r, mut peer_w) = tokio::io::split(peer);
        let drain = tokio::spawn(async move {
            let mut got = Vec::with_capacity(TOTAL);
            let mut buf = vec![0u8; 4096];
            while got.len() < TOTAL {
                let n = peer_r.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..n]);
                // Slow reader: keep the relay's forward write blocked.
                tokio::time::sleep(std::time::Duration::from_micros(200)).await;
            }
            got
        });
        // Reverse traffic so the other direction keeps completing — that is
        // what used to cancel the forward write.
        let push_back = tokio::spawn(async move {
            for _ in 0..64 {
                peer_w.write_all(&[0x5a; 1024]).await.unwrap();
            }
        });

        let (mut client_r, mut client_w) = tokio::net::TcpStream::connect(addr)
            .await
            .unwrap()
            .into_split();
        let client_drain = tokio::spawn(async move {
            let mut sink = vec![0u8; 4096];
            while client_r.read(&mut sink).await.unwrap_or(0) > 0 {}
        });

        let payload: Vec<u8> = (0..TOTAL).map(|i| u8::try_from(i % 251).unwrap()).collect();
        client_w.write_all(&payload).await.unwrap();
        client_w.shutdown().await.unwrap();

        let got = drain.await.unwrap();
        assert_eq!(
            got.len(),
            TOTAL,
            "relay dropped {} byte(s) under backpressure",
            TOTAL - got.len()
        );
        assert_eq!(got, payload, "relay reordered or corrupted the stream");
        push_back.await.unwrap();
        client_drain.abort();
        let _ = relayed.await.unwrap();
    }

    /// `TUNNEL_READ` bounds inactivity of the TUNNEL, not of one direction: on
    /// any download the uplink is quiet for the whole transfer. With a timer
    /// per direction the quiet half fires at 30s and `try_join!` takes the
    /// streaming half down with it.
    #[tokio::test(start_paused = true)]
    async fn relay_idle_deadline_is_shared_between_directions() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (upstream_side, peer) = tokio::io::duplex(4096);
        let relayed = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            relay(sock, Box::new(upstream_side)).await
        });
        let (mut peer_r, mut peer_w) = tokio::io::split(peer);
        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();

        // Uplink: one short request, then silence for the rest of the test.
        client.write_all(b"GET /\r\n").await.unwrap();
        let mut req = [0u8; 7];
        peer_r.read_exact(&mut req).await.unwrap();

        // Downlink: three chunks 20 virtual seconds apart. Every gap is under
        // TUNNEL_READ (30s), but the uplink stays quiet for 40s in total.
        let mut got = [0u8; 512];
        for tag in [1u8, 2, 3] {
            if tag > 1 {
                tokio::time::advance(Duration::from_secs(20)).await;
            }
            peer_w.write_all(&[tag; 512]).await.unwrap();
            client.read_exact(&mut got).await.unwrap();
            assert_eq!(
                got[0],
                tag,
                "chunk {tag} lost after {}s of uplink silence",
                u32::from(tag - 1) * 20
            );
        }
        assert!(
            !relayed.is_finished(),
            "relay ended while the tunnel was live"
        );
    }
}
