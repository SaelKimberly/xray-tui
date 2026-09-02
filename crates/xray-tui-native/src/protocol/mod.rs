//! Protocol phase: the INNERMOST layer — write the protocol handshake onto
//! the secured stream and produce the byte tunnel.
//!
//! Dispatch strategy (see `shape.rs`): the uniform handshake-over-stream
//! pipeline applies to the TCP-stream family. Device tunnels
//! (WireGuard/Tailscale), own-handshake protocols (SSH/Tor), and the
//! outbound-only kinds (Redirect/TProxy/Mixed) take divergent paths at their
//! own `connect()` and return `NotImplemented` here until those paths exist.

use xray_tui_proto::proto_spec::{ProtocolConfig, ProtocolKind};

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;
use crate::protocol::vless::PacketMode;

pub mod anytls;
pub mod http;
pub mod hysteria1;
pub mod hysteria2;
pub mod mixed;
pub mod naive;
pub mod redirect;
pub mod shadowtls;
pub mod socks;
pub mod ss;
pub mod ssh;
pub mod ssr;
pub mod tailscale;
pub mod tor;
pub mod tproxy;
pub mod trojan;
pub mod tuic;
pub mod vless;
pub mod vmess;
pub mod wireguard;

/// One-line error shorthand for the placeholder arms.
fn not_impl(feature: &str) -> Result<BoxStream, NativeError> {
    Err(NativeError::NotImplemented {
        feature: format!("protocol {feature}"),
    })
}

/// Run the protocol phase: handshake + tunnel over the given stream.
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Vless(cfg) => vless::connect(ctx, stream, cfg).await,
        ProtocolConfig::Vmess(cfg) => vmess::connect(ctx, stream, cfg).await,
        ProtocolConfig::Trojan(cfg) => trojan::connect(ctx, stream, cfg).await,
        ProtocolConfig::Hysteria2(_) => not_impl("hysteria2"),
        ProtocolConfig::Ss(_) => not_impl("shadowsocks"),
        ProtocolConfig::Ssr(_) => not_impl("shadowsocksr"),
        ProtocolConfig::Tuic(_) => not_impl("tuic"),
        ProtocolConfig::Wireguard(_) => not_impl("wireguard"),
        ProtocolConfig::Socks(cfg) => socks::connect(ctx, stream, cfg).await,
        ProtocolConfig::Http(_) => not_impl("http"),
        ProtocolConfig::Naive(_) => not_impl("naive"),
        ProtocolConfig::AnyTls(_) => not_impl("anytls"),
        ProtocolConfig::ShadowTls(_) => not_impl("shadowtls"),
        ProtocolConfig::Tor(_) => not_impl("tor"),
        ProtocolConfig::Ssh(_) => not_impl("ssh"),
        ProtocolConfig::Tailscale(_) => not_impl("tailscale"),
        ProtocolConfig::Hysteria1(_) => not_impl("hysteria1"),
        ProtocolConfig::Redirect(_) => not_impl("redirect (outbound-only kind)"),
        ProtocolConfig::TProxy(_) => not_impl("tproxy (outbound-only kind)"),
        ProtocolConfig::Mixed(_) => not_impl("mixed (outbound-only kind)"),
    }
}

/// True when the link's protocol is a QUIC-family proxy (Hysteria2 /
/// Hysteria1 / TUIC) whose dial replaces the TCP dial + security + upgrade
/// chain entirely (shape.rs `ConnectShape::Quic`). The chain dispatches
/// these to [`connect_quic`] instead of the uniform transport pipeline.
#[must_use]
pub(crate) const fn is_quic_link(ctx: &LinkContext) -> bool {
    matches!(
        ctx.params.protocol,
        ProtocolConfig::Hysteria2(_) | ProtocolConfig::Hysteria1(_) | ProtocolConfig::Tuic(_)
    )
}

