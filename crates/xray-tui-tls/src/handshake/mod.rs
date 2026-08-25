//! TLS 1.3 client handshake (plain TLS, verifier seam).
//!
//! Port of `thirdparty/tls-fingerprint/src/handshake.rs`, adapted to the
//! crate's seams: the stream is generic (`AsyncRead + AsyncWrite + Unpin +
//! Send`), the key schedule carries its own transcript (crypto Task 5),
//! record reads go through `record::skip_ccs` (record Task 6), and the
//! established connection is a [`TlsStream`] (record Task 7). The two
//! deliberate Task-8 additions over the reference are:
//!
//! - **HRR detection** — a `ServerHello` whose random is the RFC 8446 §4.1.4
//!   magic value is reported as [`TlsError::HelloRetryRequest`] instead of
//!   being misparsed as a missing `key_share`.
//! - **Verifier seam** — the certificate chain, the `CertificateVerify`
//!   signature scheme and raw body, and the transcript up to (excluding) the
//!   `CertificateVerify` are handed to [`ServerVerifier::verify`] before the
//!   handshake continues. Task 9 implements the real
//!   `WebPKI` verifier; [`AcceptAll`] is the test-only seam.
//!
//! Transcript accumulation follows the reference exactly: every raw
//! handshake message (`ClientHello` through `Finished`) is appended in wire
//! order; the client `Finished` and the application traffic secrets are
//! derived over `ClientHello..server Finished` (RFC 8446 §4.4.4, §7.1).

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::SecureRandom;
use crate::crypto::mlkem::{Mlkem768, SecretKey as MlkemSecretKey};
use crate::crypto::{AeadKey, CipherSuiteId, KeySchedule, X25519KeyPair};
use crate::error::{Result, TlsError};
use crate::hello::{BuildParams, build_hello, to_record};
use crate::record::stream::{AppKeys, TlsStream};
use crate::record::{
    CONTENT_APPLICATION_DATA, CONTENT_HANDSHAKE, HS_CERTIFICATE, HS_CERTIFICATE_VERIFY,
    HS_ENCRYPTED_EXTENSIONS, HS_FINISHED, HS_SERVER_HELLO, aead_aad, make_app_data_record,
    parse_handshake_messages, skip_ccs,
};
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup};

// ── Verifier seam ──────────────────────────────────────────────────────────

/// Verifies the server's `CertificateVerify` signature over the handshake
/// transcript.
///
/// Task 9 implements the real `WebPKI` verifier; Task 8 hands it every raw
/// piece it needs. Per RFC 8446 §4.4.3 the `CertificateVerify` signature is
/// computed over the transcript up to but *excluding* the `CertificateVerify`
/// message itself, so the signed bytes are
/// `Hash(ClientHello .. Certificate)` — exactly the wire-order prefix
/// `transcript` carries (with the `CertificateVerify` deliberately not yet
/// appended).
pub trait ServerVerifier: Send + Sync {
    /// Returns `Ok(())` when the server identity is acceptable.
    fn verify(&self, ctx: &VerifyContext<'_>) -> Result<()>;
}

/// Everything a verifier needs to authenticate the server.
pub struct VerifyContext<'a> {
    /// DER-encoded certificate chain, leaf first.
    pub chain: &'a [Vec<u8>],
    /// The `server_name` the client offered.
    pub sni: &'a str,
    /// The `signature_scheme` from `CertificateVerify` (RFC 8446 §4.2.3).
    pub signature_scheme: u16,
    /// The raw `CertificateVerify` body (scheme + signature).
    pub cert_verify_body: &'a [u8],
    /// The transcript up to (excluding) the `CertificateVerify`.
    pub transcript: &'a [u8],
    /// The cipher suite the server selected.
    pub suite: CipherSuiteId,
}

/// Test-only verifier: accepts any server.
pub struct AcceptAll;

impl ServerVerifier for AcceptAll {
    fn verify(&self, _: &VerifyContext<'_>) -> Result<()> {
        Ok(())
    }
}

/// Connection-level inputs for [`connect`].
pub struct HandshakeParams<'a> {
    /// The `ClientHello` fingerprint spec.
    pub spec: &'a ClientHelloSpec,
    /// SNI host name, carried in the `server_name` extension and handed to
    /// the verifier.
    pub server_name: &'a str,
    /// ALPN protocols to offer; `None` uses the spec's own list.
    pub alpn: Option<&'a [&'a str]>,
    /// The server-authentication seam.
    pub verifier: &'a dyn ServerVerifier,
    /// Random source — the crate's [`SecureRandom`] seam. ring's
    /// `SystemRandom` coerces via the blanket impl; tests implement the
    /// trait directly for fixed-seed vectors.
    pub rng: &'a dyn SecureRandom,
}

// ── Constants ──────────────────────────────────────────────────────────────

/// RFC 8446 §4.1.4: the `ServerHello.random` that marks a
/// `HelloRetryRequest`.
const HRR_RANDOM: [u8; 32] = [
    0xCF, 0x21, 0xAD, 0x74, 0xE5, 0x9A, 0x61, 0x11, 0xBE, 0x1D, 0x8C, 0x02, 0x1E, 0x65, 0xB8, 0x91,
    0xC2, 0xA2, 0x11, 0x16, 0x7A, 0xBB, 0x8C, 0x5E, 0x07, 0x9E, 0x09, 0xE2, 0xC8, 0xA8, 0x33, 0x9C,
];

/// `key_share` extension type (RFC 8446 §4.2.8).
const EXT_KEY_SHARE: u16 = 0x0033;
/// `supported_versions` extension type (RFC 8446 §4.2.1).
const EXT_SUPPORTED_VERSIONS: u16 = 0x002b;
/// `application_layer_protocol_negotiation` extension type (RFC 7301).
const EXT_ALPN: u16 = 0x0010;

/// AEAD tag length for the TLS 1.3 AEADs in this engine (all 16 bytes).
const AEAD_TAG_LEN: usize = 16;

/// Maximum accumulated plaintext for the encrypted server flight, in bytes.
///
/// A single handshake message is at most `4 + 0xFF_FFFF` (type(1) +
/// length(3) + a uint24 body, RFC 8446 §4), so this cap admits every legal
/// message while bounding cross-record accumulation (RFC 8446 §5.1) against
/// unbounded growth.
const MAX_FLIGHT_BUFFER: usize = 4 + 0xFF_FFFF;

// ── Handshake driver ───────────────────────────────────────────────────────

