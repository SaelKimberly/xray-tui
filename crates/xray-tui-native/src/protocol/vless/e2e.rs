//! E2E case for VLESS (TLS + TCP) — declarative input to the e2e pipeline.
#![cfg(feature = "native-e2e")]

use std::net::SocketAddr;

use xray_tui_proto::proto_spec::ProtocolConfig;

use crate::NativeConnectParams;
use crate::e2e::{Certs, CoreKind, E2eCase, E2eExpect, ServerEnv};
use crate::security;

pub struct VlessCase;

const UUID: &str = "00000000-0000-0000-0000-000000000000";
const BODY: &str = "hello native core";

impl E2eCase for VlessCase {
    fn label(&self) -> String {
        "vless/tcp/tls".to_string()
    }

    fn server_config(&self, core: CoreKind, env: &ServerEnv) -> String {
        // Configs below reference the PEM FILES on disk, not the byte buffers.
        let cert_path = env.tmp.join("server.crt").to_string_lossy().into_owned();
        let key_path = env.tmp.join("server.key").to_string_lossy().into_owned();
        let json = match core {
            CoreKind::Xray => serde_json::json!({
                "inbounds": [{
                    "listen": "127.0.0.1", "port": env.port, "protocol": "vless",
                    "settings": { "clients": [{ "id": UUID }], "decryption": "none" },
                    "streamSettings": { "network": "tcp", "security": "tls",
                        "tlsSettings": { "certificates": [
                            { "certificateFile": cert_path, "keyFile": key_path }
                        ], "alpn": ["http/1.1"] } }
                }],
                "outbounds": [{ "protocol": "freedom" }]
            }),
            CoreKind::SingBox => serde_json::json!({
                "log": { "level": "warn" },
                "inbounds": [{ "type": "vless", "listen": "127.0.0.1", "listen_port": env.port,
                    "users": [{ "uuid": UUID }],
                    "tls": { "enabled": true, "certificate_path": cert_path, "key_path": key_path,
                        "alpn": ["http/1.1"] } }],
                "outbounds": [{ "type": "direct" }]
            }),
        };
        serde_json::to_string(&json).expect("vless server config serializes")
    }

    fn client_params(&self, port: u16, target: SocketAddr) -> NativeConnectParams {
        let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vless",
            "uuid": UUID,
            "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
            "transport": { "type": "tcp" }
        }))
        .expect("vless client config parses");
        let server =
            xray_tui_proto::proto_spec::endpoint::EndpointEssentials::new("127.0.0.1", port);
        NativeConnectParams::new(
            protocol,
            server,
            crate::addr::TargetAddr::new(crate::addr::Host::Ip(target.ip()), target.port()),
        )
    }

    fn expected(&self) -> E2eExpect {
        E2eExpect {
            status: 200,
            body: BODY.into(),
        }
    }

    fn client_trust(&self, certs: &Certs) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));
    }
}

impl Default for VlessCase {
    fn default() -> Self {
        Self
    }
}
