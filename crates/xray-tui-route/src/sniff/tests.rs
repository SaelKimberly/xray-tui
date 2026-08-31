use super::{SniffResult, SniffedProtocol, probe};
use std::path::Path;

/// Deterministic RNG feeding back a fixed byte sequence (same shape as the
/// `xray-tui-tls` hello tests).
struct FixedRandom {
    bytes: Vec<u8>,
}

impl xray_tui_tls::SecureRandom for FixedRandom {
    fn fill(&self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified> {
        dest.iter_mut().enumerate().for_each(|(i, b)| {
            *b = self.bytes[i % self.bytes.len()];
        });
        Ok(())
    }
}

/// Renders the `chrome_130` hand-profile hello to wire bytes via the
/// `xray-tui-tls` public API and persists it as
/// `tests/fixtures/tls_hello_chrome.bin` (commit alongside). Deterministic:
/// fixed RNG material, fixed key share. The file is loaded when present;
/// re-run any test with `RENDER_FIXTURE=1` to force a re-render and
/// overwrite (provenance re-check: output must be byte-identical).
fn fixture_bytes() -> Vec<u8> {
    use xray_tui_tls::hello::{BuildParams, build_hello};

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tls_hello_chrome.bin");
    if std::env::var_os("RENDER_FIXTURE").is_none()
        && let Ok(existing) = std::fs::read(&path)
    {
        return existing;
    }
    // Deterministic inputs: RNG bytes 0x42 (every GREASE draw lands on
    // 0x2A2A; random + session id all 0x42), X25519 key [0xAB; 32].
    let rng = FixedRandom {
        bytes: vec![0x42; 128],
    };
    let spec = xray_tui_tls::profiles::hand_selected::chrome_130();
    let hello = build_hello(
        &spec,
        &BuildParams {
            server_name: "example.com",
            alpn: Some(&["h2", "http/1.1"]),
            x25519_pub: &[0xAB; 32],
            mlkem768_pub: None,
            rng: &rng,
        },
    )
    .expect("chrome_130 spec must render");
    let record = hello.record_bytes;
    match std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|()| std::fs::write(&path, &record))
    {
        Ok(()) => {}
        Err(e) => panic!("write tls fixture: {e}"),
    }
    record
}

#[test]
fn tls_hello_yields_tls_with_sni_target_example_com() {
    let fixture = fixture_bytes();
    // The renderer SNI is "example.com" (see BuildParams above).
    let result = probe(&fixture).expect("chrome hello must sniff as TLS");
    assert_eq!(result.protocol, SniffedProtocol::Tls);
    assert_eq!(result.host.as_deref(), Some("example.com"));
}

#[test]
fn sni_carries_wire_case() {
    // Case preservation from the wire: SNI bytes returned unmodified.
    // Locate the host bytes by walking the hello (not raw searching),
    // overwrite them with mixed case, and expect probe to return exactly
    // the mixed-case string.
    // Host byte offset in the fixture: record(5) + hs hdr(4) + ver(2) +
    // random(32) + sid_len(1) + sid(32) + cs_len(2) + ciphers(32) +
    // comp_len(1) + comp(1) + ext_len(2) = 113; first ext is GREASE
    // (4 hdr + 1 val = 5); SNI ext: ty(2) len(2) list_len(2) + type(1) +
    // host_len(2) = 9 → walk total 128 = host start, len 11 ("example.com").
    const HOST_OFF: usize = 128;
    let mut hello = fixture_bytes();
    let result = probe(&hello).expect("parses");
    assert_eq!(result.host.as_deref(), Some("example.com"));

    let mixed = b"ExAmPlE.COM";
    hello[HOST_OFF..HOST_OFF + mixed.len()].copy_from_slice(mixed);
    let result = probe(&hello).expect("mutated hello still parses");
    assert_eq!(result.host.as_deref(), Some("ExAmPlE.COM"));
}

#[test]
fn http_get_request_yields_http_host_case_insensitive_and_trimmed() {
    let req = b"GET /path HTTP/1.1\r\nUser-Agent: x\r\nHOST:  \tExample.COM:8443  \r\n\r\n";
    let result = probe(req).expect("http request must sniff");
    assert_eq!(result.protocol, SniffedProtocol::Http);
    // Name matched case-insensitively; value trimmed, case preserved.
    assert_eq!(result.host.as_deref(), Some("Example.COM:8443"));
}

#[test]
fn http_response_is_not_a_request() {
    let resp = b"HTTP/1.1 200 OK\r\nHost: example.com\r\n\r\n";
    assert_eq!(probe(resp), None);
}

#[test]
fn garbage_returns_none() {
    assert_eq!(probe(b"\x00\x01\x02\x03garbage"), None);
    assert_eq!(probe(b"\xff\xff\xff\xff"), None);
    assert_eq!(probe(b""), None);
}

