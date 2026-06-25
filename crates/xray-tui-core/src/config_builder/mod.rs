pub mod singbox;
pub mod xray;

use crate::core_type::CoreType;
use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};

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
    pub listen: String,
    pub sniffing: bool,
}

use serde::Serialize;

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
        profile: &Profile,
        core_type: CoreType,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<BackendConfig, BuildError> {
        match core_type {
            CoreType::Xray => {
                let config = xray::XrayConfigBuilder::build(profile, params, routing, dns)?;
                Ok(BackendConfig::Xray(config))
            }
            CoreType::SingBox => {
                let config = singbox::SingBoxConfigBuilder::build(profile, params, routing, dns)?;
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

    fn test_profile(config_type: i32) -> Profile {
        let mut profile = Profile {
            id: "test-smoke".to_string(),
            config_type,
            core_type: String::new(),
            remarks: Some("smoke test".to_string()),
            address: Some("example.com".to_string()),
            port: Some(443),
            user_id: Some("test-uuid".to_string()),
            security: Some("auto".to_string()),
            network: Some("tcp".to_string()),
            stream_settings: None,
            protocol_settings: None,
            is_sub: Some(0),
            sub_id: None,
            group_id: None,
            sort_order: Some(0),
            is_active: Some(0),
            created_at: None,
            updated_at: None,
            sub_uid: None,
        };
        profile.sub_uid = Some(profile.compute_sub_uid() as i64);
        profile
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
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config = ConfigBuilder::build(&profile, CoreType::Xray, &params, &rules, &dns).unwrap();
        assert!(matches!(config, BackendConfig::Xray(_)));
    }

    #[test]
    fn build_singbox_tuic_via_dispatch() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            ConfigBuilder::build(&profile, CoreType::SingBox, &params, &rules, &dns).unwrap();
        assert!(matches!(config, BackendConfig::SingBox(_)));
    }

    #[test]
    fn build_common_protocol_forced_to_singbox() {
        // Shadowsocks is supported by both xray and sing-box builders
        let profile = test_profile(Protocol::Shadowsocks.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            ConfigBuilder::build(&profile, CoreType::SingBox, &params, &rules, &dns).unwrap();
        assert!(matches!(config, BackendConfig::SingBox(_)));
    }

    #[test]
    fn build_auto_returns_error() {
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let result = ConfigBuilder::build(&profile, CoreType::Auto, &params, &rules, &dns);
        assert!(result.is_err());
    }
}
