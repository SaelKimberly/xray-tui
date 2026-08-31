//! Protocol phase: the INNERMOST layer — write the protocol handshake onto
//! the secured stream and produce the byte tunnel.
//!
//! Dispatch strategy (see `shape.rs`): the uniform handshake-over-stream
//! pipeline applies to the TCP-stream family. Device tunnels
//! (WireGuard/Tailscale), own-handshake protocols (SSH/Tor), and the
//! outbound-only kinds (Redirect/TProxy/Mixed) take divergent paths at their
//! own `connect()` and return `NotImplemented` here until those paths exist.

use xray_tui_proto::proto_spec::ProtocolConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

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
        ProtocolConfig::Socks(_) => not_impl("socks5"),
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

/// Run the UDP protocol phase: VLESS command 0x02 + packet framing over the
/// given stream — the last link of a `connect_udp` chain.
///
/// Only VLESS has a native UDP path; every other protocol stays
/// `NotImplemented` here.
pub async fn connect_udp(
    ctx: &LinkContext,
    stream: BoxStream,
) -> Result<crate::protocol::vless::PacketConn<BoxStream>, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Vless(cfg) => vless::connect_udp(ctx, stream, cfg).await,
        _ => Err(NativeError::NotImplemented {
            feature: "udp protocol connect (native UDP path is vless-only)".into(),
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
