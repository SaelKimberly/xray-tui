//! VLESS — the reference protocol for the native core.

use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::{SecurityConfig, VlessConfig};

use crate::BoxStream;
use crate::addr::{Host, TargetAddr};
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vless::stream::VlessClientStream;
use crate::protocol::vless::vision::{FLOW_XTLS_RPRX_VISION, VisionStream};

pub mod header;
pub(crate) mod mux;
pub mod packet;
pub(crate) mod packetaddr;
pub mod stream;
pub(crate) mod udp;
pub(crate) mod vision;

pub use mux::{MuxClient, MuxTarget, SessionStream};
pub use packet::{PacketConn, PacketMode};

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
/// Guarded preconditions, checked before any protocol-phase I/O (the
/// header write):
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
    // (VlessClientStream) stays OUTSIDE it. The server's `[0,0]` response
    // header is buffered (EncodeResponseHeader + SetFlushNext) and flushed
    // TOGETHER with the first padded frame in ONE outer-TLS record
    // (`[0,0][uuid][cmd]...`); the codec's Unpadder skips the leading
    // `[0,0]` before the UUID gate (vision.rs), and the lenient peel hands
    // the first non-0x00 content byte back as payload (spec §4.6, §5.4).
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

/// UDP/flow guard: the vision flow cannot carry UDP packets (xray rewrites
/// UDP to the XUDP mux path under vision; until SP3 adds XUDP the honest
/// behavior is a client-side rejection — spec §2, §6).
fn check_udp_allowed(flow: Option<&str>, udp: Option<PacketMode>) -> Result<(), NativeError> {
    if udp.is_some() && flow == Some(FLOW_XTLS_RPRX_VISION) {
        return Err(NativeError::NotImplemented {
            feature: "VLESS vision does not support UDP (no XUDP mux)".into(),
        });
    }
    Ok(())
}

/// UDP connect (protocol phase, parallel to [`connect_plain`]).
///
/// The chain runs dial → security → transport exactly as for TCP; this
/// writes the request header with `command = 0x02` and the UDP destination
/// (port-first, spec §4.1), then wraps the tunnel in the packet-framed
/// [`PacketConn`] for the configured mode. The vision+UDP guard
/// ([`check_udp_allowed`]) runs before any protocol-phase I/O (the header
/// write).
pub async fn connect_udp(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VlessConfig,
) -> Result<PacketConn<BoxStream>, NativeError> {
    let uuid = header::uuid_bytes(&cfg.uuid)?;
    check_udp_allowed(cfg.flow.as_deref(), ctx.params.udp)?;
    let mode = ctx.params.udp.ok_or_else(|| {
        NativeError::Config("vless udp connect requires params.udp (None = TCP path)".into())
    })?;
    match cfg.flow.as_deref() {
        None | Some("") => connect_udp_plain(ctx, stream, uuid, mode).await,
        Some(other) => Err(NativeError::NotImplemented {
            feature: format!("vless flow {other}"),
        }),
    }
}

/// Plain UDP connect: request header without addons (`cmd = CMD_UDP`), then
/// the packet-framed tunnel.
async fn connect_udp_plain(
    ctx: &LinkContext,
    stream: BoxStream,
    uuid: [u8; 16],
    mode: PacketMode,
) -> Result<PacketConn<BoxStream>, NativeError> {
    let request =
        header::encode_request(&uuid, &udp_header_target(ctx, mode)?, header::CMD_UDP, None)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless udp request write",
            limit: timeout,
        })??;

    Ok(PacketConn::new(stream, mode))
}

/// The header destination for the UDP command.
///
/// Raw mode: the UDP target itself (port-first). `PacketAddr` mode: the magic
/// fqdn with port 0 — the header carries no real destination, the
/// per-packet magic-address headers do (spec §4.3; sing-vmess
/// `DialEarlyPacketConn(conn, Socksaddr{Fqdn: magic})`). Domain targets are
/// rejected in packetaddr mode (spec §4.3, sing's `ErrFqdnUnsupported`).
fn udp_header_target(ctx: &LinkContext, mode: PacketMode) -> Result<TargetAddr, NativeError> {
    match mode {
        PacketMode::Raw => Ok(ctx.target.clone()),
        PacketMode::PacketAddr => {
            if matches!(ctx.target.host, Host::Domain(_)) {
                return Err(NativeError::Config(
                    "packetaddr: domain destination is not supported".into(),
                ));
            }
            Ok(TargetAddr::new(
                Host::Domain(packetaddr::MAGIC.to_string()),
                0,
            ))
        }
    }
}

