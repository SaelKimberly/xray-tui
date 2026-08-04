#![allow(clippy::must_use_candidate, clippy::missing_const_for_fn)]
use ratatui::style::{Modifier, Style};
use ratatui_cheese::theme::Palette;

/// Palette-derived style methods for screen modules.
///
/// Each method maps a semantic role to the closest [`Palette`] color.
/// These are progressively eliminated as screens migrate to native
/// ratatui-cheese widget styles (via `from_palette()`).
pub struct ThemeStyles;

impl ThemeStyles {
    // ── Tab bar ─────────────────────────────────────────────────────────
    pub fn tab_selected(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD)
    }
    pub fn tab_deselected(palette: &Palette) -> Style {
        Style::default().fg(palette.muted)
    }

    // ── Table / profile grid ────────────────────────────────────────────
    pub fn table_header(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.foreground)
            .bg(palette.surface)
            .add_modifier(Modifier::BOLD)
    }
    pub fn table_row_selected(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.foreground)
            .bg(palette.surface)
            .add_modifier(Modifier::BOLD)
    }
    pub fn table_row_normal(palette: &Palette) -> Style {
        Style::default().fg(palette.foreground)
    }
    pub fn table_row_connected(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.foreground)
            .bg(palette.highlight)
            .add_modifier(Modifier::BOLD)
    }
    /// Selected protocol sub-row inside an expanded endpoint panel — a
    /// REVERSE highlight. The expanded panel sits on the endpoint row's
    /// highlight background (`surface`), so the selected sub-row drops back
    /// to the common top-level table background (explicit `Color::Reset` —
    /// `Cell::set_style` merges, so a bg-less style would leave `surface`
    /// in place): the whole expandable reads as a highlighted block with one
    /// "notch" for the active protocol. Works under every palette (a light
    /// `on_highlight` would hide light text).
    pub fn panel_row_selected(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.foreground)
            .bg(ratatui::style::Color::Reset)
            .add_modifier(Modifier::BOLD)
    }

    // ── Profiles Test column ────────────────────────────────────────────
    /// Delay under the warning threshold: reachable, fast server.
    pub fn test_delay_ok(palette: &Palette) -> Style {
        Style::default().fg(palette.success)
    }
    /// Delay at/above the warning threshold (500ms).
    pub fn test_delay_warn(_palette: &Palette) -> Style {
        Style::default().fg(ratatui::style::Color::Yellow)
    }
    /// Unreachable / delay at/above the error threshold (1000ms).
    pub fn test_delay_bad(palette: &Palette) -> Style {
        Style::default().fg(palette.error)
    }

    // ── Containers / borders ────────────────────────────────────────────
    pub fn container_border(palette: &Palette) -> Style {
        Style::default().fg(palette.border)
    }
    pub fn container_title(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.secondary)
            .add_modifier(Modifier::BOLD)
    }

    // ── Status bar ──────────────────────────────────────────────────────
    pub fn status_bar_bg(palette: &Palette) -> Style {
        Style::default().bg(palette.surface).fg(palette.foreground)
    }
    pub fn status_bar_mode(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD)
    }

    // ── Feedback / progress ────────────────────────────────────────────
    pub fn progress_bar(palette: &Palette) -> Style {
        Style::default().fg(palette.foreground).bg(palette.surface)
    }
    pub fn progress_fill(palette: &Palette) -> Style {
        Style::default().fg(palette.primary).bg(palette.primary)
    }
    pub fn spinner(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.primary)
            .add_modifier(Modifier::BOLD)
    }

    // ── Semantic ───────────────────────────────────────────────────────
    pub fn error(palette: &Palette) -> Style {
        Style::default()
            .fg(palette.error)
            .add_modifier(Modifier::BOLD)
    }
    pub fn warning(palette: &Palette) -> Style {
        Style::default().fg(palette.primary)
    }
    pub fn success(palette: &Palette) -> Style {
        Style::default().fg(palette.success)
    }
    pub fn hint(palette: &Palette) -> Style {
        Style::default().fg(palette.muted)
    }
    pub fn status_footer(palette: &Palette) -> Style {
        Style::default().bg(palette.surface).fg(palette.foreground)
    }
    pub fn footer_label(palette: &Palette) -> Style {
        Style::default().fg(palette.muted)
    }
    pub fn footer_value(palette: &Palette) -> Style {
        Style::default().fg(palette.foreground)
    }

    // ── Scrollbar ─────────────────────────────────────────────────────
    pub fn scrollbar_thumb(palette: &Palette) -> Style {
        Style::default().fg(palette.primary).bg(palette.surface)
    }
    pub fn scrollbar_track(palette: &Palette) -> Style {
        Style::default().bg(palette.surface)
    }
}
