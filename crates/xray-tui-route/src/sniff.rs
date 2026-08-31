//! Payload sniffer: TLS `ClientHello` SNI + HTTP `Host` + QUIC `Initial`
//! `ClientHello` SNI over a bounded leading slice.
//!
//! `probe` returns `None` for anything indeterminate — garbage, truncated
//! wire data, an HTTP response, an unsupported QUIC version, or an oversize
//! slice — and never panics on malformed input.

use aes::cipher::{BlockCipherEncrypt, KeyInit};

/// Protocol identified by the sniffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffedProtocol {
    /// TLS handshake (`ClientHello`).
    Tls,
    /// Plain HTTP request.
    Http,
    /// QUIC `Initial` packet (TLS `ClientHello` inside a `CRYPTO` frame).
    Quic,
}

impl From<SniffedProtocol> for crate::ir::SniffedProtocol {
    fn from(p: SniffedProtocol) -> Self {
        match p {
            SniffedProtocol::Tls => Self::Tls,
            SniffedProtocol::Http => Self::Http,
            SniffedProtocol::Quic => Self::Quic,
        }
    }
}

/// Sniff outcome: protocol plus the host carried on the wire, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffResult {
    pub protocol: SniffedProtocol,
    pub host: Option<String>,
}

/// Hard cap on the sniffed prefix: anything longer is refused up front.
const MAX_SNIFF_LEN: usize = 64 * 1024;

/// Hard cap on the reassembled CRYPTO stream (`ClientHello`) from QUIC
/// `Initial` packets.  Matches xray's `ErrShortBuffer` gate at 32767
/// (`sniff.go:227`).  The `ClientHello` in practice is <3 KiB, so this is
/// generous.
const MAX_CRYPTO_LEN: usize = 32767;

/// Cap on `Initial` packets walked per `probe_quic` call: the key schedule
/// is derived per packet from its DCID (identical across a datagram), so a
/// hostile multi-thousand-packet prefix must not amortize unbounded
/// AES/HKDF work and per-packet allocations into the decision hot path.
const MAX_INITIALS: usize = 8;

/// Sniffs a leading payload slice for a TLS `ClientHello` SNI, an HTTP
/// `Host` header, or a QUIC `Initial` packet's `ClientHello` SNI.
///
/// `None` = indeterminate (garbage / truncated / oversize / HTTP response /
/// unsupported QUIC version); never panics on malformed wire data.
#[must_use]
pub fn probe(bytes: &[u8]) -> Option<SniffResult> {
    if bytes.len() > MAX_SNIFF_LEN {
        return None;
    }
    match bytes.first() {
        Some(&0x16) => probe_tls(bytes),
        // QUIC long header: fixed bit + long-header form (0xC0), packet
        // subtype validated in `probe_quic`.
        Some(&b) if b & 0xC0 == 0xC0 => probe_quic(bytes),
        Some(b) if b.is_ascii_alphabetic() => probe_http(bytes),
        _ => None,
    }
}

/// TLS arm: record walk with every length validated against the remaining
/// slice before advancing.
fn probe_tls(bytes: &[u8]) -> Option<SniffResult> {
    // Record header (5 bytes): content type 0x16, version u16 >= 0x0301,
    // record length u16 (validated, not used to bound — the slice bounds).
    if bytes.len() < 5 || u16::from_be_bytes([bytes[1], bytes[2]]) < 0x0301 {
        return None;
    }
    // Handshake header (4 bytes, after the record header): type 0x01
    // (ClientHello) + 3-byte big-endian length; all present before reading.
    if bytes.len() < 9 || bytes[5] != 0x01 {
        return None;
    }
    let hs_len = u32::from_be_bytes([0, bytes[6], bytes[7], bytes[8]]);
    let hs_end = 9usize.checked_add(hs_len as usize)?;
    if hs_end > bytes.len() {
        // Truncated handshake: refuse rather than walk a partial body.
        return None;
    }
    let host = match sni_from_hello_body(&bytes[9..hs_end]) {
        HelloSni::Host(h) => Some(h),
        HelloSni::NoSni => None,
        HelloSni::Malformed => return None,
    };
    Some(SniffResult {
        protocol: SniffedProtocol::Tls,
        host,
    })
}

