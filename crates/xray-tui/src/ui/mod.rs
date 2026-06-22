pub mod add_server;
pub mod groups;
pub mod logs;
pub mod profiles;
pub mod settings;
pub mod statistics;
pub mod status_bar;
pub mod theme;

use crate::{AppMode, AppState, ConfirmAction, SortColumn, Tab};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::{Frame, Terminal};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

// ── Entry point ───────────────────────────────────────────────────────

pub fn run(state: &mut AppState) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
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

    // Trigger startup version check if enabled
    if state.config.updates.check_on_startup {
        state.spawn_update_check();
    }

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
    // Delete confirmation: only y/n/esc — check before mode dispatch so
    // confirmation is handled even when inside a sub-mode like ManageGroups.
    if state.confirmation.is_some() {
        match key.code {
            KeyCode::Char('y' | 'Y') => match state.confirmation.take() {
                Some(ConfirmAction::DeleteProfile(id)) => state.delete_profile(&id),
                Some(ConfirmAction::DeleteGroup(id)) => state.delete_group(&id),
                None => {}
            },
            KeyCode::Char('n' | 'N' | 'q' | 'Q') | KeyCode::Esc => {
                state.confirmation = None;
            }
            _ => {}
        }
        return;
    }

    // Group management overlay mode: pass to groups handler
    if matches!(&state.mode, crate::AppMode::ManageGroups { .. }) {
        groups::handle_key(state, key);
        return;
    }

    // Group form mode: route to groups handler (with Ctrl+C quit)
    if matches!(
        &state.mode,
        crate::AppMode::AddGroup { .. } | crate::AppMode::EditGroup { .. }
    ) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true
            }
            _ => groups::handle_key(state, key),
        }
        return;
    }

    // Settings mode: route to settings handler (with Ctrl+C quit)
    if matches!(&state.mode, crate::AppMode::Settings { .. }) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true;
            }
            _ => settings::handle_key(state, key),
        }
        return;
    }

    // Form mode: route all keys to add_server handler (except Ctrl+C quit)
    if !matches!(state.mode, crate::AppMode::List) && !matches!(&state.mode, crate::AppMode::Help) {
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
                state.filter_cache_valid.set(false);
            }
            KeyCode::Backspace => {
                state.search_query.pop();
                state.filter_cache_valid.set(false);
            }
            KeyCode::Esc => {
                state.search_focused = false;
                state.search_query.clear();
                state.filter_cache_valid.set(false);
            }
            _ => {}
        }
        return;
    }

    // Speed test menu mode
    if matches!(&state.mode, crate::AppMode::SpeedTestMenu { .. }) {
        match key.code {
            KeyCode::Up => {
                if let crate::AppMode::SpeedTestMenu { ref mut selected } = state.mode {
                    *selected = selected.saturating_sub(1);
                    // Skip the separator
                    if *selected == 4 {
                        *selected = 3;
                    }
                }
            }
            KeyCode::Down => {
                if let crate::AppMode::SpeedTestMenu { ref mut selected } = state.mode {
                    let max = 7usize;
                    if *selected < max {
                        *selected += 1;
                    }
                    // Skip the separator
                    if *selected == 4 {
                        *selected = 5;
                    }
                }
            }
            KeyCode::Enter => {
                let selected = match &state.mode {
                    crate::AppMode::SpeedTestMenu { selected } => *selected,
                    _ => 0,
                };
                state.mode = crate::AppMode::List;
                match selected {
                    0 => {
                        if let Some(id) = state.selected_profile_id() {
                            state.start_tcp_ping(&id);
                        }
                    }
                    1 => {
                        if let Some(id) = state.selected_profile_id() {
                            state.start_real_ping(&id);
                        }
                    }
                    2 => {
                        if let Some(id) = state.selected_profile_id() {
                            state.start_speed_test(&id);
                        }
                    }
                    3 => {
                        if let Some(id) = state.selected_profile_id() {
                            state.start_udp_test(&id);
                        }
                    }
                    5 => {
                        state.start_batch_ping();
                    }
                    6 => {
                        state.sort_column = SortColumn::Delay;
                        state.sort_ascending = true;
                        state.filter_cache_valid.set(false);
                    }
                    7 => {
                        state.remove_failed_servers();
                    }
                    _ => {}
                }
            }

            KeyCode::Esc => {
                state.mode = crate::AppMode::List;
            }
            KeyCode::Char('?') => {
                state.previous_mode = Some(Box::new(state.mode.clone()));
                state.mode = crate::AppMode::Help;
            }
            _ => {}
        }
        return;
    }


    match key.code {
        // Help mode: Esc or ? to close, ignore other keys
        KeyCode::Char('?') | KeyCode::Esc if matches!(&state.mode, crate::AppMode::Help) => {
            state.mode = *state.previous_mode.take().unwrap_or(Box::new(crate::AppMode::List));
        }
        // Open help overlay from List mode
        KeyCode::Char('?') if !matches!(&state.mode, crate::AppMode::Help) => {
            state.previous_mode = Some(Box::new(state.mode.clone()));
            state.mode = crate::AppMode::Help;
        }
        KeyCode::Char('q' | 'Q') => {
            state.should_quit = true;
        }
        KeyCode::Char('C')
            if key.modifiers.contains(KeyModifiers::CONTROL) && state.connected_core.is_some() =>
        {
            state.disconnect();
        }

        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
        }
        KeyCode::Tab => {
            let idx = Tab::ALL
                .iter()
                .position(|t| *t == state.current_tab)
                .unwrap_or(0);
            state.current_tab = Tab::ALL[(idx + 1) % Tab::ALL.len()];
        }
        KeyCode::BackTab => {
            let idx = Tab::ALL
                .iter()
                .position(|t| *t == state.current_tab)
                .unwrap_or(0);
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
            let max = state.filtered_len().saturating_sub(1);
            if state.selected_index < max {
                state.selected_index += 1;
            }
        }
        KeyCode::Home if state.current_tab == Tab::Profiles => {
            state.selected_index = 0;
        }
        KeyCode::End if state.current_tab == Tab::Profiles => {
            state.selected_index = state.filtered_len().saturating_sub(1);
        }
        KeyCode::Up
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            state.move_profile_up();
        }
        KeyCode::Down
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            state.move_profile_down();
        }
        KeyCode::Enter
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            if let Some(id) = state.selected_profile_id() {
                state.connect_to_profile(&id);
            }
        }

        KeyCode::Enter if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.set_active(&id);
            }
        }
        KeyCode::Enter if state.current_tab == Tab::Settings => {
            state.enter_settings();
        }
        KeyCode::Char(' ') if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.toggle_multi_select(&id);
            }
        }
        KeyCode::Char('/') if state.current_tab == Tab::Profiles => {
            state.search_focused = true;
            state.search_query.clear();
            state.filter_cache_valid.set(false);
        }
        // Speed test menu
        KeyCode::Char('t' | 'T') if state.current_tab == Tab::Profiles => {
            state.mode = crate::AppMode::SpeedTestMenu { selected: 0 };
        }
        // Cycle sort column
        KeyCode::Char('o' | 'O') if state.current_tab == Tab::Profiles => {
            let all = &[
                SortColumn::Remarks,
                SortColumn::Address,
                SortColumn::Port,
                SortColumn::Delay,
                SortColumn::Speed,
                SortColumn::Traffic,
                SortColumn::ConfigType,
                SortColumn::Core,
            ];
            let current_idx = all
                .iter()
                .position(|c| *c == state.sort_column)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % all.len();
            state.sort_column = all[next_idx];
            state.sort_ascending = true;
            state.filter_cache_valid.set(false);
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
        KeyCode::Char('c' | 'C')
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            if let Some(id) = state.selected_profile_id() {
                state.clone_profile(&id);
            }
        }
        KeyCode::Char('v')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            let clipboard_text = arboard::Clipboard::new()
                .ok()
                .and_then(|mut cb| cb.get_text().ok())
                .unwrap_or_default();
            state.mode = crate::AppMode::ImportUrl {
                input: clipboard_text,
                error: None,
            };
        }
        KeyCode::Char('S')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && state.current_tab == Tab::Profiles =>
        {
            if let Some(id) = state.selected_profile_id()
                && let Some(profile) = state
                    .filtered_profiles()
                    .iter()
                    .find(|r| r.profile.id == id)
                && let Ok(url) = xray_tui_config::import_export::format_share_url(&profile.profile)
            {
                state.clipboard = Some(url);
                state.add_log("info", "Share URL copied to clipboard");
            }
        }
        KeyCode::Esc => {
            if state.confirmation.is_some() {
                state.confirmation = None;
            } else {
                state.selected_group_id = None;
                state.filter_cache_valid.set(false);
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
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // In form/import mode, render form instead of tabs
    // In form/import/modal mode, render appropriate UI
    if !matches!(state.mode, crate::AppMode::List) {
        // SpeedTestMenu renders the profiles underneath, then overlays the menu
        if matches!(&state.mode, crate::AppMode::SpeedTestMenu { .. }) {
            render_tabs(frame, chunks[0], state);
            profiles::render(frame, chunks[1], state);
            render_speed_test_menu(frame, chunks[1], state);
            status_bar::render(frame, chunks[2], state);
            return;
        }

        // Help overlay: render base UI first, then overlay help
        if matches!(&state.mode, crate::AppMode::Help) {
            render_tabs(frame, chunks[0], state);
            match state.current_tab {
                Tab::Profiles => profiles::render(frame, chunks[1], state),
                Tab::Settings => settings::render(frame, chunks[1], state),
                Tab::Logs => logs::render(frame, chunks[1], state),
                Tab::Statistics => statistics::render(frame, chunks[1], state),
            }
            render_help_overlay(frame, chunks[1], state);
            status_bar::render(frame, chunks[2], state);
            return;
        }

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
            crate::AppMode::Settings { .. } => {
                render_tabs(frame, chunks[0], state);
                settings::render(frame, chunks[1], state);
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
        Tab::Logs => logs::render(frame, chunks[1], state),
        Tab::Statistics => statistics::render(frame, chunks[1], state),
    }
    status_bar::render(frame, chunks[2], state);
}
fn help_content(state: &AppState) -> Vec<(&'static str, &'static str)> {
    // Determine context from previous mode
    match &state.mode {
        _ if state.current_tab == Tab::Profiles => vec![
            ("↑↓ / PgUp PgDn", "Navigate profiles"),
            ("Enter", "Set as active server"),
            ("Ctrl+Enter", "Connect to selected server"),
            ("Space", "Toggle multi-select"),
            ("a", "Add new server"),
            ("e", "Edit selected server"),
            ("d", "Delete selected server(s)"),
            ("c", "Clone selected server"),
            ("g", "Manage subscription groups"),
            ("t", "Open speed test menu"),
            ("o", "Cycle sort column"),
            ("/", "Search/filter"),
            ("Ctrl+V", "Import share URL"),
            ("Ctrl+Shift+C", "Copy share URL"),
            ("Tab / Shift+Tab", "Cycle tabs"),
            ("?", "Toggle this help"),
            ("q / Ctrl+C", "Quit"),
        ],
        _ if state.current_tab == Tab::Settings => vec![
            ("↑↓", "Navigate settings menu"),
            ("Enter", "Open selected section"),
            ("Esc", "Close settings"),
            ("Tab / Shift+Tab", "Cycle tabs"),
            ("?", "Toggle this help"),
            ("q / Ctrl+C", "Quit"),
        ],
        _ if state.current_tab == Tab::Statistics => vec![
            ("Tab / Shift+Tab", "Cycle tabs"),
            ("?", "Toggle this help"),
            ("q / Ctrl+C", "Quit"),
        ],
        _ if state.current_tab == Tab::Logs => vec![
            ("Tab / Shift+Tab", "Cycle tabs"),
            ("?", "Toggle this help"),
            ("q / Ctrl+C", "Quit"),
        ],
        _ => vec![
            ("Tab / Shift+Tab", "Cycle tabs"),
            ("?", "Toggle this help"),
            ("q / Ctrl+C", "Quit"),
        ],
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = help_content(state);
    let lines: Vec<Line> = content
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!(" {:<20}", key), crate::ui::theme::Theme::HINT),
                Span::raw("  "),
                Span::styled(*desc, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let popup_width = 52u16.min(area.width.saturating_sub(4));
    let popup_height = (content.len() as u16 + 2).min(area.height.saturating_sub(4));

    let vert_pad = (area.height.saturating_sub(popup_height)) / 2;
    let horiz_pad = (area.width.saturating_sub(popup_width)) / 2;

    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vert_pad),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area)[1];
    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horiz_pad),
            Constraint::Length(popup_width),
            Constraint::Min(0),
        ])
        .split(popup_area)[1];

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}

