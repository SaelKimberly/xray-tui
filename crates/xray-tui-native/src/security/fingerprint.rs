//! Fingerprint parsing + verifier building for the `xray-tui-tls` engine.
//!
//! `wrap()` routes every TLS connect through the engine
//! (`xray_tui_tls::client::connect`); this module supplies the engine-side
//! pieces: the URL `fp` id parser, the id → [`Fingerprint`] identity mapping, and
//! the [`WebPkiVerifier`] builder — harness CA (test/e2e), `webpki-roots`
//! (production), `allowInsecure`, or a `pinSHA256` SPKI pin.

use base64::Engine as _;
use xray_tui_tls::fingerprints::{Browser, Device, Fingerprint, Os};
use xray_tui_tls::verify::WebPkiVerifier;

use crate::error::NativeError;

/// Browser fingerprint id, mirrored from the URL `fp` parameter.
///
/// The accepted set is xray's `PresetFingerprints` minus the ids this engine
/// has no hello for (`360`, `qq`, `android`, `randomizednoalpn`, `unsafe`):
/// a subscription carrying one of those is reported untestable rather than
/// probed with a shape it did not ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintId {
    Chrome,
    ChromeRandomized,
    Firefox,
    Safari,
    Edge,
    /// `ios` — uTLS `HelloIOS_Auto`; the engine's Safari-on-iOS identity.
    Ios,
    Random,
}

impl std::fmt::Display for FingerprintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Chrome => "chrome",
            Self::ChromeRandomized => "chrome-randomized",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Edge => "edge",
            Self::Ios => "ios",
            Self::Random => "random",
        };
        f.write_str(s)
    }
}

/// Parse a URL `fp` value into the engine's fingerprint id.
///
/// `randomized` is xray's alias for the randomized-Chrome hello
/// (`HelloRandomized`), and `edge`/`ios` are presets the roster carries
/// (`edge_106`, Safari-on-iOS) — three ids that used to be refused although a
/// profile existed, which marked ~19 links per subscription "not testable by
/// the native engine". `360`, `qq`, `android`, `randomizednoalpn` and
/// `unsafe` have no engine hello and stay refused.
pub fn parse_fingerprint_id(s: &str) -> Result<FingerprintId, NativeError> {
    match s {
        "chrome" => Ok(FingerprintId::Chrome),
        "chrome-randomized" | "randomized" => Ok(FingerprintId::ChromeRandomized),
        "firefox" => Ok(FingerprintId::Firefox),
        "safari" => Ok(FingerprintId::Safari),
        "edge" => Ok(FingerprintId::Edge),
        "ios" => Ok(FingerprintId::Ios),
        "random" => Ok(FingerprintId::Random),
        other => Err(NativeError::Config(format!(
            "unknown fingerprint id {other:?} (expected chrome, chrome-randomized, \
             randomized, firefox, safari, edge, ios or random)"
        ))),
    }
}

/// Resolve a fingerprint id to the engine's [`Fingerprint`] identity.
///
/// `Random` maps to the Chrome-130 capture, matching the engine's
/// "generic Chrome" convention. The concrete versions are the roster rows the
/// resolution table carries (`edge_106`, Safari-on-iOS 26) — pinned by
/// `every_fingerprint_id_resolves_to_a_roster_row`, so a roster change fails
/// the test instead of silently probing with a shape the link did not ask for.
pub const fn profile_for(fp: FingerprintId) -> Result<Fingerprint, NativeError> {
    Ok(match fp {
        FingerprintId::Chrome | FingerprintId::ChromeRandomized | FingerprintId::Random => {
            Fingerprint::new(Browser::Chrome).with_version(130)
        }
        FingerprintId::Firefox => Fingerprint::new(Browser::Firefox)
            .with_version(128)
            .with_os(Os::Linux),
        FingerprintId::Safari => Fingerprint::new(Browser::Safari)
            .with_version(17)
            .with_os(Os::MacOs),
        FingerprintId::Edge => Fingerprint::new(Browser::Edge)
            .with_version(106)
            .with_os(Os::Windows)
            .with_device(Device::Desktop),
        FingerprintId::Ios => Fingerprint::new(Browser::Safari)
            .with_version(26)
            .with_os(Os::Ios)
            .with_device(Device::Phone),
    })
}

