#![allow(dead_code)]

use base64::Engine;
use serde::Deserialize;
use url::Url;
use xray_tui_core::protocol::Protocol;
use xray_tui_db::models::Profile;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported URL scheme")]
    UnsupportedScheme,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("missing required field: {0}")]
    MissingField(String),
}

pub type Result<T> = std::result::Result<T, ImportError>;

/// VmessQRCode JSON structure used in vmess:// share links.
#[derive(Deserialize)]
struct VmessQRCode {
    #[serde(default)]
    v: i32,
    #[serde(default)]
    ps: String,
    #[serde(default)]
    add: String,
    #[serde(default)]
    port: i32,
    #[serde(default)]
    id: String,
    #[serde(default)]
    aid: i32,
    #[serde(default)]
    scy: String,
    #[serde(default)]
    net: String,
    #[serde(default)]
    #[allow(dead_code)]
    r#type: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    tls: String,
    #[serde(default)]
    sni: String,
    #[serde(default)]
    alpn: String,
    #[serde(default)]
    fp: String,
    #[serde(default)]
    insecure: String,
}

/// Parse a share URL into a Profile.
pub fn parse_share_url(url: &str) -> Result<Profile> {
    if url.starts_with("vmess://") {
        parse_vmess(url)
    } else if url.starts_with("vless://") {
        parse_vless(url)
    } else if url.starts_with("ss://") {
        parse_shadowsocks(url)
    } else if url.starts_with("trojan://") {
        parse_trojan(url)
    } else if url.starts_with("socks://") {
        parse_socks(url)
    } else if url.starts_with("hysteria2://") || url.starts_with("hy2://") {
        parse_hysteria2(url)
    } else if url.starts_with("hysteria://") || url.starts_with("hy://") {
        parse_hysteria(url)
    } else if url.starts_with("tuic://") {
        parse_tuic(url)
    } else if url.starts_with("naive+https://") || url.starts_with("naive+quic://") {
        parse_naive(url)
    } else if url.starts_with("anytls://") {
        parse_anytls(url)
    } else if url.starts_with("shadowtls://") {
        parse_shadowtls(url)
    } else if url.starts_with("wireguard://") {
        parse_wireguard(url)
    } else {
        Err(ImportError::UnsupportedScheme)
    }
}

/// Format a Profile back into a share URL string.
pub fn format_share_url(profile: &Profile) -> Result<String> {
    let protocol = Protocol::try_from_i32(profile.config_type)
        .ok_or_else(|| ImportError::Parse("unknown config type".into()))?;
    match protocol {
        Protocol::Vmess => format_vmess(profile),
        Protocol::Vless => format_vless(profile),
        Protocol::Shadowsocks | Protocol::Shadowsocks2022 => format_shadowsocks(profile),
        Protocol::Trojan => format_trojan(profile),
        Protocol::Socks => format_socks(profile),
        Protocol::Hysteria2 => format_hysteria2(profile),
        Protocol::Hysteria => format_hysteria(profile),
        Protocol::Tuic => format_tuic(profile),
        Protocol::Naive => format_naive(profile),
        Protocol::AnyTls => format_anytls(profile),
        Protocol::ShadowTls => format_shadowtls(profile),
        Protocol::WireGuard => format_wireguard(profile),
        _ => Err(ImportError::UnsupportedScheme),
    }
}

// ── VMess ───────────────────────────────────────────────────────────────

