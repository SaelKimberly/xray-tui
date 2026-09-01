//! SOCKS5 inbound: a local SOCKS5 server (RFC 1928/1929) that accepts
//! connections, routes them through the [`xray_tui_route`] engine, and
//! forwards to direct / block / proxy outbounds.
//!
//! Composition mirrors xray/sing-box: `inbound → router → outbound`. The
//! router is the compiled [`Engine`]; each [`Decision::Route`] names an
//! outbound tag resolved against [`Socks5InboundConfig::outbounds`]. The
//! "proxy" outbound reuses [`crate::connect`] — see [`outbound`].
//!
//! Scope: TCP CONNECT only. BIND and UDP ASSOCIATE are refused with
//! `0x07 Command not supported`; a `HijackDns` routing decision is refused
//! with `0x02` (the inbound has no built-in DNS interceptor).

pub mod outbound;
pub mod socks5;

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use xray_tui_route::engine::decide_async;
use xray_tui_route::ir::NetworkMask;
use xray_tui_route::{ConnMeta, Decision, Engine, NetAddr, NetHost};

use crate::addr::{Host, TargetAddr};
use crate::error::{NativeError, timeouts};

pub use outbound::{Outbound, OutboundKind, ProxyOutbound};

/// Inbound tag reported to the router when [`Socks5InboundConfig`] doesn't
/// override it.
pub const DEFAULT_INBOUND_TAG: &str = "socks-in";

/// The `BND.ADDR`/`BND.PORT` echoed in replies: this inbound reports a
/// zero address (the client already knows its peer; RFC 1928 allows it).
const BIND_ZERO: TargetAddr = TargetAddr {
    host: Host::Ip(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    port: 0,
};

/// Configuration for a [`Socks5Inbound`].
#[derive(Clone)]
pub struct Socks5InboundConfig {
    /// Address to bind (e.g. `127.0.0.1:1080`).
    pub listen: SocketAddr,
    /// RFC 1929 username/password; `None` = no-auth.
    pub auth: Option<(String, String)>,
    /// Inbound tag reported to the router (`ConnMeta.inbound_tag`).
    pub inbound_tag: String,
    /// Compiled routing engine.
    pub engine: Arc<Engine>,
    /// Tagged outbounds the router may select.
    pub outbounds: Vec<Outbound>,
}

impl Socks5InboundConfig {
    /// Builds a config with no auth and the default inbound tag.
    #[must_use]
    pub fn new(listen: SocketAddr, engine: Arc<Engine>, outbounds: Vec<Outbound>) -> Self {
        Self {
            listen,
            auth: None,
            inbound_tag: DEFAULT_INBOUND_TAG.to_owned(),
            engine,
            outbounds,
        }
    }

    /// Requires RFC 1929 username/password authentication.
    #[must_use]
    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Some((username.into(), password.into()));
        self
    }
}

impl fmt::Debug for Socks5InboundConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Socks5InboundConfig")
            .field("listen", &self.listen)
            .field("auth", &self.auth.as_ref().map(|_| "<redacted>"))
            .field("inbound_tag", &self.inbound_tag)
            .field("outbounds", &self.outbounds)
            .finish_non_exhaustive()
    }
}

/// A bound SOCKS5 server.
pub struct Socks5Inbound {
    listener: TcpListener,
    config: Arc<Socks5InboundConfig>,
}

impl Socks5Inbound {
    /// Binds the listener; call [`Self::serve`] to accept connections.
    ///
    /// # Errors
    /// Returns [`NativeError::Io`] when the listen address cannot be bound.
    pub async fn bind(config: Socks5InboundConfig) -> Result<Self, NativeError> {
        let listener = TcpListener::bind(config.listen).await?;
        Ok(Self {
            listener,
            config: Arc::new(config),
        })
    }

    /// The bound local address (useful when listening on port 0).
    ///
    /// # Errors
    /// Returns [`NativeError::Io`] when the kernel reports no local address.
    pub fn local_addr(&self) -> Result<SocketAddr, NativeError> {
        self.listener.local_addr().map_err(NativeError::Io)
    }

    /// Runs the accept loop forever, spawning one task per connection.
    ///
    /// Returns only when the listener fails. Dropping the task running this
    /// future aborts the accept loop; in-flight connections keep their own
    /// tasks.
    ///
    /// # Errors
    /// Returns [`NativeError::Io`] when the accept loop fails.
    pub async fn serve(self) -> Result<(), NativeError> {
        loop {
            let (conn, peer) = self.listener.accept().await?;
            let config = Arc::clone(&self.config);
            tokio::spawn(async move {
                if let Err(error) = Box::pin(handle_conn(&config, conn, peer)).await {
                    tracing::debug!(%peer, %error, "socks5 inbound: connection closed");
                }
            });
        }
    }
}

