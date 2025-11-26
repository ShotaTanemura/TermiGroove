//! Stateless application service.
//!
//! This module provides `AppService`, a stateless service that orchestrates
//! application use cases by coordinating domain logic and producing effects.
//!
//! The service receives state as parameters and returns effects, ensuring
//! it remains reusable and testable without holding internal state.

use crate::application::dto::input_action::{InputAction, KeyCode, KeyModifiers};
use crate::application::service::effect::Effect;
use crate::application::state::ApplicationState;
use crate::audio::AudioCommand;
use crate::domain::r#loop::LoopState;
use crate::presentation::ViewModel;
use ratatui::crossterm::event::{Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyModifiers as CrosstermModifiers};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

/// Stateless application service that orchestrates use cases.
///
/// This service has no internal state and receives all state as parameters.
/// It coordinates domain logic and produces effects that need to be executed
/// by infrastructure layer components.
#[derive(Debug)]
pub struct AppService {
    /// Audio command sender for producing audio effects
    #[allow(dead_code)] // Reserved for future direct audio command sending
    audio_tx: Sender<AudioCommand>,
}

impl AppService {
    /// Creates a new instance of the stateless application service.
    pub fn new(audio_tx: Sender<AudioCommand>) -> Self {
        Self { audio_tx }
    }

    /// Handles an input action and produces effects.
    ///
    /// This method orchestrates input handling logic, mutating the provided
    /// application state and view model, and returning effects that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `app_state` - Mutable reference to the application state
    /// * `view_model` - Mutable reference to the view model (for presentation state changes)
    /// * `input_action` - The input action to handle
    ///
    /// # Returns
    ///
    /// A vector of effects that need to be executed, or an error if handling failed.
    pub fn handle_input(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        input_action: InputAction,
    ) -> anyhow::Result<Vec<Effect>> {
        let mut effects = Vec::new();

        match input_action {
            InputAction::KeyPressed { key, modifiers } => {
                self.handle_key_pressed(app_state, view_model, key, modifiers, &mut effects)?;
            }
            InputAction::KeyReleased { .. } => {
                // Key release events are not currently handled
            }
            InputAction::Resize { .. } => {
                // Resize events are handled by UI layer, no effects needed
            }
        }

        Ok(effects)
    }

    /// Updates the loop engine state.
    ///
    /// This method orchestrates loop update logic, mutating the provided
    /// application state and returning effects that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `app_state` - Mutable reference to the application state
    ///
    /// # Returns
    ///
    /// A vector of effects that need to be executed.
    pub fn update_loop(&self, app_state: &mut ApplicationState) -> Vec<Effect> {
        app_state.update_loop();
        // Currently, update_loop doesn't produce effects, but this is where
        // they would be added if needed (e.g., scheduled audio playback)
        Vec::new()
    }

    /// Handles a key press event.
    fn handle_key_pressed(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        key: KeyCode,
        modifiers: KeyModifiers,
        effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        match view_model.mode {
            crate::app_state::Mode::Browse => {
                self.handle_browse_mode_key(app_state, view_model, key, effects)?;
            }
            crate::app_state::Mode::Pads => {
                self.handle_pads_mode_key(app_state, view_model, key, modifiers, effects)?;
            }
        }

        Ok(())
    }

