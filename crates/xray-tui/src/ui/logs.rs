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

    // Filter bar (1 line at top of inner)
    let (filter_area, log_area) = {
        let rects = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(inner);
        (rects[0], rects[1])
    };
    let core_label = if state.logs_show_core { "ON" } else { "OFF" };
    let tui_label = if state.logs_show_tui { "ON" } else { "OFF" };
    let bar = Line::from(Span::styled(
        format!(" [C]ore: {}  [T]UI: {}", core_label, tui_label),
        Theme::HINT,
    ));
    frame.render_widget(Paragraph::new(bar), filter_area);

    // Filter log buffer by source
    let filtered: Vec<&crate::LogLine> = state.log_buffer.iter()
        .filter(|l| (l.source == "core" && state.logs_show_core)
                 || (l.source == "tui" && state.logs_show_tui)
                 || (l.source != "core" && l.source != "tui"))
        .collect();

    let log_count = filtered.len();
    if log_count == 0 {
        let paragraph = Paragraph::new(Line::from("No logs"))
            .style(Theme::HINT);
        frame.render_widget(paragraph, log_area);
        return;
    }

    // Clamp scroll (0 = newest at bottom)
    let scroll = state.log_scroll.min(log_count.saturating_sub(1));

    // Compute visible range (newest at highest index, scroll up reveals older)
    let height = (log_area.height as usize).saturating_sub(1);
    let start = log_count.saturating_sub(scroll + height);
    let end = log_count.saturating_sub(scroll);
    let visible = &filtered[start..end];

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
    frame.render_widget(paragraph, log_area);
}
