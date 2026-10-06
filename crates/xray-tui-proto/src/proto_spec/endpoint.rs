//! Parse boundary types.
//!
//! Parsing produces a [`ParsedProto`]: 0..N [`EndpointEssentials`] plus one
//! [`ProtocolEssentials`]. Identity (`sig`/`cred_hash`/`uid`) is computed over
//! the serialized protocol payload only — endpoints (host/port) never
//! influence a profile's uid, so the same protocol pointed at different
//! servers dedups to one identity.
//!
//! Later tasks (T4/T5) rework every protocol parser to produce this shape; the
//! db crate (phase B) stores these types.

use super::ProtoIdentity;
use super::identity::{IdentityWriter, tag};
use crate::proto_spec::{CoreType, ProtocolConfig, ProtocolKind};
use serde::{Deserialize, Serialize};

/// Endpoint host kind. Plain enum (this crate); the db crate has its own
/// `toasty::Embed` copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostKind {
    Ipv4,
    Ipv6,
    Dns,
    Undefined,
}

/// Server endpoint, normalized for the parse boundary: host + port(s) only.
///
/// Never participates in identity hashing — only [`ProtocolEssentials`] does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointEssentials {
    pub host: String,
    pub host_type: HostKind,
    pub port: u16,       // primary port
    pub ports: Vec<u16>, // full port spec; empty when single-port
}

impl EndpointEssentials {
    /// Create a single-port endpoint. `ports` is seeded with `vec![port]`
    /// unless later overridden by a multi-port parse.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            host_type: HostKind::Undefined,
            port,
            ports: vec![port],
        }
    }
}

/// The parse-boundary protocol identity: kind, config shape, core, and the
/// exact serializable protocol definition (sans host/port).
///
/// `config` is the identity-hashed payload — `sig`/`cred_hash`/`uid` hash its
/// canonical serialized form — so it MUST NOT carry endpoint-derived values
/// (host/port). The host-free parse mandate (T4/T5) enforces this: parsers
/// never call `TransportConfig::with_host`, so the ws/http/grpc host fields
/// and `SecurityConfig::sni` hold only explicit protocol parameters. The db
/// crate's cached transport/security columns are derived from `config` at
/// write time via config accessors, not stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ProtocolEssentials {
    pub proto_kind: ProtocolKind,
    pub core_type: CoreType,
    /// The exact serializable protocol definition (config struct sans
    /// host/port). This is the identity-hashed payload.
    pub config: ProtocolConfig,
}

/// The parse boundary: 0..N endpoints (may be empty for encrypted configs) +
/// exactly one [`ProtocolEssentials`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ParsedProto {
    pub endpoints: Vec<EndpointEssentials>,
    pub protocol: ProtocolEssentials,
}

impl ProtocolEssentials {
    /// Write the parse-boundary discriminators, then delegate to the typed
    /// config's per-kind writer.
    ///
    /// `proto_kind` is load-bearing beyond the config enum: `Redirect`, `TProxy`
    /// and `Mixed` share one `PlaceholderConfig` type, so the variant is only
    /// distinguishable here. `core_type` stays in the identity for now.
    /// `config_type` (`ShareUrl` vs `Form`) was dropped (db-rewamp D9): the
    /// origin of a config does not change what it is.
    pub(crate) fn write_identity(&self, w: &mut IdentityWriter) {
        w.str(tag::PROTO_KIND, self.proto_kind.as_str());
        w.str(tag::CORE_TYPE, self.core_type.as_str());
        self.config.write_identity(w);
    }
}

/// Reinterpret a 64-bit hash as `i64` (`from_le_bytes`, never clamping — a
/// clamp would collide every hash above `i64::MAX`).
const fn as_i64(v: u64) -> i64 {
    i64::from_le_bytes(v.to_le_bytes())
}