// Test/e2e override: a verifier trusting exactly one CA (the harness CA).
// Thread-local: each `#[tokio::test]` runs on its own OS thread with its own
// harness CA, so parallel e2e cases can't clobber each other's trust store.
// Production builds use webpki-roots and carry no test state.
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
std::thread_local! {
    static TEST_CA: std::cell::RefCell<Option<WebPkiVerifier>> =
        const { std::cell::RefCell::new(None) };
}

/// Install a test-only verifier trusting exactly `ca_der` (test/e2e harness).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn set_test_ca(ca_der: &[u8]) {
    let verifier = WebPkiVerifier::from_ca_der(ca_der).expect("harness CA der is a valid cert");
    TEST_CA.with(|c| *c.borrow_mut() = Some(verifier));
}

/// Build the verifier for a connect: test CA (harness) else webpki-roots,
/// then `insecure` / `pin_sha256` overrides from the TLS opts.
pub(crate) fn verifier_for(insecure: bool, pin: Option<[u8; 32]>) -> WebPkiVerifier {
    #[cfg(any(test, feature = "native-e2e"))]
    let verifier = TEST_CA
        .with(|c| c.borrow().clone())
        .unwrap_or_else(WebPkiVerifier::webpki_roots);
    #[cfg(not(any(test, feature = "native-e2e")))]
    let verifier = WebPkiVerifier::webpki_roots();
    let verifier = verifier.with_insecure(insecure);
    match pin {
        Some(pin) => verifier.with_pin(pin),
        None => verifier,
    }
}

/// Decode a `pinSHA256` value (base64, standard or URL-safe) to its 32 bytes.
pub(crate) fn decode_pin_sha256(s: Option<&str>) -> Result<Option<[u8; 32]>, NativeError> {
    let Some(s) = s else {
        return Ok(None);
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s))
        .map_err(|e| NativeError::Tls(format!("invalid pinSHA256 base64: {e}")))?;
    let pin: [u8; 32] = bytes
        .try_into()
        .map_err(|_| NativeError::Tls("pinSHA256 must decode to 32 bytes".into()))?;
    Ok(Some(pin))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every id the parser accepts must resolve to a row the roster actually
    /// carries: `profile_for` pins concrete versions, and a roster change
    /// (a renamed/removed row) would otherwise surface as a handshake error on
    /// a live link instead of a test failure.
    #[test]
    fn every_fingerprint_id_resolves_to_a_roster_row() {
        for id in [
            FingerprintId::Chrome,
            FingerprintId::ChromeRandomized,
            FingerprintId::Firefox,
            FingerprintId::Safari,
            FingerprintId::Edge,
            FingerprintId::Ios,
            FingerprintId::Random,
        ] {
            let fp = profile_for(id).expect("profile_for is total");
            let resolved = fp
                .resolve()
                .unwrap_or_else(|e| panic!("{id} ({}) has no roster row: {e}", fp.render()));
            assert_eq!(resolved.fingerprint.browser, fp.browser, "{id}");
        }
    }

    /// xray's own preset ids: the ones with an engine hello are accepted, the
    /// rest stay refused (they would have to be probed with a shape the link
    /// did not ask for).
    #[test]
    fn presets_split_into_served_and_refused() {
        for fp in [
            "chrome",
            "chrome-randomized",
            "randomized",
            "firefox",
            "safari",
            "edge",
            "ios",
            "random",
        ] {
            assert!(parse_fingerprint_id(fp).is_ok(), "{fp} should be served");
        }
        for fp in ["android", "360", "qq", "randomizednoalpn", "unsafe", ""] {
            assert!(
                parse_fingerprint_id(fp).is_err(),
                "{fp} has no engine hello and must stay refused"
            );
        }
    }
}