fn parse_vmess(url: &str) -> Result<Profile> {
    let b64 = url.strip_prefix("vmess://").unwrap_or(url);
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| ImportError::Parse("invalid base64 in vmess URL".into()))?;
    let qr: VmessQRCode = serde_json::from_slice(&decoded)?;

    let mut profile = base_profile(Protocol::Vmess, &qr.add, qr.port);
    profile.remarks = Some(qr.ps).filter(|s| !s.is_empty());
    profile.user_id = Some(qr.id).filter(|s| !s.is_empty());
    profile.security = Some(qr.scy).filter(|s| !s.is_empty());
    profile.network = Some(qr.net).filter(|s| !s.is_empty());

    let mut stream = serde_json::Map::new();
    if !qr.tls.is_empty() {
        stream.insert("tls.enable".into(), serde_json::Value::Bool(true));
    }
    if !qr.sni.is_empty() {
        stream.insert("sni".into(), serde_json::Value::String(qr.sni));
    }
    if !qr.alpn.is_empty() {
        stream.insert("alpn".into(), serde_json::Value::String(qr.alpn));
    }
    if !qr.fp.is_empty() {
        stream.insert("fingerprint".into(), serde_json::Value::String(qr.fp));
    }
    if qr.insecure == "1" {
        stream.insert("allow_insecure".into(), serde_json::Value::Bool(true));
    }
    if !qr.host.is_empty() {
        stream.insert("ws.host".into(), serde_json::Value::String(qr.host));
    }
    if !qr.path.is_empty() {
        stream.insert("ws.path".into(), serde_json::Value::String(qr.path));
    }
    if !stream.is_empty() {
        profile.stream_settings = Some(serde_json::to_string(&stream)?);
    }
    Ok(profile)
}

fn format_vmess(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let qr = serde_json::json!({
        "v": 2,
        "ps": profile.remarks.as_deref().unwrap_or(""),
        "add": add,
        "port": port,
        "id": profile.user_id.as_deref().unwrap_or(""),
        "aid": 0,
        "scy": profile.security.as_deref().unwrap_or("auto"),
        "net": profile.network.as_deref().unwrap_or("tcp"),
        "type": "none",
        "host": "",
        "path": "",
        "tls": profile.stream_settings.as_deref().unwrap_or(""),
        "sni": "",
        "alpn": "",
        "fp": "",
        "insecure": "0",
    });
    let json = serde_json::to_string(&qr)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(json);
    Ok(format!("vmess://{b64}"))
}

// ── VLESS ───────────────────────────────────────────────────────────────

fn parse_vless(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::Vless, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(0) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());
    if !parsed.username().is_empty() {
        profile.user_id = Some(parsed.username().to_string());
    }
    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "flow" => {
                let mut ps = serde_json::Map::new();
                ps.insert("flow".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "security" => {
                // tls/none
            }
            "sni" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "fp" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("fingerprint".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "alpn" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("alpn".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "allowInsecure" | "allow_insecure" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("allow_insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "type" => {
                if v == "tcp" || v == "ws" || v == "grpc" {
                    profile.network = Some(v.to_string());
                }
            }
            "path" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("ws.path".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "host" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("ws.host".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "serviceName" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("grpc.serviceName".into(), serde_json::Value::String(v.to_string()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "encryption" => { /* ignored */ }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_vless(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let userinfo = profile.user_id.as_deref().unwrap_or("");
    let mut query: Vec<(String, String)> = Vec::new();
    query.push(("encryption".into(), "none".into()));

    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(flow) = v.get("flow").and_then(|f| f.as_str()) {
                query.push(("flow".into(), flow.to_string()));
            }
        }
    }
    if let Some(ss) = &profile.stream_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ss) {
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()) {
                query.push(("sni".into(), sni.to_string()));
            }
            if let Some(fp) = v.get("fingerprint").and_then(|f| f.as_str()) {
                query.push(("fp".into(), fp.to_string()));
            }
        }
    }

    let qs = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("vless://{userinfo}@{add}:{port}?{qs}{fragment}"))
}

// ── Shadowsocks (SIP002) ────────────────────────────────────────────────

fn parse_shadowsocks(url: &str) -> Result<Profile> {
    // SIP002: ss://base64(method:password)@host:port?plugin=...#tag
    let rest = url.strip_prefix("ss://").unwrap_or(url);
    // Split on '@' to separate userinfo from host
    let (userinfo_b64, rest2) = rest.split_once('@')
        .ok_or_else(|| ImportError::Parse("missing @ in ss:// URL".into()))?;

    let (host_part, fragment) = rest2.split_once('#')
        .map(|(h, f)| (h, Some(f.to_string())))
        .unwrap_or((rest2, None));

    let (address, port_str) = host_part.rsplit_once(':')
        .ok_or_else(|| ImportError::Parse("missing port in ss:// URL".into()))?;
    let trimmed = userinfo_b64.trim_end_matches('=');
    // Re-pad to correct length for STANDARD decoder
    let padded = match trimmed.len() % 4 {
        2 => format!("{trimmed}=="),
        3 => format!("{trimmed}="),
        _ => trimmed.to_string(),
    };
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .map_err(|_| ImportError::Parse("invalid base64 in ss://".into()))?;
    let userinfo = String::from_utf8_lossy(&decoded);

    let (method, password) = userinfo.split_once(':')
        .ok_or_else(|| ImportError::Parse("missing : in ss:// userinfo".into()))?;

    let port: i32 = port_str.parse().map_err(|_| ImportError::Parse("invalid port".into()))?;
    let mut profile = base_profile(Protocol::Shadowsocks, address, port);
    profile.remarks = fragment;
    profile.user_id = Some(password.to_string());
    let mut ps = serde_json::Map::new();
    ps.insert("method".into(), serde_json::Value::String(method.to_string()));
    profile.protocol_settings = Some(serde_json::to_string(&ps)?);


    Ok(profile)
}

fn format_shadowsocks(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let method = profile.protocol_settings.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("method").and_then(|m| m.as_str().map(String::from)))
        .unwrap_or_else(|| "aes-256-gcm".into());
    let password = profile.user_id.as_deref().unwrap_or("");
    let userinfo = base64::engine::general_purpose::STANDARD
        .encode(format!("{method}:{password}"));
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("ss://{userinfo}@{add}:{port}{fragment}"))
}

