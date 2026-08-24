//! Browser fingerprint profiles as spec data.
//!
//! Each profile module ports one `ClientHello` shape from the reference
//! implementations (`thirdparty/tls-fingerprint/src/profiles/*.rs` and the
//! uTLS `HelloChrome_133` preset) into a [`ClientHelloSpec`]. Each module
//! exposes one `pub fn spec()`; resolution lives in
//! `crate::fingerprints`.

pub mod android11_okhttp;
pub mod brave167;
pub mod chrome;
pub mod chrome119;
pub mod chrome133;
pub mod chrome_android130;
pub mod edge;
pub mod edge106;
pub mod firefox;
pub mod firefox120;
pub mod firefox128esr;
pub mod ios14;
pub mod opera114;
pub mod safari;
pub mod safari16;
pub mod safari_ios17;
use crate::spec::{ClientHelloSpec, ExtensionSpec, SessionIdSpec};

/// Assembles a [`ClientHelloSpec`] from the pieces a `spec!` declaration
/// names. `legacy_version` is fixed at `0x0303` and compression at `[0]` —
/// both TLS 1.3 invariants (see [`ClientHelloSpec`]).
#[allow(dead_code)] // consumed by spec! (Task 5) and the equivalence tests below
pub(crate) fn spec_from_parts(
    cipher_suites: Vec<u16>,
    extensions: Vec<ExtensionSpec>,
    session_id: SessionIdSpec,
) -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites,
        compression_methods: vec![0x00],
        session_id,
        extensions,
    }
}

/// Decodes the hex body of a `spec!` `raw[ty, "hex"]` token into bytes.
/// Panics on malformed input — a `spec!` declaration is source code, so a
/// bad body is a compile-time-adjacent authoring error, surfaced the moment
/// the profile function runs.
#[allow(dead_code)] // consumed by spec! (Task 5) and the equivalence tests below
pub(crate) fn decode_hex(s: &str) -> Vec<u8> {
    assert_eq!(
        s.len() % 2,
        0,
        "spec! raw body must be even-length hex, got {s:?}"
    );
    s.as_bytes()
        .chunks_exact(2)
        .map(|c| (hex_val(c[0]) << 4) | hex_val(c[1]))
        .collect()
}

/// Decodes one hex digit into its value (spec! `raw` bodies).
fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => panic!("invalid hex digit {b:#x} in spec! raw body"),
    }
}

/// One cipher-suite token: `GREASE` (either case) or a u16 literal.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! cipher_token {
    (GREASE) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    (grease) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    ($lit:literal) => {
        $lit
    };
}

/// A u16 in a plain id-list context (`versions`, `sigalgs`): `grease` or a
/// u16 literal.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! u16_token {
    (grease) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    (GREASE) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    ($lit:literal) => {
        $lit
    };
}

/// A `groups[...]` entry: u16 literal or a named group id.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! group_token {
    (grease) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    (GREASE) => {
        $crate::spec::grease::GREASE_PLACEHOLDER
    };
    (x25519) => {
        0x001D
    };
    (mlkem768) => {
        0x11EC
    }; // x25519mlkem768 hybrid
    (p256) => {
        0x0017
    };
    (p384) => {
        0x0018
    };
    (p521) => {
        0x0019
    };
    ($lit:literal) => {
        $lit
    };
}

/// A `compress[...]` entry: RFC 8879 algorithm name or u16 literal.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! compress_token {
    (zlib) => {
        0x0001
    };
    (brotli) => {
        0x0002
    };
    (zstd) => {
        0x0003
    };
    ($lit:literal) => {
        $lit
    };
}

/// A `keyshare[...]` entry. `p521` has no [`KeyShareGroup`] variant — the
/// engine has no P-521 key exchange — so it fails with a clear error rather
/// than silently mapping to nothing.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! keyshare_token {
    (grease) => {
        $crate::spec::KeyShareGroup::Grease
    };
    (x25519) => {
        $crate::spec::KeyShareGroup::X25519
    };
    (mlkem768) => {
        $crate::spec::KeyShareGroup::X25519Mlkem768
    };
    (p256) => {
        $crate::spec::KeyShareGroup::Secp256r1Mlkem768
    };
    (p384) => {
        $crate::spec::KeyShareGroup::Secp384r1Mlkem1024
    };
    (p521) => {
        compile_error!(
            "spec!: keyshare `p521` has no KeyShareGroup variant \
             (see spec/mod.rs; the engine has no P-521 key exchange)"
        )
    };
}

