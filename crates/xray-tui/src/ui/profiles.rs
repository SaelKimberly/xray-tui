use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_db::models::GRAVEYARD_GROUP_ID;

use crate::SortColumn;
use crate::ui::render_confirmation_overlay;
use crate::ui::theme::ThemeStyles;
use crate::ui::widgets::data_table::{
    Column, ColumnWidth, DataTable, DataTableRow, DataTableState, SortDirection,
};
use crate::{AppState, ConfirmAction, ProfileRow};
use ratatui_cheese::theme::Palette;

// ── DataTable row wrapper ───────────────────────────────────────────────

struct ProfileTableRowData {
    indicator: String,
    indicator_fg: Style,
    idx_str: String,
    type_str: String,
    remarks_str: String,
    group_str: String,
    address_str: String,
    port_str: String,
    delay_str: String,
    speed_str: String,
    ip_info_str: String,
    traffic_str: String,
    row_style: Style,
    id: String,
}

impl DataTableRow for ProfileTableRowData {
    fn render(&self, col_xs: &[u16], col_widths: &[u16], buf: &mut Buffer, y: u16) {
        for (i, &x) in col_xs.iter().enumerate() {
            let (text, style) = match i {
                0 => (self.indicator.as_str(), self.indicator_fg),
                1 => (self.idx_str.as_str(), self.row_style),
                2 => (self.type_str.as_str(), self.row_style),
                3 => (self.remarks_str.as_str(), self.row_style),
                4 => (self.group_str.as_str(), self.row_style),
                5 => (self.address_str.as_str(), self.row_style),
                6 => (self.port_str.as_str(), self.row_style),
                7 => ("│", self.row_style),
                8 => (self.delay_str.as_str(), self.row_style),
                9 => (self.speed_str.as_str(), self.row_style),
                10 => (self.ip_info_str.as_str(), self.row_style),
                11 => (self.traffic_str.as_str(), self.row_style),
                _ => ("", self.row_style),
            };
            let max_w = col_widths.get(i).copied().unwrap_or(0) as usize;
            buf.set_stringn(x, y, text, max_w, style);
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows: Vec<&ProfileRow> = state.filtered_profiles().collect();
    let selected = state.selected_index;
    let palette = state.current_palette();
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

    render_filter_strip(frame, chunks[0], state, &palette);

    // Progress gauge for batch tests
    if let Some((done, total)) = state.test_progress {
        let progress = done as f64 / total.max(1) as f64;
        let gauge = Gauge::default()
            .gauge_style(ThemeStyles::progress_fill(&palette))
            .label(format!(" Batch: {done}/{total} "))
            .ratio(progress);
        frame.render_widget(gauge, chunks[1]);
    }

    // Empty state guidance
    if rows.is_empty() {
        let msg = if state.filtered_len() == 0 && state.profiles.is_empty() {
            " No profiles — press 'a' to add one "
        } else if state.filtered_len() == 0 {
            " No profiles match the current filter "
        } else {
            unreachable!()
        };
        let paragraph = Paragraph::new(msg)
            .style(ThemeStyles::hint(&palette))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, chunks[2]);
        render_footer(frame, chunks[3], state, &palette);
        render_confirmation_overlays(frame, area, &rows, state);
        return;
    }

    let show_group = state.selected_group_id.is_none()
        || state.selected_group_id.as_deref() == Some(xray_tui_db::models::ALL_GROUP_ID);
    let show_group = show_group && frame.area().width >= 107;

    render_data_grid(
        frame, chunks[2], &rows, selected, show_group, state, &palette,
    );
    render_footer(frame, chunks[3], state, &palette);
    render_confirmation_overlays(frame, area, &rows, state);
}

fn render_filter_strip(frame: &mut Frame, area: Rect, state: &AppState, palette: &Palette) {
    let group_name = state.selected_group_id.as_ref().map_or("All", |gid| {
        state
            .groups
            .iter()
            .find(|g| g.id == *gid)
            .and_then(|g| g.name.as_deref())
            .unwrap_or("All")
    });

    let group_text = format!(" Group: {group_name}");
    let group_span = Span::styled(group_text, ThemeStyles::container_title(palette));

    let search_text = if state.search_focused {
        format!("/ {}▉", state.search_query)
    } else if state.search_query.is_empty() {
        " /  (press / to search)".to_string()
    } else {
        format!("/ {}", state.search_query)
    };
    let search_span = Span::styled(search_text, ThemeStyles::warning(palette));

    let line = Line::from(vec![group_span, Span::raw("  "), search_span]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_data_grid(
    frame: &mut Frame,
    area: Rect,
    rows: &[&ProfileRow],
    selected_index: usize,
    show_group: bool,
    state: &AppState,
    palette: &Palette,
) {
    let block = Block::default()
        .title(" Profiles ")
        .borders(Borders::ALL)
        .border_style(ThemeStyles::container_border(palette))
        .title_style(ThemeStyles::container_title(palette));

    // Map sort state to DataTable indices (fixed 12-column layout)
    let sort_column = match state.sort_column {
        SortColumn::ConfigType => Some(2),
        SortColumn::Remarks => Some(3),
        SortColumn::Address => Some(5),
        SortColumn::Port => Some(6),
        SortColumn::Delay => Some(8),
        SortColumn::Speed => Some(9),
        SortColumn::Traffic => Some(11),
        SortColumn::Core => None,
    };
    let sort_direction = if state.sort_ascending {
        SortDirection::Ascending
    } else {
        SortDirection::Descending
    };

    // Build column definitions
    let mut columns = vec![
        Column::new("", ColumnWidth::Fixed(2)),
        Column::new("#", ColumnWidth::Fixed(5)),
        Column::new("Type", ColumnWidth::Fixed(12)),
        Column::new("Remarks", ColumnWidth::Fixed(24)),
    ];
    // Group column: width 0 when hidden, 12 when shown
    columns.push(Column::new(
        "Group",
        if show_group {
            ColumnWidth::Fixed(12)
        } else {
            ColumnWidth::Fixed(0)
        },
    ));
    columns.extend_from_slice(&[
        Column::new("Address", ColumnWidth::Fixed(30)),
        Column::new("Port", ColumnWidth::Fixed(6)),
        Column::new("│", ColumnWidth::Fixed(1)),
        Column::new("Delay", ColumnWidth::Fixed(6)),
        Column::new("Speed", ColumnWidth::Fixed(6)),
        Column::new("IP", ColumnWidth::Fixed(20)),
        Column::new("Traffic", ColumnWidth::Fixed(10)),
    ]);

    // Build DataTable rows
    let data_rows: Vec<ProfileTableRowData> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == selected_index;
            let is_connected = state.connected_profile_id.as_deref() == Some(&row.profile.id);
            let row_style = match (is_selected, is_connected) {
                (true, true) => ThemeStyles::table_row_connected(palette)
                    .add_modifier(ratatui::style::Modifier::UNDERLINED),
                (false, true) => ThemeStyles::table_row_connected(palette),
                (true, false) => ThemeStyles::table_row_selected(palette),
                (false, false) if i % 2 == 1 => ThemeStyles::table_row_alt(palette),
                (false, false) => ThemeStyles::table_row_normal(palette),
            };

            let (indicator, indicator_fg) = if is_connected {
                ("●".to_string(), ThemeStyles::success(palette))
            } else if let Ok(pid) = row.profile.id.parse::<uuid::Uuid>() {
                match state.testing_details.get(&pid) {
                    Some(TestType::TcpPing) => ("↔".to_string(), Style::default()),
                    Some(TestType::RealPing) => ("◎".to_string(), Style::default()),
                    Some(TestType::SpeedTest) => ("⇩".to_string(), Style::default()),
                    Some(TestType::UdpTest) => ("↗".to_string(), Style::default()),
                    None => (String::new(), Style::default()),
                }
            } else {
                (String::new(), Style::default())
            };

            let protocol =
                Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
            let is_multi = state.multi_select.contains(&row.profile.id);

            let idx_str = if is_multi {
                "  *".to_string()
            } else {
                format!("{:>3}", i + 1)
            };

            let type_str = format!("{protocol:.12}");
            let remarks = row.profile.remarks.as_deref().unwrap_or("");
            let remarks_str = truncate_pad(remarks, 24);
            let group_str = row
                .profile
                .group_id
                .as_deref()
                .filter(|gid| *gid != GRAVEYARD_GROUP_ID)
                .and_then(|gid| state.groups.iter().find(|g| g.id == *gid))
                .and_then(|g| g.name.as_deref())
                .map_or_else(String::new, |name| truncate_pad(name, 12));
            let address = row.profile.address.as_deref().unwrap_or("");
            let address_str = truncate_pad(address, 30);
            let port_str = row
                .profile
                .port
                .map_or_else(|| "     -".to_string(), |p| format!("{p:>6}"));
            let delay_str = row
                .extension
                .as_ref()
                .and_then(|e| e.delay)
                .map_or_else(|| "     -".to_string(), |d| format!("{d:>6}"));
            let speed_str = row
                .extension
                .as_ref()
                .and_then(|e| e.speed)
                .map_or_else(|| "     -".to_string(), |s| format!("{s:>6}"));
            let ip_info_str = row
                .extension
                .as_ref()
                .and_then(|e| e.ip_info.as_deref())
                .map_or_else(|| "     -".to_string(), |ip| truncate_pad(ip, 19));
            let traffic = row.stats.as_ref().map_or_else(
                || "        -".to_string(),
                |s| {
                    let total = s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0);
                    format_traffic(total as u64)
                },
            );

            ProfileTableRowData {
                id: row.profile.id.clone(),
                indicator,
                indicator_fg,
                idx_str,
                type_str,
                remarks_str,
                group_str,
                address_str,
                port_str,
                delay_str,
                speed_str,
                ip_info_str,
                traffic_str: traffic,
                row_style,
            }
        })
        .collect();
    // Scroll offset: keep selected row roughly centered
    let inner_height = area.height.saturating_sub(3) as usize;
    let data_offset = if selected_index > inner_height / 2 {
        selected_index
            .saturating_sub(inner_height / 2)
            .min(data_rows.len().saturating_sub(inner_height))
    } else {
        0
    };
    let data_table = DataTable::new(columns, &data_rows)
        .highlight_style(ThemeStyles::table_row_selected(palette))
        .column_spacing(0)
        .block(block)
        .sort_column(sort_column, sort_direction);

