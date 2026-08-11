//! Standard TLS: rustls 0.23 (ring backend) via tokio-rustls.
//!
//! This is the `TlsProvider::Standard` path. It cannot emit a mimicked
//! `ClientHello` (rustls has no modification API); a fingerprinted hello
//! comes from the `TlsProvider::Custom` engine in
//! [`crate::security::fingerprint`], dispatched by `wrap()`. Certificate
//! validation is always on in the standard path — no skip-verify anywhere in
//! the native core (the fingerprint engine reads `allowInsecure`/`pinSHA256`
//! from the TLS opts instead).
//!
//! The `TEST_CFG` thread-local below is the model for the fingerprint
//! engine's own harness override (`fingerprint::TEST_CA`): each
//! `#[tokio::test]` runs on its own OS thread with its own harness CA, so
//! parallel e2e cases can't clobber each other's trust store.

use std::sync::Arc;

use tokio_rustls::rustls;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};

// Test/e2e override: a client config trusting exactly one CA (the harness
// CA). Thread-local: each `#[tokio::test]` runs on its own OS thread with its
// own harness CA, so parallel e2e cases can't clobber each other's trust
// store. Production builds use webpki-roots and carry no test state.
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
std::thread_local! {
    static TEST_CFG: std::cell::RefCell<Option<Arc<rustls::ClientConfig>>> =
        const { std::cell::RefCell::new(None) };
}

/// Build a client config trusting exactly `ca_der` (test/e2e harness only).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
#[must_use]
pub fn test_client_config(ca_der: &[u8]) -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(rustls::pki_types::CertificateDer::from(ca_der.to_vec()))
        .expect("harness CA der is a valid cert");
    rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

/// Install a test-only TLS client config (root store = harness CA). Stored
/// per-thread so parallel e2e cases with distinct CAs don't interfere.
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn set_test_config(cfg: rustls::ClientConfig) {
    TEST_CFG.with(|c| *c.borrow_mut() = Some(Arc::new(cfg)));
}

fn default_config() -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
}

pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    #[cfg(any(test, feature = "native-e2e"))]
    let mut config = TEST_CFG
        .with(|c| c.borrow().clone())
        .unwrap_or_else(default_config);
    #[cfg(not(any(test, feature = "native-e2e")))]
    let mut config = default_config();
    let alpn = ctx.alpn_vec();
    if !alpn.is_empty() {
        Arc::make_mut(&mut config).alpn_protocols = alpn;
    }

    let connector = tokio_rustls::TlsConnector::from(config);
    let sni = ctx.sni();
    let server_name = rustls::pki_types::ServerName::try_from(sni.clone())
        .map_err(|e| NativeError::Tls(format!("invalid SNI {sni:?}: {e}")))?;

    let timeout = timeouts::SECURITY;
    let tls = tokio::time::timeout(timeout, connector.connect(server_name, stream))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "tls handshake",
            limit: timeout,
        })?
        .map_err(|e| NativeError::Tls(format!("handshake with {sni}: {e}")))?;
    Ok(Box::new(tls))
}
