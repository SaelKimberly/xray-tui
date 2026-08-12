//! REALITY protocol module: fingerprint-shaped `ClientHello` provisioning,
//! the ring port of the shoes REALITY client, and the 9-step wire contract.
//!
//! Wire contract (modeled on `shoes/src/reality/`, MIT; adapted to
//! xtls/reality 2025-10+ keyshare semantics):
//! 1. X25519 keypair; the `ClientHello.random` (32 B) is the protocol
//!    random — `[0..20]` is the HKDF salt, `[20..32]` the AES-GCM nonce.
//! 2. `auth_shared` = `ECDH(client_priv`, `server_pub` from `pbk`) — the
//!    REALITY auth key. Since 2025-10 the server's TLS keyshare is a fresh
//!    *ephemeral* key (the static key authenticates only), so the TLS 1.3
//!    key schedule uses `ECDH(client_priv`, `server_keyshare`) instead.
//! 3. `auth_key` = HKDF-SHA256(auth_shared, salt = random[0..20], info b"REALITY").
//! 4. `SessionId` plaintext 16 B = version(1,8,0) + pad(1) + timestamp u32
//!    BE + `short_id` (≤8 bytes, zero-padded).
//! 5. `ClientHello`: Chrome-133 fingerprint, SNI steal target, X25519
//!    keyshare, suites 0x1301-03, ALPN h2+http/1.1.
//! 6. AAD = hello with `SessionId` zeroed; nonce = random[20..32];
//!    `SessionId` = AES-256-GCM(auth_key, nonce, plaintext, aad).
//! 7. Restore `SessionId` (builder-returned range, never hardcoded offset).
//! 8. Server auth (port `reality_client_verify.rs`): cert signature field ==
//!    HMAC-SHA512(auth_key, `ed25519_pub`), Ed25519 `CertificateVerify` over
//!    the transcript. No PKI chain check.
//! 9. TLS 1.3 key schedule + `Finished` (server `Finished` MAC checked,
//!    client `Finished` sent), then app records via [`TlsStream`].

pub mod auth;
pub mod messages;
pub mod spider;
pub mod verify;

use std::ops::Range;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::SecureRandom;
use crate::crypto::X25519KeyPair;
use crate::error::{Result, TlsError};
use crate::handshake::{AuthOutcome, ServerAuth, drive};
use crate::hello::{BuildParams, build_hello};
use crate::profiles::BrowserProfile;
use crate::record::stream::TlsStream;
use crate::spec::SessionIdSpec;

/// The `ClientHello` ALPN list REALITY always offers.
const REALITY_ALPN: &[&str] = &["h2", "http/1.1"];

// ── Provisioner contract ───────────────────────────────────────────────────

/// A fingerprint-shaped `ClientHello` that the REALITY connector fills in:
/// keyshare, SNI, random and `SessionId` are injected at connect time.
pub trait HelloProvisioner: Send + Sync {
    /// Returns the bare `ClientHello` plus the `SessionId` byte range.
    /// The returned range is computed by the builder — never hardcoded.
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello>;
}

/// Inputs the REALITY connector hands to a provisioner.
pub struct HelloProvisionParams<'a> {
    /// The SNI steal target (the `server_name` extension host).
    pub server_name: &'a str,
    /// ALPN protocols to offer; `None` uses the spec's own list.
    pub alpn: Option<&'a [&'a str]>,
    /// The connector's X25519 public key, injected into `key_share`.
    pub x25519_pub: &'a [u8; 32],
    /// Random source for the GREASE values and the client random.
    pub rng: &'a dyn SecureRandom,
}

/// A provisioned `ClientHello` (handshake bytes, no record header).
pub struct ProvisionedHello {
    /// Handshake message (`type 0x01` + 3-byte length + body) with the
    /// `AuthPayload` slot zeroed.
    pub handshake_bytes: Vec<u8>,
    /// Byte range of the `SessionId` field within `handshake_bytes`.
    pub session_id_range: Range<usize>,
}

impl ProvisionedHello {
    /// The `SessionId` bytes (the REALITY auth payload slot). The server's
    /// `ServerHello` must echo them exactly (RFC 8446 §4.1.3).
    #[must_use]
    pub fn session_id(&self) -> &[u8] {
        &self.handshake_bytes[self.session_id_range.clone()]
    }
}

/// Fixed Chrome-133-shaped provisioner (first engine; ported from shoes).
pub struct FixedChrome133;

