use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::{HashSet, VecDeque};

use crate::AppState;
use crate::ui::render_confirmation_overlay;
use crate::ui::theme::ThemeStyles;
use crate::ui::widgets::{Column, ColumnWidth, DataTable, DataTableRow, DataTableState};
use tui_popup::{KnownSizeWrapper, Popup};

// ── DataTable row type ────────────────────────────────────────────────

/// Pre-computed log row data for `DataTable` rendering.
struct LogRow {
    ts: String,
    target: String,
    target_style: Style,
    level: String,
    level_style: Style,
    msg: String,
}

impl DataTableRow for LogRow {
    fn render(
        &self,
        col_xs: &[u16],
        col_widths: &[u16],
        buf: &mut ratatui::buffer::Buffer,
        y: u16,
    ) {
        if col_xs.len() < 4 {
            return;
        }
        buf.set_stringn(
            col_xs[0],
            y,
            &self.ts,
            col_widths[0] as usize,
            Style::default().fg(Color::DarkGray),
        );
        buf.set_stringn(
            col_xs[1],
            y,
            &self.target,
            col_widths[1] as usize,
            self.target_style,
        );
        buf.set_stringn(
            col_xs[2],
            y,
            &self.level,
            col_widths[2] as usize,
            self.level_style,
        );
        buf.set_stringn(
            col_xs[3],
            y,
            &self.msg,
            col_widths[3] as usize,
            Style::default(),
        );
    }

