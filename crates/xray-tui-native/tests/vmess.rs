//! E2E: native VMess+TLS+TCP (AES-128-GCM) through the unified pipeline.
#![cfg(feature = "native-e2e")]

use xray_tui_native::e2e::{CoreKind, CoreUnderTest, run};
use xray_tui_native::protocol::vmess::e2e::VmessCase;

const SINGBOX_VERSION: &str = "1.13.16";
const XRAY_VERSION: &str = "26.3.27";

#[tokio::test]
async fn vmess_tcp_tls_against_xray() {
    let core = CoreUnderTest::resolve(CoreKind::Xray, XRAY_VERSION).unwrap();
    run(&VmessCase, &core).await.expect("vmess/xray e2e failed");
}

#[tokio::test]
async fn vmess_tcp_tls_against_singbox() {
    let core = CoreUnderTest::resolve(CoreKind::SingBox, SINGBOX_VERSION).unwrap();
    run(&VmessCase, &core)
        .await
        .expect("vmess/singbox e2e failed");
}