#[test]
fn host_header_in_body_is_not_sniffed() {
    // A request with no Host header whose BODY contains a "host:" line:
    // the header scan must stop at the blank line, yielding None —
    // never the attacker-controlled body line.
    let req = b"POST /x HTTP/1.1\r\nContent-Length: 20\r\n\r\nhost: attacker.com\r\n";
    assert_eq!(probe(req), None);
    // Real header + decoy body line: header wins.
    let req = b"POST /x HTTP/1.1\r\nHost: real.example\r\n\r\nhost: attacker.com\r\n";
    let result = probe(req).expect("parses");
    assert_eq!(result.host.as_deref(), Some("real.example"));
}

#[test]
fn truncated_hello_returns_none() {
    let full = fixture_bytes();
    // Every prefix must return None, never panic.
    for cut in 1..full.len() {
        assert_eq!(probe(&full[..cut]), None, "prefix of {cut} bytes sniffed");
    }
}

#[test]
fn hello_field_len_overrun_returns_none() {
    // Valid outer hs_len, but a field length extends past the remaining
    // bytes: the inner Reader takes must yield None (covers the
    // take_len ?=>None arms the outer hs_end gate otherwise hides).
    let mut hello = fixture_bytes();
    // Extensions block length (u16 BE at 112, walked: 9 hdr + 2 ver +
    // 32 random + 1 sid_len + 32 sid + 2 cs_len + 32 ciphers + 1 comp_len
    // + 1 comp): blow it up past the remaining slice.
    hello[112] = 0xFF;
    hello[113] = 0xFF;
    assert_eq!(probe(&hello), None);
    // Same for the session-id length: valid header, sid claims 255 bytes
    // where only 32 remain inside the handshake body.
    let mut hello = fixture_bytes();
    hello[43] = 0xFF;
    assert_eq!(probe(&hello), None);
}

#[test]
fn truncated_http_returns_none() {
    assert_eq!(probe(b"GET / HTTP/1.1\r\n"), None);
    assert_eq!(probe(b"GET / HTTP/1.1\r\nHost: e.com\r\n"), None);
}

#[test]
fn oversized_slice_returns_none_early() {
    let big = vec![0x42u8; 64 * 1024 + 1];
    assert_eq!(probe(&big), None);
    // Exactly 64 KiB is allowed through (then garbage ⇒ None).
    let ok = vec![0x42u8; 64 * 1024];
    assert_eq!(probe(&ok), None);
}

#[test]
fn tls_record_wrong_version_returns_none() {
    let mut hello = fixture_bytes();
    hello[1] = 0x02; // record version 0x0200 < 0x0301
    assert_eq!(probe(&hello), None);
}

#[test]
fn tls_non_client_hello_handshake_type_returns_none() {
    let mut hello = fixture_bytes();
    hello[5] = 0x02; // server hello type, not client hello (byte 5, after record header)
    assert_eq!(probe(&hello), None);
}

#[test]
fn tls_hello_without_sni_yields_tls_with_no_host() {
    let mut hello = vec![
        0x16, 0x03, 0x01, // record: handshake, TLS 1.0+
        0x00, 0x2f, // record length 47
        0x01, 0x00, 0x00, 0x2b, // handshake: client hello, 43 bytes
        0x03, 0x03, // legacy version
    ];
    hello.extend_from_slice(&[0x00; 32]); // random
    hello.push(0x00); // session id len
    hello.extend_from_slice(&0x0002u16.to_be_bytes()); // ciphers len
    hello.extend_from_slice(&0x1301u16.to_be_bytes()); // one cipher
    hello.push(0x01); // compression len
    hello.push(0x00); // null compression
    hello.extend_from_slice(&0x0000u16.to_be_bytes()); // extensions len
    let result = probe(&hello).expect("valid hello");
    assert_eq!(result.protocol, SniffedProtocol::Tls);
    assert_eq!(result.host, None);
}

