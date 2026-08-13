//! E2E: native VLESS matrix — network {tcp, ws, grpc} × TLS variant ×
//! core {xray, sing-box}. One generated test per (case, core).
//!
//! `clippy::future_not_send` is allowed file-wide: rstest 0.26.1 clears the
//! source fn's attributes when embedding it beside the generated tests, so no
//! fn-/statement-level `#[allow]` reaches the nested async fn that holds the
//! non-`Send` `CaseSpec` across the runner await. No narrower scope works.
#![cfg(feature = "native-e2e")]
#![allow(clippy::future_not_send)]

mod common;

use common::{certs, cores, echo, pick, tls_echo};
use rstest::rstest;
use xray_tui_native::e2e::{
    CaseSpec, Certs, CoreKind, CoreUnderTest, EchoServer, FingerprintTls,
    PlainServerRealityClientTls, RealityServerPlainClientTls, RealityTls, RealityWrongPbkTls,
    RealityWrongSidTls, TlsEchoServer, run_against,
};

#[rstest]
#[case::tcp_plain(CaseSpec::vless())]
#[case::tcp_chrome(CaseSpec::vless().with_tls(Box::new(FingerprintTls("chrome"))))]
#[case::tcp_reality(CaseSpec::vless().with_tls(Box::new(RealityTls::fresh())))]
#[case::tcp_reality_wrong_pbk(CaseSpec::vless().with_tls(Box::new(RealityWrongPbkTls::fresh())))]
#[case::tcp_reality_wrong_sid(CaseSpec::vless().with_tls(Box::new(RealityWrongSidTls::fresh())))]
#[case::tcp_plain_into_reality_server(
    CaseSpec::vless().with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::tcp_reality_into_plain_server(
    CaseSpec::vless().with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
#[case::ws_plain(CaseSpec::vless().with_network("ws"))]
#[case::ws_chrome(CaseSpec::vless().with_network("ws").with_tls(Box::new(FingerprintTls("chrome"))))]
#[ignore = "xray-core 26.3.27 rejects reality+ws inbounds (REALITY only supports RAW, XHTTP and gRPC); sing-box serves it"]
#[case::ws_reality(CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityTls::fresh())))]
#[case::ws_reality_wrong_pbk(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityWrongPbkTls::fresh()))
)]
#[case::ws_reality_wrong_sid(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityWrongSidTls::fresh()))
)]
#[ignore = "plain client into reality server: dest sees ws framing, not HTTP (transport unreachable pre-proxy)"]
#[case::ws_plain_into_reality_server(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::ws_reality_into_plain_server(
    CaseSpec::vless().with_network("ws").with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
#[case::grpc_plain(CaseSpec::vless().with_network("grpc"))]
#[case::grpc_chrome(CaseSpec::vless().with_network("grpc").with_tls(Box::new(FingerprintTls("chrome"))))]
#[case::grpc_reality(CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityTls::fresh())))]
#[case::grpc_reality_wrong_pbk(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityWrongPbkTls::fresh()))
)]
#[case::grpc_reality_wrong_sid(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityWrongSidTls::fresh()))
)]
#[ignore = "plain client into reality server: dest sees grpc framing, not HTTP (transport unreachable pre-proxy)"]
#[case::grpc_plain_into_reality_server(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(RealityServerPlainClientTls::fresh()))
)]
#[case::grpc_reality_into_plain_server(
    CaseSpec::vless().with_network("grpc").with_tls(Box::new(PlainServerRealityClientTls::fresh()))
)]
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
