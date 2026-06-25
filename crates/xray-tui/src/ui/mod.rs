pub mod actions_log;
pub mod add_server;
pub mod groups;
pub mod logs;
pub mod profiles;
pub mod settings;
pub mod statistics;
pub mod status_bar;
pub mod theme;
use crate::{AppMode, AppState, ConfirmAction, SortColumn, Tab};
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::{Frame, Terminal};
use std::io;
use std::sync::atomic::Ordering;
use std::time::Duration;
use xray_tui_config::subscription::subscription_url_split;

pub fn render_confirmation_overlay(frame: &mut Frame, area: Rect, text: &str) {
    use unicode_width::UnicodeWidthStr;
    let width = text.width() + 4;
    let popup_width = width as u16;
    let popup_height = 3u16; // border(1) + text(1) + border(1)

    // Return early if terminal too narrow for the popup
    if area.width < popup_width {
        return;
    }

    let h_pad = area.width.saturating_sub(popup_width) / 2;
    let v_pad = area.height.saturating_sub(popup_height + 2);

    let overlay_area = Rect::new(h_pad, v_pad, popup_width.min(area.width), popup_height);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD))
        .style(Style::new().bg(Color::Rgb(40, 20, 20)));
    let paragraph = Paragraph::new(text.to_string())
        .style(
            Style::new()
                .fg(Color::White)
                .bg(Color::Rgb(40, 20, 20))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);
    frame.render_widget(paragraph, inner);
}
// ── Entry point ───────────────────────────────────────────────────────

pub async fn run(state: &mut AppState) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let ts = terminal.size().unwrap_or_default();
    state.actions_compact = ts.height < 20;
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(64);
    tokio::task::spawn_blocking(move || {
        while let Ok(ev) = event::read() {
            if tx.try_send(ev).is_err() {
                break;
            }
        }
    });
    let refresh_interval = Duration::from_secs(state.config.gui.refresh_interval_secs);
    let mut last_tick = std::time::Instant::now();

    // (channels already created in AppState::new)

    // Trigger startup version check if enabled
    if state.config.updates.check_on_startup {
        state.spawn_update_check();
    }
    while !state.should_quit {
        loop {
            match rx.try_recv() {
                Ok(ev) => handle_event(&ev, state).await,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    state.should_quit = true;
                    break;
                }
            }
        }

        // Process core process events (connect/disconnect/error)
        state.poll_core_events().await;

        if last_tick.elapsed() >= refresh_interval {
            last_tick = std::time::Instant::now();
        }
        let ts = terminal.size().unwrap_or_default();
        state.term_height.set(ts.height);
        terminal.draw(|f| render(f, &*state))?;
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    // Signal background tasks to stop
    state.shutdown_token.store(true, Ordering::Relaxed);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ── Event handling ────────────────────────────────────────────────────

