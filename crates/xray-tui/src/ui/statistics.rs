use crate::AppState;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
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

    // Split area into 3 sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Traffic
            Constraint::Length(5), // System
            Constraint::Min(3),    // Connection
        ])
        .split(area);

    // ── Traffic section ────────────────────────────────────────────
    let mut traffic_lines: Vec<Line> = Vec::new();
    if let Some(ref stats) = profile.stats {
        let today_up = stats.today_up.unwrap_or(0) as i64;
        let today_down = stats.today_down.unwrap_or(0) as i64;
        let total_up = stats.total_up.unwrap_or(0) as i64;
        let total_down = stats.total_down.unwrap_or(0) as i64;

        traffic_lines.push(Line::from(vec![
            Span::raw("  Today:  "),
            Span::styled("↑", Style::default().fg(Color::Green)),
            Span::raw(format!(" {}  ", format_bytes(today_up))),
            Span::styled("↓", Style::default().fg(Color::Red)),
            Span::raw(format!(" {}", format_bytes(today_down))),
        ]));
        traffic_lines.push(Line::from(vec![
            Span::raw("  Total:  "),
            Span::styled("↑", Style::default().fg(Color::Green)),
            Span::raw(format!(" {}  ", format_bytes(total_up))),
            Span::styled("↓", Style::default().fg(Color::Red)),
            Span::raw(format!(" {}", format_bytes(total_down))),
        ]));
    } else {
        traffic_lines.push(Line::from("  No traffic data yet"));
    }
    let traffic_block = Block::default()
        .title(format!(" Traffic — {profile_name} [{core_type}] "))
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let traffic_para = Paragraph::new(traffic_lines).block(traffic_block);
    frame.render_widget(traffic_para, chunks[0]);

    // ── System section ─────────────────────────────────────────────
    let mut sys_lines: Vec<Line> = Vec::new();
    if let Some(ref sys) = state.system_stats {
        sys_lines.push(Line::from(format!(
            "  Memory:    {} / {}",
            format_bytes(sys.alloc as i64),
            format_bytes(sys.sys as i64),
        )));
        sys_lines.push(Line::from(format!("  Routines:  {}", sys.num_goroutine)));
        sys_lines.push(Line::from(format!(
            "  Uptime:    {}",
            format_uptime(sys.uptime),
        )));
    } else {
        sys_lines.push(Line::from("  No system data yet"));
    }
    let sys_block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let sys_para = Paragraph::new(sys_lines).block(sys_block);
    frame.render_widget(sys_para, chunks[1]);

    // ── Connection section ─────────────────────────────────────────
    let status_style = if connected {
        Theme::SUCCESS
    } else {
        Theme::ERROR
    };
    let conn_lines = vec![
        Line::from(format!("  API endpoint:  {API_ENDPOINT}")),
        Line::from(vec![
            Span::raw("  Status:       "),
            Span::styled("Connected", status_style),
        ]),
    ];
    let conn_block = Block::default()
        .title(" Connection ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let conn_para = Paragraph::new(conn_lines).block(conn_block);
    frame.render_widget(conn_para, chunks[2]);
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Statistics ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let paragraph =
        Paragraph::new(" Not connected — select a profile and press Ctrl+Enter to connect ")
            .style(Theme::HINT)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, area);
}