    fn height(&self, available_width: u16) -> u16 {
        let text_width = unicode_width::UnicodeWidthStr::width(self.msg.as_str());
        let msg_width = available_width.saturating_sub(25).max(1);
        1 + (text_width as u16).saturating_sub(1) / msg_width
    }
}
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let palette = state.current_palette();
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(ThemeStyles::container_border(&palette))
        .title_style(ThemeStyles::container_title(&palette));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter bar (1 line)
    let (filter_area, log_area) = {
        let rects = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (rects[0], rects[1])
    };

    let active_count = state.selected_targets.len();
    let sel_active = state.log_select_anchor.is_some();
    let filter_label = if active_count > 0 {
        format!("({active_count} active)")
    } else {
        "(all)".into()
    };
    let filter_text = if sel_active {
        format!(
            " [T] Filter {filter_label}  \u{2191}\u{2193} Select  y Copy  Y All  PgUp/PgDn  Home/End  Esc done",
        )
    } else {
        format!(
            " [T] Filter {filter_label}  \u{2191}\u{2193} Scroll  y Copy  Shift+\u{2191}\u{2193} Select  PgUp/PgDn  Home/End",
        )
    };
    let bar = Line::from(Span::styled(filter_text, ThemeStyles::hint(&palette)));
    frame.render_widget(Paragraph::new(bar), filter_area);

    let log_count = state.log_cache.len();
    if log_count == 0 {
        let empty_msg = if state.log_has_older {
            " Loading logs..."
        } else {
            " No logs yet. Run a profile or wait for core output."
        };
        let paragraph = Paragraph::new(Line::from(empty_msg)).style(ThemeStyles::hint(&palette));
        frame.render_widget(paragraph, log_area);
        return;
    }

    // Filter cache by target
    let mut filtered_indices = Vec::with_capacity(state.log_cache.len());
    filtered_indices.extend(state.log_cache.iter().enumerate().filter_map(|(i, l)| {
        if state.selected_targets.is_empty() || state.selected_targets.contains(&l.target) {
            Some(i)
        } else {
            None
        }
    }));

    let filtered_count = filtered_indices.len();
    if filtered_count == 0 {
        let paragraph = Paragraph::new(Line::from(" No logs match current filter"))
            .style(ThemeStyles::hint(&palette));
        frame.render_widget(paragraph, log_area);
        return;
    }

    // Build DataTable rows from the full filtered set
    let log_rows: Vec<LogRow> = filtered_indices
        .iter()
        .map(|&idx| {
            let log = &state.log_cache[idx];
            let target_style = if log.target.starts_with("xray") {
                Style::default().fg(Color::Cyan)
            } else if log.target.starts_with("sing") {
                Style::default().fg(Color::Green)
            } else if log.target == "tui" {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            let level_style = match log.level.as_str() {
                "error" | "fatal" | "panic" => ThemeStyles::error(&palette),
                "warning" | "warn" => ThemeStyles::warning(&palette),
                "debug" | "trace" => ThemeStyles::hint(&palette),
                _ => Style::default(),
            };
            LogRow {
                ts: fmt_ts(log.timestamp_nanos),
                target: shorten_target(&log.target),
                target_style,
                level: log.level.clone(),
                level_style,
                msg: log.message.clone(),
            }
        })
        .collect();

    let columns = vec![
        Column::new("Time", ColumnWidth::Fixed(25)),
        Column::new("Target", ColumnWidth::Ratio(1)),
        Column::new("Lvl", ColumnWidth::Fixed(5)),
        Column::new("Message", ColumnWidth::Ratio(4)),
    ];

    // Compute offset — during selection, pin viewport so entire range is visible
    let approx_visible = (log_area.height as usize).saturating_sub(2);
    let offset = if let Some(anchor) = state.log_select_anchor {
        let oldest_row = filtered_count.saturating_sub(state.log_scroll.max(anchor) + 1);
        let newest_row = filtered_count.saturating_sub(state.log_scroll.min(anchor) + 1);
        let range_height = newest_row.saturating_sub(oldest_row).saturating_add(1);
        if range_height <= approx_visible {
            oldest_row
        } else {
            filtered_count.saturating_sub(state.log_scroll + approx_visible)
        }
    } else {
        filtered_count.saturating_sub(state.log_scroll + approx_visible)
    };

    // Build multi-selection set from anchor range (offset-from-bottom)
    let mut multi_selected = HashSet::new();
    if let Some(anchor) = state.log_select_anchor {
        let lo = state.log_scroll.min(anchor);
        let hi = state.log_scroll.max(anchor);
        // Convert offset-from-bottom to row indices (0 = oldest)
        let lo_row = filtered_count.saturating_sub(hi + 1);
        let hi_row = filtered_count.saturating_sub(lo + 1);
        for i in lo_row..=hi_row {
            multi_selected.insert(i);
        }
    }

    // Cursor row in filtered index space
    let cursor_row = filtered_count.saturating_sub(state.log_scroll + 1);
    let selected = cursor_row
        .checked_sub(offset)
        .filter(|&s| s < log_rows.len());

    let data_table = DataTable::new(columns, &log_rows)
        .column_spacing(1)
        .selection_style(ThemeStyles::table_row_selected(&palette))
        .scrollbar(
            ThemeStyles::scrollbar_thumb(&palette),
            ThemeStyles::scrollbar_track(&palette),
        );
    let mut dt_state = DataTableState {
        offset,
        selected,
        multi_selected,
    };
    frame.render_stateful_widget(data_table, log_area, &mut dt_state);

    if matches!(state.confirmation, Some(crate::ConfirmAction::ClearLogs)) {
        render_confirmation_overlay(frame, area, " Clear all logs? (y/N) ");
    }
    if matches!(
        state.confirmation,
        Some(crate::ConfirmAction::PurgeLogsDatabase)
    ) {
        render_confirmation_overlay(frame, area, " Purge entire log database? (y/N) ");
    }
}

/// Shorten a target string for display (max ~18 chars).
fn shorten_target(target: &str) -> String {
    if target.len() <= 18 {
        return target.to_string();
    }
    // For "xray::infra::conf::serial" → "xray::infra::con.."
    format!("{}..", &target[..16])
}

