//! E2E harness: local HTTP echo, real core server spawn, certs, probe.
//!
//! Gating: feature `native-e2e` AND env `XRAY_TUI_CORE_BIN_DIR` pointing at
//! a directory containing `xray` and/or `sing-box` binaries. Absent → callers
//! skip (eprintln + early return). Never downloads anything.

use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

pub use xray_tui_proto::proto_spec::CoreType;
use xray_tui_proto::proto_spec::ProtocolConfig;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

pub struct EchoServer {
    pub addr: SocketAddr,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for EchoServer {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawn a tiny_http responder on 127.0.0.1:ephemeral serving a fixed body.
/// tiny_http is synchronous — dedicated thread, requests drained until drop.
pub fn spawn_echo() -> EchoServer {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind echo");
    let addr = server.server_addr().to_ip().expect("ip addr");
    let handle = std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let response =
                tiny_http::Response::from_string("hello native core").with_status_code(200);
            let _ = request.respond(response);
        }
    });
    EchoServer {
        addr,
        handle: Some(handle),
    }
}

/// Return a port that was free at bind time.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub struct CoreGuard {
    child: Child,
    _tmp: tempfile::TempDir,
}

impl Drop for CoreGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Resolve the core binary from `XRAY_TUI_CORE_BIN_DIR` (never auto-download).
pub fn core_bin(core: CoreType) -> Option<PathBuf> {
    let dir = std::env::var("XRAY_TUI_CORE_BIN_DIR").ok()?;
    let dir = Path::new(&dir);
    let name = match core {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
    };
    let bin = dir.join(name);
    bin.is_file().then_some(bin)
}

/// Spawn a core with the given config JSON; wait until `port` accepts TCP.
pub fn spawn_core(bin: &Path, config_json: &str, port: u16) -> CoreGuard {
    let tmp = tempfile::tempdir().expect("tempdir");
    let conf = tmp.path().join("config.json");
    std::fs::write(&conf, config_json).expect("write config");

    let mut cmd = if bin.ends_with("sing-box") {
        let mut c = Command::new(bin);
        c.args(["run", "-c", conf.to_str().unwrap()]);
        c
    } else {
        let mut c = Command::new(bin);
        c.args(["-c", conf.to_str().unwrap()]);
        c
    };
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let child = cmd.spawn().expect("spawn core");

    let mut attempts = 0;
    loop {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        attempts += 1;
        assert!(attempts < 100, "core did not open port {port} in time");
        std::thread::sleep(Duration::from_millis(100));
    }
    CoreGuard { child, _tmp: tmp }
}

/// rcgen CA + server cert/key PEMs + CA DER. SAN covers localhost + 127.0.0.1.
pub struct Certs {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_der: Vec<u8>,
}

pub fn generate_certs() -> Certs {
    use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};

    let mut ca_params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate().unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    let server_params =
        CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]).unwrap();
    let server_key = KeyPair::generate().unwrap();
    let server_cert = server_params
        .signed_by(&server_key, &ca_cert, &ca_key)
        .unwrap();

    Certs {
        cert_pem: server_cert.pem(),
        key_pem: server_key.serialize_pem(),
        ca_der: ca_cert.der().to_vec(),
    }
}

/// Write a GET through the tunnel, return (status code, body).
pub async fn probe(tunnel: &mut xray_tui_native::NativeTunnel) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    tunnel
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut buf = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(10), tunnel.read_to_end(&mut buf))
        .await
        .expect("probe timeout")
        .unwrap();
    assert!(read > 0, "empty tunnel response");
    let text = String::from_utf8_lossy(&buf);
    let status_line = text.lines().next().unwrap_or_default().to_string();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = text
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .trim_end()
        .to_string();
    (status, body)
}

/// Build the native connect params for the e2e VLESS case (sni=localhost).
///
/// The test must install the harness-CA trust first:
/// `security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der))`.
pub fn vless_params(port: u16, target: SocketAddr) -> xray_tui_native::NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vless",
        "uuid": "00000000-0000-0000-0000-000000000000",
        "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .unwrap();
    xray_tui_native::NativeConnectParams::new(
        protocol,
        EndpointEssentials::new("127.0.0.1", port),
        xray_tui_native::addr::TargetAddr::new(
            xray_tui_native::addr::Host::Ip(target.ip()),
            target.port(),
        ),
    )
}

/// xray-core inbound config: VLESS + TLS + TCP listener on 127.0.0.1:port.
/// Certificates come from PEM files on disk (both cores take file paths).
// `allow(dead_code)`: each builder is used by exactly one test target; the
// harness module is compiled into both.
#[allow(dead_code)]
pub fn xray_vless_config(port: u16, cert_path: &Path, key_path: &Path) -> String {
    serde_json::json!({
        "inbounds": [{
            "listen": "127.0.0.1",
            "port": port,
            "protocol": "vless",
            "settings": {
                "clients": [{ "id": "00000000-0000-0000-0000-000000000000" }],
                "decryption": "none"
            },
            "streamSettings": {
                "network": "tcp",
                "security": "tls",
                "tlsSettings": {
                    "certificates": [{
                        "certificateFile": cert_path.to_str().unwrap(),
                        "keyFile": key_path.to_str().unwrap()
                    }],
                    "alpn": ["http/1.1"]
                }
            }
        }],
        "outbounds": [{ "protocol": "freedom" }]
    })
    .to_string()
}

/// sing-box inbound config: VLESS + TLS + TCP listener on 127.0.0.1:port.
#[allow(dead_code)]
pub fn singbox_vless_config(port: u16, cert_path: &Path, key_path: &Path) -> String {
    serde_json::json!({
        "log": { "level": "warn" },
        "inbounds": [{
            "type": "vless",
            "listen": "127.0.0.1",
            "listen_port": port,
            "users": [{ "uuid": "00000000-0000-0000-0000-000000000000" }],
            "tls": {
                "enabled": true,
                "certificate_path": cert_path.to_str().unwrap(),
                "key_path": key_path.to_str().unwrap(),
                "alpn": ["http/1.1"]
            }
        }],
        "outbounds": [{ "type": "direct" }]
    })
    .to_string()
}
