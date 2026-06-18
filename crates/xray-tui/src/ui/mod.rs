pub mod add_server;
pub mod dns;
pub mod logs;
pub mod profiles;
pub mod routing;
pub mod settings;
pub mod statistics;
pub mod groups;
pub mod status_bar;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};
use std::io;
use std::time::Duration;
use std::sync::mpsc;
use crate::{AppMode, AppState, ConfirmAction, Tab};

// ── Entry point ───────────────────────────────────────────────────────

pub fn run(state: &mut AppState) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel::<Event>();
    std::thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if tx.send(ev).is_err() {
                break;
            }
        }
    });

    let refresh_interval = Duration::from_secs(state.config.gui.refresh_interval_secs);
    let mut last_tick = std::time::Instant::now();

    // Create core event channel for async core process communication
    let (core_tx, core_rx) = tokio::sync::mpsc::unbounded_channel();
    state.core_event_tx = Some(core_tx);
    state.core_event_rx = Some(core_rx);

    while !state.should_quit {
        loop {
            match rx.try_recv() {
                Ok(ev) => handle_event(&ev, state),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    state.should_quit = true;
                    break;
                }
            }
        }

        // Process core process events (connect/disconnect/error)
        state.poll_core_events();

        if last_tick.elapsed() >= refresh_interval {
            last_tick = std::time::Instant::now();
        }

        terminal.draw(|f| render(f, &*state))?;
        std::thread::sleep(Duration::from_millis(16));
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ── Event handling ────────────────────────────────────────────────────

fn handle_event(ev: &Event, state: &mut AppState) {
    if let Event::Key(key) = ev {
        handle_key(key, state);
    }
}

fn handle_key(key: &KeyEvent, state: &mut AppState) {
    // Group management overlay mode: pass to groups handler
    if matches!(&state.mode, crate::AppMode::ManageGroups { .. }) {
        groups::handle_key(state, key);
        return;
    }

    // Group form mode: route to groups handler (with Ctrl+C quit)
    if matches!(&state.mode, crate::AppMode::AddGroup { .. } | crate::AppMode::EditGroup { .. }) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => state.should_quit = true,
            _ => groups::handle_key(state, key),
        }
        return;
    }

    // Form mode: route all keys to add_server handler (except Ctrl+C quit)
    if !matches!(state.mode, crate::AppMode::List) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true;
            }
            _ => add_server::handle_key(state, key),
        }
        return;
    }

    // Search mode captures all alphanumeric input
    if state.search_focused {
        match key.code {
            KeyCode::Char(c) => {
                state.search_query.push(c);
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Esc => {
                state.search_focused = false;
                state.search_query.clear();
            }
            _ => {}
        }
        return;
    }

    // Delete confirmation: only y/n/esc
    if let Some(ref confirm) = state.confirmation {
        match confirm {
            ConfirmAction::DeleteProfile(_) | ConfirmAction::DeleteGroup(_) => {},
        }
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                match state.confirmation.take() {
                    Some(ConfirmAction::DeleteProfile(id)) => state.delete_profile(&id),
                    Some(ConfirmAction::DeleteGroup(id)) => state.delete_group(&id),
                    None => {}
                }
            }
            KeyCode::Char('n' | 'N' | 'q' | 'Q') | KeyCode::Esc => {
                state.confirmation = None;
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q' | 'Q') => {
            state.should_quit = true;
        }
        KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) && state.connected_core.is_some() => {
            state.disconnect();
        }

        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
        }
        KeyCode::Tab => {
            let idx = Tab::ALL.iter().position(|t| *t == state.current_tab).unwrap_or(0);
            state.current_tab = Tab::ALL[(idx + 1) % Tab::ALL.len()];
        }
        KeyCode::BackTab => {
            let idx = Tab::ALL.iter().position(|t| *t == state.current_tab).unwrap_or(0);
            state.current_tab = if idx == 0 {
                Tab::ALL[Tab::ALL.len() - 1]
            } else {
                Tab::ALL[idx - 1]
            };
        }
        KeyCode::Up if state.current_tab == Tab::Profiles => {
            state.selected_index = state.selected_index.saturating_sub(1);
        }
        KeyCode::Down if state.current_tab == Tab::Profiles => {
            let max = state.filtered_profiles().len().saturating_sub(1);
            if state.selected_index < max {
                state.selected_index += 1;
            }
        }
        KeyCode::Home if state.current_tab == Tab::Profiles => {
            state.selected_index = 0;
        }
        KeyCode::End if state.current_tab == Tab::Profiles => {
            state.selected_index = state.filtered_profiles().len().saturating_sub(1);
        }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) && state.current_tab == Tab::Profiles => {
            state.move_profile_up();
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) && state.current_tab == Tab::Profiles => {
            state.move_profile_down();
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) && state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.connect_to_profile(&id);
            }
        }

        KeyCode::Enter if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.set_active(&id);
            }
        }
        KeyCode::Char(' ') if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.toggle_multi_select(&id);
            }
        }
        KeyCode::Char('/') if state.current_tab == Tab::Profiles => {
            state.search_focused = true;
            state.search_query.clear();
        }
        // CRUD shortcuts (profiles tab)
        KeyCode::Char('a' | 'A') if state.current_tab == Tab::Profiles => {
            state.start_add_server();
        }
        KeyCode::Char('e' | 'E') if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.start_edit_profile(&id);
            }
        }
        KeyCode::Char('g' | 'G') if state.current_tab == Tab::Profiles => {
            state.mode = AppMode::ManageGroups { selected: 0 };
        }
        KeyCode::Char('d' | 'D') if state.current_tab == Tab::Profiles => {
            if state.multi_select.len() >= 2 {
                let ids: Vec<String> = state.multi_select.iter().cloned().collect();
                for id in &ids {
                    state.delete_profile(id);
                }
                state.multi_select.clear();
            } else if let Some(id) = state.selected_profile_id() {
                state.confirmation = Some(ConfirmAction::DeleteProfile(id));
            }
        }
        KeyCode::Char('c' | 'C') if !key.modifiers.contains(KeyModifiers::CONTROL) && state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.clone_profile(&id);
            }
        }
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) && state.current_tab == Tab::Profiles => {
            state.mode = crate::AppMode::ImportUrl {
                input: String::new(),
                error: None,
            };
        }
        KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT) && state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id()
                && let Some(profile) = state.filtered_profiles().iter().find(|r| r.profile.id == id)
                    && let Ok(url) = xray_tui_config::import_export::format_share_url(&profile.profile) {
                        state.clipboard = Some(url);
                        state.add_log("info", "Share URL copied to clipboard");
                    }
        }
        KeyCode::Esc => {
            if state.confirmation.is_some() {
                state.confirmation = None;
            } else {
                state.selected_group_id = None;
            }
        }
        _ => {}
    }
}

