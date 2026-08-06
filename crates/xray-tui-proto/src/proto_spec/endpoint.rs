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

use crate::proto_spec::utils;
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

/// Where a profile's configuration came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigKind {
    ShareUrl,
    Form,
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
    pub config_type: ConfigKind,
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

impl ParsedProto {
    /// Canonical serialized form of [`ProtocolEssentials`]: converted through
    /// `serde_json::Value` so HashMap-backed fields (e.g. `headers` in
    /// `WebSocketConfig`/`HttpConfig`/`HttpUpgradeConfig`/`XHttpConfig`)
    /// materialize as sorted-key maps. serde's direct `to_vec` on a `HashMap`
    /// iterates entries in per-instance random order (fresh `RandomState` per
    /// map), which would make two value-equal protocols hash differently.
    fn canonical_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.protocol)
            .expect("ProtocolEssentials is serializable by construction")
    }

    /// rapidhash over the canonical serialized [`ProtocolEssentials`] JSON —
    /// the same stream-hasher construction every `ProtoIdentity::compute_sig`
    /// impl uses.
    fn protocol_hash(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let bytes = serde_json::to_vec(&self.canonical_json())
            .expect("canonical protocol Value is serializable");
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(&bytes);
        hasher.finish()
    }

    /// Deterministic signature over the canonical serialized protocol
    /// essentials only.
    ///
    /// Never zero: a zero rapidhash maps to 1, mirroring `Proto::materialize`'s
    /// `NonZeroU64::new(..).unwrap_or(NonZeroU64::MIN)` fallback.
    #[must_use]
    pub fn sig(&self) -> i64 {
        let sig = self.protocol_hash();
        if sig == 0 {
            1
        } else {
            // Bit-pattern reinterpretation (two's-complement wrap), the same
            // mapping the historical `as i64` produced. A clamping conversion
            // (e.g. `try_from().unwrap_or(i64::MAX)`) would collide distinct
            // hashes above i64::MAX and break the uid distinctness invariant.
            i64::from_le_bytes(sig.to_le_bytes())
        }
    }

    /// Credential hash over the canonical serialized protocol essentials only,
    /// reusing the existing `utils::compute_cred_hash` primitive (stable
    /// sorted `k=v;` pairs — same algorithm the per-config `compute_cred_hash`
    /// impls use).
    #[must_use]
    pub fn cred_hash(&self) -> i64 {
        let json = serde_json::to_string(&self.canonical_json())
            .expect("canonical protocol Value is serializable");
        let hash = utils::compute_cred_hash(&[("protocol", &json)]);
        // Two's-complement reinterpretation — see `sig` for why clamping is
        // wrong here: distinct credentials must yield distinct hashes.
        i64::from_le_bytes(hash.to_le_bytes())
    }

    /// `sig ^ cred_hash`, never zero.
    ///
    /// `sig` is guaranteed non-zero; `cred_hash` is a second independent hash
    /// of the same serialized protocol, so an xor of zero would require a
    /// 64-bit hash collision — guarded anyway so the invariant is structural,
    /// not probabilistic.
    #[must_use]
    pub fn uid(&self) -> i64 {
        let uid = self.sig() ^ self.cred_hash();
        if uid == 0 { 1 } else { uid }
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
            config_type: ConfigKind::ShareUrl,
            core_type: CoreType::Xray,
            config,
        }
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
    fn different_config_payloads_produce_different_uid() {
        // Different vless uuid -> different config payload -> different uid.
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
        assert_ne!(
            uuid_a.sig(),
            uuid_b.sig(),
            "different uuid -> different sig"
        );
        assert_ne!(
            uuid_a.uid(),
            uuid_b.uid(),
            "different uuid -> different uid"
        );

        // Same kind, different transport config must also differ.
        let ws = parsed(
            vec![],
            proto(ProtocolKind::Vless, config_from(VLESS_WS_URL)),
        );
        assert_ne!(uuid_a.uid(), ws.uid(), "transport config changes uid");
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
}
