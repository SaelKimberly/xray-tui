use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use xray_tui_config::forms::{FieldSection, FormFieldType, form_fields_for};
use xray_tui_core::SINGBOX_ONLY_PROTOCOLS;
use xray_tui_core::protocol::Protocol;

use crate::{AppMode, AppState};
use crate::ui::theme::Theme;

/// Protocol picker layout groups
const XRAY_PROTOCOLS: &[Protocol] = &[
    Protocol::Vmess,
    Protocol::Vless,
    Protocol::Shadowsocks,
    Protocol::Shadowsocks2022,
    Protocol::Socks,
    Protocol::Http,
    Protocol::Trojan,
    Protocol::WireGuard,
    Protocol::Hysteria2,
    Protocol::DokodemoDoor,
    Protocol::Freedom,
    Protocol::Blackhole,
    Protocol::Dns,
    Protocol::Loopback,
    Protocol::Custom,
];

fn singbox_protocols() -> Vec<Protocol> {
    SINGBOX_ONLY_PROTOCOLS
        .iter()
        .filter(|p| !matches!(p, Protocol::TProxy | Protocol::Mixed))
        .copied()
        .collect()
}

const ALL_PICKER_PROTOCOLS: &[Protocol] = &[
    Protocol::Vmess,
    Protocol::Vless,
    Protocol::Shadowsocks,
    Protocol::Shadowsocks2022,
    Protocol::Socks,
    Protocol::Http,
    Protocol::Trojan,
    Protocol::WireGuard,
    Protocol::Hysteria2,
    Protocol::DokodemoDoor,
    Protocol::Freedom,
    Protocol::Blackhole,
    Protocol::Dns,
    Protocol::Loopback,
    Protocol::Custom,
    Protocol::Tuic,
    Protocol::Hysteria,
    Protocol::Naive,
    Protocol::AnyTls,
    Protocol::ShadowTls,
    Protocol::Tor,
    Protocol::Ssh,
    Protocol::Tailscale,
    Protocol::ShadowsocksR,
    Protocol::Redirect,
];

// ── Render dispatch ──────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    match &state.mode {
        AppMode::AddServer { protocol: None, .. } => {
            render_protocol_picker(frame, area, state);
        }
        AppMode::AddServer {
            protocol: Some(p),
            fields,
            focus_index,
        } => {
            render_form(frame, area, *p, fields, *focus_index, false);
        }
        AppMode::EditServer {
            profile_id,
            fields,
            focus_index,
        } => {
            let proto = state
                .db
                .get_profile(profile_id)
                .ok()
                .flatten()
                .and_then(|p| Protocol::try_from_i32(p.config_type))
                .unwrap_or(Protocol::Custom);
            render_form(frame, area, proto, fields, *focus_index, true);
        }
        _ => {}
    }
}

