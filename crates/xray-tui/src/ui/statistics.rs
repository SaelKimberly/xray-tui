use crate::AppState;
use crate::ui::theme::ThemeStyles;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_cheese::fieldset::{Fieldset, FieldsetStyles};
use xray_tui_core::{API_ENDPOINT, format_bytes, format_uptime};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.current_palette();
    let connected = state.connected_core.is_some();
    if !connected {
        render_placeholder(frame, area, &palette);
        return;
    }
    // `selected_index` indexes the FILTERED list, so resolve through the same
    // `filtered_profiles()` path as the profiles footer; otherwise a search
    // filter active on the Profiles tab (it persists across tab switches)
    // would show a different profile's stats here.
    let Some(profile) = state.filtered_profiles().nth(state.selected_index) else {
        render_placeholder(frame, area, &palette);
        return;
    };
    let profile_name = format!("{}:{}", profile.endpoint.host, profile.endpoint.port);
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
    if let Some(stats) = profile.stats.get(&(profile.active_protocol().id)) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::collections::HashMap;
    use std::sync::Arc;
    use toasty::Deferred;
    use xray_tui_config::AppConfig;
    use xray_tui_core::CoreType;
    use xray_tui_db::models::{Endpoint, EndpointRow, ProfileExtension, ProtocolRow, ServerStat};

    /// Minimal `EndpointRow` with one protocol (statistics render calls
    /// `active_protocol()`); host + port drive the search filter.
    fn endpoint_row(id: i64, host: &str, port: i32) -> EndpointRow {
        EndpointRow {
            endpoint: Endpoint {
                id,
                host: host.to_string(),
                host_type: "ipv4".to_string(),
                port,
                port_spec_str: None,
                parent_id: None,
                last_source: None,
                created_at: 0,
                manual_protocol_override: None,
                resolved_as: None,
                resolved_at: None,
            },
            protocols: vec![ProtocolRow {
                id: id * 100,
                endpoint_id: id,
                sig: 0,
                cred_hash: 0,
                proto_kind: String::new(),
                spec_blob: Vec::new(),
                config_type: 1,
                core_type: "xray".to_string(),
                transport: None,
                security: None,
                last_used_at: None,
                created_at: 0,
                last_seen_at: 0,
                endpoint: Deferred::from(None::<Endpoint>),
                extension: Deferred::from(None::<ProfileExtension>),
                server_stat: Deferred::from(None::<ServerStat>),
            }],
            extensions: HashMap::new(),
            stats: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        }
    }

    async fn filtered_state(search: &str) -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        state.endpoints = vec![
            endpoint_row(1, "alpha.example", 443),
            endpoint_row(2, "beta.example", 8443),
            endpoint_row(3, "gamma.example", 443),
        ];
        state.search_query = search.to_string();
        state.filter_cache_valid.set(false);
        state.selected_index = 0;
        state.connected_core = Some(CoreType::Xray);
        state
    }

    fn render_to_string(state: &AppState) -> String {
        let mut terminal = Terminal::new(TestBackend::new(120, 24)).unwrap();
        terminal
            .draw(|frame| render(frame, frame.area(), state))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[tokio::test]
    async fn render_shows_filtered_profile_not_endpoints_index() {
        // Filtered index 0 is the "beta" row (endpoints[1]); the old code
        // rendered alpha.example:443 — the wrong profile's stats.
        let state = filtered_state("beta").await;
        let rendered = render_to_string(&state);
        assert!(rendered.contains("beta.example:8443"), "got: {rendered}");
        assert!(!rendered.contains("alpha.example"), "got: {rendered}");
        assert!(!rendered.contains("gamma.example"), "got: {rendered}");
    }

    #[tokio::test]
    async fn render_placeholders_when_filter_matches_nothing() {
        // `selected_index` (0) is past the end of the FILTERED list even
        // though `endpoints.len()` is 3; the old code rendered alpha's stats.
        let state = filtered_state("nomatch").await;
        let rendered = render_to_string(&state);
        assert!(
            rendered.contains("Not connected — select a profile"),
            "expected placeholder, got: {rendered}"
        );
        assert!(!rendered.contains("alpha.example"), "got: {rendered}");
    }
}