/// Handle key events for the Logs tab.
pub async fn handle_key(state: &mut AppState, key: &KeyEvent) {
    let height = state.term_height.get().saturating_sub(5) as usize;
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    match key.code {
        KeyCode::Up if shift => {
            state.log_select_anchor.get_or_insert(state.log_scroll);
            state.log_scroll = state.log_scroll.saturating_add(1);
            try_load_older(state).await;
        }
        KeyCode::Down if shift => {
            state.log_select_anchor.get_or_insert(state.log_scroll);
            state.log_scroll = state.log_scroll.saturating_sub(1);
        }
        KeyCode::Up => {
            state.log_scroll = state.log_scroll.saturating_add(1);
            try_load_older(state).await;
        }
        KeyCode::Down => {
            state.log_scroll = state.log_scroll.saturating_sub(1);
        }
        KeyCode::PageUp => {
            state.log_scroll = state.log_scroll.saturating_add(height);
            try_load_older(state).await;
        }
        KeyCode::PageDown => {
            state.log_scroll = state.log_scroll.saturating_sub(height);
        }
        KeyCode::Home => {
            state.log_scroll = usize::MAX;
            state.log_seek_home = state.log_has_older;
        }
        KeyCode::End => {
            state.log_scroll = 0;
        }
        KeyCode::Char('t' | 'T') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.mode = crate::AppMode::TargetPicker { selected: 0 };
        }
        KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.confirmation = Some(crate::ConfirmAction::ClearLogs);
        }
        KeyCode::Delete => {
            state.confirmation = Some(crate::ConfirmAction::PurgeLogsDatabase);
        }
        KeyCode::Char('y') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if state.log_select_anchor.is_some() {
                copy_selection(state);
            } else {
                copy_cursor_line(state);
            }
        }
        KeyCode::Char('Y') => {
            copy_all_filtered(state);
        }
        KeyCode::Esc => {
            state.log_select_anchor = None;
        }
        _ => {}
    }
}

/// Load older log entries from heed when scrolled near the top (async).
pub(super) async fn try_load_older(state: &mut AppState) {
    if !state.log_has_older {
        return;
    }
    if state.log_cache.is_empty() {
        state.log_has_older = false;
        return;
    }
    let filtered_count = count_filtered(state);
    let height = state.term_height.get().saturating_sub(5) as usize;
    if state.log_scroll < filtered_count.saturating_sub(height) {
        return;
    }

    let oldest_ts = state.log_cache[0].timestamp_nanos;

    let heed = match &state.heed_storage {
        Some(h) => h.clone(),
        None => return,
    };

    let entries = heed.read_older_than_async(oldest_ts as u64, 500).await;

    match entries {
        Ok(entries) => {
            if entries.is_empty() {
                state.log_has_older = false;
                return;
            }
            if entries.len() < 500 {
                state.log_has_older = false;
            }
            let mut new_lines: VecDeque<crate::LogLine> = entries
                .into_iter()
                .rev()
                .map(|e| crate::LogLine {
                    level: e.level,
                    target: e.target,
                    message: e.message,
                    timestamp_nanos: e.timestamp_nanos as i64,
                })
                .collect();
            let n = new_lines.len();
            state.log_scroll = state.log_scroll.saturating_add(n);
            if let Some(ref mut a) = state.log_select_anchor {
                *a = a.saturating_add(n);
            }
            new_lines.append(&mut state.log_cache);
            state.log_cache = new_lines;
            let before = state.log_cache.len();
            while state.log_cache.len() > 10_000 {
                state.log_cache.pop_front();
            }
            let popped = before - state.log_cache.len();
            state.log_scroll = state.log_scroll.saturating_sub(popped);
            if let Some(ref mut a) = state.log_select_anchor {
                *a = a.saturating_sub(popped);
            }
        }
        Err(e) => {
            state.log_has_older = false;
            tracing::error!(target: "tui::ui::logs::load", "Failed to load older logs: {e}");
        }
    }
}

