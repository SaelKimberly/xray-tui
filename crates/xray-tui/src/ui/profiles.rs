use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{resolve_core, CoreType};

use crate::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = state.filtered_profiles();
    let selected = state.selected_index;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter strip
            Constraint::Min(1),    // Data grid
        ])
        .split(area);

    render_filter_strip(frame, chunks[0], state);
    render_data_grid(frame, chunks[1], &rows, selected);
}

fn render_filter_strip(frame: &mut Frame, area: Rect, state: &AppState) {
    let group_name = match &state.selected_group_id {
        Some(gid) => state
            .groups
            .iter()
            .find(|g| g.id == *gid)
            .and_then(|g| g.name.as_deref())
            .unwrap_or("Selected"),
        None => "All",
    };

    let group_text = format!(" Group: {}", group_name);
    let group_span = Span::styled(
        group_text,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    );

    let search_text = if state.search_focused {
        format!("/ {}_", state.search_query)
    } else if state.search_query.is_empty() {
        " /  (press / to search)".to_string()
    } else {
        format!("/ {}", state.search_query)
    };
    let search_span = Span::styled(search_text, Style::default().fg(Color::Yellow));

    let line = Line::from(vec![group_span, Span::raw("  "), search_span]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_data_grid(
    frame: &mut Frame,
    area: Rect,
    rows: &[&crate::ProfileRow],
    selected_index: usize,
) {
    let block = Block::default().borders(Borders::TOP);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightBlue)
        .add_modifier(Modifier::BOLD);

    let mut text_lines: Vec<Line> = Vec::new();

    // Header
    let header_line = Line::from(
        vec![
            cell_span("  #", 5),
            cell_span("Type", 8),
            cell_span("Remarks", 24),
            cell_span("Address", 30),
            cell_span("Port", 6),
            cell_span("Delay", 6),
            cell_span("Speed", 6),
            cell_span("Traffic", 10),
            cell_span("Core", 8),
        ]
        .into_iter()
        .map(|s| s.style(header_style))
        .collect::<Vec<_>>(),
    );
    text_lines.push(header_line);

    for (i, row) in rows.iter().enumerate() {
        let is_selected = i == selected_index;

        let protocol = Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
        let core = resolve_core(
            protocol,
            Some(row.profile.core_type.parse::<CoreType>().unwrap_or(CoreType::Auto)),
        );

        let core_color = match core {
            CoreType::Xray => Color::Blue,
            CoreType::SingBox => Color::Green,
            CoreType::Auto => Color::White,
        };

        let row_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightYellow)
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(core_color)
        };

        let idx_str = format!("{:>3}", i + 1);
        let type_str = format!("{:.8}", protocol.to_string());
        let remarks = row.profile.remarks.as_deref().unwrap_or("");
        let remarks_str = truncate_pad(remarks, 24);
        let address = row.profile.address.as_deref().unwrap_or("");
        let address_str = truncate_pad(address, 30);
        let port_str = row
            .profile
            .port
            .map(|p| format!("{:>6}", p))
            .unwrap_or_else(|| "     -".to_string());
        let delay_str = row
            .extension
            .as_ref()
            .and_then(|e| e.delay)
            .map(|d| format!("{:>6}", d))
            .unwrap_or_else(|| "     -".to_string());
        let speed_str = row
            .extension
            .as_ref()
            .and_then(|e| e.speed)
            .map(|s| format!("{:>6}", s))
            .unwrap_or_else(|| "     -".to_string());
        let traffic = row
            .stats
            .as_ref()
            .map(|s| {
                let total = s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0);
                format_traffic(total as u64)
            })
            .unwrap_or_else(|| "        -".to_string());
        let core_str = format!("{:>8}", core.to_string());

        let cells: Vec<Span> = vec![
            Span::raw(format!("{:>5}", idx_str)),
            Span::raw(format!("{:>8}", type_str)),
            Span::raw(format!("{:>24}", remarks_str)),
            Span::raw(format!("{:>30}", address_str)),
            Span::raw(format!("{:>6}", port_str)),
            Span::raw(format!("{:>6}", delay_str)),
            Span::raw(format!("{:>6}", speed_str)),
            Span::raw(format!("{:>10}", traffic)),
            Span::styled(format!("{:>8}", core_str), Style::default().fg(core_color)),
        ];

        let line = Line::from(cells).style(row_style);
        text_lines.push(line);
    }

    let paragraph = Paragraph::new(text_lines);
    frame.render_widget(paragraph, inner);
}

fn cell_span(label: &str, width: usize) -> Span<'_> {
    let padded = format!("{:width$}", label, width = width);
    Span::raw(padded)
}

fn truncate_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        format!("{:width$}", &s[..width.saturating_sub(1)], width = width)
    } else {
        format!("{:width$}", s, width = width)
    }
}

fn format_traffic(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:>4.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:>4.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:>4.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:>4}B ", bytes)
    }
}
