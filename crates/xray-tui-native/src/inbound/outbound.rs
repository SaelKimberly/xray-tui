//! Outbounds for the SOCKS5 inbound: tagged destinations the router may
//! select (`Decision::Route { tag }` → [`Outbound`]).
//!
//! Composition mirrors xray/sing-box: the "proxy" outbound reuses
//! [`crate::connect`] — the native client tunnel — so a SOCKS5 inbound in
//! front of a VLESS/VMess/... proxy is the in-process equivalent of a local
//! SOCKS5 listener wired to a remote proxy.

use std::net::SocketAddr;

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

/// Bidirectionally copy bytes between the client and the outbound stream,
/// half-closing each direction on EOF and bounding every read with
/// [`timeouts::TUNNEL_READ`].
pub(crate) async fn relay(client: TcpStream, upstream: BoxStream) -> Result<(), NativeError> {
    let (mut client_r, mut client_w) = tokio::io::split(client);
    let (mut up_r, mut up_w) = tokio::io::split(upstream);
    let (mut client_done, mut upstream_done) = (false, false);
    loop {
        tokio::select! {
            done = copy_once(&mut client_r, &mut up_w), if !client_done => {
                if done? {
                    client_done = true;
                    // Send FIN to the outbound; keep draining its response.
                    let _ = up_w.shutdown().await;
                }
            }
            done = copy_once(&mut up_r, &mut client_w), if !upstream_done => {
                if done? {
                    upstream_done = true;
                    let _ = client_w.shutdown().await;
                }
            }
        }
        if client_done && upstream_done {
            return Ok(());
        }
    }
}

/// Copy one chunk from `r` into `w`; returns `Ok(true)` on EOF.
async fn copy_once<R, W>(r: &mut R, w: &mut W) -> Result<bool, std::io::Error>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0u8; 16 * 1024];
    let n = tokio::time::timeout(timeouts::TUNNEL_READ, r.read(&mut buf))
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "relay read timed out"))??;
    if n == 0 {
        return Ok(true);
    }
    w.write_all(&buf[..n]).await?;
    Ok(false)
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
}