async fn handle_event(ev: &Event, state: &mut AppState) {
    if let Event::Key(key) = ev {
        handle_key(key, state).await;
    }
}
async fn handle_key(key: &KeyEvent, state: &mut AppState) {
    // Delete confirmation: only y/n/esc — check before mode dispatch so
    // confirmation is handled even when inside a sub-mode like ManageGroups.
    if state.confirmation.is_some() {
        match key.code {
            KeyCode::Char('y' | 'Y') => match state.confirmation.take() {
                Some(ConfirmAction::DeleteProfile(id)) => state.delete_profile(&id).await,
                Some(ConfirmAction::DeleteGroup(id)) => state.delete_group(&id).await,
                Some(ConfirmAction::ClearGroup(id)) => state.clear_group(&id).await,
                Some(ConfirmAction::Quit) => state.should_quit = true,
                None => {}
            },
            KeyCode::Char('n' | 'N' | 'q' | 'Q') | KeyCode::Esc => {
                state.confirmation = None;
            }
            _ => {}
        }
        return;
    }

    // F1 toggles actions panel compact/full — works across all modes
    if matches!(key.code, KeyCode::F(1)) {
        state.actions_compact = !state.actions_compact;
        return;
    }

    // Group management overlay mode: pass to groups handler
    if matches!(&state.mode, crate::AppMode::ManageGroups { .. }) {
        groups::handle_key(state, key).await;
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
            _ => groups::handle_key(state, key).await,
        }
        return;
    }

    // Settings mode: route to settings handler (with Ctrl+C quit)
    if matches!(&state.mode, crate::AppMode::Settings { .. }) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true;
            }
            _ => settings::handle_key(state, key).await,
        }
        return;
    }

    // BatchImport: route all keys to batch import handler
    if matches!(&state.mode, crate::AppMode::BatchImport { .. }) {
        add_server::handle_batch_import_key(state, key).await;
        return;
    }

    // Form mode: route all keys to add_server handler (except Ctrl+C quit, SpeedTestMenu, BatchImport)
    if !matches!(state.mode, crate::AppMode::List)
        && !matches!(&state.mode, crate::AppMode::Help)
        && !matches!(&state.mode, crate::AppMode::SpeedTestMenu { .. })
        && !matches!(&state.mode, crate::AppMode::BatchImport { .. })
    {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true;
            }
            _ => add_server::handle_key(state, key).await,
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
                let _ = execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape);
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
                    let max = 8usize;
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
                        state.start_batch_then_real_ping();
                    }
                    7 => {
                        state.sort_column = SortColumn::Delay;
                        state.sort_ascending = true;
                        state.filter_cache_valid.set(false);
                    }
                    8 => {
                        state.remove_failed_servers().await;
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
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if state.connected_core.is_some() && state.confirmation.is_none() {
                    state.confirmation = Some(ConfirmAction::Quit);
                } else {
                    state.should_quit = true;
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        // Help mode: Esc or ? to close, ignore other keys
        KeyCode::Char('?') | KeyCode::Esc if matches!(&state.mode, crate::AppMode::Help) => {
            state.mode = *state
                .previous_mode
                .take()
                .unwrap_or(Box::new(crate::AppMode::List));
        }
        // Open help overlay from List mode
        KeyCode::Char('?') if !matches!(&state.mode, crate::AppMode::Help) => {
            state.previous_mode = Some(Box::new(state.mode.clone()));
            state.mode = crate::AppMode::Help;
        }
        KeyCode::Char('q' | 'Q') => {
            if state.connected_core.is_some() && state.confirmation.is_none() {
                state.confirmation = Some(ConfirmAction::Quit);
            } else {
                state.should_quit = true;
            }
        }
        KeyCode::Char('d')
            if key.modifiers.contains(KeyModifiers::CONTROL) && state.connected_core.is_some() =>
        {
            state.disconnect();
        }

        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if state.connected_core.is_some() && state.confirmation.is_none() {
                state.confirmation = Some(ConfirmAction::Quit);
            } else {
                state.should_quit = true;
            }
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
        KeyCode::PageUp if state.current_tab == Tab::Profiles => {
            let page = state.term_height.get().saturating_sub(5) as usize;
            state.selected_index = state.selected_index.saturating_sub(page);
        }
        KeyCode::PageDown if state.current_tab == Tab::Profiles => {
            let page = state.term_height.get().saturating_sub(5) as usize;
            let max = state.filtered_len().saturating_sub(1);
            state.selected_index = (state.selected_index + page).min(max);
        }
        // Logs tab scrolling — inverted: Up→newer, Down→older
        KeyCode::Up if state.current_tab == Tab::Logs => {
            state.log_scroll = state.log_scroll.saturating_sub(1);
        }
        KeyCode::Down if state.current_tab == Tab::Logs => {
            let max = state.log_buffer.len().saturating_sub(1);
            if state.log_scroll < max {
                state.log_scroll += 1;
            }
        }
        KeyCode::PageUp if state.current_tab == Tab::Logs => {
            state.log_scroll = state.log_scroll.saturating_sub(20);
        }
        KeyCode::PageDown if state.current_tab == Tab::Logs => {
            state.log_scroll = state.log_scroll.saturating_add(20);
        }
        KeyCode::Home if state.current_tab == Tab::Logs => {
            state.log_scroll = state.log_buffer.len().saturating_sub(1);
        }
        KeyCode::End if state.current_tab == Tab::Logs => {
            state.log_scroll = 0;
        }
        KeyCode::Up
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            state.move_profile_up().await;
        }
        KeyCode::Down
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            state.move_profile_down().await;
        }
        KeyCode::Enter
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            if let Some(id) = state.selected_profile_id() {
                state.connect_to_profile(&id);
            }
        }
        KeyCode::Char('g')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            if let Some(id) = state.selected_profile_id() {
                state.connect_to_profile(&id);
            }
        }

        KeyCode::Enter if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.set_active(&id).await;
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
        // Ctrl+A: select all filtered profiles
        KeyCode::Char('a')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.current_tab == Tab::Profiles =>
        {
            let ids: Vec<String> = state
                .filtered_profiles()
                .map(|r| r.profile.id.clone())
                .collect();
            for id in ids {
                state.multi_select.insert(id);
            }
        }
        // Ctrl+Shift+A: deselect all
        KeyCode::Char('A')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && state.current_tab == Tab::Profiles =>
        {
            state.multi_select.clear();
        }
        KeyCode::Char('/') if state.current_tab == Tab::Profiles => {
            state.search_focused = true;
            state.search_query.clear();
            state.filter_cache_valid.set(false);
            let _ = execute!(std::io::stdout(), SetCursorStyle::BlinkingBlock);
        }
        // Speed test menu
        KeyCode::Char('t' | 'T') if state.current_tab == Tab::Profiles => {
            state.mode = crate::AppMode::SpeedTestMenu { selected: 0 };
        }
        // Cycle sort column — preserve selection by profile ID
        KeyCode::Char('o' | 'O') if state.current_tab == Tab::Profiles => {
            let selected_id = state.selected_profile_id();
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
            // Restore selection by profile ID
            if let Some(pid) = selected_id {
                let pos = state.filtered_profiles().position(|r| r.profile.id == pid);
                if let Some(pos) = pos {
                    state.selected_index = pos;
                }
            }
        }
        // CRUD shortcuts (profiles tab)
        KeyCode::Char('a' | 'A') if state.current_tab == Tab::Profiles => {
            state.start_add_server();
        }
        KeyCode::Char('e' | 'E') if state.current_tab == Tab::Profiles => {
            if let Some(id) = state.selected_profile_id() {
                state.start_edit_profile(&id).await;
            }
        }
        KeyCode::Char('g' | 'G') if state.current_tab == Tab::Profiles => {
            state.mode = AppMode::ManageGroups { selected: 0 };
        }
        KeyCode::Char('[') if state.current_tab == Tab::Profiles => {
            state.cycle_group(-1);
        }
        KeyCode::Char(']') if state.current_tab == Tab::Profiles => {
            state.cycle_group(1);
        }
        KeyCode::Char('d' | 'D') if state.current_tab == Tab::Profiles => {
            if state.multi_select.len() >= 2 {
                let ids: Vec<String> = state.multi_select.iter().cloned().collect();
                for id in &ids {
                    state.delete_profile(id).await;
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
                state.clone_profile(&id).await;
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

            let urls = subscription_url_split(&clipboard_text);
            match urls.len() {
                0 => {
                    state.mode = crate::AppMode::ImportUrl {
                        input: clipboard_text,
                        error: Some("No valid share URLs found".into()),
                    };
                }
                1 => {
                    state.mode = crate::AppMode::ImportUrl {
                        input: clipboard_text,
                        error: None,
                    };
                }
                _ => {
                    state.start_batch_import(&urls);
                }
            }
        }
        KeyCode::Char('S')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.modifiers.contains(KeyModifiers::SHIFT)
                && state.current_tab == Tab::Profiles =>
        {
            let url = state.selected_profile_id().and_then(|id| {
                let row = state.filtered_profiles().find(|r| r.profile.id == id)?;
                xray_tui_config::import_export::format_share_url(&row.profile).ok()
            });
            if let Some(url) = url {
                state.clipboard = Some(url);
                state.add_log("info", "Share URL copied to clipboard", "tui");
            }
        }
        KeyCode::Esc => {
            if !state.actions_compact && state.term_height.get() < 20 {
                state.actions_compact = true; // close overlay in small terminal
            } else if state.confirmation.is_some() {
                state.confirmation = None;
            } else {
                state.selected_group_id = None;
                state.filter_cache_valid.set(false);
            }
        }
        _ => {}
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    const FULL_PANEL_HEIGHT: u16 = 8;
    const SMALL_THRESH: u16 = 20;

    let is_small = frame.area().height < SMALL_THRESH;
    let overlay = !state.actions_compact && is_small;
    let ph = if state.current_tab != Tab::Profiles
        && !matches!(state.mode, AppMode::SpeedTestMenu { .. } | AppMode::Help)
    {
        0 // actions panel only useful on Profiles tab
    } else if overlay {
        0
    } else if state.actions_compact {
        1
    } else {
        FULL_PANEL_HEIGHT
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 0: tabs
            Constraint::Min(3),     // 1: content
            Constraint::Length(ph), // 2: actions panel
            Constraint::Length(1),  // 3: status bar
        ])
        .split(frame.area());

    // Overlay mode: full panel replaces content area in small terminals
    if overlay {
        render_tabs(frame, chunks[0], state);
        actions_log::render_full(frame, chunks[1], state);
        status_bar::render(frame, chunks[3], state);
        return;
    }

    // Inline: render actions panel at bottom (chunks[2])
    if state.actions_compact {
        actions_log::render_compact(frame, chunks[2], state);
    } else {
        actions_log::render_full(frame, chunks[2], state);
    }
    // Render tabs — only in modes that show them (not AddServer, EditServer, ImportUrl, Group forms, BatchImport)
    let is_form_mode = matches!(
        &state.mode,
        crate::AppMode::AddServer { .. }
            | crate::AppMode::EditServer { .. }
            | crate::AppMode::ImportUrl { .. }
            | crate::AppMode::ManageGroups { .. }
            | crate::AppMode::AddGroup { .. }
            | crate::AppMode::EditGroup { .. }
            | crate::AppMode::BatchImport { .. }
    );
    if !is_form_mode {
        render_tabs(frame, chunks[0], state);
    }
    if !matches!(state.mode, crate::AppMode::List) {
        if matches!(&state.mode, crate::AppMode::SpeedTestMenu { .. }) {
            profiles::render(frame, chunks[1], state);
            render_speed_test_menu(frame, chunks[1], state);
            status_bar::render(frame, chunks[3], state);
            return;
        }

        if matches!(&state.mode, crate::AppMode::Help) {
            match state.current_tab {
                Tab::Profiles => profiles::render(frame, chunks[1], state),
                Tab::Settings => settings::render(frame, chunks[1], state),
                Tab::Logs => logs::render(frame, chunks[1], state),
                Tab::Statistics => statistics::render(frame, chunks[1], state),
            }
            render_help_overlay(frame, chunks[1], state);
            status_bar::render(frame, chunks[3], state);
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
            crate::AppMode::BatchImport { .. } => {
                add_server::render_batch_import(frame, chunks[1], state);
            }
            crate::AppMode::Settings { .. } => {
                settings::render(frame, chunks[1], state);
            }
            _ => {}
        }
        status_bar::render(frame, chunks[3], state);
        return;
    }

    match state.current_tab {
        Tab::Profiles => profiles::render(frame, chunks[1], state),
        Tab::Settings => settings::render(frame, chunks[1], state),
        Tab::Logs => logs::render(frame, chunks[1], state),
        Tab::Statistics => statistics::render(frame, chunks[1], state),
    }
    status_bar::render(frame, chunks[3], state);
}
fn help_content(state: &AppState) -> Vec<(&'static str, &'static str)> {
    match &state.mode {
        crate::AppMode::Help => match state
            .previous_mode
            .as_deref()
            .cloned()
            .unwrap_or(AppMode::List)
        {
            AppMode::List => match state.current_tab {
                Tab::Profiles => vec![
                    ("↑↓ / PgUp PgDn", "Navigate profiles"),
                    ("Enter", "Set as active server"),
                    ("Ctrl+Enter", "Connect to selected server"),
                    ("Ctrl+G", "Connect to selected server"),
                    ("Space", "Toggle multi-select"),
                    ("a", "Add new server"),
                    ("e", "Edit selected server"),
                    ("d", "Delete selected server(s)"),
                    ("c", "Clone selected server"),
                    ("g", "Manage subscription groups"),
                    ("[ / ]", "Cycle groups"),
                    ("t", "Open speed test menu"),
                    ("o", "Cycle sort column"),
                    ("/", "Search/filter"),
                    ("Ctrl+V", "Import share URL"),
                    ("Ctrl+Shift+S", "Copy share URL"),
                    ("Tab / Shift+Tab", "Cycle tabs"),
                    ("Ctrl+D", "Disconnect"),
                    ("?", "Toggle this help"),
                    ("q / Ctrl+C", "Quit"),
                ],
                Tab::Settings => vec![
                    ("↑↓", "Navigate settings menu"),
                    ("Enter", "Open selected section"),
                    ("Esc", "Close settings"),
                    ("Tab / Shift+Tab", "Cycle tabs"),
                    ("?", "Toggle this help"),
                    ("q / Ctrl+C", "Quit"),
                ],
                Tab::Logs => vec![
                    ("↑↓", "Scroll logs"),
                    ("PgUp / PgDn", "Page up/down"),
                    ("Home / End", "Jump to oldest/newest"),
                    ("c", "Toggle core logs"),
                    ("t", "Toggle TUI logs"),
                    ("v", "Toggle validation/subscription logs"),
                    ("Tab / Shift+Tab", "Cycle tabs"),
                    ("?", "Toggle this help"),
                    ("q / Ctrl+C", "Quit"),
                ],
                Tab::Statistics => vec![
                    ("Tab / Shift+Tab", "Cycle tabs"),
                    ("?", "Toggle this help"),
                    ("q / Ctrl+C", "Quit"),
                ],
            },
            _ => vec![
                ("Tab / Shift+Tab", "Cycle tabs"),
                ("?", "Toggle this help"),
                ("q / Ctrl+C", "Quit"),
            ],
        },
        _ => Vec::new(),
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
        "TCP Ping (All Visible)",
        "TCP + Real Ping (All Visible)",
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
        .title(" Server Tools ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}
fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split area into [indicator(3) | tabs]
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    // Connection indicator
    let indicator = if state.connecting {
        Span::styled(" ⟳", crate::ui::theme::Theme::SPINNER)
    } else if state.connected_core.is_some() {
        Span::styled(" ●", crate::ui::theme::Theme::SUCCESS)
    } else {
        Span::styled(" ○", crate::ui::theme::Theme::HINT)
    };
    let indicator_para = Paragraph::new(Line::from(indicator));
    frame.render_widget(indicator_para, chunks[0]);

    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|tab| {
            let name = match tab {
                Tab::Profiles => " Profiles ",
                Tab::Settings => " Settings ",
                Tab::Logs => " Logs ",
                Tab::Statistics => " Statistics ",
            };
            Line::from(Span::styled(name, crate::ui::theme::Theme::TAB_DESELECTED))
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
    frame.render_widget(tabs, chunks[1]);
}

pub fn render_placeholder_screen(frame: &mut Frame, area: Rect, name: &str) {
    let text = format!("{name} — Coming Soon");
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().title(name).borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}
