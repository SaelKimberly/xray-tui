//! VLESS `mlkem768x25519plus` payload encryption (xray
//! `proxy/vless/encryption` — the client side of the ML-KEM-768 + X25519
//! PFS handshake).
//!
//! Wire summary (client → server):
//! - `ClientHello`: `[16B random IV][relays][pfs key exchange][padding]`.
//!   One relay block per configured server key, in order: an ephemeral
//!   X25519 public key (32 B) or an ML-KEM-768 ciphertext (1088 B). With
//!   `xorpub`/`random` each relay block is XOR-masked with AES-256-CTR
//!   keyed by the server's own key material (`blake3.DeriveKey("VLESS",
//!   key)`); consecutive blocks are chained by XOR-masking a 32-byte
//!   blake3 hash of the next server key with a CTR stream keyed by the
//!   previous shared secret. The pfs exchange is NFS-AEAD-sealed
//!   (`[2B BE len][ephemeral ML-KEM pk 1184 || ephemeral X25519 pk 32]`),
//!   followed by sealed padding fragments sent with configurable delays.
//! - `ServerHello` (read): `[NFS-AEAD(MaxNonce) server pfs material
//!   1088+32+16][AEAD ticket 32][AEAD [2B padding length]]` then the
//!   padding itself.
//! - **Post-handshake records** both ways: `[23 03 03 len_hi len_lo]
//!   [AEAD(payload ≤8192)]`, nonce auto-incrementing from zero; the two
//!   directions use different AEAD contexts (each side's PFS public
//!   material). `random` mode additionally XOR-maskes everything except
//!   the 5-byte record headers (per-direction CTR streams in `CommonConn`).
//!
//! Deviations from upstream (documented in the task report): no 0-RTT
//! ticket resume — `0rtt` accounts still run the full 1-RTT handshake
//! (wire-valid; the server issues tickets that we never replay); AES-GCM
//! is never selected — the client always seals with ChaCha20-Poly1305 and
//! the server accepts either.

use super::b3;

use base64::Engine as _;
use chacha20poly1305::aead::Aead;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ring::rand::SecureRandom;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use aes::Aes256;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce as ChachaNonce};
use ctr::Ctr128BE;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use xray_tui_proto::proto_spec::MlkemEncryption;
use xray_tui_proto::proto_spec::MlkemMode;
use xray_tui_tls::crypto::X25519KeyPair;
use xray_tui_tls::crypto::mlkem::{Ciphertext, Mlkem768, PublicKey};

use crate::BoxStream;
use crate::error::{NativeError, timeouts};

/// ML-KEM-768 encapsulation-key / ciphertext length (FIPS 203).
pub const MLKEM_CT_LEN: usize = 1088;
/// X25519 public-key length.
const X25519_LEN: usize = 32;
/// AEAD tag length (Poly1305).
const TAG_LEN: usize = 16;
/// Record header length (`23 03 03 len_hi len_lo`).
const HEADER_LEN: usize = 5;
/// Maximum plaintext chunk per record (xray `CommonConn.Write`).
const MAX_CHUNK: usize = 8192;
/// Record-length bounds (xray `DecodeHeader`: TLS 1.3 max record + tag).
const MIN_RECORD: usize = 17;
const MAX_RECORD: usize = 16_640;
/// The all-ones nonce used for the fixed-context NFS seal of the server's
/// PFS material (xray `MaxNonce`).
const MAX_NONCE: [u8; 12] = [0xff; 12];
/// `ClientHello` layout constants (xray `ClientInstance.Handshake`).
const IV_LEN: usize = 16;
/// `18 + 1184 + 32 + 16`: sealed length prefix + client PFS public keys +
/// tags.
const PFS_EXCHANGE_LEN: usize = 18 + 1184 + X25519_LEN + TAG_LEN;

// ── wire helpers ───────────────────────────────────────────────────────────

/// Big-endian u16 into two bytes (xray `EncodeLength`).
#[allow(clippy::cast_possible_truncation)] // l < 2^16 by construction
const fn encode_length(l: usize) -> [u8; 2] {
    [(l >> 8) as u8, l as u8]
}

/// Big-endian two bytes into u16 (xray `DecodeLength`).
const fn decode_length(b: &[u8]) -> usize {
    ((b[0] as usize) << 8) | (b[1] as usize)
}

/// The AEAD cipher both directions use (xray `NewAEAD`): a ChaCha20-Poly1305
/// instance keyed by `blake3.DeriveKey(ctx, united_key)` with an
/// auto-incrementing 12-byte counter nonce starting at zero.
struct WireAead {
    cipher: ChaCha20Poly1305,
    nonce: [u8; 12],
}

impl WireAead {
    fn new(ctx: &[u8], key: &[u8]) -> Self {
        let derived = b3::derive_key_bytes(ctx, key);
        Self {
            cipher: ChaCha20Poly1305::new((&derived).into()),
            nonce: [0; 12],
        }
    }

    /// xray `IncreaseNonce`: little-endian increment from the last byte.
    fn increase_nonce(&mut self) -> [u8; 12] {
        for i in 0..12 {
            self.nonce[11 - i] += 1;
            if self.nonce[11 - i] != 0 {
                break;
            }
        }
        self.nonce
    }

