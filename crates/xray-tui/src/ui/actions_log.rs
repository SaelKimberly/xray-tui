use xray_tui_core::grpc_client::format_bytes;
use xray_tui_core::protocol::Protocol;

use crate::AppState;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const fn connection_icon(state: &AppState) -> (&'static str, Style) {
    if state.connecting {
        ("⠋", Theme::SPINNER)
    } else if state.connected_core.is_some() {
        ("●", Theme::SUCCESS)
    } else if state.connection_error.is_some() {
        ("⏹", Theme::ERROR)
    } else if state.connected_profile_id.is_some() {
        ("⏏", Theme::WARNING)
    } else {
        ("○", Theme::HINT)
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
            let remarks = r.profile.remarks.clone().unwrap_or_default();
            let addr = r.profile.address.clone().unwrap_or_default();
            let port = r.profile.port.unwrap_or(0) as u16;
            let core = state.resolved_core(r).to_string();
            (proto.to_string(), remarks, addr, port, core)
        },
    )
}

// ── Compact render (1-line bar) ────────────────────────────────────────

pub fn render_compact(frame: &mut Frame, area: Rect, state: &AppState) {
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

    // Last core log segment
    let log_snippet = state
        .last_core_log
        .as_ref()
        .map_or("", |(_, msg)| msg.as_str());

    // Build the line
    let mut spans = Vec::new();
    spans.push(Span::styled(format!("{icon} "), icon_style));
    spans.push(Span::styled(server_str, Theme::FOOTER_VALUE));

    if !test_str.is_empty() {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(test_str, Theme::FOOTER_LABEL));
    }

    if state.current_traffic_up != 0 || state.current_traffic_down != 0 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(traffic_str, Theme::FOOTER_VALUE));
    }

    if !log_snippet.is_empty() {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(log_snippet, Theme::FOOTER_LABEL));
    }

    let paragraph = Paragraph::new(Line::from(spans)).style(Theme::STATUS_FOOTER);
    frame.render_widget(paragraph, area);
}

// ── Full render (bordered panel) ───────────────────────────────────────

pub fn render_full(frame: &mut Frame, area: Rect, state: &AppState) {
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
        Theme::FOOTER_VALUE,
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
        Theme::FOOTER_LABEL,
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
        Theme::FOOTER_VALUE,
    ));

    // Row 5: Core log
    let core_log = state
        .last_core_log
        .as_ref()
        .map(|(lvl, msg)| format!("📋 Core: [{lvl}] {msg}"))
        .unwrap_or_default();
    let row5 = Line::from(Span::styled(core_log, Theme::FOOTER_LABEL));

    // Row 6: TUI log
    let tui_log = state
        .last_tui_log
        .as_ref()
        .map(|(target, lvl, msg)| format!("📋 TUI:  [{lvl}] {msg} ({target})"))
        .unwrap_or_default();
    let row6 = Line::from(Span::styled(tui_log, Theme::FOOTER_LABEL));

    // Block with border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title(Span::styled(" ⚙ Actions Log ", Theme::CONTAINER_TITLE));
    frame.render_widget(block, area);

    // Render rows inside the bordered area (offset by 1 for border)
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
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
