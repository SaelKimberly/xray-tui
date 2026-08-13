//! E2E: native VLESS matrix — network {tcp, ws, grpc, httpupgrade} × TLS
//! variant × core {xray, sing-box}, plus xray-only xhttp (splithttp) rows.
//! One generated test per (case, core).
//!
//! `clippy::future_not_send` is allowed file-wide: rstest 0.26.1 clears the
//! source fn's attributes when embedding it beside the generated tests, so no
//! fn-/statement-level `#[allow]` reaches the nested async fn that holds the
//! non-`Send` `CaseSpec` across the runner await. No narrower scope works.
#![cfg(feature = "native-e2e")]
#![allow(clippy::future_not_send)]

mod common;

use common::{
    certs, cores, echo, fp, pick, plain_server_reality_client, reality,
    reality_server_plain_client, reality_wrong_pbk, reality_wrong_sid, tls_echo,
};
use rstest::rstest;
use xray_tui_native::e2e::{
    AppKind, CaseSpec, Certs, CoreKind, CoreUnderTest, EchoServer, Flow, TlsEchoServer, TlsVariant,
    run_against,
};

/// `CaseSpec::vless()` plus the network axis; these helpers keep the
/// `#[case]` rows one-liners (mirror of the vmess matrix helpers).
fn vless(net: &'static str) -> CaseSpec {
    CaseSpec::vless().with_network(net)
}

fn vless_tls(net: &'static str, tls: Box<dyn TlsVariant>) -> CaseSpec {
    vless(net).with_tls(tls)
}

/// xhttp row with an explicit client-side mode (the client's
/// `XHttpConfig.mode` selects the dialect on the wire; the server runs auto
/// and accepts both).
fn vless_xhttp(mode: &'static str) -> CaseSpec {
    vless("xhttp").with_xhttp_mode(mode)
}

fn vless_xhttp_tls(mode: &'static str, tls: Box<dyn TlsVariant>) -> CaseSpec {
    vless_tls("xhttp", tls).with_xhttp_mode(mode)
}

/// VLESS vision rows (spec §7.4): `flow = xtls-rprx-vision`, tcp network,
/// the default certificate TLS. `app` selects the app-side probe: plain
/// HTTP over the tunnel, or an inner TLS session through the tunnel.
fn vision(app: AppKind) -> CaseSpec {
    CaseSpec::vless().with_flow(Flow::Vision).with_app(app)
}

/// VLESS vision rows with an explicit TLS variant (REALITY etc.).
fn vision_tls(app: AppKind, tls: Box<dyn TlsVariant>) -> CaseSpec {
    vision(app).with_tls(tls)
}

#[rstest]
#[case::tcp_plain(vless("tcp"))]
#[case::tcp_chrome(vless_tls("tcp", fp("chrome")))]
#[case::tcp_reality(vless_tls("tcp", reality()))]
#[case::tcp_reality_wrong_pbk(vless_tls("tcp", reality_wrong_pbk()))]
#[case::tcp_reality_wrong_sid(vless_tls("tcp", reality_wrong_sid()))]
#[case::tcp_plain_into_reality_server(vless_tls("tcp", reality_server_plain_client()))]
#[case::tcp_reality_into_plain_server(vless_tls("tcp", plain_server_reality_client()))]
#[case::ws_plain(vless("ws"))]
#[case::ws_chrome(vless_tls("ws", fp("chrome")))]
#[case::ws_reality_wrong_pbk(vless_tls("ws", reality_wrong_pbk()))]
#[case::ws_reality_wrong_sid(vless_tls("ws", reality_wrong_sid()))]
#[ignore = "plain client into reality server: dest sees ws framing, not HTTP (transport unreachable pre-proxy)"]
#[case::ws_plain_into_reality_server(vless_tls("ws", reality_server_plain_client()))]
#[case::ws_reality_into_plain_server(vless_tls("ws", plain_server_reality_client()))]
#[case::grpc_plain(vless("grpc"))]
#[case::grpc_chrome(vless_tls("grpc", fp("chrome")))]
#[case::grpc_reality(vless_tls("grpc", reality()))]
#[case::grpc_reality_wrong_pbk(vless_tls("grpc", reality_wrong_pbk()))]
#[case::grpc_reality_wrong_sid(vless_tls("grpc", reality_wrong_sid()))]
#[ignore = "plain client into reality server: dest sees grpc framing, not HTTP (transport unreachable pre-proxy)"]
#[case::grpc_plain_into_reality_server(vless_tls("grpc", reality_server_plain_client()))]
#[case::grpc_reality_into_plain_server(vless_tls("grpc", plain_server_reality_client()))]
#[case::httpupgrade_plain(vless("httpupgrade"))]
#[case::httpupgrade_chrome(vless_tls("httpupgrade", fp("chrome")))]
// VLESS vision flow axis (spec §7.4): `xtls-rprx-vision` × {tls, reality} ×
// {inner-tls, plain} — the inner-tls rows drive the Direct-splice path (the
// app establishes a real TLS 1.3 session through the tunnel to a rustls
// echo target); the plain rows exercise the End (non-TLS) path.
#[case::vision_tls_plain(vision(AppKind::Plain))]
#[case::vision_tls_inner(vision(AppKind::InnerTls))]
#[case::vision_reality_plain(vision_tls(AppKind::Plain, reality()))]
#[case::vision_reality_inner(vision_tls(AppKind::InnerTls, reality()))]
#[tokio::test]
async fn vless_against_cores(
    #[case] case: CaseSpec,
    #[values(CoreKind::Xray, CoreKind::SingBox)] core: CoreKind,
    cores: &(CoreUnderTest, CoreUnderTest),
    certs: &Certs,
    echo: EchoServer,
    tls_echo: TlsEchoServer,
) {
    // `#[once]` fixtures inject as `&T` (cores, certs); per-test fixtures
    // inject as OWNED `T` (echo, tls_echo) — borrow them for the runner.
    run_against(&case, pick(cores, core), certs, &echo, &tls_echo)
        .await
        .expect("vless e2e failed");
}

