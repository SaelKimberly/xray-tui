use xray_tui_core::grpc_client::format_bytes;
use xray_tui_core::protocol::Protocol;

use xray_tui_config::import_export::ProfileLegacy;
use crate::AppState;
use crate::ui::theme::ThemeStyles;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_cheese::fieldset::{Fieldset, FieldsetStyles};

#[allow(clippy::missing_const_for_fn)]
fn connection_icon(state: &AppState) -> (&'static str, Style) {
    let palette = state.current_palette();
    if state.connecting {
        ("⠋", ThemeStyles::spinner(&palette))
    } else if state.connected_core.is_some() {
        ("●", ThemeStyles::success(&palette))
    } else if state.connection_error.is_some() {
        ("⏹", ThemeStyles::error(&palette))
    } else if state.connected_profile_id.is_some() {
        ("⏏", ThemeStyles::warning(&palette))
    } else {
        ("○", ThemeStyles::hint(&palette))
    }
}

fn server_summary(state: &AppState) -> (String, String, String, u16, String) {
    // Try the connected profile first
    let from_connected = state
        .connected_profile_id
        .as_ref()
        .and_then(|id| state.profiles.iter().find(|r| r.profile.id == *id));

    // Fall back to selected profile
    let row = from_connected.or_else(|| {
        if state.filtered_len() == 0 {
            return None;
        }
        let idx = state.selected_index.min(state.filtered_len() - 1);
        state.filtered_profiles().nth(idx)
    });

    row.map_or_else(
        || {
            (
                "-".to_string(),
                "No server".to_string(),
                String::new(),
                0u16,
                String::new(),
            )
        },
        |r| {
            let proto = Protocol::try_from_i32(r.profile.config_type).unwrap_or(Protocol::Custom);
            let remarks = r.profile.leg("remarks").unwrap_or_default();
            let addr = r.profile.address.clone();
            let port = r.profile.port as u16;
            let core = state.resolved_core(r).to_string();
            (proto.to_string(), remarks, addr, port, core)
        },
    )
}

// ── Compact render (1-line bar) ────────────────────────────────────────

pub fn render_compact(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.current_palette();
    let (icon, icon_style) = connection_icon(state);
    let (proto, remarks, addr, port, core) = server_summary(state);

    // Server info segment
    let server_str = if addr.is_empty() {
        remarks
    } else {
        format!("{proto}/{remarks} {addr}:{port} [{core}]")
    };

    // Test results segment
    let mut test_parts = Vec::new();
    if let Some(tcp) = state.last_test_tcp {
        test_parts.push(format!("TCP:{tcp}ms"));
    }
    if let Some(rp) = state.last_test_real {
        test_parts.push(format!("RP:{rp}ms"));
    }
    if let Some(spd) = state.last_test_speed {
        let speed_str = if spd >= 1_000_000 {
            format!("{}Mbps", spd / 1_000_000)
        } else if spd >= 1_000 {
            format!("{}Kbps", spd / 1_000)
        } else {
            format!("{spd}bps")
        };
        test_parts.push(format!("SPD:{speed_str}"));
    }
    let test_str = if test_parts.is_empty() {
        String::new()
    } else {
        test_parts.join(" ")
    };

    // Traffic segment
    let traffic_up = format_bytes(state.current_traffic_up);
    let traffic_down = format_bytes(state.current_traffic_down);
    let traffic_str = format!("⬆{traffic_up} ⬇{traffic_down}");

    // Last core log segment (scan log_cache backwards for non-tui entry)
    let log_snippet = state
        .log_cache
        .iter()
        .rev()
        .find(|l| l.target != "tui")
        .map_or("", |l| l.message.as_str());

    // Build the line
    let mut spans = Vec::new();
    spans.push(Span::styled(format!("{icon} "), icon_style));
    spans.push(Span::styled(
        server_str,
        ThemeStyles::footer_value(&palette),
    ));

    if !test_str.is_empty() {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(test_str, ThemeStyles::footer_label(&palette)));
    }

    if state.current_traffic_up != 0 || state.current_traffic_down != 0 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            traffic_str,
            ThemeStyles::footer_value(&palette),
        ));
    }

    if !log_snippet.is_empty() {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            log_snippet,
            ThemeStyles::footer_label(&palette),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).style(ThemeStyles::status_footer(&palette));
    frame.render_widget(paragraph, area);
}

