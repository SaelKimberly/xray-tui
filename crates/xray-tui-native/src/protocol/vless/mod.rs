//! VLESS — the reference protocol for the native core.

use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::{SecurityConfig, VlessConfig};

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vless::stream::VlessClientStream;
use crate::protocol::vless::vision::{FLOW_XTLS_RPRX_VISION, VisionStream};

pub mod header;
pub(crate) mod packet;
pub(crate) mod packetaddr;
pub mod stream;
pub(crate) mod udp;
pub(crate) mod vision;

/// Connect through a VLESS outbound over an already-secured stream.
///
/// Writes the request header and returns a tunnel that strips the response
/// header on its first read (see `stream.rs` for the eager-vs-lazy header
/// semantics of xray-core vs sing-box).
///
/// `flow == "xtls-rprx-vision"` takes the vision path: the header carries
/// the protobuf flow addon (spec §4.1), the camouflage frame is emitted
/// right after it (spec §4.6 step 3, deviation 1), and the padded codec
/// wraps the stream (spec §5.4). Any other non-empty flow stays the
/// `NotImplemented` guard.
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VlessConfig,
) -> Result<BoxStream, NativeError> {
    let uuid = header::uuid_bytes(&cfg.uuid)?;
    match cfg.flow.as_deref() {
        None | Some("") => connect_plain(ctx, stream, uuid).await,
        Some(FLOW_XTLS_RPRX_VISION) => connect_vision(ctx, stream, uuid).await,
        Some(other) => Err(NativeError::NotImplemented {
            feature: format!("vless flow {other}"),
        }),
    }
}

/// Plain VLESS connect: request header without addons, then the tunnel.
async fn connect_plain(
    ctx: &LinkContext,
    stream: BoxStream,
    uuid: [u8; 16],
) -> Result<BoxStream, NativeError> {
    let request = header::encode_request(&uuid, &ctx.target, header::CMD_TCP, None)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless request write",
            limit: timeout,
        })??;

    Ok(Box::new(VlessClientStream::new(stream)))
}

/// Vision (`xtls-rprx-vision`) connect path (spec §5.4).
///
/// Guarded preconditions, checked before any I/O:
/// - outer security is TLS or REALITY (the engine is TLS 1.3-only, so the
///   `has_tls()` check is the whole story) — mirroring xray's rejection;
/// - the transport is raw TCP — vision requires the socket for the Direct
///   handoff (spec §5.1; ws/grpc/xhttp framing is incompatible);
/// - the command is TCP. The native VLESS client only has the TCP path
///   (this file always sends [`header::CMD_TCP`]); a future UDP path must
///   reject vision here — no XUDP mux (spec §2).
async fn connect_vision(
    ctx: &LinkContext,
    stream: BoxStream,
    uuid: [u8; 16],
) -> Result<BoxStream, NativeError> {
    if !has_tls(ctx) {
        return Err(NativeError::Config(
            "XTLS only supports TLS and REALITY directly for now".into(),
        ));
    }
    if ctx.transport_type() != Some("tcp") {
        return Err(NativeError::Config(
            "XTLS only supports the TCP transport for now".into(),
        ));
    }

    // Step 2: request header with the flow addon (addon_len = 18).
    let request = header::encode_request(
        &uuid,
        &ctx.target,
        header::CMD_TCP,
        header::encode_addons(Some(FLOW_XTLS_RPRX_VISION)).as_deref(),
    )?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless request write",
            limit: timeout,
        })??;

    // Step 3: the camouflage frame — one empty long-padding Continue frame
    // carrying the UUID, right after the header (xray delays it 500 ms via
    // a timer; we emit it immediately — same wire bytes, spec §9 deviation
    // 1). The camouflage consumes the writer's UUID, so the codec's own
    // frames never carry one.
    let rng = ring::rand::SystemRandom::new();
    let camo = VisionStream::<BoxStream>::camouflage_frame(&uuid, &rng);
    tokio::time::timeout(timeout, stream.write_all(&camo))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless vision camouflage write",
            limit: timeout,
        })??;

    // Wrap the secured stream in the padded codec; the response-header peel
    // (VlessClientStream) stays OUTSIDE it — the server's `[0,0]` arrives as
    // its own outer-TLS record before the first padded frame, the codec's
    // UUID gate passes it through, then the peel consumes it and unpadding
    // engages (spec §5.4).
    let vision = VisionStream::new(stream, uuid, rng);
    Ok(Box::new(VlessClientStream::new(Box::new(vision))))
}

/// True when the link actually runs TLS/REALITY — the check established in
/// the transports work (`transport/xhttp.rs`): the proto always carries a
/// `SecurityConfig`, even an empty one, so `security().is_some()` alone is
/// not the right test.
fn has_tls(ctx: &LinkContext) -> bool {
    ctx.security().and_then(SecurityConfig::type_str).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{Host, TargetAddr};
    use crate::context::NativeConnectParams;
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

    fn vless(flow: Option<&str>, transport: &str, security: bool) -> VlessConfig {
        let mut obj = serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": transport },
        });
        if let Some(f) = flow {
            obj["flow"] = serde_json::json!(f);
        }
        if security {
            obj["security"] = serde_json::json!({ "type": "tls", "sni": "example.com" });
        }
        serde_json::from_value(obj).expect("vless config parses")
    }

    fn ctx_for(cfg: &VlessConfig) -> LinkContext {
        let params = NativeConnectParams::new(
            xray_tui_proto::proto_spec::ProtocolConfig::Vless(cfg.clone()),
            EndpointEssentials::new("127.0.0.1", 443),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        LinkContext::new(
            params,
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        )
    }

    /// A client stream plus its live peer half (the header write must have
    /// somewhere to go; the guard-error tests fail before any write).
    fn stream_pair() -> (BoxStream, tokio::io::DuplexStream) {
        let (client, peer) = tokio::io::duplex(1024);
        (Box::new(client), peer)
    }

    #[tokio::test]
    async fn vision_requires_outer_tls_or_reality() {
        let cfg = vless(Some("xtls-rprx-vision"), "tcp", false);
        let (stream, _peer) = stream_pair();
        let err = connect(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("vision without TLS must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("TLS and REALITY")));
    }

    #[tokio::test]
    async fn vision_rejects_non_tcp_transport() {
        let cfg = vless(Some("xtls-rprx-vision"), "ws", true);
        let (stream, _peer) = stream_pair();
        let err = connect(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("vision over ws must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("TCP transport")));
    }

    #[tokio::test]
    async fn unknown_flow_stays_not_implemented() {
        let cfg = vless(Some("xtls-rprx-splice"), "tcp", true);
        let (stream, _peer) = stream_pair();
        let err = connect(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("unknown flow must fail");
        assert!(matches!(
            &err,
            NativeError::NotImplemented { feature } if feature.contains("xtls-rprx-splice")
        ));
    }

    #[tokio::test]
    async fn empty_and_missing_flow_take_plain_path() {
        // No TLS: the plain path writes the header and returns a tunnel —
        // the peer half stays alive so the header write lands. Vision would
        // reject this config; plain must not.
        for flow in [None, Some("")] {
            let cfg = vless(flow, "tcp", false);
            let (stream, _peer) = stream_pair();
            let result = connect(&ctx_for(&cfg), stream, &cfg).await;
            assert!(result.is_ok(), "flow {flow:?} must take the plain path");
        }
    }
}