// ── Trojan ──────────────────────────────────────────────────────────────

fn parse_trojan(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::Trojan, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    if !parsed.username().is_empty() {
        profile.user_id = Some(parsed.username().to_string());
    }

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "sni" | "peer" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "allowInsecure" | "allow_insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("allow_insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "fp" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("fingerprint".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_trojan(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let pw = profile.user_id.as_deref().unwrap_or("");
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()) {
                query.push(("sni".into(), sni.to_string()));
            }
        }
    }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("trojan://{pw}@{add}:{port}{qs}{fragment}"))
}

// ── SOCKS ───────────────────────────────────────────────────────────────

fn parse_socks(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::Socks, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(1080) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());
    if !parsed.username().is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert("username".into(), serde_json::Value::String(parsed.username().to_string()));
        if let Some(pw) = parsed.password() {
            ps.insert("password".into(), serde_json::Value::String(pw.to_string()));
        }
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    }
    Ok(profile)
}

fn format_socks(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("socks://@{add}:{port}{fragment}"))
}

// ── Hysteria2 ───────────────────────────────────────────────────────────

fn parse_hysteria2(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url.replace("hy2://", "hysteria2://").as_str())?;
    let mut profile = base_profile(Protocol::Hysteria2, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "auth" | "password" => profile.user_id = Some(v.to_string()),
            "obfs" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "obfs-password" | "obfs_password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs_password".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_hysteria2(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(auth) = &profile.user_id {
        query.push(("auth".into(), auth.clone()));
    }
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(obfs) = v.get("obfs").and_then(|o| o.as_str()).filter(|s| !s.is_empty()) {
                query.push(("obfs".into(), obfs.to_string()));
            }
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()).filter(|s| !s.is_empty()) {
                query.push(("sni".into(), sni.to_string()));
            }
        }
    }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("hysteria2://{add}:{port}{qs}{fragment}"))
}

// ── Hysteria v1 ─────────────────────────────────────────────────────────

fn parse_hysteria(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url.replace("hy://", "hysteria://").as_str())?;
    let mut profile = base_profile(Protocol::Hysteria, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "protocol" | "type" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("protocol".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "auth" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("auth".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "obfs" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "upmbps" | "up_mbps" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("up_mbps".into(), serde_json::Value::Number(serde_json::Number::from(v.parse::<i64>().unwrap_or(100))));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "downmbps" | "down_mbps" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("down_mbps".into(), serde_json::Value::Number(serde_json::Number::from(v.parse::<i64>().unwrap_or(100))));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_hysteria(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(auth) = v.get("auth").and_then(|a| a.as_str()).filter(|s| !s.is_empty()) {
                query.push(("auth".into(), auth.to_string()));
            }
            if let Some(up) = v.get("up_mbps").and_then(|u| u.as_i64()) {
                query.push(("upmbps".into(), up.to_string()));
            }
            if let Some(down) = v.get("down_mbps").and_then(|d| d.as_i64()) {
                query.push(("downmbps".into(), down.to_string()));
            }
        }
    }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("hysteria://{add}:{port}{qs}{fragment}"))
}

