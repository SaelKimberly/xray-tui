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
pub mod verify;

use std::ops::Range;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::SecureRandom;
use crate::crypto::{AeadKey, KeySchedule, X25519KeyPair};
use crate::error::{Result, TlsError};
use crate::handshake::{RingRng, make_hs_msg, read_server_hello, read_server_hs_messages};
use crate::hello::{BuildParams, build_hello, to_record};
use crate::record::stream::{AppKeys, TlsStream};
use crate::record::{CONTENT_HANDSHAKE, HS_FINISHED, aead_aad, make_app_data_record};
use crate::spec::SessionIdSpec;

/// AEAD tag length for the TLS 1.3 AEADs in this engine (all 16 bytes).
const AEAD_TAG_LEN: usize = 16;

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

impl HelloProvisioner for FixedChrome133 {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        // Chrome-133 fingerprint with the REALITY AuthPayload session id
        // (16 B plaintext + 16 B GCM tag, zeroed for the connector to seal).
        let mut spec = crate::profiles::chrome133::spec();
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
            TlsError::Spec("Chrome133 spec must use SessionIdSpec::AuthPayload".into())
        })?;
        Ok(ProvisionedHello {
            handshake_bytes: built.handshake_bytes,
            session_id_range,
        })
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
    /// Random source. ring's `SecureRandom` is sealed, so the object is
    /// always `Send + Sync` in practice; the bounds are declared on the
    /// trait object so `RealityParams` itself is `Send`.
    pub rng: &'a (dyn ring::rand::SecureRandom + Send + Sync),
}

/// Performs the full REALITY client handshake over `stream`, returning a
/// record-framed application-data stream.
///
/// Implements the 9-step wire contract above; the driver mirrors
/// `handshake::connect` with the REALITY session-id seal and the
/// HMAC/Ed25519 server auth in place of the verifier seam.
pub async fn connect_reality<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S,
    params: RealityParams<'_>,
) -> Result<TlsStream<S>> {
    let mut stream = stream;

    // 1. Ephemeral X25519 key pair (the ClientHello keyshare). The
    //    connector draws no randomness of its own: the client random inside
    //    the provisioned hello *is* the REALITY random.
    let keypair = X25519KeyPair::generate(params.rng)?;

    // 2. Provision the Chrome-133-shaped ClientHello with a zeroed
    //    AuthPayload slot; the connector's keyshare and rng flow through.
    let rng = RingRng(params.rng);
    let mut hello = params.provisioner.provision(&HelloProvisionParams {
        server_name: params.server_name,
        alpn: Some(REALITY_ALPN),
        x25519_pub: &keypair.public_key(),
        rng: &rng,
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

    // 6. Send the ClientHello record.
    stream.write_all(&to_record(&hello.handshake_bytes)).await?;

    // 7. ServerHello (middlebox CCS skipped; HRR rejected by the parser).
    //    Its keyshare is the server's *ephemeral* TLS key (not the static
    //    `pbk`), so no static-key comparison is performed. The server must
    //    echo the sealed session id (the REALITY auth payload).
    let server_hello = read_server_hello(&mut stream, hello.session_id()).await?;

    // 8. TLS 1.3 ECDHE with the server's keyshare: the same ephemeral
    //    scalar drives both the auth key (step 4) and this shared secret.
    let tls_shared = keypair.agree(&server_hello.peer_key)?;

    // 9. Handshake traffic secrets under the selected suite; the transcript
    //    is the wire ClientHello (sealed session id) || ServerHello.
    let mut ks = KeySchedule::new(server_hello.suite);
    ks.add_transcript(&hello.handshake_bytes);
    ks.add_transcript(&server_hello.raw);
    let hs_secret = ks.handshake_secret(&tls_shared)?;
    let (client_hs_ts, server_hs_ts) = ks.handshake_traffic_secrets(&hs_secret)?;
    let server_hs_key = AeadKey::new(server_hello.suite, &server_hs_ts)?;
    let client_hs_key = AeadKey::new(server_hello.suite, &client_hs_ts)?;

    // 9. Decrypt the encrypted server flight (EE, Certificate,
    //    CertificateVerify, Finished) with the server handshake key.
    let flight = read_server_hs_messages(&mut stream, &server_hs_key).await?;

    // 10. REALITY server authentication: certificate HMAC + Ed25519
    //     CertificateVerify over the transcript (no PKI chain).
    let cert_der = flight
        .chain
        .first()
        .ok_or_else(|| TlsError::Verify("REALITY server sent no certificate".into()))?;
    let mut transcript = Vec::with_capacity(
        hello.handshake_bytes.len()
            + server_hello.raw.len()
            + flight.ee_raw.len()
            + flight.cert_raw.len(),
    );
    transcript.extend_from_slice(&hello.handshake_bytes);
    transcript.extend_from_slice(&server_hello.raw);
    transcript.extend_from_slice(&flight.ee_raw);
    transcript.extend_from_slice(&flight.cert_raw);
    verify::verify_server(
        cert_der,
        &flight.cv_raw,
        &auth_key,
        &transcript,
        server_hello.suite.digest(),
    )?;

    // 11. Authenticate the server Finished over ClientHello..CertificateVerify.
    ks.add_transcript(&flight.ee_raw);
    ks.add_transcript(&flight.cert_raw);
    ks.add_transcript(&flight.cv_raw);
    let server_finished_key = ks.finished_key(&server_hs_ts)?;
    if ks.finished_mac(&server_finished_key) != flight.sf_verify_data {
        return Err(TlsError::Handshake(
            "server Finished MAC mismatch — possible MITM or wrong key".into(),
        ));
    }

    // 12. Client Finished over ClientHello..server Finished (RFC 8446
    //     §4.4.4), sealed with the client handshake key.
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

    // 13. Application traffic secrets over ClientHello..server Finished
    //     (the client Finished joins the transcript afterwards, matching
    //     handshake::connect's resumption-path accumulation).
    let master = ks.master_secret(&hs_secret)?;
    let (client_app_ts, server_app_ts) = ks.app_traffic_secrets(&master)?;
    let client_app_key = AeadKey::new(server_hello.suite, &client_app_ts)?;
    let server_app_key = AeadKey::new(server_hello.suite, &server_app_ts)?;
    ks.add_transcript(&cf_hs_msg);

    Ok(TlsStream::new(
        stream,
        AppKeys {
            read_key: server_app_key,
            write_key: client_app_key,
            read_seq: 0,
            write_seq: 0,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hello::parse::parse_hello;

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
}
