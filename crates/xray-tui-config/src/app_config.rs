use ratatui_themes::ThemeName;
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
    #[serde(default)]
    pub geo: GeoConfig,
    #[serde(default)]
    pub theme_name: ThemeName,
    pub clash_api_port: Option<u16>,
    #[serde(default)]
    pub clash_mixin: Option<String>,
    #[serde(default)]
    pub purgatory: PurgatoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgatoryConfig {
    #[serde(default = "default_purgatory_ttl_days")]
    pub ttl_days: u64,
    #[serde(default = "default_purgatory_retention_days")]
    pub retention_days: u64,
    #[serde(default = "default_purgatory_enabled")]
    pub enabled: bool,
}

const fn default_purgatory_ttl_days() -> u64 {
    7
}
const fn default_purgatory_retention_days() -> u64 {
    30
}
const fn default_purgatory_enabled() -> bool {
    true
}

impl Default for PurgatoryConfig {
    fn default() -> Self {
        Self {
            ttl_days: default_purgatory_ttl_days(),
            retention_days: default_purgatory_retention_days(),
            enabled: default_purgatory_enabled(),
        }
    }
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
    #[serde(default)]
    pub skip_cert_verify: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            xray_path: None,
            sing_box_path: None,
            core_type: None,
            log_level: default_log_level(),
            protocol_core_overrides: std::collections::HashMap::new(),
            skip_cert_verify: false,
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
    pub refresh_interval_secs: crate::DurationOrSecs,
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

fn default_refresh_interval() -> crate::DurationOrSecs {
    crate::DurationOrSecs::from(std::time::Duration::from_secs(5))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuxConfig {
    #[serde(default)]
    pub enabled: bool,
    pub concurrency: Option<u8>,
    #[serde(default = "default_mux_protocol")]
    pub protocol: String,
    pub max_connections: Option<u8>,
    pub min_streams: Option<u8>,
    pub max_streams: Option<u16>,
    #[serde(default)]
    pub padding: bool,
    #[serde(default)]
    pub fragment_enabled: bool,
    pub fragment_packets: Option<String>,
    pub fragment_length: Option<String>,
    pub fragment_interval: Option<String>,
}

fn default_mux_protocol() -> String {
    "smux".to_owned()
}

impl Default for MuxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            concurrency: None,
            protocol: default_mux_protocol(),
            max_connections: None,
            min_streams: None,
            max_streams: None,
            padding: false,
            fragment_enabled: false,
            fragment_packets: None,
            fragment_length: None,
            fragment_interval: None,
        }
    }
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
            let mut config: Self = serde_json::from_str(&content)?;
            // Normalize old protocol Display strings in protocol_core_overrides keys.
            // The Display impl was changed to short forms ("ss", "hy", etc.)
            // which broke backward-compat with configs saved using the old
            // kebab-case Debug forms ("shadowsocks", "hysteria", etc.).
            config.core.protocol_core_overrides = config
                .core
                .protocol_core_overrides
                .into_iter()
                .map(|(k, v)| (normalize_protocol_key(&k), v))
                .collect();
            Ok(config)
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

