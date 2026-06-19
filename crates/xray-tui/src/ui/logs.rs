use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::AppState;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState) {
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(crate::ui::theme::Theme::CONTAINER_BORDER)
        .title_style(crate::ui::theme::Theme::CONTAINER_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = "Log viewer — coming in a future update";
    let paragraph = Paragraph::new(Line::from(text))
        .style(crate::ui::theme::Theme::HINT);
    frame.render_widget(paragraph, inner);
}
