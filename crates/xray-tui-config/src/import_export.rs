#![allow(dead_code)]

use serde::Deserialize;
use xray_tui_core::protocol::Protocol;
use xray_tui_db::models::Profile;


#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported URL scheme")]
    UnsupportedScheme,
    #[error("parse error: {0}")]
    Parse(String),
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

/// Ordered list of all protocol parsers, tried in order for fallback.
const PARSE_ORDER: &[fn(&str) -> Result<Profile>] = &[
    parse_vmess,
    parse_vless,
    parse_trojan,
    parse_shadowsocks,
    parse_shadowsocksr,
    parse_hysteria2,
    parse_hysteria,
    parse_tuic,
    parse_socks,
    parse_http,
    parse_naive,
    parse_anytls,
    parse_shadowtls,
    parse_wireguard,
];

fn scheme_primary_index(scheme: &str) -> Option<usize> {
    match scheme {
        "vmess" => Some(0),
        "vless" => Some(1),
        "trojan" => Some(2),
        "ss" => Some(3),
        "ssr" => Some(4),
        "hysteria2" | "hy2" => Some(5),
        "hysteria" | "hy" => Some(6),
        "tuic" => Some(7),
        "socks" | "socks5" => Some(8),
        "http" => Some(9),
        "naive+https" | "naive+quic" => Some(10),
        "anytls" => Some(11),
        "shadowtls" => Some(12),
        "wireguard" => Some(13),
        _ => None,
    }
}
/// Parse a share URL into a Profile.
pub fn parse_share_url(url: &str) -> Result<Profile> {
    let scheme_end = url.find("://").unwrap_or(url.len());
    let scheme = &url[..scheme_end];

    let primary_idx = scheme_primary_index(scheme);

    // Unknown scheme — no fallback to try
    let Some(primary_idx) = primary_idx else {
        return Err(ImportError::UnsupportedScheme);
    };

    // Try primary parser first
    if let Ok(profile) = PARSE_ORDER[primary_idx](url) {
        return Ok(profile);
    }

    // Fallback: try all other parsers in order
    let mut last_error = ImportError::UnsupportedScheme;
    for (i, parser) in PARSE_ORDER.iter().enumerate() {
        if i == primary_idx {
            continue;
        }
        match parser(url) {
            Ok(profile) => return Ok(profile),
            Err(e) => last_error = e,
        }
    }
    Err(last_error)
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
        Protocol::ShadowsocksR => format_shadowsocksr(profile),
        Protocol::Http => format_http(profile),
        _ => Err(ImportError::UnsupportedScheme),
    }
}

// ── VMess ───────────────────────────────────────────────────────────────

fn parse_vmess(url: &str) -> Result<Profile> {
    let b64 = url.strip_prefix("vmess://").unwrap_or(url);
    let decoded = crate::base64_util::decode_base64(b64)
        .map_err(|e| ImportError::Parse(format!("invalid base64 in vmess URL: {e}")))?;
    // Use permissive JSON parser to handle single quotes, trailing commas, etc.
    let value = crate::permissive_json::permissive_json(&decoded)
        .map_err(|e| ImportError::Parse(format!("invalid JSON in vmess URL: {e}")))?;
    let qr: VmessQRCode = serde_json::from_value(value)
        .map_err(|e| ImportError::Parse(format!("invalid vmess QR structure: {e}")))?;

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
    let b64 = base64_simd::STANDARD.encode_to_string(&json);
    Ok(format!("vmess://{b64}"))
}

// ── VLESS ───────────────────────────────────────────────────────────────

