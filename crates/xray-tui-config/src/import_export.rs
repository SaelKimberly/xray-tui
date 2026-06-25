use serde::Deserialize;
use std::net::IpAddr;
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
    #[error("validation failed: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ImportError>;

/// `VmessQRCode` JSON structure used in vmess:// share links.
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
/// Strictest schemas first (low false-positive rate), most permissive last.
const PARSE_ORDER: &[fn(&str) -> Result<Profile>] = &[
    parse_shadowsocksr,
    parse_shadowsocks,
    parse_vmess,
    parse_vless,
    parse_trojan,
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
        "ssr" => Some(0),
        "ss" => Some(1),
        "vmess" => Some(2),
        "vless" => Some(3),
        "trojan" => Some(4),
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
/// Tries the scheme-mapped parser first, then falls back through all other parsers.
/// Validates the parsed profile against the given settings.
pub fn parse_share_url(url: &str, settings: &ValidationSettings) -> Result<Profile> {
    let scheme_end = url.find("://").unwrap_or(url.len());
    let scheme = &url[..scheme_end];

    let primary_idx = scheme_primary_index(scheme);
    let has_known_scheme = primary_idx.is_some();

    // Try primary parser first (if scheme is known)
    if let Some(idx) = primary_idx
        && let Ok(profile) = PARSE_ORDER[idx](url)
    {
        return normalize_and_validate(profile, settings);
    }

    // Fallback: try all parsers (skip primary if it was attempted)
    // For unknown schemes, if a parser succeeds but validation fails, continue trying.
    let mut last_error = ImportError::UnsupportedScheme;
    let mut validation_error = None;
    for (i, parser) in PARSE_ORDER.iter().enumerate() {
        if let Some(skip) = primary_idx
            && i == skip
        {
            continue;
        }
        match parser(url) {
            Ok(profile) => {
                match normalize_and_validate(profile, settings) {
                    Ok(validated) => return Ok(validated),
                    Err(e @ ImportError::Validation(_)) => {
                        // Parser matched but validation rejected — save this for later
                        // but continue trying other parsers (for unknown schemes)
                        if has_known_scheme {
                            return Err(e);
                        }
                        validation_error = Some(e);
                    }
                    Err(e) => {
                        // Parser returned Ok but validation failed with non-Validation error
                        if has_known_scheme {
                            return Err(e);
                        }
                        last_error = e;
                    }
                }
            }
            Err(e) => last_error = e,
        }
    }

    // If any parser matched but validation rejected, return that error.
    // Otherwise return the last parse error (or UnsupportedScheme if none matched).
    if let Some(ve) = validation_error {
        Err(ve)
    } else {
        Err(last_error)
    }
}

/// Apply remark normalization and all validation checks to a parsed profile.
fn normalize_and_validate(mut profile: Profile, settings: &ValidationSettings) -> Result<Profile> {
    if let Some(ref r) = profile.remarks.clone() {
        profile.remarks = Some(normalize_remark(r));
    }
    validate_required_fields(&profile)?;
    validate_host(&profile, settings)?;
    validate_security(&profile)?;
    Ok(profile)
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
    let raw = crate::base64_util::decode_base64(b64)
        .map_err(|e| ImportError::Parse(format!("invalid base64 in vmess URL: {e}")))?;

    // Trailing-garbage recovery: some providers append extra text after the JSON object
    let cleaned = if let Some(last_brace) = raw.iter().rposition(|&b| b == b'}') {
        if last_brace + 1 < raw.len() {
            &raw[..=last_brace]
        } else {
            &raw
        }
    } else {
        &raw
    };

    // Use permissive JSON parser to handle single quotes, trailing commas, etc.
    let value = crate::permissive_json::permissive_json(cleaned)
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
    let mut profile = base_profile(
        Protocol::Vless,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(0)),
    );
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        profile.user_id = Some(parsed.username.clone());
    }
    for (k, v) in &parsed.query_pairs {
        match k.as_str() {
            "flow" => {
                let mut ps = serde_json::Map::new();
                ps.insert("flow".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "security" => {
                if v == "reality" {
                    let mut ss = stream_settings(profile.stream_settings.as_deref());
                    ss.insert(
                        "security".into(),
                        serde_json::Value::String("reality".into()),
                    );
                    profile.stream_settings = Some(serde_json::to_string(&ss)?);
                }
            }
            "sni" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("sni".into(), serde_json::Value::String(v.clone()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "fp" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("fingerprint".into(), serde_json::Value::String(v.clone()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "alpn" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("alpn".into(), serde_json::Value::String(v.clone()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "allowInsecure" | "allow_insecure" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert(
                    "allow_insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "type" => {
                if v == "tcp" || v == "ws" || v == "grpc" {
                    profile.network = Some(v.clone());
                }
            }
            "path" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("ws.path".into(), serde_json::Value::String(v.clone()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "host" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert("ws.host".into(), serde_json::Value::String(v.clone()));
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "serviceName" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                ss.insert(
                    "grpc.serviceName".into(),
                    serde_json::Value::String(v.clone()),
                );
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "encryption" => { /* ignored */ }
            "pbk" | "publicKey" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                let rs = ss
                    .entry(String::from("realitySettings"))
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let Some(obj) = rs.as_object_mut() {
                    obj.insert("publicKey".into(), serde_json::Value::String(v.clone()));
                }
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "sid" | "shortId" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                let rs = ss
                    .entry(String::from("realitySettings"))
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let Some(obj) = rs.as_object_mut() {
                    obj.insert("shortId".into(), serde_json::Value::String(v.clone()));
                }
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "spx" | "spiderX" => {
                let mut ss = stream_settings(profile.stream_settings.as_deref());
                let rs = ss
                    .entry(String::from("realitySettings"))
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let Some(obj) = rs.as_object_mut() {
                    obj.insert("spiderX".into(), serde_json::Value::String(v.clone()));
                }
                profile.stream_settings = Some(serde_json::to_string(&ss)?);
            }
            "headerType" => { /* transport header type — defaults to "none" */ }
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
        && let Some(flow) = v.get("flow").and_then(|f| f.as_str())
    {
        query.push(("flow".into(), flow.to_string()));
    }
    if let Some(net) = &profile.network {
        query.push(("type".into(), net.clone()));
    }
    if let Some(ss) = &profile.stream_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ss)
    {
        if let Some(sni) = v.get("sni").and_then(|s| s.as_str()) {
            query.push(("sni".into(), sni.to_string()));
        }
        if let Some(fp) = v.get("fingerprint").and_then(|f| f.as_str()) {
            query.push(("fp".into(), fp.to_string()));
        }
        if v.get("security").and_then(|s| s.as_str()) == Some("reality") {
            query.push(("security".into(), "reality".into()));
        }
        if let Some(rs) = v.get("realitySettings").and_then(|r| r.as_object()) {
            if let Some(pbk) = rs.get("publicKey").and_then(|s| s.as_str()) {
                query.push(("pbk".into(), pbk.to_string()));
            }
            if let Some(sid) = rs.get("shortId").and_then(|s| s.as_str()) {
                query.push(("sid".into(), sid.to_string()));
            }
            if let Some(spx) = rs.get("spiderX").and_then(|s| s.as_str()) {
                query.push(("spx".into(), spx.to_string()));
            }
        }
    }

    let qs = query
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("vless://{userinfo}@{add}:{port}?{qs}{fragment}"))
}

// ── Shadowsocks (SIP002) ────────────────────────────────────────────────

fn parse_shadowsocks(url: &str) -> Result<Profile> {
    // SIP002: ss://base64(method:password)@host:port?plugin=...#tag
    let rest = url.strip_prefix("ss://").unwrap_or(url);

    // Primary: SIP002 standard format with @ separator
    if let Some((userinfo_b64, rest2)) = rest.split_once('@') {
        return parse_shadowsocks_sip002(userinfo_b64, rest2);
    }

    // Fallback: no @ separator — try treating entire body as a single base64 blob
    // that encodes method:password@host:port (some non-standard providers)
    let decoded = base64_simd::STANDARD
        .decode_to_vec(rest.trim_end_matches('=').as_bytes())
        .or_else(|_| {
            let trimmed = rest.trim_end_matches('=');
            let padded = match trimmed.len() % 4 {
                2 => format!("{trimmed}=="),
                3 => format!("{trimmed}="),
                _ => trimmed.to_string(),
            };
            base64_simd::STANDARD.decode_to_vec(padded.as_bytes())
        })
        .map_err(|_| ImportError::Parse("invalid base64 in ss://".into()))?;
    let inner = String::from_utf8_lossy(&decoded);

    if let Some((userinfo, hostport)) = inner.split_once('@') {
        let (method, password) = userinfo
            .split_once(':')
            .ok_or_else(|| ImportError::Parse("missing : in ss:// userinfo".into()))?;
        let hostport_clean = hostport.strip_suffix('?').unwrap_or(hostport);
        let (host, port_str) = hostport_clean
            .rsplit_once(':')
            .ok_or_else(|| ImportError::Parse("missing port in ss:// fallback".into()))?;
        let port: i32 = port_str
            .parse()
            .map_err(|_| ImportError::Parse("invalid port in ss:// fallback".into()))?;
        let protocol = if method.starts_with("2022-blake3-") {
            Protocol::Shadowsocks2022
        } else {
            Protocol::Shadowsocks
        };
        let mut profile = base_profile(protocol, host, port);
        profile.user_id = Some(password.to_string());
        let mut ps = serde_json::Map::new();
        ps.insert(
            "method".into(),
            serde_json::Value::String(method.to_string()),
        );
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
        return Ok(profile);
    }

    Err(ImportError::Parse("missing @ in ss:// URL".into()))
}

fn parse_shadowsocks_sip002(userinfo_b64: &str, rest2: &str) -> Result<Profile> {
    let (host_part_raw, fragment) = rest2
        .split_once('#')
        .map(|(h, f)| (h, Some(f.to_string())))
        .unwrap_or((rest2, None));
    let host_part = host_part_raw.strip_suffix('?').unwrap_or(host_part_raw);
    let (address, port_str) = host_part
        .rsplit_once(':')
        .ok_or_else(|| ImportError::Parse("missing port in ss:// URL".into()))?;
    let trimmed = userinfo_b64.trim_end_matches('=');
    let padded = match trimmed.len() % 4 {
        2 => format!("{trimmed}=="),
        3 => format!("{trimmed}="),
        _ => trimmed.to_string(),
    };
    let decoded = base64_simd::STANDARD
        .decode_to_vec(padded.as_bytes())
        .map_err(|_| ImportError::Parse("invalid base64 in ss://".into()))?;
    let userinfo = String::from_utf8_lossy(&decoded);
    let (method, password) = userinfo
        .split_once(':')
        .ok_or_else(|| ImportError::Parse("missing : in ss:// userinfo".into()))?;

    let port: i32 = port_str
        .parse()
        .map_err(|_| ImportError::Parse("invalid port".into()))?;
    let protocol = if method.starts_with("2022-blake3-") {
        Protocol::Shadowsocks2022
    } else {
        Protocol::Shadowsocks
    };
    let mut profile = base_profile(protocol, address, port);
    profile.remarks = fragment;
    profile.user_id = Some(password.to_string());
    let mut ps = serde_json::Map::new();
    ps.insert(
        "method".into(),
        serde_json::Value::String(method.to_string()),
    );
    profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    Ok(profile)
}

fn format_shadowsocks(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let method = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("method").and_then(|m| m.as_str().map(String::from)))
        .unwrap_or_else(|| "aes-256-gcm".into());
    let password = profile.user_id.as_deref().unwrap_or("");
    let userinfo = base64_simd::STANDARD.encode_to_string(format!("{method}:{password}"));
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("ss://{userinfo}@{add}:{port}{fragment}"))
}

// ── Trojan ──────────────────────────────────────────────────────────────

fn parse_trojan(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::Trojan,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
                ps.insert(
                    "allow_insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
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
        && let Some(sni) = v.get("sni").and_then(|s| s.as_str())
    {
        query.push(("sni".into(), sni.to_string()));
    }
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("trojan://{pw}@{add}:{port}{qs}{fragment}"))
}

// ── SOCKS ───────────────────────────────────────────────────────────────

fn parse_socks(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::Socks,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(1080)),
    );
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert(
            "username".into(),
            serde_json::Value::String(parsed.username.clone()),
        );
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
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("socks://@{add}:{port}{fragment}"))
}

// ── Hysteria2 ───────────────────────────────────────────────────────────

fn parse_hysteria2(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::Hysteria2,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
                ps.insert(
                    "insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(obfs) = v
            .get("obfs")
            .and_then(|o| o.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("obfs".into(), obfs.to_string()));
        }
        if let Some(sni) = v
            .get("sni")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("sni".into(), sni.to_string()));
        }
    }
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("hysteria2://{add}:{port}{qs}{fragment}"))
}

