use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (left_text, left_style) = if state.connecting {
        (
            " Connecting...".to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if let Some(err) = &state.connection_error {
        (
            format!(" Error: {err}"),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
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
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        }
    };

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
