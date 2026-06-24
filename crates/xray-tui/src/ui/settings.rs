use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::theme::Theme;
use crate::{AppMode, AppState, SettingsMode, SettingsSection};
use xray_tui_core::CoreType;
// ── Menu items ──────────────────────────────────────────────────────────

const MENU_ITEMS: &[(&str, &str)] = &[
    (
        "Core Settings",
        "binary paths, log level, default core type",
    ),
    ("Protocol Core", "per-protocol core type override"),
    ("GUI Settings", "language, theme, refresh interval"),
    ("Inbound Settings", "ports, listen address, sniffing"),
    ("Routing Rules", "add/edit/delete/reorder rules"),
    ("DNS Settings", "servers, hosts, query strategy, cache"),
    ("System Proxy", "enable/disable HTTP_PROXY, ports, bypass"),
    ("TUN Mode", "enabled, interface name, MTU"),
    ("Mux", "multiplexing settings"),
    ("Statistics", "enable/disable stats collection"),
    ("Updates", "check and install backend updates"),
    ("Speed Test", "ping URL, IP API, timeouts, batch settings"),
    ("Logging", "log retention, batch size"),
];

/// Separator position between active sections and deferred ones
const SEPARATOR_AFTER: usize = 4; // after DNS Settings

// ── Public entry points ─────────────────────────────────────────────────

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    if let AppMode::Settings { mode } = &state.mode {
        match mode {
            SettingsMode::Menu { .. } => render_menu(frame, area, state),
            SettingsMode::CoreForm { .. }
            | SettingsMode::GuiForm { .. }
            | SettingsMode::InboundForm { .. }
            | SettingsMode::SystemProxyForm { .. }
            | SettingsMode::TunForm { .. }
            | SettingsMode::MuxForm { .. }
            | SettingsMode::StatsForm { .. }
            | SettingsMode::DnsForm { .. }
            | SettingsMode::ProtocolCoreForm { .. }
            | SettingsMode::LoggingForm { .. }
            | SettingsMode::SpeedTestForm { .. } => render_form(frame, area, state),
            SettingsMode::RoutingList { .. } => render_routing_list(frame, area, state),
            SettingsMode::RoutingForm { .. } => render_routing_form(frame, area, state),
            SettingsMode::UpdateForm { .. } => render_update_form(frame, area, state),
        }
    }
}
pub async fn handle_key(state: &mut AppState, key: &KeyEvent) {
    let mode = match &state.mode {
        AppMode::Settings { mode } => mode.clone(),
        _ => return,
    };
    match mode {
        SettingsMode::Menu { .. } => handle_menu_key(state, key).await,
        SettingsMode::CoreForm { .. }
        | SettingsMode::GuiForm { .. }
        | SettingsMode::InboundForm { .. }
        | SettingsMode::SystemProxyForm { .. }
        | SettingsMode::TunForm { .. }
        | SettingsMode::MuxForm { .. }
        | SettingsMode::StatsForm { .. }
        | SettingsMode::DnsForm { .. }
        | SettingsMode::ProtocolCoreForm { .. }
        | SettingsMode::LoggingForm { .. }
        | SettingsMode::SpeedTestForm { .. } => handle_form_key(state, key),
        SettingsMode::RoutingList { .. } => handle_routing_list_key(state, key).await,
        SettingsMode::RoutingForm { .. } => handle_routing_form_key(state, key).await,
        SettingsMode::UpdateForm { .. } => handle_update_form_key(state, key),
    }
}

// ── Menu ────────────────────────────────────────────────────────────────

fn render_menu(frame: &mut Frame, area: Rect, state: &AppState) {
    let selected = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::Menu { selected },
        } => *selected,
        _ => 0,
    };

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (name, desc)) in MENU_ITEMS.iter().enumerate() {
        let is_selected = i == selected;

        if i == SEPARATOR_AFTER + 1 {
            lines.push(Line::from(Span::raw(" ─────")));
        }

        let prefix = if is_selected { "► " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        } else if i > SEPARATOR_AFTER {
            Theme::HINT
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(format!("{prefix}{name}"), style)));
        if is_selected {
            lines.push(Line::from(Span::styled(format!("    {desc}"), Theme::HINT)));
        }
    }

    lines.push(Line::from(""));
    let help = " [↑/↓] Navigate  [Enter] Open  [Esc] Close ";
    lines.push(Line::from(Span::styled(help, Theme::HINT)));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

