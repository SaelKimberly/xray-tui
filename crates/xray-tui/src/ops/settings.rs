use std::cell::RefCell;
use std::collections::HashMap;

use xray_tui_core::CoreType;
use xray_tui_db::models::{DnsSetting, RoutingRule};
use xray_tui_db::Database;
use ratatui_cheese::tree::TreeState;

use crate::AppState;
use crate::types::*;
use crate::{get_field, try_send_or_warn};

pub fn cancel_form(state: &mut AppState) {
    state.mode = AppMode::List;
}

pub fn enter_settings(state: &mut AppState) {
    state.mode = AppMode::Settings {
        mode: SettingsMode::Split {
            tree: RefCell::new(TreeState::all_expanded(5)),
            focus: SplitFocus::Tree,
            right: SplitRightPane::Empty,
        },
    };
}

pub async fn build_settings_fields(state: &AppState, section: SettingsSection) -> Vec<(String, String)> {
    use crate::SettingsSection::{
        Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
        SystemProxy, Tun, Updates,
    };
    match section {
        Core => {
            vec![
                ("xray_path".into(), state.config.core.xray_path.clone().unwrap_or_default()),
                ("sing_box_path".into(), state.config.core.sing_box_path.clone().unwrap_or_default()),
                ("default_core".into(), state.config.core.core_type.map(|c| c.to_string()).unwrap_or_else(|| "Auto".into())),
                ("log_level".into(), state.config.core.log_level.clone()),
            ]
        }
        Gui => {
            vec![
                ("language".into(), state.config.gui.language.clone()),
                ("theme".into(), state.config.gui.theme.clone().unwrap_or_default()),
                ("refresh_interval".into(), humantime::format_duration(*state.config.gui.refresh_interval_secs).to_string()),
            ]
        }
        Inbound => {
            vec![
                ("socks_port".into(), state.config.inbound.socks_port.to_string()),
                ("http_port".into(), state.config.inbound.http_port.map(|p| p.to_string()).unwrap_or_default()),
                ("mixed_port".into(), state.config.inbound.mixed_port.map(|p| p.to_string()).unwrap_or_default()),
                ("listen".into(), state.config.inbound.listen.clone()),
                ("sniffing".into(), state.config.inbound.sniffing.to_string()),
            ]
        }
        SystemProxy => {
            vec![
                ("enabled".into(), state.config.system_proxy.enabled.to_string()),
                ("http_port".into(), state.config.system_proxy.http_port.map(|p| p.to_string()).unwrap_or_default()),
                ("socks_port".into(), state.config.system_proxy.socks_port.map(|p| p.to_string()).unwrap_or_default()),
                ("bypass".into(), state.config.system_proxy.bypass.clone().unwrap_or_default()),
            ]
        }
        Tun => {
            vec![
                ("enabled".into(), state.config.tun.enabled.to_string()),
                ("interface_name".into(), state.config.tun.interface_name.clone().unwrap_or_default()),
                ("mtu".into(), state.config.tun.mtu.map(|m| m.to_string()).unwrap_or_default()),
            ]
        }
        Mux => {
            vec![
                ("enabled".into(), state.config.mux.enabled.to_string()),
                ("concurrency".into(), state.config.mux.concurrency.map(|c| c.to_string()).unwrap_or_default()),
                ("fragment_enabled".into(), state.config.mux.fragment_enabled.to_string()),
                ("fragment_packets".into(), state.config.mux.fragment_packets.clone().unwrap_or_default()),
                ("fragment_length".into(), state.config.mux.fragment_length.clone().unwrap_or_default()),
                ("fragment_interval".into(), state.config.mux.fragment_interval.clone().unwrap_or_default()),
            ]
        }
        Stats => {
            vec![
                ("enabled".into(), state.config.statistics.enabled.to_string()),
            ]
        }
        ProtocolCore => state.config.core.protocol_core_overrides
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        SpeedTest => {
            vec![
                ("ping_url".into(), state.config.speed_test.ping_url.clone()),
                ("ip_api_url".into(), state.config.speed_test.ip_api_url.clone()),
                ("tcp_timeout_secs".into(), humantime::format_duration(*state.config.speed_test.tcp_timeout_secs).to_string()),
                ("real_ping_timeout_secs".into(), humantime::format_duration(*state.config.speed_test.real_ping_timeout_secs).to_string()),
                ("batch_page_size".into(), state.config.speed_test.batch_page_size.to_string()),
                ("real_ping_retries".into(), state.config.speed_test.real_ping_retries.to_string()),
                ("real_ping_concurrency".into(), state.config.speed_test.real_ping_concurrency.to_string()),
                ("tcp_ping_concurrency".into(), state.config.speed_test.tcp_ping_concurrency.to_string()),
            ]
        }
        Dns => {
            if let Ok(Some(dns)) = state.db.get_dns_settings().await {
                vec![
                    ("servers".into(), dns.servers.unwrap_or_default()),
                    ("hosts".into(), dns.hosts.unwrap_or_default()),
                    ("query_strategy".into(), dns.query_strategy.unwrap_or_default()),
                    ("disable_cache".into(), dns.disable_cache.map(|v| v.to_string()).unwrap_or_default()),
                    ("disable_fallback".into(), dns.disable_fallback.map(|v| v.to_string()).unwrap_or_default()),
                    ("client_ip".into(), dns.client_ip.unwrap_or_default()),
                ]
            } else {
                vec![
                    ("servers".into(), String::new()),
                    ("hosts".into(), String::new()),
                    ("query_strategy".into(), String::new()),
                    ("disable_cache".into(), String::new()),
                    ("disable_fallback".into(), String::new()),
                    ("client_ip".into(), String::new()),
                ]
            }
        }
        Updates => {
            vec![
                ("check_on_startup".into(), state.config.updates.check_on_startup.to_string()),
            ]
        }
        Routing => vec![],
        Logging => {
            vec![
                ("log_ttl_secs".into(), humantime::format_duration(*state.config.logging.ttl_secs).to_string()),
            ]
        }
    }
}