/// Mux connect (protocol phase, parallel to [`connect_udp`]).
///
/// The chain runs dial → security → transport exactly as for TCP; this
/// writes the request header with `command = 0x03` and the fixed
/// `v1.mux.cool` destination (spec §4.1), then wraps the tunnel in the
/// [`MuxClient`] multiplexer. The wire carries NO destination bytes for
/// the mux command (xray's `EncodeRequestHeader` skips the address; the
/// server derives the magic fqdn from the command byte) — the target
/// built by [`mux_header_target`] is the semantic destination, not
/// encoded bytes. `params.udp` must be `None`: the mux tunnel carries
/// TCP sessions (UDP over mux / XUDP is SP3).
pub async fn connect_mux(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VlessConfig,
) -> Result<MuxClient<BoxStream>, NativeError> {
    if ctx.params.udp.is_some() {
        return Err(NativeError::Config(
            "vless mux connect is TCP-only (UDP over mux / XUDP is not implemented; use connect_udp)"
                .into(),
        ));
    }
    let uuid = header::uuid_bytes(&cfg.uuid)?;
    match cfg.flow.as_deref() {
        None | Some("") => connect_mux_plain(ctx, stream, uuid).await,
        Some(FLOW_XTLS_RPRX_VISION) => connect_mux_vision(ctx, stream, uuid).await,
        Some(other) => Err(NativeError::NotImplemented {
            feature: format!("vless flow {other}"),
        }),
    }
}

/// Plain mux connect: request header without addons (`cmd = CMD_MUX`),
/// then the multiplexer over the peeled tunnel.
async fn connect_mux_plain(
    _ctx: &LinkContext,
    stream: BoxStream,
    uuid: [u8; 16],
) -> Result<MuxClient<BoxStream>, NativeError> {
    let request = header::encode_request(&uuid, &mux_header_target(), header::CMD_MUX, None)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless mux request write",
            limit: timeout,
        })??;

    // The mux frame stream rides the peeled tunnel: the server's `[0,0]`
    // response header is consumed on the first read, then frames flow.
    let peeled: BoxStream = Box::new(VlessClientStream::new(stream));
    Ok(MuxClient::new(peeled))
}

/// Vision (`xtls-rprx-vision`) mux connect (spec §5.3): `cmd = CMD_MUX`
/// with the flow addon, the camouflage frame, then the vision-padded
/// multiplexer.
///
/// Guarded preconditions mirror [`connect_vision`], checked before any
/// protocol-phase I/O:
/// - outer security is TLS or REALITY (`has_tls()`) — mirroring xray's
///   rejection of bare vision;
/// - the transport is raw TCP — vision requires the socket for the Direct
///   handoff (spec §5.1; ws/grpc/xhttp framing is incompatible).
///
/// Unlike the TCP command path there is no command guard: xray servers
/// accept vision+mux (`inbound.go` splices `RequestCommandMux` under flow
/// XRV with `CanSpliceCopy = 3`).
async fn connect_mux_vision(
    ctx: &LinkContext,
    stream: BoxStream,
    uuid: [u8; 16],
) -> Result<MuxClient<BoxStream>, NativeError> {
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
        &mux_header_target(),
        header::CMD_MUX,
        header::encode_addons(Some(FLOW_XTLS_RPRX_VISION)).as_deref(),
    )?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless mux request write",
            limit: timeout,
        })??;

    // Step 3: the camouflage frame — one empty long-padding Continue frame
    // carrying the UUID, right after the header (same wire bytes as the
    // TCP vision path, spec §4.6 step 3 deviation 1).
    let rng = ring::rand::SystemRandom::new();
    let camo = VisionStream::<BoxStream>::camouflage_frame(&uuid, &rng);
    tokio::time::timeout(timeout, stream.write_all(&camo))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless mux vision camouflage write",
            limit: timeout,
        })??;

    // Vision codec, then the response-header peel, then the multiplexer —
    // the same composition as the TCP vision path (`connect_vision`).
    let vision = VisionStream::new(stream, uuid, rng);
    let peeled: BoxStream = Box::new(VlessClientStream::new(Box::new(vision)));
    Ok(MuxClient::new(peeled))
}

