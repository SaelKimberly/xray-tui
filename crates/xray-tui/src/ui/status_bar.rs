use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::AppState;
use crate::ui::theme::Theme;
use std::sync::atomic::{AtomicU16, Ordering};

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
static SPINNER_TICK: AtomicU16 = AtomicU16::new(0);

fn spinner_char() -> char {
    let tick = SPINNER_TICK.fetch_add(1, Ordering::Relaxed);
    SPINNER_CHARS[tick as usize % SPINNER_CHARS.len()]
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (left_text, left_style) = if state.connecting {
        (
            format!(" {} Connecting... ", spinner_char()),
            Theme::SPINNER,
        )
    } else if let Some((done, total)) = state.test_progress {
        (
            format!(" Testing: {done}/{total} profiles..."),
            Theme::WARNING,
        )
    } else if !state.testing_profiles.is_empty() {
        (" Testing...".to_string(), Theme::WARNING)
    } else if let Some(err) = &state.connection_error {
        (format!(" Error: {err}"), Theme::ERROR)
    } else {
        state.connected_core.as_ref().map_or_else(
            || (" Disconnected".to_string(), Theme::ERROR),
            |core| (format!(" Connected [{core}]"), Theme::SUCCESS),
        )
    };

    // Append update indicator if any backend has an update available
    let update_indicator_span = if state.update_status.values().any(|s| s.update_available) {
        let cores: Vec<&str> = state
            .update_status
            .iter()
            .filter(|(_, s)| s.update_available)
            .map(|(ct, _)| match ct {
                xray_tui_core::CoreType::Xray => "xray",
                xray_tui_core::CoreType::SingBox => "sing-box",
                xray_tui_core::CoreType::Auto => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        if cores.is_empty() {
            Span::raw("")
        } else {
            Span::styled(
                format!(" ⇧Update:{} ", cores.join(",")),
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        Span::raw("")
    };
    // Download progress indicator while updating backends
    let download_span = {
        let parts: Vec<String> = state
            .update_status
            .iter()
            .filter(|(_, s)| s.downloading)
            .map(|(ct, s)| {
                let name = match ct {
                    xray_tui_core::CoreType::Xray => "x",
                    xray_tui_core::CoreType::SingBox => "sb",
                    xray_tui_core::CoreType::Auto => "",
                };
                if let Some((done, total)) = s.download_progress {
                    let pct = if total > 0 {
                        format!("{}%", (done as f64 / total as f64 * 100.0) as u64)
                    } else {
                        "??".to_string()
                    };
                    format!(" ⇩{name} {pct}")
                } else {
                    format!(" ⇩{name}")
                }
            })
            .collect();
        if parts.is_empty() {
            Span::raw("")
        } else {
            Span::styled(parts.join(" "), Theme::SPINNER)
        }
    };
    // Mode indicator prefix
    let mode_prefix = match &state.mode {
        crate::AppMode::Settings { mode } => match mode {
            crate::SettingsMode::Menu { .. } => " Settings",
            crate::SettingsMode::CoreForm { .. } => " Settings > Core",
            crate::SettingsMode::GuiForm { .. } => " Settings > GUI",
            crate::SettingsMode::InboundForm { .. } => " Settings > Inbound",
            crate::SettingsMode::SystemProxyForm { .. } => " Settings > System Proxy",
            crate::SettingsMode::TunForm { .. } => " Settings > TUN",
            crate::SettingsMode::MuxForm { .. } => " Settings > Mux",
            crate::SettingsMode::StatsForm { .. } => " Settings > Statistics",
            crate::SettingsMode::DnsForm { .. } => " Settings > DNS",
            crate::SettingsMode::RoutingList { .. } | crate::SettingsMode::RoutingForm { .. } => {
                " Settings > Routing"
            }
            crate::SettingsMode::UpdateForm { .. } => " Settings > Updates",
            crate::SettingsMode::ProtocolCoreForm { .. } => " Settings > Protocol Core",
            crate::SettingsMode::LoggingForm { .. } => " Settings > Logging",
            crate::SettingsMode::SpeedTestForm { .. } => " Settings > Speed Test",
        },
        crate::AppMode::AddServer { .. } => " Add Server",
        crate::AppMode::EditServer { .. } => " Edit Server",
        crate::AppMode::SpeedTestMenu { .. } => " Server Tools",
        crate::AppMode::BatchImport { .. } => " Batch Import",
        crate::AppMode::Help => " Help",
        _ => "",
    };

    let left_text = format!("{mode_prefix}{left_text}");

    // Dynamic right-side hints
    let right_text = build_hints(state);
    let right_style = Style::default().fg(Color::Gray);

    let width = area.width as usize;
    let left_len = left_text.len();
    let padding = width.saturating_sub(left_len + right_text.len());

    let mut spans = vec![
        Span::styled(left_text, left_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(right_text, right_style),
    ];
    // Insert indicators between left text and padding
    let mut insert_pos = 1;
    if !download_span.content.is_empty() {
        spans.insert(insert_pos, Span::raw(" "));
        insert_pos += 1;
        spans.insert(insert_pos, download_span);
        insert_pos += 1;
    }
    if !update_indicator_span.content.is_empty() {
        spans.insert(insert_pos, Span::raw(" "));
        insert_pos += 1;
        spans.insert(insert_pos, update_indicator_span);
    }

    let line = Line::from(spans);

    let bg = Theme::STATUS_BAR_BG;

    let paragraph = Paragraph::new(line).style(bg);
    frame.render_widget(paragraph, area);
}

/// Build context-sensitive keyboard hints for the right side of the status bar.
const fn build_hints(state: &AppState) -> &'static str {
    if matches!(&state.mode, crate::AppMode::Help) {
        return " [Esc] Close help ";
    }
    match &state.mode {
        crate::AppMode::SpeedTestMenu { .. } => " [↑↓] Navigate  [Enter] Select  [Esc] Close ",
        crate::AppMode::Settings { mode } => match mode {
            crate::SettingsMode::Menu { .. } => " [↑↓] Navigate  [Enter] Open  [Esc] Close ",
            _ => " [Tab/Shift+Tab] Focus  [Enter] Save  [Esc] Cancel ",
        },
        _ => {
            // Default tab-based hints
            match state.current_tab {
                crate::Tab::Profiles => {
                    if matches!(state.mode, crate::AppMode::ManageGroups { .. }) {
                        " [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    } else if state.connected_core.is_some() {
                        " [g] Groups  [Ctrl+Shift+C] Disconnect  [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    } else if state.connecting {
                        " [g] Groups  [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    } else {
                        " [g] Groups  [Ctrl+Enter/Ctrl+G] Connect  [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    }
                }
                crate::Tab::Settings => " [Enter] Open  [Ctrl+Q] Quit ",
                crate::Tab::Logs | crate::Tab::Statistics => " [Ctrl+Q] Quit ",
            }
        }
    }
}
