pub mod singbox;
pub mod xray;

pub mod clash_mixin;
use crate::core_type::CoreType;
use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow, RoutingRule};

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

use serde::Serialize;
use serde_json::{Value, json};

/// Shared: extract (p_settings, s_settings) from a protocol row's spec_blob.
/// Tries typed ProtocolConfig first; falls back to raw JSON extraction.
pub(crate) fn parse_settings(protocol: &ProtocolRow) -> (Value, Value) {
    if let Ok(config) =
        serde_json::from_slice::<xray_tui_proto::proto_spec::ProtocolConfig>(&protocol.spec_blob)
    {
        return config.to_settings();
    }
    let extra: Value = serde_json::from_slice(&protocol.spec_blob).unwrap_or_else(|_| json!({}));
    let mut p_settings = extra.get("protocol_settings").cloned().unwrap_or(json!({}));
    // Inject `user_id` as `id` into p_settings if absent (legacy parsers store
    // UUID/PW at top level, not inside protocol_settings).
    if let Some(user_id) = extra.get("user_id").and_then(|v| v.as_str()) {
        if let Some(obj) = p_settings.as_object_mut() {
            if !obj.contains_key("id") && !obj.contains_key("uuid") {
                obj.entry("id".to_string())
                    .or_insert(serde_json::Value::String(user_id.to_string()));
            }
        }
    }
    let s_settings = extra.get("stream_settings").cloned().unwrap_or(json!({}));
    (p_settings, s_settings)
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
/// Bundles the endpoint, protocol, and the pre-assigned SOCKS5 port.
#[derive(Debug, Clone)]
pub struct MultiInboundItem<'a> {
    pub endpoint: &'a Endpoint,
    pub protocol: &'a ProtocolRow,
    pub assigned_port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct ConfigBuilder;

impl ConfigBuilder {
    pub fn build(
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        core_type: CoreType,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError> {
        match core_type {
            CoreType::Xray => {
                let config =
                    xray::XrayConfigBuilder::build(endpoint, protocol, params, routing, dns)?;
                Ok(BackendConfig::Xray(config))
            }
            CoreType::SingBox => {
                let config =
                    singbox::SingBoxConfigBuilder::build(endpoint, protocol, params, routing, dns)?;
                Ok(BackendConfig::SingBox(config))
            }
            CoreType::Auto => Err(BuildError::InvalidProfile(
                "Auto core type must be resolved before building config".to_string(),
            )),
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
    pub fn build_multi(
        items: &[MultiInboundItem],
        core_type: CoreType,
        base_params: &BuildParams,
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError> {
        match core_type {
            CoreType::Xray => {
                let config = xray::XrayConfigBuilder::build_multi(items, base_params, dns)?;
                Ok(BackendConfig::Xray(config))
            }
            CoreType::SingBox => {
                let config =
                    singbox::SingBoxConfigBuilder::build_multi(items, base_params, dns)?;
                Ok(BackendConfig::SingBox(config))
            }
            CoreType::Auto => Err(BuildError::InvalidProfile(
                "Auto core type must be resolved before building config".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Protocol;

    fn test_endpoint_and_protocol(config_type: i32) -> (Endpoint, ProtocolRow) {
        let endpoint = Endpoint {
            id: 0,
            host: "example.com".to_string(),
            host_type: "dns".to_string(),
            port: 443,
            port_spec_str: None,
            parent_id: None,
            last_source: None,
            created_at: 0,
            manual_protocol_override: None,
        };
        let extra = serde_json::json!({
            "remarks": "smoke test",
            "user_id": "test-uuid",
        });
        let protocol = ProtocolRow {
            id: 0,
            endpoint_id: 0,
            sig: 0,
            cred_hash: 0,
            proto_kind: String::new(),
            spec_blob: serde_json::to_vec(&extra).unwrap_or_default(),
            config_type,
            core_type: String::new(),
            transport: Some("tcp".to_string()),
            security: Some("auto".to_string()),
            remarks: None,
            created_at: 0,
            last_seen_at: 0,
            extension: Default::default(),
            endpoint: Default::default(),
            server_stat: Default::default(),
        };
        (endpoint, protocol)
    }

    fn default_params() -> (BuildParams, Vec<RoutingRule>, DnsSetting) {
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
            servers: None,
            hosts: None,
            query_strategy: None,
            disable_cache: None,
            disable_fallback: None,
            client_ip: None,
        };
        (params, rules, dns)
    }

    #[test]
    fn build_xray_via_dispatch() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            ConfigBuilder::build(&endpoint, &protocol, CoreType::Xray, &params, &rules, &dns)
                .unwrap();
        assert!(matches!(config, BackendConfig::Xray(_)));
    }

    #[test]
    fn build_singbox_tuic_via_dispatch() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config = ConfigBuilder::build(
            &endpoint,
            &protocol,
            CoreType::SingBox,
            &params,
            &rules,
            &dns,
        )
        .unwrap();
        assert!(matches!(config, BackendConfig::SingBox(_)));
    }

    #[test]
    fn build_common_protocol_forced_to_singbox() {
        // Shadowsocks is supported by both xray and sing-box builders
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Shadowsocks.to_i32());
        let (params, rules, dns) = default_params();
        let config = ConfigBuilder::build(
            &endpoint,
            &protocol,
            CoreType::SingBox,
            &params,
            &rules,
            &dns,
        )
        .unwrap();
        assert!(matches!(config, BackendConfig::SingBox(_)));
    }

    #[test]
    fn build_auto_returns_error() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let result =
            ConfigBuilder::build(&endpoint, &protocol, CoreType::Auto, &params, &rules, &dns);
        assert!(result.is_err());
    }
}