    fn seal(&mut self, plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        let n = self.increase_nonce();
        let nonce = ChachaNonce::from(n);
        let nonce = &nonce;
        self.cipher
            .encrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .expect("chacha20poly1305 seal cannot fail")
    }

    fn open(&mut self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, io::Error> {
        let n = self.increase_nonce();
        let nonce = ChachaNonce::from(n);
        let nonce = &nonce;
        self.cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "aead open failed"))
    }

    /// Seal under an explicit nonce — the fake-server tests' `MaxNonce`
    /// PFS seal (server.go `nfsAEAD.Seal(serverHello[:0], MaxNonce, ..)`).
    #[cfg(test)]
    fn seal_with(&self, nonce: [u8; 12], plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        self.cipher
            .encrypt(
                &ChachaNonce::from(nonce),
                chacha20poly1305::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .expect("chacha20poly1305 seal cannot fail")
    }

    fn open_with(
        &self,
        nonce: [u8; 12],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, io::Error> {
        self.cipher
            .decrypt(
                &ChachaNonce::from(nonce),
                chacha20poly1305::aead::Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "aead open failed"))
    }
}

/// AES-256-CTR keystream keyed like xray's `NewCTR`: the raw key material is
/// first run through `blake3.DeriveKey("VLESS", ..)`. Go's `cipher.NewCTR`
/// treats the full 16-byte IV as the initial big-endian counter block —
/// `Ctr128BE`.
fn new_ctr(key: &[u8], iv: &[u8; 16]) -> Ctr128BE<Aes256> {
    let derived = b3::derive_key_bytes(b"VLESS", key);
    Ctr128BE::<Aes256>::new(&derived.into(), iv.into())
}

/// Uniform random integer in `[lo, hi]` (xray `crypto.RandBetween`).
#[allow(clippy::cast_possible_truncation)] // usize >= u64 on supported targets
fn rand_between(lo: usize, hi: usize, rng: &ring::rand::SystemRandom) -> usize {
    debug_assert!(hi >= lo);
    let span = hi - lo + 1;
    let mut buf = [0u8; 8];
    rng.fill(&mut buf).expect("ring CSPRNG fills");
    lo + (u64::from_be_bytes(buf) % span as u64) as usize
}

// ── config ─────────────────────────────────────────────────────────────────

/// A configured server authentication key: X25519 (32 bytes) or ML-KEM-768
/// encapsulation key (1184 bytes).
#[derive(Debug, Clone)]
pub enum ServerKey {
    X25519([u8; X25519_LEN]),
    Mlkem(Vec<u8>),
}

impl ServerKey {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::X25519(k) => k.as_slice(),
            Self::Mlkem(k) => k,
        }
    }
}

/// Ready-to-use encryption parameters (parsed string + decoded keys +
/// parsed padding).
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub mode: MlkemMode,
    /// xray `account.Seconds` — `1` for `0rtt` accounts. Ticket-based
    /// resume is not implemented (deviation); every dial runs the full
    /// handshake.
    pub seconds: u32,
    pub padding_lens: Vec<[usize; 3]>,
    pub padding_gaps: Vec<[usize; 3]>,
    pub keys: Vec<ServerKey>,
}

impl EncryptionConfig {
    /// Build from the proto-parsed value: decodes the base64url key
    /// segments and parses the padding spec (xray `ParsePadding` rules).
    pub fn try_from_parsed(parsed: &MlkemEncryption) -> Result<Self, NativeError> {
        let invalid = |what: String| NativeError::Config(format!("vless mlkem encryption: {what}"));
        let mut keys = Vec::new();
        for seg in parsed.keys.split('.') {
            if seg.is_empty() {
                return Err(invalid("empty key segment".into()));
            }
            let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(seg)
                .map_err(|_| invalid("invalid base64url key segment".into()))?;
            match decoded.len() {
                32 => {
                    let mut k = [0u8; 32];
                    k.copy_from_slice(&decoded);
                    keys.push(ServerKey::X25519(k));
                }
                1184 => keys.push(ServerKey::Mlkem(decoded)),
                other => {
                    return Err(invalid(format!(
                        "key segment must be 32 or 1184 bytes, got {other}"
                    )));
                }
            }
        }
        if keys.is_empty() {
            return Err(invalid("no server keys".into()));
        }
        let (padding_lens, padding_gaps) = parse_padding(&parsed.padding)?;
        Ok(Self {
            mode: parsed.mode,
            seconds: parsed.seconds,
            padding_lens,
            padding_gaps,
            keys,
        })
    }
}

/// Parse the padding spec (xray `ParsePadding`): dot-separated
/// `probability-min-max` triples, alternating length/delay blocks starting
/// with a length block. The first block requires probability ≥100 and
/// min ≥35; total max padding ≤65553.
/// One `probability-min-max` padding block (xray `[3]int`).
type PaddingSpec = [usize; 3];

