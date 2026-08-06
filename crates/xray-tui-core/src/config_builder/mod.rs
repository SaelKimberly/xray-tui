pub mod singbox;
pub mod xray;

pub mod clash_mixin;
use crate::core_type::CoreType;
use serde::Serialize;
use serde_json::{Value, json};
use xray_tui_db::models::{DnsSetting, Endpoint, HostType, ProfileStats, Protocol, RoutingRule};
use xray_tui_proto::proto_spec::{
    CoreType as ProtoCoreType, EndpointEssentials, HostKind, ProtocolConfig, SupportError,
};

/// Port for the gRPC Stats API (shared by xray-core and sing-box).
pub const API_PORT: u16 = 62789;
/// Port for the sing-box Clash API (`experimental.clash_api`).
pub const CLASH_API_PORT: u16 = 9090;
/// Parameters extracted from app config for building backend configs.
/// Avoids a circular dependency on `xray-tui-config`.
#[derive(Debug, Clone)]
pub struct BuildParams {
    pub v2ray_api_enabled: bool,
    pub clash_api_enabled: bool,
    pub log_level: String,
    pub socks_port: u16,
    pub http_port: Option<u16>,
    pub clash_api_port: Option<u16>,
    pub listen: String,
    pub sniffing: bool,
    pub skip_cert_verify: bool,
    pub mux: Option<serde_json::Value>,
    pub clash_mixin: Option<serde_json::Value>,
}

/// The profile's shadowsocks method string when the protocol is
/// Shadowsocks/Shadowsocks2022 — used for cipher-aware core resolution and
/// builder validation. `None` for other protocols or a missing method.
///
/// Reads the typed [`Protocol::config`] deferred JSON. Returns `None` when the
/// config is unloaded (the caller must pass a `Protocol` loaded with `config`
/// included) so cipher-aware core resolution falls back to the default; the
/// config builders themselves refuse unloaded configs with a
/// [`BuildError::InvalidProfile`] instead of panicking.
pub fn shadowsocks_method(protocol: &Protocol) -> Option<String> {
    use xray_tui_proto::proto_spec::ProtocolKind;
    if !matches!(
        protocol.proto_kind,
        ProtocolKind::Shadowsocks | ProtocolKind::Shadowsocks2022
    ) {
        return None;
    }
    if protocol.config.is_unloaded() {
        return None;
    }
    match &protocol.config.get().0 {
        ProtocolConfig::Ss(config) => Some(config.method.as_str().to_string()),
        _ => None,
    }
}

/// Map a db [`Endpoint`] to the proto [`EndpointEssentials`] the outbound
/// injectors consume (host, host kind, and the full port spec). The db
/// [`HostType`] mirrors proto [`HostKind`] 1:1.
#[must_use]
pub fn endpoint_essentials(e: &Endpoint) -> EndpointEssentials {
    EndpointEssentials {
        host: e.host.clone(),
        host_type: match e.host_type {
            HostType::Ipv4 => HostKind::Ipv4,
            HostType::Ipv6 => HostKind::Ipv6,
            HostType::Dns => HostKind::Dns,
            HostType::Undefined => HostKind::Undefined,
        },
        port: e.port,
        ports: e.ports.clone(),
    }
}

/// Borrow the typed [`ProtocolConfig`] from a `Protocol`'s deferred JSON
/// column, refusing unloaded rows with a clear error instead of panicking on
/// [`toasty::Deferred::get`]. Callers (T17 connect/ping paths) must pass a
/// `Protocol` loaded with `config` included — DB read paths exclude deferred
/// columns by default.
pub(crate) fn protocol_config(protocol: &Protocol) -> Result<&ProtocolConfig, BuildError> {
    if protocol.config.is_unloaded() {
        return Err(BuildError::InvalidProfile(format!(
            "protocol {:?} config not loaded (pass a Protocol with `config` included)",
            protocol.proto_kind
        )));
    }
    Ok(&protocol.config.get().0)
}

