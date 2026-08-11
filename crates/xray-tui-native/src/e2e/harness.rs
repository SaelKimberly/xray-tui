//! E2E harness: local HTTP echo, real core server spawn, certs, probe.
//!
//! Gating: feature `native-e2e` AND env `XRAY_TUI_CORE_BIN_DIR` pointing at
//! a directory containing `xray` and/or `sing-box` binaries. Absent → callers
//! skip (eprintln + early return). Never downloads anything.

use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;

use super::core::CoreKind;

pub struct EchoServer {
    pub addr: SocketAddr,
    handle: Option<std::thread::JoinHandle<()>>,
    server: Arc<tiny_http::Server>,
}

impl Drop for EchoServer {
    fn drop(&mut self) {
        // Unblock the accept loop (it blocks forever on incoming_requests)
        // so the thread can exit and join returns instead of hanging.
        self.server.unblock();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawn a `tiny_http` responder on 127.0.0.1:ephemeral serving a fixed body.
/// `tiny_http` is synchronous — dedicated thread, requests drained until drop.
#[must_use]
pub fn spawn_echo() -> EchoServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind echo"));
    let addr = server.server_addr().to_ip().expect("ip addr");
    let srv = server.clone();
    let handle = std::thread::spawn(move || {
        for request in srv.incoming_requests() {
            let response =
                tiny_http::Response::from_string("hello native core").with_status_code(200);
            let _ = request.respond(response);
        }
    });
    EchoServer {
        addr,
        handle: Some(handle),
        server,
    }
}

/// A TLS HTTP responder on 127.0.0.1:ephemeral serving the fixed body.
///
/// REALITY inbounds need this as their `dest`/`handshake` fallback: the
/// xtls/reality server borrows the dest's TLS 1.3 `ServerHello` flight for its
/// own handshake, so a plain-HTTP echo cannot satisfy it (the server's
/// fallback detection waits for `recordTypeHandshake | typeServerHello` from
/// the dest). Uses the harness certs so the borrowed handshake completes.
#[must_use]
pub fn spawn_tls_echo(certs: &Certs) -> TlsEchoServer {
    // Workspace convention: ring is the single crypto provider (the app
    // installs it at startup; tests install here, idempotent).
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_cfg = Arc::new(tls_server_config(certs));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind tls echo");
    let addr = listener.local_addr().expect("tls echo ip addr");
    listener.set_nonblocking(true).expect("tls echo nonblocking");
    let listener = tokio::net::TcpListener::from_std(listener).expect("tls echo tokio");
    let handle = tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                break;
            };
            let cfg = server_cfg.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let acceptor = tokio_rustls::TlsAcceptor::from(cfg);
                let Ok(mut tls) = acceptor.accept(sock).await else {
                    return;
                };
                let mut buf = [0u8; 4096];
                let _ = tokio::time::timeout(Duration::from_secs(5), tls.read(&mut buf)).await;
                let _ = tls.write_all(TLS_RESPONSE).await;
                let _ = tls.shutdown().await;
            });
        }
    });
    TlsEchoServer { addr, handle }
}

pub struct TlsEchoServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for TlsEchoServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Build a rustls server config from the harness cert/key PEMs (TLS 1.3).
fn tls_server_config(certs: &Certs) -> rustls::ServerConfig {
    use rustls::pki_types::pem::PemObject;
    let cert_der: Vec<rustls::pki_types::CertificateDer<'static>> =
        rustls::pki_types::CertificateDer::pem_slice_iter(certs.cert_pem.as_bytes())
            .map(|c| c.expect("cert pem parses"))
            .collect();
    let key = rustls::pki_types::PrivateKeyDer::from_pem_slice(certs.key_pem.as_bytes())
        .expect("key pem parses");
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, key)
        .expect("tls echo server config builds")
}

const TLS_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\nhello native core";

/// Return a port that was free at bind time.
#[must_use]
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub struct CoreGuard {
    child: Child,
}

impl Drop for CoreGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Spawn a core with the given on-disk config; wait until `port` accepts TCP.
#[must_use]
pub fn spawn_core(bin: &Path, kind: CoreKind, config_path: &Path, port: u16) -> CoreGuard {
    let p = config_path.to_str().expect("config path utf8");
    let mut cmd = match kind {
        CoreKind::SingBox => {
            let mut c = Command::new(bin);
            c.args(["run", "-c", p]);
            c
        }
        CoreKind::Xray => {
            let mut c = Command::new(bin);
            c.args(["-c", p]);
            c
        }
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
    CoreGuard { child }
}

/// rcgen CA + server cert/key PEMs + CA DER. SAN covers localhost + 127.0.0.1.
pub struct Certs {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_der: Vec<u8>,
}

#[must_use]
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
///
/// Fully bounded: the write fails fast on a broken tunnel instead of
/// blocking. Returns `(0, String::new())` on a timeout or short read — the
/// e2e tests retry the whole connection on such outcomes.
pub async fn probe(tunnel: &mut crate::NativeTunnel) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    // REALITY inbounds delay the first app-data exchange ~5s while the
    // server's post-handshake record detector completes — the window must
    // cover that plus a slow echo.
    const STEP: Duration = Duration::from_secs(15);
    let write = tokio::time::timeout(STEP, tunnel.write_all(GET)).await;
    if write.is_err() {
        return (0, String::new());
    }
    if let Err(e) = write.unwrap() {
        eprintln!("probe write error: {e}");
        return (0, String::new());
    }
    let mut buf = Vec::new();
    let read = tokio::time::timeout(STEP, tunnel.read_to_end(&mut buf)).await;
    let Ok(Ok(read)) = read else {
        return (0, String::new());
    };
    if read == 0 {
        return (0, String::new());
    }
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

const GET: &[u8] = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
