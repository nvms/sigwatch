use crate::app::{App, InputMode, Panel};
use crate::widgets;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use std::io;
use std::time::{Duration, Instant};

pub fn run(app: &mut App) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, app);

    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut last_poll = Instant::now();

    while app.running {
        terminal.draw(|frame| widgets::dashboard::render(frame, app))?;

        let poll_duration = Duration::from_millis(app.poll_rate_ms);
        let elapsed = last_poll.elapsed();
        let timeout = poll_duration.saturating_sub(elapsed);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }

        if last_poll.elapsed() >= poll_duration {
            app.poll();
            last_poll = Instant::now();
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match &app.input_mode {
        InputMode::Normal => handle_normal_key(app, code, modifiers),
        InputMode::Command => handle_command_key(app, code),
    }
}

fn handle_normal_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.running = false,
        KeyCode::Char(':') => {
            app.input_mode = InputMode::Command;
            app.input_buffer.clear();
        }
        KeyCode::Tab => app.cycle_panel(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Enter => handle_enter(app),
        _ => {}
    }
}

fn handle_enter(app: &mut App) {
    if let Panel::Scanner = app.active_panel {
        let flat = app.flat_scan_addresses();
        if app.selected_index < flat.len() {
            app.input_mode = InputMode::Command;
            app.input_buffer = format!("pick {} ", app.selected_index);
        }
    }
}

fn handle_command_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let cmd = app.input_buffer.clone();
            app.input_mode = InputMode::Normal;
            app.execute_command(&cmd);
            app.input_buffer.clear();
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
}