impl ParsedProto {
    /// `sig`, `cred_hash` and `uid` from ONE per-kind binary traversal.
    ///
    /// The identity is computed over the typed [`ProtocolConfig`] fields, never
    /// over serialized JSON: no `Value` tree, no intermediate `Vec<u8>`, no
    /// per-call serialization. `sig` hashes non-credential fields only,
    /// `cred_hash` is 0 when there are no credentials (and then `uid == sig`).
    #[must_use]
    pub fn identity_once(&self) -> (i64, i64, i64) {
        let mut w = IdentityWriter::new();
        self.protocol.write_identity(&mut w);
        let id = w.finish();
        (as_i64(id.sig), as_i64(id.cred_hash), as_i64(id.uid))
    }

    /// Deterministic signature over the non-credential protocol fields: two
    /// configs differing only in credentials share a `sig`.
    #[must_use]
    pub fn sig(&self) -> i64 {
        self.identity_once().0
    }

    /// Credential-only hash; 0 when the config carries no credentials.
    #[must_use]
    pub fn cred_hash(&self) -> i64 {
        self.identity_once().1
    }

    /// `sig ^ cred_hash`, never zero.
    #[must_use]
    pub fn uid(&self) -> i64 {
        self.identity_once().2
    }

    /// The first endpoint, if any.
    #[must_use]
    pub fn first_endpoint(&self) -> Option<&EndpointEssentials> {
        self.endpoints.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto_spec::PlaceholderConfig;
    use crate::proto_spec::ProtoSpec;
    use crate::proto_spec::common::{TransportConfig, WebSocketConfig};
    use crate::urlx::RawUrlX;

    const VLESS_WS_URL: &str = "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?path=/?ed=2560&security=tls&encryption=none&sni=test.ir&type=ws";
    const SS_URL: &str = "ss://Y2xlb2Y6cGFzc3dvcmQ@1.2.3.4:8080";
    const TROJAN_URL: &str = "trojan://humanity@172.64.152.23:443?security=tls&type=ws&path=/assignment&sni=www.creationlong.org";
    const SOCKS_URL: &str = "socks://user:pass@1.2.3.4:1080";
    const HY2_URL: &str =
        "hy2://linux.do@[2a01:4f9:4b:f378::1]:13599?security=tls&insecure=1&sni=www.bing.com";

    fn config_from(url: &str) -> ProtocolConfig {
        ProtocolConfig::try_parse(&RawUrlX::from(url))
            .unwrap_or_else(|e| panic!("parse failed for {url}: {e}"))
    }

    fn proto(kind: ProtocolKind, config: ProtocolConfig) -> ProtocolEssentials {
        ProtocolEssentials {
            proto_kind: kind,
            core_type: CoreType::Xray,
            config,
        }
    }

    /// One representative config per dispatchable kind for the
    /// identity-format stability lock. Kinds with a share-URL form are parsed
    /// from it; `tor`/`ssh`/`tailscale`/placeholders have none and are built
    /// the way the app builds them (Clash conversion / `PlaceholderConfig`).
    fn identity_goldens() -> Vec<(ProtocolKind, ProtocolConfig, &'static str)> {
        use crate::clash::{ClashProxy, ClashSsh, ClashTailscale, ClashTor};

        let mut out = Vec::new();
        let clash = |proxy: &ClashProxy, kind: ProtocolKind, label: &'static str| {
            let cfg = ProtocolConfig::try_from_clash(proxy)
                .unwrap_or_else(|e| panic!("clash parse for {label}: {e}"));
            (kind, cfg, label)
        };
        out.push((
            ProtocolKind::Vless,
            config_from(VLESS_WS_URL),
            "vless-ws-tls",
        ));
        out.push((
            ProtocolKind::Vless,
            config_from(
                "vless://11111111-2222-3333-4444-555555555555@1.2.3.4:443?security=reality&pbk=AAAA&sid=1111&spx=%2F&fp=chrome&flow=xtls-rprx-vision",
            ),
            "vless-reality",
        ));
        // A mux-bearing row, so the `ID_MUX` write is pinned at the BYTE level
        // like every other tag: the rows above carry no mux, so nothing else in
        // this table would notice a change to it.
        out.push((
            ProtocolKind::Vless,
            config_from(
                "vless://11111111-2222-3333-4444-555555555555@1.2.3.4:443?type=ws&security=tls&mux=8",
            ),
            "vless-ws-mux",
        ));
        out.push((
            ProtocolKind::Vmess,
            config_from(
                "vmess://eyJ2IjoiMiIsInBzIjoieCIsImFkZCI6IjEuMi4zLjQiLCJwb3J0IjoiNDQzIiwiaWQiOiI2MjAyYjIzMC00MTdjLTRkOGUtYjYyNC0wZjcxYWZhOWM3NWQiLCJhaWQiOiIwIiwic2N5IjoiYXV0byIsIm5ldCI6IndzIiwidHlwZSI6Im5vbmUiLCJob3N0IjoiYS5leGFtcGxlIiwicGF0aCI6Ii93cyIsInRscyI6InRscyJ9",
            ),
            "vmess-ws-tls",
        ));
        out.push((
            ProtocolKind::Trojan,
            config_from(TROJAN_URL),
            "trojan-ws-tls",
        ));
        out.push((
            ProtocolKind::Shadowsocks,
            config_from(SS_URL),
            "shadowsocks",
        ));
        // The plugin identity block, pinned at the BYTE level: one row per
        // family. The fixed-point test asserts `PluginSpec` equality, which is
        // stronger for behaviour but is not a pin of a format whose doctrine
        // is that it is order-sensitive and that changing it re-keys rows.
        out.push((
            ProtocolKind::Shadowsocks,
            config_from(
                "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@1.2.3.4:8080?plugin=obfs-local%3Bobfs%3Dtls%3Bobfs-host%3DIsiBugSendiri",
            ),
            "shadowsocks-obfs-tls",
        ));
        out.push((
            ProtocolKind::Shadowsocks,
            config_from(
                "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@1.2.3.4:8080?plugin=v2ray-plugin%3Bmode%3Dwebsocket%3Bhost%3Dcdn.example%3Btls%3Bmux%3D1",
            ),
            "shadowsocks-v2ray-ws-tls",
        ));
        out.push((
            ProtocolKind::Shadowsocks2022,
            config_from("ss://MjAyMi1ibGFrZTMtYWVzLTEyOC1nY206cGFzcw@1.2.3.4:8388"),
            "shadowsocks-2022",
        ));
        out.push((
            ProtocolKind::ShadowsocksR,
            config_from(
                "ssr://ZXhhbXBsZS5jb206NDQzOm9yaWdpbjpyYzQtbWQ1OnBsYWluOmNHRnpjM2R2Y21RLz9ncm91cD1WR1Z6ZEVkeWIzVncmcmVtYXJrcz1WR1Z6ZEZObGNuWmxjZw",
            ),
            "ssr",
        ));
        out.push((ProtocolKind::Socks, config_from(SOCKS_URL), "socks5"));
        out.push((
            ProtocolKind::Http,
            config_from("http://user:pass@1.2.3.4:8080"),
            "http",
        ));
        out.push((
            ProtocolKind::Naive,
            config_from("naive+https://user:pass@example.com:443"),
            "naive",
        ));
        out.push((
            ProtocolKind::AnyTls,
            config_from("anytls://1.2.3.4:8080?password=secret"),
            "anytls",
        ));
        out.push((
            ProtocolKind::ShadowTls,
            config_from("shadowtls://1.2.3.4:443?password=pass123&version=1&sni=example.com"),
            "shadowtls",
        ));
        out.push((
            ProtocolKind::Hysteria2,
            config_from(
                "hysteria2://pw@1.2.3.4:443?sni=a.example&obfs=salamander&obfs-password=x&alpn=h3",
            ),
            "hysteria2",
        ));
        out.push((
            ProtocolKind::Hysteria,
            config_from(
                "hysteria://example.com:443?protocol=udp&obfs=xplus&up_mbps=200&down_mbps=200&insecure=1&sni=real.example.com",
            ),
            "hysteria1",
        ));
        out.push((
            ProtocolKind::Tuic,
            config_from(
                "tuic://36106e0f-4d9a-470b-a3fd-535f3b7a1e92:dongtaiwang.com@5.178.101.117:30006?congestion_control=cubic&udp_relay_mode=native&alpn=h3",
            ),
            "tuic",
        ));
        out.push((
            ProtocolKind::WireGuard,
            config_from(
                "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280",
            ),
            "wireguard",
        ));
        out.push(clash(
            &ClashProxy::Tor(ClashTor {
                name: "tor-node".into(),
                server: "127.0.0.1".into(),
                port: 9050,
            }),
            ProtocolKind::Tor,
            "tor",
        ));
        out.push(clash(
            &ClashProxy::Ssh(ClashSsh {
                name: "ssh-box".into(),
                server: "example.com".into(),
                port: 22,
                user: "root".into(),
                password: Some("sekrit".into()),
                private_key: None,
                private_key_path: Some("/home/user/.ssh/id_ed25519".into()),
                host_key: Some(vec!["ssh-ed25519 AAA".into()]),
                host_key_algorithms: Some(vec!["ssh-ed25519".into()]),
                client_version: Some("SSH-2.0-myclient".into()),
            }),
            ProtocolKind::Ssh,
            "ssh",
        ));
        out.push(clash(
            &ClashProxy::Tailscale(ClashTailscale {
                name: "ts-node".into(),
                server: "100.64.0.1".into(),
                port: 100,
                hostname: "node1".into(),
                auth_key: Some("tskey-auth-abc".into()),
                control_url: Some("https://control.example.com".into()),
                state_dir: Some("/var/lib/tailscale".into()),
                ephemeral: true,
                accept_routes: true,
                exit_node: Some("100.64.0.2".into()),
                exit_node_allow_lan_access: Some(true),
            }),
            ProtocolKind::Tailscale,
            "tailscale",
        ));
        let body = serde_json::to_vec(&serde_json::json!({"protocol_settings": {}})).unwrap();
        for (kind, name) in [
            (ProtocolKind::Redirect, "redirect"),
            (ProtocolKind::TProxy, "tproxy"),
            (ProtocolKind::Mixed, "mixed"),
        ] {
            let pc = PlaceholderConfig::new(name.to_string(), body.clone());
            let config = match kind {
                ProtocolKind::Redirect => ProtocolConfig::Redirect(pc),
                ProtocolKind::TProxy => ProtocolConfig::TProxy(pc),
                _ => ProtocolConfig::Mixed(pc),
            };
            out.push((kind, config, name));
        }
        // 22 dispatchable kinds + 2 plugin-bearing Shadowsocks rows, whose bytes
        // pin the plugin identity block.
        assert_eq!(
            out.len(),
            25,
            "every dispatchable kind is covered, plus plugin- and mux-bearing rows"
        );
        out
    }

