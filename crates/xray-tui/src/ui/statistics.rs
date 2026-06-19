use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::ui::theme::Theme;
use crate::AppState;
use xray_tui_core::{API_ENDPOINT, format_bytes, format_uptime};
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let connected = state.connected_core.is_some();
    let has_profile = !state.profiles.is_empty() && state.selected_index < state.profiles.len();

    if !connected || !has_profile {
        render_placeholder(frame, area);
        return;
    }

    let profile = &state.profiles[state.selected_index];
    let profile_name = profile.profile.remarks.as_deref().unwrap_or("Unknown");
    let core_type = state
        .connected_core
        .map(|c| c.to_string())
        .unwrap_or_default();

    let mut lines: Vec<Line> = Vec::new();

    // ── Title ──────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("Statistics ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("— {profile_name} [{core_type}]")),
    ]));
    lines.push(Line::from("───"));

    // ── Traffic section ────────────────────────────────────────────
    if let Some(ref stats) = profile.stats {
        lines.push(Line::from(Span::styled(
            "Traffic",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )));

        let today_up = stats.today_up.unwrap_or(0) as i64;
        let today_down = stats.today_down.unwrap_or(0) as i64;
        let total_up = stats.total_up.unwrap_or(0) as i64;
        let total_down = stats.total_down.unwrap_or(0) as i64;

        lines.push(Line::from(vec![
            Span::raw("  Today:  "),
            Span::styled("↑", Style::default().fg(Color::Green)),
            Span::raw(format!(" {}  ", format_bytes(today_up))),
            Span::styled("↓", Style::default().fg(Color::Red)),
            Span::raw(format!(" {}", format_bytes(today_down))),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  Total:  "),
            Span::styled("↑", Style::default().fg(Color::Green)),
            Span::raw(format!(" {}  ", format_bytes(total_up))),
            Span::styled("↓", Style::default().fg(Color::Red)),
            Span::raw(format!(" {}", format_bytes(total_down))),
        ]));
        lines.push(Line::from(""));
    } else {
        lines.push(Line::from(Span::styled(
            "Traffic",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )));
        lines.push(Line::from("  No traffic data yet"));
        lines.push(Line::from(""));
    }

    // ── System section ─────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "System",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));

    if let Some(ref sys) = state.system_stats {
        lines.push(Line::from(format!(
            "  Memory:    {} / {}",
            format_bytes(sys.alloc as i64),
            format_bytes(sys.sys as i64),
        )));
        lines.push(Line::from(format!("  Routines:  {}", sys.num_goroutine,)));
        lines.push(Line::from(format!(
            "  Uptime:    {}",
            format_uptime(sys.uptime),
        )));
    } else {
        lines.push(Line::from("  No system data yet"));
    }
    lines.push(Line::from(""));

    // ── Connection info ────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "Connection",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));
    lines.push(Line::from(format!("  API endpoint:  {API_ENDPOINT}")));

    let status_style = if connected {
        Theme::SUCCESS
    } else {
        Theme::ERROR
    };
    lines.push(Line::from(vec![
        Span::raw("  Status:       "),
        Span::styled("Connected", status_style),
    ]));

    let block = Block::default()
        .title(" Statistics ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let block = Block::default().title(" Statistics ").borders(Borders::ALL);
    let paragraph = Paragraph::new("No data — connect to a server")
        .block(block)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, area);
}
