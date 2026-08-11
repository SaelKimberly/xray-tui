//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{CaseSpec, FingerprintTls, RealityTls, run_against_cores};

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