/// Run the QUIC protocol phase: a fresh QUIC dial (dial + security +
/// upgrade REPLACED by the QUIC connection — spec §5.2), then the protocol's
/// own handshake over a QUIC stream.
///
/// The only implemented member is Hysteria2; Hysteria1 and TUIC stay
/// `NotImplemented` until their modules land.
pub async fn connect_quic(ctx: &LinkContext) -> Result<BoxStream, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Hysteria2(cfg) => hysteria2::connect(ctx, cfg).await,
        ProtocolConfig::Hysteria1(_) => not_impl("hysteria1 (QUIC)"),
        ProtocolConfig::Tuic(_) => not_impl("tuic (QUIC)"),
        _ => Err(NativeError::Config(
            "connect_quic requires a QUIC-family protocol (Hysteria2/Hysteria1/Tuic)".into(),
        )),
    }
}

/// A protocol-generic UDP datagram tunnel — the result of the UDP protocol
/// phase. Each variant owns its protocol's datagram carrier:
///
/// - [`PacketTunnel::Vless`] — command 0x02 `[2B len]` framing (+ per-packet
///   address in packetaddr mode, mux XUDP in mux mode).
/// - [`PacketTunnel::Vmess`] — command 0x02 AEAD records (one record = one
///   datagram, no per-packet address).
/// - [`PacketTunnel::Trojan`] — command 3 address-prefixed frames
///   (`ATYP|addr|port || len || CRLF || payload`).
/// - [`PacketTunnel::Hysteria2`] — QUIC DATAGRAM `UDPMessage` frames
///   (hysteria2 is a QUIC dial, not a stream chain).
///
/// All variants expose the same datagram API: [`send`](Self::send) /
/// [`recv`](Self::recv).
pub enum PacketTunnel {
    Vless(vless::PacketConn<BoxStream>),
    Vmess(vmess::udp::PacketConn<BoxStream>),
    Trojan(trojan::PacketConn<BoxStream>),
    Hysteria2(hysteria2::udp::UdpConn),
}

impl PacketTunnel {
    /// Send one datagram.
    ///
    /// `dest = None` means **the session/header destination** — the
    /// destination the tunnel was opened for. Every variant accepts it.
    ///
    /// `dest = Some(d)` is a **per-packet destination**. The carriers that
    /// have one put `d` on the wire (trojan's address-prefixed frame,
    /// hysteria2's `UDPMessage` address, vless packetaddr / XUDP). The
    /// variants that cannot carry a per-packet address — vmess records and
    /// vless `Raw` — accept `Some(d)` only when `d` IS their session target
    /// and refuse a differing one with
    /// [`std::io::ErrorKind::InvalidInput`], rather than silently
    /// mis-routing the datagram to the header destination.
    pub async fn send(
        &mut self,
        dest: Option<std::net::SocketAddr>,
        payload: &[u8],
    ) -> std::io::Result<()> {
        match self {
            Self::Vless(c) => c.send(dest, payload).await,
            Self::Vmess(c) => c.send(dest, payload).await,
            Self::Trojan(c) => c.send(dest, payload).await,
            Self::Hysteria2(c) => c.send(dest, payload).await,
        }
    }

    /// Receive one datagram. `Ok(None)` on a clean end-of-stream.
    pub async fn recv(
        &mut self,
    ) -> std::io::Result<Option<(Option<std::net::SocketAddr>, Vec<u8>)>> {
        match self {
            Self::Vless(c) => c.recv().await,
            Self::Vmess(c) => c.recv().await,
            Self::Trojan(c) => c.recv().await,
            Self::Hysteria2(c) => c.recv().await,
        }
    }
}

/// The read half of a split [`PacketTunnel`] — see [`PacketTunnel::split`].
pub enum PacketReader {
    Vless(vless::PacketReader<tokio::io::ReadHalf<BoxStream>>),
    Vmess(vmess::udp::PacketReader<tokio::io::ReadHalf<BoxStream>>),
    Trojan(trojan::PacketReader<tokio::io::ReadHalf<BoxStream>>),
    Hysteria2(hysteria2::udp::UdpReader),
}

/// The write half of a split [`PacketTunnel`] — see [`PacketTunnel::split`].
pub enum PacketWriter {
    Vless(vless::PacketWriter<tokio::io::WriteHalf<BoxStream>>),
    Vmess(vmess::udp::PacketWriter<tokio::io::WriteHalf<BoxStream>>),
    Trojan(trojan::PacketWriter<tokio::io::WriteHalf<BoxStream>>),
    Hysteria2(hysteria2::udp::UdpWriter),
}

