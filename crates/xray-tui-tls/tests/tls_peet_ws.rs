//! Tier-2 verification against tls.peet.ws (network).
//!
//! Ignored by default so `cargo test` needs no network; run with
//! `cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored`.
//!
//! Mirrors `examples/grader.rs`: connects with a real browser profile,
//! speaks HTTP/2 via [`xray_tui_tls::http2`], and asserts the
//! server-reported JA3/JA4. The `local` module is deliberately duplicated
//! from the grader (the brief scopes new files to http2/, the example, and
//! this test; see the grader's module docs for the reconciliation notes).

use std::sync::atomic::AtomicUsize;

use tokio::net::TcpStream;

use xray_tui_tls::fingerprints::{Browser, Fingerprint};
use xray_tui_tls::handshake::{HandshakeParams, connect};
use xray_tui_tls::hello::parse::parse_hello;
use xray_tui_tls::hello::{BuildParams, build_hello};
use xray_tui_tls::http2;
use xray_tui_tls::verify::WebPkiVerifier;

const HOST: &str = "tls.peet.ws";
const PORT: u16 = 443;
const API_PATH: &str = "/api/all";
const ALPN: &[&str] = &["h2", "http/1.1"];

/// Locked expected fingerprints — same constants as the grader.
const CHROME130_JA4: &str = "t13d1516h2_8daaf6152771_f37e75b10bcc";
const FIREFOX128ESR_JA3: &str = "361e0ca6ef1ca4dbe3a1d987722a1980";
const FIREFOX128ESR_JA4: &str = "t13d1314h2_07be0c029dc8_46701d79520f";

/// GREASE-stripped JA3 md5 of the Chrome 130 profile under the fixed seed
/// (all `0x42`); deterministic because stripping removes the seed-dependent
/// GREASE values. This is the value the live server reports for the same
/// profile regardless of its per-connection GREASE.
const CHROME130_JA3_STRIPPED_MD5: &str = "2b916ec56aedf4a5ecbeb5804f60c242";

/// Offline pin of the reconciliation logic + locked constants: builds each
/// profile's hello with a fixed seed and asserts the local JA4-v2 and
/// GREASE-stripped JA3 computations produce exactly the locked values.
/// No network — runs in every `cargo test`.
#[test]
fn local_fingerprints_match_locked_constants() {
    for (name, fingerprint, expected_ja4, expected_ja3) in [
        (
            "chrome_130",
            Fingerprint::new(Browser::Chrome).with_version(130),
            CHROME130_JA4,
            CHROME130_JA3_STRIPPED_MD5,
        ),
        (
            "firefox_128_esr",
            Fingerprint::new(Browser::Firefox).with_version(128),
            FIREFOX128ESR_JA4,
            FIREFOX128ESR_JA3,
        ),
    ] {
        let spec = fingerprint.resolve().expect("identity resolves").spec;
        let fixed = local::FixedRandom {
            bytes: vec![0x42; 128],
            pos: AtomicUsize::new(0),
        };
        let (mlkem_pk, _) =
            xray_tui_tls::crypto::mlkem::Mlkem768::generate_keypair().expect("mlkem keypair");
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: HOST,
                alpn: Some(ALPN),
                x25519_pub: &[0xAB; 32],
                mlkem768_pub: Some(mlkem_pk.as_bytes()),
                rng: &fixed,
            },
        )
        .expect("build hello");
        let parsed = parse_hello(&hello.handshake_bytes).expect("parse hello");

        assert_eq!(
            local::ja4_v2(&parsed),
            expected_ja4,
            "{name} JA4 v2 must match the locked constant"
        );
        let stripped = local::strip_grease(&local::ja3_dash(&parsed));
        assert_eq!(
            local::md5_hex(stripped.as_bytes()),
            expected_ja3,
            "{name} GREASE-stripped JA3 md5 must match the locked value"
        );
    }
}

#[tokio::test]
#[ignore = "network"]
async fn chrome_130_matches_expected_fingerprints() {
    let report = fetch_peet_report(&Fingerprint::new(Browser::Chrome).with_version(130)).await;
    let report = report.expect("live tls.peet.ws fetch for Chrome 130");

    // GREASE-normalized JA4 is deterministic: strict equality against both
    // the locked constant and the local FoxIO-v2 computation.
    assert_eq!(
        report.ja4, CHROME130_JA4,
        "Chrome 130 JA4 must match the locked constant"
    );
    assert_eq!(
        report.local_ja4, report.ja4,
        "local JA4 v2 must match the server-reported JA4 (wire fidelity)"
    );

    // Chrome's JA3 is GREASE-randomized: compare with GREASE stripped (the
    // server strips GREASE before hashing).
    assert_eq!(
        report.ja3_hash, report.local_ja3_stripped_md5,
        "GREASE-stripped JA3 must match (all non-GREASE JA3 fields identical)"
    );
}

