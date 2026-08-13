//! E2E: native VLESS matrix — network {tcp, ws, grpc, httpupgrade} × TLS
//! variant × core {xray, sing-box}. One generated test per (case, core).
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
    CaseSpec, Certs, CoreKind, CoreUnderTest, EchoServer, TlsEchoServer, TlsVariant, run_against,
};

/// `CaseSpec::vless()` plus the network axis; these helpers keep the
/// `#[case]` rows one-liners (mirror of the vmess matrix helpers).
fn vless(net: &'static str) -> CaseSpec {
    CaseSpec::vless().with_network(net)
}

fn vless_tls(net: &'static str, tls: Box<dyn TlsVariant>) -> CaseSpec {
    vless(net).with_tls(tls)
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
