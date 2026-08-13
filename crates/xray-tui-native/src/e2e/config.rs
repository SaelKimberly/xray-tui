//! Typed per-core server-config and client-param builders. Both core dialects
//! live in one place per protocol; a variant only supplies the security
//! strings (see `variant.rs`).

use std::net::SocketAddr;

use base64::Engine as _;
use ring::rand::SecureRandom as _;
use std::fmt::Write as _;
use xray_tui_proto::proto_spec::ProtocolConfig;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;

use super::{CoreKind, ServerEnv, TlsVariant};
use crate::NativeConnectParams;
use crate::addr::{Host, TargetAddr};

pub const UUID: &str = "00000000-0000-0000-0000-000000000000";
pub const BODY: &str = "hello native core";

// ── REALITY keypair / short-id generation ─────────────────────────────────

/// Generate a fresh REALITY keypair: `(server private key, client `pbk`)`,
/// both base64url-encoded (Xray's `privateKey` encoding).
///
/// ring 0.17 cannot serialize the private half of a generated
/// `EphemeralPrivateKey` (its `bytes()` is test-gated — see the Task 13
/// report, concern 2), so the private key is drawn as 32 raw bytes and the
/// public half is derived with x25519-dalek (the same X25519 math both cores
/// run). The client only ever sees `pbk`; the server inbound needs the raw
/// private key.
#[must_use]
pub fn reality_keypair() -> (String, String) {
    let mut private = [0u8; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut private)
        .expect("x25519 private key rng");
    let public = x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(private));
    let enc = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    (enc.encode(private), enc.encode(public.as_bytes()))
}

/// A fresh REALITY short id: 8 hex chars (4 random bytes).
#[must_use]
pub fn reality_sid() -> String {
    let mut bytes = [0u8; 4];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .expect("short id rng");
    let mut out = String::with_capacity(8);
    for b in bytes {
        write!(out, "{b:02x}").expect("writing to a String never fails");
    }
    out
}

// ── Server configs ────────────────────────────────────────────────────────