fn parse_vless(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Vless, &parsed.host, parsed.port.unwrap_or(0) as i32);
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        profile.user_id = Some(parsed.username.clone());
    }
    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
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

    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
            && let Some(flow) = v.get("flow").and_then(|f| f.as_str()) {
                query.push(("flow".into(), flow.to_string()));
            }
    if let Some(ss) = &profile.stream_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ss) {
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()) {
                query.push(("sni".into(), sni.to_string()));
            }
            if let Some(fp) = v.get("fingerprint").and_then(|f| f.as_str()) {
                query.push(("fp".into(), fp.to_string()));
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
    let decoded = base64_simd::STANDARD
        .decode_to_vec(padded.as_bytes())
        .map_err(|_| ImportError::Parse("invalid base64 in ss://".into()))?;
    let userinfo = String::from_utf8_lossy(&decoded);
    let (method, password) = userinfo.split_once(':')
        .ok_or_else(|| ImportError::Parse("missing : in ss:// userinfo".into()))?;

    let port: i32 = port_str.parse().map_err(|_| ImportError::Parse("invalid port".into()))?;
    let protocol = if method.starts_with("2022-blake3-") {
        Protocol::Shadowsocks2022
    } else {
        Protocol::Shadowsocks
    };
    let mut profile = base_profile(protocol, address, port);
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
    let userinfo = base64_simd::STANDARD.encode_to_string(format!("{method}:{password}"));
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("ss://{userinfo}@{add}:{port}{fragment}"))
}

// ── Trojan ──────────────────────────────────────────────────────────────

fn parse_trojan(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Trojan, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    if !parsed.username.is_empty() {
        profile.user_id = Some(parsed.username.clone());
    }

    for (k, v) in &parsed.query_pairs {
        match k.as_ref() {
            "sni" | "peer" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "allowInsecure" | "allow_insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("allow_insecure".into(), serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "fp" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("fingerprint".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
            && let Some(sni) = v.get("sni").and_then(|s| s.as_str()) {
                query.push(("sni".into(), sni.to_string()));
            }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("trojan://{pw}@{add}:{port}{qs}{fragment}"))
}

// ── SOCKS ───────────────────────────────────────────────────────────────

fn parse_socks(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Socks, &parsed.host, parsed.port.unwrap_or(1080) as i32);
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert("username".into(), serde_json::Value::String(parsed.username.clone()));
        if let Some(pw) = &parsed.password {
            ps.insert("password".into(), serde_json::Value::String(pw.clone()));
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
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Hysteria2, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_ref() {
            "auth" | "password" => profile.user_id = Some(v.clone()),
            "obfs" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "obfs-password" | "obfs_password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs_password".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(obfs) = v.get("obfs").and_then(|o| o.as_str()).filter(|s| !s.is_empty()) {
                query.push(("obfs".into(), obfs.to_string()));
            }
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()).filter(|s| !s.is_empty()) {
                query.push(("sni".into(), sni.to_string()));
            }
        }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("hysteria2://{add}:{port}{qs}{fragment}"))
}

// ── Hysteria v1 ─────────────────────────────────────────────────────────

fn parse_hysteria(url: &str) -> Result<Profile> {
    let parsed = split_share_url(&url.replace("hy://", "hysteria://"))?;
    let mut profile = base_profile(Protocol::Hysteria, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_ref() {
            "protocol" | "type" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("protocol".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "auth" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("auth".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "obfs" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("obfs".into(), serde_json::Value::String(v.clone()));
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
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
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
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("hysteria://{add}:{port}{qs}{fragment}"))
}

// ── TUIC ────────────────────────────────────────────────────────────────

fn parse_tuic(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Tuic, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
            "uuid" => profile.user_id = Some(v.clone()),
            "password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "congestion_control" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("congestion_control".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "udp_relay_mode" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("udp_relay_mode".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
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
    let qs = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("tuic://{add}:{port}?{qs}{fragment}"))
}

// ── Naïve ───────────────────────────────────────────────────────────────

fn parse_naive(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::Naive, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert("user".into(), serde_json::Value::String(parsed.username.clone()));
        if let Some(pw) = &parsed.password {
            ps.insert("password".into(), serde_json::Value::String(pw.clone()));
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
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::AnyTls, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
            "password" | "auth" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "alpn" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("alpn".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pw) = v.get("password").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("password".into(), pw.to_string()));
            }
            if let Some(sni) = v.get("sni").and_then(|s| s.as_str()).filter(|s| !s.is_empty()) {
                query.push(("sni".into(), sni.to_string()));
            }
        }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("anytls://{add}:{port}{qs}{fragment}"))
}

// ── ShadowTLS ───────────────────────────────────────────────────────────

fn parse_shadowtls(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(Protocol::ShadowTls, &parsed.host, parsed.port.unwrap_or(443) as i32);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
            "password" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("password".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "version" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("version".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
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
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pw) = v.get("password").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("password".into(), pw.to_string()));
            }
            if let Some(ver) = v.get("version").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                query.push(("version".into(), ver.to_string()));
            }
        }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("shadowtls://{add}:{port}{qs}{fragment}"))
}

