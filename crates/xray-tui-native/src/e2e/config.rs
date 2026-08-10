//! Typed per-core server-config and client-param builders. Both core dialects
//! live in one place per protocol; a variant only supplies the security
//! strings (see `variant.rs`).

use std::net::SocketAddr;

use xray_tui_proto::proto_spec::ProtocolConfig;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

use super::{CoreKind, ServerEnv};
use crate::NativeConnectParams;
use crate::addr::{Host, TargetAddr};

pub const UUID: &str = "00000000-0000-0000-0000-000000000000";
pub const BODY: &str = "hello native core";

/// `VMess` inbound JSON for `core`. `security` is the xray user security string
/// (mirrors intent; cosmetic for AEAD); sing-box never receives it.
#[must_use]
pub fn vmess_inbound(core: CoreKind, env: &ServerEnv, security: Option<&str>) -> String {
    // Configs reference the PEM FILES on disk, not the in-memory bytes.
    let cert_path = env.tmp.join("server.crt").to_string_lossy().into_owned();
    let key_path = env.tmp.join("server.key").to_string_lossy().into_owned();
    let json = match core {
        CoreKind::Xray => serde_json::json!({
            "inbounds": [{
                "listen": "127.0.0.1", "port": env.port, "protocol": "vmess",
                "settings": { "clients": [{
                    "id": UUID,
                    "security": security.unwrap_or("aes-128-gcm")
                }] },
                "streamSettings": { "network": "tcp", "security": "tls",
                    "tlsSettings": { "certificates": [
                        { "certificateFile": cert_path, "keyFile": key_path }
                    ], "alpn": ["http/1.1"] } }
            }],
            "outbounds": [{ "protocol": "freedom" }]
        }),
        CoreKind::SingBox => serde_json::json!({
            "log": { "level": "warn" },
            "inbounds": [{ "type": "vmess", "listen": "127.0.0.1", "listen_port": env.port,
                "users": [{ "uuid": UUID }],
                "tls": { "enabled": true, "certificate_path": cert_path, "key_path": key_path,
                    "alpn": ["http/1.1"] } }],
            "outbounds": [{ "type": "direct" }]
        }),
    };
    serde_json::to_string(&json).expect("vmess server config serializes")
}

/// VLESS inbound JSON for `core` (no payload security dimension).
#[must_use]
pub fn vless_inbound(core: CoreKind, env: &ServerEnv) -> String {
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

/// Native client params dialing a `VMess` listener with payload security `enc`.
#[must_use]
pub fn client_params_vmess(enc: &str, port: u16, target: SocketAddr) -> NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vmess",
        "uuid": UUID,
        "security": { "enc": enc, "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .expect("vmess client config parses");
    let server = EndpointEssentials::new("127.0.0.1", port);
    NativeConnectParams::new(
        protocol,
        server,
        TargetAddr::new(Host::Ip(target.ip()), target.port()),
    )
}

/// Native client params dialing a VLESS listener.
#[must_use]
pub fn client_params_vless(port: u16, target: SocketAddr) -> NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vless",
        "uuid": UUID,
        "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .expect("vless client config parses");
    let server = EndpointEssentials::new("127.0.0.1", port);
    NativeConnectParams::new(
        protocol,
        server,
        TargetAddr::new(Host::Ip(target.ip()), target.port()),
    )
}
