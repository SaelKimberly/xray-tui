use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;

use crate::SortColumn;
use crate::ui::render_confirmation_overlay;
use crate::ui::theme::ThemeStyles;
use crate::ui::widgets::data_table::{
    Column, ColumnWidth, DataTable, DataTableRow, DataTableState, SortDirection,
};
use crate::{AppState, ConfirmAction, EndpointRow, format_ts, iso_to_flag};

/// One row of the expanded per-protocol sub-table inside an endpoint panel.
struct PanelRow {
    /// "●" for the active protocol, "○" otherwise.
    marker: String,
    proto_id_hex: String,
    last_seen: String,
    last_used: String,
    config_type: String,
    delay: String,
    speed: String,
    traffic: String,
    outbound: String,
    outbound_country: String,
}

/// A single-line endpoint row; the expanded sub-table is drawn inside the
/// row's own height (`1 + panel_rows + 4`) by `render_expansion_panel`.
struct DisplayRowData {
    indicator: String,
    indicator_fg: Style,
    idx_str: String,
    type_str: String,
    country_flag: String,
    address_port_str: String,
    /// Whitelist feature flags: `🏁` DNS unresolved, `🏳️` IP/CIDR or SNI
    /// whitelisted (4 cells, one per flag).
    feat_str: String,
    config_type_str: String,
    /// `[ 12 ]`-style Test cell: `[value]` where value is 4 cells wide,
    /// or the red `[name]`/`[fast]`/`[real]` problem labels, or blank.
    test_str: String,
    test_style: Style,
    outbound_addr: String,
    outbound_country: String,
    has_sub_rows: bool,
    expanded: bool,
    row_style: Style,
    panel_selected_style: Style,
    panel_selected: Option<usize>,
    panel_ips: String,
    panel_resolve_hint: bool,
    panel_rows: Vec<PanelRow>,
}

impl DataTableRow for DisplayRowData {
    fn render(
        &self,
        col_xs: &[u16],
        col_widths: &[u16],
        buf: &mut Buffer,
        y: u16,
        clip_bottom: u16,
    ) {
        // A row that starts past the clip line has nothing visible to draw.
        if y >= clip_bottom {
            return;
        }
        let tree_marker = if self.has_sub_rows {
            if self.expanded { "▾" } else { "▶" }
        } else {
            " "
        };
        for (i, &x) in col_xs.iter().enumerate() {
            let (text, style) = match i {
                0 => (tree_marker, self.row_style),
                1 => (self.indicator.as_str(), self.indicator_fg),
                2 => (self.idx_str.as_str(), self.row_style),
                3 => (self.type_str.as_str(), self.row_style),
                4 | 13 => ("[", self.row_style),
                5 => (self.country_flag.as_str(), self.row_style),
                6 => (self.address_port_str.as_str(), self.row_style),
                7 => ("][", self.row_style),
                8 => (self.feat_str.as_str(), self.row_style),
                9 => ("]=>{", self.row_style),
                10 => (self.config_type_str.as_str(), self.row_style),
                11 => ("}=>", self.row_style),
                12 => (self.test_str.as_str(), self.test_style),
                14 => (self.outbound_addr.as_str(), self.row_style),
                15 => (self.outbound_country.as_str(), self.row_style),
                16 => ("]", self.row_style),
                _ => ("", self.row_style),
            };
            let max_w = col_widths.get(i).copied().unwrap_or(0) as usize;
            buf.set_stringn(x, y, text, max_w, style);
        }

        if self.expanded {
            let total_w: u16 = col_widths.iter().sum();
            // Panel is 2 lines shorter than the row: it starts one line below
            // the endpoint and leaves one blank line (gap) after its bottom
            // border so it never touches the next row.
            self.render_expansion_panel(
                buf,
                col_xs[0],
                y + 1,
                self.height(0) - 2,
                total_w,
                self.row_style,
                clip_bottom,
            );
        }
    }

    fn height(&self, _available_width: u16) -> u16 {
        if self.expanded {
            // 1 endpoint line + panel (top border + IPs + separator + sub
            // rows + bottom border = rows + 4) + 1 gap line after the panel.
            1 + self.panel_rows.len() as u16 + 4 + 1
        } else {
            1
        }
    }
}

