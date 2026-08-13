//! E2E: native `VMess` matrix — security {aes-128-gcm, chacha20-poly1305} ×
//! network {tcp, ws, grpc} × TLS {standard, chrome, reality} × core.
//! One generated test per (case, core).
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
    Aes128GcmVariant, CaseSpec, Certs, Chacha20Poly1305Variant, CoreKind, CoreUnderTest,
    EchoServer, FingerprintTls, RealityTls, SecurityVariant, TlsEchoServer, TlsVariant,
    run_against,
};

/// `CaseSpec::vmess` takes the security variant by value; these helpers keep
/// the `#[case]` rows one-liners.
fn vmess(sec: impl SecurityVariant + 'static, net: &'static str) -> CaseSpec {
    CaseSpec::vmess(sec).with_network(net)
}

fn vmess_tls(
    sec: impl SecurityVariant + 'static,
    net: &'static str,
    tls: Box<dyn TlsVariant>,
) -> CaseSpec {
    vmess(sec, net).with_tls(tls)
}

#[rstest]
#[case::tcp_aes128gcm(vmess(Aes128GcmVariant, "tcp"))]
#[case::tcp_chacha20(vmess(Chacha20Poly1305Variant, "tcp"))]
#[case::tcp_aes128gcm_chrome(vmess_tls(
    Aes128GcmVariant,
    "tcp",
    Box::new(FingerprintTls("chrome"))
))]
#[case::tcp_chacha20_chrome(vmess_tls(
    Chacha20Poly1305Variant,
    "tcp",
    Box::new(FingerprintTls("chrome"))
))]
#[case::tcp_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "tcp", Box::new(RealityTls::fresh())))]
#[case::tcp_chacha20_reality(vmess_tls(
    Chacha20Poly1305Variant,
    "tcp",
    Box::new(RealityTls::fresh())
))]
#[case::ws_aes128gcm(vmess(Aes128GcmVariant, "ws"))]
#[case::ws_chacha20(vmess(Chacha20Poly1305Variant, "ws"))]
#[case::ws_aes128gcm_chrome(vmess_tls(Aes128GcmVariant, "ws", Box::new(FingerprintTls("chrome"))))]
#[case::ws_chacha20_chrome(vmess_tls(
    Chacha20Poly1305Variant,
    "ws",
    Box::new(FingerprintTls("chrome"))
))]
#[case::ws_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "ws", Box::new(RealityTls::fresh())))]
#[case::ws_chacha20_reality(vmess_tls(
    Chacha20Poly1305Variant,
    "ws",
    Box::new(RealityTls::fresh())
))]
#[case::grpc_aes128gcm(vmess(Aes128GcmVariant, "grpc"))]
#[case::grpc_chacha20(vmess(Chacha20Poly1305Variant, "grpc"))]
#[case::grpc_aes128gcm_chrome(vmess_tls(
    Aes128GcmVariant,
    "grpc",
    Box::new(FingerprintTls("chrome"))
))]
#[case::grpc_chacha20_chrome(vmess_tls(
    Chacha20Poly1305Variant,
    "grpc",
    Box::new(FingerprintTls("chrome"))
))]
#[case::grpc_aes128gcm_reality(vmess_tls(Aes128GcmVariant, "grpc", Box::new(RealityTls::fresh())))]
#[case::grpc_chacha20_reality(vmess_tls(
    Chacha20Poly1305Variant,
    "grpc",
    Box::new(RealityTls::fresh())
))]
#[tokio::test]
async fn vmess_against_cores(
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
        .expect("vmess e2e failed");
}