/// Convert typed `DnsSetting.hosts` entries (each `"domain:ip"`) into the
/// core JSON `hosts` object shape (`{ "domain": "ip" }`). The split is on the
/// FIRST colon — everything after it (IPv6 colons included) is the address —
/// so IPv6 values survive intact. Entries without a `:` separator are skipped.
pub(crate) fn build_hosts_map(hosts: &[String]) -> Value {
    let mut map = serde_json::Map::new();
    for entry in hosts {
        if let Some((domain, ip)) = entry.split_once(':') {
            let domain = domain.trim();
            let ip = ip.trim();
            if !domain.is_empty() && !ip.is_empty() {
                map.insert(domain.to_string(), json!(ip));
            }
        }
    }
    Value::Object(map)
}

#[derive(Debug, Clone, Serialize)]
pub enum BackendConfig {
    Xray(xray::XrayConfig),
    SingBox(singbox::SingBoxConfig),
}

impl BackendConfig {
    #[must_use]
    pub const fn core_type(&self) -> CoreType {
        match self {
            Self::Xray(_) => CoreType::Xray,
            Self::SingBox(_) => CoreType::SingBox,
        }
    }
}

/// A single profile in a multi-inbound batch config.
/// Bundles the endpoint, its per-pair link (core type), and the protocol,
/// plus the pre-assigned SOCKS5 port.
#[derive(Debug, Clone)]
pub struct MultiInboundItem<'a> {
    pub endpoint: &'a Endpoint,
    pub link: &'a ProfileStats,
    pub protocol: &'a Protocol,
    pub assigned_port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Support(#[from] SupportError),
}

pub struct ConfigBuilder;

