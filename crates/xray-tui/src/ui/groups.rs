use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::{AppMode, AppState, ConfirmAction};

pub fn render_group_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    // Centered overlay: ~70% width, ~70% height
    let overlay_area = Rect::new(
        area.width.saturating_sub(area.width * 70 / 100) / 2,
        area.height.saturating_sub(area.height * 70 / 100) / 2,
        area.width * 70 / 100,
        area.height * 70 / 100,
    );

    let selected = match &state.mode {
        AppMode::ManageGroups { selected } => *selected,
        _ => 0,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Groups ")
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Table header
    let header = Line::from(vec![
        Span::styled(
            " Name ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│"),
        Span::styled(
            " URL ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│"),
        Span::styled(
            " Ena ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│"),
        Span::styled(
            " Status ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│"),
        Span::styled(
            " Last Updated ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(header, chunks[0]);

    // Scrollable group list
    let list_area = chunks[0];
    let rows: Vec<Line> = state
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let name = g.name.as_deref().unwrap_or("unnamed");
            let url = g.subscription_url.as_deref().unwrap_or("-");
            let enabled = if g.subscription_enabled.unwrap_or(0) == 1 {
                "Y"
            } else {
                "N"
            };
            let status = "idle"; // TODO: look up from subscriptions table
            let last_up = "-";

            let style = if i == selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            Line::from(vec![Span::styled(
                format!(
                    " {:<20} │ {:<30} │ {} │ {:<10} │ {}",
                    name, url, enabled, status, last_up
                ),
                style,
            )])
        })
        .collect();

    for (i, row) in rows.iter().enumerate() {
        if i < list_area.height as usize {
            frame.render_widget(
                row.clone(),
                Rect::new(list_area.x, list_area.y + i as u16, list_area.width, 1),
            );
        }
    }

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        " [a] Add  [e] Edit  [d] Delete  [u] Update  [Shift+U] Update All  [Enter] Filter  [Esc] Close ",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, chunks[1]);
}

pub fn render_group_form(frame: &mut Frame, area: Rect, state: &AppState, _editing: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Group ")
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (fields, focus_index) = match &state.mode {
        AppMode::AddGroup {
            fields,
            focus_index,
        }
        | AppMode::EditGroup {
            fields,
            focus_index,
            ..
        } => (fields.clone(), *focus_index),
        _ => return,
    };

    let keys = [
        "name",
        "subscription_url",
        "user_agent",
        "update_interval",
        "core_type",
    ];
    let max_label_len = keys.iter().map(|k| k.len()).max().unwrap_or(10);

    for (i, key) in keys.iter().enumerate() {
        let value = fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let is_focused = i == focus_index;

        let label_style = if is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let value_style = if is_focused {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let label = format!(" {:<width$}", key, width = max_label_len);
        let display = format!("{} ", value);
        let line = Line::from(vec![
            Span::styled(label, label_style),
            Span::styled(display, value_style),
        ]);
        frame.render_widget(line, Rect::new(inner.x, inner.y + i as u16, inner.width, 1));
    }

    // Hint
    let hint = Paragraph::new(Line::from(Span::styled(
        " Tab/S-Tab: navigate  Enter: save  Esc: cancel ",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(
        hint,
        Rect::new(inner.x, inner.y + keys.len() as u16, inner.width, 1),
    );
}

pub fn handle_key(state: &mut AppState, key: &KeyEvent) {
    match &state.mode {
        AppMode::ManageGroups { selected } => {
            let mut sel = *selected;
            match key.code {
                KeyCode::Up => {
                    sel = sel.saturating_sub(1);
                    state.mode = AppMode::ManageGroups { selected: sel };
                }
                KeyCode::Down => {
                    let max = state.groups.len().saturating_sub(1);
                    if sel < max {
                        sel += 1;
                    }
                    state.mode = AppMode::ManageGroups { selected: sel };
                }
                KeyCode::Char('e' | 'E') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.start_edit_group(&id);
                    }
                }
                KeyCode::Char('a' | 'A') => {
                    state.start_add_group();
                }
                KeyCode::Char('d' | 'D') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.confirmation = Some(ConfirmAction::DeleteGroup(id));
                    }
                }
                KeyCode::Char('u' | 'U') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.update_group_subscriptions(&id);
                    }
                }
                KeyCode::Enter => {
                    if let Some(g) = state.groups.get(sel) {
                        state.selected_group_id = Some(g.id.clone());
                    }
                    state.mode = AppMode::List;
                }
                KeyCode::Esc => {
                    state.mode = AppMode::List;
                }
                _ => {}
            }
        }
        AppMode::AddGroup {
            fields,
            focus_index,
        }
        | AppMode::EditGroup {
            fields,
            focus_index,
            ..
        } => {
            let mut flds = fields.clone();
            let mut fi = *focus_index;
            let keys = [
                "name",
                "subscription_url",
                "user_agent",
                "update_interval",
                "core_type",
            ];

            match key.code {
                KeyCode::Tab => {
                    fi = (fi + 1) % keys.len();
                }
                KeyCode::BackTab => {
                    fi = if fi == 0 { keys.len() - 1 } else { fi - 1 };
                }
                KeyCode::Char(c) => {
                    let key = keys[fi];
                    if let Some((_, val)) = flds.iter_mut().find(|(k, _)| k == key) {
                        val.push(c);
                    }
                }
                KeyCode::Backspace => {
                    let key = keys[fi];
                    if let Some((_, val)) = flds.iter_mut().find(|(k, _)| k == key) {
                        val.pop();
                    }
                }
                KeyCode::Enter => {
                    match &state.mode {
                        AppMode::AddGroup { .. } => state.confirm_add_group(),
                        AppMode::EditGroup { .. } => state.confirm_edit_group(),
                        _ => {}
                    }
                    return;
                }
                KeyCode::Esc => {
                    state.mode = AppMode::List;
                    return;
                }
                _ => {}
            }

            // Update focus index and fields in mode
            match &state.mode {
                AppMode::AddGroup { .. } => {
                    state.mode = AppMode::AddGroup {
                        fields: flds,
                        focus_index: fi,
                    };
                }
                AppMode::EditGroup { group_id, .. } => {
                    state.mode = AppMode::EditGroup {
                        group_id: group_id.clone(),
                        fields: flds,
                        focus_index: fi,
                    };
                }
                _ => {}
            }
        }
        _ => {}
    }
}