async fn handle_menu_key(state: &mut AppState, key: &KeyEvent) {
    let selected = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::Menu { selected },
        } => *selected,
        _ => return,
    };

    match key.code {
        KeyCode::Up => {
            let new_sel = selected.saturating_sub(1);
            if let AppMode::Settings {
                mode: SettingsMode::Menu { ref mut selected },
            } = state.mode
            {
                *selected = new_sel;
            }
        }
        KeyCode::Down => {
            let max = MENU_ITEMS.len().saturating_sub(1);
            if selected < max
                && let AppMode::Settings {
                    mode: SettingsMode::Menu { ref mut selected },
                } = state.mode
            {
                *selected += 1;
            }
        }
        KeyCode::Enter => {
            let section = match selected {
                0 => SettingsSection::Core,
                1 => SettingsSection::ProtocolCore,
                2 => SettingsSection::Gui,
                3 => SettingsSection::Inbound,
                4 => SettingsSection::Routing,
                5 => SettingsSection::Dns,
                6 => SettingsSection::SystemProxy,
                7 => SettingsSection::Tun,
                8 => SettingsSection::Mux,
                9 => SettingsSection::Stats,
                10 => SettingsSection::Updates,
                11 => SettingsSection::SpeedTest,
                12 => SettingsSection::Logging,
                _ => return,
            };
            state.enter_settings_form(section).await;
        }
        KeyCode::Esc => {
            state.mode = AppMode::List;
        }
        _ => {}
    }
}
fn form_title_from_mode(mode: &SettingsMode) -> &'static str {
    match mode {
        SettingsMode::CoreForm { .. } => " Core Settings ",
        SettingsMode::GuiForm { .. } => " GUI Settings ",
        SettingsMode::InboundForm { .. } => " Inbound Settings ",
        SettingsMode::DnsForm { .. } => " DNS Settings ",
        SettingsMode::SystemProxyForm { .. } => " System Proxy ",
        SettingsMode::TunForm { .. } => " TUN Mode ",
        SettingsMode::MuxForm { .. } => " Mux / Fragment ",
        SettingsMode::StatsForm { .. } => " Statistics ",
        SettingsMode::ProtocolCoreForm { .. } => " Protocol Core ",
        SettingsMode::SpeedTestForm { .. } => " Speed Test Settings ",
        SettingsMode::LoggingForm { .. } => " Logging ",
        _ => " Settings ",
    }
}

