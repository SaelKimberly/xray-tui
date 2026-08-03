use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;

use crate::SortColumn;
use crate::ui::render_confirmation_overlay;
use crate::ui::theme::ThemeStyles;
use crate::ui::widgets::data_table::{
    Column, ColumnWidth, DataTable, DataTableRow, DataTableState, SortDirection,
};
use crate::{AppState, ConfirmAction, EndpointRow};

/// A row displayed in the profile `DataTable` — either an endpoint header or a protocol sub-row.
enum DisplayRowData {
    Endpoint {
        id: String,
        indicator: String,
        indicator_fg: Style,
        idx_str: String,
        type_str: String,
        remarks_str: String,
        address_str: String,
        port_str: String,
        delay_str: String,
        speed_str: String,
        ip_info_str: String,
        traffic_str: String,
        has_sub_rows: bool,
        expanded: bool,
        row_style: Style,
    },
    ProtocolSub {
        id: String,
        proto_kind: String,
        transport: String,
        security: String,
        remarks: String,
        is_active: bool,
        is_manual: bool,
        row_style: Style,
    },
}

impl DataTableRow for DisplayRowData {
    fn render(&self, col_xs: &[u16], col_widths: &[u16], buf: &mut Buffer, y: u16) {
        match self {
            Self::Endpoint {
                indicator,
                indicator_fg,
                idx_str,
                type_str,
                remarks_str,
                address_str,
                port_str,
                delay_str,
                speed_str,
                ip_info_str,
                traffic_str,
                has_sub_rows,
                expanded,
                row_style,
                ..
            } => {
                for (i, &x) in col_xs.iter().enumerate() {
                    let tree_marker = if *has_sub_rows {
                        if *expanded { "▾" } else { "▶" }
                    } else {
                        " "
                    };
                    let (text, style) = match i {
                        0 => (tree_marker, *row_style),
                        1 => (indicator.as_str(), *indicator_fg),
                        2 => (idx_str.as_str(), *row_style),
                        3 => (type_str.as_str(), *row_style),
                        4 => (remarks_str.as_str(), *row_style),
                        5 => ("│", *row_style),
                        6 => (address_str.as_str(), *row_style),
                        7 => (port_str.as_str(), *row_style),
                        8 => ("│", *row_style),
                        9 => (delay_str.as_str(), *row_style),
                        10 => (speed_str.as_str(), *row_style),
                        11 => (ip_info_str.as_str(), *row_style),
                        12 => (traffic_str.as_str(), *row_style),
                        _ => ("", *row_style),
                    };
                    let max_w = col_widths.get(i).copied().unwrap_or(0) as usize;
                    buf.set_stringn(x, y, text, max_w, style);
                }
            }
            Self::ProtocolSub {
                proto_kind,
                transport,
                security,
                remarks,
                is_active,
                is_manual,
                row_style,
                ..
            } => {
                let active_mark = if *is_active { "●" } else { "○" };
                let manual_label = if *is_manual { " (u)" } else { "" };
                let remarks_span_w = col_widths.get(4).copied().unwrap_or(0)
                    + col_widths.get(5).copied().unwrap_or(0)
                    + col_widths.get(6).copied().unwrap_or(0);
                let sub_remarks =
                    crate::ui::profiles::truncate_pad(remarks, remarks_span_w as usize);
                for (i, &x) in col_xs.iter().enumerate() {
                    let (text, style) = match i {
                        0 => (" ", *row_style),
                        1 => (active_mark, *row_style),
                        2 => ("  └", *row_style),
                        3 => (proto_kind.as_str(), *row_style),
                        4 => (sub_remarks.as_str(), *row_style),
                        5 => ("", *row_style),
                        6 => ("", *row_style),
                        7 => (transport.as_str(), *row_style),
                        8 => (security.as_str(), *row_style),
                        9 => (manual_label, *row_style),
                        _ => ("", *row_style),
                    };
                    let max_w = if i == 4 {
                        remarks_span_w as usize
                    } else {
                        col_widths.get(i).copied().unwrap_or(0) as usize
                    };
                    buf.set_stringn(x, y, text, max_w, style);
                }
            }
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows: Vec<&EndpointRow> = state.filtered_profiles().collect();
    let selected = state.selected_index;
    let palette = state.current_palette();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_filter_strip(frame, chunks[0], state, &palette);

    // Empty state guidance
    if rows.is_empty() {
        let msg = if state.filtered_len() == 0 && state.endpoints.is_empty() {
            " No profiles — press 'a' to add one "
        } else if state.filtered_len() == 0 {
            " No profiles match the current filter "
        } else {
            unreachable!()
        };
        let paragraph = Paragraph::new(msg)
            .style(ThemeStyles::hint(&palette))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, chunks[1]);
        render_footer(frame, chunks[2], state, &palette);
        return;
    }

    // Build display rows with optional sub-rows for expanded endpoints
    let display_rows = build_display_rows(&rows, selected, state, &palette);
    let display_selected = resolve_display_selected(&display_rows, selected, state.selected_sub);

    render_data_grid(
        frame,
        chunks[1],
        &display_rows,
        display_selected,
        state,
        &palette,
    );
    render_footer(frame, chunks[2], state, &palette);
    render_confirmation_overlays(frame, area, &rows, state);
}

fn build_display_rows(
    rows: &[&EndpointRow],
    _selected: usize,
    state: &AppState,
    palette: &ratatui_cheese::theme::Palette,
) -> Vec<DisplayRowData> {
    let mut result = Vec::with_capacity(rows.len() + 16);
    for (i, row) in rows.iter().enumerate() {
        let is_connected = state.connected_protocol_id.as_ref() == Some(&row.endpoint.id);
        let base_style = match (i == _selected, is_connected) {
            (true, true) => ThemeStyles::table_row_connected(palette)
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
            (false, true) => ThemeStyles::table_row_connected(palette),
            (true, false) => ThemeStyles::table_row_selected(palette),
            (false, false) if i % 2 == 1 => ThemeStyles::table_row_alt(palette),
            (false, false) => ThemeStyles::table_row_normal(palette),
        };

        let (indicator, indicator_fg) = if is_connected {
            ("●".to_string(), ThemeStyles::success(palette))
        } else {
            match state.testing_details.get(&row.active_protocol().id) {
                Some(TestType::TcpPing) => ("↔".to_string(), Style::default()),
                Some(TestType::RealPing) => ("◎".to_string(), Style::default()),
                Some(TestType::SpeedTest) => ("⇩".to_string(), Style::default()),
                Some(TestType::UdpTest) => ("↗".to_string(), Style::default()),
                None => (String::new(), Style::default()),
            }
        };

        let protocol =
            Protocol::try_from_i32(row.active_protocol().config_type).unwrap_or(Protocol::Custom);
        let is_multi = state.multi_select.contains(&row.endpoint.id);

        let idx_str = if is_multi {
            "  *".to_string()
        } else {
            format!("{:>3}", i + 1)
        };

        let type_str = format!("{protocol:.12}");
        let remarks = row.active_protocol().remarks.clone().unwrap_or_default();
        let remarks_str = truncate_pad(&remarks, 24);
        let address = row.endpoint.host.as_str();
        let address_str = truncate_pad(address, 30);
        let port_str = format!("{:>6}", row.endpoint.port);
        let delay_str = row
            .extensions
            .get(&row.active_protocol().id)
            .and_then(|e| e.delay)
            .map_or_else(|| "     -".to_string(), |d| format!("{d:>6}"));
        let speed_str = row
            .extensions
            .get(&row.active_protocol().id)
            .and_then(|e| e.speed)
            .map_or_else(|| "     -".to_string(), |s| format!("{s:>6}"));
        let ip_info_str = row
            .extensions
            .get(&row.active_protocol().id)
            .and_then(|e| e.ip_info.as_deref())
            .map_or_else(|| "     -".to_string(), |ip| truncate_pad(ip, 19));
        let traffic = row.stats.get(&row.active_protocol().id).map_or_else(
            || "        -".to_string(),
            |s| {
                let total = s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0);
                format_traffic(total as u64)
            },
        );

        let has_sub_rows = row.protocols.len() > 1;

        result.push(DisplayRowData::Endpoint {
            id: row.endpoint.id.to_string(),
            indicator,
            indicator_fg,
            idx_str,
            type_str,
            remarks_str,
            address_str,
            port_str,
            delay_str,
            speed_str,
            ip_info_str,
            traffic_str: traffic,
            has_sub_rows,
            expanded: row.expanded,
            row_style: base_style,
        });

        // If expanded, add protocol sub-rows
        if row.expanded {
            for (pi, proto) in row.protocols.iter().enumerate() {
                let is_active = pi == row.selected_protocol;
                let is_manual = row.endpoint.manual_protocol_override == Some(proto.id);
                let proto_kind = &proto.proto_kind;
                let transport = proto.transport.as_deref().unwrap_or("-");
                let security = proto.security.as_deref().unwrap_or("-");
                let proto_remarks = proto.remarks.as_deref().unwrap_or("");

                result.push(DisplayRowData::ProtocolSub {
                    id: proto.id.to_string(),
                    proto_kind: format!("  {proto_kind:.10}"),
                    transport: transport.to_string(),
                    security: security.to_string(),
                    remarks: proto_remarks.to_string(),
                    is_active,
                    is_manual,
                    row_style: if is_active {
                        base_style
                    } else {
                        Style::default()
                    },
                });
            }
        }
    }
    result
}

