use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use ratatui_cheese::fieldset::{Fieldset, FieldsetFill};
use ratatui_cheese::input::{Input, InputState};
use ratatui_cheese::list::{List, ListItem, ListItemContext, ListState};

use tui_popup::{KnownSizeWrapper, Popup};

use crate::ui::profiles::truncate_pad;
use crate::ui::theme::ThemeStyles;
use crate::{AppMode, AppState, ConfirmAction};
use xray_tui_db::models::Group;

// ---------------------------------------------------------------------------
// GroupListItem — implements ListItem for rendering in the List widget
// ---------------------------------------------------------------------------

struct GroupListItem {
    display_name: String,
    status: String,
}

impl ListItem for GroupListItem {
    fn height(&self) -> u16 {
        1
    }
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &ListItemContext) {
        let style = if ctx.selected {
            ThemeStyles::table_row_selected(&ctx.palette)
        } else {
            ThemeStyles::table_row_normal(&ctx.palette)
        };
        let text = format!(
            " {:<27} │ {:<10} │ {}",
            truncate_pad(&self.display_name, 27),
            self.status,
            "-",
        );
        buf.set_string(area.x, area.y, &text, style);
    }
}
// ---------------------------------------------------------------------------

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
    let palette = state.current_palette();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Groups ")
        .border_style(ThemeStyles::container_border(&palette))
        .title_style(ThemeStyles::container_title(&palette));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Build sorted group items
    let mut sorted_groups: Vec<(usize, &Group)> = state.groups.iter().enumerate().collect();
    sorted_groups.sort_by_key(|(_, g)| {
        g.name.as_deref().unwrap_or("").to_string()
    });

    let items: Vec<GroupListItem> = sorted_groups
        .iter()
        .map(|(_, g)| {
            let name = g.name.as_deref().unwrap_or("unnamed");
            let status = g.status.as_deref().unwrap_or("idle");
            GroupListItem {
                display_name: name.to_string(),
                status: status.to_string(),
            }
        })
        .collect();

    // Header + list area
    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(chunks[0]);

    // Table header
    let header = Line::from(vec![
        Span::styled(" Name ", ThemeStyles::table_header(&palette)),
        Span::raw("│"),
        Span::styled(" Status ", ThemeStyles::table_header(&palette)),
    ]);
    frame.render_widget(header, list_chunks[0]);
    let list_area = list_chunks[1];
    let list = List::new(&items);
    let mut list_state = ListState::default();
    list_state.select(selected, items.len());
    frame.render_stateful_widget(list, list_area, &mut list_state);
    let bottom_chunk = chunks[1];
    let hints = Line::from(vec![
        Span::raw(" "),
        Span::styled("Enter", ThemeStyles::hint(&palette)),
        Span::styled("=switch  ", ThemeStyles::hint(&palette)),
        Span::styled("c", ThemeStyles::hint(&palette)),
        Span::styled("=clear  ", ThemeStyles::hint(&palette)),
        Span::styled("u", ThemeStyles::hint(&palette)),
        Span::styled("=update  ", ThemeStyles::hint(&palette)),
        Span::styled("d", ThemeStyles::hint(&palette)),
        Span::styled("=delete  ", ThemeStyles::hint(&palette)),
        Span::styled("Esc", ThemeStyles::hint(&palette)),
        Span::styled("=back", ThemeStyles::hint(&palette)),
    ]);
    frame.render_widget(hints, bottom_chunk);
    // Confirmation popups
    if let Some(action) = &state.confirmation {
        match action {
            ConfirmAction::DeleteGroup(group_id) => {
                let group_name = state
                    .groups
                    .iter()
                    .find(|g| g.id == *group_id)
                    .and_then(|g| g.name.as_deref())
                    .unwrap_or("unknown");
                let text = format!(" Delete \"{group_name}\"? (y/N) ");
                let width = text.len();
                let para = Paragraph::new(text).alignment(Alignment::Center).style(
                    Style::new()
                        .fg(palette.foreground)
                        .bg(palette.surface)
                        .add_modifier(Modifier::BOLD),
                );
                let sized = KnownSizeWrapper {
                    inner: para,
                    width,
                    height: 1,
                };
                let popup = Popup::new(sized)
                    .title(" Confirm ")
                    .style(Style::new().bg(palette.surface))
                    .border_set(border::ROUNDED)
                    .border_style(Style::new().fg(palette.error).add_modifier(Modifier::BOLD));
                frame.render_widget(popup, area);
            }
            ConfirmAction::ClearGroup(group_id) => {
                let group_name = state
                    .groups
                    .iter()
                    .find(|g| g.id == *group_id)
                    .and_then(|g| g.name.as_deref())
                    .unwrap_or("unknown");
                let text = format!(" Clear all profiles in \"{group_name}\"? (y/N) ");
                let width = text.len();
                let para = Paragraph::new(text).alignment(Alignment::Center).style(
                    Style::new()
                        .fg(palette.foreground)
                        .bg(palette.surface)
                        .add_modifier(Modifier::BOLD),
                );
                let sized = KnownSizeWrapper {
                    inner: para,
                    width,
                    height: 1,
                };
                let popup = Popup::new(sized)
                    .title(" Confirm ")
                    .style(Style::new().bg(palette.surface))
                    .border_set(border::ROUNDED)
                    .border_style(Style::new().fg(palette.error).add_modifier(Modifier::BOLD));
                frame.render_widget(popup, area);
            }
            _ => {}
        }
    }
}
pub fn render_group_form(frame: &mut Frame, area: Rect, state: &AppState, _editing: bool) {
    let palette = state.current_palette();
    let fieldset = Fieldset::new()
        .title(" Group ")
        .fill(FieldsetFill::Dash)
        .palette(&palette);
    let inner = fieldset.inner(area);
    frame.render_widget(&fieldset, area);

    let (fields, focus_index) = match &state.mode {
        AppMode::AddGroup {
            fields,
            focus_index,
        }
        | AppMode::EditGroup {
            fields,
            focus_index,
            ..
        } => (fields, *focus_index),
        _ => return,
    };

    let keys = [
        "name",
        "subscription_url",
        "user_agent",
        "update_interval",
        "core_type",
    ];

    // 2 lines per field (title + input line)
    let field_height = 2u16;
    for (i, key) in keys.iter().enumerate() {
        let is_focused = i == focus_index;
        let value = fields
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str());

        let field_area = Rect::new(
            inner.x,
            inner.y + i as u16 * field_height,
            inner.width,
            field_height,
        );

        let input = Input::new(key).palette(&palette).prompt("");
        let mut input_state = InputState::new();
        input_state.set_value(value.to_string());
        input_state.set_focused(is_focused);
        frame.render_stateful_widget(input, field_area, &mut input_state);
    }

    // Hint
    let hint_y = inner.y + keys.len() as u16 * field_height;
    let hint = Paragraph::new(Line::from(Span::styled(
        " Tab/S-Tab: navigate  Enter: save  Esc: cancel ",
        ThemeStyles::hint(&palette),
    )));
    frame.render_widget(hint, Rect::new(inner.x, hint_y, inner.width, 1));
}

