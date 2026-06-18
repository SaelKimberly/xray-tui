use serde::Serialize;
use serde_json::{json, Value};
use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};

use crate::protocol::Protocol;

use super::{BuildError, BuildParams};


// ── Xray JSON config structs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayConfig {
    pub log: LogConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub routing: RoutingConfig,
    pub dns: DnsConfig,
    pub stats: StatsConfig,
    pub api: ApiConfig,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub loglevel: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inbound {
    pub listen: String,
    pub port: u16,
    pub protocol: &'static str,
    pub settings: Value,
    pub sniffing: Value,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outbound {
    pub tag: String,
    pub protocol: String,
    pub settings: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    pub domain_strategy: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub balancers: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsConfig {
    pub servers: Vec<Value>,
    pub hosts: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsConfig {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub tag: &'static str,
    pub services: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyConfig {
    pub levels: Value,
    pub system: Value,
}

// ── Builder ──────────────────────────────────────────────────────────

pub struct XrayConfigBuilder;

impl XrayConfigBuilder {
    pub fn build(
        profile: &Profile,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<XrayConfig, BuildError> {
        let outbounds = vec![
            build_proxy_outbound(profile)?,
            build_dns_outbound(),
            build_direct_outbound(),
            build_block_outbound(),
        ];

        Ok(XrayConfig {
            log: LogConfig {
                loglevel: params.log_level.clone(),
            },
            inbounds: build_inbounds(params),
            outbounds,
            routing: build_routing(routing),
            dns: build_dns(dns),
            stats: StatsConfig {},
            api: ApiConfig {
                tag: "api",
                services: vec!["HandlerService", "LoggerService", "StatsService"],
            },
            policy: PolicyConfig {
                levels: json!({
                    "0": {
                        "statsUserUplink": true,
                        "statsUserDownlink": true
                    }
                }),
                system: json!({
                    "statsInboundUplink": true,
                    "statsOutboundUplink": true
                }),
            },
        })
    }
}

// ── Inbound construction ─────────────────────────────────────────────

fn build_inbounds(params: &BuildParams) -> Vec<Inbound> {
    let mut inbounds = vec![Inbound {
        listen: params.listen.clone(),
        port: params.socks_port,
        protocol: "socks",
        settings: json!({
            "auth": "noauth",
            "udp": true,
            "ip": "127.0.0.1"
        }),
        sniffing: json!({
            "enabled": params.sniffing,
            "destOverride": ["http", "tls", "quic"]
        }),
        tag: "socks-in".to_string(),
    }];

    if let Some(http_port) = params.http_port {
        inbounds.push(Inbound {
            listen: params.listen.clone(),
            port: http_port,
            protocol: "http",
            settings: json!({}),
            sniffing: json!({}),
            tag: "http-in".to_string(),
        });
    }

    // API inbound for gRPC Commander
    inbounds.push(Inbound {
        listen: "127.0.0.1".into(),
        port: super::API_PORT,
        protocol: "dokodemo-door",
        settings: json!({ "address": "127.0.0.1" }),
        sniffing: json!({ "enabled": false, "destOverride": [] }),
        tag: "api".into(),
    });
    inbounds
}

// ── Outbound construction ────────────────────────────────────────────

fn build_proxy_outbound(profile: &Profile) -> Result<Outbound, BuildError> {
    let protocol = Protocol::try_from_i32(profile.config_type).ok_or_else(|| {
        BuildError::InvalidProfile(format!("Unknown config_type: {}", profile.config_type))
    })?;

    let address = profile.address.as_deref().unwrap_or("127.0.0.1");
    let port = profile.port.unwrap_or(0) as u16;
    let user_id = profile.user_id.as_deref().unwrap_or("");

    let p_settings: Value = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    let s_settings: Option<Value> = profile
        .stream_settings
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    match protocol {
        Protocol::Vmess => Ok(Outbound {
            tag: "proxy".to_string(),
            protocol: "vmess".to_string(),
            settings: json!({
                "vnext": [{
                    "address": address,
                    "port": port,
                    "users": [{
                        "id": user_id,
                        "security": profile.security.as_deref().unwrap_or("auto")
                    }]
                }]
            }),
            stream_settings: s_settings,
        }),
        Protocol::Vless => {
            let flow = p_settings
                .get("flow")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "vless".to_string(),
                settings: json!({
                    "vnext": [{
                        "address": address,
                        "port": port,
                        "users": [{
                            "id": user_id,
                            "encryption": "none",
                            "flow": flow
                        }]
                    }]
                }),
                stream_settings: s_settings,
            })
        }
        Protocol::Shadowsocks => {
            let method = p_settings
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("aes-256-gcm");
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "shadowsocks".to_string(),
                settings: json!({
                    "servers": [{
                        "address": address,
                        "port": port,
                        "method": method,
                        "password": user_id,
                        "uot": true
                    }]
                }),
                stream_settings: None,
            })
        }
        Protocol::Trojan => {
            let flow = p_settings
                .get("flow")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "trojan".to_string(),
                settings: json!({
                    "servers": [{
                        "address": address,
                        "port": port,
                        "password": user_id,
                        "flow": flow
                    }]
                }),
                stream_settings: s_settings,
            })
        }
        Protocol::Socks => {
            let mut server = json!({
                "address": address,
                "port": port,
            });
            add_user_if_present(&mut server, &p_settings);
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "socks".to_string(),
                settings: json!({ "servers": [server] }),
                stream_settings: None,
            })
        }
        Protocol::Http => {
            let mut server = json!({
                "address": address,
                "port": port,
            });
            add_user_if_present(&mut server, &p_settings);
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "http".to_string(),
                settings: json!({ "servers": [server] }),
                stream_settings: None,
            })
        }
        Protocol::Hysteria2 => {
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or(user_id);
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "hysteria2".to_string(),
                settings: json!({
                    "servers": [{
                        "address": address,
                        "port": port,
                        "password": password
                    }]
                }),
                stream_settings: None,
            })
        }
        _ => Err(BuildError::InvalidProfile(format!(
            "Protocol {protocol:?} not supported for xray outbound"
        ))),
    }
}

