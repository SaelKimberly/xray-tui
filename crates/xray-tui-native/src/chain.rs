//! Client-side chaining: tunnel through several proxies in order.
//!
//! Parity with xray `outbound.proxySettings.socksTag` and sing-box `detour`.
//! `links[0]` is dialed directly and carries `links[1]`'s traffic
//! (links[0].target = links[1].server); the last link reaches `target`.
//!
//! Fold (n=1 is `connect`): for each link, run transport (dial first hop /
//! reuse previous tunnel on later hops) → security → protocol, threading the
//! resulting stream into the next link as its `base`.

use crate::addr::TargetAddr;
use crate::context::{LinkContext, NativeConnectParams};
use crate::error::NativeError;
use crate::protocol;
use crate::security;
use crate::transport;
use crate::{BoxStream, NativeTunnel};

/// The destination of link `i`: the next link's server, or `target` for the
/// last link. Pure function — unit-tested independently of the fold.
fn next_target(links: &[NativeConnectParams], i: usize, target: &TargetAddr) -> TargetAddr {
    links
        .get(i + 1)
        .map(|next| TargetAddr::new(next.server.host.as_str(), next.server.port))
        .unwrap_or_else(|| target.clone())
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
        let transported = transport::connect(&ctx, base).await?;
        let secured = security::wrap(&ctx, transported).await?;
        base = Some(protocol::connect(&ctx, secured).await?);
    }
    base.map(NativeTunnel::from_stream)
        .ok_or_else(|| NativeError::Config("empty chain".into()))
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
