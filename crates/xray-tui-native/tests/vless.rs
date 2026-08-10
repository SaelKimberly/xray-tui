//! E2E: native VLESS+TLS+TCP through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{CoreKind, CoreUnderTest, run};
use xray_tui_native::protocol::vless::e2e::VlessCase;

const SINGBOX_VERSION: &str = "1.13.16";
const XRAY_VERSION: &str = "26.3.27";

#[tokio::test]
async fn vless_tcp_tls_against_xray() {
    let core = CoreUnderTest::resolve(CoreKind::Xray, XRAY_VERSION).unwrap();
    run(&VlessCase, &core).await.expect("vless/xray e2e failed");
}

#[tokio::test]
async fn vless_tcp_tls_against_singbox() {
    let core = CoreUnderTest::resolve(CoreKind::SingBox, SINGBOX_VERSION).unwrap();
    run(&VlessCase, &core)
        .await
        .expect("vless/singbox e2e failed");
}
