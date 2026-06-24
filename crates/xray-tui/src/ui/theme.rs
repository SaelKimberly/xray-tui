use ratatui::style::{Color, Modifier, Style};

/// Central color palette and style definitions for the TUI.
///
/// All UI modules should reference `Theme::*` rather than defining inline
/// `Style` values, ensuring a consistent visual theme across screens.
pub struct Theme;

impl Theme {
    // Tab bar
    pub const TAB_SELECTED: Style = Style::new()
        .fg(Color::White)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    pub const TAB_DESELECTED: Style = Style::new().fg(Color::White).bg(Color::DarkGray);

    // Table / profile grid
    pub const TABLE_HEADER: Style = Style::new()
        .fg(Color::White)
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    pub const TABLE_ROW_SELECTED: Style = Style::new()
        .fg(Color::White)
        .bg(Color::Rgb(50, 60, 90))
        .add_modifier(Modifier::BOLD);
    pub const TABLE_ROW_ALT: Style = Style::new().fg(Color::White).bg(Color::Rgb(25, 25, 35));
    pub const TABLE_ROW_NORMAL: Style = Style::new().fg(Color::White);
    pub const TABLE_ROW_CONNECTED: Style = Style::new()
        .fg(Color::White)
        .bg(Color::Rgb(30, 70, 40))
        .add_modifier(Modifier::BOLD);

    // Containers / borders
    pub const CONTAINER_BORDER: Style = Style::new().fg(Color::Cyan);
    pub const CONTAINER_TITLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    // Status bar
    pub const STATUS_BAR_BG: Style = Style::new().bg(Color::Rgb(20, 30, 60)).fg(Color::White);
    pub const STATUS_BAR_MODE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    // Feedback / progress
    pub const PROGRESS_BAR: Style = Style::new().fg(Color::White).bg(Color::DarkGray);
    pub const PROGRESS_FILL: Style = Style::new().fg(Color::Cyan).bg(Color::Cyan);
    pub const SPINNER: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    // Semantic
    pub const ERROR: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);
    pub const WARNING: Style = Style::new().fg(Color::Yellow);
    pub const SUCCESS: Style = Style::new().fg(Color::Green);
    pub const HINT: Style = Style::new().fg(Color::Gray);
    pub const STATUS_FOOTER: Style = Style::new()
        .bg(Color::Rgb(15, 25, 50))
        .fg(Color::Rgb(180, 200, 220));
    pub const FOOTER_LABEL: Style = Style::new().fg(Color::Rgb(120, 150, 180));
    pub const FOOTER_VALUE: Style = Style::new().fg(Color::White);
}