/// The server-authentication strategy for the shared [`drive`] handshake.
///
/// [`drive`] is the common middle of `handshake::connect` and
/// `reality::connect_reality`: both write the same shape of `ClientHello`
/// record and derive the same TLS 1.3 key schedule; they differ only in
/// how the server's certificate flight is authenticated.
pub(crate) enum ServerAuth<'a> {
    /// Plain-TLS mode: hand the flight to a [`ServerVerifier`].
    Verifier(&'a dyn ServerVerifier),
    /// REALITY mode: authenticate with the REALITY auth key (HMAC +
    /// Ed25519 `CertificateVerify` over the transcript).
    Reality { auth_key: &'a [u8; 32] },
}

/// The result of the auth dispatch inside [`drive`].
pub(crate) enum AuthOutcome {
    /// The server flight authenticated under the chosen strategy; the
    /// handshake completes normally.
    Ok,
    /// REALITY mode: the flight failed REALITY auth, so the server is
    /// serving a real certificate (transparent proxy / possible MITM or
    /// redirection). The handshake still completes; the caller decides
    /// whether to fall back (xray's `!Verified` handling).
    RealityFallback,
}

/// Performs the shared TLS 1.3 client handshake over `stream`: writes the
/// `ClientHello` record, reads the `ServerHello` (skipping middlebox CCS
/// records; a `HelloRetryRequest` magic random is rejected with
/// [`TlsError::HelloRetryRequest`]), derives the handshake traffic secrets
/// with the *selected* suite, decrypts the server flight
/// (`EncryptedExtensions`, `Certificate`, `CertificateVerify`, `Finished`),
/// dispatches [`ServerAuth`], authenticates the server `Finished` MAC, sends
/// the client `Finished` (sealed with the client handshake key), derives the
/// application traffic secrets, and builds the [`TlsStream`].
///
/// The `transcript` (`ClientHello .. Certificate`) is consumed only by the
/// auth dispatch — the key schedule carries its own transcript
/// (`KeySchedule::add_transcript`) for the `Finished` MACs and app secrets.
/// `offered_session_id` must be the legacy session id of `hello`; the
/// `offered_session_id` must be the legacy session id of `hello`; the
/// server's `ServerHello` must echo it exactly (RFC 8446 §4.1.3).
pub(crate) async fn drive<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S,
    hello: &[u8],
    offered_session_id: &[u8],
    keypair: X25519KeyPair,
    // ML-KEM-768 secret key when the ClientHello offered a hybrid key
    // share; decapsulation runs only if the server actually selected the
    // hybrid group (its `key_share` then carries the ciphertext).
    mlkem_sk: Option<&MlkemSecretKey>,
    server_name: &str,
    // RFC 8879 `compress_certificate` algorithms the ClientHello offered
    // (0x0001 zlib, 0x0002 brotli, 0x0003 zstd). Empty when the hello
    // advertised none; the server then must not compress its Certificate.
    offered_compress: &[u16],
    auth: ServerAuth<'_>,
) -> Result<(TlsStream<S>, AuthOutcome)> {
    let mut stream = stream;
    stream.write_all(&to_record(hello)).await?;

    let server_hello = read_server_hello(&mut stream, offered_session_id).await?;
    let classical_shared = keypair.agree(&server_hello.peer_key)?;

    // Hybrid key exchange (Go crypto/tls X25519MLKEM768): the key-schedule
    // IKM is `mlkem_shared || classical_shared` — Go's
    // handshake_client_tls13 appends the ECDH secret AFTER the ML-KEM
    // shared secret (`sharedKey = append(mlkemShared, sharedKey...)`).
    let shared: Vec<u8> = match (mlkem_sk, server_hello.mlkem_ciphertext.as_deref()) {
        (Some(sk), Some(ct)) => {
            let pq_shared = Mlkem768::decapsulate(
                sk,
                &crate::crypto::mlkem::Ciphertext::from_bytes(ct)
                    .map_err(|e| TlsError::Crypto(e.to_string()))?,
            )
            .map_err(|e| TlsError::Crypto(e.to_string()))?;
            let mut combined = Vec::with_capacity(classical_shared.len() + 32);
            combined.extend_from_slice(pq_shared.as_bytes());
            combined.extend_from_slice(&classical_shared);
            combined
        }
        _ => classical_shared.to_vec(),
    };

    let mut ks = KeySchedule::new(server_hello.suite);
    ks.add_transcript(hello);
    ks.add_transcript(&server_hello.raw);
    let hs_secret = ks.handshake_secret(&shared)?;
    let (client_hs_ts, server_hs_ts) = ks.handshake_traffic_secrets(&hs_secret)?;
    let server_hs_key = AeadKey::new(server_hello.suite, &server_hs_ts)?;
    let client_hs_key = AeadKey::new(server_hello.suite, &client_hs_ts)?;

    let flight = read_server_hs_messages(&mut stream, &server_hs_key, offered_compress).await?;

    // Transcript up to (excluding) CertificateVerify — consumed only by the
    // auth dispatch below (RFC 8446 §4.4.3).
    let mut transcript = Vec::with_capacity(
        hello.len() + server_hello.raw.len() + flight.ee_raw.len() + flight.cert_raw.len(),
    );
    transcript.extend_from_slice(hello);
    transcript.extend_from_slice(&server_hello.raw);
    transcript.extend_from_slice(&flight.ee_raw);
    transcript.extend_from_slice(&flight.cert_raw);

    let outcome = match auth {
        ServerAuth::Verifier(verifier) => {
            verifier.verify(&VerifyContext {
                chain: &flight.chain,
                sni: server_name,
                signature_scheme: flight.signature_scheme,
                cert_verify_body: &flight.cert_verify_body,
                transcript: &transcript,
                suite: server_hello.suite,
            })?;
            AuthOutcome::Ok
        }
        ServerAuth::Reality { auth_key } => {
            let Some(cert_der) = flight.chain.first() else {
                return Err(TlsError::Verify(
                    "REALITY server sent no certificate".into(),
                ));
            };
            match crate::reality::verify::verify_server(
                cert_der,
                &flight.cv_raw,
                auth_key,
                &transcript,
                server_hello.suite.digest(),
            ) {
                Ok(()) => AuthOutcome::Ok,
                // Any REALITY auth failure = a real certificate (transparent
                // proxy / possible MITM or redirection), matching xray's
                // `!Verified` handling.
                Err(_) => AuthOutcome::RealityFallback,
            }
        }
    };

    ks.add_transcript(&flight.ee_raw);
    ks.add_transcript(&flight.cert_raw);
    ks.add_transcript(&flight.cv_raw);
    let server_finished_key = ks.finished_key(&server_hs_ts)?;
    if ks.finished_mac(&server_finished_key) != flight.sf_verify_data {
        return Err(TlsError::Handshake(
            "server Finished MAC mismatch — possible MITM or wrong key".into(),
        ));
    }

    let sf_raw = make_hs_msg(HS_FINISHED, &flight.sf_verify_data);
    ks.add_transcript(&sf_raw);
    let client_finished_key = ks.finished_key(&client_hs_ts)?;
    let client_finished_mac = ks.finished_mac(&client_finished_key);
    let cf_hs_msg = make_hs_msg(HS_FINISHED, &client_finished_mac);
    let mut cf_inner = cf_hs_msg.clone();
    cf_inner.push(CONTENT_HANDSHAKE);
    let cf_ciphertext =
        client_hs_key.seal(0, &aead_aad(cf_inner.len() + AEAD_TAG_LEN), &cf_inner)?;
    stream
        .write_all(&make_app_data_record(&cf_ciphertext))
        .await?;

    let master = ks.master_secret(&hs_secret)?;
    let (client_app_ts, server_app_ts) = ks.app_traffic_secrets(&master)?;
    let client_app_key = AeadKey::new(server_hello.suite, &client_app_ts)?;
    let server_app_key = AeadKey::new(server_hello.suite, &server_app_ts)?;
    let mut tls = TlsStream::new(
        stream,
        AppKeys {
            read_key: server_app_key,
            write_key: client_app_key,
            read_seq: 0,
            write_seq: 0,
        },
    );
    tls.set_negotiated_hybrid(server_hello.mlkem_ciphertext.is_some());
    Ok((tls, outcome))
}

