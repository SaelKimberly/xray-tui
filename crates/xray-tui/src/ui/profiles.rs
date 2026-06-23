use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};
use xray_tui_core::protocol::Protocol;


use crate::ui::theme::Theme;
use crate::ui::render_confirmation_overlay;
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
            Constraint::Length(1),
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
    let inner_height = chunks[2].height.saturating_sub(3) as usize;
    let total = rows.len();
    let scroll_offset = if total <= inner_height {
        0
    } else {
        selected.saturating_sub(inner_height / 2).min(total - inner_height)
    };
    let visible_end = (scroll_offset + inner_height).min(total);
    let visible_rows = &rows[scroll_offset..visible_end];
    let adjusted_selected = selected - scroll_offset;

    let show_group = state.selected_group_id.is_none()
        || state.selected_group_id.as_deref() == Some(xray_tui_db::models::ALL_GROUP_ID);
    // Hide Group column on narrow terminals (needs ≥107 cols to fit)
    let show_group = show_group && frame.area().width >= 107;
    render_data_grid(frame, chunks[2], visible_rows, adjusted_selected, scroll_offset, show_group, state);
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
    show_group: bool,
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
    let mut header_items = vec![
        " #  ", "Type    ", "Remarks                  ",
        "Address                        ", "Port  ",
        "Delay ", "Speed ", "Traffic   ",
    ];
    if show_group {
        header_items.insert(3, "Group       ");
    }
    let header_cells = header_items.iter()
        .map(|h| Cell::from(*h).style(Theme::TABLE_HEADER));
    let header = Row::new(header_cells);

    // Column widths
    let mut widths = vec![
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(24),
        Constraint::Length(30),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
    ];
    if show_group {
        widths.insert(3, Constraint::Length(12));
    }

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

            // Build cells — insert Group cell after remarks if on All tab
            let mut cells = vec![
                Cell::from(idx_str),
                Cell::from(type_str),
                Cell::from(remarks_str.clone()),
            ];
            if show_group {
                let group_name = row.profile.group_id.as_deref()
                    .and_then(|gid| state.groups.iter().find(|g| g.id == *gid))
                    .and_then(|g| g.name.as_deref())
                    .unwrap_or("-");
                cells.push(Cell::from(truncate_pad(group_name, 12)));
            }
            cells.extend_from_slice(&[
                Cell::from(address_str),
                Cell::from(port_str.trim().to_string()),
                Cell::from(delay_str.trim().to_string()),
                Cell::from(speed_str.trim().to_string()),
                Cell::from(traffic.trim().to_string()),
            ]);

            Row::new(cells).style(row_style)
        })
        .collect();
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

    let line = if has_profile {
        let row = &state.profiles[state.selected_index];
        let core = state.resolved_core(row);
        let remarks = row.profile.remarks.as_deref().unwrap_or("-");
        let addr = row.profile.address.as_deref().unwrap_or("-");
        let port = row.profile.port.map(|p| p.to_string()).unwrap_or_else(|| "-".into());
        Line::from(vec![
            Span::styled(" Server: ", Theme::FOOTER_LABEL),
            Span::styled(remarks, Theme::FOOTER_VALUE),
            Span::styled(format!("  {}:{}  ", addr, port), Theme::FOOTER_VALUE),
            Span::styled(format!("[{}] ", core), Theme::FOOTER_VALUE),
        ])
    } else {
        Line::from(Span::styled(" Server: (none selected)", Theme::FOOTER_LABEL))
    };
    let footer = Paragraph::new(line).style(Theme::STATUS_FOOTER);
    frame.render_widget(footer, area);
}