// ── Hysteria v1 ─────────────────────────────────────────────────────────

fn parse_hysteria(url: &str) -> Result<Profile> {
    let parsed = split_share_url(&url.replace("hy://", "hysteria://"))?;
    let mut profile = base_profile(
        Protocol::Hysteria,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
                ps.insert(
                    "up_mbps".into(),
                    serde_json::Value::Number(serde_json::Number::from(
                        v.parse::<i64>().unwrap_or(100),
                    )),
                );
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "downmbps" | "down_mbps" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert(
                    "down_mbps".into(),
                    serde_json::Value::Number(serde_json::Number::from(
                        v.parse::<i64>().unwrap_or(100),
                    )),
                );
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "sni" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert("sni".into(), serde_json::Value::String(v.clone()));
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "insecure" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert(
                    "insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(auth) = v
            .get("auth")
            .and_then(|a| a.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("auth".into(), auth.to_string()));
        }
        if let Some(up) = v.get("up_mbps").and_then(serde_json::Value::as_i64) {
            query.push(("upmbps".into(), up.to_string()));
        }
        if let Some(down) = v.get("down_mbps").and_then(serde_json::Value::as_i64) {
            query.push(("downmbps".into(), down.to_string()));
        }
    }
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("hysteria://{add}:{port}{qs}{fragment}"))
}