// ── Full render (bordered panel) ───────────────────────────────────────

pub fn render_full(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.current_palette();
    let (icon, icon_style) = connection_icon(state);
    let (proto, remarks, addr, port, core) = server_summary(state);

    // Row 1: Connection status
    let status_text = if state.connecting {
        format!("{icon} Connecting...")
    } else if let Some(core) = &state.connected_core {
        format!("{icon} Connected [{core}]")
    } else if let Some(err) = &state.connection_error {
        format!("{icon} Error: {err}")
    } else {
        format!("{icon} Disconnected")
    };
    let row1 = Line::from(Span::styled(status_text, icon_style));

    // Row 2: Server info
    let server_info = if addr.is_empty() {
        remarks
    } else {
        format!("{proto} {remarks} {addr}:{port} [{core}]")
    };
    let row2 = Line::from(Span::styled(
        if server_info == "No server" || server_info == "- No server" {
            "- No server -".to_string()
        } else {
            format!("🖥 {server_info}")
        },
        ThemeStyles::footer_value(&palette),
    ));

    // Row 3: Test results
    let tcp_str = state
        .last_test_tcp
        .map_or_else(|| "-".to_string(), |v| format!("{v}ms"));
    let rp_str = state
        .last_test_real
        .map_or_else(|| "-".to_string(), |v| format!("{v}ms"));
    let spd_str = state.last_test_speed.map_or_else(
        || "-".to_string(),
        |v| {
            if v >= 1_000_000 {
                format!("{}Mbps", v / 1_000_000)
            } else if v >= 1_000 {
                format!("{}Kbps", v / 1_000)
            } else {
                format!("{v}bps")
            }
        },
    );
    let row3 = Line::from(Span::styled(
        format!("⏱ TCP:{tcp_str}  RP:{rp_str}  SPD:{spd_str}"),
        ThemeStyles::footer_label(&palette),
    ));

    // Row 4: Traffic & memory
    let traffic_up = format_bytes(state.current_traffic_up);
    let traffic_down = format_bytes(state.current_traffic_down);
    let mem_mb = if state.current_memory > 0 {
        format!("{:.1}MB", state.current_memory as f64 / 1_048_576.0)
    } else {
        "-".to_string()
    };
    let row4 = Line::from(Span::styled(
        format!("📊 ⬆{traffic_up}  ⬇{traffic_down}    💾 {mem_mb}"),
        ThemeStyles::footer_value(&palette),
    ));
    // Row 5: Core log (scan log_cache backwards for non-tui entry)
    let core_log = state
        .log_cache
        .iter()
        .rev()
        .find(|l| l.target != "tui")
        .map(|l| format!("📋 Core: [{}] {}", l.level, l.message))
        .unwrap_or_default();
    let row5 = Line::from(Span::styled(core_log, ThemeStyles::footer_label(&palette)));

    // Row 6: TUI log (scan log_cache backwards for tui entry)
    let tui_log = state
        .log_cache
        .iter()
        .rev()
        .find(|l| l.target == "tui")
        .map(|l| format!("📋 TUI:  [{}] {} ({})", l.level, l.message, l.target))
        .unwrap_or_default();
    let row6 = Line::from(Span::styled(tui_log, ThemeStyles::footer_label(&palette)));

    let fieldset = Fieldset::new()
        .title(" ⚙ Actions Log ")
        .styles(FieldsetStyles::from_palette(&palette));
    let inner_area = fieldset.inner(area);
    frame.render_widget(fieldset, area);
    let available = inner_area.height as usize;
    let mut all_rows = vec![row1, row2, row3, row4, row5, row6];
    all_rows.truncate(available);

    for (i, row) in all_rows.into_iter().enumerate() {
        let y = inner_area.y + i as u16;
        if y < inner_area.y + inner_area.height {
            let r = Rect::new(inner_area.x, y, inner_area.width, 1);
            frame.render_widget(Paragraph::new(row), r);
        }
    }
}
