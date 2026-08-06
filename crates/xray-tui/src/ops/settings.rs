use std::cell::RefCell;
use std::collections::HashMap;

use ratatui_cheese::tree::TreeState;
use xray_tui_core::CoreType;
use xray_tui_db::models::RoutingRule;

use crate::AppState;
use crate::types::{AppMode, SettingsMode, SettingsSection, SplitFocus, SplitRightPane};

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

pub async fn build_settings_fields(
    state: &AppState,
    section: SettingsSection,
) -> Vec<(String, String)> {
    use crate::SettingsSection::{
        Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
        Subscriptions, SystemProxy, Tun, Updates,
    };
    match section {
        Core => {
            vec![
                (
                    "xray_path".into(),
                    state.config.core.xray_path.clone().unwrap_or_default(),
                ),
                (
                    "sing_box_path".into(),
                    state.config.core.sing_box_path.clone().unwrap_or_default(),
                ),
                (
                    "default_core".into(),
                    state
                        .config
                        .core
                        .core_type
                        .map_or_else(|| "Auto".into(), |c| c.to_string()),
                ),
                ("log_level".into(), state.config.core.log_level.clone()),
                (
                    "skip_cert_verify".into(),
                    state.config.core.skip_cert_verify.to_string(),
                ),
                (
                    "clash_mixin".into(),
                    state.config.clash_mixin.clone().unwrap_or_default(),
                ),
            ]
        }
        Gui => {
            vec![
                ("language".into(), state.config.gui.language.clone()),
                (
                    "theme".into(),
                    state.config.gui.theme.clone().unwrap_or_default(),
                ),
                (
                    "refresh_interval".into(),
                    humantime::format_duration(*state.config.gui.refresh_interval_secs).to_string(),
                ),
            ]
        }
        Inbound => {
            vec![
                (
                    "socks_port".into(),
                    state.config.inbound.socks_port.to_string(),
                ),
                (
                    "http_port".into(),
                    state
                        .config
                        .inbound
                        .http_port
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "mixed_port".into(),
                    state
                        .config
                        .inbound
                        .mixed_port
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                ),
                ("listen".into(), state.config.inbound.listen.clone()),
                ("sniffing".into(), state.config.inbound.sniffing.to_string()),
            ]
        }
        SystemProxy => {
            vec![
                (
                    "enabled".into(),
                    state.config.system_proxy.enabled.to_string(),
                ),
                (
                    "http_port".into(),
                    state
                        .config
                        .system_proxy
                        .http_port
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "socks_port".into(),
                    state
                        .config
                        .system_proxy
                        .socks_port
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "bypass".into(),
                    state.config.system_proxy.bypass.clone().unwrap_or_default(),
                ),
            ]
        }
        Tun => {
            vec![
                ("enabled".into(), state.config.tun.enabled.to_string()),
                (
                    "interface_name".into(),
                    state.config.tun.interface_name.clone().unwrap_or_default(),
                ),
                (
                    "mtu".into(),
                    state
                        .config
                        .tun
                        .mtu
                        .map(|m| m.to_string())
                        .unwrap_or_default(),
                ),
            ]
        }
        Mux => {
            vec![
                ("enabled".into(), state.config.mux.enabled.to_string()),
                (
                    "concurrency".into(),
                    state
                        .config
                        .mux
                        .concurrency
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                ),
                ("protocol".into(), state.config.mux.protocol.clone()),
                (
                    "max_connections".into(),
                    state
                        .config
                        .mux
                        .max_connections
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "min_streams".into(),
                    state
                        .config
                        .mux
                        .min_streams
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "max_streams".into(),
                    state
                        .config
                        .mux
                        .max_streams
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                ),
                ("padding".into(), state.config.mux.padding.to_string()),
                (
                    "fragment_enabled".into(),
                    state.config.mux.fragment_enabled.to_string(),
                ),
                (
                    "fragment_packets".into(),
                    state
                        .config
                        .mux
                        .fragment_packets
                        .clone()
                        .unwrap_or_default(),
                ),
                (
                    "fragment_length".into(),
                    state.config.mux.fragment_length.clone().unwrap_or_default(),
                ),
                (
                    "fragment_interval".into(),
                    state
                        .config
                        .mux
                        .fragment_interval
                        .clone()
                        .unwrap_or_default(),
                ),
            ]
        }
        Stats => {
            vec![(
                "enabled".into(),
                state.config.statistics.enabled.to_string(),
            )]
        }
        ProtocolCore => state
            .config
            .core
            .protocol_core_overrides
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        SpeedTest => {
            vec![
                ("ping_url".into(), state.config.speed_test.ping_url.clone()),
                (
                    "ip_api_url".into(),
                    state.config.speed_test.ip_api_url.clone(),
                ),
                (
                    "tcp_timeout_secs".into(),
                    humantime::format_duration(*state.config.speed_test.tcp_timeout_secs)
                        .to_string(),
                ),
                (
                    "real_ping_timeout_secs".into(),
                    humantime::format_duration(*state.config.speed_test.real_ping_timeout_secs)
                        .to_string(),
                ),
                (
                    "batch_page_size".into(),
                    state.config.speed_test.batch_page_size.to_string(),
                ),
                (
                    "real_ping_retries".into(),
                    state.config.speed_test.real_ping_retries.to_string(),
                ),
                (
                    "real_ping_concurrency".into(),
                    state.config.speed_test.real_ping_concurrency.to_string(),
                ),
                (
                    "real_ping_window".into(),
                    state.config.speed_test.real_ping_window.to_string(),
                ),
                (
                    "fast_ping_concurrency".into(),
                    state.config.speed_test.fast_ping_concurrency.to_string(),
                ),
                (
                    "real_ping_test_all_protocols".into(),
                    state
                        .config
                        .speed_test
                        .real_ping_test_all_protocols
                        .to_string(),
                ),
                ("geoip_url".into(), state.config.geo.geoip_url.clone()),
                ("geosite_url".into(), state.config.geo.geosite_url.clone()),
                (
                    "geo_auto_update".into(),
                    state.config.geo.auto_update.to_string(),
                ),
                (
                    "geo_update_interval".into(),
                    state.config.geo.update_interval_hours.to_string(),
                ),
            ]
        }
        Dns => {
            if let Ok(Some(dns)) = state.db.get_dns_settings().await {
                vec![
                    // Vec fields render comma-joined in the form; split back
                    // on save (join/split at the form boundary).
                    ("servers".into(), dns.servers.join(",")),
                    ("hosts".into(), dns.hosts.join(",")),
                    (
                        "query_strategy".into(),
                        match dns.query_strategy {
                            Some(xray_tui_db::models::QueryStrategy::UseIp) => "UseIP".into(),
                            Some(xray_tui_db::models::QueryStrategy::UseIpv4) => "UseIPv4".into(),
                            Some(xray_tui_db::models::QueryStrategy::UseIpv6) => "UseIPv6".into(),
                            None => String::new(),
                        },
                    ),
                    ("disable_cache".into(), dns.disable_cache.to_string()),
                    ("disable_fallback".into(), dns.disable_fallback.to_string()),
                    ("client_ip".into(), dns.client_ip.unwrap_or_default()),
                    (
                        "cache_ttl_secs".into(),
                        dns.cache_ttl_secs
                            .map_or_else(String::new, |t| t.to_string()),
                    ),
                ]
            } else {
                vec![
                    ("servers".into(), String::new()),
                    ("hosts".into(), String::new()),
                    ("query_strategy".into(), String::new()),
                    ("disable_cache".into(), String::new()),
                    ("disable_fallback".into(), String::new()),
                    ("client_ip".into(), String::new()),
                    ("cache_ttl_secs".into(), String::new()),
                ]
            }
        }
        Updates => {
            vec![(
                "check_on_startup".into(),
                state.config.updates.check_on_startup.to_string(),
            )]
        }
        Routing | Subscriptions => vec![],
        Logging => {
            vec![
                (
                    "log_ttl_secs".into(),
                    humantime::format_duration(*state.config.logging.ttl_secs).to_string(),
                ),
                (
                    "log_to_file".into(),
                    state.config.logging.log_to_file.to_string(),
                ),
                (
                    "log_file_path".into(),
                    state.config.logging.log_file_path.clone(),
                ),
            ]
        }
    }
}