/// `VMess` inbound JSON for `core`. `security` is the xray user security string
/// (mirrors intent; cosmetic for AEAD); sing-box never receives it. `tls`
/// selects certificate TLS vs REALITY.
#[must_use]
pub fn vmess_inbound(
    core: CoreKind,
    env: &ServerEnv,
    security: Option<&str>,
    tls: &dyn TlsVariant,
) -> String {
    if let Some(private_key) = tls.reality_private_key() {
        let sid = tls
            .reality_sid()
            .expect("reality variant carries a short id");
        return vmess_reality_inbound(core, env, security, tls.sni(), private_key, sid);
    }
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
///
/// `tls` selects certificate TLS vs REALITY; `network` selects the transport
/// (tcp/ws/grpc). For REALITY the transport runs inside the reality tunnel
/// (xray-core serves reality over raw/grpc only; sing-box also over ws).
#[must_use]
pub fn vless_inbound(
    core: CoreKind,
    env: &ServerEnv,
    tls: &dyn TlsVariant,
    network: &str,
) -> String {
    if let Some(private_key) = tls.reality_private_key() {
        let sid = tls
            .reality_sid()
            .expect("reality variant carries a short id");
        return vless_reality_inbound(core, env, tls.sni(), private_key, sid, network);
    }
    // Configs below reference the PEM FILES on disk, not the byte buffers.
    let cert_path = env.tmp.join("server.crt").to_string_lossy().into_owned();
    let key_path = env.tmp.join("server.key").to_string_lossy().into_owned();
    // grpc and xhttp ride HTTP/2 (need h2 ALPN); ws/httpupgrade upgrade
    // with http/1.1 (the default below).
    let alpn = match network {
        "grpc" | "xhttp" => serde_json::json!(["h2"]),
        _ => serde_json::json!(["http/1.1"]),
    };
    let json = match core {
        CoreKind::Xray => {
            let mut stream = serde_json::json!({
                "network": network,
                "security": "tls",
                "tlsSettings": {
                    "certificates": [
                        { "certificateFile": cert_path, "keyFile": key_path }
                    ],
                    "alpn": alpn
                }
            });
            match network {
                "ws" => {
                    stream["wsSettings"] = serde_json::json!({ "path": "/ws" });
                }
                "grpc" => {
                    stream["grpcSettings"] = serde_json::json!({ "serviceName": "gun" });
                }
                "httpupgrade" => {
                    stream["httpupgradeSettings"] =
                        serde_json::json!({ "path": "/hu", "host": "localhost" });
                }
                // xray's splithttp dialect: network "splithttp" + settings
                // key "splithttpSettings". Server mode defaults to auto
                // (accepts packet-up + stream-up POSTs).
                "xhttp" => {
                    stream["network"] = serde_json::json!("splithttp");
                    stream["splithttpSettings"] =
                        serde_json::json!({ "path": "/x", "host": "localhost" });
                }
                _ => {}
            }
            serde_json::json!({
                "inbounds": [{
                    "listen": "127.0.0.1", "port": env.port, "protocol": "vless",
                    "settings": { "clients": [{ "id": UUID }], "decryption": "none" },
                    "streamSettings": stream
                }],
                "outbounds": [{ "protocol": "freedom" }]
            })
        }
        CoreKind::SingBox => {
            let mut inbound = serde_json::json!({
                "type": "vless", "listen": "127.0.0.1", "listen_port": env.port,
                "users": [{ "uuid": UUID }],
                "tls": { "enabled": true, "certificate_path": cert_path, "key_path": key_path,
                    "alpn": alpn }
            });
            match network {
                "ws" => {
                    inbound["transport"] = serde_json::json!({ "type": "ws", "path": "/ws" });
                }
                "grpc" => {
                    inbound["transport"] =
                        serde_json::json!({ "type": "grpc", "service_name": "gun" });
                }
                "httpupgrade" => {
                    inbound["transport"] = serde_json::json!({
                        "type": "httpupgrade", "path": "/hu", "host": "localhost"
                    });
                }
                _ => {}
            }
            serde_json::json!({
                "log": { "level": "warn" },
                "inbounds": [inbound],
                "outbounds": [{ "type": "direct" }]
            })
        }
    };
    serde_json::to_string(&json).expect("vless server config serializes")
}

/// The REALITY block per core dialect: xray's `streamSettings` object,
/// sing-box's `tls.reality` object. `dest`/`handshake` point at the local
/// TLS echo server — xtls/reality borrows the dest's TLS 1.3 `ServerHello`
/// flight for its own handshake, so a plain-HTTP echo cannot serve as the
/// fallback. (sing-box ≥1.13 nests reality inside `tls`; a top-level
/// `reality` field is rejected with "unknown field".)
fn reality_stream(
    core: CoreKind,
    env: &ServerEnv,
    sni: &str,
    private_key: &str,
    sid: &str,
) -> serde_json::Value {
    match core {
        CoreKind::Xray => serde_json::json!({
            "network": "tcp", "security": "reality",
            "realitySettings": {
                "show": false,
                "dest": format!("{}:{}", env.tls_echo.ip(), env.tls_echo.port()),
                "serverNames": [sni],
                "privateKey": private_key,
                "shortIds": [sid]
            }
        }),
        CoreKind::SingBox => serde_json::json!({
            "enabled": true,
            "server_name": sni,
            "reality": {
                "enabled": true,
                "handshake": {
                    "server": env.tls_echo.ip().to_string(),
                    "server_port": env.tls_echo.port()
                },
                "private_key": private_key,
                "short_id": [sid]
            }
        }),
    }
}

/// VLESS REALITY inbound JSON for `core`. `network` selects the transport
/// declared INSIDE the reality tunnel (tcp/ws/grpc/xhttp) — reality is the
/// outermost layer, the transport framing runs beneath it. xray-core only
/// accepts reality over raw/grpc/xhttp ("REALITY only supports RAW, XHTTP
/// and gRPC for now"), so xray's reality inbound stays tcp-only for
/// ws/httpupgrade; sing-box serves reality+ws.
fn vless_reality_inbound(
    core: CoreKind,
    env: &ServerEnv,
    sni: &str,
    private_key: &str,
    sid: &str,
    network: &str,
) -> String {
    let mut stream = reality_stream(core, env, sni, private_key, sid);
    let mut transport = None;
    match network {
        "ws" if core == CoreKind::SingBox => {
            transport = Some(serde_json::json!({ "type": "ws", "path": "/ws" }));
        }
        // xray-core refuses reality over httpupgrade ("REALITY only supports
        // RAW, XHTTP and gRPC"), so the httpupgrade reality row runs on
        // sing-box only.
        "httpupgrade" if core == CoreKind::SingBox => {
            transport = Some(serde_json::json!({
                "type": "httpupgrade", "path": "/hu", "host": "localhost"
            }));
        }
        "grpc" => {
            transport = Some(serde_json::json!({ "type": "grpc", "service_name": "gun" }));
            if core == CoreKind::Xray {
                stream["network"] = serde_json::json!("grpc");
                stream["grpcSettings"] = serde_json::json!({ "serviceName": "gun" });
            }
        }
        // xray reality serves splithttp (XHTTP is in its allowlist).
        "xhttp" if core == CoreKind::Xray => {
            stream["network"] = serde_json::json!("splithttp");
            stream["splithttpSettings"] = serde_json::json!({ "path": "/x", "host": "localhost" });
        }
        _ => {}
    }
    let json = match core {
        CoreKind::Xray => serde_json::json!({
            "inbounds": [{
                "listen": "127.0.0.1", "port": env.port, "protocol": "vless",
                "settings": { "clients": [{ "id": UUID }], "decryption": "none" },
                "streamSettings": stream
            }],
            "outbounds": [{ "protocol": "freedom" }]
        }),
        CoreKind::SingBox => {
            let mut inbound = serde_json::json!({
                "type": "vless", "listen": "127.0.0.1", "listen_port": env.port,
                "users": [{ "uuid": UUID }],
                "tls": stream
            });
            if let Some(t) = transport {
                inbound["transport"] = t;
            }
            serde_json::json!({
                "log": { "level": "warn" },
                "inbounds": [inbound],
                "outbounds": [{ "type": "direct" }]
            })
        }
    };
    serde_json::to_string(&json).expect("vless reality server config serializes")
}

/// `VMess` REALITY inbound JSON for `core`.
fn vmess_reality_inbound(
    core: CoreKind,
    env: &ServerEnv,
    security: Option<&str>,
    sni: &str,
    private_key: &str,
    sid: &str,
) -> String {
    let stream = reality_stream(core, env, sni, private_key, sid);
    let json = match core {
        CoreKind::Xray => serde_json::json!({
            "inbounds": [{
                "listen": "127.0.0.1", "port": env.port, "protocol": "vmess",
                "settings": { "clients": [{
                    "id": UUID,
                    "security": security.unwrap_or("aes-128-gcm")
                }] },
                "streamSettings": stream
            }],
            "outbounds": [{ "protocol": "freedom" }]
        }),
        CoreKind::SingBox => serde_json::json!({
            "log": { "level": "warn" },
            "inbounds": [{ "type": "vmess", "listen": "127.0.0.1", "listen_port": env.port,
                "users": [{ "uuid": UUID }],
                "tls": stream }],
            "outbounds": [{ "type": "direct" }]
        }),
    };
    serde_json::to_string(&json).expect("vmess reality server config serializes")
}

// ── Client params ─────────────────────────────────────────────────────────

/// The `security` object for `tls`: `reality` (with pbk/sid) when the
/// variant is REALITY, else plain `tls` with an optional `fp`.
fn client_security(tls: &dyn TlsVariant, network: &str) -> serde_json::Value {
    tls.reality_pbk().map_or_else(
        || plain_client_security(tls, network),
        |pbk| reality_client_security(tls, pbk),
    )
}

/// The plain-TLS client `security` object, with an optional `fp`. ALPN is
/// transport-aware: grpc rides h2, everything else upgrades with http/1.1.
fn plain_client_security(tls: &dyn TlsVariant, network: &str) -> serde_json::Value {
    let alpn = match network {
        "grpc" | "xhttp" => "h2",
        // ws and httpupgrade both upgrade over HTTP/1.1 (the default below).
        _ => "http/1.1",
    };
    let mut security = serde_json::json!({
        "type": "tls", "sni": tls.sni(), "alpn": alpn
    });
    if let Some(fp) = tls.fingerprint() {
        security["fp"] = serde_json::json!(fp);
    }
    security
}

/// The REALITY client `security` object: pbk/sid with the Chrome provisioner.
fn reality_client_security(tls: &dyn TlsVariant, pbk: &str) -> serde_json::Value {
    // The client presents its own sid when the variant overrides it (the
    // wrong-sid fallback scenario), else the server's sid.
    let sid = tls
        .reality_client_sid()
        .or_else(|| tls.reality_sid())
        .expect("reality variant carries a short id");
    serde_json::json!({
        "type": "reality",
        "sni": tls.sni(),
        "pbk": pbk,
        "sid": sid,
        "fp": "chrome"
    })
}

/// Native client params dialing a `VMess` listener with payload security `enc`.
#[must_use]
pub fn client_params_vmess(
    enc: &str,
    port: u16,
    target: SocketAddr,
    tls: &dyn TlsVariant,
) -> NativeConnectParams {
    let mut security = client_security(tls, "tcp");
    security["enc"] = serde_json::json!(enc);
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vmess",
        "uuid": UUID,
        "security": security,
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

/// Native client params dialing a VLESS listener. `xhttp_mode` selects the
/// client-side xhttp dialect ("stream-up"; `None` → packet-up) — ignored for
/// non-xhttp networks.
#[must_use]
pub fn client_params_vless(
    port: u16,
    target: SocketAddr,
    tls: &dyn TlsVariant,
    network: &str,
    xhttp_mode: Option<&'static str>,
) -> NativeConnectParams {
    let transport = match network {
        "ws" => serde_json::json!({ "type": "ws", "path": "/ws" }),
        "grpc" => serde_json::json!({ "type": "grpc", "service_name": "gun" }),
        // The proto's `TransportConfig` serde tag is snake_case
        // (`http_upgrade`); `transport_type()` reports the wire name
        // `httpupgrade` that the dispatch arms match on.
        "httpupgrade" => {
            serde_json::json!({ "type": "http_upgrade", "path": "/hu", "host": "localhost" })
        }
        // Same snake_case tag for xhttp: the variant parses as `x_http`
        // (wire/type_str name `xhttp`). The xray server dialect is
        // `splithttp`; the CLIENT mode drives the client-side dialect.
        // Packet-up is the default (forced explicitly — xray's client
        // auto-defaults to stream-one under REALITY; the packet-up-over-
        // REALITY row tests that on purpose). Stream-up rows pass their
        // mode through.
        "xhttp" => serde_json::json!({
            "type": "x_http", "path": "/x", "host": "localhost",
            "mode": xhttp_mode.unwrap_or("packet-up")
        }),
        _ => serde_json::json!({ "type": "tcp" }),
    };
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "Vless",
        "uuid": UUID,
        "security": client_security(tls, network),
        "transport": transport
    }))
    .expect("vless client config parses");
    let server = EndpointEssentials::new("127.0.0.1", port);
    NativeConnectParams::new(
        protocol,
        server,
        TargetAddr::new(Host::Ip(target.ip()), target.port()),
    )
}