    fn parsed(endpoints: Vec<EndpointEssentials>, protocol: ProtocolEssentials) -> ParsedProto {
        ParsedProto {
            endpoints,
            protocol,
        }
    }

    #[test]
    fn uid_is_equal_for_identical_protocol_with_different_endpoints() {
        let protocol = proto(ProtocolKind::Vless, config_from(VLESS_WS_URL));
        let a = parsed(
            vec![EndpointEssentials::new("1.2.3.4", 443)],
            protocol.clone(),
        );
        let b = parsed(
            vec![
                EndpointEssentials::new("example.com", 443),
                EndpointEssentials::new("10.0.0.1", 8443),
            ],
            protocol,
        );
        assert_eq!(a.sig(), b.sig(), "sig ignores endpoints");
        assert_eq!(a.cred_hash(), b.cred_hash(), "cred_hash ignores endpoints");
        assert_eq!(a.uid(), b.uid(), "uid ignores endpoints");
    }

    #[test]
    fn credentials_move_uid_not_sig() {
        // Different vless uuid -> different credential hash -> different uid,
        // but the SAME sig: sig is the "same way configured" grouping key and
        // must not see credentials.
        let uuid_a = parsed(
            vec![],
            proto(
                ProtocolKind::Vless,
                config_from(
                    "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?type=tcp",
                ),
            ),
        );
        let uuid_b = parsed(
            vec![],
            proto(
                ProtocolKind::Vless,
                config_from(
                    "vless://22222222-3333-4444-5555-666666666666@159.223.24.65:443?type=tcp",
                ),
            ),
        );
        assert_eq!(
            uuid_a.sig(),
            uuid_b.sig(),
            "uuid is a credential: sig ignores it"
        );
        assert_ne!(
            uuid_a.cred_hash(),
            uuid_b.cred_hash(),
            "uuid changes cred_hash"
        );
        assert_ne!(uuid_a.uid(), uuid_b.uid(), "uuid changes uid");

        // Same kind, different transport config must also differ in sig.
        let ws = parsed(
            vec![],
            proto(ProtocolKind::Vless, config_from(VLESS_WS_URL)),
        );
        assert_ne!(uuid_a.sig(), ws.sig(), "transport config changes sig");
        assert_ne!(uuid_a.uid(), ws.uid(), "transport config changes uid");
    }

