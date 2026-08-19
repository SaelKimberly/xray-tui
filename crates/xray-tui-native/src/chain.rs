//! Client-side chaining: tunnel through several proxies in order.
//!
//! Parity with xray `outbound.proxySettings.socksTag` and sing-box `detour`.
//! `links[0]` is dialed directly and carries `links[1]`'s traffic
//! (links[0].target = links[1].server); the last link reaches `target`.
//!
//! Fold (n=1 is `connect`): for each link, dial (first hop) or reuse the
//! previous tunnel, then run security (TLS/REALITY) OUTERMOST, then the
//! transport upgrade (ws/grpc framing inside the TLS session), then the
//! protocol, threading the resulting stream into the next link as its
//! `base`.

use crate::addr::TargetAddr;
use crate::context::{LinkContext, NativeConnectParams};
use crate::error::NativeError;
use crate::protocol;
use crate::protocol::vless::{MuxClient, PacketConn};
use crate::security;
use crate::transport;
use crate::{BoxStream, NativeTunnel};

/// The destination of link `i`: the next link's server, or `target` for the
/// last link. Pure function — unit-tested independently of the fold.
fn next_target(links: &[NativeConnectParams], i: usize, target: &TargetAddr) -> TargetAddr {
    links.get(i + 1).map_or_else(
        || target.clone(),
        |next| TargetAddr::new(next.server.host.as_str(), next.server.port),
    )
}

/// One link's stream after the dial: security (TLS/REALITY) OUTERMOST, then
/// the transport upgrade (ws/grpc framing inside the TLS session) — except
/// for self-contained dials (xhttp+h3) where the dial already IS the
/// finished tunnel and the engine TLS/upgrade must be skipped (spec §5.2:
/// the QUIC dial replaces dial + security + upgrade; quinn's rustls is
/// internal).
async fn secured_upgraded(ctx: &LinkContext, dialed: BoxStream) -> Result<BoxStream, NativeError> {
    if transport::is_self_contained(ctx) {
        return Ok(dialed);
    }
    let secured = security::wrap(ctx, dialed).await?;
    transport::upgrade(ctx, secured).await
}

/// Connect through a chain of proxies to the final `target`.
pub async fn connect_chain(
    links: &[NativeConnectParams],
    target: TargetAddr,
) -> Result<NativeTunnel, NativeError> {
    let mut base: Option<BoxStream> = None;
    for (i, link) in links.iter().enumerate() {
        let to = next_target(links, i, &target);
        let ctx = LinkContext::new(link.clone(), to);
        let dialed = transport::connect(&ctx, base).await?;
        let upgraded = secured_upgraded(&ctx, dialed).await?;
        base = Some(protocol::connect(&ctx, upgraded).await?);
    }
    base.map(NativeTunnel::from_stream)
        .ok_or_else(|| NativeError::Config("empty chain".into()))
}

/// Connect through a chain of proxies to `target` with a VLESS UDP datagram
/// tunnel (command 0x02).
///
/// Identical to [`connect_chain`] except the LAST link runs the UDP
/// protocol phase and the result is the packet-framed [`PacketConn`]
/// instead of a byte tunnel. Intermediate links tunnel TCP as usual — they
/// carry the UDP tunnel as a byte stream to the next hop.
pub async fn connect_chain_udp(
    links: &[NativeConnectParams],
    target: TargetAddr,
) -> Result<PacketConn<BoxStream>, NativeError> {
    let mut base: Option<BoxStream> = None;
    for (i, link) in links.iter().enumerate() {
        let to = next_target(links, i, &target);
        let ctx = LinkContext::new(link.clone(), to);
        let dialed = transport::connect(&ctx, base).await?;
        let upgraded = secured_upgraded(&ctx, dialed).await?;
        if i + 1 == links.len() {
            // The last link speaks the UDP protocol phase: it owns the
            // stream and returns the datagram tunnel.
            return protocol::connect_udp(&ctx, upgraded).await;
        }
        base = Some(protocol::connect(&ctx, upgraded).await?);
    }
    Err(NativeError::Config("empty chain".into()))
}

/// Connect through a chain of proxies to `target` with a VLESS v1.mux.cool
/// multiplexed tunnel (command 0x03).
///
/// Identical to [`connect_chain`] except the LAST link runs the mux
/// protocol phase and the result is the [`MuxClient`] multiplexer instead
/// of a byte tunnel. Intermediate links tunnel TCP as usual — they carry
/// the mux tunnel as a byte stream to the next hop.
pub async fn connect_chain_mux(
    links: &[NativeConnectParams],
    target: TargetAddr,
) -> Result<MuxClient<BoxStream>, NativeError> {
    let mut base: Option<BoxStream> = None;
    for (i, link) in links.iter().enumerate() {
        let to = next_target(links, i, &target);
        let ctx = LinkContext::new(link.clone(), to);
        let dialed = transport::connect(&ctx, base).await?;
        let upgraded = secured_upgraded(&ctx, dialed).await?;
        if i + 1 == links.len() {
            // The last link speaks the mux protocol phase: it owns the
            // stream and returns the multiplexer.
            return protocol::connect_mux(&ctx, upgraded).await;
        }
        base = Some(protocol::connect(&ctx, upgraded).await?);
    }
    Err(NativeError::Config("empty chain".into()))
}

#[cfg(test)]
mod tests {
    use xray_tui_proto::proto_spec::ProtocolConfig;
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

    use super::*;
    use crate::addr::Host;

    fn vless_empty() -> ProtocolConfig {
        serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        }))
        .expect("vless config parses")
    }

    fn params(host: &str, port: u16) -> NativeConnectParams {
        NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new(host, port),
            TargetAddr::new(Host::Domain("ignored".into()), 1),
        )
    }

    #[test]
    fn next_target_uses_next_server() {
        let links = [params("a.example", 10), params("b.example", 20)];
        let final_target = TargetAddr::new(Host::Domain("www.example.com".into()), 443);
        assert_eq!(
            next_target(&links, 0, &final_target),
            TargetAddr::new(Host::Domain("b.example".into()), 20)
        );
        assert_eq!(next_target(&links, 1, &final_target), final_target);
    }

    #[test]
    fn empty_chain_errors() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(connect_chain(
            &[],
            TargetAddr::new(Host::Domain("x".into()), 1),
        ));
        assert!(matches!(res, Err(NativeError::Config(_))));
    }
}