impl ConfigBuilder {
    /// Build a backend config for one profile.
    ///
    /// The core is taken from `link.core_type` — the per-pair override,
    /// resolved at parse time (never `Auto`). The outbound block is produced
    /// by `protocol.config.inject_to(...)`, which until Tasks 14/15 errors
    /// with [`SupportError::UnsupportedProtocol`] (surfaced as
    /// [`BuildError::Support`]).
    pub fn build(
        endpoint: &Endpoint,
        link: &ProfileStats,
        protocol: &Protocol,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError> {
        match link.core_type {
            ProtoCoreType::Xray => {
                let config = xray::XrayConfigBuilder::build(
                    endpoint,
                    protocol,
                    link.core_type,
                    params,
                    routing,
                    dns,
                )?;
                Ok(BackendConfig::Xray(config))
            }
            ProtoCoreType::SingBox => {
                let config = singbox::SingBoxConfigBuilder::build(
                    endpoint,
                    protocol,
                    link.core_type,
                    params,
                    routing,
                    dns,
                )?;
                Ok(BackendConfig::SingBox(config))
            }
        }
    }

    /// Build a multi-inbound config for batch real ping.
    ///
    /// Creates N SOCKS5 inbounds (one per profile on its `assigned_port`),
    /// N proxy outbounds, plus standard dns-out/direct/block outbounds.
    /// Routing rules direct traffic from each inbound to its matching outbound.
    ///
    /// Pattern from v2rayN's `LoadCoreConfigSpeedtest(List<ServerTestItem>)` —
    /// one core serves an entire batch page instead of spawning one core per profile.
    ///
    /// All items must share one `link.core_type` (a batch page runs on a
    /// single core); the dispatch derives the core from the items.
    pub fn build_multi(
        items: &[MultiInboundItem],
        base_params: &BuildParams,
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError> {
        let core_type = items
            .first()
            .map(|item| item.link.core_type)
            .ok_or_else(|| {
                BuildError::InvalidProfile("build_multi: empty item list".to_string())
            })?;
        if let Some(mismatch) = items.iter().find(|item| item.link.core_type != core_type) {
            return Err(BuildError::InvalidProfile(format!(
                "build_multi: mixed core types in one batch ({core_type} vs {})",
                mismatch.link.core_type
            )));
        }
        match core_type {
            ProtoCoreType::Xray => {
                let config = xray::XrayConfigBuilder::build_multi(items, base_params, dns)?;
                Ok(BackendConfig::Xray(config))
            }
            ProtoCoreType::SingBox => {
                let config = singbox::SingBoxConfigBuilder::build_multi(items, base_params, dns)?;
                Ok(BackendConfig::SingBox(config))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toasty::{Deferred, Json};
    use xray_tui_db::models::{ConfigType, TrafficStats, Transport};
    use xray_tui_proto::proto_spec::common::TransportConfig;
    use xray_tui_proto::proto_spec::{
        CoreType as ProtoCoreType, ProtocolKind, SecurityConfig, SecurityType, TransportType,
        VlessConfig,
    };

    pub(super) fn ts(secs: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(secs).expect("valid ts")
    }

    pub(super) fn tcp_transport() -> Transport {
        Transport {
            r#type: TransportType::Tcp,
            data: Deferred::from(Json(TransportConfig::Tcp)),
        }
    }

    pub(super) fn no_security() -> xray_tui_db::models::Security {
        xray_tui_db::models::Security {
            r#type: SecurityType::None,
            sni: None,
            fp: None,
            insecure: None,
            data: Deferred::from(Json(SecurityConfig::default())),
        }
    }

    pub(super) fn vless_config() -> ProtocolConfig {
        ProtocolConfig::Vless(VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            uuid_origin: None,
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        })
    }

    pub(super) fn endpoint(host: &str, port: u16) -> Endpoint {
        Endpoint {
            id: xray_tui_db::models::EndpointId::new(1),
            host: host.to_string(),
            host_type: HostType::Dns,
            port,
            ports: Vec::new(),
            parent_id: None,
            last_source: None,
            manual_protocol_override: None,
            resolved_as: Vec::new(),
            resolved_at: None,
            created_at: ts(0),
            links: Deferred::default(),
            group_links: Deferred::default(),
        }
    }

    pub(super) fn protocol(proto_kind: ProtocolKind, config: ProtocolConfig) -> Protocol {
        Protocol {
            id: xray_tui_db::models::ProtocolId::new(1),
            sig: 0,
            cred_hash: 0,
            proto_kind,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(config)),
            created_at: ts(0),
            links: Deferred::default(),
        }
    }

    pub(super) fn link(core_type: ProtoCoreType) -> ProfileStats {
        ProfileStats {
            protocol_id: xray_tui_db::models::ProtocolId::new(1),
            endpoint_id: xray_tui_db::models::EndpointId::new(1),
            core_type,
            config_type: ConfigType::ShareUrl,
            last_used_at: None,
            last_seen_at: ts(0),
            task_id: None,
            task_queue: Vec::new(),
            latency: None,
            speed_bps: None,
            error: None,
            traffic: TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            },
            created_at: ts(0),
            updated_at: ts(0),
            version: 1,
            protocol: Deferred::default(),
            endpoint: Deferred::default(),
        }
    }

    pub(super) fn default_params() -> (BuildParams, Vec<RoutingRule>, DnsSetting) {
        let params = BuildParams {
            v2ray_api_enabled: true,
            clash_api_enabled: false,
            log_level: "warning".to_string(),
            socks_port: 10808,
            http_port: None,
            listen: "127.0.0.1".to_string(),
            sniffing: false,
            clash_api_port: None,
            mux: None,
            clash_mixin: None,
            skip_cert_verify: false,
        };
        let rules = vec![];
        let dns = DnsSetting {
            id: "default".to_string(),
            name: None,
            servers: Vec::new(),
            hosts: Vec::new(),
            query_strategy: None,
            disable_cache: false,
            disable_fallback: false,
            client_ip: None,
            cache_ttl_secs: None,
        };
        (params, rules, dns)
    }

    pub(super) fn ss_config(method: &str) -> ProtocolConfig {
        ProtocolConfig::Ss(xray_tui_proto::proto_spec::SsConfig {
            method: method.into(),
            password: "pw".to_string(),
            security: SecurityConfig::default(),
            remarks: None,
            plugin: None,
            plugin_opts: None,
        })
    }

    /// A VLESS config whose security is `reality` without publicKey/SNI —
    /// xray-core refuses it at build time (empty "realitySettings"/publicKey
    /// kills the core at startup; the old builder rejected it too).
    pub(super) fn vless_reality_without_key_config() -> ProtocolConfig {
        ProtocolConfig::Vless(VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            uuid_origin: None,
            security: SecurityConfig {
                tls: Some(xray_tui_proto::proto_spec::common::TlsConfig::Reality(
                    xray_tui_proto::proto_spec::common::RealityOpts::default(),
                )),
                enc: None,
            },
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        })
    }

    pub(super) fn hy2_config() -> ProtocolConfig {
        ProtocolConfig::Hysteria2(xray_tui_proto::proto_spec::Hysteria2Config {
            auth: "hy2-secret".to_string(),
            security: SecurityConfig::default(),
            obfs: None,
            obfs_password: None,
            up: None,
            down: None,
            hop_interval: None,
            pin_sha256: None,
            remarks: None,
        })
    }

    pub(super) fn wg_config() -> ProtocolConfig {
        ProtocolConfig::Wireguard(xray_tui_proto::proto_spec::WireguardConfig {
            private_key: "abc123def456".to_string(),
            security: SecurityConfig::default(),
            address: "10.0.0.1/24".into(),
            public_key: "pubkey789".to_string(),
            preshared_key: Some("psk_value".to_string()),
            reserved: Some("1,2,3".into()),
            mtu: Some("1380".into()),
            persistent_keepalive: Some(25),
            dns: None,
            remote_dns_resolve: None,
            remarks: None,
        })
    }

    /// Assert a successful dispatch build and the full-config skeleton: the
    /// proxy outbound is present with the right protocol/type name, the
    /// standard outbounds are present, and the top-level skeleton is intact.
    fn assert_ok_dispatch(
        result: Result<BackendConfig, BuildError>,
        expected_core: ProtoCoreType,
        expected_protocol: &str,
        proxy_field: &str,
    ) -> serde_json::Value {
        let backend = result.unwrap_or_else(|e| panic!("build must succeed: {e}"));
        assert_eq!(
            backend.core_type(),
            match expected_core {
                ProtoCoreType::Xray => crate::core_type::CoreType::Xray,
                ProtoCoreType::SingBox => crate::core_type::CoreType::SingBox,
            }
        );
        let json = match &backend {
            BackendConfig::Xray(config) => serde_json::to_value(config).unwrap(),
            BackendConfig::SingBox(config) => serde_json::to_value(config).unwrap(),
        };
        assert!(json.get("log").is_some(), "missing log");
        assert!(json.get("inbounds").is_some(), "missing inbounds");
        assert!(json.get("outbounds").is_some(), "missing outbounds");
        assert!(json.get("dns").is_some(), "missing dns");
        // xray uses "routing", sing-box uses "route".
        assert!(
            json.get("routing").is_some() || json.get("route").is_some(),
            "missing routing/route"
        );
        let outbounds = json["outbounds"].as_array().expect("outbounds array");
        let proxy = outbounds
            .iter()
            .find(|o| o["tag"] == "proxy")
            .expect("proxy outbound");
        assert_eq!(
            proxy[proxy_field].as_str().unwrap(),
            expected_protocol,
            "proxy protocol mismatch"
        );
        let tags: Vec<&str> = outbounds.iter().filter_map(|o| o["tag"].as_str()).collect();
        for required in ["direct", "block"] {
            assert!(tags.contains(&required), "missing {required} outbound");
        }
        json
    }

    #[test]
    fn build_xray_via_dispatch() {
        let endpoint = endpoint("example.com", 443);
        let protocol = protocol(ProtocolKind::Vless, vless_config());
        let link = link(ProtoCoreType::Xray);
        let (params, rules, dns) = default_params();
        let json = assert_ok_dispatch(
            ConfigBuilder::build(&endpoint, &link, &protocol, &params, &rules, &dns),
            ProtoCoreType::Xray,
            "vless",
            "protocol",
        );
        // xray skeleton extras: stats/api/policy + the api inbound.
        assert!(json.get("stats").is_some(), "missing stats");
        assert!(json.get("api").is_some(), "missing api");
        assert!(json.get("policy").is_some(), "missing policy");
        let inbounds = json["inbounds"].as_array().unwrap();
        assert!(inbounds.iter().any(|i| i["tag"] == "api"));
        // vless identity in the settings block.
        assert_eq!(
            json["outbounds"][0]["settings"]["vnext"][0]["address"],
            "example.com"
        );
        assert_eq!(json["outbounds"][0]["settings"]["vnext"][0]["port"], 443);
    }

    #[test]
    fn build_singbox_tuic_via_dispatch() {
        let endpoint = endpoint("example.com", 443);
        let tuic = ProtocolConfig::Tuic(xray_tui_proto::proto_spec::TuicConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            password: "pw".to_string(),
            congestion_control: None,
            udp_relay_mode: None,
            security: SecurityConfig::default(),
            remarks: None,
        });
        let protocol = protocol(ProtocolKind::Tuic, tuic);
        let link = link(ProtoCoreType::SingBox);
        let (params, rules, dns) = default_params();
        let json = assert_ok_dispatch(
            ConfigBuilder::build(&endpoint, &link, &protocol, &params, &rules, &dns),
            ProtoCoreType::SingBox,
            "tuic",
            "type",
        );
        // sing-box skeleton extras: experimental.v2ray_api (listen + stats).
        let exp = &json["experimental"];
        assert_eq!(exp["v2ray_api"]["listen"], format!("127.0.0.1:{API_PORT}"));
        assert_eq!(exp["v2ray_api"]["stats"]["enabled"], true);
        // tuic identity in the outbound.
        assert_eq!(
            json["outbounds"][0]["uuid"],
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(json["outbounds"][0]["server"], "example.com");
        assert_eq!(json["outbounds"][0]["server_port"], 443);
    }

    #[test]
    fn build_with_unloaded_config_returns_clear_error() {
        // Deferred config rows (default DB read paths) must error clearly,
        // never panic on `Deferred::get`.
        let endpoint = endpoint("example.com", 443);
        let mut protocol = protocol(ProtocolKind::Vless, vless_config());
        protocol.config = Deferred::default();
        let link = link(ProtoCoreType::Xray);
        let (params, rules, dns) = default_params();
        let err = ConfigBuilder::build(&endpoint, &link, &protocol, &params, &rules, &dns)
            .expect_err("unloaded config must be rejected");
        assert!(
            err.to_string().contains("not loaded"),
            "error must mention the unloaded config: {err}"
        );
    }

    #[test]
    fn endpoint_essentials_maps_host_type() {
        let e = endpoint("example.com", 443);
        let ess = endpoint_essentials(&e);
        assert_eq!(ess.host, "example.com");
        assert_eq!(ess.port, 443);
        assert_eq!(ess.host_type, HostKind::Dns);
        assert_eq!(ess.ports, Vec::<u16>::new());
    }

    #[test]
    fn shadowsocks_method_reads_typed_config() {
        let ss = ProtocolConfig::Ss(xray_tui_proto::proto_spec::SsConfig {
            method: "aes-256-gcm".into(),
            password: "pw".to_string(),
            security: SecurityConfig::default(),
            remarks: None,
            plugin: None,
            plugin_opts: None,
        });
        let protocol = protocol(ProtocolKind::Shadowsocks, ss);
        assert_eq!(
            shadowsocks_method(&protocol).as_deref(),
            Some("aes-256-gcm")
        );
    }

    #[test]
    fn shadowsocks_method_none_for_other_kinds() {
        let protocol = protocol(ProtocolKind::Vless, vless_config());
        assert_eq!(shadowsocks_method(&protocol), None);
    }

    #[test]
    fn build_hosts_map_parses_domain_ip_entries() {
        let hosts = build_hosts_map(&[
            "example.com:1.2.3.4".to_string(),
            "ipv6.test:2001:db8::1".to_string(),
            "malformed".to_string(),
        ]);
        assert_eq!(hosts["example.com"], "1.2.3.4");
        assert_eq!(hosts["ipv6.test"], "2001:db8::1");
        assert!(hosts.get("malformed").is_none());
    }
}