/// HTTP arm: request-line must start with a method token (alphabetic),
/// response lines (`HTTP/`) are not requests, and a `host:` header is
/// matched case-insensitively with the value trimmed.
fn probe_http(bytes: &[u8]) -> Option<SniffResult> {
    // Find end of request line.
    let line_end = bytes.iter().position(|&b| b == b'\n')?;
    let line = &bytes[..line_end];
    // Response line starts "HTTP/" — not a request.
    if line.len() >= 5 && line[..5].eq_ignore_ascii_case(b"HTTP/") {
        return None;
    }
    // Method token: alphabetic chars then a space.
    let sp = line.iter().position(|&b| b == b' ')?;
    if sp == 0 || !line[..sp].iter().all(u8::is_ascii_alphabetic) {
        return None;
    }
    // Full header block must be present within the slice.
    let _ = bytes.windows(4).position(|w| w == b"\r\n\r\n")?;

    // Scan header lines for `host:` (case-insensitive name match). The
    // scan stops at the blank line that ends the header block: lines in
    // the request BODY are never searched, so a body line like
    // "host: attacker.com" cannot be mistaken for a header.
    let mut rest = &bytes[line_end + 1..];
    while !rest.is_empty() {
        let Some(nl) = rest.iter().position(|&b| b == b'\n') else {
            break;
        };
        let line = rest[..nl].strip_suffix(b"\r").unwrap_or(&rest[..nl]);
        rest = &rest[nl + 1..];
        if line.is_empty() {
            // Blank line: end of headers; host not found in the block.
            break;
        }
        let Some(colon) = line.iter().position(|&b| b == b':') else {
            continue;
        };
        if !line[..colon].eq_ignore_ascii_case(b"host") {
            continue;
        }
        // Name matched: trim OWS, return value as-is (case preserved).
        let value = std::str::from_utf8(&line[colon + 1..]).ok()?;
        let host = value.trim();
        if host.is_empty() {
            return None;
        }
        return Some(SniffResult {
            protocol: SniffedProtocol::Http,
            host: Some(host.to_owned()),
        });
    }
    None
}

/// Outcome of walking a `ClientHello` body for its SNI.
#[derive(Debug, Clone, PartialEq, Eq)]
enum HelloSni {
    /// Hello is complete; the `server_name` extension was absent.
    NoSni,
    /// Hello is complete; the SNI host (when the extension was present).
    Host(String),
    /// Body is malformed or truncated — the caller treats this as
    /// indeterminate and (for QUIC) keeps accumulating `CRYPTO` data.
    Malformed,
}

