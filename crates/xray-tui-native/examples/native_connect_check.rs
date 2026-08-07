//! Manual e2e check without the cargo test harness.
//!
//! Spawns a real xray-core / sing-box server (VLESS + TLS + TCP inbound),
//! connects with the native client, and verifies an HTTP 200 + body through
//! the tunnel. Run:
//!
//! ```text
//! XRAY_TUI_CORE_BIN_DIR=/path/to/cores \
//!   cargo run -p xray-tui-native --example native_connect_check --features native-e2e -- xray
//! XRAY_TUI_CORE_BIN_DIR=/path/to/cores \
//!   cargo run -p xray-tui-native --example native_connect_check --features native-e2e -- sing-box
//! ```
//!
//! This is the authoritative local proof of wire interop; the `tests/e2e`
//! variants of the same flow run under `cargo test` but depend on the host
//! test harness (broken-waker stalls were observed in one sandbox).
#![cfg_attr(
    not(feature = "native-e2e"),
    allow(unused_imports, unused_variables, dead_code)
)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use xray_tui_native::{connect, security};

fn core_bin(kind: &str) -> Option<PathBuf> {
    let dir = std::env::var("XRAY_TUI_CORE_BIN_DIR").ok()?;
    let bin = std::path::Path::new(&dir).join(kind);
    bin.is_file().then_some(bin)
}

#[tokio::main]
async fn main() {
    #[cfg(not(feature = "native-e2e"))]
    {
        eprintln!("run with --features native-e2e");
        return;
    }
    #[cfg(feature = "native-e2e")]
    {
        run().await
    }
}

#[cfg(feature = "native-e2e")]
async fn run() {
    let kind = std::env::args().nth(1).unwrap_or_else(|| "sing-box".into());
    let bin = core_bin(&kind).expect("core binary");
    println!("REPRO: bin {bin:?}");

    // echo server
    let echo = std::sync::Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
    let echo_port = echo.server_addr().to_ip().unwrap().port();
    let echo_th = std::thread::spawn({
        let echo = Arc::clone(&echo);
        move || {
            for req in echo.incoming_requests() {
                let _ = req.respond(
                    tiny_http::Response::from_string("hello native core").with_status_code(200),
                );
            }
        }
    });
    println!("REPRO: echo on {echo_port}");

    // port + certs (rcgen)
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let (cert_pem, key_pem, ca_der) = {
        use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};
        let mut ca_p = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
        ca_p.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca_k = KeyPair::generate().unwrap();
        let ca_c = ca_p.self_signed(&ca_k).unwrap();
        let s_p =
            CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]).unwrap();
        let s_k = KeyPair::generate().unwrap();
        let s_c = s_p.signed_by(&s_k, &ca_c, &ca_k).unwrap();
        (s_c.pem(), s_k.serialize_pem(), ca_c.der().to_vec())
    };
    println!("REPRO: certs done");

    // write config + certs to tempdir
    let tmp = tempfile::tempdir().unwrap();
    let cert_path = tmp.path().join("server.crt");
    let key_path = tmp.path().join("server.key");
    std::fs::write(&cert_path, &cert_pem).unwrap();
    std::fs::write(&key_path, &key_pem).unwrap();

    let config_json = if kind == "xray" {
        serde_json::json!({
            "inbounds": [{
                "listen": "127.0.0.1", "port": port, "protocol": "vless",
                "settings": { "clients": [{ "id": "00000000-0000-0000-0000-000000000000" }], "decryption": "none" },
                "streamSettings": { "network": "tcp", "security": "tls",
                    "tlsSettings": { "certificates": [{ "certificateFile": cert_path.to_str().unwrap(), "keyFile": key_path.to_str().unwrap() }], "alpn": ["http/1.1"] } }
            }],
            "outbounds": [{ "protocol": "freedom" }]
        })
        .to_string()
    } else {
        serde_json::json!({
            "log": { "level": "warn" },
            "inbounds": [{ "type": "vless", "listen": "127.0.0.1", "listen_port": port,
                "users": [{ "uuid": "00000000-0000-0000-0000-000000000000" }],
                "tls": { "enabled": true, "certificate_path": cert_path.to_str().unwrap(), "key_path": key_path.to_str().unwrap(), "alpn": ["http/1.1"] } }],
            "outbounds": [{ "type": "direct" }]
        })
        .to_string()
    };
    let conf = tmp.path().join("config.json");
    std::fs::write(&conf, &config_json).unwrap();

    let mut cmd = std::process::Command::new(&bin);
    if kind == "xray" {
        cmd.args(["-c", conf.to_str().unwrap()]);
    } else {
        cmd.args(["run", "-c", conf.to_str().unwrap()]);
    }
    let mut child = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    println!("REPRO: core pid {} port {port}", child.id());

    let mut up = false;
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            up = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(up, "core never opened port");
    println!("REPRO: core ready");

    let _ = rustls::crypto::ring::default_provider().install_default();
    security::tls::set_test_config(security::tls::test_client_config(&ca_der));
    println!("REPRO: tls cfg set");

    let protocol: xray_tui_proto::proto_spec::ProtocolConfig =
        serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
            "transport": { "type": "tcp" }
        }))
        .unwrap();
    let params = xray_tui_native::NativeConnectParams::new(
        protocol,
        xray_tui_proto::proto_spec::endpoint::EndpointEssentials::new("127.0.0.1", port),
        xray_tui_native::addr::TargetAddr::new(
            xray_tui_native::addr::Host::Ip("127.0.0.1".parse().unwrap()),
            echo_port,
        ),
    );
    println!("REPRO: params built, connecting");
    let mut tunnel = connect(params).await.expect("native connect");
    println!("REPRO: connected, writing GET");
    tunnel
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .expect("write GET");
    println!("REPRO: GET written, reading");
    let mut buf = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(10), tunnel.read_to_end(&mut buf))
        .await
        .expect("read timeout")
        .unwrap();
    println!("REPRO: read {read} bytes");
    let text = String::from_utf8_lossy(&buf);
    assert!(text.contains("200 OK"), "no 200 in {text:?}");
    assert!(text.contains("hello native core"), "no body in {text:?}");
    println!("REPRO: DONE");

    // Close the core; the process exits and takes the echo thread with it.
    let _ = child.kill();
    let _ = child.wait();
}