impl PacketTunnel {
    /// Split the tunnel into halves that may be used from separate tasks.
    ///
    /// A bidirectional relay needs this: [`Self::recv`] is a multi-step frame
    /// read for the stream carriers, so racing it against a send in a
    /// `tokio::select!` would drop a partially-read frame and desynchronise
    /// the tunnel. With halves the reader lives in its own task and its
    /// future is never cancelled.
    ///
    /// # Errors
    /// Returns [`std::io::ErrorKind::InvalidInput`] for a carrier that has no
    /// halves — the VLESS XUDP mode, whose datagrams ride a shared mux tunnel
    /// rather than a stream.
    pub fn split(self) -> std::io::Result<(PacketReader, PacketWriter)> {
        match self {
            Self::Vless(c) => {
                let (r, w) = c.split()?;
                Ok((PacketReader::Vless(r), PacketWriter::Vless(w)))
            }
            Self::Vmess(c) => {
                let (r, w) = c.split()?;
                Ok((PacketReader::Vmess(r), PacketWriter::Vmess(w)))
            }
            Self::Trojan(c) => {
                let (r, w) = c.split()?;
                Ok((PacketReader::Trojan(r), PacketWriter::Trojan(w)))
            }
            Self::Hysteria2(c) => {
                let (r, w) = c.split()?;
                Ok((PacketReader::Hysteria2(r), PacketWriter::Hysteria2(w)))
            }
        }
    }
}

impl PacketReader {
    /// Receive one datagram. `Ok(None)` on a clean end-of-stream.
    ///
    /// Cancellation-safe for every carrier: partial frame progress lives in
    /// the reader, not in the future.
    pub async fn recv(
        &mut self,
    ) -> std::io::Result<Option<(Option<std::net::SocketAddr>, Vec<u8>)>> {
        match self {
            Self::Vless(r) => r.recv().await,
            Self::Vmess(r) => r.recv().await,
            Self::Trojan(r) => r.recv().await,
            Self::Hysteria2(r) => r.recv().await,
        }
    }
}

impl PacketWriter {
    /// Send one datagram — the same contract as [`PacketTunnel::send`].
    pub async fn send(
        &mut self,
        dest: Option<std::net::SocketAddr>,
        payload: &[u8],
    ) -> std::io::Result<()> {
        match self {
            Self::Vless(w) => w.send(dest, payload).await,
            Self::Vmess(w) => w.send(dest, payload).await,
            Self::Trojan(w) => w.send(dest, payload).await,
            Self::Hysteria2(w) => w.send(dest, payload).await,
        }
    }
}

/// Refuse the VLESS-only packet modes on a non-VLESS UDP link.
///
/// [`PacketMode::PacketAddr`] (per-packet magic-address headers — spec §4.3)
/// and [`PacketMode::XUdp`] (datagrams over a mux UDP session — §4.1) are
/// VLESS wire features. The other carriers have their own per-packet
/// address (trojan frames, hysteria2 `UDPMessage`) or none at all (vmess
/// records), so a row asking for those modes on a vmess / trojan /
/// hysteria2 link is a misconfiguration — refuse it instead of silently
/// handing back a header-dest tunnel that ignores the request.
pub(crate) fn reject_vless_only_mode(
    ctx: &LinkContext,
    kind: ProtocolKind,
) -> Result<(), NativeError> {
    if let Some(mode @ (PacketMode::PacketAddr | PacketMode::XUdp)) = ctx.params.udp {
        return Err(NativeError::Protocol {
            kind,
            detail: format!(
                "packet mode {mode:?} exists only on the VLESS wire (packetaddr magic-address headers, mux XUDP sessions) — this protocol's UDP tunnel cannot carry it"
            ),
        });
    }
    Ok(())
}

