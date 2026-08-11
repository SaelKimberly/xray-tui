//! Custom TLS connector backed by the `xray-tui-tls` fingerprint engine.
//!
//! This is the `TlsProvider::Custom` engine: it emits a browser-shaped
//! `ClientHello` (Chrome 130 / Firefox 128 ESR / Safari 17) via
//! `xray_tui_tls::handshake::connect` and verifies the server through
//! `xray_tui_tls::verify::WebPkiVerifier` — the harness CA (test/e2e),
//! `webpki-roots` (production), `allowInsecure`, or a `pinSHA256` SPKI pin.
//!
//! The dispatch rule lives in [`crate::security::wrap`]: an `fp` value in the
//! TLS config, or an explicit `TlsProvider::Custom`, routes here.

use std::future::Future;

use base64::Engine as _;
use xray_tui_tls::handshake::{HandshakeParams, connect as handshake_connect};
use xray_tui_tls::profiles::BrowserProfile;
use xray_tui_tls::verify::WebPkiVerifier;

use crate::BoxStream;
use crate::error::{NativeError, timeouts};
use crate::security::tls_provider::{self, FingerprintId, TlsConnector, TlsParams};

/// The default custom engine: `xray-tui-tls` with the selected fingerprint.
#[derive(Debug, Default)]
pub struct FingerprintConnector;

/// Parse a URL `fp` value into the engine's fingerprint id.
///
/// Accepts exactly the ids Xray/v2rayN emit: `chrome`, `chrome-randomized`,
/// `firefox`, `safari`, `random`.
pub fn parse_fingerprint_id(s: &str) -> Result<tls_provider::FingerprintId, NativeError> {
    match s {
        "chrome" => Ok(FingerprintId::Chrome),
        "chrome-randomized" => Ok(FingerprintId::ChromeRandomized),
        "firefox" => Ok(FingerprintId::Firefox),
        "safari" => Ok(FingerprintId::Safari),
        "random" => Ok(FingerprintId::Random),
        other => Err(NativeError::Tls(format!(
            "unknown fingerprint id {other:?} (expected chrome, chrome-randomized, \
             firefox, safari or random)"
        ))),
    }
}

/// Resolve a fingerprint id to the concrete browser profile.
///
/// `Random` maps to the latest stable Chrome capture, matching the engine's
/// "generic Chrome" convention.
pub const fn profile_for(fp: FingerprintId) -> Result<BrowserProfile, NativeError> {
    Ok(match fp {
        FingerprintId::Chrome | FingerprintId::ChromeRandomized | FingerprintId::Random => {
            BrowserProfile::Chrome130
        }
        FingerprintId::Firefox => BrowserProfile::Firefox128Esr,
        FingerprintId::Safari => BrowserProfile::Safari17,
    })
}

// Test/e2e override: a verifier trusting exactly one CA (the harness CA).
// Thread-local, mirroring `tls.rs`'s `TEST_CFG`: each `#[tokio::test]` runs
// on its own OS thread with its own harness CA, so parallel e2e cases can't
// clobber each other's trust store. Production builds use webpki-roots and
// carry no test state.
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
fn verifier_for(params: &TlsParams) -> WebPkiVerifier {
    #[cfg(any(test, feature = "native-e2e"))]
    let verifier = TEST_CA
        .with(|c| c.borrow().clone())
        .unwrap_or_else(WebPkiVerifier::webpki_roots);
    #[cfg(not(any(test, feature = "native-e2e")))]
    let verifier = WebPkiVerifier::webpki_roots();
    let verifier = verifier.with_insecure(params.insecure);
    match params.pin_sha256 {
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

impl TlsConnector for FingerprintConnector {
    fn connect(
        &self,
        stream: BoxStream,
        params: TlsParams,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<BoxStream, NativeError>> + Send>> {
        Box::pin(async move {
            // No explicit `fp` (bare `Custom` provider): default to the
            // latest stable Chrome capture, like `fp=random`.
            let profile = profile_for(params.fingerprint.unwrap_or(FingerprintId::Random))?;
            let spec = profile.spec();
            let alpn: Option<Vec<&str>> = {
                let items = params
                    .alpn
                    .iter()
                    .map(|a| {
                        std::str::from_utf8(a)
                            .map_err(|e| NativeError::Tls(format!("invalid ALPN bytes: {e}")))
                    })
                    .collect::<Result<Vec<&str>, _>>()?;
                (!items.is_empty()).then_some(items)
            };
            let verifier = verifier_for(&params);
            let rng = ring::rand::SystemRandom::new();
            let timeout = timeouts::SECURITY;
            let tls = tokio::time::timeout(
                timeout,
                handshake_connect(
                    stream,
                    HandshakeParams {
                        spec: &spec,
                        server_name: &params.sni,
                        alpn: alpn.as_deref(),
                        verifier: &verifier,
                        rng: &rng,
                    },
                ),
            )
            .await
            .map_err(|_| NativeError::Timeout {
                step: "tls handshake",
                limit: timeout,
            })?
            .map_err(|e| NativeError::Tls(format!("fingerprint handshake: {e}")))?;
            Ok(Box::new(tls) as BoxStream)
        })
    }
}