/// A `session:` token.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! session_token {
    (random32) => {
        $crate::spec::SessionIdSpec::Random32
    };
    (empty) => {
        $crate::spec::SessionIdSpec::Empty
    };
}

/// One `exts:` token: a bare unit-variant id or a `name[args]` tuple form.
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! ext_token {
    (grease) => { $crate::spec::ExtensionSpec::Grease };
    (sni) => { $crate::spec::ExtensionSpec::ServerName };
    (reneg) => { $crate::spec::ExtensionSpec::RenegotiationInfo };
    (ecpf) => { $crate::spec::ExtensionSpec::EcPointFormats };
    (ticket) => { $crate::spec::ExtensionSpec::SessionTicket };
    (status) => { $crate::spec::ExtensionSpec::StatusRequest };
    (sct) => { $crate::spec::ExtensionSpec::SignedCertificateTimestamp };
    (psk) => { $crate::spec::ExtensionSpec::PskKeyExchangeModes };
    (padding) => { $crate::spec::ExtensionSpec::Padding };
    (groups[$($g:tt),*]) => {
        $crate::spec::ExtensionSpec::SupportedGroups(vec![$(group_token!($g)),*])
    };
    (keyshare[$($k:tt),*]) => {
        $crate::spec::ExtensionSpec::KeyShare(vec![$(keyshare_token!($k)),*])
    };
    (versions[$($v:tt),*]) => {
        $crate::spec::ExtensionSpec::SupportedVersions(vec![$(u16_token!($v)),*])
    };
    (sigalgs[$($s:tt),*]) => {
        $crate::spec::ExtensionSpec::SignatureAlgorithms(vec![$(u16_token!($s)),*])
    };
    (compress[$($c:tt),*]) => {
        $crate::spec::ExtensionSpec::CompressCertificate(vec![$(compress_token!($c)),*])
    };
    (alpn[$($p:literal),*]) => {
        $crate::spec::ExtensionSpec::Alpn(vec![$($p.to_string()),*])
    };
    (appsettings[$($p:literal),*]) => {
        $crate::spec::ExtensionSpec::ApplicationSettings(vec![$($p.to_string()),*])
    };
    (rslimit[$n:literal]) => { $crate::spec::ExtensionSpec::RecordSizeLimit($n) };
    (raw[$ty:literal, $data:literal]) => {
        $crate::spec::ExtensionSpec::Raw {
            ty: $ty,
            data: $crate::profiles::decode_hex($data),
        }
    };
}

/// Declaratively defines a fingerprint profile function.
///
/// Expands `name` into `pub(crate) fn name() -> ClientHelloSpec` — the
/// [`SpecEntry`] shape — with `legacy_version` fixed at `0x0303` and
/// compression at `[0]` (TLS 1.3 invariants):
///
/// ```ignore
/// spec! {
///     chrome_gen_137,
///     ciphers: GREASE, 0x1301, 0x1302, 0x1303, 0xc02b,
///     session: random32,
///     exts: grease, sni, groups[grease, x25519, mlkem768],
///           versions[0x0304, 0x0303], sigalgs[0x0403, 0x0804],
///           alpn["h2", "http/1.1"], psk, padding
/// }
/// ```
///
/// # Token grammar
///
/// `ciphers:` — u16 literals (decimal or `0x`-hex) or `GREASE` (either
/// case) for a GREASE slot. A GREASE cipher, when present, must be the
/// first token (the generator and every Chromium-family profile emit it
/// first; this is what keeps the list unambiguous for the parser).
///
/// `session:` — `random32` | `empty`.
///
/// `exts:` — extension tokens in profile order:
/// - bare ids (unit variants): `grease`, `sni`, `reneg`, `ecpf`,
///   `ticket`, `status`, `sct`, `psk`, `padding`;
/// - `groups[..]` — `supported_groups`: u16 literals or named group ids
///   (`grease`, `x25519`, `mlkem768`, `p256`, `p384`, `p521`);
/// - `keyshare[..]` — `key_share`: `grease`, `x25519`, `mlkem768`, `p256`,
///   `p384` (no `p521` — no [`KeyShareGroup`] variant exists);
/// - `versions[..]`, `sigalgs[..]` — u16 literals or `grease`;
/// - `alpn[..]`, `appsettings[..]` — string literals;
/// - `compress[..]` — `zlib` | `brotli` | `zstd` or u16 literals;
/// - `rslimit[N]` — `record_size_limit`;
/// - `raw[ty, "hex"]` — arbitrary extension: u16 `ty` and the body as a
///   hex string (`""` for an empty body).
///
/// A trailing comma after the last cipher, the session value, or the last
/// extension is accepted.
#[allow(unused_macros)] // consumed by profiles/generated/*.rs (Task 5) and the equivalence tests below
macro_rules! spec {
    ($name:ident,
     ciphers: $first:tt $(, $cipher:literal)*,
     session: $session:tt,
     exts: $($ext_tail:tt)*) => {
        pub(crate) fn $name() -> $crate::spec::ClientHelloSpec {
            $crate::profiles::spec_from_parts(
                vec![cipher_token!($first) $(, cipher_token!($cipher))*],
                spec_exts!($($ext_tail)*),
                session_token!($session),
            )
        }
    };
}

