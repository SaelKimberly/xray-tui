//! E2E: native VMess+TLS+TCP payload variants through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{
    Aes128GcmVariant, CaseSpec, Chacha20Poly1305Variant, run_against_cores,
};

#[tokio::test]
async fn vmess_tcp_tls_aes128gcm_against_cores() {
    run_against_cores(&CaseSpec::vmess(Aes128GcmVariant))
        .await
        .expect("vmess/aes-128-gcm e2e failed");
}

#[tokio::test]
async fn vmess_tcp_tls_chacha20poly1305_against_cores() {
    run_against_cores(&CaseSpec::vmess(Chacha20Poly1305Variant))
        .await
        .expect("vmess/chacha20-poly1305 e2e failed");
}