/// The header destination for the mux command: the fixed `v1.mux.cool`
/// fqdn and port (spec §4.1 — xray `common/mux/client.go`
/// `muxCoolAddress` / `muxCoolPort`). Passed to
/// [`header::encode_request`] as the mux command's target; the `CMD_MUX`
/// arm does not encode it on the wire.
fn mux_header_target() -> TargetAddr {
    TargetAddr::new(Host::Domain(mux::MUX_DEST.to_string()), mux::MUX_PORT)
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

    /// A `LinkContext` with the UDP mode set, otherwise like [`ctx_for`].
    fn udp_ctx_for(cfg: &VlessConfig, mode: PacketMode) -> LinkContext {
        let mut ctx = ctx_for(cfg);
        ctx.params.udp = Some(mode);
        ctx
    }

    #[tokio::test]
    async fn vision_udp_is_rejected() {
        // Vision + UDP is rejected wholesale (no XUDP mux until SP3) in
        // BOTH packet modes, before any I/O — the guard, exercised through
        // the real connect_udp entry.
        for mode in [PacketMode::Raw, PacketMode::PacketAddr] {
            let cfg = vless(Some("xtls-rprx-vision"), "tcp", true);
            let (stream, _peer) = stream_pair();
            let err = connect_udp(&udp_ctx_for(&cfg, mode), stream, &cfg)
                .await
                .err()
                .expect("vision + UDP must fail");
            assert!(
                matches!(&err, NativeError::NotImplemented { feature } if feature.contains("XUDP")),
                "mode {mode:?}: {err}"
            );
        }
    }

    #[test]
    fn check_udp_allowed_guard_matrix() {
        // The guard rejects ONLY vision + UDP; every other combination
        // passes (plain flows, unknown flows, vision on the TCP path).
        for mode in [Some(PacketMode::Raw), Some(PacketMode::PacketAddr)] {
            let err = check_udp_allowed(Some(FLOW_XTLS_RPRX_VISION), mode).unwrap_err();
            assert!(
                matches!(&err, NativeError::NotImplemented { feature } if feature.contains("XUDP"))
            );
        }
        assert!(check_udp_allowed(None, Some(PacketMode::Raw)).is_ok());
        assert!(check_udp_allowed(Some(""), Some(PacketMode::PacketAddr)).is_ok());
        assert!(check_udp_allowed(Some("xtls-rprx-splice"), Some(PacketMode::Raw)).is_ok());
        assert!(check_udp_allowed(Some(FLOW_XTLS_RPRX_VISION), None).is_ok());
        assert!(check_udp_allowed(None, None).is_ok());
    }

    #[tokio::test]
    async fn udp_connect_requires_udp_mode() {
        // connect_udp with params.udp = None is a config error (None = the
        // TCP path), never a silent default.
        let cfg = vless(None, "tcp", false);
        let (stream, _peer) = stream_pair();
        let err = connect_udp(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("udp: None must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("params.udp")));
    }

    #[tokio::test]
    async fn udp_connect_writes_cmd_udp_header() {
        // Raw mode: version, uuid, addon_len 0, cmd 0x02, then the
        // port-first UDP dest — identical to TCP except the command byte
        // (spec §4.1). The returned PacketConn is live over the tunnel.
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let cfg = vless(None, "tcp", false);
        let mut ctx = ctx_for(&cfg);
        ctx.target = TargetAddr::new(Host::new("1.2.3.4"), 8080);
        ctx.params.udp = Some(PacketMode::Raw);
        let (stream, mut peer) = stream_pair();
        let mut conn = connect_udp(&ctx, stream, &cfg)
            .await
            .expect("raw udp connect");

        let mut expected = vec![0x00];
        expected.extend_from_slice(&header::uuid_bytes(&cfg.uuid).unwrap());
        expected.push(0x00); // addon_len 0
        expected.push(0x02); // cmd UDP
        expected.extend_from_slice(&[0x1f, 0x90, 0x01, 1, 2, 3, 4]); // port-first dest
        let mut got = [0u8; 26];
        peer.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, &expected[..]);

        // One datagram roundtrip through the tunnel.
        conn.send(None, b"ping").await.unwrap();
        let mut frame = [0u8; 6];
        peer.read_exact(&mut frame).await.unwrap();
        assert_eq!(&frame, &[0x00, 0x04, b'p', b'i', b'n', b'g']);
        peer.write_all(&[0x00, 0x00]).await.unwrap(); // response header
        peer.write_all(&[0x00, 0x02, b'o', b'k']).await.unwrap();
        let (dest, payload) = conn.recv().await.unwrap().unwrap();
        assert_eq!(dest, None);
        assert_eq!(payload, b"ok");
    }

    #[tokio::test]
    async fn udp_packetaddr_header_is_magic_fqdn() {
        // PacketAddr mode: the header destination is the magic fqdn with
        // port 0 — the UDP target stays out of the header; per-packet
        // destinations carry it (spec §4.3).
        use tokio::io::AsyncReadExt;

        let cfg = vless(None, "tcp", false);
        let mut ctx = ctx_for(&cfg);
        ctx.target = TargetAddr::new(Host::new("1.2.3.4"), 8080);
        ctx.params.udp = Some(PacketMode::PacketAddr);
        let (stream, mut peer) = stream_pair();
        let mut conn = connect_udp(&ctx, stream, &cfg)
            .await
            .expect("packetaddr udp connect");

        let mut expected = vec![0x00];
        expected.extend_from_slice(&header::uuid_bytes(&cfg.uuid).unwrap());
        expected.push(0x00); // addon_len 0
        expected.push(0x02); // cmd UDP
        expected.extend_from_slice(&[0x00, 0x00]); // port 0
        expected.push(0x02); // domain atyp
        expected.push(u8::try_from(packetaddr::MAGIC.len()).expect("magic fqdn fits one byte"));
        expected.extend_from_slice(packetaddr::MAGIC.as_bytes());
        let mut got = [0u8; 1 + 16 + 1 + 1 + 2 + 1 + 1 + 25];
        peer.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, &expected[..]);

        // The packetaddr tunnel is live: a per-packet destination travels
        // with the datagram (spec §4.3).
        let dest = "1.2.3.4:8080".parse().unwrap();
        conn.send(Some(dest), b"p").await.unwrap();
        let mut frame = vec![0u8; 2 + packetaddr::encode_dest(dest).len() + 1];
        peer.read_exact(&mut frame).await.unwrap();
        let mut want = packetaddr::encode_dest(dest);
        want.push(b'p');
        // Frame length = 1 (atyp) + 4 (addr) + 2 (port) + 1 (payload) = 8 —
        // no magic prefix in the frame (sing serializer; Task 5 report).
        let mut expected_frame = vec![0x00, 0x08];
        expected_frame.extend_from_slice(&want);
        assert_eq!(&frame, &expected_frame[..]);
    }

    #[tokio::test]
    async fn udp_packetaddr_rejects_domain_target() {
        // packetaddr fqdn targets are unsupported (spec §4.3): a domain UDP
        // target errors before any header write (sing's ErrFqdnUnsupported).
        let cfg = vless(None, "tcp", false);
        let (stream, _peer) = stream_pair();
        let err = connect_udp(&udp_ctx_for(&cfg, PacketMode::PacketAddr), stream, &cfg)
            .await
            .err()
            .expect("packetaddr + domain target must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("packetaddr")));
    }

    #[tokio::test]
    async fn mux_rejects_udp_params() {
        // Mux is TCP-only: params.udp set → config error before any I/O
        // (UDP over mux / XUDP is SP3).
        let cfg = vless(None, "tcp", false);
        let (stream, _peer) = stream_pair();
        let err = connect_mux(&udp_ctx_for(&cfg, PacketMode::Raw), stream, &cfg)
            .await
            .err()
            .expect("mux + udp params must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("TCP-only")));
    }

    #[tokio::test]
    async fn mux_plain_writes_cmd_mux_header() {
        // Header: version, uuid, addon_len 0, cmd 0x03 — and NOTHING after
        // (the mux command carries no destination bytes; the wire ends at
        // the command byte). The returned MuxClient is live over the
        // tunnel: an eager New frame + a Keep data roundtrip.
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let cfg = vless(None, "tcp", false);
        let (stream, mut peer) = stream_pair();
        let client = connect_mux(&ctx_for(&cfg), stream, &cfg)
            .await
            .expect("plain mux connect");

        let mut expected = vec![0x00];
        expected.extend_from_slice(&header::uuid_bytes(&cfg.uuid).unwrap());
        expected.push(0x00); // addon_len 0
        expected.push(0x03); // cmd MUX
        let mut got = [0u8; 19];
        peer.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, &expected[..]);

        // Server response header, then a session open through the mux.
        peer.write_all(&[0x00, 0x00]).await.unwrap();
        let mut session = client
            .open_session(MuxTarget::TcpDomain("example.com".into(), 80))
            .await
            .expect("open session");

        // The eager New frame arrives at the server (spec §8 deviation 1).
        let frame = mux::read_frame(&mut peer)
            .await
            .unwrap()
            .expect("eager new frame");
        assert_eq!(frame.session_id, 1);
        assert_eq!(frame.status, mux::STATUS_NEW);
        assert_eq!(
            frame.target,
            Some(MuxTarget::TcpDomain("example.com".into(), 80))
        );

        // App data roundtrips as a Keep frame with the payload.
        session.write_all(b"ping").await.unwrap();
        let frame = mux::read_frame(&mut peer)
            .await
            .unwrap()
            .expect("keep frame");
        assert_eq!(frame.session_id, 1);
        assert_eq!(frame.status, mux::STATUS_KEEP);
        assert_eq!(frame.payload.as_ref(), b"ping");

        // The server's Keep reply reaches the session stream.
        mux::write_frame(
            &mut peer,
            &mux::Frame {
                session_id: 1,
                status: mux::STATUS_KEEP,
                option: mux::OPT_DATA,
                target: None,
                payload: bytes::Bytes::from_static(b"ok"),
            },
        )
        .await
        .unwrap();
        let mut buf = [0u8; 2];
        session.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ok");
    }

    #[tokio::test]
    async fn mux_vision_requires_outer_tls_or_reality() {
        let cfg = vless(Some("xtls-rprx-vision"), "tcp", false);
        let (stream, _peer) = stream_pair();
        let err = connect_mux(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("vision mux without TLS must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("TLS and REALITY")));
    }

    #[tokio::test]
    async fn mux_vision_rejects_non_tcp_transport() {
        let cfg = vless(Some("xtls-rprx-vision"), "ws", true);
        let (stream, _peer) = stream_pair();
        let err = connect_mux(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("vision mux over ws must fail");
        assert!(matches!(&err, NativeError::Config(msg) if msg.contains("TCP transport")));
    }

    #[tokio::test]
    async fn mux_unknown_flow_stays_not_implemented() {
        let cfg = vless(Some("xtls-rprx-splice"), "tcp", true);
        let (stream, _peer) = stream_pair();
        let err = connect_mux(&ctx_for(&cfg), stream, &cfg)
            .await
            .err()
            .expect("unknown flow must fail");
        assert!(matches!(
            &err,
            NativeError::NotImplemented { feature } if feature.contains("xtls-rprx-splice")
        ));
    }

    #[tokio::test]
    async fn mux_vision_writes_addon_and_camouflage() {
        // Vision mux header: the flow addon (addon_len 18) + cmd 0x03, then
        // the camouflage frame ([uuid][CMD_CONTINUE][clen=0][long plen][zeros])
        // right after — the vision wire precedes the mux frames (spec §5.3).
        use tokio::io::AsyncReadExt;

        let cfg = vless(Some("xtls-rprx-vision"), "tcp", true);
        // The camouflage frame (up to ~1.4 KiB) is written before the test
        // starts reading, so the duplex buffer must hold header + frame.
        let (stream, mut peer) = tokio::io::duplex(4096);
        let stream: BoxStream = Box::new(stream);
        let client = connect_mux(&ctx_for(&cfg), stream, &cfg)
            .await
            .expect("vision mux connect");
        let uuid = header::uuid_bytes(&cfg.uuid).unwrap();
        let addons = header::encode_addons(Some(FLOW_XTLS_RPRX_VISION)).unwrap();

        let mut expected = vec![0x00];
        expected.extend_from_slice(&uuid);
        expected.push(18); // addon_len
        expected.extend_from_slice(&addons);
        expected.push(0x03); // cmd MUX
        let mut got = vec![0u8; 1 + 16 + 1 + 18 + 1];
        peer.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, &expected[..]);

        let mut head = [0u8; 21];
        peer.read_exact(&mut head).await.unwrap();
        assert_eq!(&head[..16], &uuid);
        assert_eq!(head[16], vision::CMD_CONTINUE);
        assert_eq!(&head[17..19], &[0x00, 0x00]); // clen 0 (empty content)
        let plen = usize::from(u16::from_be_bytes([head[19], head[20]]));
        // Long padding for empty content: 900 + rand(0..500).
        assert!((900..=1399).contains(&plen), "plen {plen}");
        let mut pad = vec![0u8; plen];
        peer.read_exact(&mut pad).await.unwrap();
        assert!(pad.iter().all(|&b| b == 0));

        drop(client);
    }
}