impl DisplayRowData {
    /// Rounded panel under the endpoint line: IPs line, separator, sub-table.
    /// `panel_w` is the table's actual rendered width (viewport-capped) so the
    /// panel never exceeds the buffer. The panel is clipped to `clip_bottom`:
    /// an expanded row taller than the viewport must not write past it.
    fn render_expansion_panel(
        &self,
        buf: &mut Buffer,
        x0: u16,
        y0: u16,
        panel_height: u16,
        panel_w: u16,
        row_style: Style,
        clip_bottom: u16,
    ) {
        let visible_h = clip_bottom.saturating_sub(y0);
        if visible_h == 0 {
            return;
        }
        let panel_height = panel_height.min(visible_h);
        let rect = Rect {
            x: x0,
            y: y0,
            width: panel_w,
            height: panel_height,
        };
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .render(rect, buf);

        let inner_x = x0 + 1;
        let inner_w = (panel_w - 2) as usize;

        // IPs line (y0+1 is inside the top border)
        if y0 + 1 < clip_bottom {
            let ips_text = format!(" IPs: {}", self.panel_ips);
            buf.set_stringn(inner_x, y0 + 1, &ips_text, inner_w, row_style);
            if self.panel_resolve_hint {
                let hint = "(x resolve)";
                let hx = inner_x + (inner_w.saturating_sub(hint.len())) as u16;
                buf.set_stringn(hx, y0 + 1, hint, hint.len(), row_style);
            }
        }

        // Separator
        if y0 + 2 < clip_bottom {
            let sep_x = inner_x;
            let sep_y = y0 + 2;
            let sep_line = "─".repeat(inner_w);
            buf.set_stringn(sep_x, sep_y, &sep_line, inner_w, Style::default());
        }

        // Sub-table rows
        let cols: [(usize, usize); 10] = [
            (0, 3),   // marker
            (3, 10),  // id
            (13, 20), // last_seen
            (33, 20), // last_used
            (53, 12), // config
            (65, 8),  // delay
            (73, 8),  // speed
            (81, 11), // traffic
            (92, 16), // outbound
            (108, 7), // country
        ];
        for (n, pr) in self.panel_rows.iter().enumerate() {
            let y = y0 + 3 + n as u16;
            if y >= clip_bottom {
                break;
            }
            let style = if Some(n) == self.panel_selected {
                self.panel_selected_style
            } else {
                row_style
            };
            if Some(n) == self.panel_selected {
                // Reverse highlight across the ENTIRE sub-row width: replace
                // the panel's highlight background (painted by the DataTable
                // over the selected endpoint row) with the common background,
                // not just over written glyphs — column gaps would keep the
                // highlight otherwise.
                for x in inner_x..(inner_x + inner_w as u16) {
                    buf[(x, y)].set_style(style);
                }
            }
            let mut x = inner_x;
            let cell_texts = [
                pr.marker.as_str(),
                pr.proto_id_hex.as_str(),
                pr.last_seen.as_str(),
                pr.last_used.as_str(),
                pr.config_type.as_str(),
                pr.delay.as_str(),
                pr.speed.as_str(),
                pr.traffic.as_str(),
                pr.outbound.as_str(),
                pr.outbound_country.as_str(),
            ];
            for ((_, w), text) in cols.iter().zip(cell_texts.iter()) {
                buf.set_stringn(x, y, text, *w, style);
                x += *w as u16;
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
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[1]);
        render_footer(frame, chunks[2], state, &palette);
        return;
    }

    // One display row per endpoint; the expanded panel lives inside the row.
    let display_rows = build_display_rows(&rows, selected, state, &palette);

    render_data_grid(frame, chunks[1], &display_rows, selected, state, &palette);
    render_footer(frame, chunks[2], state, &palette);
    render_confirmation_overlays(frame, area, &rows, state);
}

const fn test_glyph(test_type: TestType) -> &'static str {
    match test_type {
        TestType::TcpPing => "↔",
        TestType::RealPing => "◎",
        TestType::SpeedTest => "⇩",
        TestType::UdpTest => "↗",
    }
}

/// Latency thresholds for the Test column: green below the warning threshold,
/// yellow at/above it, red at/above the error threshold.
const TEST_WARN_MS: i32 = 500;
const TEST_BAD_MS: i32 = 1000;

/// Compute the Test cell for one endpoint row: `[value]` with the active
/// link's last measured delay, colored by magnitude, or the red problem
/// labels — `[name]` when the DNS name could not be resolved, `[real]`/`[fast]`
/// when any of the endpoint's links carries a persisted failure marker of
/// that class (`link.error.kind`, round maps removed in T17). Precedence
/// follows the tier model (decision 16): real-err (3) ranks above fast-err
/// (4) — when both marker classes are present the deeper real check wins —
/// and DNS-unresolved (5) is the deepest, so `[name]` beats both.
fn compute_test_cell(
    row: &EndpointRow,
    resolved: bool,
    palette: &ratatui_cheese::theme::Palette,
) -> (String, Style) {
    use xray_tui_db::models::ProfileErr;
    let mut fast_failed = false;
    let mut real_failed = false;
    for link in &row.links {
        match link.error.as_ref().map(|e| e.kind) {
            Some(ProfileErr::Fast) => fast_failed = true,
            Some(ProfileErr::Real | ProfileErr::Name) => real_failed = true,
            None => {}
        }
    }
    let active_delay = row.active_link().and_then(|l| match l.latency {
        Some(xray_tui_db::models::Latency::Real { delay, .. })
        | Some(xray_tui_db::models::Latency::Fast { delay }) => Some(delay),
        None => None,
    });
    test_cell_content(
        row.endpoint.host_type == xray_tui_db::models::HostType::Dns,
        resolved,
        fast_failed,
        real_failed,
        active_delay,
        palette,
    )
}