fn render_speed_test_menu(frame: &mut Frame, area: Rect, state: &AppState) {
    let menu_items = [
        "TCP Ping (Selected)",
        "Real Ping (Selected)",
        "Speed Test (Selected)",
        "UDP Test (Selected)",
        "",
        "Batch TCP Ping (All Visible)",
        "Sort by Delay",
        "Remove Bad Servers",
    ];

    let selected = match &state.mode {
        crate::AppMode::SpeedTestMenu { selected } => *selected,
        _ => return,
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, item) in menu_items.iter().enumerate() {
        if item.is_empty() {
            lines.push(Line::from(Span::raw(" ─────")));
            continue;
        }
        let prefix = if i == selected { "► " } else { "  " };
        let style = if i == selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(format!("{prefix}{item}"), style)));
    }

    // Calculate popup dimensions
    let item_count = menu_items.len() as u16;
    let popup_width = 34u16;
    let popup_height = item_count + 2; // border top/bottom

    let vert_pad = (area.height.saturating_sub(popup_height)) / 2;
    let horiz_pad = (area.width.saturating_sub(popup_width)) / 2;

    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vert_pad),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area)[1];

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horiz_pad),
            Constraint::Length(popup_width),
            Constraint::Min(0),
        ])
        .split(popup_area)[1];

    let block = Block::default()
        .title(" Speed Test ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}
fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|tab| {
            let name = match tab {
                Tab::Profiles => " Profiles ",
                Tab::Settings => " Settings ",
                Tab::Logs => " Logs ",
                Tab::Statistics => " Statistics ",
            };
            Line::from(Span::styled(
                name,
                crate::ui::theme::Theme::TAB_DESELECTED,
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(
            Tab::ALL
                .iter()
                .position(|t| *t == state.current_tab)
                .unwrap_or(0),
        )
        .highlight_style(crate::ui::theme::Theme::TAB_SELECTED)
        .divider(Span::raw(""));
    frame.render_widget(tabs, area);
}

pub fn render_placeholder_screen(frame: &mut Frame, area: Rect, name: &str) {
    let text = format!("{name} — Coming Soon");
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().title(name).borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}
