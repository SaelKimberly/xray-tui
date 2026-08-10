//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{CaseSpec, run_against_cores};

#[tokio::test]
async fn vless_tcp_tls_against_cores() {
    run_against_cores(&CaseSpec::vless())
        .await
        .expect("vless e2e failed");
}
