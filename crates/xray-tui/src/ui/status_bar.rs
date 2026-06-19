use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (left_text, left_style) = if state.connecting {
        (
            " Connecting...".to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if let Some((done, total)) = state.test_progress {
        (
            format!(" Testing: {done}/{total} profiles..."),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if !state.testing_profiles.is_empty() {
        (
            " Testing...".to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if let Some(err) = &state.connection_error {
        (
            format!(" Error: {err}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        match &state.connected_core {
            Some(core) => (
                format!(" Connected [{}]", core),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            None => (
                " Disconnected".to_string(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        }
    };

    // Append update indicator if any backend has an update available
    let update_indicator = if state.update_status.values().any(|s| s.update_available) {
        let cores: Vec<&str> = state.update_status.iter()
            .filter(|(_, s)| s.update_available)
            .map(|(ct, _)| match ct {
                xray_tui_core::CoreType::Xray => "xray",
                xray_tui_core::CoreType::SingBox => "sing-box",
                xray_tui_core::CoreType::Auto => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        if cores.is_empty() {
            String::new()
        } else {
            format!(" [Update: {}]", cores.join(", "))
        }
    } else {
        String::new()
    };
    let left_text = format!("{left_text}{update_indicator}");

    let right_text = if state.connected_core.is_some() {
        " [Ctrl+Shift+C] Disconnect  [Tab] Next  [Ctrl+Q] Quit "
    } else if state.connecting {
        " [Tab] Next  [Ctrl+Q] Quit "
    } else {
        " [Ctrl+Enter] Connect  [Tab] Next  [Ctrl+Q] Quit "
    };
    let right_style = Style::default().fg(Color::Gray);

    // Pad left to fill area, right-align the hint
    let width = area.width as usize;
    let left_len = left_text.len();
    let padding = width.saturating_sub(left_len + right_text.len());

    let line = Line::from(vec![
        Span::styled(left_text.clone(), left_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(right_text, right_style),
    ]);

    let bg = Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::REVERSED);

    let paragraph = Paragraph::new(line).style(bg);
    frame.render_widget(paragraph, area);
}