fn add_user_if_present(server: &mut Value, p_settings: &Value) {
    let username = p_settings
        .get("username")
        .or_else(|| p_settings.get("user"))
        .and_then(|v| v.as_str());
    let password = p_settings
        .get("password")
        .or_else(|| p_settings.get("pass"))
        .and_then(|v| v.as_str());
    if let (Some(u), Some(p)) = (username, password)
        && !u.is_empty() && !p.is_empty() {
            server["users"] = json!([{ "user": u, "pass": p }]);
        }
}

fn build_dns_outbound() -> Outbound {
    Outbound {
        tag: "dns-out".to_string(),
        protocol: "dns".to_string(),
        settings: json!({}),
        stream_settings: None,
    }
}

fn build_direct_outbound() -> Outbound {
    Outbound {
        tag: "direct".to_string(),
        protocol: "freedom".to_string(),
        settings: json!({ "domainStrategy": "UseIP" }),
        stream_settings: None,
    }
}

fn build_block_outbound() -> Outbound {
    Outbound {
        tag: "block".to_string(),
        protocol: "blackhole".to_string(),
        settings: json!({ "response": { "type": "http" } }),
        stream_settings: None,
    }
}

// ── Routing ──────────────────────────────────────────────────────────

fn build_routing(rules: &[RoutingRule]) -> RoutingConfig {
    let json_rules: Vec<Value> = rules
        .iter()
        .filter_map(|r| {
            let mut rule = json!({ "type": "field" });
            if let Some(domains) = &r.domains {
                rule["domain"] = json!(parse_comma_list(domains));
            }
            if let Some(ips) = &r.ips {
                rule["ip"] = json!(parse_comma_list(ips));
            }
            if let Some(inbound_tags) = &r.inbound_tags {
                rule["inboundTag"] = json!(parse_comma_list(inbound_tags));
            }
            if let Some(port) = &r.port {
                rule["port"] = json!(port);
            }
            if let Some(source_ports) = &r.source_ports {
                rule["sourcePort"] = json!(source_ports);
            }
            if let Some(network) = &r.network {
                rule["network"] = json!(network);
            }
            // outboundTag or balancerTag
            if let Some(tag) = &r.outbound_tag {
                rule["outboundTag"] = json!(tag);
            } else if let Some(tag) = &r.balancer_tag {
                rule["balancerTag"] = json!(tag);
            } else {
                // Skip rules without a target outbound
                return None;
            }
            Some(rule)
        })
        .collect();

    RoutingConfig {
        domain_strategy: "AsIs".to_string(),
        rules: json_rules,
        balancers: vec![],
    }
}

// ── DNS ──────────────────────────────────────────────────────────────