/// Shared `ClientHello` body walk: from `legacy_version` onward, over a
/// body already bounded by a validated handshake length.
///
/// Returns [`HelloSni::Host`] when the SNI extension is present,
/// [`HelloSni::NoSni`] when the hello is complete without one, and
/// [`HelloSni::Malformed`] for a truncated/malformed body.
fn sni_from_hello_body(body: &[u8]) -> HelloSni {
    let r = &mut Reader::new(body);
    // legacy_version (2) + random (32); a short body is malformed.
    if r.take(2).is_none() || r.take(32).is_none() {
        return HelloSni::Malformed;
    }
    // session_id: u8 length prefix (skipped, not needed).
    if r.take_len8().is_none() {
        return HelloSni::Malformed;
    }
    // cipher_suites: u16 length prefix.
    if r.take_len16().is_none() {
        return HelloSni::Malformed;
    }
    // compression_methods: u8 length prefix.
    if r.take_len8().is_none() {
        return HelloSni::Malformed;
    }
    // Extensions block: u16 length prefix; absent (empty) is valid.
    let Some(exts) = r.take_len16() else {
        return HelloSni::Malformed;
    };

    // Iterate (type u16, len u16) pairs until the server_name (0x0000).
    let mut e = Reader::new(exts);
    while e.remaining() > 0 {
        let (Some(ty), Some(val)) = (e.take_u16(), e.take_len16()) else {
            return HelloSni::Malformed;
        };
        if ty == 0x0000 {
            // server_name_list: u16 list length, then one entry —
            // name_type 0x00 (host_name), u16 host length, host bytes.
            let mut s = Reader::new(val);
            let Some(list_len) = s.take_u16() else {
                return HelloSni::Malformed;
            };
            if usize::from(list_len) + 2 > val.len() {
                return HelloSni::Malformed;
            }
            let Some(name_type) = s.take(1) else {
                return HelloSni::Malformed;
            };
            if name_type[0] != 0x00 {
                return HelloSni::Malformed;
            }
            let Some(host) = s.take_len16() else {
                return HelloSni::Malformed;
            };
            // SNI MUST be ASCII (RFC 6066); reject anything else.
            if host.is_empty() || host.contains(&0x00) || !host.iter().all(|&b| b.is_ascii()) {
                return HelloSni::Malformed;
            }
            return String::from_utf8(host.to_vec()).map_or(HelloSni::Malformed, HelloSni::Host);
        }
    }
    HelloSni::NoSni
}