#[test]
fn sni_with_non_ascii_rejected() {
    // Hand-built hello carrying a non-ASCII server_name — must return
    // None (indeterminate), never panic or emit invalid UTF-8.
    let host = [0xFFu8; 4];
    let mut sni_ext = 0x0000u16.to_be_bytes().to_vec();
    sni_ext.extend_from_slice(&u16::try_from(5 + host.len()).unwrap().to_be_bytes());
    sni_ext.extend_from_slice(&u16::try_from(1 + 2 + host.len()).unwrap().to_be_bytes());
    sni_ext.push(0x00); // host_name
    sni_ext.extend_from_slice(&u16::try_from(host.len()).unwrap().to_be_bytes());
    sni_ext.extend_from_slice(&host);

    let mut body = vec![0x03, 0x03];
    body.extend_from_slice(&[0x00; 32]);
    body.push(0x00); // sid len
    body.extend_from_slice(&0x0002u16.to_be_bytes());
    body.extend_from_slice(&0x1301u16.to_be_bytes());
    body.push(0x01);
    body.push(0x00);
    body.extend_from_slice(&u16::try_from(sni_ext.len()).unwrap().to_be_bytes());
    body.extend_from_slice(&sni_ext);

    let mut hello = vec![0x16, 0x03, 0x01];
    hello.extend_from_slice(&u16::try_from(body.len()).unwrap().to_be_bytes());
    hello.push(0x01);
    let body_len = u32::try_from(body.len()).unwrap();
    hello.extend_from_slice(&body_len.to_be_bytes()[1..]);
    hello.extend_from_slice(&body);
    assert_eq!(probe(&hello), None);
}

#[test]
fn sni_result_shape() {
    // Sanity on the public shape.
    let r = SniffResult {
        protocol: SniffedProtocol::Tls,
        host: None,
    };
    assert_eq!(r.protocol, SniffedProtocol::Tls);
}

#[test]
fn sniff_result_is_debug_clone_eq() {
    let r = probe(b"GET / HTTP/1.1\r\nHost: a.com\r\n\r\n").unwrap();
    let r2 = r.clone();
    assert_eq!(r, r2);
    assert!(format!("{r:?}").contains("Http"));
}

/// Loads one of the real-traffic QUIC `Initial` fixtures (captured from
/// quic-go, ported from xray's `common/protocol/quic/sniff_test.go`).
fn quic_fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap_or_else(|e| panic!("read quic fixture {name}: {e}"))
}

#[test]
fn quic_initial_yields_quic_with_sni_target_www_google_com() {
    // Real quic-go client `Initial` packet carrying a TLS ClientHello with
    // SNI `www.google.com` (xray sniff_test.go fixture).
    let pkt = quic_fixture("quic_initial_google.bin");
    let r = probe(&pkt).expect("real quic packet must sniff as QUIC");
    assert_eq!(r.protocol, SniffedProtocol::Quic);
    assert_eq!(r.host.as_deref(), Some("www.google.com"));
}

#[test]
fn quic_incomplete_hello_returns_none() {
    // The `play.google.com` hello spans two coalesced `Initial` packets; the
    // first packet alone does not carry the whole ClientHello (the SNI host
    // is split mid-name), so it must be indeterminate — never a truncated
    // host.
    let pkt = quic_fixture("quic_initial_play_part1.bin");
    assert_eq!(probe(&pkt), None, "partial hello must not yield a host");
}

#[test]
fn quic_hello_split_across_packets_assembles_sni() {
    // Both coalesced `Initial` packets carry the complete ClientHello; the
    // CRYPTO stream reassembled across them yields the full SNI.
    let pkt = quic_fixture("quic_initial_play_full.bin");
    let r = probe(&pkt).expect("coalesced quic packets must sniff");
    assert_eq!(r.protocol, SniffedProtocol::Quic);
    assert_eq!(r.host.as_deref(), Some("play.google.com"));
}

#[test]
fn quic_garbage_and_short_slices_return_none() {
    // Not a long-header packet: indeterminate.
    assert_eq!(probe(b"\x00\x01\x02\x03"), None);
    assert_eq!(
        probe(b"GET / HTTP/1.1\r\nHost: a\r\n\r\n")
            .unwrap()
            .protocol,
        SniffedProtocol::Http
    );
    // A real packet truncated mid-header is indeterminate, not a panic.
    let pkt = quic_fixture("quic_initial_google.bin");
    assert_eq!(probe(&pkt[..3]), None);
    assert_eq!(probe(&pkt[..8]), None);
    // Unsupported version (e.g. a future QUIC version byte) is indeterminate.
    let mut bad = pkt;
    bad[1..5].copy_from_slice(&0xfeed_faceu32.to_be_bytes());
    assert_eq!(probe(&bad), None);
}

#[test]
fn quic_scid_length_overflow_returns_none_not_panic() {
    // Long-header Initial with an SCID length byte that exceeds the
    // remaining slice.  The unchecked `p[1 + scid_len..]` slice used to
    // panic here (regression: sniff.rs:230); the DCID of the same shape was
    // already bounds-checked.  Must be indeterminate, never a panic.
    let pkt = [0xc3, 0x00, 0x00, 0x00, 0x01, 0x00, 0xff];
    assert_eq!(probe(&pkt), None);
    let pkt = [0xc3, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x11, 0x22];
    assert_eq!(probe(&pkt), None);
}

