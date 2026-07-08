use crate::AppState;
use crate::ui::theme::ThemeStyles;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_cheese::fieldset::{Fieldset, FieldsetStyles};
use xray_tui_core::{API_ENDPOINT, format_bytes, format_uptime};
use xray_tui_config::import_export::ProfileLegacy;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.current_palette();
    let connected = state.connected_core.is_some();
    let has_profile = !state.profiles.is_empty() && state.selected_index < state.profiles.len();

    if !connected || !has_profile {
        render_placeholder(frame, area, &palette);
        return;
    }

    let profile = &state.profiles[state.selected_index];
    let profile_name = profile.profile.leg("remarks").unwrap_or_else(|| "Unknown".to_string());
    let core_type = state.connected_core.map_or("", |c| c.as_str());

    // Split area into 3 sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(3),
        ])
        .split(area);

    let traffic_title = format!(" Traffic — {profile_name} [{core_type}] ");
    let traffic_fieldset = Fieldset::new()
        .title(&traffic_title)
        .styles(FieldsetStyles::from_palette(&palette));
    let traffic_inner = traffic_fieldset.inner(chunks[0]);
    frame.render_widget(traffic_fieldset, chunks[0]);

    let mut traffic_lines: Vec<Line> = Vec::new();
    if let Some(ref stats) = profile.stats {
        let today_up = stats.today_up.unwrap_or(0);
        let today_down = stats.today_down.unwrap_or(0);
        let total_up = stats.total_up.unwrap_or(0);
        let total_down = stats.total_down.unwrap_or(0);

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
    frame.render_widget(Paragraph::new(traffic_lines), traffic_inner);

    // ── System section ─────────────────────────────────────────────
    let sys_fieldset = Fieldset::new()
        .title(" System ")
        .styles(FieldsetStyles::from_palette(&palette));
    let sys_inner = sys_fieldset.inner(chunks[1]);
    frame.render_widget(sys_fieldset, chunks[1]);

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
            format_uptime(sys.uptime)
        )));
    } else {
        sys_lines.push(Line::from("  No system data yet"));
    }
    frame.render_widget(Paragraph::new(sys_lines), sys_inner);

    // ── Connection section ─────────────────────────────────────────
    let status_style = if connected {
        ThemeStyles::success(&palette)
    } else {
        ThemeStyles::error(&palette)
    };
    let conn_lines = vec![
        Line::from(format!("  API endpoint:  {API_ENDPOINT}")),
        Line::from(vec![
            Span::raw("  Status:       "),
            Span::styled("Connected", status_style),
        ]),
    ];
    let conn_fieldset = Fieldset::new()
        .title(" Connection ")
        .styles(FieldsetStyles::from_palette(&palette));
    let conn_inner = conn_fieldset.inner(chunks[2]);
    frame.render_widget(conn_fieldset, chunks[2]);
    frame.render_widget(Paragraph::new(conn_lines), conn_inner);
}

fn render_placeholder(frame: &mut Frame, area: Rect, palette: &ratatui_cheese::theme::Palette) {
    let fieldset = Fieldset::new()
        .title(" Statistics ")
        .styles(FieldsetStyles::from_palette(palette));
    let inner = fieldset.inner(area);
    frame.render_widget(fieldset, area);
    let paragraph =
        Paragraph::new(" Not connected — select a profile and press Ctrl+Enter to connect ")
            .style(ThemeStyles::hint(palette))
            .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner);
}