/// QUIC arm: walk the long-header packets coalesced in the slice (RFC 9000
/// §12.2), skip non-Initial packets, and for each `Initial` packet derive
/// the RFC 9001 §5.2 initial keys from the destination connection ID,
/// remove header protection (RFC 9001 §5.4.2), decrypt the payload (RFC
/// 9001 §5.3), and walk its frames collecting `CRYPTO` data — the TLS
/// `ClientHello`. Once the hello is complete, read the SNI.
///
/// `None` = indeterminate (not QUIC, non-Initial only, unsupported version,
/// truncated, or the hello split across datagrams we don't hold).
fn probe_quic(bytes: &[u8]) -> Option<SniffResult> {
    let mut buf = bytes;
    // Reassembled CRYPTO stream (the ClientHello) across frames/packets.
    let mut crypto: Vec<u8> = Vec::new();
    let mut initials = 0usize;
    while !buf.is_empty() {
        // Zero padding after the final packet (RFC 9000 §12.2) ends the
        // walk; any other trailing byte is not QUIC.
        if buf[0] == 0 {
            break;
        }
        let first = *buf.first()?;
        if initials >= MAX_INITIALS {
            return None;
        }
        initials += 1;
        // Long-header form with the fixed bit set (RFC 9000 §17.2).
        if first & 0xC0 != 0xC0 {
            return None;
        }
        if buf.len() < 5 {
            return None;
        }
        let version = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let salt: &[u8] = match version {
            // QUIC v1 (RFC 9000).
            0x0000_0001 => &QUIC_SALT_V1,
            // draft-29 initial salt.
            0xff00_001d => &QUIC_SALT_DRAFT29,
            _ => return None,
        };
        // Packet type: bits 5-4 of the first byte; 0 = Initial.
        let packet_type = (first >> 4) & 0x03;
        let mut p = &buf[5..];
        // DCID: u8 length + bytes.
        let dcid_len = usize::from(*p.first()?);
        let dcid = p.get(1..1 + dcid_len)?;
        p = &p[1 + dcid_len..];
        // SCID: u8 length + bytes.
        let scid_len = usize::from(*p.first()?);
        p = p.get(1 + scid_len..)?;
        // Initial-only token: varint length + bytes.
        if packet_type == 0 {
            let token_len = usize::try_from(read_varint(&mut p)?).ok()?;
            p = p.get(token_len..)?;
        }
        // Packet length (RFC 9000 §17.2.3): varint, covers pn + payload.
        let packet_len = usize::try_from(read_varint(&mut p)?).ok()?;
        // An Initial packet carries ≥1 pn byte + a 16-byte AEAD tag, so a
        // shorter length cannot decrypt (the sample/tag checks below are the
        // real floor; this is the earliest cheap rejection).
        if packet_len < 17 {
            return None;
        }
        let pn_off = buf.len() - p.len(); // header length up to the pn field
        let ext_len = pn_off + packet_len; // full packet (pn + payload)
        if ext_len > buf.len() {
            return None; // truncated packet in the datagram
        }
        let rest = &buf[ext_len..];

        if packet_type != 0 {
            // 0-RTT / Handshake / Retry: no ClientHello CRYPTO here; keep
            // walking the coalesced packets.
            buf = rest;
            continue;
        }

        let mut pkt = buf[..ext_len].to_vec();

        // --- Key schedule (RFC 9001 §5.2) ---
        // initial_secret = HKDF-Extract(salt, client_dcid)
        // secret = HKDF-Expand-Label(initial_secret, "client in", "", 32)
        // key/iv/hp = HKDF-Expand-Label(secret, "quic key"/"quic iv"/"quic hp", …, len)
        let initial_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, salt);
        let initial_secret = ring::hmac::sign(&initial_key, dcid);
        let mut client_secret = [0u8; 32];
        hkdf_expand_label(
            initial_secret.as_ref(),
            b"client in",
            b"",
            &mut client_secret,
        )?;
        let mut key_bytes = [0u8; 16];
        hkdf_expand_label(&client_secret, b"quic key", b"", &mut key_bytes)?;
        let mut iv = [0u8; 12];
        hkdf_expand_label(&client_secret, b"quic iv", b"", &mut iv)?;
        let mut hp_key = [0u8; 16];
        hkdf_expand_label(&client_secret, b"quic hp", b"", &mut hp_key)?;

        // --- Header protection removal (RFC 9001 §5.4.2) ---
        // Sample: 16 bytes starting 4 bytes after the packet number field;
        // always inside the payload because pn ≤ 4 bytes. AES-ECB(hp, sample)
        // is the AES-GCM mask (RFC 9001 §5.4.3).
        if pkt.len() < pn_off + 4 + 16 {
            return None;
        }
        let sample: [u8; 16] = pkt[pn_off + 4..pn_off + 4 + 16].try_into().ok()?;
        let mut mask = [0u8; 16];
        let hp = aes::Aes128::new_from_slice(&hp_key).ok()?;
        hp.encrypt_block_b2b((&sample).into(), (&mut mask).into());
        // Long header: the low 4 bits (pn length + 2 reserved) are masked.
        pkt[0] ^= mask[0] & 0x0f;
        let pn_len = usize::from(pkt[0] & 0x03) + 1;
        for (i, b) in pkt[pn_off..pn_off + pn_len].iter_mut().enumerate() {
            *b ^= mask[1 + i];
        }

        // --- Payload decryption (RFC 9001 §5.3) ---
        // nonce = IV XOR (0^pn_len || pn); AAD = full header incl. pn.
        let pn = &pkt[pn_off..pn_off + pn_len];
        let ext_hdr_len = pn_off + pn_len;
        if pkt.len() < ext_hdr_len + 16 {
            return None; // no room for the AEAD tag
        }
        let mut payload = pkt[ext_hdr_len..].to_vec();
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&iv);
        for (i, b) in pn.iter().enumerate() {
            nonce[12 - pn_len + i] ^= b;
        }
        let aead = ring::aead::UnboundKey::new(&ring::aead::AES_128_GCM, &key_bytes).ok()?;
        let aead = ring::aead::LessSafeKey::new(aead);
        let plain = aead
            .open_in_place(
                ring::aead::Nonce::assume_unique_for_key(nonce),
                ring::aead::Aad::from(&pkt[..ext_hdr_len]),
                &mut payload,
            )
            .ok()?;

        // --- Frame walk (RFC 9000 §17.2.2: only the listed frames are
        // permitted in an Initial packet) ---
        let mut f: &[u8] = plain;
        while !f.is_empty() {
            let ft = f[0];
            f = &f[1..];
            match ft {
                // PADDING / PING: no fields.
                0x00 | 0x01 => {}
                0x02 | 0x03 => {
                    // ACK: Largest Acknowledged, ACK Delay, ACK Range Count,
                    // First ACK Range, then per-range Gap + ACK Range Length;
                    // type 0x03 appends ECN Counts.
                    let _ = read_varint(&mut f)?;
                    let _ = read_varint(&mut f)?;
                    let range_count = read_varint(&mut f)?;
                    let _ = read_varint(&mut f)?;
                    for _ in 0..range_count {
                        let _ = read_varint(&mut f)?;
                        let _ = read_varint(&mut f)?;
                    }
                    if ft == 0x03 {
                        let _ = read_varint(&mut f)?;
                        let _ = read_varint(&mut f)?;
                        let _ = read_varint(&mut f)?;
                    }
                }
                0x06 => {
                    // CRYPTO: offset, length, then the data (the ClientHello).
                    // `offset` is an attacker-chosen varint — clamp the
                    // reassembly span so a forged frame cannot force a huge
                    // `resize` (process abort / OOM).  xray applies the same
                    // cap via `io.ErrShortBuffer` (`sniff.go:227`).
                    let offset = usize::try_from(read_varint(&mut f)?).ok()?;
                    let length = usize::try_from(read_varint(&mut f)?).ok()?;
                    let data = f.get(..length)?;
                    f = &f[length..];
                    let end = offset.checked_add(length)?;
                    if end > MAX_CRYPTO_LEN {
                        return None;
                    }
                    if end > crypto.len() {
                        crypto.resize(end, 0);
                    }
                    crypto[offset..end].copy_from_slice(data);
                }
                0x1c => {
                    // CONNECTION_CLOSE: Error Code, Frame Type, Reason Length.
                    let _ = read_varint(&mut f)?;
                    let _ = read_varint(&mut f)?;
                    let reason_len = usize::try_from(read_varint(&mut f)?).ok()?;
                    f = f.get(reason_len..)?;
                }
                _ => return None, // not a valid Initial frame set
            }
        }

        // The ClientHello may span several Initial packets: attempt the
        // parse each time, keep walking if still assembling.
        match hello_sni(&crypto) {
            HelloSni::Host(host) => {
                return Some(SniffResult {
                    protocol: SniffedProtocol::Quic,
                    host: Some(host),
                });
            }
            HelloSni::NoSni => {
                return Some(SniffResult {
                    protocol: SniffedProtocol::Quic,
                    host: None,
                });
            }
            HelloSni::Malformed => {}
        }
        buf = rest;
    }
    None
}