pub static PROTOCOL_CORE_DEFS: &[(&str, &str, &str)] = &[
    ("vmess", "VMess", "Select:Auto,Xray,SingBox"),
    ("vless", "VLESS", "Select:Auto,Xray,SingBox"),
    ("ss", "Shadowsocks", "Select:Auto,Xray,SingBox"),
    ("shadowsocks-2022", "SS-2022", "Select:Auto,Xray,SingBox"),
    ("socks", "SOCKS", "Select:Auto,Xray,SingBox"),
    ("http", "HTTP", "Select:Auto,Xray,SingBox"),
    ("trojan", "Trojan", "Select:Auto,Xray,SingBox"),
    ("wire-guard", "WireGuard", "Select:Auto,Xray,SingBox"),
    ("hy2", "Hysteria2", "Select:Auto,Xray,SingBox"),
    ("hy", "Hysteria", "Select:Auto,Xray,SingBox"),
    ("tuic", "TUIC", "Select:Auto,Xray,SingBox"),
    ("naive", "Naïve", "Select:Auto,Xray,SingBox"),
    ("any-tls", "AnyTLS", "Select:Auto,Xray,SingBox"),
    ("shadow-tls", "ShadowTLS", "Select:Auto,Xray,SingBox"),
    ("tor", "Tor", "Select:Auto,Xray,SingBox"),
    ("ssh", "SSH", "Select:Auto,Xray,SingBox"),
    ("ssr", "ShadowsocksR", "Select:Auto,Xray,SingBox"),
    ("redirect", "Redirect", "Select:Auto,Xray,SingBox"),
    ("dokodemo-door", "Dokodemo-door", "Select:Auto,Xray,SingBox"),
    ("t-proxy", "TProxy", "Select:Auto,Xray,SingBox"),
    ("mixed", "Mixed", "Select:Auto,Xray,SingBox"),
    ("tailscale", "Tailscale", "Select:Auto,Xray,SingBox"),
];
fn form_field_defs(mode: &SettingsMode) -> &'static [(&'static str, &'static str, &'static str)] {
    // &[(key, label, field_type)]
    // field_type: "Text", "Number", "Boolean", "Select" (comma-separated)
    match mode {
        SettingsMode::CoreForm { .. } => &[
            ("xray_path", "Xray Path", "Text"),
            ("sing_box_path", "Sing-Box Path", "Text"),
            ("default_core", "Default Core", "Select:Auto,Xray,SingBox"),
            ("log_level", "Log Level", "Select:debug,info,warning,error"),
        ],
        SettingsMode::GuiForm { .. } => &[
            ("language", "Language", "Select:en,zh"),
            ("theme", "Theme", "Text"),
            ("refresh_interval", "Refresh Interval (s)", "Number"),
        ],
        SettingsMode::InboundForm { .. } => &[
            ("socks_port", "SOCKS Port", "Number"),
            ("http_port", "HTTP Port", "Number"),
            ("mixed_port", "Mixed Port", "Number"),
            ("listen", "Listen Address", "Text"),
            ("sniffing", "Sniffing", "Boolean"),
        ],
        SettingsMode::DnsForm { .. } => &[
            ("servers", "Servers (JSON)", "Text"),
            ("hosts", "Hosts (JSON)", "Text"),
            (
                "query_strategy",
                "Query Strategy",
                "Select:,,UseIP,UseIPv4,UseIPv6",
            ),
            ("disable_cache", "Disable Cache", "Boolean"),
            ("disable_fallback", "Disable Fallback", "Boolean"),
            ("client_ip", "Client IP", "Text"),
        ],
        SettingsMode::SystemProxyForm { .. } => &[
            ("enabled", "Enabled", "Boolean"),
            ("http_port", "HTTP Port", "Number"),
            ("socks_port", "SOCKS Port", "Number"),
            ("bypass", "Bypass", "Text"),
        ],
        SettingsMode::TunForm { .. } => &[
            ("enabled", "Enabled", "Boolean"),
            ("interface_name", "Interface Name", "Text"),
            ("mtu", "MTU", "Number"),
        ],
        SettingsMode::MuxForm { .. } => &[
            ("enabled", "Enabled", "Boolean"),
            ("concurrency", "Concurrency", "Number"),
            ("fragment_enabled", "Fragment Enabled", "Boolean"),
            ("fragment_packets", "Fragment Packets", "Text"),
            ("fragment_length", "Fragment Length", "Text"),
            ("fragment_interval", "Fragment Interval", "Text"),
        ],
        SettingsMode::ProtocolCoreForm { .. } => PROTOCOL_CORE_DEFS,
        SettingsMode::SpeedTestForm { .. } => &[
            ("ping_url", "Ping Test URL", "Text"),
            ("ip_api_url", "IP API URL", "Text"),
            ("tcp_timeout_secs", "TCP Timeout (sec)", "Number"),
            (
                "real_ping_timeout_secs",
                "Real Ping Timeout (sec)",
                "Number",
            ),
            ("batch_page_size", "Batch Page Size", "Number"),
            ("batch_delay_ms", "Batch Delay (ms)", "Number"),
            ("real_ping_retries", "Real Ping Retries", "Number"),
            ("real_ping_concurrency", "Real Ping Concurrency", "Number"),
        ],
        SettingsMode::LoggingForm { .. } => &[
            ("log_ttl_hours", "Log Retention (hours)", "Number"),
            ("log_batch_size", "Log Batch Size", "Number"),
        ],
        _ => &[],
    }
}