// ── WireGuard ───────────────────────────────────────────────────────────

fn parse_wireguard(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    // WireGuard URLs may not have a host:port authority; use query params
    let mut profile = base_profile(Protocol::WireGuard, "", 0);
    profile.remarks = parsed.fragment.clone();

    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
            "private_key" | "privateKey" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("private_key".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "public_key" | "publicKey" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("public_key".into(), serde_json::Value::String(v.clone()));
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
                ps.insert("allowed_ips".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "mtu" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("mtu".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "dns" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("dns".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            _ => {}
        }
    }
    Ok(profile)
}

fn format_wireguard(profile: &Profile) -> Result<String> {
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
            if let Some(pk) = v.get("private_key").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("privateKey".into(), pk.to_string()));
            }
            if let Some(pubk) = v.get("public_key").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
                query.push(("publicKey".into(), pubk.to_string()));
            }
        }
    if let Some(addr) = &profile.address
        && let Some(port) = profile.port {
            query.push(("endpoint".into(), format!("{addr}:{port}")));
        }
    let qs = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("wireguard://?{qs}{fragment}"))
}

// ── URL splitter ───────────────────────────────────────────────────────

/// Parsed URL components that avoid the edge-case failures of `url::Url::parse`
/// (Trojan `#` in password, `@` in query values, etc.).
struct UrlComponents {
    scheme: String,
    username: String,
    password: Option<String>,
    host: String,
    port: Option<u16>,
    path: Option<String>,
    query_pairs: Vec<(String, String)>,
    fragment: Option<String>,
}

/// Robust URL splitting that handles Trojan `#`-in-password and `@`-in-query edge cases.
/// Returns components without using `url::Url`.
fn split_share_url(url: &str) -> Result<UrlComponents> {
    // 1. Extract scheme
    let (scheme, rest) = url.split_once("://")
        .ok_or_else(|| ImportError::Parse("missing scheme in URL".into()))?;
    let scheme = scheme.to_string();
    let mut unparsed = rest;

    // 2. Extract userinfo (find the correct @ before any # or ?)
    let (username, password, rest) = find_userinfo(unparsed, &scheme);
    unparsed = rest;

    // 3. Extract fragment
    let fragment = unparsed.split_once('#').map(|(r, f)| {
        unparsed = r;
        f.to_string()
    });

    // 4. Extract query
    let query_pairs = if let Some((r, q)) = unparsed.split_once('?') {
        unparsed = r;
        parse_query_params(q)
    } else {
        Vec::new()
    };

    // 5. Extract path
    let path = unparsed.find('/').map(|pos| {
        let (r, p) = unparsed.split_at(pos);
        unparsed = r;
        percent_decode(p)
    });

    // 6. Remaining: host:port
    // Handle the case where userinfo was not present (unparsed is still the full body)
    let hostport = if unparsed.contains('@') {
        // Multiple @ signs — take the part after the last @
        unparsed.split_once('@').map(|(_, r)| r).unwrap_or(unparsed)
    } else {
        unparsed
    };

    let (host, port) = parse_hostport(hostport);

    Ok(UrlComponents {
        scheme,
        username,
        password,
        host,
        port,
        path,
        query_pairs,
        fragment,
    })
}

/// Find userinfo separator (`@`) with Trojan password `#` handling.
/// Returns `(username, password, rest_of_url)`.
/// When no `@` found at the correct position, username and password are empty and rest is the full body.
fn find_userinfo<'a>(s: &'a str, scheme: &str) -> (String, Option<String>, &'a str) {
    // Find the @ that appears before any # or ?
    let at_pos = s.find('@').and_then(|pos| {
        let earliest = s.find('#').or_else(|| s.find('?'));
        match earliest {
            None => Some(pos),
            Some(early) if pos < early => Some(pos),
            Some(_early) => {
                // Trojan special case: 16-char ASCII password with # in it
                // e.g., "8r<[9'l6hAO#8ZQi@host:port"
                if scheme.starts_with("trojan") && pos == 16 && s[..pos].is_ascii() {
                    // Validate that what follows @ is a host:port
                    let after_at = &s[pos + 1..];
                    let host_end = after_at.find('/')
                        .or_else(|| after_at.find('?'))
                        .or_else(|| after_at.find('#'))
                        .unwrap_or(after_at.len());
                    if host_end > 0 {
                        let candidate = &after_at[..host_end];
                        if candidate.contains(':') {
                            return Some(pos);
                        }
                    }
                }
                None // @ is after ? or # so it's part of query/fragment
            }
        }
    });

    match at_pos {
        Some(pos) => {
            let userinfo = &s[..pos];
            let rest = &s[pos + 1..];
            match userinfo.split_once(':') {
                Some((u, p)) => (percent_decode(u), Some(percent_decode(p)), rest),
                None => (percent_decode(userinfo), None, rest),
            }
        }
        None => (String::new(), None, s),
    }
}