/// Performs the full TLS 1.3 client handshake over `stream`, returning a
/// record-framed application-data stream.
///
/// Generates an ephemeral X25519 key pair — plus an ML-KEM-768 key pair
/// when the spec's `key_share` carries a hybrid entry — builds the
/// `ClientHello` record (fingerprint from `params.spec`), and hands both to
/// the shared [`drive`] with a [`ServerAuth::Verifier`] dispatch.
pub async fn connect<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S,
    params: HandshakeParams<'_>,
) -> Result<TlsStream<S>> {
    let keypair = X25519KeyPair::generate(params.rng)?;
    let x25519_pub = keypair.public_key();

    let offers_hybrid = params.spec.extensions.iter().any(|ext| {
        matches!(
            ext,
            ExtensionSpec::KeyShare(groups)
                if groups.iter().any(|g| matches!(
                    g,
                    KeyShareGroup::X25519Mlkem768
                        | KeyShareGroup::Secp256r1Mlkem768
                        | KeyShareGroup::Secp384r1Mlkem1024
                ))
        )
    });
    let mlkem = if offers_hybrid {
        let (pk, sk) = Mlkem768::generate_keypair().map_err(|e| TlsError::Crypto(e.to_string()))?;
        (Some(pk), Some(sk))
    } else {
        (None, None)
    };

    let hello = build_hello(
        params.spec,
        &BuildParams {
            server_name: params.server_name,
            alpn: params.alpn,
            x25519_pub: &x25519_pub,
            mlkem768_pub: mlkem
                .0
                .as_ref()
                .map(crate::crypto::mlkem::PublicKey::as_bytes),
            rng: params.rng,
        },
    )?;
    let offered_compress: Vec<u16> = params
        .spec
        .extensions
        .iter()
        .find_map(|ext| match ext {
            ExtensionSpec::CompressCertificate(algs) => Some(algs.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let (tls, _) = drive(
        stream,
        &hello.handshake_bytes,
        hello.session_id(),
        keypair,
        mlkem.1.as_ref(),
        params.server_name,
        &offered_compress,
        ServerAuth::Verifier(params.verifier),
    )
    .await?;
    Ok(tls)
}

// ── ServerHello ────────────────────────────────────────────────────────────

/// A parsed `ServerHello`.
pub(crate) struct ServerHelloData {
    /// The raw handshake message (type + length + body), for the transcript.
    pub(crate) raw: Vec<u8>,
    /// The server's classical (X25519) public key from `key_share`.
    pub(crate) peer_key: [u8; 32],
    /// The ML-KEM ciphertext from the server's hybrid key share
    /// (`Some` only when a hybrid group was selected).
    pub(crate) mlkem_ciphertext: Option<Vec<u8>>,
    /// The cipher suite the server selected.
    pub(crate) suite: CipherSuiteId,
}

/// The `ServerHello` body fields the engine needs.
#[derive(Debug)]
struct ParsedServerHello {
    suite: CipherSuiteId,
    peer_key: [u8; 32],
    mlkem_ciphertext: Option<Vec<u8>>,
}

/// Read and parse the `ServerHello` record, skipping CCS records. The
/// server must echo the offered legacy session id (RFC 8446 §4.1.3).
pub(crate) async fn read_server_hello<S>(
    stream: &mut S,
    offered_session_id: &[u8],
) -> Result<ServerHelloData>
where
    S: AsyncRead + Unpin,
{
    let rec = skip_ccs(stream).await?;
    if rec.content_type != CONTENT_HANDSHAKE {
        return Err(TlsError::Handshake(format!(
            "expected Handshake record (0x16), got 0x{:02X}",
            rec.content_type
        )));
    }
    let msgs = parse_handshake_messages(&rec.payload)?;
    let (msg_type, body) = msgs
        .into_iter()
        .next()
        .ok_or_else(|| TlsError::Handshake("empty record for ServerHello".into()))?;
    if msg_type != HS_SERVER_HELLO {
        return Err(TlsError::Handshake(format!(
            "expected ServerHello (0x02), got 0x{msg_type:02X}"
        )));
    }
    let parsed = parse_server_hello(&body, offered_session_id)?;
    Ok(ServerHelloData {
        raw: make_hs_msg(HS_SERVER_HELLO, &body),
        peer_key: parsed.peer_key,
        mlkem_ciphertext: parsed.mlkem_ciphertext,
        suite: parsed.suite,
    })
}

/// Parse a `ServerHello` body:
/// `version(2) || random(32) || sid_len(1) || sid || cipher(2) || comp(1) ||
/// ext_list_len(2) || extensions`.
///
/// `offered_session_id` is the legacy session id the client sent in its
/// `ClientHello`; the server's echo must match it exactly (RFC 8446 §4.1.3).
fn parse_server_hello(body: &[u8], offered_session_id: &[u8]) -> Result<ParsedServerHello> {
    if body.len() < 38 {
        return Err(TlsError::Handshake("ServerHello body too short".into()));
    }
    let legacy_version = u16::from_be_bytes([body[0], body[1]]);
    if legacy_version != 0x0303 {
        return Err(TlsError::Handshake(format!(
            "ServerHello legacy_version is 0x{legacy_version:04X}, expected 0x0303"
        )));
    }
    let mut random = [0u8; 32];
    random.copy_from_slice(&body[2..34]);

    let session_id_len = usize::from(body[34]);
    let mut pos = 35 + session_id_len;
    if pos + 5 > body.len() {
        return Err(TlsError::Handshake(
            "ServerHello truncated at cipher suite".into(),
        ));
    }

    // RFC 8446 §4.1.4: a HelloRetryRequest is a ServerHello whose random is
    // the magic value; it carries no key_share for the rejected group, so
    // the check must precede extension parsing. HRR is rejected outright
    // (this engine does not implement it), so the session-id echo check
    // below guards the real ServerHello path only.
    if random == HRR_RANDOM {
        return Err(TlsError::HelloRetryRequest);
    }

    // RFC 8446 §4.1.3: the server's `legacy_session_id` must echo the one
    // the client offered (a random 32-byte id, a REALITY auth payload, or
    // none).
    if &body[35..pos] != offered_session_id {
        return Err(TlsError::Handshake(
            "ServerHello session id does not echo the offered one".into(),
        ));
    }

    let suite_wire = u16::from_be_bytes([body[pos], body[pos + 1]]);
    pos += 3; // cipher suite (2) + compression method (1)
    let suite = CipherSuiteId::from_u16(suite_wire).ok_or_else(|| {
        TlsError::Handshake(format!(
            "server selected unsupported cipher suite 0x{suite_wire:04X}"
        ))
    })?;

    if pos + 2 > body.len() {
        return Err(TlsError::Handshake("ServerHello: no extensions".into()));
    }
    let ext_list_len = usize::from(u16::from_be_bytes([body[pos], body[pos + 1]]));
    pos += 2;
    let ext_end = pos + ext_list_len;
    if ext_end > body.len() {
        return Err(TlsError::Handshake(
            "ServerHello extensions truncated".into(),
        ));
    }

    let mut peer_key = None;
    let mut mlkem_ciphertext = None;
    while pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([body[pos], body[pos + 1]]);
        let ext_len = usize::from(u16::from_be_bytes([body[pos + 2], body[pos + 3]]));
        pos += 4;
        if pos + ext_len > ext_end {
            return Err(TlsError::Handshake(
                "ServerHello extension overruns list".into(),
            ));
        }
        let ext_data = &body[pos..pos + ext_len];
        pos += ext_len;
        if ext_type == EXT_KEY_SHARE {
            // ServerHello key_share: NamedGroup(2) + key_len(2) + key
            // (RFC 8446 §4.2.8). X25519MLKEM768 (0x11EC) carries 1120
            // bytes in the Go crypto/tls wire order (the reality/utls
            // client splits `data[:1088]` as the ML-KEM ciphertext and
            // `data[1088:]` as the X25519 public key):
            // `ML-KEM-768 ciphertext (1088) || X25519 pub (32)`. Any other
            // group or malformed length is a protocol error rather than a
            // silently-missing keyshare.
            if ext_data.len() < 4 {
                return Err(TlsError::Handshake(
                    "ServerHello key_share extension too short".into(),
                ));
            }
            let group = u16::from_be_bytes([ext_data[0], ext_data[1]]);
            let key_len = usize::from(u16::from_be_bytes([ext_data[2], ext_data[3]]));
            match (group, key_len, ext_data.len()) {
                (0x001D, 32, n) if n == 4 + key_len => {
                    peer_key = Some(ext_data[4..36].try_into().expect("32 bytes"));
                }
                (0x11EC, 1120, n) if n == 4 + key_len => {
                    mlkem_ciphertext = Some(ext_data[4..4 + 1088].to_vec());
                    peer_key = Some(ext_data[4 + 1088..].try_into().expect("32 bytes"));
                }
                (0x11EB | 0x11ED, ..) => {
                    return Err(TlsError::Handshake(
                        "ServerHello selected SecP256r1MLKEM768/SecP384r1MLKEM1024: no P-256/P-384 key exchange in this engine".into(),
                    ));
                }
                _ => {
                    return Err(TlsError::Handshake(format!(
                        "ServerHello key_share: group 0x{group:04X} key length {key_len}"
                    )));
                }
            }
        }
    }

    if pos != ext_end {
        return Err(TlsError::Handshake(
            "trailing garbage in ServerHello extension list".into(),
        ));
    }

    let peer_key = peer_key
        .ok_or_else(|| TlsError::Handshake("ServerHello has no key_share extension".into()))?;
    Ok(ParsedServerHello {
        suite,
        peer_key,
        mlkem_ciphertext,
    })
}

// ── Encrypted server flight ────────────────────────────────────────────────

/// The parsed server flight: raw messages for the transcript plus the values
/// the verifier seam and client `Finished` need.
pub(crate) struct ServerFlight {
    pub(crate) ee_raw: Vec<u8>,
    pub(crate) cert_raw: Vec<u8>,
    pub(crate) cv_raw: Vec<u8>,
    pub(crate) sf_verify_data: Vec<u8>,
    /// DER certificates, leaf first.
    pub(crate) chain: Vec<Vec<u8>>,
    /// The `signature_scheme` from `CertificateVerify`.
    signature_scheme: u16,
    /// The raw `CertificateVerify` body (scheme + signature).
    cert_verify_body: Vec<u8>,
}

/// Read and decrypt the encrypted server handshake records
/// (`EncryptedExtensions` … `Finished`) with `server_hs_key`, sequencing
/// from zero.
///
/// RFC 8446 §5.1 allows a handshake message to span records (rustls
/// fragments plaintext at 16 KiB, so a certificate chain larger than one
/// record is split) and a single record to carry several complete messages.
/// Decrypted plaintext is therefore accumulated into a buffer and each
/// message is consumed only once its full `type(1) || length(3) || body` is
/// present; a record may hold a partial message, several complete messages,
/// or a message continuing into the next record.
pub(crate) async fn read_server_hs_messages<S>(
    stream: &mut S,
    server_hs_key: &AeadKey,
    // RFC 8879 `compress_certificate` algorithms the ClientHello offered.
    offered_compress: &[u16],
) -> Result<ServerFlight>
where
    S: AsyncRead + Unpin,
{
    let mut ee_raw: Option<Vec<u8>> = None;
    let mut cert_raw: Option<Vec<u8>> = None;
    let mut cv_raw: Option<Vec<u8>> = None;
    let mut finished_data: Option<Vec<u8>> = None;
    let mut chain: Option<Vec<Vec<u8>>> = None;
    let mut signature_scheme = 0u16;
    let mut cert_verify_body = Vec::new();
    let mut seq = 0u64;
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Consume every complete handshake message buffered so far.
        let mut consumed = 0usize;
        while buf.len() >= consumed + 4 {
            let msg_len = u24(&buf[consumed + 1..consumed + 4]);
            let total = 4 + msg_len;
            if buf.len() < consumed + total {
                break; // body incomplete — wait for the next record
            }
            let msg_type = buf[consumed];
            let body = buf[consumed + 4..consumed + total].to_vec();
            consumed += total;

            let raw = make_hs_msg(msg_type, &body);
            match msg_type {
                HS_ENCRYPTED_EXTENSIONS => {
                    parse_encrypted_extensions(&body)?;
                    ee_raw = Some(raw);
                }
                HS_CERTIFICATE => {
                    chain = Some(parse_certificate_message(&body, offered_compress)?);
                    cert_raw = Some(raw);
                }
                HS_CERTIFICATE_VERIFY => {
                    signature_scheme = parse_certificate_verify(&body)?;
                    cert_verify_body = body;
                    cv_raw = Some(raw);
                }
                HS_FINISHED => finished_data = Some(body),
                _ => {} // unknown handshake messages are skipped
            }

            if finished_data.is_some() {
                // The server flight ends with Finished; any later messages
                // (e.g. NewSessionTicket) are not part of the flight, and a
                // buffered remainder is intentionally left unconsumed.
                return Ok(ServerFlight {
                    ee_raw: ee_raw
                        .ok_or_else(|| TlsError::Handshake("missing EncryptedExtensions".into()))?,
                    cert_raw: cert_raw.unwrap_or_else(|| make_hs_msg(HS_CERTIFICATE, &[])),
                    cv_raw: cv_raw.unwrap_or_else(|| make_hs_msg(HS_CERTIFICATE_VERIFY, &[])),
                    sf_verify_data: finished_data
                        .ok_or_else(|| TlsError::Handshake("missing server Finished".into()))?,
                    chain: chain.unwrap_or_default(),
                    signature_scheme,
                    cert_verify_body,
                });
            }
        }
        buf.drain(..consumed);

        let rec = skip_ccs(stream).await?;
        if rec.content_type != CONTENT_APPLICATION_DATA {
            return Err(TlsError::Handshake(format!(
                "expected encrypted record (0x17), got 0x{:02X}",
                rec.content_type
            )));
        }

        let aad = aead_aad(rec.payload.len());
        let mut payload = rec.payload;
        let plaintext = server_hs_key.open(seq, &aad, &mut payload)?;
        seq += 1;

        let (content_type, hs_data) = strip_padding(&plaintext)?;
        if content_type != CONTENT_HANDSHAKE {
            continue;
        }
        if buf.len() + hs_data.len() > MAX_FLIGHT_BUFFER {
            return Err(TlsError::Handshake(
                "server flight exceeds the 16 MiB reassembly bound".into(),
            ));
        }
        buf.extend_from_slice(hs_data);
    }
}

/// Parse an `EncryptedExtensions` body:
/// `ext_list_len(2) || extensions`.
///
/// The `supported_versions` extension (when present) must be TLS 1.3, and
/// the ALPN extension must be well-formed; neither value has a consumer in
/// the Task-8 seams yet, so both are validated and discarded.
fn parse_encrypted_extensions(body: &[u8]) -> Result<()> {
    if body.len() < 2 {
        return Err(TlsError::Handshake("EncryptedExtensions too short".into()));
    }
    let ext_list_len = usize::from(u16::from_be_bytes([body[0], body[1]]));
    let ext_end = 2 + ext_list_len;
    if ext_end > body.len() {
        return Err(TlsError::Handshake("EncryptedExtensions truncated".into()));
    }
    let mut pos = 2;
    while pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([body[pos], body[pos + 1]]);
        let ext_len = usize::from(u16::from_be_bytes([body[pos + 2], body[pos + 3]]));
        pos += 4;
        if pos + ext_len > ext_end {
            return Err(TlsError::Handshake(
                "EncryptedExtensions extension overruns list".into(),
            ));
        }
        let ext_data = &body[pos..pos + ext_len];
        pos += ext_len;
        match ext_type {
            EXT_SUPPORTED_VERSIONS => {
                // uint8 length (always 2) || uint16 version.
                if ext_data.len() != 3 || usize::from(ext_data[0]) != 2 {
                    return Err(TlsError::Handshake(
                        "malformed supported_versions in EncryptedExtensions".into(),
                    ));
                }
                let version = u16::from_be_bytes([ext_data[1], ext_data[2]]);
                if version != 0x0304 {
                    return Err(TlsError::Handshake(format!(
                        "server negotiated TLS version 0x{version:04X}, expected 0x0304"
                    )));
                }
            }
            EXT_ALPN => {
                // uint16 list length || (uint8 len || protocol)*.
                if ext_data.len() < 2 {
                    return Err(TlsError::Handshake(
                        "malformed ALPN in EncryptedExtensions".into(),
                    ));
                }
                let list_len = usize::from(u16::from_be_bytes([ext_data[0], ext_data[1]]));
                if ext_data.len() != 2 + list_len {
                    return Err(TlsError::Handshake(
                        "ALPN protocol list length mismatch".into(),
                    ));
                }
                let mut p = 2;
                while p < ext_data.len() {
                    let proto_len = usize::from(ext_data[p]);
                    p += 1;
                    if p + proto_len > ext_data.len() {
                        return Err(TlsError::Handshake(
                            "ALPN protocol entry overruns list".into(),
                        ));
                    }
                    p += proto_len;
                }
            }
            _ => {}
        }
    }
    if pos != ext_end {
        return Err(TlsError::Handshake(
            "trailing garbage in EncryptedExtensions extension list".into(),
        ));
    }
    Ok(())
}

