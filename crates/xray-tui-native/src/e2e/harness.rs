//! E2E harness: local HTTP echo, real core server spawn, certs, probe.
//!
//! Gating: feature `native-e2e` AND env `XRAY_TUI_CORE_BIN_DIR` pointing at
//! a directory containing `xray` and/or `sing-box` binaries. Absent → callers
//! skip (eprintln + early return). Never downloads anything.

use std::io::Read as _;
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::core::CoreKind;
use xray_tui_tls::http2::PREFACE;

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
    listener
        .set_nonblocking(true)
        .expect("tls echo nonblocking");
    let listener = tokio::net::TcpListener::from_std(listener).expect("tls echo tokio");
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let recorded_loop = recorded.clone();
    let handle = tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                break;
            };
            let cfg = server_cfg.clone();
            let recorded = recorded_loop.clone();
            tokio::spawn(async move {
                use std::io::{Read as _, Write as _};
                // The REALITY inbound borrows this server's ServerHello flight
                // over the same socket; rustls's connection API is
                // synchronous, so each session runs on a blocking thread.
                let Ok(mut std_sock) = sock.into_std() else {
                    return;
                };
                // The listener is nonblocking (tokio); this session runs on a
                // blocking thread with read/write timeouts, so switch the
                // socket to blocking mode — otherwise reads return WouldBlock
                // instantly and the timeout never applies.
                if std_sock.set_nonblocking(false).is_err() {
                    return;
                }
                tokio::task::spawn_blocking(move || {
                    let timeout = Duration::from_secs(5);
                    let _ = std_sock.set_read_timeout(Some(timeout));
                    let _ = std_sock.set_write_timeout(Some(timeout));
                    let Ok(mut conn) = rustls::ServerConnection::new(cfg) else {
                        return;
                    };
                    while conn.is_handshaking() && conn.complete_io(&mut std_sock).is_ok() {}
                    if conn.is_handshaking() {
                        // The REALITY server forwards the client's records to
                        // this connection (MirrorConn), but they are encrypted
                        // under the REALITY session keys — which the dest can
                        // never derive — so rustls errors on them and the
                        // handshake cannot complete. The connection MUST stay
                        // open anyway: xray's post-handshake detector reads the
                        // dest, and an early close makes it abort the client's
                        // REALITY session (conn.Close() → client BrokenPipe).
                        // Hold the raw socket, draining, until xray closes it
                        // right after the auth'd handshake completes.
                        hold_until_close(&mut std_sock);
                        return;
                    }
                    // Read the first application-data chunk (the spider's
                    // request / the probe's GET), then answer and close.
                    let mut buf = [0u8; 4096];
                    let mut got = 0;
                    loop {
                        if conn.read_tls(&mut std_sock).unwrap_or(0) == 0 {
                            break;
                        }
                        let _ = conn.process_new_packets();
                        while got < buf.len() {
                            match conn.reader().read(&mut buf[got..]) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => got += n,
                            }
                            if got > 0 {
                                break;
                            }
                        }
                        if got > 0 {
                            break;
                        }
                    }
                    // Record the client's first app-data bytes (bounded) so
                    // the fallback assertion can detect the spider's h2
                    // preface on the dest.
                    {
                        let mut recorded = recorded.lock().unwrap();
                        if let Some(room) = RECORD_LIMIT.checked_sub(recorded.len()) {
                            let take = got.min(room);
                            recorded.extend_from_slice(&buf[..take]);
                        }
                    }
                    let _ = conn.writer().write_all(TLS_RESPONSE);
                    let _ = conn.write_tls(&mut std_sock);
                    conn.send_close_notify();
                    let _ = conn.write_tls(&mut std_sock);
                    // Drain anything the client races in before the close
                    // lands, so the spider's follow-up GETs never wedge the
                    // socket (bounded by the 5s read timeout above).
                    let _ = conn.read_tls(&mut std_sock);
                    let _ = conn.process_new_packets();
                });
            });
        }
    });
    TlsEchoServer {
        addr,
        handle,
        recorded,
    }
}

/// How much post-handshake app data the TLS echo records per run (the h2
/// preface is 24 bytes; 64 covers it plus the first frames).
const RECORD_LIMIT: usize = 64;