/// A `HelloProvisioner` shaped by any browser profile: the profile's
/// `ClientHello` spec with the REALITY `AuthPayload` session id slot.
pub struct ProfileProvisioner(pub BrowserProfile);

impl HelloProvisioner for ProfileProvisioner {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        let mut spec = self.0.spec();
        spec.session_id = SessionIdSpec::AuthPayload { len: 32 };
        let built = build_hello(
            &spec,
            &BuildParams {
                server_name: params.server_name,
                alpn: params.alpn,
                x25519_pub: params.x25519_pub,
                rng: params.rng,
            },
        )?;
        let session_id_range = built.session_id_range.ok_or_else(|| {
            TlsError::Spec("profile spec must use SessionIdSpec::AuthPayload".into())
        })?;
        Ok(ProvisionedHello {
            handshake_bytes: built.handshake_bytes,
            session_id_range,
        })
    }
}

impl HelloProvisioner for FixedChrome133 {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        ProfileProvisioner(BrowserProfile::Chrome133).provision(params)
    }
}

// ── Spider session ─────────────────────────────────────────────────────────

/// Bounded spider session configuration for the REALITY fallback path.
///
/// When the server fails REALITY auth (a real certificate — transparent
/// proxy / possible MITM or redirection), the engine surfaces
/// [`TlsError::RealityFallback`] and the caller may re-connect as a plain
/// browser to the steal target: the "spider" walks `paths` with
/// `request_interval` between requests, `max_gets` requests per path
/// (bounded, mirroring xray's anti-probing). The values are defaults only;
/// the spider itself lands with the fallback re-wire.
#[derive(Clone)]
pub struct SpiderConfig {
    /// Paths from the URL `spx`; default `["/"]`.
    pub paths: Vec<String>,
    /// Bounded spider session (GETs before close); default 4.
    pub max_gets: usize,
    /// Delay between GETs; default 1s.
    pub request_interval: std::time::Duration,
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            paths: vec!["/".to_string()],
            max_gets: 4,
            request_interval: std::time::Duration::from_secs(1),
        }
    }
}

// ── Connector ──────────────────────────────────────────────────────────────

/// Connection-level inputs for [`connect_reality`].
pub struct RealityParams<'a> {
    /// The SNI steal target.
    pub server_name: &'a str,
    /// The server's static X25519 public key (decoded `pbk`).
    pub public_key: &'a [u8; 32],
    /// REALITY short id, ≤8 bytes.
    pub short_id: &'a [u8],
    /// The `ClientHello` fingerprint provisioner.
    pub provisioner: &'a dyn HelloProvisioner,
    /// Random source — the crate's [`SecureRandom`] seam. ring's
    /// `SystemRandom` coerces via the blanket impl; tests implement the
    /// trait directly for fixed-seed vectors.
    pub rng: &'a dyn SecureRandom,
    /// Spider session for the fallback path (defaults are fine until the
    /// fallback re-wire lands).
    pub spider: &'a SpiderConfig,
}

