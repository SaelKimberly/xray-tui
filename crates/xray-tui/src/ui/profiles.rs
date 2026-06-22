use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{CoreType, resolve_core, format_bytes, format_uptime};

use crate::ui::theme::Theme;
use crate::{AppState, ConfirmAction};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = state.filtered_profiles();
    let selected = state.selected_index;
    let gauge_height = state.test_progress.map_or(0, |_| 1u16);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(gauge_height),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_filter_strip(frame, chunks[0], state);

    // Progress gauge for batch tests
    if let Some((done, total)) = state.test_progress {
        let progress = done as f64 / total.max(1) as f64;
        let gauge = Gauge::default()
            .gauge_style(Theme::PROGRESS_FILL)
            .label(format!(" Batch: {done}/{total} "))
            .ratio(progress);
        frame.render_widget(gauge, chunks[1]);
    }

    // Window: only build Row widgets for rows within the visible area
    let inner_height = chunks[2].height.saturating_sub(1) as usize;
    let total = rows.len();
    let scroll_offset = if total <= inner_height {
        0
    } else {
        selected.saturating_sub(inner_height / 2).min(total - inner_height)
    };
    let visible_end = (scroll_offset + inner_height).min(total);
    let visible_rows = &rows[scroll_offset..visible_end];
    let adjusted_selected = selected - scroll_offset;

    render_data_grid(frame, chunks[2], visible_rows, adjusted_selected, scroll_offset, state);
    render_footer(frame, chunks[3], state);
    // Confirmation overlays: DeleteProfile and ClearGroup
    match state.confirmation {
        Some(ConfirmAction::DeleteProfile(ref delete_id)) => {
            let profile_name = rows
                .iter()
                .find(|r| r.profile.id == *delete_id)
                .and_then(|r| r.profile.remarks.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(frame, area, &format!(" Delete \"{profile_name}\"? (y/N) "));
        }
        Some(ConfirmAction::ClearGroup(ref group_id)) => {
            let group_name = state
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .and_then(|g| g.name.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(frame, area, &format!(" Clear all profiles in \"{group_name}\"? (y/N) "));
        }
        _ => {}
    }
}

fn render_confirmation_overlay(frame: &mut Frame, area: Rect, text: &str) {
    let overlay_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Red)
        .add_modifier(Modifier::BOLD);
    let overlay_para = Paragraph::new(text.to_string()).style(overlay_style);
    let overlay_area = Rect::new(
        area.width
            .saturating_sub(text.len() as u16 + 4)
            .min(area.width),
        area.height.saturating_sub(2),
        text.len() as u16 + 4,
        1,
    );
    frame.render_widget(overlay_para, overlay_area);
}