// ── Rendering ─────────────────────────────────────────────────────────

fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab row
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // In form/import mode, render form instead of tabs
    // In form/import/modal mode, render appropriate UI
    if !matches!(state.mode, crate::AppMode::List) {
        match &state.mode {
            crate::AppMode::AddServer { .. } | crate::AppMode::EditServer { .. } => {
                add_server::render(frame, chunks[1], state);
            }
            crate::AppMode::ImportUrl { .. } => {
                add_server::render_import_url(frame, chunks[1], state);
            }
            crate::AppMode::ManageGroups { .. } => {
                groups::render_group_overlay(frame, chunks[1], state);
            }
            crate::AppMode::AddGroup { .. } => {
                groups::render_group_form(frame, chunks[1], state, false);
            }
            crate::AppMode::EditGroup { .. } => {
                groups::render_group_form(frame, chunks[1], state, true);
            }
            _ => {}
        }
        status_bar::render(frame, chunks[2], state);
        return;
    }

    render_tabs(frame, chunks[0], state);

    match state.current_tab {
        Tab::Profiles => profiles::render(frame, chunks[1], state),
        Tab::Settings => settings::render(frame, chunks[1], state),
        Tab::Routing => routing::render(frame, chunks[1], state),
        Tab::Dns => dns::render(frame, chunks[1], state),
        Tab::Logs => logs::render(frame, chunks[1], state),
        Tab::Statistics => statistics::render(frame, chunks[1], state),
    }

    status_bar::render(frame, chunks[2], state);
}

fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let spans: Vec<Span> = Tab::ALL
        .iter()
        .map(|tab| {
            let name = match tab {
                Tab::Profiles => " Profiles ",
                Tab::Settings => " Settings ",
                Tab::Routing => " Routing ",
                Tab::Dns => " DNS ",
                Tab::Logs => " Logs ",
                Tab::Statistics => " Statistics ",
            };
            let selected = *tab == state.current_tab;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            };
            Span::styled(name, style)
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

pub fn render_placeholder_screen(frame: &mut Frame, area: Rect, name: &str) {
    let text = format!("{name} — Coming Soon");
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().title(name).borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}
