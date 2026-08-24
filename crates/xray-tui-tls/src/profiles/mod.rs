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
        for (name, _) in ALL_SPECS {
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
