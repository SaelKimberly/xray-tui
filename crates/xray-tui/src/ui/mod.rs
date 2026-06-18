pub mod dns;
pub mod logs;
pub mod profiles;
pub mod routing;
pub mod settings;
pub mod statistics;
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
use crate::{AppState, Tab};

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

    match key.code {
        KeyCode::Char('q' | 'Q') => {
            state.should_quit = true;
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
        KeyCode::Enter if state.current_tab == Tab::Profiles => {
            state.add_log(
                "info",
                &format!("Enter pressed on profile {}", state.selected_index + 1),
            );
        }
        KeyCode::Char('/') if state.current_tab == Tab::Profiles => {
            state.search_focused = true;
            state.search_query.clear();
        }
        KeyCode::Esc => {
            state.selected_group_id = None;
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
    let text = format!(" {} — Coming Soon ", name);
    let block = Block::default()
        .title(text)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);
}