/// Parse a `Certificate` body:
/// `cert_req_ctx_len(1) || ctx || certificate_list_len(3) || certificate_list`.
///
/// Each list entry is `cert_data_len(3) || cert_data || extensions_len(2) ||
/// extensions`; returns the DER chain, leaf first. Per RFC 8879 the server
/// may compress each `cert_data` with one of the `compress_certificate`
/// algorithms the client offered (see [`decompress_cert`]).
fn parse_certificate_message(body: &[u8], offered_compress: &[u16]) -> Result<Vec<Vec<u8>>> {
    if body.len() < 4 {
        return Err(TlsError::Handshake("Certificate message too short".into()));
    }
    let ctx_len = usize::from(body[0]);
    let mut pos = 1 + ctx_len;
    if pos + 3 > body.len() {
        return Err(TlsError::Handshake(
            "Certificate list length truncated".into(),
        ));
    }
    let list_len = u24(&body[pos..pos + 3]);
    pos += 3;
    let list_end = pos + list_len;
    if list_end > body.len() {
        return Err(TlsError::Handshake("Certificate list truncated".into()));
    }

    let mut chain = Vec::new();
    while pos < list_end {
        if pos + 3 > list_end {
            return Err(TlsError::Handshake(
                "certificate entry length truncated".into(),
            ));
        }
        let cert_len = u24(&body[pos..pos + 3]);
        pos += 3;
        if pos + cert_len > list_end {
            return Err(TlsError::Handshake("certificate data truncated".into()));
        }
        chain.push(decompress_cert(
            &body[pos..pos + cert_len],
            offered_compress,
        )?);
        pos += cert_len;
        if pos + 2 > list_end {
            return Err(TlsError::Handshake(
                "certificate extensions length truncated".into(),
            ));
        }
        let ext_len = usize::from(u16::from_be_bytes([body[pos], body[pos + 1]]));
        pos += 2 + ext_len;
        if pos > list_end {
            return Err(TlsError::Handshake(
                "certificate extensions overrun list".into(),
            ));
        }
    }
    Ok(chain)
}

/// Decompress one RFC 8879 `cert_data` entry.
///
/// The server compresses each certificate in the chain with one of the
/// `compress_certificate` algorithms the client offered (the same extension
/// list). A raw (uncompressed) DER certificate is used as-is; otherwise each
/// offered algorithm is tried in offer order and the first that yields valid
/// DER wins. If no offered algorithm recovers valid DER, the raw bytes are
/// returned so the verifier rejects a genuinely malformed certificate
/// cleanly.
fn decompress_cert(data: &[u8], offered: &[u16]) -> Result<Vec<u8>> {
    if is_der_cert(data) {
        return Ok(data.to_vec());
    }
    for &alg in offered {
        let out = match alg {
            0x0001 => flate2_zlib_decompress(data),
            0x0002 => brotli_decompress(data),
            0x0003 => zstd_decompress(data),
            _ => None,
        };
        if let Some(out) = out {
            if is_der_cert(&out) {
                return Ok(out);
            }
        }
    }
    Ok(data.to_vec())
}

