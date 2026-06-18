pub mod singbox;
pub mod xray;

use crate::core_type::CoreType;
use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};


/// Port for the gRPC Stats API (shared by xray-core and sing-box).
pub const API_PORT: u16 = 62789;
/// Parameters extracted from app config for building backend configs.
/// Avoids a circular dependency on `xray-tui-config`.
#[derive(Debug, Clone)]
pub struct BuildParams {
    pub log_level: String,
    pub socks_port: u16,
    pub http_port: Option<u16>,
    pub listen: String,
    pub sniffing: bool,
}

#[derive(Debug, Clone)]
pub enum BackendConfig {
    Xray(xray::XrayConfig),
    SingBox(singbox::SingBoxConfig),
}

impl BackendConfig {
    pub fn core_type(&self) -> CoreType {
        match self {
            BackendConfig::Xray(_) => CoreType::Xray,
            BackendConfig::SingBox(_) => CoreType::SingBox,
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