#[tokio::test]
#[ignore = "network"]
async fn firefox_128_esr_matches_expected_fingerprints() {
    let report = fetch_peet_report(&Fingerprint::new(Browser::Firefox).with_version(128)).await;
    let report = report.expect("live tls.peet.ws fetch for Firefox 128 ESR");

    // Firefox 128 ESR is GREASE-free: JA3 and JA4 are both stable.
    assert_eq!(
        report.ja3_hash, FIREFOX128ESR_JA3,
        "Firefox JA3 must match the locked constant"
    );
    assert_eq!(
        report.ja4, FIREFOX128ESR_JA4,
        "Firefox JA4 must match the locked constant"
    );
    assert_eq!(
        report.local_ja4, report.ja4,
        "local JA4 v2 must match the server-reported JA4 (wire fidelity)"
    );
    assert_eq!(
        report.ja3_hash, report.local_ja3_stripped_md5,
        "GREASE-stripped JA3 must match"
    );
}

/// One live round-trip: build the profile hello (fixed seed), connect,
/// HTTP/2 GET `/api/all`, and compute both the server's report and the
/// local reconciliation fingerprints.
async fn fetch_peet_report(
    fingerprint: &Fingerprint,
) -> Result<PeetReport, Box<dyn std::error::Error>> {
    let spec = fingerprint.resolve().expect("identity resolves").spec;
    let fixed = local::FixedRandom {
        bytes: vec![0x42; 128],
        pos: AtomicUsize::new(0),
    };
    let (mlkem_pk, _) =
        xray_tui_tls::crypto::mlkem::Mlkem768::generate_keypair().expect("mlkem keypair");
    let local_hello = build_hello(
        &spec,
        &BuildParams {
            server_name: HOST,
            alpn: Some(ALPN),
            x25519_pub: &[0xAB; 32],
            mlkem768_pub: Some(mlkem_pk.as_bytes()),
            rng: &fixed,
        },
    )?;
    let parsed = parse_hello(&local_hello.handshake_bytes)?;

    let stream = TcpStream::connect((HOST, PORT)).await?;
    let verifier = WebPkiVerifier::webpki_roots();
    let mut conn = connect(
        stream,
        HandshakeParams {
            spec: &spec,
            server_name: HOST,
            alpn: Some(ALPN),
            verifier: &verifier,
            rng: &ring::rand::SystemRandom::new(),
        },
    )
    .await?;
    let body = http2::get(&mut conn, API_PATH, HOST).await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;
    let tls = &json["tls"];

    assert_eq!(
        tls["tls_version_negotiated"].as_str(),
        Some("772"),
        "tls_version_negotiated must be 772 (TLS 1.3)"
    );

    let local_ja3 = local::ja3_dash(&parsed);
    Ok(PeetReport {
        ja3_hash: tls["ja3_hash"].as_str().unwrap_or_default().to_string(),
        ja4: tls["ja4"].as_str().unwrap_or_default().to_string(),
        local_ja3_stripped_md5: local::md5_hex(local::strip_grease(&local_ja3).as_bytes()),
        local_ja4: local::ja4_v2(&parsed),
    })
}

struct PeetReport {
    ja3_hash: String,
    ja4: String,
    local_ja3_stripped_md5: String,
    local_ja4: String,
}

/// Reconciliation helpers — duplicated from `examples/grader.rs` (see the
/// grader's module docs for the JA3-separator and JA4-version notes).
mod local {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use md5::{Digest, Md5};
    use ring::digest;

    use xray_tui_tls::SecureRandom;
    use xray_tui_tls::hello::parse::ParsedClientHello;
    use xray_tui_tls::spec::grease::is_grease;

    pub struct FixedRandom {
        pub bytes: Vec<u8>,
        pub pos: AtomicUsize,
    }

    impl SecureRandom for FixedRandom {
        fn fill(&self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified> {
            let mut pos = self.pos.load(Ordering::Relaxed);
            for b in dest.iter_mut() {
                *b = *self.bytes.get(pos).ok_or(ring::error::Unspecified)?;
                pos += 1;
            }
            self.pos.store(pos, Ordering::Relaxed);
            Ok(())
        }
    }

    pub fn md5_hex(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut hasher = Md5::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let mut out = String::with_capacity(32);
        for b in digest {
            let _ = write!(out, "{b:02x}");
        }
        out
    }

    fn sha256_hex12(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(12);
        for b in digest::digest(&digest::SHA256, data)
            .as_ref()
            .iter()
            .take(6)
        {
            let _ = write!(out, "{b:02x}");
        }
        out
    }

