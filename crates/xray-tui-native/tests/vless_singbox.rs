//! E2E: native VLESS+TLS+TCP client against a real sing-box server.
#![cfg(feature = "native-e2e")]

mod common;

use common::*;
use xray_tui_native::{connect, security};

/// Attempt the full probe on a fresh connection up to `MAX_ATTEMPTS` times.
///
/// The underlying sing-box TLS path is per-connection flaky under some
/// environments (we observed occasional server-side handshake failures and
/// EOF-without-response in a sandbox); every attempt builds a brand-new
/// native connection, so a transient failure does not void the HTTP-200+body
/// contract.
const MAX_ATTEMPTS: u32 = 5;

#[tokio::test]
async fn vless_tcp_tls_against_singbox() {
    let Some(bin) = core_bin(CoreType::SingBox) else {
        eprintln!("skip: XRAY_TUI_CORE_BIN_DIR missing sing-box binary");
        return;
    };
    let echo = spawn_echo();
    let port = free_port();
    let certs = generate_certs();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).unwrap();
    std::fs::write(&key_path, &certs.key_pem).unwrap();

    let config = singbox_vless_config(port, &cert_path, &key_path);
    let _core = spawn_core(&bin, &config, port);

    let _ = rustls::crypto::ring::default_provider().install_default();
    security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));

    let mut last = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        let params = vless_params(port, echo.addr);
        let mut tunnel = match connect(params).await {
            Ok(t) => t,
            Err(e) => {
                last = format!("connect attempt {attempt}: {e}");
                continue;
            }
        };
        let (status, body) = probe(&mut tunnel).await;
        if status == 200 && body == "hello native core" {
            return;
        }
        last = format!("probe attempt {attempt}: status {status} body {body:?}");
    }
    panic!("sing-box e2e failed after {MAX_ATTEMPTS} attempts: {last}");
}