/// Poll heed for new log entries newer than `last_seen_log_ns` (async).
pub(super) async fn poll_new_logs(state: &mut AppState) {
    let heed = match &state.heed_storage {
        Some(h) => h.clone(),
        None => return,
    };

    let entries = match heed
        .read_newer_than_async(state.last_seen_log_ns, 100)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            tracing::error!(target: "tui::ui::logs::poll", "Failed to poll new logs: {e}");
            return;
        }
    };

    if entries.is_empty() {
        return;
    }

    // Update last_seen_log_ns from the newest entry
    if let Some(newest) = entries.first() {
        state.last_seen_log_ns = newest.timestamp_nanos;
    }

    // Append new entries (entries are newest-first, reverse to append in order)
    let new_count = entries.len();
    for entry in entries.into_iter().rev() {
        state.log_cache.push_back(crate::LogLine {
            level: entry.level,
            target: entry.target,
            message: entry.message,
            timestamp_nanos: entry.timestamp_nanos as i64,
        });
    }
    // Move scroll to show new entries if at bottom
    if state.log_scroll == 0 {
        // Scroll is at 0 (bottom), keep it there
    } else {
        // If not at bottom, adjust scroll by new entries count
        state.log_scroll = state.log_scroll.saturating_add(new_count);
        if let Some(ref mut a) = state.log_select_anchor {
            *a = a.saturating_add(new_count);
        }
    }
    let excess = state.log_cache.len().saturating_sub(10_000);
    if excess > 0 {
        state.log_cache.drain(0..excess);
        if state.log_scroll > 0 {
            state.log_scroll = state.log_scroll.saturating_sub(excess);
            if let Some(ref mut a) = state.log_select_anchor {
                *a = a.saturating_sub(excess);
            }
        }
    }
}

/// Count how many log entries match the current target filter.
pub(super) fn count_filtered(state: &AppState) -> usize {
    if state.selected_targets.is_empty() {
        state.log_cache.len()
    } else {
        state
            .log_cache
            .iter()
            .filter(|l| state.selected_targets.contains(&l.target))
            .count()
    }
}

// ── Target Picker ────────────────────────────────────────────────────

/// Render the target picker overlay as centered popup.
pub fn render_target_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    let selected = match &state.mode {
        crate::AppMode::TargetPicker { selected } => *selected,
        _ => return,
    };
    let palette = state.current_palette();

    // Empty state: small centered popup
    if state.known_targets.is_empty() {
        let text = Paragraph::new(Line::from(" No targets available"))
            .style(ThemeStyles::hint(&palette))
            .alignment(Alignment::Center);
        let sized = KnownSizeWrapper::new(text, 30, 3);
        let popup = Popup::new(sized)
            .title(" Select Targets (t=done, Enter=toggle) ")
            .border_set(ratatui::symbols::border::ROUNDED)
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));
        frame.render_widget(popup, area);
        return;
    }

    let max_target_width = state
        .known_targets
        .iter()
        .map(std::string::String::len)
        .max()
        .unwrap_or(0);
    let popup_width = (max_target_width + 8).max(36) as u16;
    let popup_height = (state.known_targets.len() + 2).min(20) as u16;

    let items: Vec<Line> = state
        .known_targets
        .iter()
        .enumerate()
        .map(|(i, target)| {
            let checked = if state.selected_targets.contains(target) {
                "[\u{2713}]" // [✓]
            } else {
                "[ ]"
            };
            let prefix = if i == selected { " > " } else { "   " };
            Line::from(Span::styled(
                format!("{prefix}{checked} {target}"),
                if i == selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            ))
        })
        .collect();

    let para = Paragraph::new(items).scroll((selected.saturating_sub(5) as u16, 0));
    let sized = KnownSizeWrapper::new(para, popup_width as usize, popup_height as usize);
    let popup = Popup::new(sized)
        .title(" Select Targets (t=done, Enter=toggle) ")
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));
    frame.render_widget(popup, area);
}