    #[test]
    fn explicitly_declared_defaults_do_not_move_identity() {
        // `type=tcp` and `security=none` are the values the builder uses when
        // the parameter is absent, so spelling them out must not split a row.
        let bare = parsed(
            vec![],
            proto(
                ProtocolKind::Vless,
                config_from("vless://6202b230-417c-4d8e-b624-0f71afa9c75d@1.2.3.4:443"),
            ),
        );
        let spelled = parsed(
            vec![],
            proto(
                ProtocolKind::Vless,
                config_from(
                    "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@1.2.3.4:443?type=tcp&security=none&encryption=none",
                ),
            ),
        );
        assert_eq!(bare.uid(), spelled.uid(), "explicit defaults are elided");
    }

    #[test]
    fn parse_boundary_discriminators_are_in_identity() {
        let config = || config_from(VLESS_WS_URL);
        let base = parsed(vec![], proto(ProtocolKind::Vless, config()));

        let mut singbox = base.protocol.clone();
        singbox.core_type = CoreType::SingBox;
        let singbox = parsed(vec![], singbox);
        assert_ne!(base.uid(), singbox.uid(), "core_type is part of identity");

        let mut mixed = base.protocol.clone();
        mixed.proto_kind = ProtocolKind::Mixed;
        let mixed = parsed(vec![], mixed);
        assert_ne!(base.uid(), mixed.uid(), "proto_kind is part of identity");
    }