fn parse_padding(padding: &str) -> Result<(Vec<PaddingSpec>, Vec<PaddingSpec>), NativeError> {
    let invalid = |s: &str| NativeError::Config(format!("vless mlkem padding parameter: {s}"));
    let mut lens = Vec::new();
    let mut gaps = Vec::new();
    if padding.is_empty() {
        return Ok((lens, gaps));
    }
    let mut max_len = 0usize;
    for (i, s) in padding.split('.').enumerate() {
        let x: Vec<&str> = s.split('-').collect();
        if x.len() < 3 || x.iter().any(|p| p.is_empty()) {
            return Err(invalid(s));
        }
        let mut y = [0usize; 3];
        for (slot, part) in y.iter_mut().zip(x) {
            *slot = part.parse().map_err(|_| invalid(s))?;
        }
        if i == 0 && (y[0] < 100 || y[1] < 18 + 17 || y[2] < 18 + 17) {
            return Err(NativeError::Config(
                "vless mlkem padding: first padding length must not be smaller than 35".into(),
            ));
        }
        if i % 2 == 0 {
            max_len += y[1].max(y[2]);
            lens.push(y);
        } else {
            gaps.push(y);
        }
    }
    if max_len > 18 + 65_535 {
        return Err(NativeError::Config(
            "vless mlkem padding: total padding length must not be larger than 65553".into(),
        ));
    }
    Ok((lens, gaps))
}

/// Draw one padded fragment plan (xray `CreatPadding`): each block fires
/// with its probability, producing a length/gap in `[min, max]`; empty
/// specs fall back to the documented defaults.
fn create_padding(
    lens: &[[usize; 3]],
    gaps: &[[usize; 3]],
    rng: &ring::rand::SystemRandom,
) -> (Vec<usize>, Vec<usize>) {
    let default_lens: [[usize; 3]; 2] = [[100, 111, 1111], [50, 0, 3333]];
    let default_gaps: [[usize; 3]; 1] = [[75, 0, 111]];
    let (lens, gaps) = if lens.is_empty() {
        (default_lens.as_slice(), default_gaps.as_slice())
    } else {
        (lens, gaps)
    };
    let draw = |y: [usize; 3]| {
        if y[0] >= rand_between(0, 100, rng) {
            rand_between(y[1], y[2], rng)
        } else {
            0
        }
    };
    let frag_lens: Vec<usize> = lens.iter().copied().map(draw).collect();
    let gap_ms: Vec<usize> = gaps.iter().copied().map(draw).collect();
    (frag_lens, gap_ms)
}

// ── handshake ──────────────────────────────────────────────────────────────

