//! Security phase: wrap the transport stream (TLS, Reality — or identity).
//!
//! MIDDLE layer: transport → SECURITY → protocol. Security dispatches on
//! `SecurityConfig` from the proto payload.

pub mod reality;
pub mod tls;
pub mod tls_provider;

use xray_tui_proto::proto_spec::TlsConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

/// Wrap the transport stream according to the profile's security config.
pub async fn wrap(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let Some(sec) = ctx.security() else {
        return Ok(stream);
    };
    if sec.is_empty() {
        return Ok(stream);
    }
    match &sec.tls {
        Some(TlsConfig::Tls(_)) => tls::connect(ctx, stream).await,
        Some(TlsConfig::Reality(_)) => Err(NativeError::NotImplemented {
            feature: "security reality".into(),
        }),
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

    fn vless_with_security(security: serde_json::Value) -> ProtocolConfig {
        serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": security,
            "transport": { "type": "tcp" }
        }))
        .expect("vless config parses")
    }

    fn vless_with_tls(sni: &str) -> ProtocolConfig {
        vless_with_security(serde_json::json!({
            "type": "tls", "sni": sni, "alpn": "http/1.1"
        }))
    }

    fn vless_with_reality() -> ProtocolConfig {
        vless_with_security(serde_json::json!({
            "type": "reality", "sni": "example.com", "pbk": "Zm9vYmFy"
        }))
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
    async fn reality_security_is_not_implemented() {
        let ctx = ctx_for(vless_with_reality());
        let (a, _) = tokio::io::duplex(64);
        let out = wrap(&ctx, Box::new(a)).await;
        assert!(matches!(out, Err(NativeError::NotImplemented { .. })));
    }
}
