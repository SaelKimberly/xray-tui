use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let log_count = state.log_buffer.len();
    if log_count == 0 {
        let paragraph = Paragraph::new(Line::from("No logs"))
            .style(Theme::HINT);
        frame.render_widget(paragraph, inner);
        return;
    }

    // Clamp scroll (0 = newest at bottom)
    let scroll = state.log_scroll.min(log_count.saturating_sub(1));

    // Compute visible range (newest at highest index, scroll up reveals older)
    let height = (inner.height as usize).saturating_sub(1);
    let start = log_count.saturating_sub(scroll + height);
    let end = log_count.saturating_sub(scroll);
    let visible = &state.log_buffer[start..end];

    let lines: Vec<Line> = visible
        .iter()
        .map(|log| {
            let style = match log.level.as_str() {
                "error" | "fatal" | "panic" => Theme::ERROR,
                "warning" | "warn" => Theme::WARNING,
                "info" => Style::default(),
                "debug" | "trace" => Theme::HINT,
                _ => Style::default(),
            };
            Line::from(Span::styled(&log.message, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