/// Parse a TLS handshake message assembled from `CRYPTO` frames: the first
/// byte must be `ClientHello` (0x01), the 3-byte length bounds the body,
/// and the body is walked for the SNI.
///
/// [`HelloSni::Host`]/[`HelloSni::NoSni`] = the hello is complete;
/// [`HelloSni::Malformed`] = truncated (still assembling) or not a
/// `ClientHello`.
fn hello_sni(handshake: &[u8]) -> HelloSni {
    if handshake.len() < 4 || handshake[0] != 0x01 {
        return HelloSni::Malformed;
    }
    let hs_len = u32::from_be_bytes([0, handshake[1], handshake[2], handshake[3]]) as usize;
    let Some(hs_end) = 4usize.checked_add(hs_len) else {
        return HelloSni::Malformed;
    };
    // Both the outgoing TLS record and the QUIC CRYPTO stream deliver a
    // complete handshake message or nothing — the handshake must exactly
    // fill the available bytes (xray `ReadClientHello` equivalently
    // requires `extensionsLength == len(data)`). A truncated hello is
    // still assembling and returns `Malformed`.
    if hs_end != handshake.len() {
        return HelloSni::Malformed;
    }
    sni_from_hello_body(&handshake[4..hs_end])
}

/// Initial salt for QUIC v1 (RFC 9001 §5.2).
const QUIC_SALT_V1: [u8; 20] = [
    0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c, 0xad,
    0xcc, 0xbb, 0x7f, 0x0a,
];