    #[test]
    fn uid_never_zero() {
        for (kind, url) in [
            (ProtocolKind::Vless, VLESS_WS_URL),
            (ProtocolKind::Shadowsocks, SS_URL),
            (ProtocolKind::Trojan, TROJAN_URL),
            (ProtocolKind::Socks, SOCKS_URL),
            (ProtocolKind::Hysteria2, HY2_URL),
        ] {
            let p = parsed(
                vec![EndpointEssentials::new("1.2.3.4", 443)],
                proto(kind, config_from(url)),
            );
            assert_ne!(p.sig(), 0, "sig must never be zero for {kind:?}");
            assert_ne!(p.uid(), 0, "uid must never be zero for {kind:?}");
        }
    }

    #[test]
    fn first_endpoint_none_when_endpoints_empty() {
        let p = parsed(
            vec![],
            proto(ProtocolKind::Vless, config_from(VLESS_WS_URL)),
        );
        assert_eq!(p.first_endpoint(), None);

        let p = parsed(
            vec![EndpointEssentials::new("1.2.3.4", 443)],
            proto(ProtocolKind::Vless, config_from(VLESS_WS_URL)),
        );
        let e = p.first_endpoint().expect("first endpoint present");
        assert_eq!(e.host, "1.2.3.4");
        assert_eq!(e.port, 443);
    }