/// Splits the `exts:` token tail into `ExtensionSpec`s, in order.
///
/// Each item is either a bare unit-variant id (`grease`) or an id with a
/// bracketed argument group (`groups[0x001d, x25519]`) — two token trees —
/// so the list is munched one item at a time instead of parsed with a
/// comma-separated `tt` repetition (which cannot express the optional
/// trailing group).
#[allow(unused_macros)] // consumed by spec! (Task 5) and the equivalence tests below
macro_rules! spec_exts {
    () => { Vec::new() };
    ($ext:tt , $($rest:tt)*) => {{
        let mut v = vec![ext_token!($ext)];
        v.extend(spec_exts!($($rest)*));
        v
    }};
    ($ext:tt $args:tt , $($rest:tt)*) => {{
        let mut v = vec![ext_token!($ext $args)];
        v.extend(spec_exts!($($rest)*));
        v
    }};
    ($ext:tt) => { vec![ext_token!($ext)] };
    ($ext:tt $args:tt) => { vec![ext_token!($ext $args)] };
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use crate::SecureRandom;
    use crate::crypto::fingerprint::ja3::{Ja3Fields, ja3_hash};
    use crate::crypto::fingerprint::ja4::ja4_a;
    use crate::hello::parse::parse_hello;
    use crate::hello::{BuildParams, build_hello};
    use crate::spec::ClientHelloSpec;
    use crate::spec::grease::is_grease;

    /// Every transcribed profile: stable `snake_case` name + its spec
    /// function. The resolution table in `crate::fingerprints` points at
    /// these same functions.
    type SpecEntry = (&'static str, fn() -> ClientHelloSpec);
    const ALL_SPECS: &[SpecEntry] = &[
        ("chrome", super::chrome::spec),
        ("chrome_119", super::chrome119::spec),
        ("chrome_130", super::chrome::spec),
        ("chrome_133", super::chrome133::spec),
        ("chrome_android_130", super::chrome_android130::spec),
        ("edge_106", super::edge106::spec),
        ("edge_130", super::edge::spec),
        ("brave_167", super::brave167::spec),
        ("opera_114", super::opera114::spec),
        ("firefox", super::firefox::spec),
        ("firefox_120", super::firefox120::spec),
        ("firefox_128_esr", super::firefox128esr::spec),
        ("safari_16", super::safari16::spec),
        ("safari_17", super::safari::spec),
        ("safari_ios_17", super::safari_ios17::spec),
        ("ios_14", super::ios14::spec),
        ("android_11_okhttp", super::android11_okhttp::spec),
    ];

    /// Deterministic RNG feeding back a fixed byte sequence (mirrors the
    /// `hello` test double; `AtomicUsize` keeps it `Sync` for the
    /// `SecureRandom` supertrait).
    struct FixedRandom {
        bytes: Vec<u8>,
        pos: AtomicUsize,
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

    /// Decodes a hex digit into its value (test helper).
    fn hex_val(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit: {b:#x}"),
        }
    }

    /// Decodes a hex string into bytes (test helper).
    fn decode_hex(s: &str) -> Vec<u8> {
        assert_eq!(s.len() % 2, 0, "hex string must have even length");
        s.as_bytes()
            .chunks_exact(2)
            .map(|c| (hex_val(c[0]) << 4) | hex_val(c[1]))
            .collect()
    }

    #[test]
    fn all_profiles_build_and_parse() {
        let (mlkem_pk, _) = crate::crypto::mlkem::Mlkem768::generate_keypair().unwrap();
        for (name, spec_fn) in ALL_SPECS {
            let spec = spec_fn();
            let rng = FixedRandom {
                bytes: vec![0x5A; 256],
                pos: AtomicUsize::new(0),
            };
            let hello = build_hello(
                &spec,
                &BuildParams {
                    server_name: "tls.peet.ws",
                    alpn: None, // use spec's Alpn
                    x25519_pub: &[0xAB; 32],
                    mlkem768_pub: Some(mlkem_pk.as_bytes()),
                    rng: &rng,
                },
            )
            .unwrap();
            let parsed = parse_hello(&hello.handshake_bytes).unwrap();
            let fields = Ja3Fields::from(&parsed);
            // The tls-fingerprint Firefox models, the uTLS
            // HelloFirefox_120 preset and the uTLS HelloAndroid_11_OkHttp
            // preset carry no GREASE placeholders; the Chromium family,
            // HelloIOS_14 and the uTLS Safari 16 / Edge 106 presets do.
            // GREASE-free profiles have a JA3 that is stable across seeds.
            if !matches!(
                *name,
                "firefox"
                    | "firefox_120"
                    | "firefox_128_esr"
                    | "safari_17"
                    | "safari_ios_17"
                    | "android_11_okhttp"
            ) {
                assert!(
                    parsed.cipher_suites.iter().any(|c| is_grease(*c)),
                    "{name} must carry a GREASE cipher slot"
                );
            }
            assert!(!ja3_hash(&fields).is_empty(), "{name} JA3");
            assert!(ja4_a(&fields).starts_with("t13d"), "{name} JA4-A prefix");
        }
    }

    /// Golden Firefox 128 ESR `ClientHello` captured from the reference
    /// implementation (`tls-fingerprint` crate `profiles/firefox128esr.rs`,
    /// same constants + extension list) with a fixed-seed RNG (all `0x42`),
    /// X25519 public key `[0xAB; 32]`, and the builder's 512-byte padding
    /// rule (the reference `build()` pads to a fixed 312 bytes; the
    /// JA3-invisible padding length is the only divergence). The ported
    /// spec must reproduce it byte-for-byte.
    const EXPECTED_FF128_HELLO_HEX: &str = "010001f703034242424242424242424242424242424242424242424242424242424242424242204242424242424242424242424242424242424242424242424242424242424242001a130113021303c02bc02fc02cc030cca9cca8c013c014002f00350100019400000010000e00000b746c732e706565742e777300170000ff01000100000a000e000c001d00170018001901000101000b00020100002300000010000e000c02683208687474702f312e31000500050100000000003300260024001d0020abababababababababababababababababababababababababababababababab002b00050403040303000d001c001a0403050306030807080808040805080604010501060102010203002d00020101001b00050400030002001500da0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn firefox128esr_golden_hello_with_fixed_seed() {
        let spec = super::firefox128esr::spec();
        let rng = FixedRandom {
            bytes: vec![0x42; 128],
            pos: AtomicUsize::new(0),
        };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "tls.peet.ws",
                alpn: None,
                x25519_pub: &[0xAB; 32],
                mlkem768_pub: None,
                rng: &rng,
            },
        )
        .unwrap();
        assert_eq!(hello.handshake_bytes, decode_hex(EXPECTED_FF128_HELLO_HEX));
    }

    #[test]
    fn all_spec_names_are_unique_snake_case() {
        let mut seen = Vec::new();
        for (name, _) in ALL_SPECS.iter().chain(std::iter::once(&(
            "chrome_133_macro",
            super::macro_tests::chrome133_macro as fn() -> ClientHelloSpec,
        ))) {
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
                "{name} must be snake_case"
            );
            assert!(!seen.contains(name), "duplicate profile name {name}");
            seen.push(name);
        }
    }
}

/// Rebuilds `chrome133::spec` from declarative `spec!` tokens and asserts
/// field-for-field equality with the hand-transcribed profile: same cipher
/// suite order, GREASE slots, extension order, and raw extension bodies.
#[cfg(test)]
#[allow(clippy::redundant_pub_crate)] // spec! emits pub(crate) fn; the test module is private
mod macro_tests {
    use super::chrome133;

    spec! {
        chrome133_macro,
        ciphers: GREASE, 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c,
                 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014, 0x009c, 0x009d,
                 0x002f, 0x0035,
        session: random32,
        exts: grease, sni, raw[0x0017, ""], reneg,
              groups[grease, mlkem768, x25519, p256, p384],
              ecpf, ticket, alpn["h2", "http/1.1"], status,
              sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501,
                      0x0806, 0x0601],
              sct, keyshare[grease, mlkem768, x25519], psk,
              versions[grease, 0x0304, 0x0303], compress[zstd],
              raw[0x446d, "0003026832"], grease
    }

    #[test]
    fn macro_rebuilds_chrome133() {
        assert_eq!(chrome133_macro(), chrome133::spec());
    }
}