fn apply_settings_fields(state: &mut AppState, section: SettingsSection, fields: &[(String, String)]) {
    use crate::SettingsSection::{
        Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
        SystemProxy, Tun, Updates,
    };
    let get_str = |key: &str| -> &str {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str())
    };
    let get = |key: &str| get_str(key).to_owned();
    let get_opt = |key: &str| {
        let v = get_str(key);
        if v.is_empty() {
            None
        } else {
            Some(v.to_owned())
        }
    };
    match section {
        Core => {
            state.config.core.xray_path = get_opt("xray_path");
            state.config.core.sing_box_path = get_opt("sing_box_path");
            let core_str = get_str("default_core");
            state.config.core.core_type = if core_str.is_empty() || core_str == "Auto" {
                None
            } else {
                core_str.parse::<xray_tui_core::CoreType>().ok()
            };
            if !get_str("log_level").is_empty() {
                state.config.core.log_level = get("log_level");
            }
        }
        Gui => {
            state.config.gui.language = get("language");
            state.config.gui.theme = get_opt("theme");
            if let Ok(d) = humantime::parse_duration(get_str("refresh_interval")) {
                *state.config.gui.refresh_interval_secs = d;
            }
        }
        Inbound => {
            if let Ok(v) = get_str("socks_port").parse::<u16>() {
                state.config.inbound.socks_port = v;
            }
            state.config.inbound.http_port = get_str("http_port").parse::<u16>().ok();
            state.config.inbound.mixed_port = get_str("mixed_port").parse::<u16>().ok();
            if !get_str("listen").is_empty() {
                state.config.inbound.listen = get("listen");
            }
            state.config.inbound.sniffing = get_str("sniffing") == "true";
        }
        SystemProxy => {
            state.config.system_proxy.enabled = get_str("enabled") == "true";
            state.config.system_proxy.http_port = get_str("http_port").parse::<u16>().ok();
            state.config.system_proxy.socks_port = get_str("socks_port").parse::<u16>().ok();
            state.config.system_proxy.bypass = get_opt("bypass");
        }
        Tun => {
            state.config.tun.enabled = get_str("enabled") == "true";
            state.config.tun.interface_name = get_opt("interface_name");
            state.config.tun.mtu = get_str("mtu").parse::<u16>().ok();
        }
        Mux => {
            state.config.mux.enabled = get_str("enabled") == "true";
            state.config.mux.concurrency = get_str("concurrency").parse::<u8>().ok();
            state.config.mux.fragment_enabled = get_str("fragment_enabled") == "true";
            state.config.mux.fragment_packets = get_opt("fragment_packets");
            state.config.mux.fragment_length = get_opt("fragment_length");
            state.config.mux.fragment_interval = get_opt("fragment_interval");
        }
        Stats => {
            state.config.statistics.enabled = get_str("enabled") == "true";
        }
        ProtocolCore => {
            for (key, val) in fields {
                if val == "Auto" {
                    state.config
                        .core
                        .protocol_core_overrides
                        .remove(key.as_str());
                } else {
                    state.config
                        .core
                        .protocol_core_overrides
                        .insert(key.clone(), val.clone());
                }
            }
        }
        SpeedTest => {
            if !get_str("ping_url").is_empty() {
                state.config.speed_test.ping_url = get("ping_url");
            }
            if !get_str("ip_api_url").is_empty() {
                state.config.speed_test.ip_api_url = get("ip_api_url");
            }
            if let Ok(d) = humantime::parse_duration(get_str("tcp_timeout_secs")) {
                *state.config.speed_test.tcp_timeout_secs = d;
            }
            if let Ok(d) = humantime::parse_duration(get_str("real_ping_timeout_secs")) {
                *state.config.speed_test.real_ping_timeout_secs = d;
            }
            if let Ok(v) = get_str("batch_page_size").parse::<usize>() {
                state.config.speed_test.batch_page_size = v;
            }
            if let Ok(v) = get_str("real_ping_retries").parse::<u32>() {
                state.config.speed_test.real_ping_retries = v;
            }
            if let Ok(v) = get_str("real_ping_concurrency").parse::<usize>() {
                state.config.speed_test.real_ping_concurrency = v;
            }
            if let Ok(v) = get_str("tcp_ping_concurrency").parse::<usize>() {
                state.config.speed_test.tcp_ping_concurrency = v.max(1);
            }
        }
        // Dns and Routing are handled separately (DB-backed)
        Dns | Routing | Updates => {}
        Logging => {
            if let Ok(d) = humantime::parse_duration(get_str("log_ttl_secs")) {
                *state.config.logging.ttl_secs = d;
            }
        }
    }
}