fn render_form(frame: &mut Frame, area: Rect, state: &AppState) {
    let mode = match &state.mode {
        AppMode::Settings { mode } => mode,
        _ => return,
    };
    let (fields, focus_index) = match mode {
        SettingsMode::CoreForm {
            fields,
            focus_index,
        }
        | SettingsMode::GuiForm {
            fields,
            focus_index,
        }
        | SettingsMode::InboundForm {
            fields,
            focus_index,
        }
        | SettingsMode::DnsForm {
            fields,
            focus_index,
        }
        | SettingsMode::SystemProxyForm {
            fields,
            focus_index,
        }
        | SettingsMode::TunForm {
            fields,
            focus_index,
        }
        | SettingsMode::MuxForm {
            fields,
            focus_index,
        }
        | SettingsMode::StatsForm {
            fields,
            focus_index,
        }
        | SettingsMode::ProtocolCoreForm {
            fields,
            focus_index,
        }
        | SettingsMode::SpeedTestForm {
            fields,
            focus_index,
        }
        | SettingsMode::LoggingForm {
            fields,
            focus_index,
        } => (fields, *focus_index),
        _ => return,
    };

    let title = form_title_from_mode(mode);
    let field_defs = form_field_defs(mode);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let label_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let focus_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightYellow)
        .add_modifier(Modifier::REVERSED);
    let hint_style = Style::default().fg(Color::Gray);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (_key, label, field_type)) in field_defs.iter().enumerate() {
        let is_focused = i == focus_index;
        let val = fields.get(i).map(|(_, v)| v.as_str()).unwrap_or("");

        let display_val = if let Some(options_csv) = field_type.strip_prefix("Select:") {
            let options: Vec<&str> = options_csv.split(',').collect();
            let idx = options.iter().position(|o| *o == val).unwrap_or(0);
            format!("{}/{} {}  ←→ change", idx + 1, options.len(), val)
        } else if *field_type == "Boolean" {
            if val == "true" {
                "[✓] Yes".into()
            } else {
                "[✗] No".into()
            }
        } else {
            if val.is_empty() {
                "(empty)".into()
            } else {
                val.to_string()
            }
        };

        let prefix = if is_focused { "> " } else { "  " };
        let label_text = format!("{prefix}{label}: ");
        let value_text = format!("[{}]", display_val);

        let spans = vec![
            Span::styled(
                label_text,
                if is_focused { focus_style } else { label_style },
            ),
            Span::styled(
                value_text,
                if is_focused {
                    focus_style
                } else {
                    Style::default()
                },
            ),
        ];
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    let help = " [Tab] Next  [Shift+Tab] Prev  [Enter] Save  [Esc] Cancel ";
    lines.push(Line::from(Span::styled(help, hint_style)));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn section_from_mode(mode: &SettingsMode) -> Option<SettingsSection> {
    match mode {
        SettingsMode::CoreForm { .. } => Some(SettingsSection::Core),
        SettingsMode::GuiForm { .. } => Some(SettingsSection::Gui),
        SettingsMode::InboundForm { .. } => Some(SettingsSection::Inbound),
        SettingsMode::DnsForm { .. } => Some(SettingsSection::Dns),
        SettingsMode::SystemProxyForm { .. } => Some(SettingsSection::SystemProxy),
        SettingsMode::TunForm { .. } => Some(SettingsSection::Tun),
        SettingsMode::MuxForm { .. } => Some(SettingsSection::Mux),
        SettingsMode::UpdateForm { .. } => Some(SettingsSection::Updates),
        SettingsMode::ProtocolCoreForm { .. } => Some(SettingsSection::ProtocolCore),
        SettingsMode::SpeedTestForm { .. } => Some(SettingsSection::SpeedTest),
        SettingsMode::LoggingForm { .. } => Some(SettingsSection::Logging),
        _ => None,
    }
}

fn handle_form_key(state: &mut AppState, key: &KeyEvent) {
    // Clone mode to extract data, then work through mutable state
    let mode_snapshot = match &state.mode {
        AppMode::Settings { mode } => mode.clone(),
        _ => return,
    };
    let section = match section_from_mode(&mode_snapshot) {
        Some(s) => s,
        None => return,
    };

    // Get field defs
    let field_defs = form_field_defs(&mode_snapshot);
    if field_defs.is_empty() {
        return;
    }
    let max_idx = field_defs.len().saturating_sub(1);

    // Extract current fields and focus_index from mutable state
    let (fields, focus_index) = match &mut state.mode {
        AppMode::Settings {
            mode:
                SettingsMode::CoreForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::GuiForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::InboundForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::DnsForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::SystemProxyForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::TunForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::MuxForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::StatsForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::ProtocolCoreForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::SpeedTestForm {
                    fields,
                    focus_index,
                }
                | SettingsMode::LoggingForm {
                    fields,
                    focus_index,
                },
        } => (fields, focus_index),
        _ => return,
    };

    match key.code {
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            *focus_index = if *focus_index >= max_idx {
                0
            } else {
                *focus_index + 1
            };
        }
        KeyCode::BackTab | KeyCode::Tab => {
            *focus_index = if *focus_index == 0 {
                max_idx
            } else {
                *focus_index - 1
            };
        }
        KeyCode::Up => {
            *focus_index = focus_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if *focus_index < max_idx {
                *focus_index += 1;
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if *focus_index >= fields.len() {
                return;
            }
            let def = field_defs[*focus_index];
            let (_, ref mut val) = fields[*focus_index];

            let field_type = def.2;
            if let Some(options_csv) = field_type.strip_prefix("Select:") {
                let options: Vec<&str> = options_csv.split(',').collect();
                let current_idx = options.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                let new_idx = (current_idx + 1) % options.len();
                val.clear();
                val.push_str(options[new_idx]);
            } else if field_type == "Boolean" {
                val.clear();
                val.push_str(if val == "true" { "false" } else { "true" });
            } else if field_type == "Number" {
                if c.is_ascii_digit() || c == '-' {
                    val.push(c);
                }
            } else {
                val.push(c);
            }
        }
        KeyCode::Backspace => {
            if *focus_index < fields.len() {
                let (_, ref mut val) = fields[*focus_index];
                val.pop();
            }
        }
        KeyCode::Enter => {
            let saved_fields = fields.clone();
            state.save_settings_form(section, &saved_fields);
        }
        KeyCode::Esc => {
            state.enter_settings();
        }
        KeyCode::Left | KeyCode::Right if *focus_index < field_defs.len() => {
            let def = field_defs[*focus_index];
            let field_type = def.2;
            if let Some(options_csv) = field_type.strip_prefix("Select:") {
                let options: Vec<&str> = options_csv.split(',').collect();
                let (_, ref mut val) = fields[*focus_index];
                let current_idx = options.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                let new_idx = if key.code == KeyCode::Right {
                    (current_idx + 1) % options.len()
                } else if current_idx == 0 {
                    options.len() - 1
                } else {
                    current_idx - 1
                };
                val.clear();
                val.push_str(options[new_idx]);
            }
        }
        _ => {}
    }
}

// ── Update form ──────────────────────────────────────────────────────────

fn render_update_form(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_xray = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::UpdateForm { status_xray, .. },
        } => status_xray,
        _ => return,
    };
    let status_singbox = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::UpdateForm { status_singbox, .. },
        } => status_singbox,
        _ => return,
    };

    let title = " Backend Updates ";
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let label_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let avail_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let error_style = Style::default().fg(Color::Red);
    let hint_style = Style::default().fg(Color::Gray);

    let mut lines: Vec<Line> = Vec::new();

    // ── Xray-core ──
    lines.push(Line::from(Span::styled("  Xray-core", header_style)));
    if let Some(ver) = &status_xray.current_version {
        lines.push(Line::from(format!("    Current: {ver}")));
    } else {
        lines.push(Line::from(Span::styled(
            "    Current: not installed",
            label_style,
        )));
    }
    if let Some(ver) = &status_xray.latest_version {
        lines.push(Line::from(format!("    Latest:  {ver}")));
    } else if status_xray.error.is_some() {
        lines.push(Line::from(Span::styled(
            "    Latest:  (check failed)",
            error_style,
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    Latest:  (checking...)",
            hint_style,
        )));
    }
    if status_xray.update_available {
        lines.push(Line::from(Span::styled(
            "    [Update available!]",
            avail_style,
        )));
    }
    if status_xray.downloading {
        lines.push(Line::from(Span::styled("    Downloading...", hint_style)));
    }
    if let Some(err) = &status_xray.error {
        lines.push(Line::from(Span::styled(
            format!("    Error: {err}"),
            error_style,
        )));
    }

    // ── Sing-box ──
    lines.push(Line::from(Span::styled("  Sing-box", header_style)));
    if let Some(ver) = &status_singbox.current_version {
        lines.push(Line::from(format!("    Current: {ver}")));
    } else {
        lines.push(Line::from(Span::styled(
            "    Current: not installed",
            label_style,
        )));
    }
    if let Some(ver) = &status_singbox.latest_version {
        lines.push(Line::from(format!("    Latest:  {ver}")));
    } else if status_singbox.error.is_some() {
        lines.push(Line::from(Span::styled(
            "    Latest:  (check failed)",
            error_style,
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    Latest:  (checking...)",
            hint_style,
        )));
    }
    if status_singbox.update_available {
        lines.push(Line::from(Span::styled(
            "    [Update available!]",
            avail_style,
        )));
    }
    if status_singbox.downloading {
        lines.push(Line::from(Span::styled("    Downloading...", hint_style)));
    }
    if let Some(err) = &status_singbox.error {
        lines.push(Line::from(Span::styled(
            format!("    Error: {err}"),
            error_style,
        )));
    }

    // Action hints
    lines.push(Line::from(""));
    let any_updates = status_xray.update_available || status_singbox.update_available;
    let help = if any_updates {
        " [C] Check for Updates  [D] Download & Install  [Esc] Back "
    } else {
        " [C] Check for Updates  [Esc] Back "
    };
    lines.push(Line::from(Span::styled(help, hint_style)));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn handle_update_form_key(state: &mut AppState, key: &KeyEvent) {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C')
            if !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            state.spawn_update_check();
            // Refresh the form with current status
            let status_xray = state
                .update_status
                .get(&CoreType::Xray)
                .cloned()
                .unwrap_or_default();
            let status_singbox = state
                .update_status
                .get(&CoreType::SingBox)
                .cloned()
                .unwrap_or_default();
            state.mode = AppMode::Settings {
                mode: SettingsMode::UpdateForm {
                    status_xray,
                    status_singbox,
                },
            };
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Download updates for all cores that have them available
            let any_updates = state.update_status.values().any(|s| s.update_available);
            if !any_updates {
                return;
            }
            // Check each core and trigger download if available
            let xray_avail = state
                .update_status
                .get(&CoreType::Xray)
                .map(|s| s.update_available)
                .unwrap_or(false);
            let singbox_avail = state
                .update_status
                .get(&CoreType::SingBox)
                .map(|s| s.update_available)
                .unwrap_or(false);

            if xray_avail {
                state.spawn_update_download(CoreType::Xray);
            }
            if singbox_avail {
                state.spawn_update_download(CoreType::SingBox);
            }
            // Refresh the form with current status
            let status_xray = state
                .update_status
                .get(&CoreType::Xray)
                .cloned()
                .unwrap_or_default();
            let status_singbox = state
                .update_status
                .get(&CoreType::SingBox)
                .cloned()
                .unwrap_or_default();
            state.mode = AppMode::Settings {
                mode: SettingsMode::UpdateForm {
                    status_xray,
                    status_singbox,
                },
            };
        }
        KeyCode::Esc => {
            state.enter_settings();
        }
        _ => {}
    }
}