    /// Handles key presses in Browse mode.
    fn handle_browse_mode_key(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        key: KeyCode,
        effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        match key {
            KeyCode::Tab => {
                view_model.toggle_focus();
                effects.push(Effect::StatusMessage(view_model.focus_status_message()));
            }
            KeyCode::Enter => {
                match app_state.enter_pads() {
                    Ok(preload_commands) => {
                        // Convert preload commands to effects
                        for cmd in preload_commands {
                            effects.push(Effect::AudioCommand(cmd));
                        }
                        // Update mode in view model
                        view_model.mode = crate::app_state::Mode::Pads;
                        effects.push(Effect::StatusMessage(
                            "[Pads] Press Esc to go back. Press Q/W/…/< to trigger.".to_string(),
                        ));
                    }
                    Err(e) => {
                        effects.push(Effect::StatusMessage(e.to_string()));
                    }
                }
            }
            _ => {
                // Route keys based on focused pane
                match view_model.focus {
                    crate::app_state::FocusPane::LeftExplorer => {
                        match key {
                            KeyCode::Char(' ') => {
                                self.handle_file_selection(app_state, view_model, effects)?;
                            }
                            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                                self.handle_file_explorer_navigation(view_model, key, effects)?;
                            }
                            _ => {}
                        }
                    }
                    crate::app_state::FocusPane::RightSelected => {
                        self.handle_selection_management(app_state, view_model, key, effects)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles key presses in Pads mode.
    fn handle_pads_mode_key(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        key: KeyCode,
        modifiers: KeyModifiers,
        effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        // Handle popup if open
        if view_model.is_bpm_popup_open() {
            return self.handle_popup_key(app_state, view_model, key, effects);
        }

        match key {
            KeyCode::Esc => {
                app_state.cancel_loop();
                view_model.mode = crate::app_state::Mode::Browse;
                effects.push(Effect::StatusMessage("Back to browse".to_string()));
            }
            KeyCode::Char(' ') if modifiers.control => {
                app_state.clear_loop();
                effects.push(Effect::StatusMessage("Loop cleared".to_string()));
            }
            KeyCode::Char(' ') => {
                app_state.handle_loop_space();
                // Status message update based on loop state would be handled elsewhere
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                view_model.focus_summary_box();
            }
            KeyCode::Enter => {
                if matches!(
                    view_model.popup_focus(),
                    crate::app_state::PopupFocus::SummaryBox
                ) {
                    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());
                }
            }
            KeyCode::Char(c) => {
                let k = c.to_ascii_lowercase();
                if app_state.pads.key_to_slot.contains_key(&k) {
                    // Check debounce
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    if let Some(prev) = app_state.pads.last_press_ms.get(&k).cloned()
                        && now_ms.saturating_sub(prev) < 100
                    {
                        return Ok(());
                    }
                    app_state.pads.last_press_ms.insert(k, now_ms);
                    app_state.pads.active_keys.insert(k);

                    // Record loop event and potentially play audio
                    let loop_state = app_state.loop_state();
                    if !matches!(loop_state, LoopState::Recording { .. }) {
                        effects.push(Effect::AudioCommand(AudioCommand::Play { key: k }));
                    }
                    app_state.record_loop_event(k);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handles key presses when popup is open.
    fn handle_popup_key(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        key: KeyCode,
        _effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        use crate::app_state::PopupFocus;
        use ratatui::crossterm::event::Event;
        use tui_input::InputRequest;
        use tui_input::backend::crossterm::to_input_request;

        match key {
            KeyCode::Esc => {
                view_model.close_bpm_bars_popup();
            }
            KeyCode::Enter => match view_model.popup_focus() {
                PopupFocus::PopupOk => {
                    // Apply popup changes
                    let mut changed = false;
                    if let Ok(bpm) = view_model.draft_bpm().value().parse::<u16>() {
                        let before = app_state.get_bpm();
                        app_state.set_bpm(bpm);
                        if app_state.get_bpm() != before {
                            changed = true;
                        }
                    }
                    if let Ok(bars) = view_model.draft_bars().value().parse::<u16>() {
                        let before = app_state.get_bars();
                        app_state.set_bars(bars);
                        if app_state.get_bars() != before {
                            changed = true;
                        }
                    }
                    if changed {
                        app_state.reset_loop_for_tempo();
                    }
                    view_model.close_bpm_bars_popup();
                }
                PopupFocus::PopupCancel => {
                    view_model.close_bpm_bars_popup();
                }
                _ => {}
            },
            KeyCode::Up => {
                view_model.popup_focus_up();
            }
            KeyCode::Down => {
                view_model.popup_focus_down();
            }
            KeyCode::Left | KeyCode::Right => {
                view_model.popup_toggle_ok_cancel();
            }
            _ => {
                // Handle all other keys (including Char, Backspace, Delete, etc.) for text input
                // Convert KeyCode to KeyEvent for TextInput handling
                // This is a temporary workaround until we refactor TextInput to use InputAction
                if let Ok(event) = self.keycode_to_event(key) {
                    if let Event::Key(crossterm_key) = event {
                        if let Some(req) = to_input_request(&Event::Key(crossterm_key)) {
                            // Enforce digits-only input for InsertChar requests
                            let should_apply = match req {
                                InputRequest::InsertChar(ch) => ch.is_ascii_digit(),
                                _ => true,
                            };
                            if should_apply {
                                match view_model.popup_focus() {
                                    PopupFocus::PopupFieldBpm => {
                                        let _ = view_model.draft_bpm_mut().handle(req);
                                    }
                                    PopupFocus::PopupFieldBars => {
                                        let _ = view_model.draft_bars_mut().handle(req);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Temporary helper to convert KeyCode back to Event::Key for FileExplorer.
    /// TODO: Abstract FileExplorer behind a trait to avoid this dependency.
    fn keycode_to_event(&self, key: KeyCode) -> anyhow::Result<Event> {
        let crossterm_key = match key {
            KeyCode::Tab => CrosstermKeyCode::Tab,
            KeyCode::Enter => CrosstermKeyCode::Enter,
            KeyCode::Esc => CrosstermKeyCode::Esc,
            KeyCode::Up => CrosstermKeyCode::Up,
            KeyCode::Down => CrosstermKeyCode::Down,
            KeyCode::Left => CrosstermKeyCode::Left,
            KeyCode::Right => CrosstermKeyCode::Right,
            KeyCode::Char(c) => CrosstermKeyCode::Char(c),
            KeyCode::Delete => CrosstermKeyCode::Delete,
            KeyCode::Backspace => CrosstermKeyCode::Backspace,
            KeyCode::Other(_) => {
                return Err(anyhow::anyhow!("Cannot convert Other key code to Event"));
            }
        };
        Ok(Event::Key(KeyEvent::new(
            crossterm_key,
            CrosstermModifiers::empty(),
        )))
    }

    /// Handle file explorer navigation keys.
    fn handle_file_explorer_navigation(
        &self,
        view_model: &mut ViewModel,
        key: KeyCode,
        _effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        // Convert KeyCode to Event::Key for FileExplorer
        let event = self.keycode_to_event(key)?;
        view_model.file_explorer.handle(&event)?;

        // Update current_left_item based on explorer selection
        let idx = view_model.file_explorer.selected_idx();
        if let Some(entry) = view_model.file_explorer.files().get(idx) {
            view_model.current_left_item = Some(entry.path().to_path_buf());
            view_model.current_left_is_dir = entry.is_dir();
        }
        Ok(())
    }

    /// Handle file selection (Space key in left pane).
    fn handle_file_selection(
        &self,
        app_state: &mut ApplicationState,
        view_model: &mut ViewModel,
        effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        if view_model.current_left_is_dir {
            effects.push(Effect::StatusMessage("Only files can be selected".to_string()));
        } else if let Some(path) = view_model.current_left_item.clone() {
            app_state.selection.add_file(path);
            effects.push(Effect::StatusMessage(app_state.selection.status.clone()));
        }
        Ok(())
    }

    /// Handle selection management (Up/Down/Delete in right pane).
    fn handle_selection_management(
        &self,
        app_state: &mut ApplicationState,
        _view_model: &mut ViewModel,
        key: KeyCode,
        effects: &mut Vec<Effect>,
    ) -> anyhow::Result<()> {
        match key {
            KeyCode::Up => {
                app_state.selection.move_up();
            }
            KeyCode::Down => {
                app_state.selection.move_down();
            }
            KeyCode::Char(' ') | KeyCode::Delete | KeyCode::Char('d') => {
                let before_len = app_state.selection.items.len();
                app_state.selection.remove_at_cursor();
                if app_state.selection.items.len() < before_len {
                    effects.push(Effect::StatusMessage(app_state.selection.status.clone()));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
