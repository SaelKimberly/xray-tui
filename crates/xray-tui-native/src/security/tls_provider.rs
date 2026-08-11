//! TLS provider plug: standard rustls vs a fingerprint-capable engine.
//!
//! Stock rustls (0.23) exposes no `ClientHello` modification API, so browser
//! fingerprint mimicry (the DPI/CDN bypass requirement) lives behind the
//! [`TlsConnector`] trait. `Custom` engines serve BOTH plain TLS (with
//! certificate validation) and REALITY. The fingerprint engine
//! ([`crate::security::fingerprint`]) is the default `Custom` connector;
//! `Standard` remains the no-fingerprint rustls path.

use std::sync::Arc;

use crate::BoxStream;
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

/// TLS connection parameters resolved at connect time.
#[derive(Debug, Clone)]
pub struct TlsParams {
    pub sni: String,
    pub alpn: Vec<Vec<u8>>,
    pub fingerprint: Option<FingerprintId>,
    /// Skip certificate validation entirely (v2rayN `allowInsecure`).
    pub insecure: bool,
    /// When set, only the leaf SPKI SHA-256 is compared (v2rayN `pinSHA256`).
    pub pin_sha256: Option<[u8; 32]>,
}

/// A TLS connector that can emit arbitrary `ClientHellos` (fingerprint mimicry).
pub trait TlsConnector: Send + Sync {
    fn connect(
        &self,
        stream: BoxStream,
        params: TlsParams,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<BoxStream, NativeError>> + Send>>;
}

/// Provider selection for a connect.
///
/// `Standard` ignores `fingerprint` (stock rustls cannot emit a mimicked
/// `ClientHello`); a fingerprinted hello comes from `Custom` — the engine is
/// [`crate::security::fingerprint::FingerprintConnector`], wired by `wrap()`
/// when `fp` is set or a `Custom` provider is selected.
#[derive(Clone, Default)]
pub enum TlsProvider {
    #[default]
    Standard,
    Custom(Arc<dyn TlsConnector>),
}

impl std::fmt::Debug for TlsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => f.write_str("Standard"),
            // `dyn TlsConnector` is deliberately not `Debug`; the concrete
            // engine is observable through the `FingerprintConnector` type
            // itself when it matters.
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}