/// Run the client handshake over the secured stream and return the encrypted
/// tunnel (xray `ClientInstance.Handshake`). On success the VLESS request
/// header is written through the returned connection.
pub async fn handshake(
    stream: BoxStream,
    cfg: &EncryptionConfig,
) -> Result<CommonConn, NativeError> {
    let mut stream = stream;
    let rng = ring::rand::SystemRandom::new();

    // Relay blocks: one per server key, chained by hash32 + CTR streams.
    let relays_len: usize = cfg
        .keys
        .iter()
        .map(|k| match k {
            ServerKey::X25519(_) => 32 + 32,
            ServerKey::Mlkem(_) => MLKEM_CT_LEN + 32,
        })
        .sum::<usize>()
        - 32;

    // Padding plan (drawn before the buffer so its length is known).
    let (mut frag_lens, gap_ms) = create_padding(&cfg.padding_lens, &cfg.padding_gaps, &rng);
    let padding_len: usize = frag_lens.iter().sum();

    let mut hello = vec![0u8; IV_LEN + relays_len + PFS_EXCHANGE_LEN + padding_len];
    rng.fill(&mut hello[..IV_LEN])
        .map_err(|_| NativeError::Config("rng failure".into()))?;
    let iv: [u8; 16] = hello[..IV_LEN].try_into().expect("iv slice");

    let mut nfs_key = Vec::new();
    let mut last_ctr: Option<Ctr128BE<Aes256>> = None;
    let mut off = IV_LEN;
    for (j, key) in cfg.keys.iter().enumerate() {
        let index = match key {
            ServerKey::X25519(pk) => {
                let eph =
                    X25519KeyPair::generate(&rng).map_err(|e| NativeError::Tls(e.to_string()))?;
                hello[off..off + X25519_LEN].copy_from_slice(&eph.public_key());
                nfs_key = eph
                    .agree(pk)
                    .map_err(|e| NativeError::Tls(e.to_string()))?
                    .to_vec();
                X25519_LEN
            }
            ServerKey::Mlkem(ek) => {
                let pk = PublicKey::from_bytes(ek).map_err(|e| NativeError::Tls(e.to_string()))?;
                let (ct, ss) =
                    Mlkem768::encapsulate(&pk).map_err(|e| NativeError::Tls(e.to_string()))?;
                hello[off..off + MLKEM_CT_LEN].copy_from_slice(ct.as_bytes());
                nfs_key = ss.as_bytes().to_vec();
                MLKEM_CT_LEN
            }
        };
        if cfg.mode != MlkemMode::Native {
            // xorpub/random: mask this relay block with a CTR stream keyed
            // by the server's own key material (recoverable by the server).
            new_ctr(key.bytes(), &iv).apply_keystream(&mut hello[off..off + index]);
        }
        if let Some(c) = last_ctr.as_mut() {
            // Make this relay irreplaceable: unmask the previous hop's
            // hash32 check region.
            c.apply_keystream(&mut hello[off..off + 32]);
        }
        if j == cfg.keys.len() - 1 {
            break;
        }
        // Chain to the next key: write blake3(next server key) masked by a
        // CTR stream keyed by THIS hop's shared secret.
        let mut c = new_ctr(&nfs_key, &iv);
        let hash = b3::hash32(cfg.keys[j + 1].bytes());
        hello[off + index..off + index + 32].copy_from_slice(&hash);
        c.apply_keystream(&mut hello[off + index..off + index + 32]);
        last_ctr = Some(c);
        off += index + 32;
    }

    // NFS AEAD over the IV context.
    let mut nfs_aead = WireAead::new(&iv, &nfs_key);

    // Client PFS key exchange: ephemeral ML-KEM-768 + ephemeral X25519.
    let (mlkem_pk, mlkem_dsk) =
        Mlkem768::generate_keypair().map_err(|e| NativeError::Tls(e.to_string()))?;
    let x25519_eph = X25519KeyPair::generate(&rng).map_err(|e| NativeError::Tls(e.to_string()))?;
    let mut pfs_public = Vec::with_capacity(1184 + X25519_LEN);
    pfs_public.extend_from_slice(mlkem_pk.as_bytes());
    pfs_public.extend_from_slice(&x25519_eph.public_key());

    let pfs_off = IV_LEN + relays_len;
    hello[pfs_off..pfs_off + 18]
        .copy_from_slice(&nfs_aead.seal(&encode_length(PFS_EXCHANGE_LEN - 18), &[]));
    hello[pfs_off + 18..pfs_off + PFS_EXCHANGE_LEN]
        .copy_from_slice(&nfs_aead.seal(&pfs_public, &[]));

    // Padding: a sealed [2B len] prefix, then the zero body sealed so the
    // ciphertext (body + tag) fills the remainder exactly (xray seals
    // `padding[18:paddingLength-16]` — zeros at that point — into the tail).
    debug_assert!(padding_len >= 34, "padding first block min is 35");
    let pad_off = pfs_off + PFS_EXCHANGE_LEN;
    hello[pad_off..pad_off + 18]
        .copy_from_slice(&nfs_aead.seal(&encode_length(padding_len - 18), &[]));
    let body = vec![0u8; padding_len - 34];
    let ct = nfs_aead.seal(&body, &[]);
    hello[pad_off + 18..pad_off + padding_len].copy_from_slice(&ct);

    // Fragmented send: the first fragment carries the whole pre-padding
    // prefix (xray folds it into paddingLens[0]); gaps between fragments
    // shape the traffic pattern.
    frag_lens[0] += pfs_off + PFS_EXCHANGE_LEN;
    let mut sent = 0usize;
    for (i, l) in frag_lens.into_iter().enumerate() {
        if l > 0 {
            write_all_timeout(
                &mut stream,
                &hello[sent..sent + l],
                "vless mlkem hello fragment",
            )
            .await?;
            sent += l;
        }
        if gap_ms.len() > i && gap_ms[i] > 0 {
            tokio::time::sleep(Duration::from_millis(gap_ms[i] as u64)).await;
        }
    }

    // ── ServerHello ──
    let encrypted_pfs = read_exact_timeout(
        &mut stream,
        MLKEM_CT_LEN + X25519_LEN + TAG_LEN,
        "vless mlkem server pfs",
    )
    .await?;
    let server_pfs = nfs_aead.open_with(MAX_NONCE, &encrypted_pfs, &[])?;
    let mlkem_ct = Ciphertext::from_bytes(&server_pfs[..MLKEM_CT_LEN])
        .map_err(|e| NativeError::Tls(e.to_string()))?;
    let mlkem_key = Mlkem768::decapsulate(&mlkem_dsk, &mlkem_ct)
        .map_err(|e| NativeError::Tls(e.to_string()))?;
    let mut peer_x25519 = [0u8; X25519_LEN];
    peer_x25519.copy_from_slice(&server_pfs[MLKEM_CT_LEN..MLKEM_CT_LEN + X25519_LEN]);
    let x25519_key = x25519_eph
        .agree(&peer_x25519)
        .map_err(|e| NativeError::Tls(e.to_string()))?;

    // united = pfs(64) || nfs(32); direction-specific AEAD contexts.
    let mut united = Vec::with_capacity(96);
    united.extend_from_slice(mlkem_key.as_bytes());
    united.extend_from_slice(&x25519_key);
    united.extend_from_slice(&nfs_key);

    let self_aead = WireAead::new(&pfs_public, &united);
    let mut peer_aead = WireAead::new(&server_pfs, &united);

    let encrypted_ticket = read_exact_timeout(&mut stream, 32, "vless mlkem ticket").await?;
    // xray opens the ticket IN PLACE and reuses its first 16 bytes
    // (plaintext) as the random-mode inbound CTR IV.
    let ticket_plain = peer_aead.open(&encrypted_ticket, &[])?;
    let encrypted_len = read_exact_timeout(&mut stream, 18, "vless mlkem padding length").await?;
    let length_plain = peer_aead.open(&encrypted_len, &[])?;
    let peer_padding_len = decode_length(&length_plain);

    // random mode: XOR-mask everything past the handshake except record
    // headers. The server's Hello tail (the padding still unread below) was
    // written before its masking layer engaged, so the inbound side skips
    // exactly those bytes (xray `NewXorConn(.., 0, peerPaddingLen)`).
    let (out_xor, in_xor) = if cfg.mode == MlkemMode::Random {
        let ticket_iv: [u8; 16] = ticket_plain[..16].try_into().expect("ticket slice");
        (
            Some(new_ctr(&united, &iv)),
            Some(new_ctr(&united, &ticket_iv)),
        )
    } else {
        (None, None)
    };

    Ok(CommonConn {
        inner: stream,
        aead: self_aead,
        peer_aead,
        peer_padding: Some(vec![0u8; peer_padding_len]),
        pad_pos: 0,
        in_buf: Vec::new(),
        in_pos: 0,
        phase: ReadPhase::Padding,
        header: [0; HEADER_LEN],
        header_pos: 0,
        payload: Vec::new(),
        payload_pos: 0,
        out_pending: Vec::new(),
        out_pos: 0,
        out_xor,
        in_xor,
    })
}