    let mut table_state = DataTableState {
        offset: data_offset,
        selected: Some(selected_index),
        multi_selected: std::collections::HashSet::new(),
    };

    // Populate multi_selected from state.multi_select
    for (i, row) in data_rows.iter().enumerate() {
        if state.multi_select.contains(&row.id) {
            table_state.multi_selected.insert(i);
        }
    }

    frame.render_stateful_widget(data_table, area, &mut table_state);
}

fn format_traffic(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:>4.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:>4.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:>4.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes:>4}B ")
    }
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
        format!("{s:width$}")
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState, palette: &Palette) {
    if area.height < 1 {
        return;
    }

    let has_profile = state.selected_index < state.profiles.len();

    let line = if has_profile {
        let row = &state.profiles[state.selected_index];
        let core = state.resolved_core(row);
        let remarks = row.profile.remarks.as_deref().unwrap_or("-");
        let addr = row.profile.address.as_deref().unwrap_or("-");
        let port = row
            .profile
            .port
            .map_or_else(|| "-".into(), |p| p.to_string());
        Line::from(vec![
            Span::styled(" Server: ", ThemeStyles::footer_label(palette)),
            Span::styled(remarks, ThemeStyles::footer_value(palette)),
            Span::styled(
                format!("  {addr}:{port}  "),
                ThemeStyles::footer_value(palette),
            ),
            Span::styled(format!("[{core}] "), ThemeStyles::footer_value(palette)),
        ])
    } else {
        Line::from(Span::styled(
            " Server: (none selected)",
            ThemeStyles::footer_label(palette),
        ))
    };
    let footer = Paragraph::new(line).style(ThemeStyles::status_footer(palette));
    frame.render_widget(footer, area);
}

fn render_confirmation_overlays(
    frame: &mut Frame,
    area: Rect,
    rows: &[&ProfileRow],
    state: &AppState,
) {
    match state.confirmation {
        Some(ConfirmAction::DeleteProfile(ref delete_id)) => {
            let profile_name = rows
                .iter()
                .find(|r| r.profile.id == *delete_id)
                .and_then(|r| r.profile.remarks.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(
                frame,
                area,
                &format!(" Delete \"{profile_name}\"? (y/N) "),
            );
        }
        Some(ConfirmAction::ClearGroup(ref group_id)) => {
            let group_name = state
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .and_then(|g| g.name.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(
                frame,
                area,
                &format!(" Clear all profiles in \"{group_name}\"? (y/N) "),
            );
        }
        _ => {}
    }
}