/// Parse `host:port` string.
fn parse_hostport(s: &str) -> (String, Option<u16>) {
    // Handle IPv6: [::1]:port
    if let Some(inner) = s.strip_prefix('[') {
        if let Some((host, port_part)) = inner.split_once("]:") {
            let port = port_part.parse::<u16>().ok();
            return (format!("[{}]", host), port);
        }
        let trimmed = s.trim_end_matches(']');
        return (format!("[{}]", &trimmed[1..]), None);
    }

    match s.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => {
            let port = p.parse::<u16>().ok();
            (h.to_string(), port)
        }
        _ => (s.to_string(), None),
    }
}

/// Percent-decode a string. Fallback to original on failure.
fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val_sub(bytes[i + 1]);
            let lo = hex_val_sub(bytes[i + 2]);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[inline]
fn hex_val_sub(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Parse query string into `Vec<(String, String)>` — linear scan, no HashMap.
fn parse_query_params(query: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            result.push((k.to_string(), percent_decode(v)));
        } else if !pair.is_empty() {
            result.push((pair.to_string(), String::new()));
        }
    }
    result
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
        sub_uid: None,
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


// ── ShadowsocksR ─────────────────────────────────────────────────────────

/// Parse `ssr://` URL.
///
/// Format: `ssr://base64(host:port:protocol:method:obfs:base64(password)/?params)`
fn parse_shadowsocksr(url: &str) -> Result<Profile> {
    let b64 = url.strip_prefix("ssr://").unwrap_or(url);
    let decoded = crate::base64_util::decode_base64(b64)
        .map_err(|_| ImportError::Parse("invalid base64 in ssr URL".into()))?;
    let text = String::from_utf8(decoded)
        .map_err(|_| ImportError::Parse("invalid UTF-8 in ssr URL".into()))?;

    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() < 6 {
        return Err(ImportError::Parse("ssr: expected at least 6 colon-delimited fields".into()));
    }

    // Index from end for IPv6 support
    let raw_host = parts[..parts.len() - 5].join(":");
    let raw_port = parts[parts.len() - 5];
    let raw_protocol = parts[parts.len() - 4];
    let raw_method = parts[parts.len() - 3];
    let raw_obfs = parts[parts.len() - 2];
    let password_raw = parts[parts.len() - 1..].join(":");

    let (password, query_str) = password_raw
        .split_once("/?")
        .or_else(|| password_raw.split_once('?'))
        .unwrap_or((&password_raw, ""));

    let port: i32 = raw_port.parse()
        .map_err(|_| ImportError::Parse("ssr: invalid port".into()))?;

    let mut params = std::collections::HashMap::new();
    if !query_str.is_empty() {
        for pair in query_str.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                params.insert(k.to_string(), v.to_string());
            }
        }
    }

    let mut profile = base_profile(Protocol::ShadowsocksR, &raw_host, port);
    profile.user_id = Some(password.to_string());

    let mut ps = serde_json::Map::new();
    ps.insert("method".into(), serde_json::Value::String(raw_method.to_string()));
    ps.insert("protocol".into(), serde_json::Value::String(raw_protocol.to_string()));
    ps.insert("obfs".into(), serde_json::Value::String(raw_obfs.to_string()));

    // Decode base64 query params
    for (key, src_field) in [("obfsparam", "obfsparam"), ("protoparam", "protoparam"), ("group", "group")] {
        if let Some(val_b64) = params.get(src_field)
            && let Ok(bytes) = crate::base64_util::decode_base64(val_b64)
                && let Ok(val_decoded) = String::from_utf8(bytes) {
                    ps.insert((*key).into(), serde_json::Value::String(val_decoded));
                }
    }
    profile.protocol_settings = Some(serde_json::to_string(&ps)?);

    if let Some(remarks_b64) = params.get("remarks")
        && let Ok(bytes) = crate::base64_util::decode_base64(remarks_b64)
            && let Ok(decoded) = String::from_utf8(bytes) {
                profile.remarks = Some(decoded);
            }

    Ok(profile)
}