fn render_filter_strip(frame: &mut Frame, area: Rect, state: &AppState) {
    let group_name = match &state.selected_group_id {
        Some(gid) => state
            .groups
            .iter()
            .find(|g| g.id == *gid)
            .and_then(|g| g.name.as_deref())
            .unwrap_or("All"),
        None => "All",
    };

    let group_text = format!(" Group: {}", group_name);
    let group_span = Span::styled(group_text, Theme::CONTAINER_TITLE);

    let search_text = if state.search_focused {
        format!("/ {}_", state.search_query)
    } else if state.search_query.is_empty() {
        " /  (press / to search)".to_string()
    } else {
        format!("/ {}", state.search_query)
    };
    let search_span = Span::styled(search_text, Theme::WARNING);

    let line = Line::from(vec![group_span, Span::raw("  "), search_span]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_data_grid(
    frame: &mut Frame,
    area: Rect,
    rows: &[&crate::ProfileRow],
    selected_index: usize,
    scroll_offset: usize,
    state: &AppState,
) {
    let block = Block::default()
        .title(" Profiles ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Header row
    let header_cells = [
        " #  ", "Type    ", "Remarks                  ",
        "Address                        ", "Port  ",
        "Delay ", "Speed ", "Traffic   ", "Core    ",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Theme::TABLE_HEADER));
    let header = Row::new(header_cells);

    // Data rows
    let data_rows: Vec<Row> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == selected_index;
            let row_style = if is_selected {
                Theme::TABLE_ROW_SELECTED
            } else if i % 2 == 1 {
                Theme::TABLE_ROW_ALT
            } else {
                Theme::TABLE_ROW_NORMAL
            };

            let protocol =
                Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
            let core = resolve_core(
                protocol,
                Some(
                    row.profile
                        .core_type
                        .parse::<CoreType>()
                        .unwrap_or(CoreType::Auto),
                ),
            );

            let core_color = match core {
                CoreType::Xray => Color::Blue,
                CoreType::SingBox => Color::Green,
                CoreType::Auto => Color::White,
            };

            let is_multi = state.multi_select.contains(&row.profile.id);
            let idx_str = if is_multi {
                "  *".to_string()
            } else {
                format!("{:>3}", scroll_offset + i + 1)
            };

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
            let core_str = core.to_string();

            let cells = vec![
                Cell::from(idx_str),
                Cell::from(type_str),
                Cell::from(remarks_str),
                Cell::from(address_str),
                Cell::from(port_str.trim().to_string()),
                Cell::from(delay_str.trim().to_string()),
                Cell::from(speed_str.trim().to_string()),
                Cell::from(traffic.trim().to_string()),
                Cell::from(core_str).style(Style::default().fg(core_color)),
            ];

            Row::new(cells).style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(24),
        Constraint::Length(30),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(8),
    ];

    let table = Table::new(data_rows, widths)
        .header(header)
        .column_spacing(0);
    frame.render_widget(table, inner);
}

pub(crate) fn truncate_pad(s: &str, width: usize) -> String {
    if unicode_width::UnicodeWidthStr::width(s) > width {
        let mut char_w = 0usize;
        let mut end = 0usize;
        for (i, c) in s.char_indices() {
            let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if char_w + cw > width.saturating_sub(1) {
                break;
            }
            char_w += cw;
            end = i + c.len_utf8();
        }
        format!("{:width$}", &s[..end], width = width)
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

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 1 {
        return;
    }

    let has_profile = state.selected_index < state.profiles.len();
    let connected = state.connected_core.is_some();

    let mut lines = Vec::new();

    // Line 1: server info (always present if profiles exist)
    if has_profile {
        let row = &state.profiles[state.selected_index];
        let protocol = Protocol::try_from_i32(row.profile.config_type)
            .unwrap_or(Protocol::Custom);
        let core = resolve_core(
            protocol,
            Some(row.profile.core_type.parse::<CoreType>().unwrap_or(CoreType::Auto)),
        );
        let remarks = row.profile.remarks.as_deref().unwrap_or("-");
        let addr = row.profile.address.as_deref().unwrap_or("-");
        let port = row.profile.port.map(|p| p.to_string()).unwrap_or_else(|| "-".into());

        lines.push(Line::from(vec![
            Span::styled(" Server: ", Theme::FOOTER_LABEL),
            Span::styled(remarks, Theme::FOOTER_VALUE),
            Span::styled(format!("  {}:{}  ", addr, port), Theme::FOOTER_VALUE),
            Span::styled(format!("[{}] ", protocol), Theme::FOOTER_VALUE),
            Span::styled(core.to_string(), Theme::FOOTER_VALUE),
        ]));
    } else {
        lines.push(Line::from(Span::styled(" Server: (none selected)", Theme::FOOTER_LABEL)));
    }

    // Line 2: traffic
    if connected && has_profile {
        if let Some(ref stats) = state.profiles[state.selected_index].stats {
            let tu = format_traffic(stats.total_up.unwrap_or(0) as u64);
            let td = format_traffic(stats.total_down.unwrap_or(0) as u64);
            let du = format_traffic(stats.today_up.unwrap_or(0) as u64);
            let dd = format_traffic(stats.today_down.unwrap_or(0) as u64);
            lines.push(Line::from(vec![
                Span::styled(" Traffic: ", Theme::FOOTER_LABEL),
                Span::styled(format!("Today {}↑  {}↓  ", du.trim(), dd.trim()), Theme::FOOTER_VALUE),
                Span::styled(format!("Total {}↑  {}↓", tu.trim(), td.trim()), Theme::FOOTER_VALUE),
            ]));
        } else {
            lines.push(Line::from(Span::styled(" Traffic: (no data yet)", Theme::FOOTER_LABEL)));
        }
    } else {
        lines.push(Line::from(Span::styled(" Traffic: (not connected)", Theme::FOOTER_LABEL)));
    }

    // Line 3: system stats
    if connected {
        if let Some(ref sys) = state.system_stats {
            let mem = format_bytes(sys.alloc as i64);
            lines.push(Line::from(vec![
                Span::styled(" System: ", Theme::FOOTER_LABEL),
                Span::styled(format!("Mem: {}  ", mem), Theme::FOOTER_VALUE),
                Span::styled(format!("Goroutines: {}  ", sys.num_goroutine), Theme::FOOTER_VALUE),
                Span::styled(format!("Uptime: {}", format_uptime(sys.uptime)), Theme::FOOTER_VALUE),
            ]));
        } else {
            lines.push(Line::from(Span::styled(" System: (no data yet)", Theme::FOOTER_LABEL)));
        }
    } else {
        lines.push(Line::from(Span::styled(" System: (not connected)", Theme::FOOTER_LABEL)));
    }

    let footer = Paragraph::new(lines).style(Theme::STATUS_FOOTER);
    frame.render_widget(footer, area);
}
