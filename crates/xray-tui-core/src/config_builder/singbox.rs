use serde::Serialize;
use serde_json::{Value, json};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v2ray_api: Option<V2RayApi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clash_api: Option<ClashApiOptions>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClashApiOptions {
    pub external_controller: String,
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
            experimental: Some(ExperimentalConfig {
                v2ray_api: params.v2ray_api_enabled.then(|| V2RayApi {
                    listen: format!("127.0.0.1:{}", super::API_PORT),
                    stats: StatsConfig {
                        enabled: true,
                        outbounds: vec!["proxy", "direct"],
                    },
                }),
                clash_api: params.clash_api_enabled.then(|| ClashApiOptions {
                    external_controller: format!(
                        "127.0.0.1:{}",
                        params.clash_api_port.unwrap_or(super::CLASH_API_PORT)
                    ),
                }),
            }),
        })
    }
}

fn build_proxy_outbound(profile: &Profile) -> Result<Value, BuildError> {
    let protocol = Protocol::try_from_i32(profile.config_type).ok_or_else(|| {
        BuildError::InvalidProfile(format!("Unknown config_type: {}", profile.config_type))
    })?;

    let address = profile.address.as_str();
    let port = profile.port as u16;
    let user_id = ""; // TODO: cached on Profile
    let (p_settings, _s_settings) = parse_settings(profile);

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
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(100);
            let down_mbps = p_settings
                .get("down_mbps")
                .and_then(serde_json::Value::as_u64)
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
            if let Some(username) = p_settings
                .get("username")
                .or_else(|| p_settings.get("user"))
                && let Some(u) = username.as_str()
                && !u.is_empty()
            {
                out["username"] = json!(u);
                if let Some(password) = p_settings
                    .get("password")
                    .or_else(|| p_settings.get("pass"))
                    .and_then(|v| v.as_str())
                    && !password.is_empty()
                {
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
            if let Some(username) = p_settings
                .get("username")
                .or_else(|| p_settings.get("user"))
                && let Some(u) = username.as_str()
                && !u.is_empty()
            {
                out["username"] = json!(u);
                if let Some(password) = p_settings
                    .get("password")
                    .or_else(|| p_settings.get("pass"))
                    .and_then(|v| v.as_str())
                    && !password.is_empty()
                {
                    out["password"] = json!(password);
                }
            }
            Ok(out)
        }
        Protocol::ShadowsocksR => {
            let method = p_settings
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("aes-256-cfb");
            Ok(json!({
                "tag": "proxy",
                "type": "shadowsocksr",
                "server": address,
                "server_port": port,
                "method": method,
                "password": user_id,
                "obfs": p_settings.get("obfs").and_then(|v| v.as_str()).unwrap_or(""),
                "obfs_param": p_settings.get("obfs_param").and_then(|v| v.as_str()).unwrap_or(""),
                "protocol": p_settings.get("protocol").and_then(|v| v.as_str()).unwrap_or(""),
                "protocol_param": p_settings.get("protocol_param").and_then(|v| v.as_str()).unwrap_or(""),
            }))
        }
        Protocol::Hysteria => {
            let auth = p_settings
                .get("auth")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let up = p_settings
                .get("up_mbps")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(100);
            let down = p_settings
                .get("down_mbps")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(100);
            let mut out = json!({
                "tag": "proxy",
                "type": "hysteria",
                "server": address,
                "server_port": port,
                "up_mbps": up,
                "down_mbps": down,
            });
            if !auth.is_empty() {
                out["auth_str"] = json!(auth);
            }
            if let Some(obfs) = p_settings
                .get("obfs")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                out["obfs"] = json!(obfs);
            }
            // Hysteria v1 always uses TLS
            let mut tls = serde_json::Map::new();
            tls.insert("enabled".into(), json!(true));
            let sni = p_settings
                .get("sni")
                .and_then(|v| v.as_str())
                .unwrap_or(address);
            tls.insert("server_name".into(), json!(sni));
            if p_settings
                .get("insecure")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                tls.insert("insecure".into(), json!(true));
            }
            out["tls"] = json!(tls);
            Ok(out)
        }
        Protocol::Naive => {
            let username = p_settings
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or(user_id);
            let mut out = json!({
                "tag": "proxy",
                "type": "naive",
                "server": address,
                "server_port": port,
                "password": password,
            });
            if !username.is_empty() {
                out["username"] = json!(username);
            }
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::AnyTls => {
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut out = json!({
                "tag": "proxy",
                "type": "anytls",
                "server": address,
                "server_port": port,
                "password": password,
            });
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::ShadowTls => {
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let version = p_settings
                .get("version")
                .and_then(serde_json::Value::as_i64)
                .or_else(|| {
                    p_settings
                        .get("version")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                })
                .unwrap_or(3);
            let mut out = json!({
                "tag": "proxy",
                "type": "shadowtls",
                "server": address,
                "server_port": port,
                "password": password,
                "version": version,
            });
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::Tor => {
            let mut out = json!({
                "tag": "proxy",
                "type": "tor",
            });
            if let Some(data_dir) = p_settings
                .get("data_dir")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                out["data_directory"] = json!(data_dir);
            }
            Ok(out)
        }
        Protocol::Ssh => {
            let ssh_host = p_settings
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or(address);
            let ssh_port = p_settings
                .get("ssh_port")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_else(|| u64::from(port));
            let username = p_settings
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("root");
            let password = p_settings
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let private_key = p_settings
                .get("private_key")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut out = json!({
                "tag": "proxy",
                "type": "ssh",
                "server": ssh_host,
                "server_port": ssh_port,
                "user": username,
            });
            if !password.is_empty() {
                out["password"] = json!(password);
            }
            if !private_key.is_empty() {
                out["private_key"] = json!([private_key]);
            }
            Ok(out)
        }
        Protocol::Tailscale => {
            let mut out = json!({
                "tag": "proxy",
                "type": "tailscale",
            });
            if let Some(key) = p_settings
                .get("auth_key")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                out["auth_key"] = json!(key);
            }
            if let Some(url) = p_settings
                .get("control_url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                out["control_url"] = json!(url);
            }
            if p_settings
                .get("ephemeral")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                out["ephemeral"] = json!(true);
            }
            Ok(out)
        }
        Protocol::Vmess => {
            let security = "auto"; // TODO: from ProtocolConfig
            let mut out = json!({
                "tag": "proxy",
                "type": "vmess",
                "server": address,
                "server_port": port,
                "uuid": user_id,
                "security": security,
            });
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::Vless => {
            let flow = p_settings
                .get("flow")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut out = json!({
                "tag": "proxy",
                "type": "vless",
                "server": address,
                "server_port": port,
                "uuid": user_id,
            });
            if !flow.is_empty() {
                out["flow"] = json!(flow);
            }
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::Trojan => {
            let mut out = json!({
                "tag": "proxy",
                "type": "trojan",
                "server": address,
                "server_port": port,
                "password": user_id,
            });
            if let Some(tls) = build_tls(profile) {
                out["tls"] = tls;
            }
            Ok(out)
        }
        Protocol::WireGuard => {
            let private_key = p_settings
                .get("private_key")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let public_key = p_settings
                .get("public_key")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let allowed_ips = p_settings
                .get("allowed_ips")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0.0/0");
            let mtu = p_settings
                .get("mtu")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1420);

            // Peer endpoint: URL import sets profile.address:port; form stores in protocol_settings["endpoint"]
            let profile_addr = (!profile.address.is_empty()).then(|| profile.address.as_str());
            let (peer_addr, peer_port): (String, u16) = if let Some(addr) = profile_addr {
                (addr.to_string(), port)
            } else if let Some(ep) = p_settings.get("endpoint").and_then(|v| v.as_str()) {
                if let Some((host, port_str)) = ep.rsplit_once(':') {
                    (host.to_string(), port_str.parse::<u16>().unwrap_or(51820))
                } else {
                    (ep.to_string(), 51820)
                }
            } else {
                (String::new(), 0u16)
            };

            let mut out = json!({
                "tag": "proxy",
                "type": "wireguard",
                "private_key": private_key,
                "address": [],
                "mtu": mtu,
            });

            if !public_key.is_empty() && !peer_addr.is_empty() {
                let peer = json!({
                    "address": peer_addr,
                    "port": peer_port,
                    "public_key": public_key,
                    "allowed_ips": [allowed_ips],
                });
                out["peers"] = json!([peer]);
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
        .unwrap_or_else(|| json!({}));

    SingBoxDnsConfig { servers, hosts }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_comma_list(s: &str) -> Vec<&str> {
    s.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

/// Extract protocol_settings and stream_settings from profile.spec_blob.
/// These are JSON-encoded by ProfileMut setters and consumed by the builder.
fn parse_settings(profile: &Profile) -> (Value, Value) {
    let extra: Value = serde_json::from_slice(&profile.spec_blob)
        .unwrap_or_else(|_| json!({}));
    let p_settings = extra.get("protocol_settings").cloned().unwrap_or(json!({}));
    let s_settings = extra.get("stream_settings").cloned().unwrap_or(json!({}));
    (p_settings, s_settings)
}

/// Build the TLS sub-object for sing-box outbound.
/// Checks both `protocol_settings` (URL imports) and `stream_settings` (forms).
fn build_tls(profile: &Profile) -> Option<Value> {
    let (p_settings, s_settings) = parse_settings(profile);

    let enabled = p_settings
        .get("sni")
        .and_then(|v| v.as_str())
        .as_ref()
        .is_some_and(|s| !s.is_empty())
        || s_settings
            .get("tls.enable")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        || s_settings.get("security").and_then(|v| v.as_str()) == Some("reality")
        || s_settings
            .get("reality.show")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

    if !enabled {
        return None;
    }

    let mut tls = serde_json::Map::new();
    tls.insert("enabled".into(), json!(true));

    // server_name: protocol.sni > stream.sni > profile.address
    let sni = p_settings
        .get("sni")
        .and_then(|v| v.as_str())
        .or_else(|| s_settings.get("sni").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty())
        .or_else(|| (!profile.address.is_empty()).then_some(profile.address.as_str()));
    if let Some(sni) = sni {
        tls.insert("server_name".into(), json!(sni));
    }

    // insecure: protocol.insecure > protocol.allow_insecure > stream.allow_insecure
    let insecure = p_settings
        .get("insecure")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        || p_settings
            .get("allow_insecure")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        || s_settings
            .get("allow_insecure")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
    if insecure {
        tls.insert("insecure".into(), json!(true));
    }

    // alpn: comma-separated string
    let alpn = p_settings
        .get("alpn")
        .and_then(|v| v.as_str())
        .or_else(|| s_settings.get("alpn").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty());
    if let Some(a) = alpn {
        let parts: Vec<&str> = a
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if !parts.is_empty() {
            tls.insert("alpn".into(), json!(parts));
        }
    }

    // utls.fingerprint: stream.fingerprint
    if let Some(fp) = s_settings
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        let mut utls = serde_json::Map::new();
        utls.insert("enabled".into(), json!(true));
        utls.insert("fingerprint".into(), json!(fp));
        tls.insert("utls".into(), json!(utls));
    }

    // reality: check both URL-import (security) and form (reality.show) paths
    let is_reality = s_settings.get("security").and_then(|v| v.as_str()) == Some("reality")
        || s_settings
            .get("reality.show")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
    if is_reality {
        let mut reality = serde_json::Map::new();
        reality.insert("enabled".into(), json!(true));
        // stream_settings reality keys: URL import uses flat "pbk"/"sid"/"spx"
        // realitySettings sub-object also checked for backwards compat
        if let Some(pbk) = s_settings
            .get("pbk")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                s_settings
                    .get("realitySettings")
                    .and_then(|v| v.as_object())
                    .and_then(|m| m.get("publicKey").and_then(|v| v.as_str()))
            })
        {
            reality.insert("public_key".into(), json!(pbk));
        }
        if let Some(sid) = s_settings
            .get("sid")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                s_settings
                    .get("realitySettings")
                    .and_then(|v| v.as_object())
                    .and_then(|m| m.get("shortId").and_then(|v| v.as_str()))
            })
        {
            reality.insert("short_id".into(), json!(sid));
        }
        if let Some(spx) = s_settings
            .get("spx")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                s_settings
                    .get("realitySettings")
                    .and_then(|v| v.as_object())
                    .and_then(|m| m.get("spiderX").and_then(|v| v.as_str()))
            })
        {
            reality.insert("short_id".into(), json!(spx));
        }
        // Only add reality block if it has meaningful content beyond "enabled"
        if reality.len() > 1 {
            tls.insert("reality".into(), json!(reality));
        }
    }

    Some(json!(tls))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_db::models::{DnsSetting, Profile, RoutingRule};
    use xray_tui_config::import_export::ProfileMut;

    fn test_profile(config_type: i32) -> Profile {
        let mut profile = Profile {
            id: 0,
            sig: 0,
            cred_hash: 0,
            proto_kind: String::new(),
            spec_blob: Vec::new(),
            config_type,
            core_type: String::new(),
            address: "example.com".to_string(),
            port: 443,
            transport: Some("tcp".to_string()),
            security: Some("auto".to_string()),
            created_at: 0,
            extension: Default::default(),
            server_stat: Default::default(),
        };
        let extra = serde_json::json!({
            "remarks": "test",
            "user_id": "test-uuid-or-pass",
        });
        profile.spec_blob = serde_json::to_vec(&extra).unwrap_or_default();
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
            clash_api_port: None,
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
        let tags: Vec<&str> = outbounds.iter().filter_map(|o| o["tag"].as_str()).collect();
        assert!(tags.contains(&"direct"), "missing direct");
        assert!(tags.contains(&"block"), "missing block");
    }

    #[test]
    fn singbox_tuic_config() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "tuic");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_hysteria2_config() {
        let mut profile = test_profile(Protocol::Hysteria2.to_i32());
        profile.set_protocol_settings(
            Some(r#"{"password": "sekret", "up_mbps": 50, "down_mbps": 200}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
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
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "shadowsocks");
    }

    #[test]
    fn singbox_socks_config() {
        let profile = test_profile(Protocol::Socks.to_i32());
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "socks");
    }

    #[test]
    fn singbox_http_config() {
        let profile = test_profile(Protocol::Http.to_i32());
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "http");
    }

    #[test]
    fn singbox_tuic_with_tls() {
        let mut profile = test_profile(Protocol::Tuic.to_i32());
        profile.set_protocol_settings(Some(r#"{"password": "pass123"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
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
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
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
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let exp = &json["experimental"];
        let v2ray = &exp["v2ray_api"];
        assert_eq!(
            v2ray["listen"],
            format!("127.0.0.1:{}", crate::config_builder::API_PORT)
        );
        assert_eq!(v2ray["stats"]["enabled"], true);
        // clash_api should be absent when clash_api_enabled is false
        assert!(exp["clash_api"].is_null(), "clash_api should be absent");
    }

    #[test]
    fn singbox_clash_api_config() {
        let profile = test_profile(Protocol::Tuic.to_i32());
        let (mut params, rules, dns) = default_params();
        params.clash_api_enabled = true;
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let exp = &json["experimental"];
        // clash_api should be present when clash_api_enabled is true
        assert_eq!(
            exp["clash_api"]["external_controller"],
            format!("127.0.0.1:{}", crate::config_builder::CLASH_API_PORT)
        );
        // v2ray_api should still be present (it was true in default_params)
        assert!(exp["v2ray_api"].is_object(), "v2ray_api should be present");
    }

    #[test]
    fn singbox_shadowsocksr_config() {
        let mut profile = test_profile(Protocol::ShadowsocksR.to_i32());
        profile.config_type = Protocol::ShadowsocksR.to_i32();
        profile.set_protocol_settings(Some(r#"{"method":"aes-256-cfb","obfs":"tls1.2_ticket_auth","obfs_param":"www.example.com","protocol":"auth_aes128_md5","protocol_param":"test"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "shadowsocksr");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["method"], "aes-256-cfb");
        assert_eq!(proxy["obfs"], "tls1.2_ticket_auth");
    }

    #[test]
    fn singbox_hysteria_config() {
        let mut profile = test_profile(Protocol::Hysteria.to_i32());
        profile.config_type = Protocol::Hysteria.to_i32();
        profile.set_protocol_settings(Some(
            r#"{"auth":"test123","up_mbps":50,"down_mbps":100,"sni":"custom.example.com"}"#
                .to_string(),
        ));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "hysteria");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["auth_str"], "test123");
        assert_eq!(proxy["up_mbps"], 50);
        assert_eq!(proxy["down_mbps"], 100);
        assert_eq!(proxy["tls"]["server_name"], "custom.example.com");
    }

    #[test]
    fn singbox_naive_config() {
        let mut profile = test_profile(Protocol::Naive.to_i32());
        profile.config_type = Protocol::Naive.to_i32();
        profile.set_protocol_settings(Some(r#"{"user":"me","password":"pass456"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "naive");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_anytls_config() {
        let mut profile = test_profile(Protocol::AnyTls.to_i32());
        profile.config_type = Protocol::AnyTls.to_i32();
        profile.set_protocol_settings(
            Some(r#"{"password":"any-secret","sni":"tls.example.com"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "anytls");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_shadowtls_config() {
        let mut profile = test_profile(Protocol::ShadowTls.to_i32());
        profile.config_type = Protocol::ShadowTls.to_i32();
        profile.set_protocol_settings(Some(
            r#"{"password":"shadow-pw","version":"3","sni":"shadow.example.com"}"#.to_string(),
        ));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "shadowtls");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["version"], 3);
    }

    #[test]
    fn singbox_tor_config() {
        let profile = test_profile(Protocol::Tor.to_i32());
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "tor");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_ssh_config() {
        let mut profile = test_profile(Protocol::Ssh.to_i32());
        profile.set_protocol_settings(Some(
            r#"{"host":"ssh.example.com","ssh_port":2222,"username":"admin","password":"ssh-pw"}"#
                .to_string(),
        ));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "ssh");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["server"], "ssh.example.com");
        assert_eq!(proxy["server_port"], 2222);
        assert_eq!(proxy["user"], "admin");
    }

    #[test]
    fn singbox_tailscale_config() {
        let mut profile = test_profile(Protocol::Tailscale.to_i32());
        profile.set_protocol_settings(Some(
            r#"{"auth_key":"tskey-auth-xxxx","control_url":"https://control.tailscale.com"}"#
                .to_string(),
        ));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "tailscale");
        assert_has_standard_outbounds(&json);
    }

    #[test]
    fn singbox_vmess_config() {
        let mut profile = test_profile(Protocol::Vmess.to_i32());
        profile.set_stream_settings(
            Some(r#"{"tls.enable":true,"sni":"vmess.example.com"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "vmess");
        assert_has_standard_outbounds(&json);
        assert_eq!(
            json["outbounds"].as_array().unwrap()[0]["tls"]["enabled"],
            true
        );
    }

    #[test]
    fn singbox_vless_config() {
        let mut profile = test_profile(Protocol::Vless.to_i32());
        profile.set_protocol_settings(
            Some(r#"{"flow":"xtls-rprx-vision","sni":"vless.example.com"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "vless");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["flow"], "xtls-rprx-vision");
        assert_eq!(proxy["tls"]["server_name"], "vless.example.com");
    }

    #[test]
    fn singbox_trojan_config() {
        let mut profile = test_profile(Protocol::Trojan.to_i32());
        profile.set_stream_settings(
            Some(r#"{"tls.enable":true,"sni":"trojan.example.com"}"#.to_string()));
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "trojan");
        assert_has_standard_outbounds(&json);
        assert_eq!(
            json["outbounds"].as_array().unwrap()[0]["tls"]["server_name"],
            "trojan.example.com"
        );
    }

    #[test]
    fn singbox_wireguard_config() {
        let mut profile = test_profile(Protocol::WireGuard.to_i32());
        profile.set_protocol_settings(Some(r#"{"private_key":"abc123def456","public_key":"pubkey789","allowed_ips":"0.0.0.0/0","mtu":1380,"endpoint":"wg.example.com:51820"}"#.to_string()));
        profile.address = String::new();
        profile.port = 0;
        let (params, rules, dns) = default_params();
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_singbox_top_level(&json);
        assert_proxy_outbound(&json, "wireguard");
        assert_has_standard_outbounds(&json);
        let proxy = &json["outbounds"].as_array().unwrap()[0];
        assert_eq!(proxy["mtu"], 1380);
        assert_eq!(proxy["peers"][0]["address"], "wg.example.com");
        assert_eq!(proxy["peers"][0]["port"], 51820);
    }

    #[test]
    fn singbox_truly_unsupported_protocol_returns_error() {
        // Dns is xray-only, not supported by sing-box outbound builder
        let profile = test_profile(Protocol::Dns.to_i32());
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
        let config = SingBoxConfigBuilder::build(&profile, &params, &rules, &dns).unwrap();
        let json = serde_json::to_value(&config).unwrap();
        let route = &json["route"];
        assert_eq!(route["rules"].as_array().unwrap().len(), 1);
        assert_eq!(route["rules"][0]["domain"][0], "example.com");
        assert_eq!(route["rules"][0]["outbound"], "direct");
        assert_eq!(route["final"], "proxy");
    }
}