/// Single-core cases: the named core accepts the configuration, the other
/// rejects it at startup (xray-core 26.3.27 refuses REALITY-over-WebSocket
/// inbounds — "REALITY only supports RAW, XHTTP and gRPC"; sing-box serves
/// it). Each row names its core explicitly instead of `#[values]`.
#[rstest]
#[case::ws_reality_singbox(vless_tls("ws", reality()), CoreKind::SingBox)]
// xray-core refuses REALITY over httpupgrade inbounds — "REALITY only
// supports RAW, XHTTP and gRPC" — so the reality row runs on sing-box only.
#[case::httpupgrade_reality_singbox(vless_tls("httpupgrade", reality()), CoreKind::SingBox)]
// xhttp (splithttp) is xray-only: sing-box's own dialect is the v2rayhttp
// transport (a different plan task). Packet-up over h1 (no TLS) is covered
// by the hermetic unit test; these rows exercise the h2 arm (TLS) and the
// REALITY path (xray's splithttp server accepts packet-up under REALITY —
// the mode gate is on the client side; verified empirically below).
#[case::xhttp_packet_plain(vless("xhttp"), CoreKind::Xray)]
#[case::xhttp_packet_chrome(vless_tls("xhttp", fp("chrome")), CoreKind::Xray)]
#[case::xhttp_packet_reality(vless_tls("xhttp", reality()), CoreKind::Xray)]
// stream-up: the client config carries mode "stream-up" (the client-side
// mode drives the dialect); both rows run over h2 (TLS → h2). The h1 arm
// is covered by the hermetic unit test.
#[case::xhttp_stream_plain(vless_xhttp("stream-up"), CoreKind::Xray)]
#[case::xhttp_stream_chrome(vless_xhttp_tls("stream-up", fp("chrome")), CoreKind::Xray)]
// v2rayhttp (sing-box `type: http`) is sing-box-only: xray-core removed the
// h2 transport in 26.x. One row over h2 + the chrome fingerprint (the
// client JSON carries the fp so the engine fingerprints; ALPN h2 comes from
// the context's transport-implied alpn arm).
#[case::v2rayhttp_chrome(vless_tls("h2", fp("chrome")), CoreKind::SingBox)]
#[tokio::test]
async fn vless_single_core(
    #[case] case: CaseSpec,
    #[case] core: CoreKind,
    cores: &(CoreUnderTest, CoreUnderTest),
    certs: &Certs,
    echo: EchoServer,
    tls_echo: TlsEchoServer,
) {
    run_against(&case, pick(cores, core), certs, &echo, &tls_echo)
        .await
        .expect("vless single-core e2e failed");
}
