use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use xray_tui_core::CoreType;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub gui: GuiConfig,
    #[serde(default)]
    pub inbound: InboundConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub xray_path: Option<String>,
    pub sing_box_path: Option<String>,
    pub core_type: Option<CoreType>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            xray_path: None,
            sing_box_path: None,
            core_type: None,
            log_level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    "warning".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    #[serde(default = "default_language")]
    pub language: String,
    pub theme: Option<String>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme: None,
            refresh_interval_secs: default_refresh_interval(),
        }
    }
}

fn default_language() -> String {
    "en".to_owned()
}

fn default_refresh_interval() -> u64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    #[serde(default = "default_socks_port")]
    pub socks_port: u16,
    pub http_port: Option<u16>,
    pub mixed_port: Option<u16>,
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default)]
    pub sniffing: bool,
}

impl Default for InboundConfig {
    fn default() -> Self {
        Self {
            socks_port: default_socks_port(),
            http_port: None,
            mixed_port: None,
            listen: default_listen(),
            sniffing: false,
        }
    }
}

fn default_socks_port() -> u16 {
    10808
}

fn default_listen() -> String {
    "127.0.0.1".to_owned()
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = default_config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = default_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xray-tui")
        .join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let original = AppConfig::default();
        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.core.log_level, "warning");
        assert_eq!(restored.gui.language, "en");
        assert_eq!(restored.gui.refresh_interval_secs, 5);
        assert_eq!(restored.inbound.socks_port, 10808);
        assert_eq!(restored.inbound.listen, "127.0.0.1");
        assert!(!restored.inbound.sniffing);
    }

    #[test]
    fn core_type_field_accepts_null() {
        // When core_type is absent in JSON, it should become None
        let json = r#"{"core":{},"gui":{},"inbound":{}}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert!(config.core.core_type.is_none());

        // When present and valid, it should parse
        let json = r#"{"core":{"core_type":"xray"},"gui":{},"inbound":{}}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.core.core_type, Some(CoreType::Xray));
    }
}
