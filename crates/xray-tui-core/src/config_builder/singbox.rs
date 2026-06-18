use serde::Serialize;
use serde_json::{json, Value};
use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};

use crate::protocol::Protocol;

use super::{BuildError, BuildParams};

// ── Sing-box JSON config structs ──────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SingBoxConfig {
    pub log: SingBoxLogConfig,
    pub inbounds: Vec<SingBoxInbound>,
    pub outbounds: Vec<Value>,
    pub route: RouteConfig,
    pub dns: SingBoxDnsConfig,
    pub experimental: ExperimentalConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SingBoxLogConfig {
    pub level: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SingBoxInbound {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
    pub sniff: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RouteConfig {
    pub rules: Vec<Value>,
    pub rule_set: Vec<Value>,
    #[serde(rename = "final")]
    pub final_outbound: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SingBoxDnsConfig {
    pub servers: Vec<Value>,
    pub hosts: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ExperimentalConfig {
    pub v2ray_api: V2RayApi,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct V2RayApi {
    pub listen: String,
    pub stats: StatsConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StatsConfig {
    pub enabled: bool,
    pub outbounds: Vec<&'static str>,
}

// ── Builder ──────────────────────────────────────────────────────────

pub struct SingBoxConfigBuilder;

impl SingBoxConfigBuilder {
    pub fn build(
        profile: &Profile,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<SingBoxConfig, BuildError> {
        let outbounds = vec![
            build_proxy_outbound(profile)?,
            build_direct_outbound(),
            build_block_outbound(),
        ];

        Ok(SingBoxConfig {
            log: SingBoxLogConfig {
                level: params.log_level.clone(),
            },
            inbounds: vec![SingBoxInbound {
                type_: "socks",
                tag: "socks-in".to_string(),
                listen: params.listen.clone(),
                listen_port: params.socks_port,
                sniff: params.sniffing,
            }],
            outbounds,
            route: build_routing(routing),
            dns: build_dns(dns),
            experimental: ExperimentalConfig {
                v2ray_api: V2RayApi {
                    listen: format!("127.0.0.1:{}", super::API_PORT),
                    stats: StatsConfig {
                        enabled: true,
                        outbounds: vec!["proxy", "direct"],
                    },
                },
            },
        })
    }
}

fn build_proxy_outbound(profile: &Profile) -> Result<Value, BuildError> {
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

    match protocol {
        Protocol::Tuic => {
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(json!({
                "tag": "proxy",
                "type": "tuic",
                "server": address,
                "server_port": port,
                "uuid": user_id,
                "password": password,
                "tls": {
                    "enabled": true,
                    "server_name": address
                }
            }))
        }
        Protocol::Hysteria2 => {
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or(user_id);
            let up_mbps = p_settings
                .get("up_mbps")
                .and_then(|v| v.as_u64())
                .unwrap_or(100);
            let down_mbps = p_settings
                .get("down_mbps")
                .and_then(|v| v.as_u64())
                .unwrap_or(100);
            Ok(json!({
                "tag": "proxy",
                "type": "hysteria2",
                "server": address,
                "server_port": port,
                "password": password,
                "up_mbps": up_mbps,
                "down_mbps": down_mbps,
                "tls": {
                    "enabled": true,
                    "server_name": address
                }
            }))
        }
        Protocol::Shadowsocks => {
            let method = p_settings
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("aes-256-gcm");
            Ok(json!({
                "tag": "proxy",
                "type": "shadowsocks",
                "server": address,
                "server_port": port,
                "method": method,
                "password": user_id
            }))
        }
        Protocol::Socks => {
            let mut out = json!({
                "tag": "proxy",
                "type": "socks",
                "server": address,
                "server_port": port
            });
            if let Some(username) = p_settings.get("username").or_else(|| p_settings.get("user"))
                && let Some(u) = username.as_str()
                    && !u.is_empty() {
                        out["username"] = json!(u);
                        if let Some(password) = p_settings
                            .get("password")
                            .or_else(|| p_settings.get("pass"))
                            .and_then(|v| v.as_str())
                            && !password.is_empty() {
                                out["password"] = json!(password);
                            }
                    }
            Ok(out)
        }
        Protocol::Http => {
            let mut out = json!({
                "tag": "proxy",
                "type": "http",
                "server": address,
                "server_port": port
            });
            if let Some(username) = p_settings.get("username").or_else(|| p_settings.get("user"))
                && let Some(u) = username.as_str()
                    && !u.is_empty() {
                        out["username"] = json!(u);
                        if let Some(password) = p_settings
                            .get("password")
                            .or_else(|| p_settings.get("pass"))
                            .and_then(|v| v.as_str())
                            && !password.is_empty() {
                                out["password"] = json!(password);
                            }
                    }
            Ok(out)
        }
        _ => Err(BuildError::InvalidProfile(format!(
            "Protocol {protocol:?} not supported for sing-box outbound"
        ))),
    }
}

fn build_direct_outbound() -> Value {
    json!({
        "tag": "direct",
        "type": "direct"
    })
}

fn build_block_outbound() -> Value {
    json!({
        "tag": "block",
        "type": "block"
    })
}

// ── Routing ──────────────────────────────────────────────────────────

fn build_routing(rules: &[RoutingRule]) -> RouteConfig {
    let json_rules: Vec<Value> = rules
        .iter()
        .filter_map(|r| {
            let mut rule = json!({});
            if let Some(domains) = &r.domains {
                rule["domain"] = json!(parse_comma_list(domains));
            }
            if let Some(ips) = &r.ips {
                rule["ip_cidr"] = json!(parse_comma_list(ips));
            }
            if let Some(inbound_tags) = &r.inbound_tags {
                rule["inbound"] = json!(parse_comma_list(inbound_tags));
            }
            if let Some(port) = &r.port {
                // Parse port range string into array of values
                if let Ok(p) = port.parse::<u16>() {
                    rule["port"] = json!([p]);
                } else {
                    rule["port"] = json!([port]);
                }
            }
            if let Some(network) = &r.network {
                rule["network"] = json!([network]);
            }
            if let Some(tag) = &r.outbound_tag {
                rule["outbound"] = json!(tag);
            } else if let Some(tag) = &r.balancer_tag {
                rule["outbound"] = json!(tag);
            } else {
                return None;
            }
            Some(rule)
        })
        .collect();

    RouteConfig {
        rules: json_rules,
        rule_set: vec![],
        final_outbound: "proxy".to_string(),
    }
}

// ── DNS ──────────────────────────────────────────────────────────────

fn build_dns(dns: &DnsSetting) -> SingBoxDnsConfig {
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

    SingBoxDnsConfig { servers, hosts }
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

    fn assert_singbox_top_level(json: &Value) {
        assert!(json.get("log").is_some(), "missing log");
        assert!(json.get("inbounds").is_some(), "missing inbounds");
        assert!(json.get("outbounds").is_some(), "missing outbounds");
        assert!(json.get("route").is_some(), "missing route");
        assert!(json.get("dns").is_some(), "missing dns");
        assert!(json.get("experimental").is_some(), "missing experimental");
    }

    fn assert_proxy_outbound(json: &Value, expected_type: &str) {
        let outbounds = json["outbounds"].as_array().expect("outbounds array");
        let proxy = outbounds
            .iter()
            .find(|o| o["tag"] == "proxy")
            .expect("proxy outbound");
        assert_eq!(
            proxy["type"].as_str().unwrap(),
            expected_type,
            "type mismatch"
        );
    }

    fn assert_has_standard_outbounds(json: &Value) {
        let outbounds = json["outbounds"].as_array().expect("outbounds array");
        let tags: Vec<&str> = outbounds
            .iter()
            .filter_map(|o| o["tag"].as_str())
            .collect();
        assert!(tags.contains(&"direct"), "missing direct");
        assert!(tags.contains(&"block"), "missing block");
    }

    #[test]
    fn singbox_tuic_config() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "tuic");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_hysteria2_config() {
        let mut profile = test_profile(Protocol::Hysteria2.to_i32());
        profile.protocol_settings = Some(r#"{"password": "sekret", "up_mbps": 50, "down_mbps": 200}"#.to_string());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "hysteria2");

        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["up_mbps"], 50);
        assert_eq!(proxy["down_mbps"], 200);
    }

    #[test]
    fn singbox_shadowsocks_config() {
        let profile = test_profile(Protocol::Shadowsocks.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "shadowsocks");
    }

    #[test]
    fn singbox_socks_config() {
        let profile = test_profile(Protocol::Socks.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "socks");
    }

    #[test]
    fn singbox_http_config() {
        let profile = test_profile(Protocol::Http.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "http");
    }

    #[test]
    fn singbox_tuic_with_tls() {
        let mut profile = test_profile(Protocol::Tuic.to_i32());
        profile.protocol_settings = Some(r#"{"password": "pass123"}"#.to_string());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["tls"]["enabled"], true);
        assert_eq!(proxy["tls"]["server_name"], "example.com");
        assert_eq!(proxy["password"], "pass123");
    }

    #[test]
    fn singbox_default_inbound() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let inbounds = json["inbounds"].as_array().unwrap();
        assert_eq!(inbounds.len(), 1);
        assert_eq!(inbounds[0]["type"], "socks");
        assert_eq!(inbounds[0]["listen_port"], 10808);
    }

    #[test]
    fn singbox_experimental_config() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let exp = &json["experimental"]["v2ray_api"];
        assert_eq!(exp["listen"], format!("127.0.0.1:{}", crate::config_builder::API_PORT));
        assert_eq!(exp["stats"]["enabled"], true);
    }

    #[test]
    fn singbox_unsupported_protocol_returns_error() {
        // Vless is xray-native, not sing-box
        let profile = test_profile(Protocol::Vless.to_i32());
        let (params, rules, dns) = default_params();
        let result = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns);
        assert!(result.is_err(), "unsupported protocol should return error");
        match result {
            Err(BuildError::InvalidProfile(_)) => {} // expected
            _ => panic!("expected InvalidProfile error"),
        }
    }

    #[test]
    fn singbox_route_config() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, _, dns) = default_params();
        let rules = vec![RoutingRule {
            id: "r1".to_string(),
            group_id: None,
            r#type: 0,
            domain_matcher: None,
            domains: Some("example.com".to_string()),
            ips: None,
            inbound_tags: None,
            port: None,
            source_ports: None,
            network: None,
            protocols: None,
            domain_strategy: None,
            outbound_tag: Some("direct".to_string()),
            balancer_tag: None,
            rule_set_file: None,
            rule_set_url: None,
            sort_order: Some(0),
        }];
        let config =
            SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let route = &json["route"];
        assert_eq!(route["rules"].as_array().unwrap().len(), 1);
        assert_eq!(route["rules"][0]["domain"][0], "example.com");
        assert_eq!(route["rules"][0]["outbound"], "direct");
        assert_eq!(route["final"], "proxy");
    }
}