/// Is `data` a complete DER X.509 certificate (a SEQUENCE whose length
/// field covers exactly the payload)? Compressed blobs never start with
/// `0x30` followed by a self-consistent length, so this distinguishes raw
/// from compressed `cert_data`.
fn is_der_cert(data: &[u8]) -> bool {
    if data.first() != Some(&0x30) {
        return false;
    }
    der_sequence_length(data) == Some(data.len())
}

/// Length in bytes of a DER SEQUENCE (`0x30`, short/long form) given its
/// header; `None` when the header is malformed or truncated.
fn der_sequence_length(data: &[u8]) -> Option<usize> {
    let b = *data.get(1)?;
    if b & 0x80 == 0 {
        return Some(2 + usize::from(b));
    }
    let n = usize::from(b & 0x7f);
    if n == 0 || n > 4 || data.len() < 2 + n {
        return None;
    }
    let mut len = 0usize;
    for &byte in &data[2..2 + n] {
        len = (len << 8) | usize::from(byte);
    }
    Some(2 + n + len)
}

fn flate2_zlib_decompress(data: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data)
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

fn brotli_decompress(data: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    brotli::Decompressor::new(data, 1 << 20)
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

fn zstd_decompress(data: &[u8]) -> Option<Vec<u8>> {
    zstd::stream::decode_all(data).ok()
}

/// Parse a `CertificateVerify` body: `signature_scheme(2) ||
/// signature_len(2) || signature`; returns the signature scheme.
fn parse_certificate_verify(body: &[u8]) -> Result<u16> {
    if body.len() < 4 {
        return Err(TlsError::Handshake("CertificateVerify too short".into()));
    }
    let scheme = u16::from_be_bytes([body[0], body[1]]);
    let sig_len = usize::from(u16::from_be_bytes([body[2], body[3]]));
    if body.len() != 4 + sig_len {
        return Err(TlsError::Handshake(
            "CertificateVerify signature length mismatch".into(),
        ));
    }
    Ok(scheme)
}

// ── Wire helpers ───────────────────────────────────────────────────────────

/// Wrap a handshake body as a raw handshake message: `type(1) || len(3) ||
/// body` (RFC 8446 §4).
pub(crate) fn make_hs_msg(msg_type: u8, body: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(4 + body.len());
    msg.push(msg_type);
    let len = u32::try_from(body.len()).unwrap_or(u32::MAX);
    msg.extend_from_slice(&len.to_be_bytes()[1..]);
    msg.extend_from_slice(body);
    msg
}

/// Read a 3-byte big-endian length (RFC 8446 lengths are uint24).
fn u24(b: &[u8]) -> usize {
    (usize::from(b[0]) << 16) | (usize::from(b[1]) << 8) | usize::from(b[2])
}

/// Remove TLS 1.3 record padding and extract the inner content type
/// (RFC 8446 §5.4): `actual_content || type_byte || optional zero padding`.
fn strip_padding(plaintext: &[u8]) -> Result<(u8, &[u8])> {
    let mut end = plaintext.len();
    while end > 0 && plaintext[end - 1] == 0 {
        end -= 1;
    }
    if end == 0 {
        return Err(TlsError::Handshake("all-zero TLSInnerPlaintext".into()));
    }
    let content_type = plaintext[end - 1];
    Ok((content_type, &plaintext[..end - 1]))
}

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::{
        AEAD_TAG_LEN, AcceptAll, CONTENT_APPLICATION_DATA, HRR_RANDOM, HS_CERTIFICATE,
        HS_CERTIFICATE_VERIFY, HS_ENCRYPTED_EXTENSIONS, HS_FINISHED, HS_SERVER_HELLO,
        HandshakeParams, connect, is_der_cert, make_hs_msg, parse_certificate_message,
        parse_certificate_verify, parse_encrypted_extensions, parse_server_hello,
        read_server_hs_messages,
    };
    use crate::crypto::{AeadKey, CipherSuiteId, KeySchedule};
    use crate::error::TlsError;
    use crate::record::{CONTENT_HANDSHAKE, aead_aad, make_app_data_record};
    use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

    // ── helpers ────────────────────────────────────────────────────────────

    /// Decodes a hex string into bytes (test helper).
    fn decode_hex(s: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(s.len() / 2);
        let mut hi = None;
        for b in s.bytes() {
            let v = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => panic!("invalid hex digit: {b:#x}"),
            };
            match hi {
                None => hi = Some(v),
                Some(h) => {
                    out.push((h << 4) | v);
                    hi = None;
                }
            }
        }
        assert_eq!(hi, None, "hex string must have even length");
        out
    }

    /// A Chrome-flavoured spec with only TLS 1.3 suites/groups/versions, the
    /// minimal surface a rustls server needs to complete a 1-RTT handshake.
    fn test_spec() -> ClientHelloSpec {
        ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![0x1301, 0x1302, 0x1303],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![
                ExtensionSpec::ServerName,
                ExtensionSpec::SupportedGroups(vec![0x001D]),
                ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519]),
                ExtensionSpec::SupportedVersions(vec![0x0304, 0x0303]),
                ExtensionSpec::SignatureAlgorithms(vec![
                    0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601,
                ]),
                ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
                ExtensionSpec::EcPointFormats,
                ExtensionSpec::SessionTicket,
                ExtensionSpec::PskKeyExchangeModes,
            ],
        }
    }

    fn server_config(cert: &rcgen::Certificate, key: &rcgen::KeyPair) -> rustls::ServerConfig {
        use rustls::pki_types::{CertificateDer, PrivateKeyDer};
        // Workspace feature unification compiles rustls with BOTH backends
        // (ring via our crates, aws-lc-rs via reqwest/hickory/quinn), so
        // rustls cannot auto-select a provider here — install ring
        // explicitly, matching `install_tls_provider()` in the binary.
        // Idempotent: a concurrent/earlier install returns `Err`, ignored.
        let _ = rustls::crypto::ring::default_provider().install_default();
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(cert.der().to_vec())],
                PrivateKeyDer::try_from(key.serialize_der()).unwrap(),
            )
            .unwrap()
    }

    // ── interop: our client vs a real rustls server ───────────────────────

    #[tokio::test]
    async fn handshake_completes_against_rustls_server() {
        let certified =
            rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
                .unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg = server_config(&certified.cert, &certified.signing_key);

        let server = tokio::task::spawn_blocking(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let timeout = std::time::Duration::from_secs(15);
            sock.set_read_timeout(Some(timeout)).unwrap();
            sock.set_write_timeout(Some(timeout)).unwrap();
            let mut conn = rustls::ServerConnection::new(Arc::new(cfg)).unwrap();
            while conn.is_handshaking() {
                conn.complete_io(&mut sock).unwrap();
            }
            // Read the client's 4-byte application data, echo it back, and
            // flush without issuing another blocking read.
            let mut echo = [0u8; 4];
            let mut got = 0;
            while got < echo.len() {
                conn.read_tls(&mut sock).unwrap();
                let state = conn.process_new_packets().unwrap();
                conn.write_tls(&mut sock).unwrap();
                if state.plaintext_bytes_to_read() > 0 {
                    got += conn.reader().read(&mut echo[got..]).unwrap();
                }
            }
            conn.writer().write_all(&echo).unwrap();
            conn.write_tls(&mut sock).unwrap();
        });

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let rng = ring::rand::SystemRandom::new();
        let mut tls = connect(
            stream,
            HandshakeParams {
                spec: &test_spec(),
                server_name: "localhost",
                alpn: Some(&["http/1.1"]),
                verifier: &AcceptAll,
                rng: &rng,
            },
        )
        .await
        .unwrap();

        tls.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        tls.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        server.await.unwrap();
    }

    // ── HRR detection ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn server_hello_with_hrr_magic_is_rejected() {
        let (client_stream, mut server_side) = tokio::io::duplex(8192);

        // A minimal HRR ServerHello: magic random, no extensions (a real HRR
        // carries no key_share for the rejected group).
        let mut body = Vec::with_capacity(40);
        body.extend_from_slice(&0x0303u16.to_be_bytes());
        body.extend_from_slice(&HRR_RANDOM);
        body.push(0); // session id length
        body.extend_from_slice(&0x1301u16.to_be_bytes());
        body.push(0); // compression method
        body.extend_from_slice(&0x0000u16.to_be_bytes()); // no extensions
        let mut sh_msg = vec![HS_SERVER_HELLO];
        let len = u32::try_from(body.len()).unwrap();
        sh_msg.extend_from_slice(&len.to_be_bytes()[1..]);
        sh_msg.extend_from_slice(&body);
        let mut rec = vec![0x16, 0x03, 0x03];
        rec.extend_from_slice(&u16::try_from(sh_msg.len()).unwrap().to_be_bytes());
        rec.extend_from_slice(&sh_msg);
        server_side.write_all(&rec).await.unwrap();

        let rng = ring::rand::SystemRandom::new();
        let result = connect(
            client_stream,
            HandshakeParams {
                spec: &test_spec(),
                server_name: "localhost",
                alpn: None,
                verifier: &AcceptAll,
                rng: &rng,
            },
        )
        .await;
        match result {
            Err(TlsError::HelloRetryRequest) => {}
            Err(e) => panic!("expected HelloRetryRequest, got {e:?}"),
            Ok(_) => panic!("expected HelloRetryRequest, got a TLS stream"),
        }
    }

    // ── parse unit tests (RFC 8448 §3 vectors) ────────────────────────────

    /// RFC 8448 §3 `ServerHello` handshake message (full, with 4-byte header):
    /// AES-128-GCM-SHA256, X25519 keyshare `c98288…e751f0f`.
    const SERVER_HELLO_VEC: &str = concat!(
        "020000560303a6af06a4121860dc5e6e60249cd34c95930c8ac5cb1434dac155772ed3e26928",
        "00130100002e00330024001d0020c9828876112095fe66762bdbf7c672e156d6cc253b833df1",
        "dd69b1b04e751f0f002b00020304",
    );

    /// RFC 8448 §3 `Certificate` handshake message (full, with 4-byte header):
    /// one RSA self-signed certificate (`0x308201ac…`).
    const CERTIFICATE_VEC: &str = concat!(
        "0b0001b9000001b50001b0308201ac30820115a003020102020102300d06092a864886f70d01",
        "010b0500300e310c300a06035504031303727361301e170d3136303733303031323335395a17",
        "0d3236303733303031323335395a300e310c300a0603550403130372736130819f300d06092a",
        "864886f70d010101050003818d0030818902818100b4bb498f8279303d980836399b36c6988c",
        "0c68de55e1bdb826d3901a2461eafd2de49a91d015abbc9a95137ace6c1af19eaa6af98c7ce",
        "d43120998e187a80ee0ccb0524b1b018c3e0b63264d449a6d38e22a5fda430846748030530ef",
        "0461c8ca9d9efbfae8ea6d1d03e2bd193eff0ab9a8002c47428a6d35a8d88d79f7f1e3f0203",
        "010001a31a301830090603551d1304023000300b0603551d0f0404030205a0300d06092a8648",
        "86f70d01010b05000381810085aad2a0e5b9276b908c65f73a7267170618a54c5f8a7b337d2d",
        "f7a594365417f2eae8f8a58c8f8172f9319cf36b7fd6c55b80f21a03015156726096fd335e5e",
        "67f2dbf102702e608ccae6bec1fc63a42a99be5c3eb7107c3c54e9b9eb2bd5203b1c3b84e0a8",
        "b2f759409ba3eac9d91d402dcc0cc8f8961229ac9187b42b4de10000",
    );

    /// RFC 8448 §3 `CertificateVerify` handshake message (full, with 4-byte
    /// header): `ecdsa_secp256r1_sha256` (0x0804), 128-byte signature.
    const CERTIFICATE_VERIFY_VEC: &str = concat!(
        "0f000084080400805a747c5d88fa9bd2e55ab085a61015b7211f824cd484145ab3ff52f1fda8",
        "477b0b7abc90db78e2d33a5c141a078653fa6bef780c5ea248eeaaa785c4f394cab6d30bbe8d",
        "4859ee511f602957b15411ac027671459e46445c9ea58c181e818e95b8c3fb0bf3278409d3be",
        "152a3da5043e063dda65cdf5aea20d53dfacd42f74f3",
    );

    #[test]
    fn parse_server_hello_extracts_suite_and_keyshare() {
        let msg = decode_hex(SERVER_HELLO_VEC);
        let sh = parse_server_hello(&msg[4..], &[]).unwrap();
        assert_eq!(sh.suite, crate::crypto::CipherSuiteId::Aes128GcmSha256);
        let expected: [u8; 32] =
            decode_hex("c9828876112095fe66762bdbf7c672e156d6cc253b833df1dd69b1b04e751f0f")
                .try_into()
                .unwrap();
        assert_eq!(sh.peer_key, expected);

        // A non-0x0303 legacy_version is rejected by the parser itself.
        let mut bad = msg[4..].to_vec();
        bad[1] = 0x04;
        assert!(parse_server_hello(&bad, &[]).is_err());
    }

    #[test]
    fn parse_server_hello_rejects_hrr_magic() {
        let mut body = vec![0x03, 0x03];
        body.extend_from_slice(&HRR_RANDOM);
        body.extend_from_slice(&[0x00, 0x13, 0x01, 0x00, 0x00, 0x00]);
        let Err(err) = parse_server_hello(&body, &[]) else {
            panic!("expected HelloRetryRequest, got a parsed ServerHello");
        };
        assert!(matches!(err, TlsError::HelloRetryRequest));
    }

    /// A minimal `ServerHello` body: TLS 1.3 legacy version, 32 zero random
    /// bytes, no session id, AES-128-GCM, no compression, the given raw
    /// extension bytes.
    fn minimal_server_hello(exts: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&0x0303u16.to_be_bytes());
        body.extend_from_slice(&[0xAB; 32]);
        body.push(0); // session id length
        body.extend_from_slice(&0x1301u16.to_be_bytes());
        body.push(0); // compression method
        body.extend_from_slice(&u16::try_from(exts.len()).unwrap().to_be_bytes());
        body.extend_from_slice(exts);
        body
    }

    /// A `key_share` extension for X25519 (0x001D) with a 32-byte key.
    fn x25519_keyshare_ext(key: &[u8; 32]) -> Vec<u8> {
        let mut ext = Vec::new();
        ext.extend_from_slice(&0x0033u16.to_be_bytes());
        ext.extend_from_slice(&0x0024u16.to_be_bytes()); // ext len 36
        ext.extend_from_slice(&0x001Du16.to_be_bytes()); // X25519
        ext.extend_from_slice(&0x0020u16.to_be_bytes()); // key len 32
        ext.extend_from_slice(key);
        ext
    }

    #[test]
    fn parse_server_hello_enforces_conformance_guards() {
        let key = [0x44u8; 32];
        let body = minimal_server_hello(&x25519_keyshare_ext(&key));
        assert_eq!(parse_server_hello(&body, &[]).unwrap().peer_key, key);

        // A keyshare for a group other than X25519 is rejected outright.
        let mut wrong_group = x25519_keyshare_ext(&key);
        wrong_group[4] = 0x00;
        wrong_group[5] = 0x13; // secp256r1
        assert!(parse_server_hello(&minimal_server_hello(&wrong_group), &[]).is_err());

        // A keyshare whose key length is not 32 is rejected outright.
        let mut wrong_len = x25519_keyshare_ext(&key);
        wrong_len[6] = 0x00;
        wrong_len[7] = 0x10; // key len 16
        assert!(parse_server_hello(&minimal_server_hello(&wrong_len), &[]).is_err());

        // The session id echo must match the offered one exactly.
        let mut sid_body = minimal_server_hello(&x25519_keyshare_ext(&key));
        sid_body[34] = 32; // session id length
        sid_body.splice(35..35, std::iter::repeat_n(0x11u8, 32));
        assert_eq!(
            parse_server_hello(&sid_body, &[0x11; 32]).unwrap().peer_key,
            key
        );
        assert!(parse_server_hello(&sid_body, &[0x22; 32]).is_err());

        // Bytes left over inside the declared extension list are rejected.
        let mut padded = x25519_keyshare_ext(&key);
        padded.extend_from_slice(&[0x00, 0x00]);
        let mut garbage = Vec::new();
        garbage.extend_from_slice(&0x0303u16.to_be_bytes());
        garbage.extend_from_slice(&[0xAB; 32]);
        garbage.push(0);
        garbage.extend_from_slice(&0x1301u16.to_be_bytes());
        garbage.push(0);
        garbage.extend_from_slice(&u16::try_from(padded.len()).unwrap().to_be_bytes());
        garbage.extend_from_slice(&padded);
        assert!(parse_server_hello(&garbage, &[]).is_err());
    }

    #[test]
    fn parse_certificate_message_extracts_leaf_first_chain() {
        let msg = decode_hex(CERTIFICATE_VEC);
        let chain = parse_certificate_message(&msg[4..], &[]).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].len(), 0x0000_01b0);
        assert_eq!(&chain[0][..2], &[0x30, 0x82]);

        // Two entries keep leaf-first order and tolerate per-cert extensions.
        let leaf = [0x30, 0x01, 0xAA];
        let root = [0x30, 0x01, 0xBB];
        let entry = |der: &[u8]| {
            let mut e = Vec::new();
            e.extend_from_slice(&u32::try_from(der.len()).unwrap().to_be_bytes()[1..]);
            e.extend_from_slice(der);
            e.extend_from_slice(&0x0002u16.to_be_bytes()); // 2 bytes of ext data
            e.extend_from_slice(&[0x00, 0x00]); // empty per-cert extensions
            e
        };
        let mut list = entry(&leaf);
        list.extend_from_slice(&entry(&root));
        let mut body = vec![0x00]; // no certificate_request_context
        body.extend_from_slice(&u32::try_from(list.len()).unwrap().to_be_bytes()[1..]);
        body.extend_from_slice(&list);
        let chain = parse_certificate_message(&body, &[]).unwrap();
        assert_eq!(chain, vec![leaf.to_vec(), root.to_vec()]);
    }

    #[test]
    fn parse_certificate_message_decompresses_offered_cert() {
        use std::io::{Read, Write};

        // A self-consistent DER blob (0x30 SEQUENCE, 0x81 long form, 26-byte
        // body → total 29 bytes) that `is_der_cert` recognizes after
        // decompression.
        let mut der = vec![0x30u8, 0x81, 26];
        der.extend_from_slice(&[0xAA; 26]);
        assert_eq!(is_der_cert(&der), true);

        // Server compressed it with brotli (0x0002) — one of the offered
        // algorithms. On drop the writer flushes the stream.
        let mut compressed = Vec::new();
        {
            let mut w = brotli::CompressorWriter::new(&mut compressed, 4096, 5, 22);
            w.write_all(&der).unwrap();
        }
        assert_eq!(is_der_cert(&compressed), false);

        // Wrap the compressed cert_data in a Certificate body.
        let mut entry = Vec::new();
        entry.extend_from_slice(&u32::try_from(compressed.len()).unwrap().to_be_bytes()[1..]);
        entry.extend_from_slice(&compressed);
        entry.extend_from_slice(&0x0000u16.to_be_bytes()); // no per-cert extensions
        let mut body = vec![0x00]; // no certificate_request_context
        body.extend_from_slice(&u32::try_from(entry.len()).unwrap().to_be_bytes()[1..]);
        body.extend_from_slice(&entry);

        // Without offering compression the raw bytes pass through.
        assert_eq!(
            parse_certificate_message(&body, &[]).unwrap(),
            vec![compressed.clone()]
        );
        // With brotli offered, the original DER is recovered.
        let chain = parse_certificate_message(&body, &[0x0001, 0x0002, 0x0003]).unwrap();
        assert_eq!(chain, vec![der]);
    }

    #[test]
    fn parse_certificate_verify_extracts_signature_scheme() {
        let msg = decode_hex(CERTIFICATE_VERIFY_VEC);
        assert_eq!(parse_certificate_verify(&msg[4..]).unwrap(), 0x0804);
    }

    #[test]
    fn parse_encrypted_extensions_validates_alpn_and_versions() {
        // supported_versions = 0x0304, ALPN = [h2, http/1.1].
        let mut exts = Vec::new();
        exts.extend_from_slice(&0x002bu16.to_be_bytes());
        exts.extend_from_slice(&[0x00, 0x03, 0x02, 0x03, 0x04]);
        let alpn = [
            0x00, 0x0c, 0x02, b'h', b'2', 0x08, b'h', b't', b't', b'p', b'/', b'1', b'.', b'1',
        ];
        exts.extend_from_slice(&0x0010u16.to_be_bytes());
        exts.extend_from_slice(&u16::try_from(alpn.len()).unwrap().to_be_bytes());
        exts.extend_from_slice(&alpn);
        let mut body = Vec::new();
        body.extend_from_slice(&u16::try_from(exts.len()).unwrap().to_be_bytes());
        body.extend_from_slice(&exts);
        parse_encrypted_extensions(&body).unwrap();

        // An empty EE is valid: no ALPN negotiated, no supported_versions.
        parse_encrypted_extensions(&[0x00, 0x00]).unwrap();

        // A declared extension list longer than its content is trailing
        // garbage and rejected.
        let mut garbage = vec![0x00, 0x05];
        garbage.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(parse_encrypted_extensions(&garbage).is_err());
    }

    #[test]
    fn parse_encrypted_extensions_rejects_unsupported_version() {
        let mut exts = Vec::new();
        exts.extend_from_slice(&0x002bu16.to_be_bytes());
        exts.extend_from_slice(&[0x00, 0x03, 0x02, 0x03, 0x03]); // TLS 1.2
        let mut body = Vec::new();
        body.extend_from_slice(&u16::try_from(exts.len()).unwrap().to_be_bytes());
        body.extend_from_slice(&exts);
        assert!(parse_encrypted_extensions(&body).is_err());
    }

    // ── multi-record flight reassembly (RFC 8446 §5.1) ────────────────────

    /// Seal `chunk` as one TLS 1.3 record whose `TLSInnerPlaintext` is
    /// `chunk || CONTENT_HANDSHAKE` (no padding), under `key` with `seq`.
    async fn write_encrypted_flight_chunk<W: tokio::io::AsyncWrite + Unpin>(
        w: &mut W,
        key: &AeadKey,
        seq: u64,
        chunk: &[u8],
    ) {
        let mut inner = chunk.to_vec();
        inner.push(CONTENT_HANDSHAKE);
        let ciphertext = key
            .seal(seq, &aead_aad(inner.len() + AEAD_TAG_LEN), &inner)
            .unwrap();
        w.write_all(&make_app_data_record(&ciphertext))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn server_flight_parses_certificate_split_across_records() {
        let suite = CipherSuiteId::Aes128GcmSha256;
        let key = AeadKey::from_key_bytes(suite, &[0x11; 16]).unwrap();

        // A certificate chain larger than one record's 16 KiB plaintext, so
        // the Certificate handshake message MUST span two TLS records (the
        // rustls fragmentation a >16 KiB chain produces).
        let fake_cert = vec![0x30u8; 20_000];
        let mut entry = Vec::with_capacity(3 + fake_cert.len() + 2);
        entry.extend_from_slice(&u32::try_from(fake_cert.len()).unwrap().to_be_bytes()[1..]);
        entry.extend_from_slice(&fake_cert);
        entry.extend_from_slice(&0x0000u16.to_be_bytes()); // no per-cert extensions
        let mut cert_body = vec![0x00]; // no certificate_request_context
        cert_body.extend_from_slice(&u32::try_from(entry.len()).unwrap().to_be_bytes()[1..]);
        cert_body.extend_from_slice(&entry);
        let cert_msg = make_hs_msg(HS_CERTIFICATE, &cert_body);
        assert!(cert_msg.len() > 16_384, "Certificate must span records");

        let ee_msg = make_hs_msg(HS_ENCRYPTED_EXTENSIONS, &[0x00, 0x00]); // empty ext list
        let mut cv_body = Vec::new();
        cv_body.extend_from_slice(&0x0403u16.to_be_bytes()); // ecdsa_secp256r1_sha256
        cv_body.extend_from_slice(&0x0040u16.to_be_bytes());
        cv_body.extend_from_slice(&[0xAA; 64]);
        let cv_msg = make_hs_msg(HS_CERTIFICATE_VERIFY, &cv_body);
        let finished_msg = make_hs_msg(HS_FINISHED, &[0xBB; 32]);

        // Record 1 ends inside the Certificate message; record 2 carries the
        // remainder of the flight.
        let mut raw_flight = Vec::new();
        raw_flight.extend_from_slice(&ee_msg);
        raw_flight.extend_from_slice(&cert_msg);
        raw_flight.extend_from_slice(&cv_msg);
        raw_flight.extend_from_slice(&finished_msg);
        let split = ee_msg.len() + cert_msg.len() / 2;
        assert!(
            split < ee_msg.len() + cert_msg.len(),
            "split inside Certificate"
        );

        let (mut client_stream, mut server_side) = tokio::io::duplex(64 * 1024);
        write_encrypted_flight_chunk(&mut server_side, &key, 0, &raw_flight[..split]).await;
        write_encrypted_flight_chunk(&mut server_side, &key, 1, &raw_flight[split..]).await;

        let parsed = read_server_hs_messages(&mut client_stream, &key, &[])
            .await
            .unwrap();
        assert_eq!(parsed.chain.len(), 1);
        assert_eq!(parsed.chain[0], fake_cert);
        assert_eq!(parsed.signature_scheme, 0x0403);
        assert_eq!(parsed.cert_verify_body, cv_body);
        assert_eq!(parsed.sf_verify_data, vec![0xBB; 32]);
    }

    // ── hybrid X25519MLKEM768 handshake (fake PQ server double) ───────────

    /// A loopback server double that negotiates X25519MLKEM768: it
    /// encapsulates to the client's ML-KEM-768 key share, feeds
    /// `classical || pq` through the same key schedule, and proves both
    /// sides derived matching traffic keys by an encrypted app-data echo.
    #[tokio::test]
    async fn hybrid_handshake_completes_against_fake_pq_server() {
        use crate::crypto::X25519KeyPair;
        use crate::crypto::mlkem::{Mlkem768, PublicKey as MlkemPublicKey};
        use crate::hello::parse::parse_hello;

        let suite = CipherSuiteId::Aes128GcmSha256;
        let mut spec = test_spec();
        for ext in &mut spec.extensions {
            match ext {
                ExtensionSpec::SupportedGroups(groups) => *groups = vec![0x11EC, 0x001D],
                ExtensionSpec::KeyShare(groups) => *groups = vec![KeyShareGroup::X25519Mlkem768],
                _ => {}
            }
        }

        let (client_stream, mut server_side) = tokio::io::duplex(64 * 1024);

        let server = tokio::spawn(async move {
            // Read the ClientHello record and pull the hybrid key share.
            let mut hdr = [0u8; 5];
            server_side.read_exact(&mut hdr).await.unwrap();
            assert_eq!(hdr[0], 0x16);
            let mut ch = vec![0u8; u16::from_be_bytes([hdr[3], hdr[4]]) as usize];
            server_side.read_exact(&mut ch).await.unwrap();
            let parsed = parse_hello(&ch).unwrap();
            let ks_ext = parsed.extension(0x0033).unwrap();
            // list_len(2) + entry(group 11ec, kx_len 04c0, 1216 bytes) in
            // the Go wire order: ML-KEM ek (1184) first, X25519 pub last.
            assert_eq!(&ks_ext[..4], &[0x04, 0xc4, 0x11, 0xec]);
            let client_mlkem_pk = MlkemPublicKey::from_bytes(&ks_ext[6..6 + 1184]).unwrap();
            let client_x25519: [u8; 32] = ks_ext[6 + 1184..6 + 1216].try_into().unwrap();

            // Classical + PQ shared secrets, exactly as a real hybrid server.
            let rng = ring::rand::SystemRandom::new();
            let server_kp = X25519KeyPair::generate(&rng).unwrap();
            let classical = server_kp.agree(&client_x25519).unwrap();
            let (ct, pq_ss) = Mlkem768::encapsulate(&client_mlkem_pk).unwrap();

            // ServerHello: key_share group 11ec, share = ct(1088) || pub(32).
            let mut sh_body = Vec::new();
            sh_body.extend_from_slice(&0x0303u16.to_be_bytes());
            sh_body.extend_from_slice(&[0x5A; 32]);
            sh_body.push(u8::try_from(parsed.session_id.len()).unwrap());
            sh_body.extend_from_slice(&parsed.session_id);
            sh_body.extend_from_slice(&0x1301u16.to_be_bytes());
            sh_body.push(0); // compression method
            let mut kse = Vec::new();
            kse.extend_from_slice(&0x0033u16.to_be_bytes());
            kse.extend_from_slice(&1124u16.to_be_bytes()); // 4 + 1120
            kse.extend_from_slice(&0x11ECu16.to_be_bytes());
            kse.extend_from_slice(&1120u16.to_be_bytes());
            // Go wire order: ct (1088) first, X25519 pub (32) last.
            kse.extend_from_slice(ct.as_bytes());
            kse.extend_from_slice(&server_kp.public_key());
            sh_body.extend_from_slice(&u16::try_from(kse.len()).unwrap().to_be_bytes());
            sh_body.extend_from_slice(&kse);
            let sh_msg = make_hs_msg(HS_SERVER_HELLO, &sh_body);
            let mut rec = vec![0x16, 0x03, 0x03];
            rec.extend_from_slice(&u16::try_from(sh_msg.len()).unwrap().to_be_bytes());
            rec.extend_from_slice(&sh_msg);
            server_side.write_all(&rec).await.unwrap();

            // Key schedule over `pq || classical` — the Go hybrid contract.
            let mut combined = pq_ss.as_bytes().to_vec();
            combined.extend_from_slice(&classical);
            assert_eq!(combined.len(), 64);
            let mut sk = KeySchedule::new(suite);
            sk.add_transcript(&ch);
            sk.add_transcript(&sh_msg);
            let hs_secret = sk.handshake_secret(&combined).unwrap();
            let (_client_hs_ts, server_hs_ts) = sk.handshake_traffic_secrets(&hs_secret).unwrap();
            let server_hs_key = AeadKey::new(suite, &server_hs_ts).unwrap();

            // Encrypted flight with a REAL server Finished MAC.
            let ee_msg = make_hs_msg(HS_ENCRYPTED_EXTENSIONS, &[0x00, 0x00]);
            let cert_msg = make_hs_msg(
                HS_CERTIFICATE,
                &[
                    0x00, // no certificate_request_context
                    0x00, 0x00, 0x08, // certificate_list length
                    0x00, 0x00, 0x03, 0x30, 0x01, 0xAA, 0x00, 0x00, // one entry
                ],
            );
            let mut cv_body = Vec::new();
            cv_body.extend_from_slice(&0x0403u16.to_be_bytes());
            cv_body.extend_from_slice(&0x0040u16.to_be_bytes());
            cv_body.extend_from_slice(&[0xAA; 64]);
            let cv_msg = make_hs_msg(HS_CERTIFICATE_VERIFY, &cv_body);
            sk.add_transcript(&ee_msg);
            sk.add_transcript(&cert_msg);
            sk.add_transcript(&cv_msg);
            let sf_key = sk.finished_key(&server_hs_ts).unwrap();
            let sf_wire = make_hs_msg(HS_FINISHED, &sk.finished_mac(&sf_key));
            let mut flight = Vec::new();
            flight.extend_from_slice(&ee_msg);
            flight.extend_from_slice(&cert_msg);
            flight.extend_from_slice(&cv_msg);
            flight.extend_from_slice(&sf_wire);
            sk.add_transcript(&sf_wire);
            write_encrypted_flight_chunk(&mut server_side, &server_hs_key, 0, &flight).await;

            // Client Finished record (seq 0 under the client hs key) — skip.
            let mut fin_hdr = [0u8; 5];
            server_side.read_exact(&mut fin_hdr).await.unwrap();
            let mut fin = vec![0u8; u16::from_be_bytes([fin_hdr[3], fin_hdr[4]]) as usize];
            server_side.read_exact(&mut fin).await.unwrap();

            // App traffic secrets; decrypt and echo the client's ping.
            let master = sk.master_secret(&hs_secret).unwrap();
            let (client_app_ts, server_app_ts) = sk.app_traffic_secrets(&master).unwrap();
            let client_app_key = AeadKey::new(suite, &client_app_ts).unwrap();
            let server_app_key = AeadKey::new(suite, &server_app_ts).unwrap();

            let mut ping_hdr = [0u8; 5];
            server_side.read_exact(&mut ping_hdr).await.unwrap();
            let mut ping_ct = vec![0u8; u16::from_be_bytes([ping_hdr[3], ping_hdr[4]]) as usize];
            server_side.read_exact(&mut ping_ct).await.unwrap();
            let ping = client_app_key
                .open(0, &aead_aad(ping_ct.len()), &mut ping_ct)
                .unwrap();
            assert!(ping.starts_with(b"ping"));

            let mut inner = b"ping".to_vec();
            inner.push(CONTENT_APPLICATION_DATA);
            let echo_ct = server_app_key
                .seal(0, &aead_aad(inner.len() + AEAD_TAG_LEN), &inner)
                .unwrap();
            server_side
                .write_all(&make_app_data_record(&echo_ct))
                .await
                .unwrap();
        });

        let rng = ring::rand::SystemRandom::new();
        let mut tls = connect(
            client_stream,
            HandshakeParams {
                spec: &spec,
                server_name: "localhost",
                alpn: None,
                verifier: &AcceptAll,
                rng: &rng,
            },
        )
        .await
        .unwrap();
        tls.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        tls.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        server.await.unwrap();
    }
}
