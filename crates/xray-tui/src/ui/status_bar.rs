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
        match &state.connected_core {
            Some(core) => (format!(" Connected [{core}]"), Theme::SUCCESS),
            None => (" Disconnected".to_string(), Theme::ERROR),
        }
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
            crate::SettingsMode::RoutingList { .. } => " Settings > Routing",
            crate::SettingsMode::UpdateForm { .. } => " Settings > Updates",
            crate::SettingsMode::ProtocolCoreForm { .. } => " Settings > Protocol Core",
            crate::SettingsMode::LoggingForm { .. } => " Settings > Logging",
            crate::SettingsMode::SpeedTestForm { .. } => " Settings > Speed Test",
            crate::SettingsMode::RoutingForm { .. } => " Settings > Routing",
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

    // Pad left to fill area, right-align the hint
    let width = area.width as usize;
    let left_len = left_text.len();
    let padding = width.saturating_sub(left_len + right_text.len());

    let mut spans = vec![
        Span::styled(left_text, left_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(right_text, right_style),
    ];
    // Insert update indicator between left text and padding if non-empty
    if !update_indicator_span.content.is_empty() {
        spans.insert(1, Span::raw(" "));
        spans.insert(2, update_indicator_span);
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
                    if state.connected_core.is_some() {
                        " [Ctrl+Shift+C] Disconnect  [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    } else if state.connecting {
                        " [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    } else {
                        " [Ctrl+Enter/Ctrl+G] Connect  [Tab] Next  [?] Help  [Ctrl+Q] Quit "
                    }
                }
                crate::Tab::Settings => " [Enter] Open  [Ctrl+Q] Quit ",
                crate::Tab::Logs | crate::Tab::Statistics => " [Ctrl+Q] Quit ",
            }
        }
    }
}