/// Handle key events for the target picker overlay.
pub fn handle_target_picker_key(state: &mut AppState, key: &KeyEvent) {
    let selected = match &state.mode {
        crate::AppMode::TargetPicker { selected } => *selected,
        _ => return,
    };

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            let new_sel = selected.saturating_sub(1);
            state.mode = crate::AppMode::TargetPicker { selected: new_sel };
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let new_sel = if state.known_targets.is_empty() {
                0
            } else {
                (selected + 1).min(state.known_targets.len() - 1)
            };
            state.mode = crate::AppMode::TargetPicker { selected: new_sel };
        }
        KeyCode::Enter => {
            // Toggle the selected target
            if let Some(target) = state.known_targets.get(selected) {
                if let Some(pos) = state.selected_targets.iter().position(|t| t == target) {
                    state.selected_targets.remove(pos);
                } else {
                    state.selected_targets.push(target.clone());
                }
            }
        }
        KeyCode::Char('t' | 'T' | 'q' | 'Q') | KeyCode::Esc => {
            state.mode = crate::AppMode::List;
        }
        _ => {}
    }
}

// ── Copy —───────────────────────────────────────────────────────────

/// Copy the selected log range to the system clipboard.
fn copy_selection(state: &AppState) {
    let Some(anchor) = state.log_select_anchor else {
        return copy_cursor_line(state);
    };
    if state.log_cache.is_empty() {
        return;
    }
    let filtered_indices: Vec<usize> = state
        .log_cache
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if state.selected_targets.is_empty() || state.selected_targets.contains(&l.target) {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    let filtered_count = filtered_indices.len();
    if filtered_count == 0 {
        return;
    }
    let lo = state.log_scroll.min(anchor);
    let hi = state.log_scroll.max(anchor);
    // Convert offset-from-bottom to filtered indices
    let lo_fi = filtered_count.saturating_sub(hi + 1);
    let hi_fi = filtered_count.saturating_sub(lo + 1);
    if lo_fi > hi_fi || lo_fi >= filtered_count {
        return;
    }
    let hi_fi = hi_fi.min(filtered_count - 1);
    let lines: Vec<String> = filtered_indices[lo_fi..=hi_fi]
        .iter()
        .filter_map(|&ci| state.log_cache.get(ci))
        .map(|log| {
            format!(
                "{} [{}] [{}] {}",
                fmt_ts(log.timestamp_nanos),
                log.level,
                log.target,
                log.message,
            )
        })
        .collect();
    if lines.is_empty() {
        return;
    }
    let text = lines.join("\n");
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

/// Copy the log line under the cursor to the system clipboard.
fn copy_cursor_line(state: &AppState) {
    if state.log_cache.is_empty() {
        return;
    }
    let filtered_indices: Vec<usize> = state
        .log_cache
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if state.selected_targets.is_empty() || state.selected_targets.contains(&l.target) {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    let filtered_count = filtered_indices.len();
    if filtered_count == 0 {
        return;
    }
    let cursor_idx = filtered_count.saturating_sub(state.log_scroll + 1);
    let Some(&cache_idx) = filtered_indices.get(cursor_idx) else {
        return;
    };
    let Some(log) = state.log_cache.get(cache_idx) else {
        return;
    };
    let text = format!(
        "{} [{}] [{}] {}",
        fmt_ts(log.timestamp_nanos),
        log.level,
        log.target,
        log.message,
    );
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

/// Copy ALL filtered log entries to the system clipboard.
fn copy_all_filtered(state: &AppState) {
    if state.log_cache.is_empty() {
        return;
    }
    let lines: Vec<String> = state
        .log_cache
        .iter()
        .filter(|l| state.selected_targets.is_empty() || state.selected_targets.contains(&l.target))
        .map(|log| {
            format!(
                "{} [{}] [{}] {}",
                fmt_ts(log.timestamp_nanos),
                log.level,
                log.target,
                log.message,
            )
        })
        .collect();
    if lines.is_empty() {
        return;
    }
    let text = lines.join("\n");
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn fmt_ts(ts_nanos: i64) -> String {
    let secs = if ts_nanos >= 0 {
        ts_nanos / 1_000_000_000
    } else {
        (ts_nanos - 999_999_999) / 1_000_000_000
    };
    let sub_nanos = ts_nanos.rem_euclid(1_000_000_000) as u32;
    chrono::DateTime::from_timestamp(secs, sub_nanos).map_or_else(
        || format!("{}s", ts_nanos / 1_000_000_000),
        |dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
    )
}