/// Forges a valid QUIC v1 client `Initial` packet whose single `CRYPTO`
/// frame declares `offset` with 1 data byte (padded + protected + sealed,
/// keys derived from the fixed DCID — same recipe as xray's `sniff_test.go`
/// fixtures).  Lets the test reach the reassembly path with an
/// attacker-chosen offset.
fn forge_initial_with_crypto_offset(offset: u64) -> Vec<u8> {
    use aes::cipher::{BlockCipherEncrypt, KeyInit};

    const SALT: [u8; 20] = [
        0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c,
        0xad, 0xcc, 0xbb, 0x7f, 0x0a,
    ];
    fn expand_label(secret: &[u8], label: &[u8], out: &mut [u8]) {
        let mut info = Vec::new();
        info.extend_from_slice(&u16::try_from(out.len()).expect("small").to_be_bytes());
        info.push(u8::try_from(6 + label.len()).expect("small"));
        info.extend_from_slice(b"tls13 ");
        info.extend_from_slice(label);
        info.push(0);
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret);
        let mut ctx = ring::hmac::Context::with_key(&key);
        ctx.update(&info);
        ctx.update(&[1]);
        let tag = ctx.sign();
        out.copy_from_slice(&tag.as_ref()[..out.len()]);
    }
    fn varint8(v: u64) -> [u8; 8] {
        let mut b = v.to_be_bytes();
        b[0] |= 0xc0;
        b
    }

    let dcid = [0u8; 8];
    let initial_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &SALT);
    let initial_secret = ring::hmac::sign(&initial_key, &dcid);
    let mut cs = [0u8; 32];
    expand_label(initial_secret.as_ref(), b"client in", &mut cs);
    let mut key = [0u8; 16];
    expand_label(&cs, b"quic key", &mut key);
    let mut iv = [0u8; 12];
    expand_label(&cs, b"quic iv", &mut iv);
    let mut hp = [0u8; 16];
    expand_label(&cs, b"quic hp", &mut hp);

    let mut frames = vec![0x06u8]; // CRYPTO
    frames.extend_from_slice(&varint8(offset));
    frames.push(0x01); // length = 1
    frames.push(0x01); // 1 byte of "crypto data"
    frames.resize(frames.len() + 32, 0x00); // PADDING

    let pn_len = 4usize;
    let length = pn_len + frames.len() + 16; // pn + ciphertext + tag
    let mut hdr = vec![0xc3u8]; // long, fixed bit, Initial, 4-byte pn
    hdr.extend_from_slice(&1u32.to_be_bytes());
    hdr.push(8);
    hdr.extend_from_slice(&dcid);
    hdr.push(0); // scid len
    hdr.push(0); // token len varint = 0
    hdr.extend_from_slice(&(u16::try_from(length).expect("small") | 0x4000).to_be_bytes());
    let pn_off = hdr.len();
    hdr.extend_from_slice(&[0u8; 4]); // packet number 0

    let mut payload = frames;
    let aead = ring::aead::LessSafeKey::new(
        ring::aead::UnboundKey::new(&ring::aead::AES_128_GCM, &key).unwrap(),
    );
    aead.seal_in_place_append_tag(
        ring::aead::Nonce::assume_unique_for_key(iv),
        ring::aead::Aad::from(&hdr),
        &mut payload,
    )
    .unwrap();

    let mut pkt = hdr;
    pkt.extend_from_slice(&payload);
    // Header protection.
    let sample: [u8; 16] = pkt[pn_off + 4..pn_off + 20].try_into().unwrap();
    let mut mask = [0u8; 16];
    aes::Aes128::new_from_slice(&hp)
        .unwrap()
        .encrypt_block_b2b((&sample).into(), (&mut mask).into());
    pkt[0] ^= mask[0] & 0x0f;
    for i in 0..pn_len {
        pkt[pn_off + i] ^= mask[1 + i];
    }
    pkt
}

#[test]
fn quic_crypto_offset_is_capped_not_allocated() {
    // A CRYPTO frame with a huge offset used to `crypto.resize(end)` an
    // attacker-chosen size — a 1 TiB allocation aborts the process
    // (regression: sniff.rs:346-350).  The reassembly span must be capped
    // at MAX_CRYPTO_LEN and reported indeterminate.
    for offset in [1u64 << 20, 1u64 << 40, (1u64 << 62) - 1] {
        let pkt = forge_initial_with_crypto_offset(offset);
        assert_eq!(
            probe(&pkt),
            None,
            "offset {offset} must be refused, not allocated"
        );
    }
    // Sanity: the forged packet is well-formed — a small offset decrypts
    // and is accepted into the (incomplete) hello, still yielding None.
    let pkt = forge_initial_with_crypto_offset(64);
    assert_eq!(probe(&pkt), None);
}
