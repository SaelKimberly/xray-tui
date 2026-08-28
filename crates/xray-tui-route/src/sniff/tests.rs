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
