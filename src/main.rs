mod app_state;
mod audio;
mod input;
mod selection;
mod state;
mod ui;

use anyhow::Result;
use app_state::{AppState, Mode};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<()> {
    // Terminal init
    let mut terminal = setup_terminal()?;

    // App state
    let mut state = AppState::new()?;

    // Minimal event/render loop with exit on 'q'
    loop {
        terminal.draw(|f| ui::draw_ui(f, &state))?;

        if event::poll(std::time::Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) => {
                    // Delegate handling; handler will update state.status_message
                    input::handle_event(&mut state, Event::Key(key))?;
                    // Handle quit when in Browse mode and 'q' pressed
                    if let KeyCode::Char('q') = key.code
                        && matches!(state.mode, Mode::Browse)
                    {
                        break;
                    }
                }
                ev @ Event::Resize(_, _) => {
                    input::handle_event(&mut state, ev)?;
                }
                other => {
                    input::handle_event(&mut state, other)?;
                }
            }
        }

        state.update_loop();
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;
    Ok(())
}
