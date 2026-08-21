//! Browser fingerprint profiles as spec data.
//!
//! Each profile module ports one `ClientHello` shape from the reference
//! implementations (`thirdparty/tls-fingerprint/src/profiles/*.rs` and the
//! uTLS `HelloChrome_133` preset) into a [`ClientHelloSpec`]. The
//! [`define_profiles!`] macro turns the profile list into the
//! [`BrowserProfile`] enum with `name()` / `spec()` / `all()` dispatch.

pub mod brave167;
pub mod chrome;
pub mod chrome119;
pub mod chrome133;
pub mod chrome_android130;
pub mod edge;
pub mod firefox;
pub mod firefox128esr;
pub mod opera114;
pub mod safari;
pub mod safari_ios17;

use crate::spec::ClientHelloSpec;

/// Generates the [`BrowserProfile`] enum and its dispatch impl.
///
/// Input: a comma-separated variant list, then a `;`, then
/// `Variant => ("name", path::spec)` pairs. The variant list drives the
/// enum and `all()`; the pairs drive `name()` and `spec()`. Every variant
/// MUST appear in exactly one pair — the generated `match` arms are
/// exhaustive, so a missing pair is a compile error.
///
/// Adapted from `thirdparty/wreq-util/src/emulate.rs` `define_enum!`.
macro_rules! define_profiles {
    (
        $(#[$meta:meta])*
        $(
            $variant:ident
        ),+ $(,)?;
        $(
            $paired:ident => ($name:expr, $spec_fn:path)
        ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum BrowserProfile {
            $(
                $variant,
            )*
        }

        impl BrowserProfile {
            /// Stable `snake_case` identifier (used in configs and logs).
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(
                        Self::$paired => $name,
                    )*
                }
            }

            /// The `ClientHello` spec for this profile.
            #[must_use]
            pub fn spec(self) -> ClientHelloSpec {
                match self {
                    $(
                        Self::$paired => $spec_fn(),
                    )*
                }
            }

            /// All known profiles.
            #[must_use]
            pub const fn all() -> &'static [BrowserProfile] {
                &[
                    $(
                        Self::$variant,
                    )*
                ]
            }
        }
    };
}

define_profiles! {
    /// Supported browser fingerprint profiles.
    Chrome, Chrome119, Chrome130, ChromeAndroid130, Edge130, Brave167, Opera114,
    Firefox, Firefox128Esr, Safari17, SafariIos17, Chrome133;
    // `Chrome` is the generic/latest-Chrome alias (chrome::spec is the
    // Chrome 130 capture); the skeleton's pair list is otherwise verbatim.
    Chrome          => ("chrome",            chrome::spec),
    Chrome119       => ("chrome_119",        chrome119::spec),
    Chrome130       => ("chrome_130",        chrome::spec),
    Chrome133       => ("chrome_133",        chrome133::spec),
    ChromeAndroid130=> ("chrome_android_130", chrome_android130::spec),
    Edge130         => ("edge_130",          edge::spec),
    Brave167        => ("brave_167",         brave167::spec),
    Opera114        => ("opera_114",         opera114::spec),
    Firefox         => ("firefox",           firefox::spec),
    Firefox128Esr   => ("firefox_128_esr",   firefox128esr::spec),
    Safari17        => ("safari_17",         safari::spec),
    SafariIos17     => ("safari_ios_17",     safari_ios17::spec),
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::BrowserProfile;
    use crate::SecureRandom;
    use crate::crypto::fingerprint::ja3::{Ja3Fields, ja3_hash};
    use crate::crypto::fingerprint::ja4::ja4_a;
    use crate::hello::parse::parse_hello;
    use crate::hello::{BuildParams, build_hello};
    use crate::spec::grease::is_grease;

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
        for profile in BrowserProfile::all() {
            let spec = profile.spec();
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
            // The tls-fingerprint Firefox AND Safari models carry no GREASE
            // placeholders; the Chromium family does. GREASE-free profiles
            // have a JA3 that is stable across seeds.
            if !matches!(
                profile,
                BrowserProfile::Firefox
                    | BrowserProfile::Firefox128Esr
                    | BrowserProfile::Safari17
                    | BrowserProfile::SafariIos17
            ) {
                assert!(
                    parsed.cipher_suites.iter().any(|c| is_grease(*c)),
                    "{profile:?} must carry a GREASE cipher slot"
                );
            }
            assert!(!ja3_hash(&fields).is_empty(), "{profile:?} JA3");
            assert!(
                ja4_a(&fields).starts_with("t13d"),
                "{profile:?} JA4-A prefix"
            );
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
        let spec = BrowserProfile::Firefox128Esr.spec();
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
    fn macro_dispatch_names() {
        assert_eq!(BrowserProfile::Chrome119.name(), "chrome_119");
        assert_eq!(BrowserProfile::Chrome130.name(), "chrome_130");
        assert_eq!(BrowserProfile::Chrome.name(), "chrome");
        assert_eq!(BrowserProfile::Chrome133.name(), "chrome_133");
        assert_eq!(
            BrowserProfile::ChromeAndroid130.name(),
            "chrome_android_130"
        );
        assert_eq!(BrowserProfile::Edge130.name(), "edge_130");
        assert_eq!(BrowserProfile::Brave167.name(), "brave_167");
        assert_eq!(BrowserProfile::Opera114.name(), "opera_114");
        assert_eq!(BrowserProfile::Firefox.name(), "firefox");
        assert_eq!(BrowserProfile::Firefox128Esr.name(), "firefox_128_esr");
        assert_eq!(BrowserProfile::Safari17.name(), "safari_17");
        assert_eq!(BrowserProfile::SafariIos17.name(), "safari_ios_17");
    }
}