// ── timeout helpers ────────────────────────────────────────────────────────

async fn write_all_timeout(
    stream: &mut BoxStream,
    data: &[u8],
    step: &'static str,
) -> Result<(), NativeError> {
    use tokio::io::AsyncWriteExt;
    let timeout = timeouts::PROTOCOL;
    tokio::time::timeout(timeout, stream.write_all(data))
        .await
        .map_err(|_| NativeError::Timeout {
            step,
            limit: timeout,
        })?
        .map_err(NativeError::from)
}

async fn read_exact_timeout(
    stream: &mut BoxStream,
    len: usize,
    step: &'static str,
) -> Result<Vec<u8>, NativeError> {
    use tokio::io::AsyncReadExt;
    let timeout = timeouts::PROTOCOL;
    let mut buf = vec![0u8; len];
    tokio::time::timeout(timeout, stream.read_exact(&mut buf))
        .await
        .map_err(|_| NativeError::Timeout {
            step,
            limit: timeout,
        })?
        .map_err(NativeError::from)?;
    Ok(buf)
}

// ── post-handshake record connection ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadPhase {
    /// Draining the sealed server-hello padding (raw — written before the
    /// random-mode masking engaged).
    Padding,
    /// Collecting the 5-byte record header (never masked).
    Header,
    /// Collecting the record payload (masked under `random`).
    Payload,
}

/// The encrypted tunnel (xray `CommonConn`).
///
/// Outbound writes are chunked to ≤8192 bytes and sealed as
/// `[23 03 03 len_hi len_lo][AEAD(chunk, aad=header)]` with an
/// auto-incrementing nonce; inbound reads mirror the framing after draining
/// the server padding. `random` mode XOR-masks every byte except the 5-byte
/// record headers (per-direction CTR streams keyed with the united key).
///
/// Deviation from upstream: the `MaxNonce` re-key branch (xray re-derives the
/// AEAD from the record bytes after 2⁹⁶ records) is not implemented — it is
/// unreachable at any realistic traffic volume.
pub struct CommonConn {
    inner: BoxStream,
    aead: WireAead,
    peer_aead: WireAead,
    peer_padding: Option<Vec<u8>>,
    pad_pos: usize,
    in_buf: Vec<u8>,
    in_pos: usize,
    phase: ReadPhase,
    header: [u8; HEADER_LEN],
    header_pos: usize,
    payload: Vec<u8>,
    payload_pos: usize,
    out_pending: Vec<u8>,
    out_pos: usize,
    out_xor: Option<Ctr128BE<Aes256>>,
    in_xor: Option<Ctr128BE<Aes256>>,
}

impl AsyncRead for CommonConn {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;
        let Self {
            inner,
            peer_aead,
            peer_padding,
            pad_pos,
            in_buf,
            in_pos,
            phase,
            header,
            header_pos,
            payload,
            payload_pos,
            in_xor,
            ..
        } = this;
        loop {
            // Leftover decrypted payload first: never lose a filled record.
            if *in_pos < in_buf.len() {
                let n = std::cmp::min(in_buf.len() - *in_pos, buf.remaining());
                let end = *in_pos + n;
                buf.put_slice(&in_buf[*in_pos..end]);
                *in_pos = end;
                return Poll::Ready(Ok(()));
            }
            match phase {
                ReadPhase::Padding => {
                    let Some(mut padding) = peer_padding.take() else {
                        *phase = ReadPhase::Header;
                        continue;
                    };
                    while *pad_pos < padding.len() {
                        let mut slice = ReadBuf::new(&mut padding[*pad_pos..]);
                        match Pin::new(&mut **inner).poll_read(cx, &mut slice)? {
                            Poll::Ready(()) if slice.filled().is_empty() => {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "vless mlkem: eof in server padding",
                                )));
                            }
                            Poll::Ready(()) => *pad_pos += slice.filled().len(),
                            Poll::Pending => {
                                *peer_padding = Some(padding);
                                return Poll::Pending;
                            }
                        }
                    }
                    peer_aead.open(&padding, &[])?;
                    *pad_pos = 0;
                    *header_pos = 0;
                    *phase = ReadPhase::Header;
                }
                ReadPhase::Header => {
                    while *header_pos < HEADER_LEN {
                        let mut byte = [0u8; 1];
                        let mut slice = ReadBuf::new(&mut byte);
                        match Pin::new(&mut **inner).poll_read(cx, &mut slice)? {
                            Poll::Ready(()) if slice.filled().is_empty() => {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "vless mlkem: eof in record header",
                                )));
                            }
                            Poll::Ready(()) => {
                                header[*header_pos] = byte[0];
                                *header_pos += 1;
                            }
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    let l = decode_length(&header[3..]);
                    if header[0] != 23
                        || header[1] != 3
                        || header[2] != 3
                        || !(MIN_RECORD..=MAX_RECORD).contains(&l)
                    {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "vless mlkem: invalid record header",
                        )));
                    }
                    *payload = vec![0u8; l];
                    *payload_pos = 0;
                    *phase = ReadPhase::Payload;
                }
                ReadPhase::Payload => {
                    while *payload_pos < payload.len() {
                        let mut slice = ReadBuf::new(&mut payload[*payload_pos..]);
                        match Pin::new(&mut **inner).poll_read(cx, &mut slice)? {
                            Poll::Ready(()) if slice.filled().is_empty() => {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "vless mlkem: eof in record payload",
                                )));
                            }
                            Poll::Ready(()) => *payload_pos += slice.filled().len(),
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                    if let Some(ctr) = in_xor {
                        ctr.apply_keystream(payload);
                    }
                    let taken = std::mem::take(payload);
                    *in_buf = peer_aead.open(&taken, header)?;
                    *in_pos = 0;
                    *header_pos = 0;
                    *phase = ReadPhase::Header;
                }
            }
        }
    }
}