    fn u16_list(body: Option<&[u8]>) -> Vec<u16> {
        let Some(body) = body else {
            return Vec::new();
        };
        let Some(rest) = body.get(2..) else {
            return Vec::new();
        };
        let len = usize::from(u16::from_be_bytes([body[0], body[1]]));
        rest[..len.min(rest.len())]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect()
    }

    fn alpn(hello: &ParsedClientHello) -> Option<String> {
        let body = hello.extension(0x0010)?;
        let list_len = usize::from(u16::from_be_bytes([body[0], body[1]]));
        let rest = body.get(2..)?;
        let mut off = 0;
        while off < list_len.min(rest.len()) {
            let len = usize::from(rest[off]);
            off += 1;
            if off + len <= rest.len() {
                return String::from_utf8(rest[off..off + len].to_vec()).ok();
            }
        }
        None
    }

    fn dash_join_decimal(values: &[u16]) -> String {
        values
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join("-")
    }

    fn ja3_canonical(hello: &ParsedClientHello) -> String {
        let ext_types: Vec<u16> = hello.extensions.iter().map(|(t, _)| *t).collect();
        let curves = u16_list(hello.extension(0x000a));
        let point_formats = hello
            .extension(0x000b)
            .map(|body| {
                let len = usize::from(body[0]);
                body[1..(1 + len).min(body.len())].to_vec()
            })
            .unwrap_or_default();
        format!(
            "{},{},{},{},{}",
            hello.legacy_version,
            dash_join_decimal(&hello.cipher_suites),
            dash_join_decimal(&ext_types),
            dash_join_decimal(&curves),
            point_formats
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join("-"),
        )
    }

    /// Removes the GREASE values from a canonical JA3 string — tls.peet.ws
    /// strips GREASE before hashing (its `fingerprint_tls.go` skips GREASE
    /// ciphers/extensions/curves).
    pub fn strip_grease(canonical: &str) -> String {
        canonical
            .split(',')
            .map(|group| {
                group
                    .split('-')
                    .filter(|tok| !tok.parse::<u16>().is_ok_and(is_grease))
                    .collect::<Vec<_>>()
                    .join("-")
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Classic ja3er canonical string (dashes within a group, commas
    /// between groups, decimal ids) of a locally built hello — WITH the
    /// builder's GREASE values (stripped before comparison).
    pub fn ja3_dash(hello: &ParsedClientHello) -> String {
        ja3_canonical(hello)
    }

    /// Current `FoxIO` JA4 (`ja4.py`, FoxIO-LLC/ja4; `ja4.go`, `TrackMe`).
    /// tls.peet.ws's `ja4.go` explicitly skips the padding extension
    /// (0x0015) from the hash list (`hexStr == "0015"`), a documented
    /// deviation from the `FoxIO` `python` implementation which includes it.
    /// The extension *count* in the `_a_` section still includes padding.
    pub fn ja4_v2(hello: &ParsedClientHello) -> String {
        let ciphers: Vec<u16> = hello
            .cipher_suites
            .iter()
            .copied()
            .filter(|&c| !is_grease(c))
            .collect();
        let exts: Vec<u16> = hello
            .extensions
            .iter()
            .map(|(t, _)| *t)
            .filter(|&t| !is_grease(t))
            .collect();
        let sigs: Vec<u16> = u16_list(hello.extension(0x000d))
            .into_iter()
            .filter(|&s| !is_grease(s))
            .collect();

        let cipher_count = ciphers.len().min(99);
        let ext_count = exts.len().min(99);

        let mut sorted_ciphers: Vec<String> = ciphers.iter().map(|c| format!("{c:04x}")).collect();
        sorted_ciphers.sort_unstable();
        let cipher_sha = sha256_hex12(sorted_ciphers.join(",").as_bytes());

        let mut sorted_exts: Vec<String> = exts
            .iter()
            .map(|e| format!("{e:04x}"))
            .filter(|e| e != "0000" && e != "0010" && e != "0015")
            .collect();
        sorted_exts.sort_unstable();
        let sig_str = sigs
            .iter()
            .map(|s| format!("{s:04x}"))
            .collect::<Vec<_>>()
            .join(",");
        let ext_sha = sha256_hex12(format!("{}_{}", sorted_exts.join(","), sig_str).as_bytes());

        let alpn_token = alpn(hello).map_or_else(
            || "00".to_string(),
            |p| {
                let mut chars = p.chars();
                match (chars.next(), chars.last()) {
                    (Some(first), Some(last)) => format!("{first}{last}"),
                    _ => p,
                }
            },
        );

        format!("t13d{cipher_count:02}{ext_count:02}{alpn_token}_{cipher_sha}_{ext_sha}")
    }
}