// ── TUIC ────────────────────────────────────────────────────────────────

fn parse_tuic(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::Tuic,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
                ps.insert(
                    "congestion_control".into(),
                    serde_json::Value::String(v.clone()),
                );
                profile.protocol_settings = Some(serde_json::to_string(&ps)?);
            }
            "udp_relay_mode" => {
                let mut ps = protocol_settings(profile.protocol_settings.as_deref());
                ps.insert(
                    "udp_relay_mode".into(),
                    serde_json::Value::String(v.clone()),
                );
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
                ps.insert(
                    "insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(pw) = v
            .get("password")
            .and_then(|p| p.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("password".into(), pw.to_string()));
        }
        if let Some(cc) = v
            .get("congestion_control")
            .and_then(|c| c.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("congestion_control".into(), cc.to_string()));
        }
        if let Some(urm) = v
            .get("udp_relay_mode")
            .and_then(|u| u.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("udp_relay_mode".into(), urm.to_string()));
        }
    }
    let qs = query
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("tuic://{add}:{port}?{qs}{fragment}"))
}

// ── Naïve ───────────────────────────────────────────────────────────────

fn parse_naive(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::Naive,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
    profile.remarks = parsed.fragment.clone();
    if !parsed.username.is_empty() {
        let mut ps = serde_json::Map::new();
        ps.insert(
            "user".into(),
            serde_json::Value::String(parsed.username.clone()),
        );
        if let Some(pw) = &parsed.password {
            ps.insert("password".into(), serde_json::Value::String(pw.clone()));
        }
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    }
    Ok(profile)
}

fn format_naive(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let user = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("user").and_then(|u| u.as_str().map(String::from)))
        .unwrap_or_default();
    let password = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("password").and_then(|p| p.as_str().map(String::from)))
        .unwrap_or_default();
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    let userinfo = if user.is_empty() {
        String::new()
    } else {
        format!("{user}:{password}@")
    };
    Ok(format!("naive+https://{userinfo}{add}:{port}{fragment}"))
}

// ── AnyTLS ──────────────────────────────────────────────────────────────

fn parse_anytls(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::AnyTls,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
                ps.insert(
                    "insecure".into(),
                    serde_json::Value::Bool(v == "1" || v.eq_ignore_ascii_case("true")),
                );
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(pw) = v
            .get("password")
            .and_then(|p| p.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("password".into(), pw.to_string()));
        }
        if let Some(sni) = v
            .get("sni")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("sni".into(), sni.to_string()));
        }
    }
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("anytls://{add}:{port}{qs}{fragment}"))
}

// ── ShadowTLS ───────────────────────────────────────────────────────────

