//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{
    CaseSpec, FingerprintTls, PlainServerRealityClientTls, RealityServerPlainClientTls, RealityTls,
    RealityWrongPbkTls, RealityWrongSidTls, run_against_cores,
};

#[tokio::test]
async fn vless_tcp_tls_against_cores() {
    run_against_cores(&CaseSpec::vless())
        .await
        .expect("vless e2e failed");
}

#[tokio::test]
async fn vless_tls_chrome_against_cores() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(FingerprintTls("chrome"))))
        .await
        .expect("vless tls chrome e2e failed");
}

#[tokio::test]
async fn vless_reality_against_cores() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityTls::fresh())))
        .await
        .expect("vless reality e2e failed");
}

#[tokio::test]
async fn vless_reality_wrong_pbk_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityWrongPbkTls::fresh())))
        .await
        .expect("vless reality wrong-pbk e2e failed");
}

#[tokio::test]
async fn vless_reality_wrong_sid_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityWrongSidTls::fresh())))
        .await
        .expect("vless reality wrong-sid e2e failed");
}

#[tokio::test]
async fn vless_plain_client_through_reality_server_is_stealth() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityServerPlainClientTls::fresh())))
        .await
        .expect("vless plain-probe e2e failed");
}

#[tokio::test]
async fn vless_reality_client_into_plain_server_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(PlainServerRealityClientTls::fresh())))
        .await
        .expect("vless reality-into-plain e2e failed");
}

#[tokio::test]
async fn vless_ws_against_cores() {
    run_against_cores(&CaseSpec::vless().with_network("ws"))
        .await
        .expect("vless ws e2e failed");
}

#[tokio::test]
async fn vless_ws_chrome_against_cores() {
    run_against_cores(
        &CaseSpec::vless()
            .with_network("ws")
            .with_tls(Box::new(FingerprintTls("chrome"))),
    )
    .await
    .expect("vless ws chrome e2e failed");
}

#[tokio::test]
async fn vless_grpc_against_cores() {
    run_against_cores(&CaseSpec::vless().with_network("grpc"))
        .await
        .expect("vless grpc e2e failed");
}

#[tokio::test]
async fn vless_grpc_chrome_against_cores() {
    run_against_cores(
        &CaseSpec::vless()
            .with_network("grpc")
            .with_tls(Box::new(FingerprintTls("chrome"))),
    )
    .await
    .expect("vless grpc chrome e2e failed");
}