// ---------------------------------------------------------------------------
// handle_key (unchanged — rendering only)
// ---------------------------------------------------------------------------

pub async fn handle_key(state: &mut AppState, key: &KeyEvent) {
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
                KeyCode::Home => {
                    state.mode = AppMode::ManageGroups { selected: 0 };
                }
                KeyCode::End => {
                    let last = state.groups.len().saturating_sub(1);
                    state.mode = AppMode::ManageGroups { selected: last };
                }
                KeyCode::Char('e' | 'E') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.start_edit_group(&id);
                    }
                }
                KeyCode::Char('d' | 'D') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.confirmation = Some(ConfirmAction::DeleteGroup(id));
                    }
                }
                KeyCode::Char('c' | 'C') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.confirmation = Some(ConfirmAction::ClearGroup(id));
                    }
                }
                KeyCode::Char('u') => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.update_group_subscriptions(&id);
                    }
                }
                KeyCode::Char('U') => {
                    state.update_all_subscriptions();
                }
                KeyCode::Enter => {
                    let gid = state.groups.get(sel).map(|g| g.id.clone());
                    if let Some(id) = gid {
                        state.selected_group_id = Some(id);
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
        AppMode::AddGroup { .. } | AppMode::EditGroup { .. } => {
            // Enter/Esc handled first — need &state before any &mut borrow
            match key.code {
                KeyCode::Enter => {
                    match &state.mode {
                        AppMode::AddGroup { .. } => state.confirm_add_group().await,
                        AppMode::EditGroup { .. } => state.confirm_edit_group().await,
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

            // Now safe to &mut borrow state.mode for in-place edits
            let (fields, focus_index) = match &mut state.mode {
                AppMode::AddGroup {
                    fields,
                    focus_index,
                }
                | AppMode::EditGroup {
                    fields,
                    focus_index,
                    ..
                } => (fields, focus_index),
                _ => return,
            };

            let keys = [
                "name",
                "subscription_url",
                "user_agent",
                "update_interval",
                "core_type",
            ];

            match key.code {
                KeyCode::Tab => {
                    *focus_index = (*focus_index + 1) % keys.len();
                }
                KeyCode::BackTab => {
                    *focus_index = if *focus_index == 0 {
                        keys.len() - 1
                    } else {
                        *focus_index - 1
                    };
                }
                KeyCode::Char(c) => {
                    let key = keys[*focus_index];
                    if let Some((_, val)) = fields.iter_mut().find(|(k, _)| k == key) {
                        match key {
                            "core_type" => {
                                const OPTIONS: &[&str] = &["Auto", "Xray", "SingBox"];
                                let idx =
                                    OPTIONS.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                                val.clear();
                                val.push_str(OPTIONS[(idx + 1) % OPTIONS.len()]);
                            }
                            _ => {
                                val.push(c);
                            }
                        }
                    }
                }
                KeyCode::Right => {
                    let field_key = keys[*focus_index];
                    if field_key == "core_type"
                        && let Some((_, val)) = fields.iter_mut().find(|(k, _)| k == field_key)
                    {
                        const OPTIONS: &[&str] = &["Auto", "Xray", "SingBox"];
                        let idx = OPTIONS.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                        let new_idx = (idx + 1) % OPTIONS.len();
                        val.clear();
                        val.push_str(OPTIONS[new_idx]);
                    }
                }
                KeyCode::Left => {
                    let field_key = keys[*focus_index];
                    if field_key == "core_type"
                        && let Some((_, val)) = fields.iter_mut().find(|(k, _)| k == field_key)
                    {
                        const OPTIONS: &[&str] = &["Auto", "Xray", "SingBox"];
                        let idx = OPTIONS.iter().position(|o| *o == val.as_str()).unwrap_or(0);
                        let new_idx = if idx == 0 { OPTIONS.len() - 1 } else { idx - 1 };
                        val.clear();
                        val.push_str(OPTIONS[new_idx]);
                    }
                }
                KeyCode::Backspace => {
                    let key = keys[*focus_index];
                    if let Some((_, val)) = fields.iter_mut().find(|(k, _)| k == key) {
                        val.pop();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}