/// Drain a socket until the peer closes or errors, keeping it open.
///
/// Used for the REALITY dest-borrow connection, whose rustls handshake
/// cannot complete (see `spawn_tls_echo`): the connection must survive until
/// xray's post-handshake detector finishes (~5s) and xray closes it.
fn hold_until_close(std_sock: &mut std::net::TcpStream) {
    // The borrow conn goes quiet right after the forwarded client records;
    // outlive the detector's ~5s window so xray closes it, not us.
    let _ = std_sock.set_read_timeout(Some(Duration::from_secs(15)));
    let mut buf = [0u8; 4096];
    loop {
        match std_sock.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

pub struct TlsEchoServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
    /// Post-handshake application data received from clients (bounded to
    /// [`RECORD_LIMIT`] bytes, across all sessions).
    recorded: Arc<Mutex<Vec<u8>>>,
}

impl TlsEchoServer {
    /// True if any connection received the HTTP/2 client preface
    /// (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) as its first app data.
    #[must_use]
    pub fn saw_h2_preface(&self) -> bool {
        let b = self.recorded.lock().unwrap();
        b.windows(PREFACE.len()).any(|w| w == PREFACE)
    }

    /// Clear the recorded post-handshake bytes. The echo outlives the e2e
    /// attempt loop (it is created once before the retries), so each attempt
    /// must reset before asserting fresh spider bytes — otherwise bytes from
    /// attempt N could satisfy attempt N+1's fallback poll.
    pub fn reset_recording(&self) {
        self.recorded.lock().unwrap().clear();
    }
}

impl Drop for TlsEchoServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// SNI/cert CN of the inner-TLS echo target (spec §7.4): the app-side
/// engine TLS client dials this hostname THROUGH the tunnel, so the vision
/// server's TLS filter sees a real TLS `ClientHello` and splices the
/// connection to the target.
const INNER_TLS_SNI: &str = "echo.vision.test";

/// The inner-TLS echo target for vision rows (spec §7.4).
///
/// A tokio TLS server with a self-signed cert for [`INNER_TLS_SNI`],
/// completing a real TLS 1.3 session through the tunnel and answering the
/// HTTP probe. Distinct from [`TlsEchoServer`] (the REALITY dest-borrow
/// target): the inner echo must complete a NORMAL TLS handshake — the
/// spliced tunnel forwards the app's engine-client records raw, so this
/// server never sees the REALITY-mirrored encrypted records that the dest
/// borrow does.
pub struct InnerTlsEchoServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for InnerTlsEchoServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawn the inner-TLS echo target on 127.0.0.1:ephemeral.
#[must_use]
pub fn spawn_inner_tls_echo() -> InnerTlsEchoServer {
    // Workspace convention: ring is the single crypto provider (the app
    // installs it at startup; tests install here, idempotent).
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certified = rcgen::generate_simple_self_signed(vec![INNER_TLS_SNI.to_string()])
        .expect("inner tls echo cert");
    let cert_der = rustls::pki_types::CertificateDer::from(certified.cert.der().to_vec());
    let key_der = rustls::pki_types::PrivateKeyDer::try_from(certified.signing_key.serialize_der())
        .expect("inner tls echo key der");
    let server_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("inner tls echo server config");
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_cfg));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind inner tls echo");
    let addr = listener.local_addr().expect("inner tls echo ip addr");
    listener
        .set_nonblocking(true)
        .expect("inner tls echo nonblocking");
    let listener = tokio::net::TcpListener::from_std(listener).expect("inner tls echo tokio");
    let handle = tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                break;
            };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                // The tunnel (vision splice + REALITY's post-handshake
                // detector) can delay the inner ClientHello several seconds;
                // bound the whole exchange generously.
                let Ok(Ok(mut tls)) =
                    tokio::time::timeout(Duration::from_secs(15), acceptor.accept(sock)).await
                else {
                    return;
                };
                // Read the request headers (bounded), then answer and close
                // with close_notify so the client's read_to_end terminates.
                let mut buf = [0u8; 4096];
                let mut got = 0;
                let _ = tokio::time::timeout(Duration::from_secs(15), async {
                    loop {
                        match tls.read(&mut buf[got..]).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                got += n;
                                if got >= buf.len()
                                    || buf[..got].windows(4).any(|w| w == b"\r\n\r\n")
                                {
                                    break;
                                }
                            }
                        }
                    }
                })
                .await;
                let _ = tls.write_all(TLS_RESPONSE).await;
                let _ = tls.flush().await;
                let _ = tls.shutdown().await;
                // Hold the TCP open briefly after close_notify: the server's
                // vision downlink writes the response Direct frame in the
                // same window as the echo's FIN. An immediate close lets the
                // server's teardown (triggered by the echo EOF) truncate the
                // in-flight response write; holding gives it time to flush.
                tokio::time::sleep(Duration::from_secs(3)).await;
            });
        }
    });
    InnerTlsEchoServer { addr, handle }
}