impl AsyncWrite for CommonConn {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = &mut *self;
        let Self {
            inner,
            aead,
            out_pending,
            out_pos,
            out_xor,
            ..
        } = this;
        let mut accepted = 0usize;
        loop {
            // Drive any parked record first; its bytes are this buf's prefix.
            while *out_pos < out_pending.len() {
                match Pin::new(&mut **inner).poll_write(cx, &out_pending[*out_pos..]) {
                    Poll::Ready(Ok(n)) => *out_pos += n,
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => {
                        return if accepted > 0 {
                            Poll::Ready(Ok(accepted))
                        } else {
                            Poll::Pending
                        };
                    }
                }
            }
            out_pending.clear();
            *out_pos = 0;
            if accepted >= buf.len() {
                return Poll::Ready(Ok(accepted));
            }
            // Seal the next ≤8192-byte chunk as one record.
            let end = std::cmp::min(buf.len(), accepted + MAX_CHUNK);
            let chunk = &buf[accepted..end];
            let mut record = Vec::with_capacity(HEADER_LEN + chunk.len() + TAG_LEN);
            let hdr = encode_length_header(chunk.len() + TAG_LEN);
            record.extend_from_slice(&hdr);
            record.extend_from_slice(&aead.seal(chunk, &hdr));
            if let Some(ctr) = out_xor {
                ctr.apply_keystream(&mut record[HEADER_LEN..]);
            }
            *out_pending = record;
            accepted = end;
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let Self {
            inner,
            out_pending,
            out_pos,
            ..
        } = &mut *self;
        while *out_pos < out_pending.len() {
            match Pin::new(&mut **inner).poll_write(cx, &out_pending[*out_pos..]) {
                Poll::Ready(Ok(n)) => *out_pos += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        out_pending.clear();
        *out_pos = 0;
        Pin::new(&mut **inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let Self {
            inner,
            out_pending,
            out_pos,
            ..
        } = &mut *self;
        while *out_pos < out_pending.len() {
            match Pin::new(&mut **inner).poll_write(cx, &out_pending[*out_pos..]) {
                Poll::Ready(Ok(n)) => *out_pos += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        out_pending.clear();
        *out_pos = 0;
        Pin::new(&mut **inner).poll_shutdown(cx)
    }
}

/// Record header: `23 03 03` + big-endian u16 length (xray `EncodeHeader`).
const fn encode_length_header(l: usize) -> [u8; HEADER_LEN] {
    let [hi, lo] = encode_length(l);
    [23, 3, 3, hi, lo]
}

#[cfg(test)]
mod tests {
    use super::b3;
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

    /// An independent server-side implementation of xray
    /// `ServerInstance.Handshake` + the record layer (server.go semantics,
    /// NOT a `CommonConn` twin — the framing is re-derived from the Go
    /// source so symmetric bugs surface). Echoes every received record
    /// back until the client disconnects.
    async fn fake_server(
        mut conn: DuplexStream,
        keys: Vec<ServerKey>,
        x25519_sk: std::sync::Arc<X25519KeyPair>,
        mlkem_sk: xray_tui_tls::crypto::mlkem::SecretKey,
        mode: MlkemMode,
    ) -> std::io::Result<()> {
        let relays_len: usize = keys
            .iter()
            .map(|k| match k {
                ServerKey::X25519(_) => 64,
                ServerKey::Mlkem(_) => MLKEM_CT_LEN + 32,
            })
            .sum::<usize>()
            - 32;

        // ── ClientHello: iv + relays ──
        let mut buf = vec![0u8; IV_LEN + relays_len];
        conn.read_exact(&mut buf).await?;
        let iv: [u8; IV_LEN] = buf[..IV_LEN].try_into().expect("iv");
        let mut relays = buf[IV_LEN..].to_vec();
        let mut nfs_key = Vec::new();
        let mut last_ctr: Option<Ctr128BE<Aes256>> = None;
        let mut off = 0usize;
        for (j, key) in keys.iter().enumerate() {
            if let Some(c) = &mut last_ctr {
                c.apply_keystream(&mut relays[off..off + 32]); // recover this relay
            }
            let index = match key {
                ServerKey::X25519(_) => X25519_LEN,
                ServerKey::Mlkem(_) => MLKEM_CT_LEN,
            };
            if mode != MlkemMode::Native {
                new_ctr(key.bytes(), &iv).apply_keystream(&mut relays[off..off + index]);
            }
            match key {
                ServerKey::X25519(_) => {
                    let mut peer = [0u8; X25519_LEN];
                    peer.copy_from_slice(&relays[off..off + X25519_LEN]);
                    nfs_key = x25519_sk.agree(&peer).expect("agree").to_vec();
                }
                ServerKey::Mlkem(_) => {
                    let ct = xray_tui_tls::crypto::mlkem::Ciphertext::from_bytes(
                        &relays[off..off + MLKEM_CT_LEN],
                    )
                    .expect("ct");
                    nfs_key = Mlkem768::decapsulate(&mlkem_sk, &ct)
                        .expect("decapsulate")
                        .as_bytes()
                        .to_vec();
                }
            }
            if j == keys.len() - 1 {
                break;
            }
            // hash32 chain check (server.go: unexpected hash32 → error). The
            // SAME CTR instance continues into the next hop's unmask.
            let mut c = new_ctr(&nfs_key, &iv);
            let mut got = relays[off + index..off + index + 32].to_vec();
            c.apply_keystream(&mut got);
            assert_eq!(got, b3::hash32(keys[j + 1].bytes()), "hash32 chain");
            last_ctr = Some(c);
            off += index + 32;
        }

        let mut nfs_aead = WireAead::new(&iv, &nfs_key);

        // ── sealed pfs key-exchange length + body ──
        let mut enc_len = [0u8; 18];
        conn.read_exact(&mut enc_len).await?;
        let length = decode_length(&nfs_aead.open(&enc_len, &[])?);
        let mut enc_pfs = vec![0u8; length];
        conn.read_exact(&mut enc_pfs).await?;
        let client_pfs_public = nfs_aead.open(&enc_pfs, &[])?;

        // ── server pfs: encapsulate + ephemeral X25519 ──
        let client_mlkem_pk =
            xray_tui_tls::crypto::mlkem::PublicKey::from_bytes(&client_pfs_public[..1184])
                .expect("pk");
        let (server_ct, server_ss) = Mlkem768::encapsulate(&client_mlkem_pk).expect("encapsulate");
        let server_x = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).expect("x");
        let mut peer = [0u8; X25519_LEN];
        peer.copy_from_slice(&client_pfs_public[1184..]);
        let x_key = server_x.agree(&peer).expect("agree");
        let mut united = Vec::with_capacity(96);
        united.extend_from_slice(server_ss.as_bytes());
        united.extend_from_slice(&x_key);
        united.extend_from_slice(&nfs_key);

        let mut server_pfs_public = Vec::with_capacity(MLKEM_CT_LEN + X25519_LEN);
        server_pfs_public.extend_from_slice(server_ct.as_bytes());
        server_pfs_public.extend_from_slice(&server_x.public_key());

        let mut self_aead = WireAead::new(&server_pfs_public, &united);
        let mut peer_aead = WireAead::new(&client_pfs_public, &united);

        // ── ServerHello: [MaxNonce pfs 1136][ticket 32][padlen 18][padding] ──
        let mut ticket_plain = [0u8; 16];
        ring::rand::SystemRandom::new()
            .fill(&mut ticket_plain)
            .expect("rng");
        ticket_plain[..2].copy_from_slice(&encode_length(0)); // seconds = 0

        let mut hello = Vec::new();
        hello.extend(nfs_aead.seal_with(MAX_NONCE, &server_pfs_public, &[]));
        hello.extend(self_aead.seal(&ticket_plain, &[]));
        let pad_len = 40usize;
        hello.extend(self_aead.seal(&encode_length(pad_len - 18), &[]));
        hello.extend(self_aead.seal(&vec![0u8; pad_len - 34], &[]));
        conn.write_all(&hello).await?;

        // ── client padding (nfs-sealed) ──
        let mut enc_len = [0u8; 18];
        conn.read_exact(&mut enc_len).await?;
        // The decoded length is the body CIPHERTEXT length (plaintext + tag).
        let ct_len = decode_length(&nfs_aead.open(&enc_len, &[])?);
        let mut enc_pad = vec![0u8; ct_len];
        conn.read_exact(&mut enc_pad).await?;
        nfs_aead.open(&enc_pad, &[])?;

        // ── record echo loop (random mode masks everything but headers) ──
        let mut out_ctr = (mode == MlkemMode::Random).then(|| new_ctr(&united, &ticket_plain));
        let mut in_ctr = (mode == MlkemMode::Random).then(|| new_ctr(&united, &iv));
        loop {
            let mut hdr = [0u8; HEADER_LEN];
            if let Err(e) = conn.read_exact(&mut hdr).await {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    return Ok(());
                }
                return Err(e);
            }
            let l = decode_length(&hdr[3..]);
            let mut payload = vec![0u8; l];
            conn.read_exact(&mut payload).await?;
            if let Some(c) = &mut in_ctr {
                c.apply_keystream(&mut payload);
            }
            let plain = peer_aead.open(&payload, &hdr)?;
            let mut record = Vec::with_capacity(HEADER_LEN + l);
            record.extend_from_slice(&hdr);
            record.extend_from_slice(&self_aead.seal(&plain, &hdr));
            if let Some(c) = &mut out_ctr {
                c.apply_keystream(&mut record[HEADER_LEN..]);
            }
            conn.write_all(&record).await?;
        }
    }

    fn test_keys() -> (
        Vec<ServerKey>,
        std::sync::Arc<X25519KeyPair>,
        xray_tui_tls::crypto::mlkem::SecretKey,
    ) {
        let x = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).expect("x25519");
        let (pk, sk) = Mlkem768::generate_keypair().expect("mlkem");
        (
            vec![
                ServerKey::X25519(x.public_key()),
                ServerKey::Mlkem(pk.as_bytes().to_vec()),
            ],
            std::sync::Arc::new(x),
            sk,
        )
    }

    async fn roundtrip(mode: MlkemMode, padding: &str, payload_len: usize) {
        let (keys, x_sk, mlkem_sk) = test_keys();
        let cfg = EncryptionConfig {
            mode,
            seconds: 0,
            padding_lens: parse_padding(padding).expect("padding").0,
            padding_gaps: parse_padding(padding).expect("padding").1,
            keys: keys.clone(),
        };
        let (client_side, server_side) = tokio::io::duplex(1 << 16);
        let server = tokio::spawn(async move {
            if let Err(e) = fake_server(
                server_side,
                keys.clone(),
                std::sync::Arc::clone(&x_sk),
                mlkem_sk,
                mode,
            )
            .await
            {
                eprintln!("FAKE SERVER ERR: {e}");
            }
        });

        let mut conn = handshake(Box::new(client_side), &cfg)
            .await
            .expect("handshake");

        // Exercise the record layer: a payload larger than MAX_CHUNK forces
        // multi-record writes; read it back through the echo server.
        let payload: Vec<u8> = (0..payload_len)
            .map(|i| u8::try_from(i % 251).expect("in range"))
            .collect();
        conn.write_all(&payload).await.expect("write");
        conn.flush().await.expect("flush");

        let mut got = Vec::new();
        let mut chunk = vec![0u8; 4096];
        while got.len() < payload.len() {
            let n = tokio::time::timeout(std::time::Duration::from_secs(5), conn.read(&mut chunk))
                .await
                .expect("read timeout")
                .expect("read");
            assert!(n > 0);
            got.extend_from_slice(&chunk[..n]);
        }
        assert_eq!(got, payload, "roundtrip payload");

        drop(conn);
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn roundtrip_native_default_padding() {
        roundtrip(MlkemMode::Native, "", 10_000).await;
    }

    #[tokio::test]
    async fn roundtrip_xorpub_custom_padding() {
        roundtrip(MlkemMode::XorPub, "100-35-70.0-0-0.50-100-200.0-0-0", 5_000).await;
    }

    #[tokio::test]
    async fn roundtrip_random_default_padding() {
        roundtrip(MlkemMode::Random, "", 17_345).await;
    }

    #[tokio::test]
    async fn roundtrip_random_custom_padding() {
        roundtrip(MlkemMode::Random, "100-35-111.0-0-0", 100).await;
    }

    /// The config builder decodes base64url keys and rejects bad segments.
    #[test]
    fn config_from_parsed_string() {
        use base64::Engine as _;
        let x = X25519KeyPair::generate(&ring::rand::SystemRandom::new()).expect("x");
        let key_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(x.public_key());
        let value = format!("mlkem768x25519plus.native.1rtt.100-35-70.0-0-0.{key_b64}");
        let parsed = xray_tui_proto::proto_spec::parse_mlkem_encryption(&value)
            .expect("parse")
            .expect("is mlkem");
        let cfg = EncryptionConfig::try_from_parsed(&parsed).expect("config");
        assert_eq!(cfg.mode, MlkemMode::Native);
        assert_eq!(cfg.seconds, 0); // 1rtt → xray Seconds=0 (no 0-RTT resume)
        assert_eq!(cfg.keys.len(), 1);
        assert_eq!(cfg.keys[0].bytes(), x.public_key().as_slice());

        // A malformed key segment is rejected at parse time.
        let bad = format!("mlkem768x25519plus.native.0rtt.{key_b64}!!!");
        assert!(xray_tui_proto::proto_spec::parse_mlkem_encryption(&bad).is_err());
    }

    /// Padding validation mirrors xray `ParsePadding` bounds.
    #[test]
    fn padding_validation() {
        assert!(parse_padding("100-35-70.0-0-0").is_ok());
        assert!(parse_padding("50-35-70").is_err()); // first probability < 100
        assert!(parse_padding("100-34-70").is_err()); // first min < 35
        assert!(parse_padding("100-35-70.0-0").is_err()); // missing slot
        let big = "100-35-65535.".repeat(3);
        assert!(parse_padding(big.trim_end_matches('.')).is_err()); // total > 65553
    }
}