/// Performs the full REALITY client handshake over `stream`, returning a
/// record-framed application-data stream.
///
/// Implements the 9-step wire contract above: the REALITY session-id seal
/// (steps 1–5) runs here, then the shared [`drive`] takes over with a
/// [`ServerAuth::Reality`] dispatch — the HMAC/Ed25519 server auth in place
/// of the plain-TLS verifier seam. A server flight that fails REALITY auth
/// (a real certificate) is reported as [`TlsError::RealityFallback`], not a
/// hard error: the caller may fall back to the spider session.
pub async fn connect_reality<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
    stream: S,
    params: RealityParams<'_>,
) -> Result<TlsStream<S>> {
    // 1. Ephemeral X25519 key pair (the ClientHello keyshare). The
    //    connector draws no randomness of its own: the client random inside
    //    the provisioned hello *is* the REALITY random.
    let keypair = X25519KeyPair::generate(params.rng)?;

    // 2. Provision the Chrome-133-shaped ClientHello with a zeroed
    //    AuthPayload slot; the connector's keyshare and rng flow through.
    let mut hello = params.provisioner.provision(&HelloProvisionParams {
        server_name: params.server_name,
        alpn: Some(REALITY_ALPN),
        x25519_pub: &keypair.public_key(),
        rng: params.rng,
    })?;
    let client_random = messages::extract_client_random(&hello.handshake_bytes)?;

    // 3. SessionId plaintext: version(1,8,0) || pad(0) || timestamp ||
    //    short_id (≤8 bytes, zero-padded).
    let timestamp = messages::now_timestamp()?;
    let plaintext = messages::build_session_id_plaintext(timestamp, params.short_id)?;

    // 4. ECDH with the server's static public key → the REALITY auth key.
    //    (The TLS 1.3 key schedule below uses the server's *ephemeral*
    //    keyshare instead — xtls/reality 2025-10+ no longer reuses the
    //    static key as the TLS keyshare.)
    let auth_shared = keypair.agree(params.public_key)?;
    let auth_key = auth::derive_auth_key(&auth_shared, &client_random[..20], b"REALITY")?;

    // 5. Seal the SessionId (AAD = the hello with the slot zeroed, i.e. the
    //    provisioned bytes) and splice it into the builder-returned range.
    messages::seal_and_splice(&mut hello, &plaintext, &auth_key, &client_random[20..])?;

    // 6. Drive the handshake: the shared `drive()` writes the ClientHello
    //    record, reads the ServerHello (which must echo the sealed session
    //    id — the REALITY auth payload), derives the TLS 1.3 key schedule
    //    over the wire ClientHello, and dispatches the REALITY server auth
    //    (certificate HMAC + Ed25519 CertificateVerify over the transcript,
    //    no PKI chain). Any auth failure means the server is not REALITY —
    //    a real certificate (transparent proxy / possible MITM or
    //    redirection) — which surfaces as [`AuthOutcome::RealityFallback`]
    //    rather than a hard error, matching xray's `!Verified` handling.
    let (tls, outcome) = drive(
        stream,
        &hello.handshake_bytes,
        hello.session_id(),
        keypair,
        params.server_name,
        ServerAuth::Reality {
            auth_key: &auth_key,
        },
    )
    .await?;
    match outcome {
        AuthOutcome::Ok => Ok(tls),
        AuthOutcome::RealityFallback => {
            // The TLS session to the real site is established and usable —
            // hand it to the Spider-X task so the connection looks like a
            // browsing session, then report the fallback to the caller.
            let spider = params.spider.clone();
            let sni = params.server_name.to_string();
            tokio::spawn(spider::run(tls, spider, sni));
            Err(TlsError::RealityFallback)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::io::{Read, Write};
    use std::sync::Arc;

    use super::*;
    use crate::hello::parse::parse_hello;
    use crate::http2;
    use parking_lot::Mutex;

    /// The Chrome-133 provisioner lays out a valid `ClientHello` with a
    /// 32-byte zeroed `AuthPayload` slot at the reported range.
    #[test]
    fn fixed_chrome133_provisioner_builds_auth_payload_hello() {
        let rng = ring::rand::SystemRandom::new();
        let hello = FixedChrome133
            .provision(&HelloProvisionParams {
                server_name: "www.microsoft.com",
                alpn: Some(REALITY_ALPN),
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            })
            .unwrap();

        assert_eq!(hello.session_id_range.len(), 32);
        assert!(hello.session_id_range.end <= hello.handshake_bytes.len());

        let parsed = parse_hello(&hello.handshake_bytes).unwrap();

        // The AuthPayload slot is in the legacy session id, currently zeroed.
        assert_eq!(
            &hello.handshake_bytes[hello.session_id_range.clone()],
            &[0u8; 32]
        );
        assert_eq!(parsed.session_id, vec![0u8; 32]);

        // Chrome-133 surface: TLS 1.3 suites, X25519 keyshare carrying the
        // connector's public key, SNI steal target, ALPN h2 + http/1.1.
        for suite in [0x1301u16, 0x1302, 0x1303] {
            assert!(
                parsed.cipher_suites.contains(&suite),
                "missing cipher suite {suite:#06x}"
            );
        }
        let key_share = parsed.extension(0x0033).expect("key_share extension");
        assert!(
            key_share.windows(32).any(|w| w == [0xAB; 32]),
            "key_share must carry the connector's X25519 public key"
        );
        let sni = parsed.extension(0x0000).expect("server_name extension");
        assert!(
            sni.windows(b"www.microsoft.com".len())
                .any(|w| w == b"www.microsoft.com"),
            "SNI must carry the steal target"
        );
        let alpn = parsed.extension(0x0010).expect("ALPN extension");
        assert!(alpn.windows(2).any(|w| w == b"h2"), "ALPN must offer h2");
        assert!(
            alpn.windows(8).any(|w| w == b"http/1.1"),
            "ALPN must offer http/1.1"
        );
    }

    /// Deterministic RNG feeding back a fixed byte sequence (same shape as
    /// `hello::tests::FixedRandom`), so two provision calls draw identical
    /// bytes and the golden-equality assertion is meaningful.
    struct FixedRandom {
        bytes: Vec<u8>,
        pos: AtomicUsize,
    }

    impl SecureRandom for FixedRandom {
        fn fill(&self, dest: &mut [u8]) -> std::result::Result<(), ring::error::Unspecified> {
            let mut pos = self.pos.load(Ordering::Relaxed);
            for b in dest.iter_mut() {
                *b = *self.bytes.get(pos).ok_or(ring::error::Unspecified)?;
                pos += 1;
            }
            self.pos.store(pos, Ordering::Relaxed);
            Ok(())
        }
    }

    #[test]
    fn profile_provisioner_chrome133_matches_fixed_chrome133() {
        // A fresh fixed-seed RNG per call: both provisioners must consume
        // the identical byte stream (0x42 → every GREASE value 0x2A2A), so
        // byte equality proves `ProfileProvisioner(BrowserProfile::Chrome133)`
        // produces exactly what the fixed Chrome-133 provisioner produces.
        let fixed_rng = FixedRandom {
            bytes: vec![0x42; 128],
            pos: AtomicUsize::new(0),
        };
        let fixed = FixedChrome133
            .provision(&HelloProvisionParams {
                server_name: "www.microsoft.com",
                alpn: Some(REALITY_ALPN),
                x25519_pub: &[0xAB; 32],
                rng: &fixed_rng,
            })
            .unwrap();
        let profile_rng = FixedRandom {
            bytes: vec![0x42; 128],
            pos: AtomicUsize::new(0),
        };
        let profile = ProfileProvisioner(BrowserProfile::Chrome133)
            .provision(&HelloProvisionParams {
                server_name: "www.microsoft.com",
                alpn: Some(REALITY_ALPN),
                x25519_pub: &[0xAB; 32],
                rng: &profile_rng,
            })
            .unwrap();
        assert_eq!(fixed.handshake_bytes, profile.handshake_bytes);
        assert_eq!(fixed.session_id_range, profile.session_id_range);
    }

    /// A plain rustls server (no REALITY on the wire): echoes the client's
    /// legacy session id (RFC 8446 §4.1.3) and serves a real self-signed
    /// certificate. Workspace feature unification compiles rustls with BOTH
    /// backends, so install the ring provider explicitly (idempotent).
    fn plain_tls_server_config(
        cert: &rcgen::Certificate,
        key: &rcgen::KeyPair,
    ) -> rustls::ServerConfig {
        use rustls::pki_types::{CertificateDer, PrivateKeyDer};
        let _ = rustls::crypto::ring::default_provider().install_default();
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(cert.der().to_vec())],
                PrivateKeyDer::try_from(key.serialize_der()).unwrap(),
            )
            .unwrap()
    }

    /// Spawns the plain rustls server on a blocking thread, recording every
    /// decrypted post-handshake byte (the spider's HTTP/2 plaintext) into
    /// `recorded` — bounded to the first 512 bytes, then keeps draining
    /// reads so the spider's writes never wedge on TCP backpressure.
    ///
    /// After the handshake it sends its own empty SETTINGS (RFC 7540 §3.5),
    /// ACKs the client's SETTINGS, and answers every `END_STREAM` HEADERS
    /// with an empty HEADERS + `END_STREAM` on the same stream (a 204-style
    /// reply) so the spider's GETs complete promptly. The handshake itself
    /// completes because `drive()` sends the client `Finished` even on the
    /// fallback path.
    fn spawn_recording_plain_tls_server(recorded: Arc<Mutex<Vec<u8>>>) -> std::net::SocketAddr {
        let certified =
            rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
                .unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg = plain_tls_server_config(&certified.cert, &certified.signing_key);

        tokio::task::spawn_blocking(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let timeout = std::time::Duration::from_secs(15);
            sock.set_read_timeout(Some(timeout)).unwrap();
            sock.set_write_timeout(Some(timeout)).unwrap();
            let mut conn = rustls::ServerConnection::new(Arc::new(cfg)).unwrap();
            while conn.is_handshaking() {
                if conn.complete_io(&mut sock).is_err() {
                    return;
                }
            }
            // Server connection preface: our own empty SETTINGS.
            conn.writer()
                .write_all(&http2::make_frame(http2::FRAME_SETTINGS, 0, 0, &[]))
                .unwrap();
            conn.write_tls(&mut sock).unwrap();

            // Decrypted HTTP/2 plaintext: 24-byte preface, then 9-byte
            // frame header + payload; `off` is the parse cursor.
            let mut plain = Vec::new();
            let mut off = 0usize;
            loop {
                if conn.read_tls(&mut sock).unwrap() == 0 {
                    break; // the spider task finished and closed the connection
                }
                let state = conn.process_new_packets().unwrap();
                if state.plaintext_bytes_to_read() > 0 {
                    let mut buf = vec![0u8; state.plaintext_bytes_to_read()];
                    // The buffer is exactly the available plaintext, so a
                    // blocking read_exact cannot stall.
                    conn.reader().read_exact(&mut buf).unwrap();
                    plain.extend_from_slice(&buf);
                }

                // Record everything new since the last parse pass, bounded.
                {
                    let mut rec = recorded.lock();
                    let take = (plain.len() - off).min(512usize.saturating_sub(rec.len()));
                    rec.extend_from_slice(&plain[off..off + take]);
                }

                // Skip the client preface (may span several reads).
                if off < http2::PREFACE.len() {
                    if plain.len() < http2::PREFACE.len() {
                        continue;
                    }
                    off = http2::PREFACE.len();
                }

                // Parse complete frames and reply per RFC 7540.
                loop {
                    if plain.len() - off < 9 {
                        break; // incomplete frame header
                    }
                    let len = (usize::from(plain[off]) << 16)
                        | (usize::from(plain[off + 1]) << 8)
                        | usize::from(plain[off + 2]);
                    let frame_len = 9usize + len;
                    if plain.len() - off < frame_len {
                        break; // incomplete frame payload
                    }
                    let ftype = plain[off + 3];
                    let flags = plain[off + 4];
                    let stream_id = u32::from_be_bytes([
                        plain[off + 5] & 0x7F,
                        plain[off + 6],
                        plain[off + 7],
                        plain[off + 8],
                    ]);
                    off += frame_len;
                    match ftype {
                        http2::FRAME_SETTINGS if flags & http2::FLAG_ACK == 0 => {
                            conn.writer()
                                .write_all(&http2::make_frame(
                                    http2::FRAME_SETTINGS,
                                    http2::FLAG_ACK,
                                    0,
                                    &[],
                                ))
                                .unwrap();
                            conn.write_tls(&mut sock).unwrap();
                        }
                        http2::FRAME_HEADERS if flags & http2::FLAG_END_STREAM != 0 => {
                            conn.writer()
                                .write_all(&http2::make_frame(
                                    http2::FRAME_HEADERS,
                                    http2::FLAG_END_STREAM | http2::FLAG_END_HEADERS,
                                    stream_id,
                                    &[],
                                ))
                                .unwrap();
                            conn.write_tls(&mut sock).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        });

        addr
    }

    /// A REALITY-mode client against a plain TLS server must yield
    /// [`TlsError::RealityFallback`], not a hard error: the server echoes
    /// the sealed session id, so the handshake proceeds to the
    /// `CertificateVerify`, where the REALITY HMAC check fails on the real
    /// certificate — the fallback signal the caller needs.
    #[tokio::test]
    async fn reality_against_plain_tls_server_is_fallback() {
        let recorded: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let addr = spawn_recording_plain_tls_server(recorded.clone());

        let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();
        let rng = ring::rand::SystemRandom::new();
        let result = connect_reality(
            tcp,
            RealityParams {
                server_name: "localhost",
                public_key: &[0xAB; 32],
                short_id: &[],
                provisioner: &FixedChrome133,
                rng: &rng,
                spider: &SpiderConfig::default(),
            },
        )
        .await;
        assert!(
            matches!(result, Err(TlsError::RealityFallback)),
            "expected RealityFallback, got an unexpected handshake result"
        );

        // The spawned Spider-X session must keep the established TLS
        // session alive: poll the recorded bytes (up to 5s) for the HTTP/2
        // client preface, which the spider writes on its first GET. The
        // guard is scoped to a block so it never spans the await below.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let preface_seen = {
                let bytes = recorded.lock();
                bytes
                    .windows(http2::PREFACE.len())
                    .any(|w| w == http2::PREFACE)
            };
            if preface_seen {
                break;
            }
            assert!(
                std::time::Instant::now() <= deadline,
                "spider never sent the h2 preface to the dest"
            );
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}