/// Normalize old protocol (Display) key names to current short forms.
/// The `Protocol::Display` impl was changed from kebab-case Debug-derived
/// names to short forms (e.g. "shadowsocks" → "ss", "hysteria" → "hy").
/// This maps old config-file keys to their current equivalents.
fn normalize_protocol_key(key: &str) -> String {
    match key {
        "shadowsocks" => "ss",
        "shadowsocks-2022" => "ss-2022",
        "shadowsocks-r" => "ssr",
        "hysteria" => "hy",
        "hysteria-2" => "hy2",
        "dokodemo-door" => "dokodemo",
        "wire-guard" => "wireguard",
        other => other,
    }
    .to_owned()
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
        assert_eq!(
            *restored.gui.refresh_interval_secs,
            std::time::Duration::from_secs(5)
        );
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

    #[test]
    fn normalize_protocol_key_known_pairs() {
        // Each known old Display string → current short form
        let cases = [
            ("shadowsocks", "ss"),
            ("shadowsocks-2022", "ss-2022"),
            ("shadowsocks-r", "ssr"),
            ("hysteria", "hy"),
            ("hysteria-2", "hy2"),
            ("dokodemo-door", "dokodemo"),
            ("wire-guard", "wireguard"),
        ];
        for (old, expected) in &cases {
            assert_eq!(normalize_protocol_key(old), *expected, "failed for {old}");
        }
    }

    #[test]
    fn normalize_protocol_key_passes_through_unknown() {
        // Already-short or unknown keys pass through unchanged
        let passthrough = [
            "vmess",
            "vless",
            "ss",
            "ss-2022",
            "ssr",
            "hy",
            "hy2",
            "trojan",
            "socks",
            "http",
            "dokodemo",
            "wireguard",
            "tuic",
            "naive",
            "any-tls",
            "shadow-tls",
            "tor",
            "ssh",
            "redirect",
            "t-proxy",
            "mixed",
            "tailscale",
        ];
        for key in &passthrough {
            assert_eq!(
                normalize_protocol_key(key),
                *key,
                "passthrough failed for {key}"
            );
        }
    }

    #[test]
    fn normalise_protocol_key_round_trips_config() {
        // Simulate loading a config with old keys — they should survive
        // normalization and be usable at runtime.
        let old_config_json = r#"{
            "core": {
                "protocol_core_overrides": {
                    "shadowsocks-2022": "xray",
                    "wire-guard": "sing-box",
                    "dokodemo-door": "xray"
                }
            },
            "gui": {},
            "inbound": {}
        }"#;
        let mut config: AppConfig = serde_json::from_str(old_config_json).unwrap();
        // Apply normalization (same as AppConfig::load() does)
        config.core.protocol_core_overrides = config
            .core
            .protocol_core_overrides
            .into_iter()
            .map(|(k, v)| (normalize_protocol_key(&k), v))
            .collect();
        let overrides = &config.core.protocol_core_overrides;
        assert_eq!(overrides.get("ss-2022").map(String::as_str), Some("xray"));
        assert_eq!(
            overrides.get("wireguard").map(String::as_str),
            Some("sing-box")
        );
        assert_eq!(overrides.get("dokodemo").map(String::as_str), Some("xray"));
        // Old keys should no longer exist
        assert!(overrides.get("shadowsocks-2022").is_none());
        assert!(overrides.get("wire-guard").is_none());
        assert!(overrides.get("dokodemo-door").is_none());
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
    pub tcp_timeout_secs: crate::DurationOrSecs,
    #[serde(default = "default_real_ping_timeout_secs")]
    pub real_ping_timeout_secs: crate::DurationOrSecs,
    #[serde(default = "default_batch_page_size")]
    pub batch_page_size: usize,
    #[serde(default = "default_real_ping_retries")]
    pub real_ping_retries: u32,
    #[serde(default = "default_real_ping_concurrency")]
    pub real_ping_concurrency: usize,
    #[serde(default = "default_real_ping_window")]
    pub real_ping_window: usize,
    #[serde(default = "default_fast_ping_concurrency")]
    pub fast_ping_concurrency: usize,
    #[serde(default = "default_real_ping_test_all_protocols")]
    pub real_ping_test_all_protocols: bool,
}

fn default_ping_url() -> String {
    "https://www.gstatic.com/generate_204".to_string()
}

fn default_ip_api_url() -> String {
    "https://ip-api.com/json/".to_string()
}

fn default_tcp_timeout_secs() -> crate::DurationOrSecs {
    crate::DurationOrSecs::from(std::time::Duration::from_secs(5))
}

fn default_real_ping_timeout_secs() -> crate::DurationOrSecs {
    crate::DurationOrSecs::from(std::time::Duration::from_secs(5))
}

const fn default_batch_page_size() -> usize {
    1000
}

const fn default_real_ping_retries() -> u32 {
    2
}

const fn default_real_ping_concurrency() -> usize {
    100
}

const fn default_real_ping_window() -> usize {
    20
}

const fn default_fast_ping_concurrency() -> usize {
    200
}

const fn default_real_ping_test_all_protocols() -> bool {
    false
}

impl Default for SpeedTestConfig {
    fn default() -> Self {
        Self {
            ping_url: default_ping_url(),
            ip_api_url: default_ip_api_url(),
            tcp_timeout_secs: default_tcp_timeout_secs(),
            real_ping_timeout_secs: default_real_ping_timeout_secs(),
            batch_page_size: default_batch_page_size(),
            real_ping_retries: default_real_ping_retries(),
            real_ping_concurrency: default_real_ping_concurrency(),
            real_ping_window: default_real_ping_window(),
            fast_ping_concurrency: default_fast_ping_concurrency(),
            real_ping_test_all_protocols: default_real_ping_test_all_protocols(),
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
    #[serde(default = "default_log_ttl_secs")]
    pub ttl_secs: crate::DurationOrSecs,
    #[serde(default)]
    pub log_to_file: bool,
    #[serde(default = "default_log_file_path")]
    pub log_file_path: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_log_ttl_secs(),
            log_to_file: false,
            log_file_path: default_log_file_path(),
        }
    }
}

fn default_log_ttl_secs() -> crate::DurationOrSecs {
    crate::DurationOrSecs::from(std::time::Duration::from_hours(72))
}

fn default_log_file_path() -> String {
    "xray-tui.log".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoConfig {
    #[serde(default = "default_geoip_url")]
    pub geoip_url: String,
    #[serde(default = "default_geosite_url")]
    pub geosite_url: String,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_geo_update_interval")]
    pub update_interval_hours: u64,
}

impl Default for GeoConfig {
    fn default() -> Self {
        Self {
            geoip_url: default_geoip_url(),
            geosite_url: default_geosite_url(),
            auto_update: false,
            update_interval_hours: 24,
        }
    }
}

fn default_geoip_url() -> String {
    "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat".into()
}

fn default_geosite_url() -> String {
    "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat".into()
}

const fn default_geo_update_interval() -> u64 {
    24
}