fn format_shadowsocksr(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let password = profile.user_id.as_deref().unwrap_or("");

    let (method, protocol, obfs) = profile.protocol_settings.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .map(|v| {
            let m = v.get("method").and_then(|m| m.as_str()).unwrap_or("rc4-md5");
            let p = v.get("protocol").and_then(|p| p.as_str()).unwrap_or("origin");
            let o = v.get("obfs").and_then(|o| o.as_str()).unwrap_or("plain");
            (m.to_string(), p.to_string(), o.to_string())
        })
        .unwrap_or_else(|| ("rc4-md5".into(), "origin".into(), "plain".into()));

    let mut query_str = String::new();
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
            && let Some(obfsparam) = v.get("obfsparam").and_then(|o| o.as_str()).filter(|s| !s.is_empty()) {
                let encoded = base64_simd::URL_SAFE_NO_PAD.encode_to_string(obfsparam);
                query_str.push_str(&format!("obfsparam={}&", encoded));
            }
    if let Some(remarks) = &profile.remarks {
        let encoded = base64_simd::URL_SAFE_NO_PAD.encode_to_string(remarks);
        query_str.push_str(&format!("remarks={}", encoded));
    }

    let raw = format!("{add}:{port}:{protocol}:{method}:{obfs}:{password}");
    let full = if query_str.is_empty() {
        raw
    } else {
        format!("{raw}/?{query_str}")
    };
    let encoded = base64_simd::URL_SAFE_NO_PAD.encode_to_string(full.as_bytes());
    Ok(format!("ssr://{encoded}"))
}

// ── HTTP ──────────────────────────────────────────────────────────────────

/// Parse `http://` proxy URL.
///
/// Format: `http://user:pass@host:port#remarks`
fn parse_http(url: &str) -> Result<Profile> {
    let rest = url.strip_prefix("http://").unwrap_or(url);
    // Split @ for userinfo
    let (userinfo, hostpart) = rest.split_once('@')
        .unwrap_or(("", rest));
    // Split # for fragment (remark)
    let (hostport, fragment) = hostpart.split_once('#')
        .map(|(h, f)| (h, Some(f.to_string())))
        .unwrap_or((hostpart, None));

    let (host, port_str) = hostport.rsplit_once(':')
        .ok_or_else(|| ImportError::Parse("http: missing port".into()))?;
    let port: i32 = port_str.parse()
        .map_err(|_| ImportError::Parse("http: invalid port".into()))?;

    let mut profile = base_profile(Protocol::Http, host, port);
    profile.remarks = fragment.filter(|s| !s.is_empty());

    if !userinfo.is_empty() {
        let mut ps = serde_json::Map::new();
        if let Some((username, password)) = userinfo.split_once(':') {
            ps.insert("username".into(), serde_json::Value::String(username.to_string()));
            ps.insert("password".into(), serde_json::Value::String(password.to_string()));
        } else {
            ps.insert("username".into(), serde_json::Value::String(userinfo.to_string()));
        }
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    }

    Ok(profile)
}

fn format_http(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let (user, pass) = profile.protocol_settings.as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .map(|v| {
            let u = v.get("username").and_then(|u| u.as_str()).unwrap_or("").to_string();
            let p = v.get("password").and_then(|p| p.as_str()).unwrap_or("").to_string();
            (u, p)
        })
        .unwrap_or_default();

    let userinfo = if user.is_empty() {
        String::new()
    } else if pass.is_empty() {
        format!("{user}@")
    } else {
        format!("{user}:{pass}@")
    };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() { String::new() } else { format!("#{remark}") };
    Ok(format!("http://{userinfo}{add}:{port}{fragment}"))
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
        let b64 = base64_simd::STANDARD.encode_to_string(serde_json::to_string(&qr).unwrap());
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