pub fn render_import_url(frame: &mut Frame, area: Rect, state: &AppState) {
    let (input, error) = match &state.mode {
        AppMode::ImportUrl { input, error } => (input, error),
        _ => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let title = "Import URL — Ctrl+V paste, Enter parse, Esc cancel";
    let block = Block::default().title(title).borders(Borders::ALL);
    let input_text = if input.is_empty() {
        " (paste URL here) ".to_string()
    } else {
        input.clone()
    };
    let input_para = Paragraph::new(input_text).block(block);
    frame.render_widget(input_para, chunks[0]);

    if let Some(err) = error {
        let err_style = Style::default().fg(Color::Red);
        let err_line = Line::from(Span::styled(format!("Error: {err}"), err_style));
        frame.render_widget(Paragraph::new(err_line), chunks[1]);
    }
}

// ── Protocol picker ──────────────────────────────────────────────────────

fn render_protocol_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("Select Protocol")
        .borders(Borders::ALL)
        .border_style(crate::ui::theme::Theme::CONTAINER_BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let xray_header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let singbox_header_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightYellow)
        .add_modifier(Modifier::REVERSED);

    let mut lines: Vec<Line> = Vec::new();
    let mut idx: usize = 0;
    let picker_offset = state.selected_index;

    lines.push(Line::from(Span::styled(
        "─ Xray-core ──────────────",
        xray_header_style,
    )));
    for proto in XRAY_PROTOCOLS {
        let is_sel = idx == picker_offset;
        let text = format!("  {proto}");
        let style = if is_sel {
            selected_style
        } else {
            Style::default().fg(Color::Blue)
        };
        lines.push(Line::from(Span::styled(text, style)));
        idx += 1;
    }

    lines.push(Line::from(Span::styled(
        "─ Sing-box ────────────────",
        singbox_header_style,
    )));
    for proto in singbox_protocols() {
        let is_sel = idx == picker_offset;
        let text = format!("  {proto}");
        let style = if is_sel {
            selected_style
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(Span::styled(text, style)));
        idx += 1;
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ↑↓ Navigate  Enter Select  Esc Cancel",
        Style::default().fg(Color::Gray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

// ── Form render ──────────────────────────────────────────────────────────

fn render_form(
    frame: &mut Frame,
    area: Rect,
    protocol: Protocol,
    fields: &[(String, String)],
    focus_index: usize,
    _is_edit: bool,
) {
    let title = format!(" {protocol} ");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(crate::ui::theme::Theme::CONTAINER_BORDER)
        .title_style(crate::ui::theme::Theme::CONTAINER_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let form_fields = form_fields_for(protocol);
    let focus_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightYellow)
        .add_modifier(Modifier::REVERSED);
    let label_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let required_style = Style::default().fg(Color::Red);
    let hint_style = Theme::HINT;

    let mut lines: Vec<Line> = Vec::new();

    for (i, ff) in form_fields.iter().enumerate() {
        let is_focused = i == focus_index;
        let val = fields.get(i).map(|(_, v)| v.as_str()).unwrap_or("");
        let display_val = match ff.field_type {
            FormFieldType::Password => {
                if val.is_empty() {
                    "".into()
                } else {
                    "••••••".into()
                }
            }
            FormFieldType::Boolean => {
                if val == "true" {
                    "✓ true".into()
                } else {
                    "✗ false".into()
                }
            }
            FormFieldType::Select(options) => {
                let idx = options.iter().position(|o| *o == val).unwrap_or(0);
                format!("{}/{} {}  ←→ change", idx + 1, options.len(), val)
            }
            _ => {
                if val.is_empty() {
                    "(empty)".into()
                } else {
                    val.to_string()
                }
            }
        };

        let req_mark = if ff.required { "*" } else { " " };
        let label_text = format!("{}{}: ", req_mark, ff.label);
        let value_text = format!("[{}]", display_val);
        let section_hint = match ff.section {
            FieldSection::Common => "",
            FieldSection::StreamSetting => " [stream]",
            FieldSection::ProtocolSetting => " [proto]",
        };

        let spans = vec![
            Span::styled(
                label_text,
                if is_focused && ff.required {
                    required_style
                } else {
                    label_style
                },
            ),
            Span::styled(
                value_text,
                if is_focused {
                    focus_style
                } else {
                    Style::default()
                },
            ),
            Span::styled(section_hint, hint_style),
        ];
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    let help = " Tab/Shift+Tab focus  ↵ Enter save  Esc cancel  Ctrl+S save";
    lines.push(Line::from(Span::styled(help, hint_style)));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

// ── Key handler ──────────────────────────────────────────────────────────

pub fn handle_key(state: &mut AppState, key: &KeyEvent) {
    // Determine mode type first without borrowing state.mode deeply
    let is_picker = matches!(&state.mode, AppMode::AddServer { protocol: None, .. });
    let is_import = matches!(&state.mode, AppMode::ImportUrl { .. });
    let is_form = matches!(
        &state.mode,
        AppMode::AddServer {
            protocol: Some(_),
            ..
        } | AppMode::EditServer { .. }
    );

    if is_picker {
        handle_picker_key(state, key);
        return;
    }

    if is_import {
        handle_import_key(state, key);
        return;
    }

    if is_form {
        handle_form_key(state, key);
    }
}

fn handle_picker_key(state: &mut AppState, key: &KeyEvent) {
    let protocol_count = XRAY_PROTOCOLS.len() + singbox_protocols().len();
    if protocol_count == 0 {
        return;
    }

    match key.code {
        KeyCode::Up => {
            state.selected_index = state.selected_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if state.selected_index < protocol_count - 1 {
                state.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if state.selected_index < ALL_PICKER_PROTOCOLS.len() {
                let proto = ALL_PICKER_PROTOCOLS[state.selected_index];
                let fields = form_fields_for(proto)
                    .iter()
                    .map(|f| (f.key.to_string(), f.default.to_string()))
                    .collect::<Vec<_>>();
                state.mode = AppMode::AddServer {
                    protocol: Some(proto),
                    fields,
                    focus_index: 0,
                };
            }
        }
        KeyCode::Esc => state.cancel_form(),
        _ => {}
    }
}
fn handle_import_key(state: &mut AppState, key: &KeyEvent) {
    let (url_to_import, should_import) = match &mut state.mode {
        AppMode::ImportUrl { input, error: _ } => match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                input.push(c);
                (None, false)
            }
            KeyCode::Backspace => {
                input.pop();
                (None, false)
            }
            KeyCode::Enter => (Some(input.clone()), true),
            KeyCode::Esc => {
                state.cancel_form();
                return;
            }
            _ => (None, false),
        },
        _ => (None, false),
    };

    if should_import && let Some(url) = url_to_import {
        state.import_url(&url);
    }
}

fn handle_form_key(state: &mut AppState, key: &KeyEvent) {
    // Extract protocol from current mode
    let protocol = match &state.mode {
        AppMode::AddServer {
            protocol: Some(p), ..
        } => Some(*p),
        AppMode::EditServer { profile_id, .. } => state
            .db
            .get_profile(profile_id)
            .ok()
            .flatten()
            .and_then(|p| Protocol::try_from_i32(p.config_type)),
        _ => None,
    };

    let protocol = match protocol {
        Some(p) => p,
        None => return,
    };

    // Now match on mode again to get mutable refs to fields
    let field_data = match &mut state.mode {
        AppMode::AddServer {
            protocol: Some(_),
            fields,
            focus_index,
        } => Some((protocol, fields, focus_index, false)),
        AppMode::EditServer {
            fields,
            focus_index,
            ..
        } => Some((protocol, fields, focus_index, true)),
        _ => None,
    };

    let (protocol, fields, focus_index, is_edit) = match field_data {
        Some(d) => d,
        None => return,
    };

    let form_fields = form_fields_for(protocol);
    if form_fields.is_empty() {
        return;
    }
    let max_idx = form_fields.len().saturating_sub(1);

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
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if is_edit {
                state.confirm_edit_server();
            } else {
                state.confirm_add_server();
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if *focus_index >= fields.len() {
                return;
            }
            let ff = &form_fields[*focus_index];
            let (_, ref mut val) = fields[*focus_index];

            match ff.field_type {
                FormFieldType::Select(options) => {
                    let current_idx = options.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                    let new_idx = (current_idx + 1) % options.len();
                    val.clear();
                    val.push_str(options[new_idx]);
                }
                FormFieldType::Boolean => {
                    val.clear();
                    val.push_str(if val == "true" { "false" } else { "true" });
                }
                FormFieldType::Number => {
                    if c.is_ascii_digit() || c == '-' {
                        val.push(c);
                    }
                }
                _ => {
                    val.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if *focus_index < fields.len() {
                let (_, ref mut val) = fields[*focus_index];
                val.pop();
            }
        }
        KeyCode::Enter => {
            if is_edit {
                state.confirm_edit_server();
            } else {
                state.confirm_add_server();
            }
        }
        KeyCode::Esc => state.cancel_form(),
        KeyCode::Left | KeyCode::Right if *focus_index < form_fields.len() => {
            let ff = &form_fields[*focus_index];
            if let FormFieldType::Select(options) = ff.field_type {
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