// ── TUIC ────────────────────────────────────────────────────────────────

fn parse_tuic(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::Tuic, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "uuid" => profile.user_id = Some(v.to_string()),
            "password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "congestion_control" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("congestion_control".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "udp_relay_mode" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("udp_relay_mode".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "allow_insecure" | "insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_tuic(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let uuid = profile.user_id.as_deref().unwrap_or("");
    let mut query: Vec<(String, String)> = Vec::new();
    query.push(("uuid".into(), uuid.to_string()));
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pw) = v.get("password").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("password".into(), pw.to_string()));
            }
            if let Some(cc) = v.get("congestion_control").and_then(|c| c.as_str()).filter(|s| !s.is_empty()) {
                query.push(("congestion_control".into(), cc.to_string()));
            }
            if let Some(urm) = v.get("udp_relay_mode").and_then(|u| u.as_str()).filter(|s| !s.is_empty()) {
                query.push(("udp_relay_mode".into(), urm.to_string()));
            }
        }
    }
    let qs = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("tuic://{add}:{port}?{qs}{fragment}"))
}

// ── Naïve ───────────────────────────────────────────────────────────────

fn parse_naive(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::Naive, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());
    if !parsed.username().is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert("user".into(), serde_json::Value::String(parsed.username().to_string()));
        if let Some(pw) = parsed.password() {
            ps.insert("password".into(), serde_json::Value::String(pw.to_string()));
        }
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    }
    Ok(profile)
}

fn format_naive(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let user = profile.protocol_settings.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("user").and_then(|u| u.as_str().map(String::from)))
        .unwrap_or_default();
    let password = profile.protocol_settings.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("password").and_then(|p| p.as_str().map(String::from)))
        .unwrap_or_default();
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    let userinfo = if user.is_empty() { String::new() } else { format!("{user}:{password}@") };
    Ok(format!("naive+https://{userinfo}{add}:{port}{fragment}"))
}

// ── AnyTLS ──────────────────────────────────────────────────────────────

fn parse_anytls(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::AnyTls, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "password" | "auth" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "insecure" | "allow_insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_anytls(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pw) = v.get("password").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("password".into(), pw.to_string()));
            }
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()).filter(|s| !s.is_empty()) {
                query.push(("sni".into(), sni.to_string()));
            }
        }
    }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("anytls://{add}:{port}{qs}{fragment}"))
}

// ── ShadowTLS ───────────────────────────────────────────────────────────

fn parse_shadowtls(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    let mut profile = base_profile(Protocol::ShadowTls, parsed.host_str().unwrap_or(""), parsed.port().unwrap_or(443) as i32);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "version" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("version".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_shadowtls(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let remark = profile.remarks.as_deref().unwrap_or("");
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pw) = v.get("password").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("password".into(), pw.to_string()));
            }
            if let Some(ver) = v.get("version").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                query.push(("version".into(), ver.to_string()));
            }
        }
    }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("shadowtls://{add}:{port}{qs}{fragment}"))
}

// ── WireGuard ───────────────────────────────────────────────────────────

