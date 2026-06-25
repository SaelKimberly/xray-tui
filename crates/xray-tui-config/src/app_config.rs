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
    #[serde(default)]
    pub tun: TunConfig,
    #[serde(default)]
    pub mux: MuxConfig,
    #[serde(default)]
    pub system_proxy: SystemProxyConfig,
    #[serde(default)]
    pub statistics: StatisticsConfig,
    #[serde(default)]
    pub updates: UpdateConfig,
    #[serde(default)]
    pub speed_test: SpeedTestConfig,
    #[serde(default)]
    pub logging: LogConfig,
    #[serde(default)]
    pub parsing: ParsingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub xray_path: Option<String>,
    pub sing_box_path: Option<String>,
    pub core_type: Option<CoreType>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub protocol_core_overrides: std::collections::HashMap<String, String>,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            xray_path: None,
            sing_box_path: None,
            core_type: None,
            log_level: default_log_level(),
            protocol_core_overrides: std::collections::HashMap::new(),
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

const fn default_refresh_interval() -> u64 {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TunConfig {
    #[serde(default)]
    pub enabled: bool,
    pub interface_name: Option<String>,
    pub mtu: Option<u16>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuxConfig {
    #[serde(default)]
    pub enabled: bool,
    pub concurrency: Option<u8>,
    #[serde(default)]
    pub fragment_enabled: bool,
    pub fragment_packets: Option<String>,
    pub fragment_length: Option<String>,
    pub fragment_interval: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemProxyConfig {
    #[serde(default)]
    pub enabled: bool,
    pub http_port: Option<u16>,
    pub socks_port: Option<u16>,
    pub bypass: Option<String>,
}

const fn default_clash_api_port() -> u16 {
    9090
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsConfig {
    #[serde(default = "default_stats_enabled")]
    pub enabled: bool,
    #[serde(default = "default_clash_api_port")]
    pub clash_api_port: u16,
}

const fn default_stats_enabled() -> bool {
    true
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            enabled: default_stats_enabled(),
            clash_api_port: default_clash_api_port(),
        }
    }
}

const fn default_socks_port() -> u16 {
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
            let config = Self::default();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    #[serde(default = "default_update_check_enabled")]
    pub check_on_startup: bool,
    #[serde(default)]
    pub last_check: Option<String>,
    #[serde(default)]
    pub xray_latest_known: Option<String>,
    #[serde(default)]
    pub sing_box_latest_known: Option<String>,
}

const fn default_update_check_enabled() -> bool {
    true
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            last_check: None,
            xray_latest_known: None,
            sing_box_latest_known: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestConfig {
    #[serde(default = "default_ping_url")]
    pub ping_url: String,
    #[serde(default = "default_ip_api_url")]
    pub ip_api_url: String,
    #[serde(default = "default_tcp_timeout_secs")]
    pub tcp_timeout_secs: u64,
    #[serde(default = "default_real_ping_timeout_secs")]
    pub real_ping_timeout_secs: u64,
    #[serde(default = "default_batch_page_size")]
    pub batch_page_size: usize,
    #[serde(default = "default_batch_delay_ms")]
    pub batch_delay_ms: u64,
    #[serde(default = "default_real_ping_retries")]
    pub real_ping_retries: u32,
    #[serde(default = "default_real_ping_concurrency")]
    pub real_ping_concurrency: usize,
}

fn default_ping_url() -> String {
    "https://www.gstatic.com/generate_204".to_string()
}

fn default_ip_api_url() -> String {
    "http://ip-api.com/json/".to_string()
}

const fn default_tcp_timeout_secs() -> u64 {
    5
}

const fn default_real_ping_timeout_secs() -> u64 {
    5
}

const fn default_batch_page_size() -> usize {
    1000
}

const fn default_batch_delay_ms() -> u64 {
    1000
}

const fn default_real_ping_retries() -> u32 {
    2
}

const fn default_real_ping_concurrency() -> usize {
    5
}

impl Default for SpeedTestConfig {
    fn default() -> Self {
        Self {
            ping_url: default_ping_url(),
            ip_api_url: default_ip_api_url(),
            tcp_timeout_secs: default_tcp_timeout_secs(),
            real_ping_timeout_secs: default_real_ping_timeout_secs(),
            batch_page_size: default_batch_page_size(),
            batch_delay_ms: default_batch_delay_ms(),
            real_ping_retries: default_real_ping_retries(),
            real_ping_concurrency: default_real_ping_concurrency(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsingSettings {
    #[serde(default)]
    pub allow_private_ips: bool,
    #[serde(default)]
    pub reject_insecure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_ttl_hours")]
    pub ttl_hours: u64,
    #[serde(default = "default_log_batch_size")]
    pub batch_size: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            ttl_hours: default_log_ttl_hours(),
            batch_size: default_log_batch_size(),
        }
    }
}

const fn default_log_ttl_hours() -> u64 {
    72
}
const fn default_log_batch_size() -> usize {
    500
}