fn apply_settings_fields(
    state: &mut AppState,
    section: SettingsSection,
    fields: &[(String, String)],
) {
    use crate::SettingsSection::{
        Core, Dns, Gui, Inbound, Logging, Mux, ProtocolCore, Routing, SpeedTest, Stats,
        Subscriptions, SystemProxy, Tun, Updates,
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
            state.config.core.skip_cert_verify = get_str("skip_cert_verify") == "true";
            state.config.clash_mixin = {
                let v = get_str("clash_mixin");
                if v.is_empty() {
                    None
                } else {
                    Some(v.to_owned())
                }
            };
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
            if !get_str("protocol").is_empty() {
                state.config.mux.protocol = get("protocol");
            }
            state.config.mux.max_connections = get_str("max_connections").parse::<u8>().ok();
            state.config.mux.min_streams = get_str("min_streams").parse::<u8>().ok();
            state.config.mux.max_streams = get_str("max_streams").parse::<u16>().ok();
            state.config.mux.padding = get_str("padding") == "true";
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
                    state
                        .config
                        .core
                        .protocol_core_overrides
                        .remove(key.as_str());
                } else {
                    state
                        .config
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
            if let Ok(v) = get_str("real_ping_window").parse::<usize>() {
                state.config.speed_test.real_ping_window = v;
            }
            if let Ok(v) = get_str("fast_ping_concurrency").parse::<usize>() {
                state.config.speed_test.fast_ping_concurrency = v;
            }
            state.config.speed_test.real_ping_test_all_protocols =
                get_str("real_ping_test_all_protocols") == "true";
            if !get_str("geoip_url").is_empty() {
                state.config.geo.geoip_url = get("geoip_url");
            }
            if !get_str("geosite_url").is_empty() {
                state.config.geo.geosite_url = get("geosite_url");
            }
            state.config.geo.auto_update = get_str("geo_auto_update") == "true";
            if let Ok(v) = get_str("geo_update_interval").parse::<u64>() {
                state.config.geo.update_interval_hours = v;
            }
        }
        // Dns and Routing are handled separately (DB-backed)
        Dns | Routing | Updates | Subscriptions => {}
        // Dns and Routing are handled separately (DB-backed)
        Logging => {
            if let Ok(d) = humantime::parse_duration(get_str("log_ttl_secs")) {
                *state.config.logging.ttl_secs = d;
            }
            state.config.logging.log_to_file = get_str("log_to_file") == "true";
            if !get_str("log_file_path").is_empty() {
                state.config.logging.log_file_path = get("log_file_path");
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
        SettingsSection::Subscriptions => SplitRightPane::GroupList {
            selected: 0,
            selected_mask: vec![false; state.groups.len()],
        },
        _ => SplitRightPane::Form {
            section,
            fields,
            focus_index: 0,
            form_errors: HashMap::new(),
        },
    }
}

pub fn save_settings_form(
    state: &mut AppState,
    section: SettingsSection,
    fields: &[(String, String)],
) {
    apply_settings_fields(state, section, fields);
    if let Err(e) = state.config.save() {
        state.log_trace(
            "error",
            "tui::ops::settings",
            &format!("Failed to save config: {e}"),
        );
    } else {
        state.log_trace("info", "tui::ops::settings", "Settings saved");
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
    // Vec fields are comma-joined in the form; split at the form boundary.
    // `ports`/`source_ports` are u16 lists; the other lists are strings.
    let split = |v: &str| -> Vec<String> {
        v.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    };
    let split_u16 = |v: &str| -> Vec<u16> {
        v.split(',')
            .filter_map(|t| t.trim().parse::<u16>().ok())
            .collect()
    };
    let ports = split_u16(get_str("ports"));
    let source_ports = split_u16(get_str("source_ports"));
    let rule = RoutingRule {
        id,
        group_id: None,
        r#type: get_str("type").parse::<i32>().unwrap_or(0),
        domain_matcher: get_opt("domain_matcher"),
        domains: split(get_str("domains")),
        ips: split(get_str("ips")),
        inbound_tags: split(get_str("inbound_tags")),
        ports,
        source_ports,
        network: get_opt("network"),
        protocols: split(get_str("protocols")),
        domain_strategy: get_opt("domain_strategy"),
        outbound_tag: get_opt("outbound_tag"),
        balancer_tag: get_opt("balancer_tag"),
        rule_set_file: get_opt("rule_set_file"),
        rule_set_url: get_opt("rule_set_url"),
        sort_order: None,
    };
    // rule_set_file/rule_set_url count as match conditions at save time, but
    // neither core builder emits them yet (xray has no rule-set support;
    // sing-box rule_set emission is intentionally out of scope for now). The
    // fields remain persisted for future sing-box rule-set support.
    let has_matcher = !rule.domains.is_empty()
        || !rule.ips.is_empty()
        || !rule.inbound_tags.is_empty()
        || !rule.ports.is_empty()
        || !rule.source_ports.is_empty()
        || rule.network.is_some()
        || !rule.protocols.is_empty()
        || rule.rule_set_file.is_some()
        || rule.rule_set_url.is_some();
    if !has_matcher {
        state.log_trace(
            "error",
            "tui::ops::settings",
            "Routing rule needs at least one match condition",
        );
        return;
    }
    // The typed model has no dedicated routing-rule write; upsert through the
    // pooled connection (upsert is idempotent for add and edit alike).
    let result: Result<(), String> = async {
        let mut conn = state.db.connection().await.map_err(|e| e.to_string())?;
        xray_tui_db::models::RoutingRule::upsert_by_id(rule.id.clone())
            .group_id(rule.group_id.clone())
            .r#type(rule.r#type)
            .domain_matcher(rule.domain_matcher.clone())
            .domains(rule.domains.clone())
            .ips(rule.ips.clone())
            .inbound_tags(rule.inbound_tags.clone())
            .ports(rule.ports.clone())
            .source_ports(rule.source_ports.clone())
            .network(rule.network.clone())
            .protocols(rule.protocols.clone())
            .domain_strategy(rule.domain_strategy.clone())
            .outbound_tag(rule.outbound_tag.clone())
            .balancer_tag(rule.balancer_tag.clone())
            .rule_set_file(rule.rule_set_file.clone())
            .rule_set_url(rule.rule_set_url.clone())
            .sort_order(rule.sort_order)
            .exec(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => state.log_trace("info", "tui::ops::settings", "Routing rule saved"),
        Err(e) => state.log_trace(
            "error",
            "tui::ops::settings",
            &format!("Failed to save routing rule: {e}"),
        ),
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
    let split = |v: &str| -> Vec<String> {
        v.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    };
    let query_strategy = match get_str("query_strategy") {
        "UseIP" => Some(xray_tui_db::models::QueryStrategy::UseIp),
        "UseIPv4" => Some(xray_tui_db::models::QueryStrategy::UseIpv4),
        "UseIPv6" => Some(xray_tui_db::models::QueryStrategy::UseIpv6),
        _ => None,
    };
    let dns = xray_tui_db::models::DnsSetting {
        id,
        name: None,
        servers: split(get_str("servers")),
        hosts: split(get_str("hosts")),
        query_strategy,
        disable_cache: get_str("disable_cache") == "true",
        disable_fallback: get_str("disable_fallback") == "true",
        client_ip: get_opt("client_ip"),
        cache_ttl_secs: get_opt("cache_ttl_secs").and_then(|s| s.parse().ok()),
    };
    let result: Result<(), String> = async {
        let mut conn = state.db.connection().await.map_err(|e| e.to_string())?;
        xray_tui_db::models::DnsSetting::upsert_by_id(dns.id.clone())
            .name(dns.name.clone())
            .servers(dns.servers.clone())
            .hosts(dns.hosts.clone())
            .query_strategy(dns.query_strategy)
            .disable_cache(dns.disable_cache)
            .disable_fallback(dns.disable_fallback)
            .client_ip(dns.client_ip.clone())
            .cache_ttl_secs(dns.cache_ttl_secs)
            .exec(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            // Apply immediately: TTL gate reads this on the next resolve.
            if let Some(ttl) = dns.cache_ttl_secs {
                state.dns_cache_ttl_secs = ttl;
            }
            state.log_trace("info", "tui::ops::settings", "DNS settings saved");
        }
        Err(e) => state.log_trace(
            "error",
            "tui::ops::settings",
            &format!("Failed to save DNS settings: {e}"),
        ),
    }
}