/// Run the UDP protocol phase over an already-established stream (the last
/// link of a stream-chain `connect_udp`). Hysteria2 is a QUIC dial and is
/// dispatched by [`crate::connect_chain_udp`], never here.
pub async fn connect_udp(
    ctx: &LinkContext,
    stream: BoxStream,
) -> Result<PacketTunnel, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Vless(cfg) => Ok(PacketTunnel::Vless(
            vless::connect_udp(ctx, stream, cfg).await?,
        )),
        ProtocolConfig::Vmess(cfg) => {
            reject_vless_only_mode(ctx, ProtocolKind::Vmess)?;
            Ok(PacketTunnel::Vmess(
                vmess::connect_udp(ctx, stream, cfg).await?,
            ))
        }
        ProtocolConfig::Trojan(cfg) => {
            reject_vless_only_mode(ctx, ProtocolKind::Trojan)?;
            Ok(PacketTunnel::Trojan(
                trojan::connect_udp(ctx, stream, cfg).await?,
            ))
        }
        ProtocolConfig::Hysteria2(_) => Err(NativeError::NotImplemented {
            feature:
                "hysteria2 udp over a stream chain (hy2 is a QUIC dial — use connect_chain_udp)"
                    .into(),
        }),
        // SOCKS5 UDP (ASSOCIATE) datagrams ride a raw UDP socket to the proxy,
        // not this byte stream — a divergent shape with no PacketTunnel here.
        ProtocolConfig::Socks(_) => Err(NativeError::NotImplemented {
            feature: "socks5 udp (needs a raw UDP socket shape, not the stream tunnel)".into(),
        }),
        _ => Err(NativeError::NotImplemented {
            feature: "udp protocol connect (native UDP path: vless, vmess, trojan, hysteria2)"
                .into(),
        }),
    }
}

/// Run the mux protocol phase: VLESS command 0x03 + v1.mux.cool framing
/// over the given stream — the last link of a `connect_mux` chain.
///
/// Only VLESS has a native mux path; every other protocol stays
/// `NotImplemented` here.
pub async fn connect_mux(
    ctx: &LinkContext,
    stream: BoxStream,
) -> Result<crate::protocol::vless::MuxClient<BoxStream>, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Vless(cfg) => vless::connect_mux(ctx, stream, cfg).await,
        _ => Err(NativeError::NotImplemented {
            feature: "mux protocol connect (native mux path is vless-only)".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

    use super::*;
    use crate::addr::{Host, TargetAddr};
    use crate::context::NativeConnectParams;

    /// A `LinkContext` carrying `params.udp = mode`. The mode gate never
    /// reads `params.protocol` — the caller names the kind (its dispatch arm
    /// already matched the config), so one config serves every kind here.
    fn udp_ctx(mode: Option<PacketMode>) -> LinkContext {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        }))
        .expect("config parses");
        let mut params = NativeConnectParams::new(
            protocol,
            EndpointEssentials::new("proxy.example", 443),
            TargetAddr::new(Host::new("1.2.3.4"), 53),
        );
        params.udp = mode;
        let target = params.target.clone();
        LinkContext::new(params, target)
    }

    #[test]
    fn vless_only_modes_are_refused_per_protocol() {
        // The three non-VLESS UDP entry points: the vmess and trojan arms of
        // `connect_udp`, and the hysteria2 branch of `connect_chain_udp`.
        for kind in [
            ProtocolKind::Vmess,
            ProtocolKind::Trojan,
            ProtocolKind::Hysteria2,
        ] {
            for mode in [PacketMode::PacketAddr, PacketMode::XUdp] {
                let err = reject_vless_only_mode(&udp_ctx(Some(mode)), kind)
                    .expect_err("a VLESS-only mode must be refused");
                match err {
                    NativeError::Protocol { kind: got, detail } => {
                        assert_eq!(got, kind, "the error names the refusing protocol");
                        assert!(detail.contains("VLESS"), "detail: {detail}");
                    }
                    other => panic!("expected a protocol error, got {other:?}"),
                }
            }
            // Raw (the header-dest mode every carrier has) and the TCP-path
            // `None` pass through untouched.
            reject_vless_only_mode(&udp_ctx(Some(PacketMode::Raw)), kind)
                .expect("raw is every protocol's header-dest mode");
            reject_vless_only_mode(&udp_ctx(None), kind).expect("no mode is not a mode error");
        }
    }
}
