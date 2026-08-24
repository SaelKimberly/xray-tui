//! Fingerprint parsing + verifier building for the `xray-tui-tls` engine.
//!
//! `wrap()` routes every TLS connect through the engine
//! (`xray_tui_tls::client::connect`); this module supplies the engine-side
//! pieces: the URL `fp` id parser, the id → [`Fingerprint`] identity mapping, and
//! the [`WebPkiVerifier`] builder — harness CA (test/e2e), `webpki-roots`
//! (production), `allowInsecure`, or a `pinSHA256` SPKI pin.

use base64::Engine as _;
use xray_tui_tls::fingerprints::{Browser, Fingerprint, Os};
use xray_tui_tls::verify::WebPkiVerifier;

use crate::error::NativeError;

/// Browser fingerprint id, mirrored from the URL `fp` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintId {
    Chrome,
    ChromeRandomized,
    Firefox,
    Safari,
    Random,
}

impl std::fmt::Display for FingerprintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Chrome => "chrome",
            Self::ChromeRandomized => "chrome-randomized",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Random => "random",
        };
        f.write_str(s)
    }
}

/// Parse a URL `fp` value into the engine's fingerprint id.
///
/// Accepts exactly the ids Xray/v2rayN emit: `chrome`, `chrome-randomized`,
/// `firefox`, `safari`, `random`.
pub fn parse_fingerprint_id(s: &str) -> Result<FingerprintId, NativeError> {
    match s {
        "chrome" => Ok(FingerprintId::Chrome),
        "chrome-randomized" => Ok(FingerprintId::ChromeRandomized),
        "firefox" => Ok(FingerprintId::Firefox),
        "safari" => Ok(FingerprintId::Safari),
        "random" => Ok(FingerprintId::Random),
        other => Err(NativeError::Config(format!(
            "unknown fingerprint id {other:?} (expected chrome, chrome-randomized, \
             firefox, safari or random)"
        ))),
    }
}

/// Resolve a fingerprint id to the engine's [`Fingerprint`] identity.
///
/// `Random` maps to the Chrome-130 capture, matching the engine's
/// "generic Chrome" convention.
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
