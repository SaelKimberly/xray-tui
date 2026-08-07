//! Standard TLS: rustls 0.23 (ring backend) via tokio-rustls.
//!
//! This is the `TlsProvider::Standard` path. It cannot emit a mimicked
//! ClientHello (rustls has no modification API); a fingerprinted hello comes
//! from a `TlsProvider::Custom` engine (M2). Certificate validation is always
//! on — no skip-verify anywhere in the native core.

use std::sync::Arc;

use tokio_rustls::rustls;

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::BoxStream;

/// Test/e2e override: a client config trusting exactly one CA (the harness
/// CA). Production builds use webpki-roots and carry no test state.
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
static TEST_CFG: std::sync::OnceLock<Arc<rustls::ClientConfig>> = std::sync::OnceLock::new();

/// Build a client config trusting exactly `ca_der` (test/e2e harness only).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn test_client_config(ca_der: &[u8]) -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(rustls::pki_types::CertificateDer::from(ca_der.to_vec()))
        .expect("harness CA der is a valid cert");
    rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

/// Install a test-only TLS client config (root store = harness CA).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn set_test_config(cfg: rustls::ClientConfig) {
    let _ = TEST_CFG.set(Arc::new(cfg));
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
    let mut config = TEST_CFG.get().cloned().unwrap_or_else(default_config);
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