/// A UDP echo target: every received datagram is echoed verbatim back to
/// its source — the destination for the VLESS UDP e2e rows (the probe's
/// datagrams traverse the tunnel and return through it).
pub struct UdpEchoServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for UdpEchoServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawn a `tokio::net::UdpSocket` echo on 127.0.0.1:ephemeral.
///
/// `spawn_*` is synchronous (called from fixtures / `run_against`, outside
/// an async context), so the socket is bound as a std socket and converted
/// to tokio, mirroring the TLS echo servers.
#[must_use]
pub fn spawn_udp_echo() -> UdpEchoServer {
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").expect("bind udp echo");
    let addr = socket.local_addr().expect("udp echo ip addr");
    socket.set_nonblocking(true).expect("udp echo nonblocking");
    let socket = tokio::net::UdpSocket::from_std(socket).expect("udp echo tokio");
    let handle = tokio::spawn(async move {
        // Max UDP payload is 65507 (IPv4); the VLESS frame cap is 65535
        // (heap buffer — a 64 KiB stack array trips clippy).
        let mut buf = vec![0u8; 65_535];
        while let Ok((n, peer)) = socket.recv_from(&mut buf).await {
            if socket.send_to(&buf[..n], peer).await.is_err() {
                break;
            }
        }
    });
    UdpEchoServer { addr, handle }
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

const TLS_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\nhello native core";

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
    let issuer = rcgen::Issuer::new(ca_params, &ca_key);
    let server_cert = server_params.signed_by(&server_key, &issuer).unwrap();

    Certs {
        cert_pem: server_cert.pem(),
        key_pem: server_key.serialize_pem(),
        ca_der: ca_cert.der().to_vec(),
    }
}

/// Drive one HTTP GET/response exchange over `stream`, return (status, body).
///
/// Fully bounded: the write fails fast on a broken stream instead of
/// blocking. Returns `(0, String::new())` on a timeout or short read — the
/// e2e tests retry the whole connection on such outcomes.
async fn http_exchange<S>(stream: &mut S) -> (u16, String)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    // REALITY inbounds delay the first app-data exchange ~5s while the
    // server's post-handshake record detector completes — the window must
    // cover that plus a slow echo.
    const STEP: Duration = Duration::from_secs(15);
    let write = tokio::time::timeout(STEP, stream.write_all(GET)).await;
    if write.is_err() {
        return (0, String::new());
    }
    if let Err(e) = write.unwrap() {
        eprintln!("probe write error: {e}");
        return (0, String::new());
    }
    let mut buf = Vec::new();
    let read = tokio::time::timeout(STEP, stream.read_to_end(&mut buf)).await;
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

/// Write a GET through the tunnel, return (status code, body).
///
/// Fully bounded: the write fails fast on a broken tunnel instead of
/// blocking. Returns `(0, String::new())` on a timeout or short read — the
/// e2e tests retry the whole connection on such outcomes.
pub async fn probe(tunnel: &mut crate::NativeTunnel) -> (u16, String) {
    http_exchange(tunnel).await
}

/// Inner-TLS probe (spec §7.4): establish a real TLS 1.3 session THROUGH
/// the tunnel to the rustls echo target, then drive the HTTP GET over the
/// inner stream.
///
/// The app side runs the engine TLS client (`xray_tui_tls::client::connect`
/// with a Chrome profile + insecure verifier; the vision server's TLS
/// filter detects the inner `ClientHello` and splices the connection to
/// the target). Consumes the tunnel: the engine client takes ownership of
/// the stream. Returns `(0, String::new())` when the inner session cannot
/// be established (the e2e runner retries the whole connection).
pub async fn probe_inner_tls(tunnel: crate::NativeTunnel) -> (u16, String) {
    use xray_tui_tls::client::{TlsConfig, connect as tls_connect};
    use xray_tui_tls::handshake::ServerVerifier;
    use xray_tui_tls::profiles::BrowserProfile;
    use xray_tui_tls::verify::WebPkiVerifier;
    let verifier: Arc<dyn ServerVerifier> =
        Arc::new(WebPkiVerifier::webpki_roots().with_insecure(true));
    let config = TlsConfig::plain(Some(BrowserProfile::Chrome130), verifier, INNER_TLS_SNI);
    // Bound the handshake: through the tunnel + REALITY's post-handshake
    // detector the ServerHello flight can lag several seconds.
    let mut inner =
        match tokio::time::timeout(Duration::from_secs(30), tls_connect(tunnel, &config)).await {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => {
                eprintln!("probe inner tls connect error: {e}");
                return (0, String::new());
            }
            Err(_) => {
                eprintln!("probe inner tls connect timed out");
                return (0, String::new());
            }
        };
    http_exchange(&mut inner).await
}