fn parse_shadowtls(url: &str) -> Result<Profile> {
    let parsed = split_share_url(url)?;
    let mut profile = base_profile(
        Protocol::ShadowTls,
        &parsed.host,
        i32::from(parsed.port.unwrap_or(443)),
    );
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(pw) = v
            .get("password")
            .and_then(|p| p.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("password".into(), pw.to_string()));
        }
        if let Some(ver) = v
            .get("version")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("version".into(), ver.to_string()));
        }
    }
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        )
    };
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
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
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
    {
        if let Some(pk) = v
            .get("private_key")
            .and_then(|p| p.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("privateKey".into(), pk.to_string()));
        }
        if let Some(pubk) = v
            .get("public_key")
            .and_then(|p| p.as_str())
            .filter(|s| !s.is_empty())
        {
            query.push(("publicKey".into(), pubk.to_string()));
        }
    }
    if let Some(addr) = &profile.address
        && let Some(port) = profile.port
    {
        query.push(("endpoint".into(), format!("{addr}:{port}")));
    }
    let qs = query
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let remark = profile.remarks.as_deref().unwrap_or("");
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
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
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| ImportError::Parse("missing scheme in URL".into()))?;
    let scheme = scheme.to_string();
    let mut unparsed = rest;

    // 2. Extract userinfo (find the correct @ before any # or ?)
    let (username, password, rest) = find_userinfo(unparsed, &scheme);
    unparsed = rest;

    // 3. Extract fragment
    let fragment = unparsed.split_once('#').map(|(r, f)| {
        unparsed = r;
        percent_decode(f)
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
        unparsed.split_once('@').map_or(unparsed, |(_, r)| r)
    } else {
        unparsed
    };

    let (mut host, port) = parse_hostport(hostport);
    host = fix_percent_encoding(&host);

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
                    let host_end = after_at
                        .find('/')
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

/// Parse `host:port` string with recovery for:
/// - Port-suffix garbage: `host:443:extra` → `(host, Some(443))`
/// - Trailing garbage: `host:443abc` → `(host, Some(443))`
fn parse_hostport(s: &str) -> (String, Option<u16>) {
    // Handle IPv6: [::1]:port
    if let Some(inner) = s.strip_prefix('[') {
        if let Some((host, port_part)) = inner.split_once("]:") {
            let port = port_part.parse::<u16>().ok();
            return (format!("[{host}]"), port);
        }
        let trimmed = s.trim_end_matches(']');
        return (format!("[{}]", &trimmed[1..]), None);
    }

    // Iterative scan from right: find first colon where the suffix starts
    // with valid ASCII digits forming a u16 port number.
    // Handles port-suffix like `host:443:extra` and trailing-garbage like `host:443abc`.
    let mut remaining = s;
    let mut selected_port = None;
    while let Some((h, p)) = remaining.rsplit_once(':') {
        let digit_len = p
            .bytes()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(p.len());
        if digit_len > 0
            && digit_len <= 5
            && let Ok(parsed) = p[..digit_len].parse::<u16>()
        {
            selected_port = Some(parsed);
            remaining = h;
            break;
        }
        // Not a valid port suffix — scan further left
        remaining = h;
    }
    (remaining.to_string(), selected_port)
}

/// Fix bare `%` characters not followed by 2 valid hex digits by percent-encoding them.
fn fix_percent_encoding(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 < bytes.len() && is_hex_char(bytes[i + 1]) && is_hex_char(bytes[i + 2]) {
                out.push(b'%');
                i += 1;
            } else {
                // Bare % — encode it
                out.extend_from_slice(b"%25");
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

const fn is_hex_char(b: u8) -> bool {
    b.is_ascii_hexdigit()
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

/// Normalize a remark string: percent-decode, trim, collapse inner whitespace.
#[must_use]
pub fn normalize_remark(s: &str) -> String {
    let decoded = percent_decode(s);
    let mut out = String::with_capacity(decoded.len());
    let mut prev_was_space = false;
    for c in decoded.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(c);
            prev_was_space = false;
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

#[inline]
const fn hex_val_sub(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Parse query string into `Vec<(String, String)>` — linear scan, no `HashMap`.
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
        address: if address.is_empty() {
            None
        } else {
            Some(address.to_string())
        },
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
    let decoded = crate::base64_util::decode_base64(b64).or_else(|_| {
        // Some providers double-encode — try URL-decoding first
        let url_decoded = urlencoding::decode(b64)
            .map_err(|_| ImportError::Parse("invalid base64 in ssr URL".into()))?;
        crate::base64_util::decode_base64(&url_decoded)
            .map_err(|_| ImportError::Parse("invalid base64 in ssr URL (after URL-decode)".into()))
    })?;
    let text = String::from_utf8(decoded)
        .map_err(|_| ImportError::Parse("invalid UTF-8 in ssr URL".into()))?;

    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() < 6 {
        return Err(ImportError::Parse(
            "ssr: expected at least 6 colon-delimited fields".into(),
        ));
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

    let port: i32 = raw_port
        .parse()
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
    ps.insert(
        "method".into(),
        serde_json::Value::String(raw_method.to_string()),
    );
    ps.insert(
        "protocol".into(),
        serde_json::Value::String(raw_protocol.to_string()),
    );
    ps.insert(
        "obfs".into(),
        serde_json::Value::String(raw_obfs.to_string()),
    );

    // Decode base64 query params
    for (key, src_field) in [
        ("obfsparam", "obfsparam"),
        ("protoparam", "protoparam"),
        ("group", "group"),
    ] {
        if let Some(val_b64) = params.get(src_field)
            && let Ok(bytes) = crate::base64_util::decode_base64(val_b64)
            && let Ok(val_decoded) = String::from_utf8(bytes)
        {
            ps.insert((*key).into(), serde_json::Value::String(val_decoded));
        }
    }
    profile.protocol_settings = Some(serde_json::to_string(&ps)?);

    if let Some(remarks_b64) = params.get("remarks")
        && let Ok(bytes) = crate::base64_util::decode_base64(remarks_b64)
        && let Ok(decoded) = String::from_utf8(bytes)
    {
        profile.remarks = Some(decoded);
    }

    Ok(profile)
}

fn format_shadowsocksr(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let password = profile.user_id.as_deref().unwrap_or("");

    let (method, protocol, obfs) = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .map_or_else(
            || ("rc4-md5".into(), "origin".into(), "plain".into()),
            |v| {
                let m = v
                    .get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or("rc4-md5");
                let p = v
                    .get("protocol")
                    .and_then(|p| p.as_str())
                    .unwrap_or("origin");
                let o = v.get("obfs").and_then(|o| o.as_str()).unwrap_or("plain");
                (m.to_string(), p.to_string(), o.to_string())
            },
        );

    let mut query_str = String::new();
    if let Some(ps) = &profile.protocol_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ps)
        && let Some(obfsparam) = v
            .get("obfsparam")
            .and_then(|o| o.as_str())
            .filter(|s| !s.is_empty())
    {
        let encoded = base64_simd::URL_SAFE_NO_PAD.encode_to_string(obfsparam);
        query_str.push_str(&format!("obfsparam={encoded}&"));
    }
    if let Some(remarks) = &profile.remarks {
        let encoded = base64_simd::URL_SAFE_NO_PAD.encode_to_string(remarks);
        query_str.push_str(&format!("remarks={encoded}"));
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
fn parse_http(url: &str) -> Result<Profile> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| ImportError::Parse("expected http:// scheme".into()))?;
    // Split @ for userinfo
    let (userinfo, hostpart) = rest.split_once('@').unwrap_or(("", rest));
    // Split # for fragment (remark)
    let (hostport, fragment) = hostpart
        .split_once('#')
        .map(|(h, f)| (h, Some(f.to_string())))
        .unwrap_or((hostpart, None));

    let (host, port_str) = hostport
        .rsplit_once(':')
        .ok_or_else(|| ImportError::Parse("http: missing port".into()))?;
    let port: i32 = port_str
        .parse()
        .map_err(|_| ImportError::Parse("http: invalid port".into()))?;

    let mut profile = base_profile(Protocol::Http, host, port);
    profile.remarks = fragment.filter(|s| !s.is_empty());

    if !userinfo.is_empty() {
        let mut ps = serde_json::Map::new();
        if let Some((username, password)) = userinfo.split_once(':') {
            ps.insert(
                "username".into(),
                serde_json::Value::String(username.to_string()),
            );
            ps.insert(
                "password".into(),
                serde_json::Value::String(password.to_string()),
            );
        } else {
            ps.insert(
                "username".into(),
                serde_json::Value::String(userinfo.to_string()),
            );
        }
        profile.protocol_settings = Some(serde_json::to_string(&ps)?);
    }

    Ok(profile)
}

fn format_http(profile: &Profile) -> Result<String> {
    let (add, port) = addr_port(profile);
    let (user, pass) = profile
        .protocol_settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .map(|v| {
            let u = v
                .get("username")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            let p = v
                .get("password")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string();
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
    let fragment = if remark.is_empty() {
        String::new()
    } else {
        format!("#{remark}")
    };
    Ok(format!("http://{userinfo}{add}:{port}{fragment}"))
}
// ── Validation Layer ─────────────────────────────────────────────────────

/// Settings controlling how strictly parsed profiles are validated.
#[derive(Debug, Clone, Default)]
pub struct ValidationSettings {
    /// If true, allow private/loopback IPs. Default: false.
    pub allow_private_ips: bool,
    /// If true, reject profiles with allowInsecure=true. Default: false.
    pub reject_insecure: bool,
}

impl From<crate::app_config::ParsingSettings> for ValidationSettings {
    fn from(p: crate::app_config::ParsingSettings) -> Self {
        Self {
            allow_private_ips: p.allow_private_ips,
            reject_insecure: p.reject_insecure,
        }
    }
}

/// Per-protocol required-field validation.
fn validate_required_fields(profile: &Profile) -> Result<()> {
    let protocol = Protocol::try_from_i32(profile.config_type)
        .ok_or_else(|| ImportError::Validation("unknown protocol".into()))?;

    let missing = |field: &str| ImportError::Validation(format!("missing field: {field}"));

    match protocol {
        Protocol::Vmess | Protocol::Vless => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::Trojan => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::Shadowsocks | Protocol::Shadowsocks2022 => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::ShadowsocksR => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::Hysteria2 => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
            // Port defaults to 443 for Hysteria2
        }
        Protocol::Hysteria => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
        }
        Protocol::Tuic => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::Socks | Protocol::Http => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
        }
        Protocol::Naive | Protocol::AnyTls | Protocol::ShadowTls => {
            if profile.address.is_none() || profile.address.as_deref() == Some("") {
                return Err(missing("address"));
            }
            if profile.port.is_none() || profile.port == Some(0) {
                return Err(missing("port"));
            }
            if profile.user_id.is_none() || profile.user_id.as_deref() == Some("") {
                return Err(missing("user_id"));
            }
        }
        Protocol::WireGuard => {
            // WireGuard may use query params for address; check protocol_settings for keys
            // at minimum, need public_key in protocol_settings
            if let Some(ps) = &profile.protocol_settings {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(ps) {
                    let has_pubkey = v
                        .get("public_key")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| !s.is_empty());
                    if !has_pubkey {
                        return Err(missing("public_key in protocol_settings"));
                    }
                } else {
                    return Err(missing("protocol_settings (invalid JSON)"));
                }
            } else {
                return Err(missing("protocol_settings"));
            }
        }
        // Tor, Ssh, Tailscale, Redirect, TProxy, Mixed, etc. — minimal validation
        _ => {}
    }

    Ok(())
}

/// Validate server address is not private/loopback/link-local.
fn validate_host(profile: &Profile, settings: &ValidationSettings) -> Result<()> {
    if settings.allow_private_ips {
        return Ok(());
    }

    let addr = match &profile.address {
        Some(a) if !a.is_empty() => a,
        _ => return Ok(()), // no address to validate
    };

    // Try parsing as IP address
    if let Ok(ip) = addr.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(v4) => {
                if v4.is_loopback() {
                    return Err(ImportError::Validation("loopback IP address".into()));
                }
                if v4.is_private() {
                    return Err(ImportError::Validation("private IP address".into()));
                }
                if v4.is_link_local() {
                    return Err(ImportError::Validation("link-local IP address".into()));
                }
            }
            IpAddr::V6(v6) => {
                if v6.is_loopback() {
                    return Err(ImportError::Validation("loopback IP address".into()));
                }
                if v6.is_unique_local() {
                    return Err(ImportError::Validation("unique-local IP address".into()));
                }
                if v6.is_unicast_link_local() {
                    return Err(ImportError::Validation("link-local IP address".into()));
                }
            }
        }
    } else {
        // DNS name — check for localhost
        let lower = addr.to_lowercase();
        if lower == "localhost" || lower.ends_with(".localhost") {
            return Err(ImportError::Validation("localhost hostname".into()));
        }
    }

    Ok(())
}

/// Security validation: check for insecure settings and log warnings.
fn validate_security(profile: &Profile) -> Result<()> {
    if let Some(ss) = &profile.stream_settings
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(ss)
        && (v.get("allow_insecure").and_then(serde_json::Value::as_bool) == Some(true)
            || v.get("insecure").and_then(|s| s.as_str()) == Some("1"))
    {
        tracing::warn!(target: "validation",
            "Profile {} has allow_insecure=true",
            profile.remarks.as_deref().unwrap_or("(unnamed)")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Test settings with private IPs allowed (existing tests use various IPs).
    fn permissive_settings() -> ValidationSettings {
        ValidationSettings {
            allow_private_ips: true,
            reject_insecure: false,
        }
    }

    #[test]
    fn normalize_remark_basic() {
        assert_eq!(normalize_remark("hello"), "hello");
    }

    #[test]
    fn normalize_remark_percent_decoded() {
        // Japanese "test" in percent-encoded UTF-8
        let result = normalize_remark("%E6%B5%8B%E8%AF%95");
        assert_eq!(result, "测试");
    }

    #[test]
    fn normalize_remark_whitespace_collapsed() {
        assert_eq!(normalize_remark("  hello   world  "), "hello world");
        assert_eq!(normalize_remark("\tfoo \n bar\r\n baz"), "foo bar baz");
    }

    #[test]
    fn normalize_remark_emoji_percent_decoded() {
        // Grinning face emoji
        let result = normalize_remark("%F0%9F%98%80");
        assert_eq!(result, "😀");
    }

    #[test]
    fn normalize_remark_empty_after_trim() {
        assert_eq!(normalize_remark("  "), "");
        assert_eq!(normalize_remark("%20%20"), "");
    }

    #[test]
    fn normalize_remark_no_change_for_plain_text() {
        assert_eq!(normalize_remark("  My Server 1  "), "My Server 1");
    }

    #[test]
    fn roundtrip_vmess() {
        let mut p = base_profile(Protocol::Vmess, "example.com", 443);
        p.user_id = Some("uuid-here".into());
        p.network = Some("ws".into());
        p.stream_settings = Some(r#"{"ws.path":"/api","tls.enable":true}"#.into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("vmess://"));
        let parsed = parse_share_url(&url, &permissive_settings()).unwrap();
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
        let parsed = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
    }

    #[test]
    fn roundtrip_shadowsocks() {
        let mut p = base_profile(Protocol::Shadowsocks, "ss.example", 1080);
        p.user_id = Some("password123".into());
        let mut ps = serde_json::Map::new();
        ps.insert(
            "method".into(),
            serde_json::Value::String("aes-256-gcm".into()),
        );
        p.protocol_settings = Some(serde_json::to_string(&ps).unwrap());
        p.remarks = Some("myss".into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("ss://"));
        let parsed = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(parsed.config_type, p.config_type);
    }

    #[test]
    fn roundtrip_trojan() {
        let mut p = base_profile(Protocol::Trojan, "trojan.example", 443);
        p.user_id = Some("password".into());
        p.remarks = Some("troj".into());
        let url = format_share_url(&p).unwrap();
        assert!(url.starts_with("trojan://"));
        let parsed = parse_share_url(&url, &permissive_settings()).unwrap();
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
        let p = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p.address.as_deref(), Some("1.2.3.4"));
        assert_eq!(p.port, Some(443));
    }

    #[test]
    fn unsupported_scheme() {
        // URL without :// cannot be parsed by any parser — returns an error
        assert!(parse_share_url("not-a-url", &permissive_settings()).is_err());
    }

    const WORKING_URL_1: &str = "vless://a5ea9247-79f3-4655-aece-3fb51e1e669e@146.103.99.45:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=rezerv.yunus.guru&fp=firefox&pbk=S4WFc-SD_FpmmQdM21Of7O6XmYaLlmwcmlbgO4lZQQg&sid=a7ec6c3316eddb11&type=tcp&headerType=none#%5B332ms%20%D0%A4%D0%B8%D0%BD%D0%BB%D1%8F%D0%BD%D0%B4%D0%B8%D1%8F%20FI%20%F0%9F%87%AB%F0%9F%87%AE%20%40vlesstrojan%5D";
    const WORKING_URL_2: &str = "vless://a5ea9247-79f3-4655-aece-3fb51e1e669e@144.124.241.233:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=rezerv1.yunus.guru&fp=firefox&pbk=-X9CZv5MYKivpxPVP1vdgFKf2AJWmZ0Pju-j8LFmlh4&sid=6c88854e73e86773&type=tcp&headerType=none#%5B333ms%20%D0%A4%D0%B8%D0%BD%D0%BB%D1%8F%D0%BD%D0%B4%D0%B8%D1%8F%20FI%20%F0%9F%87%AB%F0%9F%87%AE%20%40vlesstrojan%5D";
    const WORKING_URL_3: &str = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTprMWRCT21PQjRvcWk3VW1wMzdhMWJR@82.38.31.192:8080?#%5B164ms%20%D0%90%D0%B2%D1%81%D1%82%D1%80%D0%B8%D1%8F%20AT%20%F0%9F%87%A6%F0%9F%87%B9%20%40vlesstrojan%5D";
    const WORKING_URL_4: &str = "ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpDSm1UQ0N4N0x0dWQ@108.181.126.122:8388?#%5B140ms%20%D0%90%D0%B2%D1%81%D1%82%D1%80%D0%B8%D1%8F%20AT%20%F0%9F%87%A6%F0%9F%87%B9%20%40vlesstrojan%5D";

    #[test]
    fn parse_working_txt_urls() {
        // ── VLESS Reality URL 1 ──
        let p = parse_share_url(WORKING_URL_1, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::Vless.to_i32());
        assert_eq!(p.address.as_deref(), Some("146.103.99.45"));
        assert_eq!(p.port, Some(443));
        assert_eq!(
            p.user_id.as_deref(),
            Some("a5ea9247-79f3-4655-aece-3fb51e1e669e")
        );
        assert_eq!(p.network.as_deref(), Some("tcp"));
        assert!(p.remarks.as_ref().unwrap().contains("Финляндия"));

        // Reality params now extracted (parser fix applied)
        if let Some(ss) = &p.stream_settings {
            let v: serde_json::Value =
                serde_json::from_str(ss).expect("stream_settings must be valid JSON");
            let obj = v.as_object().expect("stream_settings must be an object");
            assert_eq!(obj["sni"], "rezerv.yunus.guru");
            assert_eq!(obj["fingerprint"], "firefox");
            assert_eq!(obj["security"], "reality");
            let rs = obj
                .get("realitySettings")
                .expect("realitySettings should be present")
                .as_object()
                .expect("realitySettings must be an object");
            assert_eq!(
                rs["publicKey"],
                "S4WFc-SD_FpmmQdM21Of7O6XmYaLlmwcmlbgO4lZQQg"
            );
            assert_eq!(rs["shortId"], "a7ec6c3316eddb11");
        } else {
            panic!("VLESS URL should have stream_settings");
        }

        // ── VLESS Reality URL 2 ──
        let p = parse_share_url(WORKING_URL_2, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::Vless.to_i32());
        assert_eq!(p.address.as_deref(), Some("144.124.241.233"));
        assert_eq!(p.port, Some(443));
        assert_eq!(
            p.user_id.as_deref(),
            Some("a5ea9247-79f3-4655-aece-3fb51e1e669e")
        );
        assert_eq!(p.network.as_deref(), Some("tcp"));

        // Reality params now extracted (parser fix applied)
        if let Some(ss) = &p.stream_settings {
            let v: serde_json::Value =
                serde_json::from_str(ss).expect("stream_settings must be valid JSON");
            let obj = v.as_object().expect("stream_settings must be an object");
            assert_eq!(obj["sni"], "rezerv1.yunus.guru");
            assert_eq!(obj["fingerprint"], "firefox");
            assert_eq!(obj["security"], "reality");
            let rs = obj
                .get("realitySettings")
                .expect("realitySettings should be present")
                .as_object()
                .expect("realitySettings must be an object");
            assert_eq!(
                rs["publicKey"],
                "-X9CZv5MYKivpxPVP1vdgFKf2AJWmZ0Pju-j8LFmlh4"
            );
            assert_eq!(rs["shortId"], "6c88854e73e86773");
        } else {
            panic!("VLESS URL should have stream_settings");
        }

        // ── Shadowsocks URL 3 ──
        let p = parse_share_url(WORKING_URL_3, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::Shadowsocks.to_i32());
        assert_eq!(p.address.as_deref(), Some("82.38.31.192"));
        assert_eq!(p.port, Some(8080));
        assert!(p.remarks.as_ref().unwrap().contains("Австрия"));
        if let Some(ps) = &p.protocol_settings {
            let v: serde_json::Value =
                serde_json::from_str(ps).expect("protocol_settings must be valid JSON");
            assert_eq!(v["method"], "chacha20-ietf-poly1305");
        } else {
            panic!("Shadowsocks URL should have protocol_settings");
        }
        assert_eq!(p.user_id.as_deref(), Some("k1dBOmOB4oqi7Ump37a1bQ"));

        // ── Shadowsocks URL 4 ──
        let p = parse_share_url(WORKING_URL_4, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::Shadowsocks.to_i32());
        assert_eq!(p.address.as_deref(), Some("108.181.126.122"));
        assert_eq!(p.port, Some(8388));
        if let Some(ps) = &p.protocol_settings {
            let v: serde_json::Value =
                serde_json::from_str(ps).expect("protocol_settings must be valid JSON");
            assert_eq!(v["method"], "chacha20-ietf-poly1305");
        } else {
            panic!("Shadowsocks URL should have protocol_settings");
        }
        assert_eq!(p.user_id.as_deref(), Some("CJmTCCx7Ltud"));
    }

    #[test]
    fn roundtrip_vless_reality() {
        // Parse working VLESS Reality URL, format back, re-parse
        let p1 = parse_share_url(WORKING_URL_1, &permissive_settings()).unwrap();
        let url = format_share_url(&p1).unwrap();
        assert!(url.starts_with("vless://"));
        // Re-parse should preserve all Reality fields
        let p2 = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p2.config_type, Protocol::Vless.to_i32());
        assert_eq!(p2.address, p1.address);
        assert_eq!(p2.port, p1.port);
        assert_eq!(p2.user_id, p1.user_id);
        assert_eq!(p2.network, p1.network);
        // Compare stream_settings (Reality params)
        if let (Some(ss1), Some(ss2)) = (&p1.stream_settings, &p2.stream_settings) {
            let v1: serde_json::Value =
                serde_json::from_str(ss1).expect("stream_settings must be valid JSON");
            let v2: serde_json::Value =
                serde_json::from_str(ss2).expect("stream_settings must be valid JSON");
            assert_eq!(v1["security"], v2["security"]);
            assert_eq!(v1["realitySettings"], v2["realitySettings"]);
            assert_eq!(v1["sni"], v2["sni"]);
            assert_eq!(v1["fingerprint"], v2["fingerprint"]);
        } else {
            panic!("Both profiles should have stream_settings");
        }
    }

    #[test]
    fn roundtrip_shadowsocks_real() {
        let p1 = parse_share_url(WORKING_URL_3, &permissive_settings()).unwrap();
        let url = format_share_url(&p1).unwrap();
        assert!(url.starts_with("ss://"));
        let p2 = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p2.config_type, Protocol::Shadowsocks.to_i32());
        assert_eq!(p2.address, p1.address);
        assert_eq!(p2.port, p1.port);
        assert_eq!(p2.user_id, p1.user_id);
        if let (Some(ps1), Some(ps2)) = (&p1.protocol_settings, &p2.protocol_settings) {
            let v1: serde_json::Value =
                serde_json::from_str(ps1).expect("protocol_settings must be valid JSON");
            let v2: serde_json::Value =
                serde_json::from_str(ps2).expect("protocol_settings must be valid JSON");
            assert_eq!(v1["method"], v2["method"]);
        }
    }

    // ── Validation rejection tests ──

    #[test]
    fn reject_vmess_no_address() {
        // VMess URL with empty address
        let qr = serde_json::json!({ "v": 2, "ps": "test", "add": "", "port": 443, "id": "uuid" });
        let b64 = base64_simd::STANDARD.encode_to_string(serde_json::to_string(&qr).unwrap());
        let url = format!("vmess://{b64}");
        let settings = ValidationSettings {
            allow_private_ips: false,
            reject_insecure: false,
        };
        assert!(matches!(
            parse_share_url(&url, &settings),
            Err(ImportError::Validation(_))
        ));
    }

    #[test]
    fn reject_private_ip() {
        // Any protocol with 127.0.0.1 should be rejected when allow_private_ips=false
        let url = "vless://uuid@127.0.0.1:443?encryption=none#test";
        let settings = ValidationSettings {
            allow_private_ips: false,
            reject_insecure: false,
        };
        assert!(matches!(
            parse_share_url(url, &settings),
            Err(ImportError::Validation(_))
        ));
    }

    #[test]
    fn accept_private_ip_when_allowed() {
        // Same URL with allow_private_ips=true should succeed
        let url = "vless://uuid@127.0.0.1:443?encryption=none#test";
        let settings = ValidationSettings {
            allow_private_ips: true,
            reject_insecure: false,
        };
        let p = parse_share_url(url, &settings).unwrap();
        assert_eq!(p.address.as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn reject_localhost_hostname() {
        let url = "vless://uuid@localhost:443?encryption=none#test";
        let settings = ValidationSettings {
            allow_private_ips: false,
            reject_insecure: false,
        };
        assert!(matches!(
            parse_share_url(url, &settings),
            Err(ImportError::Validation(_))
        ));
    }

    // ── Permissive parsing tests ──

    #[test]
    fn unknown_scheme_rejected_when_all_parsers_fail_validation() {
        // Unknown scheme with private IP should be rejected by host validation.
        // Note: some permissive parsers (e.g. http) produce profiles with degraded
        // addresses, so the exact error type may vary.
        let url = "xyzzy://192.168.1.1:443";
        let settings = ValidationSettings {
            allow_private_ips: false,
            reject_insecure: false,
        };
        assert!(parse_share_url(url, &settings).is_err());
    }

    #[test]
    fn vmess_trailing_garbage() {
        // VMess base64 with extra text after the JSON object
        let qr = serde_json::json!({ "v": 2, "ps": "clean", "add": "5.6.7.8", "port": 8443, "id": "uuid2" });
        let b64 = base64_simd::STANDARD.encode_to_string(serde_json::to_string(&qr).unwrap());
        let url = format!("vmess://{}extra-garbage-here", b64);
        let p = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p.address.as_deref(), Some("5.6.7.8"));
        assert_eq!(p.port, Some(8443));
    }

    #[test]
    fn ssr_url_encoded() {
        // SSR URL with URL-encoded base64 (double encoding)
        let inner = "1.2.3.4:1234:origin:aes-256-cfb:plain:dGVzdA";
        let b64 = base64_simd::STANDARD.encode_to_string(inner);
        let url_encoded = urlencoding::encode(&b64);
        let url = format!("ssr://{url_encoded}");
        let p = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::ShadowsocksR.to_i32());
        assert_eq!(p.address.as_deref(), Some("1.2.3.4"));
        assert_eq!(p.port, Some(1234));
    }

    #[test]
    fn ss_fallback_no_at_sign() {
        // SS URL without @ separator — entire body as single base64 blob
        let inner = "aes-256-gcm:password123@9.9.9.9:4444";
        let b64 = base64_simd::STANDARD.encode_to_string(inner);
        let url = format!("ss://{b64}");
        let p = parse_share_url(&url, &permissive_settings()).unwrap();
        assert_eq!(p.config_type, Protocol::Shadowsocks.to_i32());
        assert_eq!(p.address.as_deref(), Some("9.9.9.9"));
        assert_eq!(p.port, Some(4444));
    }

    #[test]
    fn vless_trailing_garbage_missing_query_separator() {
        // VLESS URL with missing ? before query params
        let url = "vless://uuid@6.6.6.6:443security=tls&sni=example.com#test";
        let p = parse_share_url(url, &permissive_settings()).unwrap();
        assert_eq!(p.address.as_deref(), Some("6.6.6.6"));
        assert_eq!(p.port, Some(443));
        // The garbage after port is lost but URL still parses
    }
}