fn build_dns(dns: &DnsSetting) -> DnsConfig {
    let servers: Vec<Value> = dns
        .servers
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let hosts: Value = dns
        .hosts
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    DnsConfig { servers, hosts }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_comma_list(s: &str) -> Vec<&str> {
    s.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};

    fn test_profile(config_type: i32) -> Profile {
        Profile {
            id: "test-id".to_string(),
            config_type,
            core_type: String::new(),
            remarks: Some("test".to_string()),
            address: Some("example.com".to_string()),
            port: Some(443),
            user_id: Some("test-uuid-or-pass".to_string()),
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
        }
    }

    fn default_params() -> (BuildParams, Vec<RoutingRule>, DnsSetting) {
        let params = BuildParams {
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

    fn assert_xray_top_level(json: &Value) {
        assert!(json.get("log").is_some(), "missing log");
        assert!(json.get("inbounds").is_some(), "missing inbounds");
        assert!(json.get("outbounds").is_some(), "missing outbounds");
        assert!(json.get("routing").is_some(), "missing routing");
        assert!(json.get("dns").is_some(), "missing dns");
        assert!(json.get("stats").is_some(), "missing stats");
        assert!(json.get("api").is_some(), "missing api");
        assert!(json.get("policy").is_some(), "missing policy");
    }

    fn assert_proxy_outbound(json: &Value, expected_protocol: &str) {
        let outbounds = json["outbounds"].as_array().expect("outbounds array");
        let proxy = outbounds
            .iter()
            .find(|o| o["tag"] == "proxy")
            .expect("proxy outbound");
        assert_eq!(
            proxy["protocol"].as_str().unwrap(),
            expected_protocol,
            "protocol mismatch"
        );
        assert!(
            proxy["settings"].is_object(),
            "proxy settings should be an object"
        );
    }

    fn assert_has_standard_outbounds(json: &Value) {
        let outbounds = json["outbounds"].as_array().expect("outbounds array");
        let tags: Vec<&str> = outbounds
            .iter()
            .filter_map(|o| o["tag"].as_str())
            .collect();
        assert!(tags.contains(&"dns-out"), "missing dns-out");
        assert!(tags.contains(&"direct"), "missing direct");
        assert!(tags.contains(&"block"), "missing block");
    }

    #[test]
    fn xray_vmess_config() {
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "vmess");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn xray_vless_config() {
        let profile = test_profile(Protocol::Vless.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "vless");
    }

    #[test]
    fn xray_shadowsocks_config() {
        let profile = test_profile(Protocol::Shadowsocks.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "shadowsocks");
    }

    #[test]
    fn xray_trojan_config() {
        let profile = test_profile(Protocol::Trojan.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "trojan");
    }

    #[test]
    fn xray_socks_config() {
        let profile = test_profile(Protocol::Socks.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "socks");
    }

    #[test]
    fn xray_http_config() {
        let profile = test_profile(Protocol::Http.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "http");
    }

    #[test]
    fn xray_hysteria2_config() {
        let profile = test_profile(Protocol::Hysteria2.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "hysteria2");
    }

    #[test]
    fn xray_inbounds_default() {
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let inbounds = json["inbounds"].as_array().unwrap();
        assert_eq!(inbounds.len(), 2, "should have 2 inbounds (SOCKS + API)");
        assert_eq!(inbounds[0]["protocol"], "socks");
        assert_eq!(inbounds[0]["port"], 10808);
        assert_eq!(inbounds[1]["protocol"], "dokodemo-door");
        assert_eq!(inbounds[1]["tag"], "api");
    }

    #[test]
    fn xray_inbounds_with_http() {
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (mut params, rules, dns) = default_params();
        params.http_port = Some(10809);
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let inbounds = json["inbounds"].as_array().unwrap();
        assert_eq!(inbounds.len(), 3, "should have 3 inbounds (SOCKS + HTTP + API)");
        assert_eq!(inbounds[0]["protocol"], "socks");
        assert_eq!(inbounds[0]["port"], 10808);
        assert_eq!(inbounds[1]["protocol"], "http");
        assert_eq!(inbounds[1]["port"], 10809);
        assert_eq!(inbounds[2]["protocol"], "dokodemo-door");
        assert_eq!(inbounds[2]["tag"], "api");
    }

    #[test]
    fn xray_unknown_protocol_returns_error() {
        // Protocol::WireGuard = 9 — not in our supported list
        let profile = test_profile(Protocol::WireGuard.to_i32());
        let (params, rules, dns) = default_params();
        let result = XrayConfigBuilder::build(&profile, &params, &rules, &dns);
        assert!(
            result.is_err(),
            "unsupported protocol should return error"
        );
        match result {
            Err(BuildError::InvalidProfile(_)) => {} // expected
            _ => panic!("expected InvalidProfile error"),
        }
    }

    #[test]
    fn xray_routing_with_domain_rule() {
        let profile = test_profile(Protocol::Vmess.to_i32());
        let (params, _, dns) = default_params();
        let rules = vec![RoutingRule {
            id: "r1".to_string(),
            group_id: None,
            r#type: 0,
            domain_matcher: None,
            domains: Some("example.com,test.org".to_string()),
            ips: None,
            inbound_tags: None,
            port: None,
            source_ports: None,
            network: Some("tcp".to_string()),
            protocols: None,
            domain_strategy: None,
            outbound_tag: Some("direct".to_string()),
            balancer_tag: None,
            rule_set_file: None,
            rule_set_url: None,
            sort_order: Some(0),
        }];
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let routing_rules = json["routing"]["rules"].as_array().unwrap();
        assert_eq!(routing_rules.len(), 1);
        assert_eq!(routing_rules[0]["domain"][0], "example.com");
        assert_eq!(routing_rules[0]["network"], "tcp");
        assert_eq!(routing_rules[0]["outboundTag"], "direct");
    }

    #[test]
    fn xray_stream_settings_passed_through() {
        let mut profile = test_profile(Protocol::Vmess.to_i32());
        profile.stream_settings = Some(
            json!({"network": "ws", "security": "tls", "wsSettings": {"path": "/ws"}}).to_string(),
        );
        let (params, rules, dns) = default_params();
        let config =
            XrayConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let outbounds = json["outbounds"].as_array().unwrap();
        let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").unwrap();
        let ss = proxy["streamSettings"].as_object().unwrap();
        assert_eq!(ss["network"], "ws");
        assert_eq!(ss["security"], "tls");
    }
}
