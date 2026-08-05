//! Parse boundary types.
//!
//! Parsing produces a [`ParsedProto`]: 0..N [`EndpointEssentials`] plus one
//! [`ProtocolEssentials`]. Identity (`sig`/`cred_hash`/`uid`) is computed over
//! the serialized protocol part ONLY — endpoints (host/port) never influence a
//! profile's uid, so the same protocol pointed at different servers dedups to
//! one identity.
//!
//! Later tasks (T4/T5) rework every protocol parser to produce this shape; the
//! db crate (phase B) stores these types.

use crate::proto_spec::common::{SecurityConfig, TransportConfig};
use crate::proto_spec::utils;
use crate::proto_spec::{CoreType, ProtocolKind};
use serde::{Deserialize, Serialize};

/// Endpoint host kind. Plain enum (this crate); the db crate has its own
/// toasty::Embed copy.
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

/// Everything that defines *what* a protocol is — kind, config shape, core,
/// transport, security — explicitly excluding endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolEssentials {
    pub proto_kind: ProtocolKind,
    pub config_type: ConfigKind,
    pub core_type: CoreType,
    pub transport: TransportEssentials,
    pub security: SecurityEssentials,
}

/// Transport half of [`ProtocolEssentials`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportEssentials {
    pub r#type: crate::proto_spec::TransportType,
    pub config: TransportConfig,
}

/// Security half of [`ProtocolEssentials`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityEssentials {
    pub r#type: crate::proto_spec::SecurityType,
    pub sni: Option<String>,
    pub fp: Option<String>,
    pub insecure: Option<bool>,
    pub config: SecurityConfig,
}

/// The parse boundary: 0..N endpoints (may be empty for encrypted configs) +
/// exactly one [`ProtocolEssentials`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedProto {
    pub endpoints: Vec<EndpointEssentials>,
    pub protocol: ProtocolEssentials,
}

impl ParsedProto {
    /// rapidhash over the serialized [`ProtocolEssentials`] JSON — the same
    /// stream-hasher construction every `ProtoIdentity::compute_sig` impl uses.
    fn protocol_hash(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let bytes = serde_json::to_vec(&self.protocol)
            .expect("ProtocolEssentials is serializable by construction");
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(&bytes);
        hasher.finish()
    }

    /// Deterministic signature over the serialized protocol essentials only.
    ///
    /// Never zero: a zero rapidhash maps to 1, mirroring `Proto::materialize`'s
    /// `NonZeroU64::new(..).unwrap_or(NonZeroU64::MIN)` fallback.
    #[must_use]
    pub fn sig(&self) -> i64 {
        let sig = self.protocol_hash();
        if sig == 0 { 1 } else { sig as i64 }
    }

    /// Credential hash over the serialized protocol essentials only, reusing
    /// the existing `utils::compute_cred_hash` primitive (stable sorted
    /// `k=v;` pairs — same algorithm the per-config `compute_cred_hash` impls
    /// use).
    #[must_use]
    pub fn cred_hash(&self) -> i64 {
        let json = serde_json::to_string(&self.protocol)
            .expect("ProtocolEssentials is serializable by construction");
        utils::compute_cred_hash(&[("protocol", &json)]) as i64
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

    fn proto(kind: ProtocolKind) -> ProtocolEssentials {
        ProtocolEssentials {
            proto_kind: kind,
            config_type: ConfigKind::ShareUrl,
            core_type: CoreType::Xray,
            transport: TransportEssentials {
                r#type: crate::proto_spec::TransportType::Tcp,
                config: TransportConfig::Tcp,
            },
            security: SecurityEssentials {
                r#type: crate::proto_spec::SecurityType::None,
                sni: None,
                fp: None,
                insecure: None,
                config: SecurityConfig::default(),
            },
        }
    }

    fn parsed(endpoints: Vec<EndpointEssentials>, kind: ProtocolKind) -> ParsedProto {
        ParsedProto {
            endpoints,
            protocol: proto(kind),
        }
    }

    #[test]
    fn uid_is_equal_for_identical_protocol_with_different_endpoints() {
        let a = parsed(
            vec![EndpointEssentials::new("1.2.3.4", 443)],
            ProtocolKind::Vmess,
        );
        let b = parsed(
            vec![
                EndpointEssentials::new("example.com", 443),
                EndpointEssentials::new("10.0.0.1", 8443),
            ],
            ProtocolKind::Vmess,
        );
        assert_eq!(a.sig(), b.sig(), "sig ignores endpoints");
        assert_eq!(a.cred_hash(), b.cred_hash(), "cred_hash ignores endpoints");
        assert_eq!(a.uid(), b.uid(), "uid ignores endpoints");
    }

    #[test]
    fn different_protocols_produce_different_uid() {
        let vmess = parsed(vec![], ProtocolKind::Vmess);
        let vless = parsed(vec![], ProtocolKind::Vless);
        assert_ne!(vmess.sig(), vless.sig());
        assert_ne!(vmess.uid(), vless.uid());

        // Same kind, different transport config must also differ.
        let mut ws = proto(ProtocolKind::Vless);
        ws.transport = TransportEssentials {
            r#type: crate::proto_spec::TransportType::Ws,
            config: TransportConfig::Ws(crate::proto_spec::common::WebSocketConfig::default()),
        };
        let ws = ParsedProto {
            endpoints: vec![],
            protocol: ws,
        };
        assert_ne!(vless.uid(), ws.uid(), "transport config changes uid");
    }

    #[test]
    fn uid_never_zero() {
        for kind in [
            ProtocolKind::Vmess,
            ProtocolKind::Vless,
            ProtocolKind::Trojan,
            ProtocolKind::Socks,
            ProtocolKind::Hysteria2,
        ] {
            let p = parsed(vec![EndpointEssentials::new("1.2.3.4", 443)], kind);
            assert_ne!(p.sig(), 0, "sig must never be zero for {kind:?}");
            assert_ne!(p.uid(), 0, "uid must never be zero for {kind:?}");
        }
    }

    #[test]
    fn first_endpoint_none_when_endpoints_empty() {
        let p = parsed(vec![], ProtocolKind::Vmess);
        assert_eq!(p.first_endpoint(), None);

        let p = parsed(
            vec![EndpointEssentials::new("1.2.3.4", 443)],
            ProtocolKind::Vmess,
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
        let p = proto(ProtocolKind::Vmess);
        let bytes = serde_json::to_vec(&p).unwrap();
        let back: ProtocolEssentials = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, p);
    }
}
