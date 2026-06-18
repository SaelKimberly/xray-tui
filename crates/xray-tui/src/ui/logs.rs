use ratatui::Frame;
use ratatui::layout::Rect;

use crate::AppState;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState) {
    super::render_placeholder_screen(frame, area, "Logs");
}
