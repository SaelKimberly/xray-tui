//! tls.peet.ws grader — tier-2 verification of the fingerprint engine.
//!
//! Connects to `tls.peet.ws` with a browser profile (Chrome 130 or
//! Firefox 128 ESR, selectable via `--profile <name>`), speaks HTTP/2
//! through [`xray_tui_tls::http2`], and compares the server-reported
//! JA3/JA4 against the locked expected values.
//!
//! # JA3/JA4 reconciliation (plan Task 12 step 3)
//!
//! tls.peet.ws is authoritative: it computes JA3/JA4 from the observed
//! `ClientHello`. Its computations differ from this crate's offline
//! fingerprints in two documented ways, so the grader re-implements the
//! server's algorithm locally and asserts the server's report against it:
//!
//! 1. **JA3 separators.** tls.peet.ws emits the classic ja3er form —
//!    decimal ids, dashes *within* a group, commas *between* groups
//!    (e.g. `771,4865-4866-…,0-11-10-…`). The crate's offline
//!    [`ja3_string`](xray_tui_tls::crypto::fingerprint::ja3::ja3_string)
//!    joins everything with commas, so its `md5` differs. The grader
//!    computes the dash form locally (see [`local::ja3_dash`]).
//! 2. **JA4 version.** tls.peet.ws reports the current `FoxIO` JA4 (`t13d` +
//!    SNI flag + non-GREASE cipher/extension counts + 12-hex `SHA-256` of
//!    sorted ciphers / sorted extensions+sigalgs). The crate's offline
//!    [`ja4_a`](xray_tui_tls::crypto::fingerprint::ja4::ja4_a) implements
//!    the original 2023 JA4-A (`t13d` + 4-hex first cipher + `d` + counts),
//!    a different format. The grader re-implements the current `FoxIO`
//!    algorithm (verified byte-for-byte against the reference `ja4.py` and
//!    live curl captures) in [`local::ja4_v2`].
//!
//! # Assertions
//!
//! * Server JA4 == locked constant (GREASE-normalized → deterministic per
//!   profile) AND == locally computed JA4 v2 from the built hello (proves
//!   the wire bytes match what the builder produced and the algorithms
//!   agree). The local v2 replicates tls.peet.ws's exact algorithm,
//!   including its padding-extension (0x0015) exclusion from the hash
//!   list.
//! * Server JA3 hash == md5 of the locally built JA3 canonical with GREASE
//!   values stripped (tls.peet.ws strips GREASE before hashing; Chrome's
//!   GREASE is per-connection random, so this is the stable comparison).
//! * Firefox 128 ESR additionally asserts the locked, stable JA3 constant.

use std::error::Error;
use std::sync::atomic::AtomicUsize;

use tokio::net::TcpStream;

use xray_tui_tls::handshake::{HandshakeParams, connect};
use xray_tui_tls::hello::parse::parse_hello;
use xray_tui_tls::hello::{BuildParams, build_hello};
use xray_tui_tls::http2;
use xray_tui_tls::profiles::BrowserProfile;
use xray_tui_tls::verify::WebPkiVerifier;

const HOST: &str = "tls.peet.ws";
const PORT: u16 = 443;
const API_PATH: &str = "/api/all";
const ALPN: &[&str] = &["h2", "http/1.1"];

/// Locked expected fingerprints (captured live against tls.peet.ws with
/// this engine, 2026-08-11; see the task report).
///
/// Chrome 130's JA3 is GREASE-randomized per connection and is therefore
/// NOT locked — it is asserted GREASE-stripped instead. Note on the JA4
/// constants: tls.peet.ws excludes the padding extension (0x0015) from
/// the JA4-c hash list (its `ja4.go` skips `0015`), while the `FoxIO` spec
/// includes it — so the locked values below differ from the `FoxIO`
/// canonical example (`t13d1516h2_8daaf6152771_e5627efa2ab1`) purely in
/// that padding-exclusion.
const CHROME130_JA4: &str = "t13d1516h2_8daaf6152771_f37e75b10bcc";
/// Firefox 128 ESR is GREASE-free, hence its JA3 is stable and lockable.
const FIREFOX128ESR_JA3: &str = "361e0ca6ef1ca4dbe3a1d987722a1980";
const FIREFOX128ESR_JA4: &str = "t13d1314h2_07be0c029dc8_46701d79520f";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let profiles = parse_args();

    for profile in profiles {
        println!("\n{}", "=".repeat(60));
        println!("Profile : {}", profile.name());
        println!("Protocol: HTTP/2");
        println!("{}", "=".repeat(60));

        if let Err(e) = grade(profile).await {
            eprintln!("GRADE: FAIL — {e}");
            std::process::exit(1);
        }
        println!("GRADE: PASS");
    }

    Ok(())
}