/// One connection: negotiate, route, dispatch, relay.
async fn handle_conn(
    config: &Socks5InboundConfig,
    mut conn: TcpStream,
    peer: SocketAddr,
) -> Result<(), NativeError> {
    // Method negotiation (greeting + optional RFC 1929 auth).
    let negotiated = tokio::time::timeout(
        timeouts::PROTOCOL,
        socks5::negotiate(&mut conn, config.auth.as_ref()),
    )
    .await;
    match negotiated {
        Err(_) => {
            return Err(NativeError::Timeout {
                step: "socks5 negotiate",
                limit: timeouts::PROTOCOL,
            });
        }
        Ok(Err(error)) => return Err(error.into()),
        Ok(Ok(())) => {}
    }

    // Request.
    let request = tokio::time::timeout(timeouts::PROTOCOL, socks5::read_request(&mut conn))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "socks5 request",
            limit: timeouts::PROTOCOL,
        })?
        .map_err(NativeError::from)?;

    // Only CONNECT is supported; BIND and UDP ASSOCIATE get a clean refusal.
    if request.cmd != socks5::Command::Connect {
        socks5::write_reply(
            &mut conn,
            socks5::ReplyCode::CommandNotSupported,
            &BIND_ZERO,
        )
        .await?;
        return Ok(());
    }

    // Route the destination.
    let mut meta = ConnMeta {
        target: target_to_net(&request.target),
        network: NetworkMask::TCP,
        inbound_tag: Some(config.inbound_tag.clone()),
        source: Some(peer),
        source_resolved_ips: Vec::new(),
        payload_prefix: None,
        sniffed: None,
        sni_host: None,
        resolved_host_ips: Vec::new(),
    };
    let decision = decide_async(&config.engine, &mut meta).await;

    match decision {
        Decision::Route { tag, override_addr } => {
            let Some(outbound) = config.outbounds.iter().find(|o| o.tag == tag) else {
                tracing::warn!(%tag, "socks5 inbound: routing decision named an unknown outbound");
                socks5::write_reply(&mut conn, socks5::ReplyCode::GeneralFailure, &BIND_ZERO)
                    .await?;
                return Ok(());
            };
            match &outbound.kind {
                OutboundKind::Block => {
                    socks5::write_reply(
                        &mut conn,
                        socks5::ReplyCode::ConnectionNotAllowed,
                        &BIND_ZERO,
                    )
                    .await?;
                    Ok(())
                }
                kind => {
                    let target = override_addr.map(net_to_target).unwrap_or(request.target);
                    let upstream = match outbound::dial(kind, &target).await {
                        Ok(stream) => stream,
                        Err(error) => {
                            tracing::warn!(%tag, %error, "socks5 inbound: outbound dial failed");
                            socks5::write_reply(&mut conn, reply_for(&error), &BIND_ZERO).await?;
                            return Ok(());
                        }
                    };
                    socks5::write_reply(&mut conn, socks5::ReplyCode::Succeeded, &BIND_ZERO)
                        .await?;
                    Box::pin(outbound::relay(conn, upstream)).await
                }
            }
        }
        Decision::Reject { .. } => {
            socks5::write_reply(
                &mut conn,
                socks5::ReplyCode::ConnectionNotAllowed,
                &BIND_ZERO,
            )
            .await?;
            Ok(())
        }
        Decision::HijackDns => {
            // The inbound has no built-in DNS interceptor; refuse explicitly
            // (explicit absence beats silent fallthrough).
            tracing::warn!("socks5 inbound: HijackDns decision is not implemented; rejecting");
            socks5::write_reply(
                &mut conn,
                socks5::ReplyCode::ConnectionNotAllowed,
                &BIND_ZERO,
            )
            .await?;
            Ok(())
        }
    }
}

/// Map an outbound dial error to a SOCKS5 reply code.
#[must_use]
const fn reply_for(error: &NativeError) -> socks5::ReplyCode {
    match error {
        NativeError::Dial(_) | NativeError::Timeout { .. } => socks5::ReplyCode::HostUnreachable,
        NativeError::NotImplemented { .. } => socks5::ReplyCode::CommandNotSupported,
        NativeError::Config(_)
        | NativeError::Tls(_)
        | NativeError::Reality(_)
        | NativeError::Transport(_)
        | NativeError::Protocol { .. }
        | NativeError::Io(_) => socks5::ReplyCode::GeneralFailure,
    }
}

/// Convert a native wire target into the router's [`NetAddr`].
#[must_use]
fn target_to_net(target: &TargetAddr) -> NetAddr {
    NetAddr {
        host: match &target.host {
            Host::Ip(ip) => NetHost::Ip(*ip),
            Host::Domain(domain) => NetHost::Domain(domain.clone()),
        },
        port: target.port,
    }
}

/// Convert a router rewrite ([`NetAddr`]) back into a native wire target.
#[must_use]
fn net_to_target(addr: NetAddr) -> TargetAddr {
    TargetAddr {
        host: match addr.host {
            NetHost::Ip(ip) => Host::Ip(ip),
            NetHost::Domain(domain) => Host::Domain(domain),
        },
        port: addr.port,
    }
}

impl From<socks5::Socks5Error> for NativeError {
    fn from(error: socks5::Socks5Error) -> Self {
        match error {
            socks5::Socks5Error::Io(io) => Self::Io(io),
            other => Self::Config(format!("socks5: {other}")),
        }
    }
}

#[cfg(test)]
mod tests;