/// Initial salt for QUIC draft-29 (draft-ietf-quic-tls §5.2, kept as
/// `quicSaltOld` in xray `sniff.go`).
const QUIC_SALT_DRAFT29: [u8; 20] = [
    0xaf, 0xbf, 0xec, 0x28, 0x99, 0x93, 0xd2, 0x4c, 0x9e, 0x97, 0x86, 0xf1, 0x9c, 0x61, 0x11, 0xe0,
    0x43, 0x90, 0xa8, 0x99,
];

/// HKDF-Expand-Label (RFC 8446 §7.1 / RFC 9001 §5.2): HKDF-Expand with the
/// `tls13 `-prefixed label struct. Only the first block is derived — every
/// QUIC initial output (key 16 / iv 12 / hp 16 / client-secret 32) fits the
/// SHA-256 digest length, so `T(1) = HMAC(secret, info || 0x01)` suffices.
fn hkdf_expand_label(secret: &[u8], label: &[u8], context: &[u8], out: &mut [u8]) -> Option<()> {
    // Only the first SHA-256 block is derived: every QUIC initial output
    // (key 16 / iv 12 / hp 16 / client-secret 32) fits the digest length.
    if out.len() > 32 {
        return None;
    }
    let mut info = Vec::with_capacity(2 + 1 + 6 + label.len() + 1 + context.len());
    info.extend_from_slice(&u16::try_from(out.len()).ok()?.to_be_bytes());
    info.push(u8::try_from(6 + label.len()).ok()?);
    info.extend_from_slice(b"tls13 ");
    info.extend_from_slice(label);
    info.push(u8::try_from(context.len()).ok()?);
    info.extend_from_slice(context);
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret);
    let mut ctx = ring::hmac::Context::with_key(&key);
    ctx.update(&info);
    ctx.update(&[1]);
    let tag = ctx.sign();
    out.copy_from_slice(&tag.as_ref()[..out.len()]);
    Some(())
}

/// QUIC variable-length integer (RFC 9000 §16): the two most-significant
/// bits of the first byte encode the width (1/2/4/8); the remainder of the
/// first byte and the following width-1 bytes hold the value.
fn read_varint(b: &mut &[u8]) -> Option<u64> {
    let first = *b.first()?;
    let width = 1usize << (first >> 6);
    if b.len() < width {
        return None;
    }
    let mut v = u64::from(first & 0x3f);
    for byte in &b[1..width] {
        v = (v << 8) | u64::from(*byte);
    }
    *b = &b[width..];
    Some(v)
}

/// Bounds-checked cursor over a wire slice: every take validates the
/// remaining length before advancing.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    const fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    const fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.remaining() < n {
            return None;
        }
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Some(out)
    }

    /// u8-prefixed chunk.
    fn take_len8(&mut self) -> Option<&'a [u8]> {
        let n = usize::from(*self.take(1)?.first()?);
        self.take(n)
    }

    /// u16-prefixed chunk (big-endian).
    fn take_len16(&mut self) -> Option<&'a [u8]> {
        let b = self.take(2)?;
        let n = u16::from_be_bytes([b[0], b[1]]).into();
        self.take(n)
    }

    fn take_u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from_be_bytes([b[0], b[1]]))
    }
}

#[cfg(test)]
mod tests;
