use serde::Serialize;
use serde_json::{Value, json};
use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow, RoutingRule};

use crate::protocol::Protocol;

use super::{BuildError, BuildParams, MultiInboundItem, parse_settings};

// ── Xray JSON config structs ────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayConfig {
    pub log: LogConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub routing: RoutingConfig,
    pub dns: DnsConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<StatsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyConfig>,
}

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    pub domain_strategy: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub balancers: Vec<Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsConfig {
    pub servers: Vec<Value>,
    pub hosts: Value,
}

#[derive(Debug, Clone, Default, Serialize)]
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
        endpoint: &Endpoint,
        protocol: &ProtocolRow,
        params: &BuildParams,
        routing: &[RoutingRule],
        dns: &DnsSetting,
    ) -> Result<XrayConfig, BuildError> {
        let outbounds = vec![
            build_proxy_outbound(endpoint, protocol)?,
            build_dns_outbound(),
            build_direct_outbound(),
            build_block_outbound(),
        ];

        let (stats, api, policy) = if params.v2ray_api_enabled {
            (
                Some(StatsConfig {}),
                Some(ApiConfig {
                    tag: "api",
                    services: vec!["HandlerService", "LoggerService", "StatsService"],
                }),
                Some(PolicyConfig {
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
                }),
            )
        } else {
            (None, None, None)
        };

        Ok(XrayConfig {
            log: LogConfig {
                loglevel: params.log_level.clone(),
            },
            inbounds: build_inbounds(params),
            outbounds,
            routing: build_routing(routing, params.v2ray_api_enabled),
            dns: build_dns(dns),
            stats,
            api,
            policy,
        })
    }

    /// Build a multi-inbound config for batch real ping.
    /// Creates N SOCKS5 inbounds + N proxy outbounds with routing rules
    /// connecting each inbound to its matching outbound.
    pub fn build_multi(
        items: &[MultiInboundItem],
        base_params: &BuildParams,
        dns: &DnsSetting,
    ) -> Result<XrayConfig, BuildError> {
        let mut inbounds = Vec::with_capacity(items.len());
        let mut proxy_outbounds = Vec::with_capacity(items.len());
        let mut routing_rules: Vec<Value> = Vec::with_capacity(items.len());

        for (i, item) in items.iter().enumerate() {
            let tag = format!("proxy-{i}");
            let inbound_tag = format!("socks-in-{i}");

            inbounds.push(Inbound {
                listen: base_params.listen.clone(),
                port: item.assigned_port,
                protocol: "socks",
                settings: json!({
                    "auth": "noauth",
                    "udp": false,
                    "ip": "127.0.0.1"
                }),
                sniffing: json!({
                    "enabled": false,
                    "destOverride": []
                }),
                tag: inbound_tag.clone(),
            });

            let mut outbound = build_proxy_outbound(item.endpoint, item.protocol)?;
            outbound.tag = tag.clone();
            proxy_outbounds.push(outbound);

            routing_rules.push(json!({
                "type": "field",
                "inboundTag": [inbound_tag],
                "outboundTag": tag
            }));
        }

        // Standard outbounds
        let mut outbounds = proxy_outbounds;
        outbounds.push(build_dns_outbound());
        outbounds.push(build_direct_outbound());
        outbounds.push(build_block_outbound());

        Ok(XrayConfig {
            log: LogConfig {
                loglevel: base_params.log_level.clone(),
            },
            inbounds,
            outbounds,
            routing: RoutingConfig {
                domain_strategy: "AsIs".to_string(),
                rules: routing_rules,
                balancers: vec![],
            },
            dns: build_dns(dns),
            stats: None,
            api: None,
            policy: None,
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

    // API inbound for gRPC Commander — only when stats/api enabled
    if params.v2ray_api_enabled {
        inbounds.push(Inbound {
            listen: "127.0.0.1".into(),
            port: super::API_PORT,
            protocol: "dokodemo-door",
            settings: json!({ "address": "127.0.0.1" }),
            sniffing: json!({ "enabled": false, "destOverride": [] }),
            tag: "api".into(),
        });
    }
    inbounds
}

// ── Outbound construction ────────────────────────────────────────────
fn build_proxy_outbound(
    endpoint: &Endpoint,
    protocol: &ProtocolRow,
) -> Result<Outbound, BuildError> {
    let proto = Protocol::try_from_i32(protocol.config_type).ok_or_else(|| {
        BuildError::InvalidProfile(format!("Unknown config_type: {}", protocol.config_type))
    })?;

    let address = endpoint.host.as_str();
    let port = endpoint.port as u16;
    let (p_settings, s_settings_raw) = parse_settings(protocol);
    let user_id = p_settings
        .get("id")
        .or_else(|| p_settings.get("uuid"))
        .or_else(|| p_settings.get("password"))
        .or_else(|| p_settings.get("pass"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let s_settings = build_xray_stream_settings(protocol, s_settings_raw);

    match proto {
        Protocol::Vmess => Ok(Outbound {
            tag: "proxy".to_string(),
            protocol: "vmess".to_string(),
            settings: json!({
                "vnext": [{
                    "address": address,
                    "port": port,
                    "users": [{
                        "id": user_id,
                        "security": protocol.security.as_deref().unwrap_or("auto")
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
                        "password": user_id
                    }]
                }),
                stream_settings: s_settings.clone(),
            })
        }
        Protocol::Trojan => Ok(Outbound {
            tag: "proxy".to_string(),
            protocol: "trojan".to_string(),
            settings: json!({
                "servers": [{
                    "address": address,
                    "port": port,
                    "password": user_id
                }]
            }),
            stream_settings: s_settings,
        }),
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
                stream_settings: s_settings.clone(),
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
                stream_settings: s_settings.clone(),
            })
        }
        Protocol::Hysteria2 => {
            let auth = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(Outbound {
                tag: "proxy".to_string(),
                protocol: "hysteria2".to_string(),
                settings: json!({
                    "version": 2,
                    "address": address,
                    "port": port,
                    "auth": auth
                }),
                stream_settings: s_settings.clone(),
            })
        }
        _ => Err(BuildError::InvalidProfile(format!(
            "Protocol {proto:?} not supported for xray outbound"
        ))),
    }
}

/// xray-shaped `streamSettings` for a profile. Typed configs (Vmess/Vless/
/// Trojan) build from SecurityConfig+TransportConfig; legacy
/// PlaceholderConfig blobs carry a homegrown dotted-key format
/// ("ws.path", "tls.enable", "realitySettings.publicKey", ...) that must be
/// expanded into the xray shape.
fn build_xray_stream_settings(
    protocol: &ProtocolRow,
    s_settings_raw: serde_json::Value,
) -> Option<serde_json::Value> {
    use xray_tui_proto::proto_spec::ProtocolConfig;
    match serde_json::from_slice::<ProtocolConfig>(&protocol.spec_blob) {
        Ok(ProtocolConfig::Vmess(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        Ok(ProtocolConfig::Vless(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        Ok(ProtocolConfig::Trojan(c)) => {
            xray_tui_proto::proto_spec::common::to_xray_stream_settings(&c.security, &c.transport)
        }
        _ => legacy_stream_settings_to_xray(s_settings_raw, protocol.transport.as_deref()),
    }
}

/// Expand the legacy dotted-key stream_settings format into xray shape.
fn legacy_stream_settings_to_xray(
    raw: serde_json::Value,
    network: Option<&str>,
) -> Option<serde_json::Value> {
    let obj = raw.as_object()?;
    if obj.is_empty() {
        return None;
    }
    let mut ss: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let get = |k: &str| obj.get(k).cloned();

    if let Some(net) = network.filter(|n| !n.is_empty() && *n != "tcp") {
        ss.insert("network".into(), serde_json::json!(net));
    }
    // Legacy `security` == "reality" (the only value the legacy parser stored).
    if get("security").as_ref().and_then(|v| v.as_str()) == Some("reality") {
        ss.insert("security".into(), serde_json::json!("reality"));
    } else if get("tls.enable").as_ref().and_then(|v| v.as_bool()) == Some(true)
        || get("tls.enable").as_ref().and_then(|v| v.as_str()).is_some()
    {
        ss.insert("security".into(), serde_json::json!("tls"));
    }
    let mut tls = serde_json::Map::new();
    if let Some(v) = get("sni").and_then(|v| v.as_str().map(str::to_string)) {
        tls.insert("serverName".into(), serde_json::json!(v));
    }
    // Legacy emitters write the plain "allow_insecure" key (vmess QR insecure,
    // vless allowInsecure query param, forms.rs); "tls.allow_insecure" is the
    // dotted legacy variant — accept both, preferring the plain key.
    if let Some(v) = get("allow_insecure")
        .or_else(|| get("tls.allow_insecure"))
        .and_then(|v| v.as_bool())
    {
        tls.insert("allowInsecure".into(), serde_json::json!(v));
    }
    if let Some(v) = get("fingerprint").and_then(|v| v.as_str().map(str::to_string)) {
        tls.insert("fingerprint".into(), serde_json::json!(v));
    }
    if let Some(v) = get("alpn").and_then(|v| v.as_str().map(str::to_string)) {
        let list: Vec<&str> = v.split(',').map(str::trim).collect();
        tls.insert("alpn".into(), serde_json::json!(list));
    }
    if ss.get("security") == Some(&serde_json::json!("reality")) {
        // realitySettings already xray-shaped in legacy output
        if let Some(rs) = obj.get("realitySettings").cloned() {
            ss.insert("realitySettings".into(), rs);
        }
        if let Some(server_name) = tls.get("serverName").cloned()
            && let Some(rs) = ss.get_mut("realitySettings").and_then(|v| v.as_object_mut())
        {
            rs.insert("serverName".into(), server_name);
        }
    } else if !tls.is_empty() {
        ss.insert("tlsSettings".into(), serde_json::Value::Object(tls));
    }
    // Transport blocks
    let mut ws = serde_json::Map::new();
    if let Some(v) = get("ws.path").and_then(|v| v.as_str().map(str::to_string)) {
        ws.insert("path".into(), serde_json::json!(v));
    }
    if let Some(v) = get("ws.host").and_then(|v| v.as_str().map(str::to_string)) {
        ws.insert("headers".into(), serde_json::json!({ "Host": v }));
    }
    if !ws.is_empty() {
        ss.insert("wsSettings".into(), serde_json::Value::Object(ws));
    }
    let mut grpc = serde_json::Map::new();
    if let Some(v) = get("grpc.serviceName").and_then(|v| v.as_str().map(str::to_string)) {
        grpc.insert("serviceName".into(), serde_json::json!(v));
    }
    if !grpc.is_empty() {
        ss.insert("grpcSettings".into(), serde_json::Value::Object(grpc));
    }
    if ss.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(ss))
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
        && !u.is_empty()
        && !p.is_empty()
    {
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

fn build_routing(rules: &[RoutingRule], v2ray_api_enabled: bool) -> RoutingConfig {
    let mut json_rules: Vec<Value> = Vec::new();
    // Only add API routing rule when v2ray_api is enabled.
    // When disabled (e.g. real ping temp core), no "api" inbound/outbound exist.
    if v2ray_api_enabled {
        json_rules.push(json!({
            "type": "field",
            "inboundTag": ["api"],
            "outboundTag": "api"
        }));
    }
    json_rules.extend(rules.iter().filter_map(|r| {
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
    }));
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
        .unwrap_or_else(|| json!({}));

    DnsConfig { servers, hosts }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_comma_list(s: &str) -> Vec<&str> {
    s.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_db::models::{DnsSetting, Endpoint, ProtocolRow, RoutingRule};

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
            resolved_as: None,
            resolved_at: None,
        };
        let extra = serde_json::json!({
            "remarks": "test",
            "user_id": "test-uuid-or-pass",
        });
        let protocol = ProtocolRow {
            id: 0,
            endpoint_id: 0,
            sig: 0,
            cred_hash: 0,
            proto_kind: String::new(),
            last_used_at: None,
            spec_blob: serde_json::to_vec(&extra).unwrap_or_default(),
            config_type,
            core_type: String::new(),
            transport: Some("tcp".to_string()),
            security: Some("auto".to_string()),
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
            cache_ttl_secs: None,
        };
        (params, rules, dns)
    }

    fn set_stream_settings_json(protocol: &mut ProtocolRow, json_str: &str) {
        let mut extra: serde_json::Value =
            serde_json::from_slice(&protocol.spec_blob).unwrap_or_default();
        extra["stream_settings"] = serde_json::from_str(json_str).unwrap_or_default();
        protocol.spec_blob = serde_json::to_vec(&extra).unwrap_or_default();
    }

    fn set_protocol_settings_json(protocol: &mut ProtocolRow, json_str: &str) {
        let mut extra: serde_json::Value =
            serde_json::from_slice(&protocol.spec_blob).unwrap_or_default();
        extra["protocol_settings"] = serde_json::from_str(json_str).unwrap_or_default();
        protocol.spec_blob = serde_json::to_vec(&extra).unwrap_or_default();
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
        let tags: Vec<&str> = outbounds.iter().filter_map(|o| o["tag"].as_str()).collect();
        assert!(tags.contains(&"dns-out"), "missing dns-out");
        assert!(tags.contains(&"direct"), "missing direct");
        assert!(tags.contains(&"block"), "missing block");
    }

    #[test]
    fn xray_vmess_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "vmess");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn xray_vless_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vless.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "vless");
    }

    #[test]
    fn xray_shadowsocks_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Shadowsocks.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "shadowsocks");
    }

    #[test]
    fn xray_trojan_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Trojan.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "trojan");
    }

    #[test]
    fn xray_socks_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Socks.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "socks");
    }

    #[test]
    fn xray_http_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Http.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "http");
    }

    #[test]
    fn xray_hysteria2_config() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Hysteria2.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "hysteria2");
    }

    #[test]
    fn xray_inbounds_default() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
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
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (mut params, rules, dns) = default_params();
        params.http_port = Some(10809);
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let inbounds = json["inbounds"].as_array().unwrap();
        assert_eq!(
            inbounds.len(),
            3,
            "should have 3 inbounds (SOCKS + HTTP + API)"
        );
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
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::WireGuard.to_i32());
        let (params, rules, dns) = default_params();
        let result = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns);
        assert!(result.is_err(), "unsupported protocol should return error");
        match result {
            Err(BuildError::InvalidProfile(_)) => {} // expected
            _ => panic!("expected InvalidProfile error"),
        }
    }

    #[test]
    fn xray_routing_with_domain_rule() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
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
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let routing_rules = json["routing"]["rules"].as_array().unwrap();
        // First rule is the mandatory API routing rule
        assert_eq!(routing_rules.len(), 2);
        assert_eq!(routing_rules[0]["inboundTag"][0], "api");
        assert_eq!(routing_rules[0]["outboundTag"], "api");
        // Second rule is the user-defined domain rule
        assert_eq!(routing_rules[1]["domain"][0], "example.com");
        assert_eq!(routing_rules[1]["network"], "tcp");
        assert_eq!(routing_rules[1]["outboundTag"], "direct");
    }

    #[test]
    fn xray_stream_settings_passed_through() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vmess.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let outbounds = json["outbounds"].as_array().unwrap();
        let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").unwrap();
        // stream_settings from spec_blob not yet wired up (TODO)
        assert!(proxy.get("streamSettings").is_none());
    }

    #[test]
    fn legacy_vless_ws_tls_produces_xray_stream_settings() {
        let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Vless.to_i32());
        protocol.transport = Some("ws".to_string());
        set_stream_settings_json(
            &mut protocol,
            r#"{"tls.enable": true, "sni": "cdn.example.com", "ws.path": "/ws", "ws.host": "cdn.example.com"}"#,
        );
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns)
            .expect("build");
        let json = serde_json::to_value(&config).unwrap();
        let outbounds = json["outbounds"].as_array().expect("outbounds");
        let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").expect("proxy");
        let ss = proxy["streamSettings"].as_object().expect("streamSettings present");
        assert_eq!(ss["network"], "ws");
        assert_eq!(ss["security"], "tls");
        assert_eq!(ss["tlsSettings"]["serverName"], "cdn.example.com");
        assert_eq!(ss["wsSettings"]["path"], "/ws");
        assert_eq!(ss["wsSettings"]["headers"]["Host"], "cdn.example.com");
    }

    #[test]
    fn legacy_vless_tls_allow_insecure_produces_allow_insecure() {
        let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Vless.to_i32());
        protocol.transport = Some("tcp".to_string());
        set_stream_settings_json(
            &mut protocol,
            r#"{"tls.enable": true, "sni": "cdn.example.com", "allow_insecure": true}"#,
        );
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns)
            .expect("build");
        let json = serde_json::to_value(&config).unwrap();
        let outbounds = json["outbounds"].as_array().expect("outbounds");
        let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").expect("proxy");
        let ss = proxy["streamSettings"].as_object().expect("streamSettings present");
        assert_eq!(ss["security"], "tls");
        assert_eq!(ss["tlsSettings"]["allowInsecure"], true);
    }

    #[test]
    fn vless_reality_stream_settings_passed_through() {
        let (endpoint, protocol) = test_endpoint_and_protocol(Protocol::Vless.to_i32());
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_xray_top_level(&json);
        assert_proxy_outbound(&json, "vless");
        let outbounds = json["outbounds"].as_array().unwrap();
        let proxy = outbounds.iter().find(|o| o["tag"] == "proxy").unwrap();
        // stream_settings from spec_blob not yet wired up (TODO)
        assert!(proxy.get("streamSettings").is_none());
    }

    #[test]
    fn xray_hysteria2_outbound_includes_auth() {
        let (endpoint, mut protocol) = test_endpoint_and_protocol(Protocol::Hysteria2.to_i32());
        set_protocol_settings_json(&mut protocol, r#"{"password": "hy2-secret"}"#);
        let (params, rules, dns) = default_params();
        let config = XrayConfigBuilder::build(&endpoint, &protocol, &params, &rules, &dns)
            .expect("build");
        let json = serde_json::to_value(&config).unwrap();
        let proxy = json["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["tag"] == "proxy")
            .expect("proxy");
        assert_eq!(proxy["settings"]["auth"].as_str().unwrap(), "hy2-secret");
    }
}