/// Profiles the grader can grade (all others are rejected at the CLI).
const GRADED_PROFILES: &[BrowserProfile] =
    &[BrowserProfile::Chrome130, BrowserProfile::Firefox128Esr];

/// Parses `--profile <name>` (repeatable); defaults to both graded
/// profiles. Only [`GRADED_PROFILES`] are accepted — any other profile
/// name is a CLI error (`grade()` cannot handle it).
fn parse_args() -> Vec<BrowserProfile> {
    let mut args = std::env::args().skip(1);
    let mut selected = Vec::new();
    while let Some(arg) = args.next() {
        if arg != "--profile" {
            eprintln!("usage: grader [--profile <name>]");
            std::process::exit(2);
        }
        let name = args.next().unwrap_or_else(|| {
            eprintln!("--profile requires a value");
            std::process::exit(2);
        });
        let profile = GRADED_PROFILES
            .iter()
            .copied()
            .find(|p| p.name() == name)
            .unwrap_or_else(|| {
                eprintln!(
                    "unknown or unsupported profile {name:?}; grader supports: {}",
                    GRADED_PROFILES
                        .iter()
                        .map(|p| p.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(2);
            });
        selected.push(profile);
    }
    if selected.is_empty() {
        GRADED_PROFILES.to_vec()
    } else {
        selected
    }
}

async fn grade(profile: BrowserProfile) -> Result<(), Box<dyn Error>> {
    // Local fingerprint of the profile (fixed seed → deterministic GREASE;
    // JA4 is GREASE-normalized anyway; JA3 is compared GREASE-stripped).
    let spec = profile.spec();
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
    let local_ja3 = local::ja3_dash(&parsed);
    let local_ja3_stripped_md5 = local::md5_hex(local::strip_grease(&local_ja3).as_bytes());
    let local_ja4 = local::ja4_v2(&parsed);

    // Live connection: TLS 1.3 with the profile's ClientHello, then an
    // HTTP/2 GET against tls.peet.ws.
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
    let tls_version = match tls["tls_version_negotiated"].as_str() {
        Some("771") => "TLS 1.2".to_string(),
        Some("772") => "TLS 1.3".to_string(),
        Some(other) => format!("0x{other}"),
        None => "<missing>".to_string(),
    };
    let cipher = tls["ciphers"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>")
        .to_string();
    let server_ja3 = tls["ja3"].as_str().unwrap_or("<missing>");
    let server_ja3_hash = tls["ja3_hash"].as_str().unwrap_or("<missing>");
    let server_ja4 = tls["ja4"].as_str().unwrap_or("<missing>");
    let server_ja4_r = tls["ja4_r"].as_str().unwrap_or("");
    let akamai = json["http2"]["akamai_fingerprint_hash"]
        .as_str()
        .unwrap_or("");

    println!("TLS version : {tls_version}");
    // `tls.ciphers` is the server's echo of the *offered* cipher list, so
    // the first entry is the first offered cipher (GREASE for Chrome), not
    // the negotiated suite.
    println!("First cipher: {cipher}");
    println!("JA3 hash    : {server_ja3_hash}");
    println!("JA3 string  : {server_ja3}");
    println!("JA4         : {server_ja4}");
    if !server_ja4_r.is_empty() {
        println!("JA4 raw     : {server_ja4_r}");
    }
    if !akamai.is_empty() {
        println!("Akamai h2   : {akamai}");
    }
    println!("--- local (reconciliation) ---");
    println!("JA3 local   : {local_ja3}");
    println!("JA3 stripped: {local_ja3_stripped_md5} (md5 of GREASE-stripped canonical)");
    println!("JA4 v2      : {local_ja4}");

    // Tier-2 assertions.
    assert_eq!(tls_version, "TLS 1.3", "tls_version_negotiated must be 772");
    assert_eq!(
        server_ja4, local_ja4,
        "server JA4 != local JA4 v2 — wire or algorithm divergence"
    );
    let expected_ja4 = match profile {
        BrowserProfile::Chrome130 => CHROME130_JA4,
        BrowserProfile::Firefox128Esr => FIREFOX128ESR_JA4,
        _ => unreachable!("grader only grades Chrome130/Firefox128Esr"),
    };
    assert_eq!(server_ja4, expected_ja4, "server JA4 != locked constant");

    // JA3: tls.peet.ws strips GREASE from its canonical string before
    // hashing (unlike the classic spec). Assert its hash equals the md5 of
    // our locally built canonical with GREASE stripped — this proves every
    // non-GREASE JA3 field is identical on the wire.
    assert_eq!(
        server_ja3_hash, local_ja3_stripped_md5,
        "server JA3 (GREASE-stripped) != local JA3 (GREASE-stripped)"
    );
    assert_eq!(
        server_ja3_hash,
        local::md5_hex(local::strip_grease(server_ja3).as_bytes()),
        "server JA3 hash must be the md5 of its own GREASE-stripped string"
    );
    if profile == BrowserProfile::Firefox128Esr {
        assert_eq!(
            server_ja3_hash, FIREFOX128ESR_JA3,
            "Firefox JA3 is stable and locked"
        );
    }

    Ok(())
}

/// Local re-implementations of the fingerprints tls.peet.ws reports.
///
/// Deliberately duplicated in `tests/tls_peet_ws.rs` so the ignored
/// network test runs the same reconciliation logic without a shared
/// module (the brief scopes new files to http2/, the example, and the
/// test).
mod local {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use md5::{Digest, Md5};
    use ring::digest;

    use xray_tui_tls::SecureRandom;
    use xray_tui_tls::hello::parse::ParsedClientHello;
    use xray_tui_tls::spec::grease::is_grease;

    /// Deterministic RNG (all `0x42`) mirroring the crate's test double.
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

    /// Extracts the sigalgs of the `signature_algorithms` extension
    /// (0x000d), in wire order.
    fn sigalgs(hello: &ParsedClientHello) -> Vec<u16> {
        u16_list(hello.extension(0x000d))
    }

    /// Decodes a u16-BE list behind a 2-byte length prefix (the RFC 8446
    /// vector shape shared by `supported_groups` and
    /// `signature_algorithms`).
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

    /// First ALPN protocol from the ALPN extension (0x0010).
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

    /// Classic ja3er canonical string (dashes within a group, commas
    /// between groups, decimal ids) — the exact shape tls.peet.ws emits.
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

    /// Removes the GREASE values from a canonical JA3 string. tls.peet.ws
    /// strips GREASE before hashing (its `fingerprint_tls.go` skips GREASE
    /// ciphers/extensions/curves), so stripping the local fixed-seed
    /// canonical yields the server's exact md5 input.
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

    /// Current `FoxIO` JA4 (`ja4.py`, FoxIO-LLC/ja4; `ja4.go`, `TrackMe`):
    /// `t13d` + SNI flag `d` + non-GREASE cipher count + non-GREASE
    /// extension count + first/last chars of the first ALPN + `_` +
    /// SHA-256[:12] of sorted non-GREASE ciphers + `_` + SHA-256[:12] of
    /// (sorted non-GREASE extensions minus SNI/ALPN/padding, then `_`,
    /// then non-GREASE sigalgs in wire order).
    ///
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
        let sigs: Vec<u16> = sigalgs(hello)
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