/// Distinct datagrams [`probe_udp`] sends per attempt; the e2e runner
/// requires every echo back, so the payloads must differ (echoes are
/// matched by content, order-independent).
pub const UDP_PROBE_PAYLOADS: [&[u8]; 3] = [b"udp-0", b"udp-1", b"udp-2"];

/// The number of datagrams [`probe_udp`] sends per attempt — the count
/// `run_against` asserts on the `(sent, received)` outcome.
pub const UDP_PROBE_COUNT: usize = UDP_PROBE_PAYLOADS.len();

/// Overall budget for the UDP probe (connect + all sends + all echoes).
/// REALITY inbounds delay the first app-data exchange ~5s while the
/// server's post-handshake record detector completes, so the window must
/// cover that plus the datagram round-trips.
const UDP_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

/// Probe the VLESS UDP path over a datagram tunnel.
///
/// Establishes the tunnel via [`crate::connect_udp`], sends
/// [`UDP_PROBE_COUNT`] distinct payloads to the case target, and receives
/// until every echo arrives (matched by payload, order-independent) or the
/// deadline expires. Fully bounded: the whole exchange (connect, sends,
/// receives) runs under one deadline. Returns `(sent, received)` — the
/// number of datagrams successfully written and the number of distinct
/// echoes received; the e2e runner asserts both equal
/// [`UDP_PROBE_COUNT`]. Returns `(0, 0)` on a failed connect or a deadline
/// expiry before any datagram was sent.
pub async fn probe_udp(params: &crate::NativeConnectParams) -> (usize, usize) {
    use crate::protocol::vless::PacketMode;
    let deadline = tokio::time::Instant::now() + UDP_PROBE_TIMEOUT;
    let mut conn = match tokio::time::timeout_at(deadline, crate::connect_udp(params)).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            eprintln!("probe_udp connect error: {e}");
            return (0, 0);
        }
        Err(_) => {
            eprintln!("probe_udp connect timed out");
            return (0, 0);
        }
    };
    // Per-datagram destination for packetaddr mode (Raw mode encodes the
    // target in the request header; send(None)). Domain targets only occur
    // in Raw mode (packetaddr rejects them client-side), so a domain target
    // yields None here.
    let ip_target = match &params.target.host {
        crate::addr::Host::Ip(ip) => Some(SocketAddr::new(*ip, params.target.port)),
        crate::addr::Host::Domain(_) => None,
    };
    let mut sent = 0;
    for payload in UDP_PROBE_PAYLOADS {
        let dest = if params.udp == Some(PacketMode::PacketAddr) {
            ip_target
        } else {
            None
        };
        match tokio::time::timeout_at(deadline, conn.send(dest, payload)).await {
            Ok(Ok(())) => sent += 1,
            _ => break,
        }
    }
    let mut received = 0;
    let mut seen = [false; UDP_PROBE_COUNT];
    while received < UDP_PROBE_COUNT && tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Some((_, payload)))) => {
                let payload = payload.as_slice();
                if let Some(i) = UDP_PROBE_PAYLOADS.iter().position(|p| *p == payload)
                    && !seen[i]
                {
                    seen[i] = true;
                    received += 1;
                }
            }
            // Clean EOF at a frame boundary (tunnel closed) or the deadline
            // expired — either way the remaining echoes will not arrive.
            Ok(Ok(None)) | Err(_) => break,
            Ok(Err(e)) => {
                eprintln!("probe_udp recv error: {e}");
                break;
            }
        }
    }
    (sent, received)
}

const GET: &[u8] = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