/// For a given endpoint index and optional sub-row, find its position in the display row list.
fn resolve_display_selected(
    display_rows: &[DisplayRowData],
    endpoint_idx: usize,
    selected_sub: Option<usize>,
) -> usize {
    let mut display_pos = 0;
    let mut ep_count = 0;
    while display_pos < display_rows.len() {
        let row = &display_rows[display_pos];
        if matches!(row, DisplayRowData::Endpoint { .. }) {
            if ep_count == endpoint_idx {
                // Found the right endpoint — if sub-row selected, advance to it
                if let Some(sub) = selected_sub {
                    // Skip the endpoint itself (1) + n sub-rows
                    display_pos += 1 + sub;
                    return display_pos.min(display_rows.len().saturating_sub(1));
                }
                return display_pos;
            }
            ep_count += 1;
        }
        display_pos += 1;
    }
    0
}

fn render_data_grid(
    frame: &mut Frame,
    area: Rect,
    display_rows: &[DisplayRowData],
    selected_display_idx: usize,
    state: &AppState,
    palette: &ratatui_cheese::theme::Palette,
) {
    let block = Block::default()
        .title(" Profiles ")
        .borders(Borders::ALL)
        .border_style(ThemeStyles::container_border(palette))
        .title_style(ThemeStyles::container_title(palette));

    // Map sort state to DataTable indices
    let sort_column = match state.sort_column {
        SortColumn::ConfigType => Some(3),
        SortColumn::Remarks => Some(4),
        SortColumn::Address => Some(6),
        SortColumn::Port => Some(7),
        SortColumn::Delay => Some(9),
        SortColumn::Speed => Some(10),
        SortColumn::Traffic => Some(12),
        SortColumn::Core => None,
    };
    let sort_direction = if state.sort_ascending {
        SortDirection::Ascending
    } else {
        SortDirection::Descending
    };

    // Build column definitions
    let mut columns = vec![
        Column::new("", ColumnWidth::Fixed(1)),      // 0 — tree marker
        Column::new("", ColumnWidth::Fixed(2)),      // 1 — indicator
        Column::new("#", ColumnWidth::Fixed(5)),     // 2 — index
        Column::new("Type", ColumnWidth::Fixed(12)), // 3 — type
        Column::new("Remarks", ColumnWidth::Fixed(24)), // 4 — remarks
        Column::new("│", ColumnWidth::Fixed(1)),     // 5 — NEW separator
    ];
    columns.extend_from_slice(&[
        Column::new("Address", ColumnWidth::Fixed(30)), // 6 — address
        Column::new("Port", ColumnWidth::Fixed(6)),     // 7 — port
        Column::new("│", ColumnWidth::Fixed(1)),        // 8 — existing separator
        Column::new("Delay", ColumnWidth::Fixed(6)),    // 9 — delay
        Column::new("Speed", ColumnWidth::Fixed(6)),    // 10 — speed
        Column::new("IP", ColumnWidth::Fixed(20)),      // 11 — ip_info
        Column::new("Traffic", ColumnWidth::Fixed(10)), // 12 — traffic
    ]);

    // Scroll offset: keep selected row roughly centered
    let inner_height = area.height.saturating_sub(3) as usize;
    let data_offset = if selected_display_idx > inner_height / 2 {
        selected_display_idx
            .saturating_sub(inner_height / 2)
            .min(display_rows.len().saturating_sub(inner_height))
    } else {
        0
    };
    let data_table = DataTable::new(columns, display_rows)
        .highlight_style(ThemeStyles::table_row_selected(palette))
        .column_spacing(0)
        .block(block)
        .sort_column(sort_column, sort_direction)
        .scrollbar(
            ThemeStyles::scrollbar_thumb(palette),
            ThemeStyles::scrollbar_track(palette),
        );

    let mut table_state = DataTableState {
        offset: data_offset,
        selected: Some(selected_display_idx),
        multi_selected: std::collections::HashSet::new(),
    };

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

fn render_footer(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    palette: &ratatui_cheese::theme::Palette,
) {
    if area.height < 1 {
        return;
    }

    let has_profile = state.selected_index < state.endpoints.len();

    let line = if has_profile {
        let row = &state.endpoints[state.selected_index];
        let core = state.resolved_core(row);
        let remarks = row
            .active_protocol()
            .remarks
            .clone()
            .unwrap_or_else(|| "-".to_string());

        let addr = if row.endpoint.host.is_empty() {
            "-"
        } else {
            &row.endpoint.host
        };
        let port = row.endpoint.port.to_string();
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

fn render_filter_strip(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    palette: &ratatui_cheese::theme::Palette,
) {
    let view_label = match state.purgatory_view {
        xray_tui_db::models::PurgatoryView::Active => "Active",
        xray_tui_db::models::PurgatoryView::Stale => "Stale",
        xray_tui_db::models::PurgatoryView::All => "All",
    };
    let view_text = format!(" View: {view_label} [P]");
    let view_span = Span::styled(view_text, ThemeStyles::container_title(palette));
    let search_text = if state.search_focused {
        format!("/ {}▉", state.search_query)
    } else if state.search_query.is_empty() {
        " /  (press / to search)".to_string()
    } else {
        format!("/ {}", state.search_query)
    };
    let search_span = Span::styled(search_text, ThemeStyles::warning(palette));

    let line = Line::from(vec![view_span, Span::raw("  "), search_span]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_confirmation_overlays(
    frame: &mut Frame,
    area: Rect,
    rows: &[&EndpointRow],
    state: &AppState,
) {
    match state.confirmation {
        Some(ConfirmAction::DeleteProfile(ref delete_id)) => {
            let profile_name = rows
                .iter()
                .find(|r| r.endpoint.id == *delete_id)
                .and_then(|r| r.active_protocol().remarks.clone())
                .unwrap_or_default();
            render_confirmation_overlay(
                frame,
                area,
                &format!(" Delete \"{profile_name}\"? (y/N) "),
            );
        }
        Some(ConfirmAction::DeleteProfiles(ref ids)) => {
            render_confirmation_overlay(
                frame,
                area,
                &format!(" Delete {} profiles? (y/N) ", ids.len()),
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
