use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

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

    // Filter bar (1 line)
    let (filter_area, log_area) = {
        let rects = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (rects[0], rects[1])
    };

    let active_count = state.selected_targets.len();
    let filter_text = if active_count > 0 {
        format!(
            " [T] Filter  ({active_count} active)  \u{2191}\u{2193} Scroll  PgUp/PgDn  Home/End",
        )
    } else {
        " [T] Filter  (all)  \u{2191}\u{2193} Scroll  PgUp/PgDn  Home/End".to_string()
    };
    let bar = Line::from(Span::styled(filter_text, Theme::HINT));
    frame.render_widget(Paragraph::new(bar), filter_area);

    let log_count = state.log_cache.len();
    if log_count == 0 {
        let empty_msg = if state.log_has_older {
            " Loading logs..."
        } else {
            " No logs yet. Run a profile or wait for core output."
        };
        let paragraph = Paragraph::new(Line::from(empty_msg)).style(Theme::HINT);
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
        let paragraph =
            Paragraph::new(Line::from(" No logs match current filter")).style(Theme::HINT);
        frame.render_widget(paragraph, log_area);
        return;
    }

    // Clamp scroll: 0 = most recent at bottom
    let scroll = state.log_scroll.min(filtered_count.saturating_sub(1));

    // Visible range
    let height = (log_area.height as usize).saturating_sub(2); // 2 for header row
    let visible_start = filtered_count.saturating_sub(scroll + height);
    let visible_end = filtered_count.saturating_sub(scroll);
    let visible_indices = &filtered_indices[visible_start..visible_end];

    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;
    let hour_ns: i64 = 3_600_000_000_000;

    // Header
    let header_cells = ["Timestamp", "Target", "Level", "Message"]
        .into_iter()
        .map(|h| Cell::from(h).style(Theme::TABLE_HEADER));
    let header = Row::new(header_cells);

    let widths = [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Min(10),
    ];

    let rows: Vec<Row> = visible_indices
        .iter()
        .map(|&idx| {
            let log = &state.log_cache[idx];
            let ts_str = fmt_ts(log.timestamp_nanos, now_nanos, hour_ns);
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
                "error" | "fatal" | "panic" => Theme::ERROR,
                "warning" | "warn" => Theme::WARNING,
                "debug" | "trace" => Theme::HINT,
                _ => Style::default(),
            };
            // Truncate target for display
            let target_display = shorten_target(&log.target);
            Row::new(vec![
                Cell::from(Span::styled(ts_str, Style::default().fg(Color::DarkGray))),
                Cell::from(Span::styled(target_display, target_style)),
                Cell::from(Span::styled(&log.level, level_style)),
                Cell::from(Span::styled(&log.message, Style::default())),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default());
    frame.render_widget(table, log_area);
}

/// Shorten a target string for display (max ~10 chars).
fn shorten_target(target: &str) -> String {
    if target.len() <= 10 {
        return target.to_string();
    }
    // For "xray::infra::conf::serial" → "xry::inf.."
    format!("{}..", &target[..8])
}

/// Handle key events for the Logs tab.
pub fn handle_key(state: &mut AppState, key: &KeyEvent) {
    let height = state.term_height.get().saturating_sub(5) as usize;
    match key.code {
        KeyCode::Up => {
            state.log_scroll = state.log_scroll.saturating_add(1);
            try_load_older(state);
        }
        KeyCode::Down => {
            state.log_scroll = state.log_scroll.saturating_sub(1);
        }
        KeyCode::PageUp => {
            state.log_scroll = state.log_scroll.saturating_add(height);
            try_load_older(state);
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
            let _ = crate::AppMode::TargetPicker { selected: 0 };
            state.mode = crate::AppMode::TargetPicker { selected: 0 };
        }
        _ => {}
    }
}

/// Load older log entries from heed when scrolled near the top.
pub(super) fn try_load_older(state: &mut AppState) {
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
        Some(h) => h,
        None => return,
    };

    let entries = heed.read_older_than(oldest_ts as u64, 500);

    match entries {
        Ok(entries) => {
            if entries.is_empty() {
                state.log_has_older = false;
                return;
            }
            if entries.len() < 500 {
                state.log_has_older = false;
            }
            let new_lines: Vec<crate::LogLine> = entries
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
            let mut combined = new_lines;
            combined.append(&mut state.log_cache);
            state.log_cache = combined;
            state.log_cache.truncate(10_000);
        }
        Err(e) => {
            state.log_has_older = false;
            tracing::error!(target: "log_worker", "Failed to load older logs: {e}");
        }
    }
}

/// Poll heal for new log entries newer than `last_seen_log_ns`.
pub(super) fn poll_new_logs(state: &mut AppState) {
    let heed = match &state.heed_storage {
        Some(h) => h,
        None => return,
    };

    let entries = match heed.read_newer_than(state.last_seen_log_ns, 100) {
        Ok(e) => e,
        Err(e) => {
            tracing::error!(target: "log_worker", "Failed to poll new logs: {e}");
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
        state.log_cache.push(crate::LogLine {
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
    }
    state.log_cache.truncate(10_000);
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

/// Render the target picker overlay.
pub fn render_target_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    let selected = match &state.mode {
        crate::AppMode::TargetPicker { selected } => *selected,
        _ => return,
    };

    let overlay = Block::default()
        .title(" Select Targets (t=done, Enter=toggle) ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    let inner = overlay.inner(area);
    frame.render_widget(overlay, area);

    if state.known_targets.is_empty() {
        let text = Paragraph::new(Line::from(" No targets available"))
            .style(Theme::HINT)
            .alignment(Alignment::Center);
        frame.render_widget(text, inner);
        return;
    }

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

    let list = Paragraph::new(items)
        .block(Block::default())
        .scroll((selected.saturating_sub(5) as u16, 0));
    frame.render_widget(list, inner);
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

// ── Helpers ──────────────────────────────────────────────────────────

fn fmt_ts(ts_nanos: i64, now_nanos: i64, hour_ns: i64) -> String {
    let secs = if ts_nanos >= 0 {
        ts_nanos / 1_000_000_000
    } else {
        (ts_nanos - 999_999_999) / 1_000_000_000
    };
    let sub_nanos = ts_nanos.rem_euclid(1_000_000_000) as u32;
    if let Some(naive) = chrono::DateTime::from_timestamp(secs, sub_nanos).map(|dt| dt.naive_utc())
    {
        let diff = now_nanos - ts_nanos;
        if diff >= 0 && diff < hour_ns {
            naive.format("%H:%M:%S").to_string()
        } else {
            let now_secs = if now_nanos >= 0 {
                now_nanos / 1_000_000_000
            } else {
                (now_nanos - 999_999_999) / 1_000_000_000
            };
            let now_sub = now_nanos.rem_euclid(1_000_000_000) as u32;
            let now_naive = chrono::DateTime::from_timestamp(now_secs, now_sub)
                .map_or(naive, |dt| dt.naive_utc());
            if naive.year() == now_naive.year() {
                naive.format("%m/%d %H:%M").to_string()
            } else {
                naive.format("%Y-%m-%d %H:%M").to_string()
            }
        }
    } else {
        format!("{}s", ts_nanos / 1_000_000_000)
    }
}