// ── Routing list ────────────────────────────────────────────────────────

fn render_routing_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let selected = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::RoutingList { selected },
        } => *selected,
        _ => 0,
    };

    let rules = &state.routing_rules;

    // Centered overlay
    let overlay_area = Rect::new(
        area.width.saturating_sub(area.width * 80 / 100) / 2,
        area.height.saturating_sub(area.height * 75 / 100) / 2,
        area.width * 80 / 100,
        area.height * 75 / 100,
    );

    let block = Block::default()
        .title(" Routing Rules ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Header
    let header = Line::from(vec![
        Span::styled(
            " #  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Type  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Domains/IPs                    ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Outbound Tag",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(header, chunks[0]);

    // Rows
    let mut rows = Vec::new();
    for (i, rule) in rules.iter().enumerate() {
        let is_selected = i == selected;
        let domains = rule.domains.as_deref().unwrap_or("");
        let ips = rule.ips.as_deref().unwrap_or("");
        let targets = if !domains.is_empty() && !ips.is_empty() {
            format!("{domains}, {ips}")
        } else if !domains.is_empty() {
            domains.to_string()
        } else {
            ips.to_string()
        };
        let outbound = rule.outbound_tag.as_deref().unwrap_or("-");
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        rows.push(Line::from(vec![
            Span::styled(
                format!("{:>2}  ", i + 1),
                if is_selected {
                    style
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(format!("{}  ", rule.r#type), style),
            Span::styled(truncate_to(&targets, 32), style),
            Span::styled(outbound.to_string(), style),
        ]));
    }

    // Render as paragraph since we don't have a scrollable list widget
    // In future: use ratatui List or Table
    let list_para = Paragraph::new(rows);
    frame.render_widget(list_para, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        " [a] Add  [e] Edit  [d] Delete  [Ctrl+↑/↓] Reorder  [Esc] Back ",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, chunks[2]);
}

async fn handle_routing_list_key(state: &mut AppState, key: &KeyEvent) {
    let selected = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::RoutingList { selected },
        } => *selected,
        _ => return,
    };

    state.reload_routing_rules().await;
    let rules = &state.routing_rules;
    let max = rules.len().saturating_sub(1);

    match key.code {
        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if selected > 0 && selected <= rules.len() {
                let mut ids: Vec<(String, i32)> = rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        (
                            r.id.clone(),
                            if i == selected {
                                i as i32 - 1
                            } else if i == selected - 1 {
                                i as i32 + 1
                            } else {
                                i as i32
                            },
                        )
                    })
                    .collect();
                for (idx, (_, order)) in ids.iter_mut().enumerate() {
                    *order = idx as i32;
                }
                let _ = state.db.reorder_routing_rules(&ids).await;
                if let AppMode::Settings {
                    mode: SettingsMode::RoutingList { ref mut selected },
                } = state.mode
                {
                    *selected = selected.saturating_sub(1);
                }
                state.reload_routing_rules().await;
            }
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if selected < max {
                let mut ids: Vec<(String, i32)> = rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        (
                            r.id.clone(),
                            if i == selected {
                                i as i32 + 1
                            } else if i == selected + 1 {
                                i as i32 - 1
                            } else {
                                i as i32
                            },
                        )
                    })
                    .collect();
                for (idx, (_, order)) in ids.iter_mut().enumerate() {
                    *order = idx as i32;
                }
                let _ = state.db.reorder_routing_rules(&ids).await;
                if let AppMode::Settings {
                    mode: SettingsMode::RoutingList { ref mut selected },
                } = state.mode
                {
                    *selected += 1;
                }
                state.reload_routing_rules().await;
            }
        }
        KeyCode::Up => {
            if let AppMode::Settings {
                mode: SettingsMode::RoutingList { ref mut selected },
            } = state.mode
            {
                *selected = selected.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if selected < max
                && let AppMode::Settings {
                    mode: SettingsMode::RoutingList { ref mut selected },
                } = state.mode
            {
                *selected += 1;
            }
        }
        KeyCode::Char('a' | 'A') => {
            let fields = routing_rule_fields(None);
            state.mode = AppMode::Settings {
                mode: SettingsMode::RoutingForm {
                    rule_id: None,
                    fields,
                    focus_index: 0,
                },
            };
        }
        KeyCode::Char('e' | 'E') => {
            if !rules.is_empty() && selected < rules.len() {
                let fields = routing_rule_fields(Some(&rules[selected]));
                state.mode = AppMode::Settings {
                    mode: SettingsMode::RoutingForm {
                        rule_id: Some(rules[selected].id.clone()),
                        fields,
                        focus_index: 0,
                    },
                };
            }
        }
        KeyCode::Char('d' | 'D') => {
            let (rule_id, list_len) = if !rules.is_empty() && selected < rules.len() {
                (Some(rules[selected].id.clone()), rules.len())
            } else {
                (None, 0)
            };
            if let Some(id) = rule_id {
                let _ = state.db.delete_routing_rule(&id).await;
                state.log_trace("info", "tui", "Routing rule deleted");
                let new_max = list_len.saturating_sub(2);
                if let AppMode::Settings {
                    mode: SettingsMode::RoutingList { ref mut selected },
                } = state.mode
                    && *selected > new_max
                    && new_max > 0
                {
                    *selected = new_max;
                }
                state.reload_routing_rules().await;
            }
        }
        KeyCode::Esc => {
            state.mode = AppMode::Settings {
                mode: SettingsMode::Menu { selected: 3 },
            };
        }
        _ => {}
    }
}

// ── Routing form ────────────────────────────────────────────────────────

fn routing_rule_fields(rule: Option<&crate::RoutingRule>) -> Vec<(String, String)> {
    let keys = [
        "type",
        "domain_matcher",
        "domains",
        "ips",
        "inbound_tags",
        "port",
        "source_ports",
        "network",
        "protocols",
        "domain_strategy",
        "outbound_tag",
        "balancer_tag",
        "rule_set_file",
        "rule_set_url",
    ];
    keys.iter()
        .map(|k| {
            let val = match rule {
                Some(r) => match *k {
                    "type" => r.r#type.to_string(),
                    "domain_matcher" => r.domain_matcher.as_deref().unwrap_or("").to_string(),
                    "domains" => r.domains.as_deref().unwrap_or("").to_string(),
                    "ips" => r.ips.as_deref().unwrap_or("").to_string(),
                    "inbound_tags" => r.inbound_tags.as_deref().unwrap_or("").to_string(),
                    "port" => r.port.as_deref().unwrap_or("").to_string(),
                    "source_ports" => r.source_ports.as_deref().unwrap_or("").to_string(),
                    "network" => r.network.as_deref().unwrap_or("").to_string(),
                    "protocols" => r.protocols.as_deref().unwrap_or("").to_string(),
                    "domain_strategy" => r.domain_strategy.as_deref().unwrap_or("").to_string(),
                    "outbound_tag" => r.outbound_tag.as_deref().unwrap_or("").to_string(),
                    "balancer_tag" => r.balancer_tag.as_deref().unwrap_or("").to_string(),
                    "rule_set_file" => r.rule_set_file.as_deref().unwrap_or("").to_string(),
                    "rule_set_url" => r.rule_set_url.as_deref().unwrap_or("").to_string(),
                    _ => String::new(),
                },
                None => String::new(),
            };
            (k.to_string(), val)
        })
        .collect()
}

const ROUTING_FIELD_DEFS: &[(&str, &str, &str)] = &[
    ("type", "Type", "Number"),
    ("domain_matcher", "Domain Matcher", "Text"),
    ("domains", "Domains", "Text"),
    ("ips", "IPs", "Text"),
    ("inbound_tags", "Inbound Tags", "Text"),
    ("port", "Port", "Text"),
    ("source_ports", "Source Ports", "Text"),
    ("network", "Network", "Text"),
    ("protocols", "Protocols", "Text"),
    ("domain_strategy", "Domain Strategy", "Text"),
    ("outbound_tag", "Outbound Tag", "Text"),
    ("balancer_tag", "Balancer Tag", "Text"),
    ("rule_set_file", "Rule Set (File)", "Text"),
    ("rule_set_url", "Rule Set (URL)", "Text"),
];

fn render_routing_form(frame: &mut Frame, area: Rect, state: &AppState) {
    let (fields, focus_index) = match &state.mode {
        AppMode::Settings {
            mode:
                SettingsMode::RoutingForm {
                    fields,
                    focus_index,
                    ..
                },
        } => (fields, *focus_index),
        _ => return,
    };

    let is_edit = match &state.mode {
        AppMode::Settings {
            mode: SettingsMode::RoutingForm { rule_id, .. },
        } => rule_id.is_some(),
        _ => false,
    };

    let title = if is_edit {
        " Edit Routing Rule "
    } else {
        " Add Routing Rule "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let label_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let focus_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightYellow)
        .add_modifier(Modifier::REVERSED);
    let hint_style = Style::default().fg(Color::Gray);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (_key, label, field_type)) in ROUTING_FIELD_DEFS.iter().enumerate() {
        let is_focused = i == focus_index;
        let val = fields.get(i).map(|(_, v)| v.as_str()).unwrap_or("");

        let display_val = if *field_type == "Boolean" {
            if val == "true" {
                "[✓] Yes".into()
            } else {
                "[✗] No".into()
            }
        } else {
            if val.is_empty() {
                "(empty)".into()
            } else {
                val.to_string()
            }
        };

        let prefix = if is_focused { "> " } else { "  " };
        let label_text = format!("{prefix}{label}: ");
        let value_text = format!("[{}]", display_val);

        let spans = vec![
            Span::styled(
                label_text,
                if is_focused { focus_style } else { label_style },
            ),
            Span::styled(
                value_text,
                if is_focused {
                    focus_style
                } else {
                    Style::default()
                },
            ),
        ];
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    let help = " [Tab] Next  [Shift+Tab] Prev  [Enter] Save  [Esc] Cancel ";
    lines.push(Line::from(Span::styled(help, hint_style)));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

async fn handle_routing_form_key(state: &mut AppState, key: &KeyEvent) {
    let (rule_id, fields, focus_index) = match &mut state.mode {
        AppMode::Settings {
            mode:
                SettingsMode::RoutingForm {
                    rule_id,
                    fields,
                    focus_index,
                },
        } => (rule_id.clone(), fields, focus_index),
        _ => return,
    };

    let max_idx = ROUTING_FIELD_DEFS.len().saturating_sub(1);

    match key.code {
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            *focus_index = if *focus_index >= max_idx {
                0
            } else {
                *focus_index + 1
            };
        }
        KeyCode::BackTab | KeyCode::Tab => {
            *focus_index = if *focus_index == 0 {
                max_idx
            } else {
                *focus_index - 1
            };
        }
        KeyCode::Up => {
            *focus_index = focus_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if *focus_index < max_idx {
                *focus_index += 1;
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if *focus_index >= fields.len() {
                return;
            }
            let def = ROUTING_FIELD_DEFS[*focus_index];
            let (_, ref mut val) = fields[*focus_index];
            let field_type = def.2;

            if field_type == "Boolean" {
                val.clear();
                val.push_str(if val == "true" { "false" } else { "true" });
            } else if field_type == "Number" {
                if c.is_ascii_digit() || c == '-' {
                    val.push(c);
                }
            } else {
                val.push(c);
            }
        }
        KeyCode::Backspace => {
            if *focus_index < fields.len() {
                let (_, ref mut val) = fields[*focus_index];
                val.pop();
            }
        }
        KeyCode::Enter => {
            let saved_fields = fields.clone();
            state
                .save_routing_rule(rule_id.clone(), &saved_fields)
                .await;
            state.mode = AppMode::Settings {
                mode: SettingsMode::RoutingList { selected: 0 },
            };
        }
        KeyCode::Esc => {
            state.mode = AppMode::Settings {
                mode: SettingsMode::RoutingList { selected: 0 },
            };
        }
        _ => {}
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn truncate_to(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