/// Pure Test-cell logic (no `AppState`) so the label precedence and color
/// thresholds are unit-testable.
fn test_cell_content(
    host_is_dns: bool,
    resolved: bool,
    fast_failed: bool,
    real_failed: bool,
    active_delay: Option<i32>,
    palette: &ratatui_cheese::theme::Palette,
) -> (String, Style) {
    let bad = ThemeStyles::test_delay_bad(palette);
    if host_is_dns && !resolved {
        return (format!("[{}]", center_cell("name", 4)), bad);
    }
    // Tier ordering (decision 16): real-err ranks above fast-err — the real
    // check is the deeper probe, so when both classes failed `[real]` wins.
    if real_failed {
        return (format!("[{}]", center_cell("real", 4)), bad);
    }
    if fast_failed {
        return (format!("[{}]", center_cell("fast", 4)), bad);
    }
    match active_delay {
        Some(d) if d >= 0 => {
            let style = if d >= TEST_BAD_MS {
                bad
            } else if d >= TEST_WARN_MS {
                ThemeStyles::test_delay_warn(palette)
            } else {
                ThemeStyles::test_delay_ok(palette)
            };
            (format!("[{}]", center_cell(&d.to_string(), 4)), style)
        }
        _ => (" ".repeat(6), Style::default()),
    }
}