fn parse_wireguard(url: &str) -> Result<Profile> {
    let parsed = Url::parse(url)?;
    // WireGuard URLs may not have a host:port authority; use query params
    let mut profile = base_profile(Protocol::WireGuard, "", 0);
    profile.remarks = parsed.fragment().map(|s| s.to_string());

    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "private_key" | "privateKey" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("private_key".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "public_key" | "publicKey" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("public_key".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "endpoint" => {
                if let Some((ep_addr, ep_port)) = v.rsplit_once(':') {
                    profile.address = Some(ep_addr.to_string());
                    profile.port = ep_port.parse::<i32>().ok();
                }
            }
            "allowed_ips" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("allowed_ips".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "mtu" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("mtu".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "dns" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("dns".into(), serde_json::Value::String(v.to_string()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_wireguard(profile: &Profile) -> Result<String> {
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pk) = v.get("private_key").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("privateKey".into(), pk.to_string()));
            }
            if let Some(pubk) = v.get("public_key").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("publicKey".into(), pubk.to_string()));
            }
        }
    }
    if let Some(addr) = &profile.address {
        if let Some(port) = profile.port {
            query.push(("endpoint".into(), format!("{addr}:{port}")));
        }
    }
    let qs = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("wireguard://?{qs}{fragment}"))
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn base_profile(protocol: Protocol, address: &str, port: i32) -> Profile {
    let id = uuid::Uuid::new_v4().to_string();
    Profile {
        id,
        config_type: protocol.to_i32(),
        core_type: "auto".into(),
        remarks: None,
        address: if address.is_empty() { None } else { Some(address.to_string()) },
        port: if port > 0 { Some(port) } else { None },
        user_id: None,
        security: None,
        network: None,
        stream_settings: None,
        protocol_settings: None,
        is_sub: Some(0),
        sub_id: None,
        group_id: None,
        sort_order: None,
        is_active: Some(0),
        created_at: None,
        updated_at: None,
    }
}

fn addr_port(profile: &Profile) -> (String, i32) {
    let addr = profile.address.as_deref().unwrap_or("").to_string();
    let port = profile.port.unwrap_or(0);
    (addr, port)
}

fn stream_settings(existing: Option<&str>) -> serde_json::Map<String, serde_json::Value> {
    existing
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object()
        .cloned()
        .unwrap_or_default()
}

fn protocol_settings(existing: Option<&str>) -> serde_json::Map<String, serde_json::Value> {
    existing
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object()
        .cloned()
        .unwrap_or_default()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_vmess() {
        let mut p = base_profile(Protocol::Vmess, "example.com", 443);
        p.user_id = Some("uuid-here".into());
        p.network = Some("ws".into());
        p.stream_settings = Some(r#"{"ws.path":"/api","tls.enable":true}"#.into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("vmess://"));
        let parsed = parse_share_url(&url).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
        assert_eq!(parsed.address, p.address);
        assert_eq!(parsed.port, p.port);
    }

    #[test]
    fn roundtrip_vless() {
        let mut p = base_profile(Protocol::Vless, "server.com", 443);
        p.user_id = Some("uuid".into());
        p.remarks = Some("my vless".into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("vless://"));
        let parsed = parse_share_url(&url).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
    }

    #[test]
    fn roundtrip_shadowsocks() {
        let mut p = base_profile(Protocol::Shadowsocks, "ss.example", 1080);
        p.user_id = Some("password123".into());
        let mut ps = serde_json::Map::new();
        ps.insert("method".into(), serde_json::Value::String("aes-256-gcm".into()));
        p.protocol_settings = Some(serde_json::to_string(&ps).unwrap());
        p.remarks = Some("myss".into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("ss://"));
        let parsed = parse_share_url(&url).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
    }

    #[test]
    fn roundtrip_trojan() {
        let mut p = base_profile(Protocol::Trojan, "trojan.example", 443);
        p.user_id = Some("password".into());
        p.remarks = Some("troj".into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("trojan://"));
        let parsed = parse_share_url(&url).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
    }

    #[test]
    fn parse_vmess_real() {
        // Minimal synthetic vmess://
        let qr = serde_json::json!({
            "v": 2, "ps": "test", "add": "1.2.3.4", "port": 443,
            "id": "uuid", "aid": 0, "scy": "auto", "net": "tcp",
            "type": "none", "host": "", "path": "", "tls": "",
            "sni": "", "alpn": "", "fp": "", "insecure": "0",
        });
        let b64 = base64::engine::general_purpose::STANDARD.encode(serde_json::to_string(&qr).unwrap());
        let url = format!("vmess://{b64}");
        let p = parse_share_url(&url).unwrap();
        assert_eq!(p.address.as_deref(), Some("1.2.3.4"));
        assert_eq!(p.port, Some(443));
    }

    #[test]
    fn unsupported_scheme() {
        assert!(matches!(parse_share_url("unknown://x"), Err(ImportError::UnsupportedScheme)));
    }
}
