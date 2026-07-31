#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "TUI display domain: ports validated to u16, timestamps fit i64 for billions of years, list indices < u16 for rendering, display precision loss acceptable"
)]
#![allow(
    clippy::future_not_send,
    clippy::manual_let_else,
    reason = "single-threaded TUI, futures never sent across threads; manual let-else where match is clearer"
)]

pub mod ops;
mod state;
mod types;
pub mod ui;

// Re-exports
pub use state::AppState;
pub use types::*;

// ── Helper functions ───────────────────────────────────────────────────

use tokio::sync::mpsc;
use tracing::warn;

use xray_tui_config::import_export::{Profile, profile_config};
use xray_tui_proto::proto_spec::ProtoSpec;

/// Helper to send a `CoreEvent` with a warning on channel full.
/// Prevents silent event loss.
fn try_send_or_warn(tx: &mpsc::Sender<CoreEvent>, event: CoreEvent, label: &'static str) {
    if let Err(_e) = tx.try_send(event) {
        warn!(target: "tui::events", "try_send dropped {label}: channel full");
    }
}

fn common_field_defaults() -> Vec<(String, String)> {
    vec![
        ("remarks".to_string(), String::new()),
        ("address".to_string(), String::new()),
        ("port".to_string(), "443".to_string()),
        ("core_type".to_string(), "auto".to_string()),
    ]
}

pub(crate) fn profile_to_fields(profile: &Profile) -> Vec<(String, String)> {
    let mut fields = common_field_defaults();
    if let Some(config) = profile_config(profile) {
        if let Some(v) = config.remarks() {
            set_field(&mut fields, "remarks", v);
        }
        if let Some(v) = xray_tui_config::import_export::profile_user_id(&config) {
            set_field(&mut fields, "user_id", &v);
        }
        if let Some(v) = config.security_type() {
            set_field(&mut fields, "security", v);
        }
        if let Some(v) = config.transport_type() {
            set_field(&mut fields, "network", v);
        }
        let (_ps, ss) = config.to_settings();
        xray_tui_config::import_export::flatten_json_to_fields(&ss, &mut fields);
    }
    // Cached fields not in spec_blob

    if !profile.address.is_empty() {
        set_field(&mut fields, "address", &profile.address);
    }
    if profile.port > 0 {
        set_field(&mut fields, "port", &profile.port.to_string());
    }
    set_field(&mut fields, "core_type", &profile.core_type);
    fields
}

fn set_field(fields: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = fields.iter_mut().find(|(k, _)| k == key) {
        *existing = value.to_string();
    } else {
        fields.push((key.to_string(), value.to_string()));
    }
}

/// Get a value from a form field list, returning None if empty or missing.
#[must_use]
pub fn get_field(fields: &[(String, String)], key: &str) -> Option<String> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
}

pub(crate) fn format_now() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert Unix timestamp to civil date using Howard Hinnant's
    // chrono-compatible algorithm (public domain).
    let (year, month, day) = civil_from_days(secs / 86400);
    let remaining = secs % 86400;
    let hour = remaining / 3600;
    let minute = (remaining % 3600) / 60;
    let second = remaining % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(z: u64) -> (u64, u64, u64) {
    let z = z as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u64, m as u64, d)
}

pub fn parse_core_log_line(
    line: &str,
    _core_type: xray_tui_core::CoreType,
) -> (String, String, String, Option<i64>) {
    // Handle xray-core format: "2024/01/01 12:00:00 [Info] message"
    // Handle sing-box format: "{\"level\":\"info\",\"msg\":\"...\"}" or "INFO: message"
    // Handle plain format: "[Info] message"
    let line = line.trim();

    // Try JSON format (sing-box)
    if line.starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let level = val
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_lowercase();
            let message = val
                .get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or(line)
                .to_string();
            let target = val
                .get("logger")
                .and_then(|v| v.as_str())
                .unwrap_or("core")
                .to_string();
            return (target, level, message, None);
        }
    }

    let parts: Vec<&str> = line.splitn(5, ' ').collect();
    if parts.len() < 2 {
        return ("core".to_string(), "info".to_string(), line.to_string(), None);
    }

    // Check if parts[0] looks like a date (YYYY/MM/DD or YYYY-MM-DD)
    // Xray-core: "2024/01/01 12:00:00 [Info] message"  → level at parts[2]
    // Plain: "[Info] message" → level at parts[0]
    let (level_str, msg_start) = if parts[0].len() >= 10
        && parts[0].as_bytes().iter().take(4).all(|b| b.is_ascii_digit())
        && (parts[0].contains('/') || parts[0].contains('-'))
    {
        // Has timestamp prefix: parts[0]=date, parts[1]=time, parts[2]=[Level]
        if parts.len() > 3 {
            (parts.get(2).copied().unwrap_or(""), 3)
        } else {
            // Timestamp but no level (e.g. "2026/07/31 18:45:48.065654 from tcp:...")
            ("info", 0)
        }
    } else if parts[0].contains('[')
        || (parts[0].len() > 2 && parts[0].chars().all(|c| c.is_uppercase() || c == ':'))
    {
        // Plain "[Level] ..." or "LEVEL: ..."
        (parts[0], 1)
    } else {
        // Banner line or bare text with no recognizable level indicator
        ("info", 0)
    };

    let mut level = level_str
        .trim_matches(|c| c == '[' || c == ']' || c == ':' || c == ' ')
        .to_lowercase();
    if level.is_empty() {
        level = "info".to_string();
    } else if level == "failed" {
        level = "error".to_string();
    } else if level == "warning" {
        level = "warn".to_string();
    }

    let message = if parts.len() > msg_start {
        parts[msg_start..].join(" ").trim().to_string()
    } else {
        line.to_string()
    };

    ("core".to_string(), level.to_string(), message, None)
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_now() {
        let s = format_now();
        assert!(s.len() == 20, "expected ISO 8601 length 20: {s}");
        assert!(s.ends_with('Z'), "expected UTC: {s}");
    }

    #[test]
    fn test_common_field_defaults() {
        let fields = common_field_defaults();
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|(k, _)| k == "address"));
        assert!(fields.iter().any(|(k, _)| k == "port"));
        assert!(fields.iter().any(|(k, _)| k == "remarks"));
        assert!(fields.iter().any(|(k, _)| k == "core_type"));
    }

    #[test]
    fn test_set_field() {
        let mut fields = common_field_defaults();
        set_field(&mut fields, "port", "8443");
        assert_eq!(
            fields
                .iter()
                .find(|(k, _)| k == "port")
                .map(|(_, v)| v.as_str()),
            Some("8443")
        );
        set_field(&mut fields, "new_key", "value");
        assert!(fields.iter().any(|(k, _)| k == "new_key"));
    }

    #[test]
    fn test_get_field() {
        let mut fields = common_field_defaults();
        set_field(&mut fields, "remarks", "my server");
        assert_eq!(get_field(&fields, "remarks"), Some("my server".to_string()));
        // Empty value should return None
        assert_eq!(get_field(&fields, "nonexistent"), None);
    }
}
