mod application;
mod audio;
mod domain;
mod presentation;
mod selection;
mod state;
mod ui;

use anyhow::Result;
use presentation::Mode;
use application::dto::input_action::InputAction;
use application::service::app_service::AppService;
use application::state::ApplicationState;
use audio::{spawn_audio_thread, SenderAudioBus, SystemClock};
use domain::r#loop::LoopEngine;
use presentation::effect_handler::apply_effects;
use presentation::ViewModel;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Theme as ExplorerTheme;
use ratatui::widgets::{Block, BorderType, Borders};
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

    // Initialize infrastructure
    let audio_tx = spawn_audio_thread();
    let bus = SenderAudioBus::new(audio_tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);

    // Initialize application and presentation state
    let mut app_state = ApplicationState::new(loop_engine);
    let theme = ExplorerTheme::default()
        .add_default_title()
        .with_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .with_title_bottom(|_| "  Enter: to pads / Space: select / Tab: switch pane / d/Delete: remove / q: quit  ".into());
    let file_explorer = FileExplorer::with_theme(theme)?;
    let mut view_model = ViewModel::new(file_explorer);

    // Initialize application service
    let app_service = AppService::new(audio_tx.clone());

    // Minimal event/render loop with exit on 'q'
    loop {
        terminal.draw(|f| ui::draw_ui(f, &view_model, &app_state))?;

        if event::poll(std::time::Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) => {
                    // Convert to InputAction and handle via AppService
                    let input_action = InputAction::from(Event::Key(key));
                    match app_service.handle_input(&mut app_state, &mut view_model, input_action) {
                        Ok(effects) => {
                            apply_effects(&mut view_model, &audio_tx, effects);
                        }
                        Err(e) => {
                            // Handle error - could add error effect in future
                            eprintln!("Error handling input: {}", e);
                        }
                    }

                    // Handle quit when in Browse mode and 'q' pressed
                    if let KeyCode::Char('q') = key.code
                        && matches!(view_model.mode, Mode::Browse)
                    {
                        break;
                    }
                }
                ev @ Event::Resize(_, _) => {
                    // Convert to InputAction
                    let input_action = InputAction::from(ev);
                    if let Ok(effects) = app_service.handle_input(&mut app_state, &mut view_model, input_action) {
                        apply_effects(&mut view_model, &audio_tx, effects);
                    }
                }
                other => {
                    // Convert to InputAction
                    let input_action = InputAction::from(other);
                    if let Ok(effects) = app_service.handle_input(&mut app_state, &mut view_model, input_action) {
                        apply_effects(&mut view_model, &audio_tx, effects);
                    }
                }
            }
        }

        // Update loop engine
        let loop_effects = app_service.update_loop(&mut app_state);
        apply_effects(&mut view_model, &audio_tx, loop_effects);
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;
    Ok(())
}