pub async fn build_right_pane(state: &mut AppState, section: SettingsSection) -> SplitRightPane {
    let fields = build_settings_fields(state, section).await;
    match section {
        SettingsSection::Routing => {
            state.reload_routing_rules().await;
            SplitRightPane::RoutingList { selected: 0 }
        }
        SettingsSection::Updates => SplitRightPane::UpdateForm {
            status_xray: state
                .update_status
                .get(&CoreType::Xray)
                .cloned()
                .unwrap_or_default(),
            status_singbox: state
                .update_status
                .get(&CoreType::SingBox)
                .cloned()
                .unwrap_or_default(),
        },
        _ => SplitRightPane::Form {
            section,
            fields,
            focus_index: 0,
            form_errors: HashMap::new(),
        },
    }
}

pub fn save_settings_form(state: &mut AppState, section: SettingsSection, fields: &[(String, String)]) {
    apply_settings_fields(state, section, fields);
    if let Err(e) = state.config.save() {
        state.log_trace("error", "tui", &format!("Failed to save config: {e}"));
    } else {
        state.log_trace("info", "tui", "Settings saved");
    }
}

pub async fn save_routing_rule(
    state: &mut AppState,
    rule_id: Option<String>,
    fields: &[(String, String)],
) {
    let id = rule_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let get_str = |key: &str| -> &str {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str())
    };
    let get_opt = |key: &str| {
        let v = get_str(key);
        if v.is_empty() {
            None
        } else {
            Some(v.to_owned())
        }
    };
    let rule = RoutingRule {
        id,
        group_id: None,
        r#type: get_str("type").parse::<i32>().unwrap_or(0),
        domain_matcher: get_opt("domain_matcher"),
        domains: get_opt("domains"),
        ips: get_opt("ips"),
        inbound_tags: get_opt("inbound_tags"),
        port: get_opt("port"),
        source_ports: get_opt("source_ports"),
        network: get_opt("network"),
        protocols: get_opt("protocols"),
        domain_strategy: get_opt("domain_strategy"),
        outbound_tag: get_opt("outbound_tag"),
        balancer_tag: get_opt("balancer_tag"),
        rule_set_file: get_opt("rule_set_file"),
        rule_set_url: get_opt("rule_set_url"),
        sort_order: None,
    };
    let result = if rule_id.is_some() {
        state.db.update_routing_rule(&rule).await
    } else {
        state.db.insert_routing_rule(&rule).await
    };
    match result {
        Ok(()) => state.log_trace("info", "tui", "Routing rule saved"),
        Err(e) => state.log_trace("error", "tui", &format!("Failed to save routing rule: {e}")),
    }
    state.reload_routing_rules().await;
}

pub async fn save_dns_settings(state: &mut AppState, fields: &[(String, String)]) {
    let id = state
        .db
        .get_dns_settings()
        .await
        .ok()
        .flatten()
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), |d| d.id);
    let get_str = |key: &str| -> &str {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str())
    };
    let get_opt = |key: &str| {
        let v = get_str(key);
        if v.is_empty() {
            None
        } else {
            Some(v.to_owned())
        }
    };
    let dns = DnsSetting {
        id,
        name: None,
        servers: get_opt("servers"),
        hosts: get_opt("hosts"),
        query_strategy: get_opt("query_strategy"),
        disable_cache: Some(i32::from(get_str("disable_cache") == "true")),
        disable_fallback: Some(i32::from(get_str("disable_fallback") == "true")),
        client_ip: get_opt("client_ip"),
    };
    match state.db.upsert_dns_settings(&dns).await {
        Ok(()) => state.log_trace("info", "tui", "DNS settings saved"),
        Err(e) => state.log_trace("error", "tui", &format!("Failed to save DNS settings: {e}")),
    }
}
