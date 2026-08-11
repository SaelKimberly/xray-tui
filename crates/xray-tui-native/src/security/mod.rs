//! Security phase: wrap the transport stream (TLS, Reality — or identity).
//!
//! MIDDLE layer: transport → SECURITY → protocol. Security dispatches on
//! `SecurityConfig` from the proto payload.

pub mod fingerprint;
pub mod reality;
pub mod tls;
pub mod tls_provider;

use std::sync::Arc;

use xray_tui_proto::proto_spec::TlsConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;
use crate::security::tls_provider::{TlsConnector, TlsParams, TlsProvider};

use self::fingerprint::FingerprintConnector;

/// Wrap the transport stream according to the profile's security config.
pub async fn wrap(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let Some(sec) = ctx.security() else {
        return Ok(stream);
    };
    if sec.is_empty() {
        return Ok(stream);
    }
    match &sec.tls {
        Some(TlsConfig::Tls(opts)) => {
            // A `Custom` provider, or any `fp` value in the config, routes
            // through the fingerprint engine (`FingerprintConnector` unless
            // the caller supplied its own `TlsConnector`).
            let custom = matches!(ctx.params.tls_provider, TlsProvider::Custom(_))
                || opts.fp.as_ref().is_some_and(|f| !f.is_empty());
            if custom {
                let connector: Arc<dyn TlsConnector> = match &ctx.params.tls_provider {
                    TlsProvider::Custom(c) => c.clone(),
                    TlsProvider::Standard => Arc::new(FingerprintConnector),
                };
                let params = TlsParams {
                    sni: ctx.sni(),
                    alpn: ctx.alpn_vec(),
                    fingerprint: opts
                        .fp
                        .as_deref()
                        .map(fingerprint::parse_fingerprint_id)
                        .transpose()?,
                    insecure: opts.insecure.unwrap_or(false),
                    pin_sha256: fingerprint::decode_pin_sha256(opts.pin_sha256.as_deref())?,
                };
                connector.connect(stream, params).await
            } else {
                tls::connect(ctx, stream).await
            }
        }
        Some(TlsConfig::Reality(_)) => reality::connect(ctx, stream).await,
        None => Ok(stream),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::rustls;
    use xray_tui_proto::proto_spec::ProtocolConfig;
    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

    use super::*;
    use crate::addr::{Host, TargetAddr};
    use crate::context::NativeConnectParams;

    fn vless_with_security(security: Option<&serde_json::Value>) -> ProtocolConfig {
        let mut obj = serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        });
        if let Some(sec) = security {
            obj["security"] = sec.clone();
        }
        serde_json::from_value(obj).expect("vless config parses")
    }

    fn vless_with_tls(sni: &str) -> ProtocolConfig {
        vless_with_security(Some(&serde_json::json!({
            "type": "tls", "sni": sni, "alpn": "http/1.1"
        })))
    }

    fn vless_with_tls_fp(sni: &str, fp: &str) -> ProtocolConfig {
        vless_with_security(Some(&serde_json::json!({
            "type": "tls", "sni": sni, "alpn": "http/1.1", "fp": fp
        })))
    }

    /// A 32-byte static public key (0xAA×32) so configs reach the handshake.
    const VALID_PBK: &str = "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqo";

    fn vless_with_reality() -> ProtocolConfig {
        vless_with_security(Some(&serde_json::json!({
            "type": "reality", "sni": "example.com", "pbk": VALID_PBK, "sid": "0011223344556677"
        })))
    }

    fn vless_with_reality_short_pbk() -> ProtocolConfig {
        vless_with_security(Some(&serde_json::json!({
            "type": "reality", "sni": "example.com", "pbk": "Zm9vYmFy"
        })))
    }

    fn ctx_for(protocol: ProtocolConfig) -> LinkContext {
        LinkContext::new(
            NativeConnectParams::new(
                protocol,
                EndpointEssentials::new("127.0.0.1", 4430),
                TargetAddr::new(Host::Domain("example.com".into()), 80),
            ),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        )
    }

    /// rcgen CA + server cert/key PEM + CA DER.
    fn rcgen_ca_and_server(sni: &str) -> (String, String, Vec<u8>) {
        use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};

        let mut ca_params = CertificateParams::new(vec![sni.to_string()]).unwrap();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca_key = KeyPair::generate().unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        let server_params =
            CertificateParams::new(vec![sni.to_string(), "127.0.0.1".to_string()]).unwrap();
        let server_key = KeyPair::generate().unwrap();
        let server_cert = server_params
            .signed_by(&server_key, &ca_cert, &ca_key)
            .unwrap();

        (
            server_cert.pem(),
            server_key.serialize_pem(),
            ca_cert.der().to_vec(),
        )
    }

    fn server_config(cert_pem: &str, key_pem: &str) -> rustls::ServerConfig {
        use rustls::pki_types::pem::PemObject;
        let certs: Vec<rustls::pki_types::CertificateDer<'static>> =
            rustls::pki_types::CertificateDer::pem_slice_iter(cert_pem.as_bytes())
                .map(|c| c.expect("cert pem parses"))
                .collect();
        let key = rustls::pki_types::PrivateKeyDer::from_pem_slice(key_pem.as_bytes())
            .expect("key pem parses");
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("server config builds")
    }

    #[tokio::test]
    async fn none_security_is_identity() {
        // No `security` key at all: `SecurityConfig` has no null representation.
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        }))
        .expect("vless config parses");
        let ctx = ctx_for(protocol);
        let (a, mut b) = tokio::io::duplex(64);
        let mut out = wrap(&ctx, Box::new(a)).await.unwrap();
        out.write_all(b"hi").await.unwrap();
        let mut buf = [0u8; 2];
        b.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hi");
    }

    #[tokio::test]
    async fn tls_wraps_and_passes_bytes() {
        // Feature unification enables both rustls backends; the app installs
        // the ring provider at startup (workspace convention), tests do it
        // here (idempotent).
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (cert_pem, key_pem, ca_der) = rcgen_ca_and_server("localhost");
        tls::set_test_config(tls::test_client_config(&ca_der));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let acceptor =
                tokio_rustls::TlsAcceptor::from(Arc::new(server_config(&cert_pem, &key_pem)));
            let mut tls = acceptor.accept(sock).await.unwrap();
            let mut buf = [0u8; 4];
            tls.read_exact(&mut buf).await.unwrap();
            tls.write_all(&buf).await.unwrap();
        });

        let ctx = ctx_for(vless_with_tls("localhost"));
        // Point the context at the test listener (server host is 127.0.0.1:addr).
        let mut params = ctx.params.clone();
        params.server = EndpointEssentials::new(addr.ip().to_string(), addr.port());
        let ctx = LinkContext::new(params, ctx.target.clone());

        let sock = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut out = wrap(&ctx, Box::new(sock)).await.unwrap();
        out.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        out.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn fingerprint_tls_wraps_and_passes_bytes() {
        // Feature unification enables both rustls backends; the app installs
        // the ring provider at startup (workspace convention), tests do it
        // here (idempotent).
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (cert_pem, key_pem, ca_der) = rcgen_ca_and_server("localhost");
        // The fingerprint engine verifies through its own thread-local test
        // CA (mirrors `tls::set_test_config` for the rustls path).
        fingerprint::set_test_ca(&ca_der);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let acceptor =
                tokio_rustls::TlsAcceptor::from(Arc::new(server_config(&cert_pem, &key_pem)));
            let mut tls = acceptor.accept(sock).await.unwrap();
            let mut buf = [0u8; 4];
            tls.read_exact(&mut buf).await.unwrap();
            tls.write_all(&buf).await.unwrap();
        });

        // Fingerprint path: `fp` id in the config AND a `Custom` provider.
        let ctx = ctx_for(vless_with_tls_fp("localhost", "chrome"));
        let mut params = ctx.params.clone();
        params.server = EndpointEssentials::new(addr.ip().to_string(), addr.port());
        params.tls_provider = TlsProvider::Custom(Arc::new(fingerprint::FingerprintConnector));
        let ctx = LinkContext::new(params, ctx.target.clone());

        let sock = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut out = wrap(&ctx, Box::new(sock)).await.unwrap();
        out.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        out.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn reality_connect_rejects_short_pbk() {
        // A `pbk` that decodes to ≠32 bytes is a config error, not a hang.
        let ctx = ctx_for(vless_with_reality_short_pbk());
        let (a, _) = tokio::io::duplex(64);
        let out = wrap(&ctx, Box::new(a)).await;
        assert!(matches!(out, Err(NativeError::Reality(_))));
    }

    #[tokio::test]
    async fn reality_handshake_errors_are_reality_errors_not_not_implemented() {
        // The arm is real: a valid config over a stream that yields EOF
        // surfaces a handshake error — the `NotImplemented` stub is gone.
        let ctx = ctx_for(vless_with_reality());
        let (a, _) = tokio::io::duplex(64);
        let out = wrap(&ctx, Box::new(a)).await;
        assert!(out.is_err(), "duplex EOF must fail the handshake");
        assert!(
            !matches!(out, Err(NativeError::NotImplemented { .. })),
            "reality must not be NotImplemented"
        );
    }
}