    #[test]
    fn endpoint_new_seeds_ports_with_primary_port() {
        let e = EndpointEssentials::new("example.com", 8443);
        assert_eq!(e.host, "example.com");
        assert_eq!(e.host_type, HostKind::Undefined);
        assert_eq!(e.port, 8443);
        assert_eq!(e.ports, vec![8443]);
    }

    #[test]
    fn identity_hash_is_canonical_across_hashmap_insertion_order() {
        // Regression: `TransportConfig` carries `headers: Option<HashMap<..>>`,
        // and serde's direct `to_vec` iterates a HashMap in per-instance
        // random order, so value-equal protocols used to hash differently.
        // The canonical Value form (sorted keys) makes them equal.
        let mk = |headers: &[(&str, &str)]| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in headers {
                map.insert((*k).to_string(), (*v).to_string());
            }
            let ProtocolConfig::Vless(mut cfg) = config_from(VLESS_WS_URL) else {
                unreachable!("vless URL parses to VlessConfig")
            };
            cfg.transport = TransportConfig::Ws(WebSocketConfig {
                headers: Some(map),
                ..Default::default()
            });
            parsed(
                vec![],
                proto(ProtocolKind::Vless, ProtocolConfig::Vless(cfg)),
            )
        };
        let a = mk(&[("X-A", "1"), ("X-B", "2")]);
        let b = mk(&[("X-B", "2"), ("X-A", "1")]);
        assert_eq!(a.protocol, b.protocol, "protocols are value-equal");
        assert_eq!(
            a.sig(),
            b.sig(),
            "sig canonical across HashMap insertion order"
        );
        assert_eq!(
            a.cred_hash(),
            b.cred_hash(),
            "cred_hash canonical across HashMap insertion order"
        );
        assert_eq!(
            a.uid(),
            b.uid(),
            "uid canonical across HashMap insertion order"
        );
    }

    #[test]
    fn transport_host_field_is_hashed_today() {
        // Pins current behavior: the ws transport's `host` field — an explicit
        // URL-level `host=` parameter, which per the host-free parse mandate
        // IS a protocol parameter that stays in the config — is part of the
        // hashed payload today, so differing hosts change the uid. T4/T5 only
        // removes endpoint-derived hosts from parse paths.
        let mk = |host: &str| {
            let url = format!(
                "vless://6202b230-417c-4d8e-b624-0f71afa9c75d@159.223.24.65:443?type=ws&security=tls&host={host}&path=%2Fws"
            );
            parsed(vec![], proto(ProtocolKind::Vless, config_from(&url)))
        };
        assert_ne!(
            mk("cdn-a.example.com").uid(),
            mk("cdn-b.example.com").uid(),
            "transport host is part of identity today"
        );
    }

    #[test]
    fn core_type_serde_uses_as_str_dialect() {
        assert_eq!(serde_json::to_string(&CoreType::Xray).unwrap(), "\"xray\"");
        assert_eq!(
            serde_json::to_string(&CoreType::SingBox).unwrap(),
            "\"sing-box\""
        );
        assert_eq!(
            serde_json::from_str::<CoreType>("\"sing-box\"").unwrap(),
            CoreType::SingBox
        );
        assert_eq!(
            serde_json::from_str::<CoreType>("\"xray\"").unwrap(),
            CoreType::Xray
        );
    }

    #[test]
    fn protocol_essentials_serde_roundtrip() {
        let p = proto(ProtocolKind::Vless, config_from(VLESS_WS_URL));
        let bytes = serde_json::to_vec(&p).unwrap();
        let back: ProtocolEssentials = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn identity_format_is_frozen_for_every_kind() {
        const GOLDEN: &[(&str, i64, i64, i64)] = &[
            (
                // Re-keyed with `IDENTITY_VERSION` 2: the mux token is now written
                // for ANY present state (not just an active one), so the uid
                // covers the cap, the stated "no", and an unusable spelling. This
                // rides the same unshipped re-key as the plugin field — HEAD is
                // `IDENTITY_VERSION` 1 / `SCHEMA_VERSION` 13, so no second reset.
                "vless-ws-mux",
                5437349299491384277,
                7237829249111972841,
                3388034286164078652,
            ),
            (
                "vless-ws-tls",
                -4160182401465950656,
                9038095373623525140,
                -4960200424530108076,
            ),
            (
                "vless-reality",
                -5370045905108754040,
                7237829249111972841,
                -3384405737303967135,
            ),
            (
                "vmess-ws-tls",
                3755779837625638772,
                4711179184087460576,
                8466289410407626132,
            ),
            (
                "trojan-ws-tls",
                -6243832246565909625,
                8846474750042557826,
                -3198243698111379963,
            ),
            (
                "shadowsocks",
                -4695531493070296767,
                7259515508699914486,
                -2708664381270537801,
            ),
            (
                "shadowsocks-obfs-tls",
                6853842879246446139,
                -4197501671529492512,
                -7304056082489187877,
            ),
            (
                "shadowsocks-v2ray-ws-tls",
                8639150827753531647,
                -4197501671529492512,
                -5594852136968706273,
            ),
            (
                "shadowsocks-2022",
                6238586135178953044,
                -1909586701634914587,
                -5481971416994056271,
            ),
            (
                "ssr",
                -7867750124614314782,
                3784699572302144416,
                -6461021939863357630,
            ),
            (
                "socks5",
                -7562594893736803364,
                -424673764407062844,
                7860757675300375832,
            ),
            (
                "http",
                -5973881477475438080,
                -8559188781230601923,
                2607343754494327613,
            ),
            (
                "naive",
                173886074361586563,
                -3188151586708707941,
                -3339228424503523816,
            ),
            (
                "anytls",
                5799290887954844030,
                4910103980576211782,
                1467902827296672312,
            ),
            (
                "shadowtls",
                -9188312548884254956,
                -5235536646410682589,
                3975320864426807351,
            ),
            (
                "hysteria2",
                -4216711160721665791,
                -9009215055611302259,
                5153223161439682444,
            ),
            (
                "hysteria1",
                9119102330889832543,
                0,
                9119102330889832543,
            ),
            (
                "tuic",
                5129408308793111597,
                -7508355781878442882,
                -3394685403026005933,
            ),
            (
                "wireguard",
                -7515075866274409979,
                5577658325956710591,
                -2678862349368294726,
            ),
            (
                "tor",
                -7833141363582691002,
                0,
                -7833141363582691002,
            ),
            (
                "ssh",
                -6521917483108332113,
                8087748422698527689,
                -3080450271763962266,
            ),
            (
                "tailscale",
                -4230922949655668576,
                -4841268782563381559,
                8762006302573917801,
            ),
            (
                "redirect",
                226407323748179428,
                0,
                226407323748179428,
            ),
            (
                "tproxy",
                -2967083918553066919,
                0,
                -2967083918553066919,
            ),
            (
                "mixed",
                -8517797057979171685,
                0,
                -8517797057979171685,
            ),
        ];
        let goldens = identity_goldens();
        for (kind, config, label) in goldens {
            let (_, sig, cred_hash, uid) = GOLDEN
                .iter()
                .find(|(name, ..)| *name == label)
                .unwrap_or_else(|| panic!("no golden for {label}"));
            let p = parsed(vec![], proto(kind, config));
            assert_eq!(
                p.identity_once(),
                (*sig, *cred_hash, *uid),
                "identity drift for {label} — the frozen format changed (schema bump required)"
            );
        }
    }

    #[test]
    fn identity_once_matches_separate_calls() {
        // Byte-identity gate for the production identity path: one
        // serialization must equal three separate `sig()`/`cred_hash()`/
        // `uid()` calls, or stored uids re-key and rows duplicate.
        let parsed = ParsedProto {
            endpoints: vec![],
            protocol: proto(ProtocolKind::Vless, config_from(VLESS_WS_URL)),
        };
        let (sig, cred, uid) = parsed.identity_once();
        assert_eq!(
            (sig, cred, uid),
            (parsed.sig(), parsed.cred_hash(), parsed.uid())
        );
    }
}