fn build_display_rows(
    rows: &[&EndpointRow],
    selected: usize,
    state: &AppState,
    palette: &ratatui_cheese::theme::Palette,
) -> Vec<DisplayRowData> {
    let mut result = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let is_connected = state.connected_protocol_id.as_ref() == Some(&row.endpoint.id.get());
        let base_style = match (i == selected, is_connected) {
            (true, true) => ThemeStyles::table_row_connected(palette)
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
            (false, true) => ThemeStyles::table_row_connected(palette),
            (true, false) => ThemeStyles::table_row_selected(palette),
            (false, false) => ThemeStyles::table_row_normal(palette),
        };

        // Indicator: connected → ●; else active protocol's test glyph; else the
        // first protocol under test.
        let (indicator, indicator_fg) = if is_connected {
            ("●".to_string(), ThemeStyles::success(palette))
        } else {
            let active_id = row.active_link().map(|l| l.protocol_id.get());
            let glyph = active_id
                .and_then(|id| state.testing_details.get(&id).copied().map(test_glyph))
                .or_else(|| {
                    row.links.iter().find_map(|l| {
                        state
                            .testing_details
                            .get(&l.protocol_id.get())
                            .copied()
                            .map(test_glyph)
                    })
                });
            glyph.map_or_else(
                || (String::new(), Style::default()),
                |g| (g.to_string(), Style::default()),
            )
        };

        let protocol = row
            .active_protocol()
            .map_or(Protocol::Custom, |(_, p)| Protocol::from(p.proto_kind));
        let is_multi = state.multi_select.contains(&row.endpoint.id.get());

        let idx_str = if is_multi {
            "  *".to_string()
        } else {
            format!("{:>3}", i + 1)
        };

        let info = state.endpoint_info.get(&row.endpoint.id.get());
        let resolved = info.is_some_and(|i| !i.resolved_ips.is_empty());

        let type_str = format!("{protocol:.12}");
        let country_flag = info
            .and_then(|i| i.country.as_deref())
            .map_or_else(|| "\u{1F3F4}".to_string(), iso_to_flag);
        let address_port_str =
            truncate_pad(&format!(" {}:{}", row.endpoint.host, row.endpoint.port), 36);
        // Feature flags, one 2-cell slot each: IP (🏁 DNS unresolved, 🏳️
        // IP/CIDR whitelisted) then SNI (🏳️ whitelisted).
        let ip_feature =
            if row.endpoint.host_type == xray_tui_db::models::HostType::Dns && !resolved {
                "\u{1F3C1}".to_string()
            } else if info
                .is_some_and(|i| i.host_features.ip_whitelisted || i.host_features.cidr_whitelisted)
            {
                "\u{1F3F3}\u{FE0F}".to_string()
            } else {
                String::new()
            };
        let sni_feature = if info.and_then(|i| i.sni_whitelisted).unwrap_or(false) {
            "\u{1F3F3}\u{FE0F}".to_string()
        } else {
            String::new()
        };
        let feat_str = format!(
            "{}{}",
            truncate_pad(&ip_feature, 2),
            truncate_pad(&sni_feature, 2)
        );
        let (test_str, test_style) = compute_test_cell(row, resolved, palette);

        let (t, s) = row.active_protocol().map_or((None, None), |(_, p)| {
            (
                Some(p.transport.r#type.as_str()),
                Some(p.security.r#type.as_str()),
            )
        });
        let config_type = match (t, s) {
            (None, None) => "-".to_string(),
            (t, s) => format!("{}/{}", t.unwrap_or("-"), s.unwrap_or("-")),
        };
        let config_type_str = center_pad(&config_type, 12);

        let outbound_addr = info
            .and_then(|i| i.outbound_ip.map(|ip| ip.to_string()))
            .unwrap_or_else(|| "—".to_string());
        let outbound_country = info
            .and_then(|i| i.outbound_country.as_deref())
            .map_or_else(
                || "—".to_string(),
                |iso| truncate_pad(&format!("{} {iso}", iso_to_flag(iso)), 7),
            );

        // Panel content
        let panel_ips = info
            .map(|i| {
                i.resolved_ips
                    .iter()
                    .map(|ip| format!("[{ip}]"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let (panel_ips, panel_resolve_hint) = if panel_ips.is_empty() {
            if row.endpoint.host_type == xray_tui_db::models::HostType::Dns {
                ("[?]".to_string(), true)
            } else {
                (panel_ips, false)
            }
        } else {
            (panel_ips, false)
        };

        let panel_rows: Vec<PanelRow> = if row.expanded {
            let active_id = row.active_link().map(|l| l.protocol_id);
            row.links
                .iter()
                .map(|link| {
                    let proto = row.protocols.get(&link.protocol_id);
                    let delay = match link.latency {
                        Some(xray_tui_db::models::Latency::Real { delay, .. })
                        | Some(xray_tui_db::models::Latency::Fast { delay }) => {
                            format!("{delay}ms")
                        }
                        None => "-".to_string(),
                    };
                    let speed = link
                        .speed_bps
                        .map_or_else(|| "-".to_string(), |s| format_traffic(s as u64));
                    let traffic = {
                        let total = link.traffic.total_down + link.traffic.total_up;
                        if total == 0 {
                            "-".to_string()
                        } else {
                            format_traffic(total as u64)
                        }
                    };
                    let (outbound, outbound_country) = link
                        .latency
                        .as_ref()
                        .and_then(|l| match l {
                            // Latency::Real.ip is the bare IP prefix now
                            // (events.rs strips any "|"-joined suffix) — no
                            // split here; country needs parsed fields, which
                            // T20 refines.
                            xray_tui_db::models::Latency::Real { ip, .. } => ip.clone(),
                            xray_tui_db::models::Latency::Fast { .. } => None,
                        })
                        .map_or_else(
                            || ("—".to_string(), "—".to_string()),
                            |ip| (ip, "—".to_string()),
                        );
                    let (t, s) = proto.map_or((None, None), |p| {
                        (
                            Some(p.transport.r#type.as_str()),
                            Some(p.security.r#type.as_str()),
                        )
                    });
                    let config_type = match (t, s) {
                        (None, None) => "-".to_string(),
                        (t, s) => format!("{}/{}", t.unwrap_or("-"), s.unwrap_or("-")),
                    };
                    PanelRow {
                        marker: if Some(link.protocol_id) == active_id {
                            "●".to_string()
                        } else {
                            "○".to_string()
                        },
                        proto_id_hex: format!("{:08x}", link.protocol_id.get() as u32),
                        last_seen: format_ts(link.last_seen_at.as_second()),
                        last_used: link
                            .last_used_at
                            .map_or_else(|| "—".to_string(), |ts| format_ts(ts.as_second())),
                        config_type,
                        delay,
                        speed,
                        traffic,
                        outbound,
                        outbound_country,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        result.push(DisplayRowData {
            indicator,
            indicator_fg,
            idx_str,
            type_str,
            country_flag,
            address_port_str,
            feat_str,
            config_type_str,
            test_str,
            test_style,
            outbound_addr,
            outbound_country,
            has_sub_rows: row.links.len() > 1,
            expanded: row.expanded,
            row_style: base_style,
            panel_selected_style: ThemeStyles::panel_row_selected(palette),
            panel_selected: if i == selected {
                state.selected_sub
            } else {
                None
            },
            panel_ips,
            panel_resolve_hint,
            panel_rows,
        });
    }
    result
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

    // Map sort state to DataTable column indices
    let sort_column = match state.sort_column {
        SortColumn::ConfigType => Some(3),
        SortColumn::Address | SortColumn::Port => Some(6),
        SortColumn::Test => Some(12),
        SortColumn::LastSeen | SortColumn::Speed | SortColumn::Traffic | SortColumn::Core => None,
    };
    let sort_direction = if state.sort_ascending {
        SortDirection::Ascending
    } else {
        SortDirection::Descending
    };

    // 17 fixed columns; headers carry only descriptive names (decorative
    // separator cells have empty headers).
    let columns = vec![
        Column::new("", ColumnWidth::Fixed(1)),      // 0 — tree marker
        Column::new("", ColumnWidth::Fixed(2)),      // 1 — indicator
        Column::new("#", ColumnWidth::Fixed(5)),     // 2 — index
        Column::new("Type", ColumnWidth::Fixed(12)), // 3
        Column::new("", ColumnWidth::Fixed(1)),      // 4 — [
        Column::new("", ColumnWidth::Fixed(4)),      // 5 — country flag
        Column::new("Address", ColumnWidth::Fixed(36)), // 6
        Column::new("", ColumnWidth::Fixed(2)),      // 7 — ][
        Column::new("Feat", ColumnWidth::Fixed(4)),  // 8 — IP+SNI flags
        Column::new("", ColumnWidth::Fixed(4)),      // 9 — ]=>{
        Column::new("", ColumnWidth::Fixed(12)),     // 10 — config type
        Column::new("", ColumnWidth::Fixed(3)),      // 11 — }=> arrow
        Column::new("Test", ColumnWidth::Fixed(6)),  // 12 — [delay]/[name]/[fast]/[real]
        Column::new("", ColumnWidth::Fixed(1)),      // 13 — [ outbound opener
        Column::new("Outbound", ColumnWidth::Fixed(16)), // 14
        Column::new("Country", ColumnWidth::Fixed(7)), // 15
        Column::new("", ColumnWidth::Fixed(1)),      // 16 — ]
    ];

    // Scroll offset: keep the selected row roughly centered, in line units —
    // expanded rows are taller than 1, so row-index math would strand the
    // last rows below the viewport.
    let heights: Vec<u16> = display_rows
        .iter()
        .map(|r| r.height(area.width.saturating_sub(2)))
        .collect();
    let data_offset = compute_scroll_offset(&heights, selected_display_idx, area.height);
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

/// First visible row index that keeps the selected row roughly centered, in
/// line units. `heights[i]` is row i's height (expanded rows are taller than
/// 1). Clamped so the last rows still fit the viewport (same math as
/// `DataTable`'s own offset clamp).
fn compute_scroll_offset(heights: &[u16], selected: usize, viewport_height: u16) -> usize {
    let inner_height = viewport_height.saturating_sub(3) as usize;
    if heights.is_empty() {
        return 0;
    }
    let sel_h = heights.get(selected).copied().unwrap_or(1) as usize;
    // Rows above the selection, in lines.
    let above: usize = heights[..selected].iter().map(|h| *h as usize).sum();
    // Centering offset: put the selected row's start at mid-viewport.
    let target_sel_start = (inner_height.saturating_sub(sel_h)) / 2;
    let ideal = above.saturating_sub(target_sel_start);
    // Height-aware clamp: earliest offset whose rows still fill the viewport.
    let mut rows_from_end = 0usize;
    let mut h_sum = 0u16;
    for h in heights.iter().rev() {
        if h_sum + h > inner_height as u16 {
            break;
        }
        h_sum += h;
        rows_from_end += 1;
    }
    let mut max_offset = heights.len().saturating_sub(rows_from_end);
    // A row taller than the viewport leaves nothing to fill it, so
    // `rows_from_end` is 0 and the raw max_offset would equal `len` — the
    // table would render nothing. Clamp so the last row is at least
    // partially (clipped) visible.
    max_offset = max_offset.min(heights.len().saturating_sub(1));
    // Minimum offset that still shows the selection's last line: when content
    // below the selection is taller than the viewport, jump toward the end.
    let o_min = above.saturating_add(sel_h).saturating_sub(inner_height);
    ideal.max(o_min.min(max_offset)).min(max_offset)
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

/// Truncate to `width` (unicode-aware) with space padding, no ellipsis.
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

/// Center `s` inside a `width`-wide cell with the extra space on the LEFT for
/// odd widths: `[ 12 ]`, `[ 123]`, `[1234]` (Test column look). Truncates
/// when wider.
fn center_cell(s: &str, width: usize) -> String {
    let w = unicode_width::UnicodeWidthStr::width(s);
    if w >= width {
        return truncate_pad(s, width);
    }
    let left = (width - w).div_ceil(2);
    let right = width - w - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// Center `s` inside a `width`-wide cell (unicode-aware); truncates when wider.
fn center_pad(s: &str, width: usize) -> String {
    let w = unicode_width::UnicodeWidthStr::width(s);
    if w >= width {
        return truncate_pad(s, width);
    }
    let left = (width - w) / 2;
    let right = width - w - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// The row the footer describes: the FILTERED row at `selected_index`.
/// With a search filter active, `selected_index` indexes the filtered list,
/// not `state.endpoints` — resolving through `filtered_profiles()` keeps the
/// footer in sync with the highlighted row (and the none-selected branch).
pub(crate) fn footer_row(state: &AppState) -> Option<&EndpointRow> {
    state.filtered_profiles().nth(state.selected_index)
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

    let line = footer_row(state).map_or_else(
        || {
            Line::from(Span::styled(
                " Server: (none selected)",
                ThemeStyles::footer_label(palette),
            ))
        },
        |row| {
            let core = state.resolved_core(row);

            let addr = if row.endpoint.host.is_empty() {
                "-"
            } else {
                &row.endpoint.host
            };
            let port = row.endpoint.port.to_string();
            Line::from(vec![
                Span::styled(" Server: ", ThemeStyles::footer_label(palette)),
                Span::styled(
                    format!("{addr}:{port}  "),
                    ThemeStyles::footer_value(palette),
                ),
                Span::styled(format!("[{core}] "), ThemeStyles::footer_value(palette)),
            ])
        },
    );
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
                .find(|r| r.endpoint.id.get() == *delete_id)
                .map(|r| format!("{}:{}", r.endpoint.host, r.endpoint.port))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel_row(marker: &str) -> PanelRow {
        PanelRow {
            marker: marker.to_string(),
            proto_id_hex: String::new(),
            last_seen: String::new(),
            last_used: String::new(),
            config_type: String::new(),
            delay: String::new(),
            speed: String::new(),
            traffic: String::new(),
            outbound: String::new(),
            outbound_country: String::new(),
        }
    }

    fn sample_row(expanded: bool, panel_rows: Vec<PanelRow>, idx: &str) -> DisplayRowData {
        DisplayRowData {
            indicator: String::new(),
            indicator_fg: Style::default(),
            idx_str: idx.to_string(),
            type_str: String::new(),
            country_flag: String::new(),
            address_port_str: String::new(),
            feat_str: String::new(),
            config_type_str: String::new(),
            test_str: String::new(),
            test_style: Style::default(),
            outbound_addr: String::new(),
            outbound_country: String::new(),
            has_sub_rows: !panel_rows.is_empty(),
            expanded,
            row_style: Style::default(),
            panel_selected_style: Style::default(),
            panel_selected: None,
            panel_ips: String::new(),
            panel_resolve_hint: false,
            panel_rows,
        }
    }

    #[test]
    fn expanded_row_height_includes_gap() {
        let row = sample_row(
            true,
            vec![sample_panel_row("●"), sample_panel_row("○")],
            "00",
        );
        // 1 endpoint + panel (2 rows + 4 border/IPs/sep lines) + 1 gap
        assert_eq!(row.height(0), 8);
        let collapsed = sample_row(false, vec![], "11");
        assert_eq!(collapsed.height(0), 1);
    }

    #[test]
    fn selected_sub_row_reverse_highlights_full_width() {
        let palette =
            crate::ui::palette_bridge::palette_from_name(&ratatui_themes::ThemeName::TokyoNight);
        let mut row = sample_row(
            true,
            vec![sample_panel_row("●"), sample_panel_row("○")],
            "00",
        );
        row.row_style = ThemeStyles::table_row_normal(&palette);
        row.panel_selected_style = ThemeStyles::panel_row_selected(&palette);
        row.panel_selected = Some(0);
        let col_xs: Vec<u16> = (0..17).collect();
        let col_widths = vec![1u16; 17];
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));

        // Simulate the DataTable's endpoint-row highlight: the whole row
        // (including the expanded panel area) sits on `surface`.
        for x in 0..17u16 {
            for y in 0..8u16 {
                buf[(x, y)].set_style(ThemeStyles::table_row_selected(&palette));
            }
        }

        row.render(&col_xs, &col_widths, &mut buf, 0, 20);

        // Panel starts at y=1; sub-rows at y0+3 → sub-row 0 at y=4, sub-row 1
        // at y=5. x=1 is the panel's first inner cell.
        // Selected sub-row drops back to the common (terminal-default)
        // background…
        assert_eq!(
            buf[(1, 4)].style().bg,
            Some(ratatui::style::Color::Reset),
            "selected sub-row must use the common background (reverse highlight)"
        );
        // …across the ENTIRE row width, not just written glyphs.
        assert_eq!(
            buf[(13, 4)].style().bg,
            Some(ratatui::style::Color::Reset),
            "reverse highlight must span the full sub-row width"
        );
        // …the unselected sub-row keeps the panel's highlight background.
        assert_eq!(
            buf[(1, 5)].style().bg,
            Some(palette.surface),
            "unselected sub-row keeps the endpoint highlight background"
        );
    }

    #[test]
    fn panel_bottom_border_does_not_touch_next_row() {
        let row0 = sample_row(
            true,
            vec![sample_panel_row("●"), sample_panel_row("○")],
            "00",
        );
        let row1 = sample_row(false, vec![], "11");
        let col_xs: Vec<u16> = (0..17).collect();
        let col_widths = vec![1u16; 17];
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));

        row0.render(&col_xs, &col_widths, &mut buf, 0, 20);
        row1.render(&col_xs, &col_widths, &mut buf, 8, 20);

        // Panel (height 6) sits at y=1..6: bottom-left corner at y=6.
        assert_eq!(buf[(0, 6)].symbol(), "\u{2570}"); // ╰
        // y=7 is the blank gap line — the next row must not start there.
        assert_eq!(buf[(0, 7)].symbol(), " ");
        // Next row's content starts at y=8, untouched by the panel.
        assert_eq!(buf[(2, 8)].symbol(), "1");
    }

    #[test]
    fn scroll_offset_reaches_last_row_when_tail_expanded() {
        // 30 collapsed + 10 expanded (6 lines), 40-line viewport (37 inner).
        let mut heights = vec![1u16; 40];
        for h in heights.iter_mut().skip(30) {
            *h = 6;
        }
        let offset = compute_scroll_offset(&heights, 39, 40);
        // Rows from the offset must fit the viewport…
        let fit: u16 = heights[offset..].iter().sum();
        assert!(fit <= 37);
        // …and the selected (last) row must be inside it.
        let sel_start: u16 = heights[offset..39].iter().sum();
        assert!(sel_start + 6 <= 37);
    }

    #[test]
    fn scroll_offset_small_list_stays_top() {
        let heights = vec![1u16; 5];
        assert_eq!(compute_scroll_offset(&heights, 4, 40), 0);
    }

    #[test]
    fn scroll_offset_centers_mid_list() {
        // 100 one-line rows, 21-line viewport (18 inner) → row 50 centered
        // at line 8 → offset 42.
        let heights = vec![1u16; 100];
        assert_eq!(compute_scroll_offset(&heights, 50, 21), 42);
    }

    #[test]
    fn scroll_offset_does_not_push_selection_offscreen() {
        // Selection in the middle of a tall list must stay visible.
        let heights = vec![1u16; 40];
        let offset = compute_scroll_offset(&heights, 20, 21);
        assert!(offset <= 20);
        let sel_start: u16 = heights[offset..20].iter().sum();
        assert!(sel_start < 18);
    }

    #[test]
    fn rows_taller_than_viewport_do_not_blank_table() {
        // heights where the selected (tall) row exceeds the viewport:
        // old code produced offset == len → nothing rendered.
        let heights = vec![1u16, 1, 1, 1, 1, 10];
        let offset = compute_scroll_offset(&heights, 5, 8);
        assert!(
            offset < heights.len(),
            "offset must stay inside the row list"
        );
    }

    #[test]
    fn scroll_offset_keeps_tall_last_row_visible() {
        // The tall selected row's start line must sit inside the viewport,
        // not be pushed past the end of the row list.
        let heights = vec![1u16, 1, 1, 1, 1, 10];
        let offset = compute_scroll_offset(&heights, 5, 8);
        assert!(offset < heights.len());
        let sel_start: u16 = heights[offset..5].iter().sum();
        assert!(sel_start <= 5);
    }

    #[test]
    fn scroll_offset_tall_selected_row_mid_list_stays_inside() {
        // Expanded row in the middle of the list: the offset must never
        // escape the row list even though the row is taller than the viewport.
        let heights = vec![1u16, 1, 10, 1, 1];
        let offset = compute_scroll_offset(&heights, 2, 8);
        assert!(offset < heights.len());
    }

    /// Minimal `EndpointRow` with just enough to be filtered and described:
    /// host + port drive the search filter; no links (linkless endpoints must
    /// render without sub-rows and without an active protocol).
    fn endpoint_row(id: i64, host: &str, port: i32) -> EndpointRow {
        use xray_tui_db::models::Endpoint;
        EndpointRow {
            endpoint: Endpoint {
                id: xray_tui_db::models::EndpointId::new(id),
                host: host.to_string(),
                host_type: xray_tui_db::models::HostType::Ipv4,
                port: port as u16,
                ports: Vec::new(),
                parent_id: None,
                last_source: None,
                manual_protocol_override: None,
                resolved_as: Vec::new(),
                resolved_at: None,
                created_at: crate::ops::profiles::test_support::ts(0),
                links: toasty::Deferred::default(),
                group_links: toasty::Deferred::default(),
            },
            links: Vec::new(),
            protocols: std::collections::HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        }
    }

    #[tokio::test]
    async fn footer_row_resolves_filtered_row_not_endpoints() {
        // `selected_index` indexes the FILTERED list. With a search filter
        // active, the footer must describe the filtered row — NOT
        // `endpoints[selected_index]`, which would show a different server
        // than the highlighted row (and wrongly report "none selected").
        let dir = tempfile::tempdir().unwrap();
        let db = std::sync::Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, xray_tui_config::AppConfig::default()).await;
        state.endpoints = vec![
            endpoint_row(1, "alpha.example", 443),
            endpoint_row(2, "beta.example", 8443),
            endpoint_row(3, "gamma.example", 443),
        ];
        state.search_query = "beta".to_string();
        state.filter_cache_valid.set(false);
        state.selected_index = 0;

        // Only "beta.example" survives the filter, so filtered index 0 is the
        // row at endpoints[1]; the old code showed alpha.example:443 instead.
        let row = footer_row(&state).expect("filtered row should exist");
        assert_eq!(row.endpoint.id.get(), 2);
        assert_eq!(row.endpoint.host, "beta.example");

        // Selection past the filtered end → none selected, even though
        // `endpoints.len()` (3) exceeds `selected_index` (1).
        state.selected_index = 1;
        assert!(footer_row(&state).is_none());
    }

    fn test_palette() -> ratatui_cheese::theme::Palette {
        crate::ui::palette_bridge::palette_from_name(&ratatui_themes::ThemeName::TokyoNight)
    }

    #[test]
    fn test_cell_colors_delay_by_threshold() {
        let palette = test_palette();
        let (t, s) = test_cell_content(false, true, false, false, Some(12), &palette);
        assert_eq!(t, "[ 12 ]");
        assert_eq!(s.fg, Some(palette.success));
        let (t, s) = test_cell_content(false, true, false, false, Some(612), &palette);
        assert_eq!(t, "[ 612]");
        assert_eq!(s.fg, Some(ratatui::style::Color::Yellow));
        let (t, s) = test_cell_content(false, true, false, false, Some(1234), &palette);
        assert_eq!(t, "[1234]");
        assert_eq!(s.fg, Some(palette.error));
    }

    #[test]
    fn test_cell_blank_without_measurement() {
        let palette = test_palette();
        let (t, _) = test_cell_content(false, true, false, false, None, &palette);
        assert_eq!(t, "      ");
    }

    #[test]
    fn test_cell_shows_name_when_dns_unresolved() {
        let palette = test_palette();
        let (t, s) = test_cell_content(true, false, false, false, Some(12), &palette);
        assert_eq!(t, "[name]");
        assert_eq!(s.fg, Some(palette.error));
        // Resolved DNS name behaves like a normal host.
        let (t, _) = test_cell_content(true, true, false, false, Some(12), &palette);
        assert_eq!(t, "[ 12 ]");
    }

    #[test]
    fn test_cell_labels_persisted_failure_markers() {
        let palette = test_palette();
        // Both marker classes present → [real]: real-err (tier 3) ranks above
        // fast-err (tier 4), the real check being the deeper probe (T20 flip).
        let (t, s) = test_cell_content(false, true, true, true, Some(12), &palette);
        assert_eq!(t, "[real]");
        assert_eq!(s.fg, Some(palette.error));
        // Only a real-class failure marker → [real].
        let (t, _) = test_cell_content(false, true, false, true, Some(12), &palette);
        assert_eq!(t, "[real]");
        // Only a fast-class failure marker → [fast].
        let (t, _) = test_cell_content(false, true, true, false, Some(12), &palette);
        assert_eq!(t, "[fast]");
        // No failure markers → the delay shows even when untested links exist.
        let (t, _) = test_cell_content(false, true, false, false, Some(30), &palette);
        assert_eq!(t, "[ 30 ]");
    }

    #[test]
    fn test_cell_label_precedence_matrix() {
        let palette = test_palette();
        // (a) only real markers → [real]
        let (t, _) = test_cell_content(false, true, false, true, None, &palette);
        assert_eq!(t, "[real]");
        // (b) only fast markers → [fast]
        let (t, _) = test_cell_content(false, true, true, false, None, &palette);
        assert_eq!(t, "[fast]");
        // (c) both real and fast markers → [real] (tier-consistent)
        let (t, _) = test_cell_content(false, true, true, true, None, &palette);
        assert_eq!(t, "[real]");
        // (d) DNS-unresolved + fast marker → [name] (DNS tier 5 is deepest)
        let (t, _) = test_cell_content(true, false, true, false, None, &palette);
        assert_eq!(t, "[name]");
        // (e) no markers, no measurement → blank
        let (t, _) = test_cell_content(false, true, false, false, None, &palette);
        assert_eq!(t, "      ");
    }
}
