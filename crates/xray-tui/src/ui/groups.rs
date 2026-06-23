use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::{AppMode, AppState, ConfirmAction};
use xray_tui_db::models::Group;
use crate::ui::theme::Theme;
use crate::ui::profiles::truncate_pad;
use crate::ui::render_confirmation_overlay;
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
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
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

    // Scrollable group list — system groups first
    let list_area = chunks[0];
    let mut sorted_groups: Vec<(usize, &Group)> = state.groups.iter().enumerate().collect();
    sorted_groups.sort_by_key(|(_, g)| {
        // System groups first (is_system == 1 → sort_key 0), then user groups
        let is_sys = g.is_system.unwrap_or(0);
        (1 - is_sys, g.name.as_deref().unwrap_or(""))
    });
    let rows: Vec<Line> = sorted_groups
        .iter()
        .map(|(orig_idx, g)| {
            let is_system = g.is_system.unwrap_or(0) == 1;
            let is_selected = *orig_idx == selected;
            let name = g.name.as_deref().unwrap_or("unnamed");
            let url = g.subscription_url.as_deref().unwrap_or("-");
            let enabled = if g.subscription_enabled.unwrap_or(0) == 1 {
                "Y"
            } else {
                "N"
            };
            let status = state.subscriptions.iter()
                .find(|s| s.group_id.as_deref() == Some(&g.id))
                .and_then(|s| s.status.as_deref())
                .unwrap_or("idle");
            let last_up = "-";

            let display_name = if is_system {
                format!("[system] {name}")
            } else {
                name.to_string()
            };

            let style = if is_selected {
                if is_system {
                    Style::default().bg(Color::Rgb(40, 50, 70)).fg(Color::Rgb(180, 200, 220))
                } else {
                    Style::default().bg(Color::Blue).fg(Color::White)
                }
            } else {
                if is_system {
                    Style::default().fg(Color::Rgb(100, 120, 140))
                } else {
                    Style::default()
                }
            };
            Line::from(vec![Span::styled(
                format!(
                    " {:<27} │ {:<30} │ {} │ {:<10} │ {}",
                    truncate_pad(&display_name, 27), truncate_pad(url, 30), enabled, status, last_up
                ),
                style,
            )])
        })
        .collect();

    let scroll_offset = selected.saturating_sub(list_area.height as usize - 1);
    for (i, row) in rows.iter().enumerate().skip(scroll_offset).take(list_area.height as usize) {
        frame.render_widget(
            row.clone(),
            Rect::new(list_area.x, list_area.y + (i - scroll_offset) as u16, list_area.width, 1),
        );
    }

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        " [a] Add  [e] Edit  [d] Delete  [u] Update  [Shift+U] Update All  [Enter] Filter  [Esc] Close ",
        Theme::HINT,
    )));
    frame.render_widget(footer, chunks[1]);

    // Confirmation overlays: DeleteGroup and ClearGroup
    match state.confirmation {
        Some(ConfirmAction::DeleteGroup(ref group_id)) => {
            let group_name = state
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .and_then(|g| g.name.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(frame, area, &format!(" Delete \"{group_name}\"? (y/N) "));
        }
        Some(ConfirmAction::ClearGroup(ref group_id)) => {
            let group_name = state
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .and_then(|g| g.name.as_deref())
                .unwrap_or("unknown");
            render_confirmation_overlay(frame, area, &format!(" Clear all profiles in \"{group_name}\"? (y/N) "));
        }
        _ => {}
    }
}

pub fn render_group_form(frame: &mut Frame, area: Rect, state: &AppState, _editing: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Group ")
        .border_style(Theme::CONTAINER_BORDER)
        .title_style(Theme::CONTAINER_TITLE);
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
        Theme::HINT,
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
                    let is_system = state.groups.get(sel).map(|g| g.is_system == Some(1)).unwrap_or(false);
                    if !is_system {
                        let gid = state.groups.get(sel).map(|g| g.id.clone());
                        if let Some(id) = gid {
                            state.start_edit_group(&id);
                        }
                    }
                }
                KeyCode::Char('a' | 'A') => {
                    state.start_add_group();
                }
                KeyCode::Char('d' | 'D') => {
                    let is_system = state.groups.get(sel).map(|g| g.is_system == Some(1)).unwrap_or(false);
                    if !is_system {
                        let gid = state.groups.get(sel).map(|g| g.id.clone());
                        if let Some(id) = gid {
                            state.confirmation = Some(ConfirmAction::DeleteGroup(id));
                        }
                    }
                }
                KeyCode::Char('c' | 'C') => {
                    let is_system = state.groups.get(sel).map(|g| g.is_system == Some(1)).unwrap_or(false);
                    if !is_system {
                        let gid = state.groups.get(sel).map(|g| g.id.clone());
                        if let Some(id) = gid {
                            state.confirmation = Some(ConfirmAction::ClearGroup(id));
                        }
                    }
                }
                KeyCode::Char('u') => {
                    let is_system = state.groups.get(sel).map(|g| g.is_system == Some(1)).unwrap_or(false);
                    if !is_system {
                        let gid = state.groups.get(sel).map(|g| g.id.clone());
                        if let Some(id) = gid {
                            state.update_group_subscriptions(&id);
                        }
                    }
                }
                KeyCode::Char('U') => {
                    state.update_all_subscriptions();
                }
                KeyCode::Enter => {
                    if let Some(g) = state.groups.get(sel) {
                        state.selected_group_id = Some(g.id.clone());
                        state.filter_cache_valid.set(false);
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