#[cfg(test)]
mod tests {
    use super::super::harness::generate_certs;
    use super::super::variant::{FingerprintTls, RealityTls, StandardTls};
    use super::*;
    use xray_tui_proto::proto_spec::ProtoSpec;

    #[test]
    fn reality_keypair_round_trips() {
        let (private, pbk) = reality_keypair();
        let priv_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&private)
            .unwrap();
        let pbk_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&pbk)
            .unwrap();
        assert_eq!(priv_bytes.len(), 32);
        assert_eq!(pbk_bytes.len(), 32);
        // The public key must be the X25519 of the private key (base point).
        let expected = x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(
            <[u8; 32]>::try_from(priv_bytes.as_slice()).unwrap(),
        ));
        assert_eq!(expected.as_bytes().as_slice(), pbk_bytes.as_slice());
    }

    #[test]
    fn reality_sid_is_8_hex_chars() {
        let sid = reality_sid();
        assert_eq!(sid.len(), 8);
        assert!(sid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn reality_inbound_serializes_per_core_dialect() {
        let tls = RealityTls::fresh();
        let env = ServerEnv {
            port: 12345,
            certs: &generate_certs(),
            tmp: std::path::Path::new("/tmp"),
            echo: "127.0.0.1:9999".parse().unwrap(),
            tls_echo: "127.0.0.1:9443".parse().unwrap(),
        };
        let xray: serde_json::Value =
            serde_json::from_str(&vless_inbound(CoreKind::Xray, &env, &tls, "tcp")).unwrap();
        let settings = &xray["inbounds"][0]["streamSettings"]["realitySettings"];
        assert_eq!(settings["show"], false);
        assert_eq!(settings["dest"], "127.0.0.1:9443");
        assert_eq!(settings["serverNames"][0], "localhost");
        assert_eq!(settings["privateKey"], tls.reality_private_key().unwrap());
        assert_eq!(settings["shortIds"][0], tls.reality_sid().unwrap());

        let sing: serde_json::Value =
            serde_json::from_str(&vless_inbound(CoreKind::SingBox, &env, &tls, "tcp")).unwrap();
        let tls_block = &sing["inbounds"][0]["tls"];
        let reality = &tls_block["reality"];
        assert_eq!(tls_block["server_name"], "localhost");
        assert_eq!(reality["enabled"], true);
        assert_eq!(reality["handshake"]["server"], "127.0.0.1");
        assert_eq!(reality["handshake"]["server_port"], 9443);
        assert_eq!(reality["private_key"], tls.reality_private_key().unwrap());
        assert_eq!(reality["short_id"][0], tls.reality_sid().unwrap());
    }

    #[test]
    fn client_params_reality_carry_pbk_and_sid() {
        let tls = RealityTls::fresh();
        let target = "1.2.3.4:80".parse().unwrap();
        let params = client_params_vless(12345, target, &tls, "tcp", None);
        let sec = params.protocol.security().unwrap();
        assert_eq!(sec.type_str(), Some("reality"));
        assert_eq!(sec.pbk(), Some(tls.reality_pbk().unwrap()));
        assert_eq!(sec.sid(), Some(tls.reality_sid().unwrap()));
    }

    #[test]
    fn client_params_fingerprint_set_fp() {
        let target = "1.2.3.4:80".parse().unwrap();
        let params = client_params_vless(12345, target, &FingerprintTls("chrome"), "tcp", None);
        assert_eq!(params.protocol.security().unwrap().fp(), Some("chrome"));
    }

    #[test]
    fn client_params_standard_has_no_fp() {
        let target = "1.2.3.4:80".parse().unwrap();
        let params = client_params_vless(12345, target, &StandardTls, "tcp", None);
        assert_eq!(params.protocol.security().unwrap().fp(), None);
    }

    #[test]
    fn client_params_xhttp_mode_drives_dialect() {
        use crate::context::LinkContext;

        let target = "1.2.3.4:80".parse().unwrap();
        let dest = TargetAddr::new(Host::Domain("dest.test".into()), 80);
        let mode_of = |params: NativeConnectParams| {
            LinkContext::new(params, dest.clone())
                .transport_xhttp()
                .and_then(|c| c.mode.as_deref())
                .map(str::to_string)
        };
        // Default (None) → packet-up; an explicit mode passes through.
        let packet = client_params_vless(12345, target, &StandardTls, "xhttp", None);
        assert_eq!(mode_of(packet).as_deref(), Some("packet-up"));
        let stream = client_params_vless(12345, target, &StandardTls, "xhttp", Some("stream-up"));
        assert_eq!(mode_of(stream).as_deref(), Some("stream-up"));
    }
}
