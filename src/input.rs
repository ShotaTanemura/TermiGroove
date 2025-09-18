use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};

use crate::app_state::AppState;
use crate::app_state::Mode;
use crate::audio::AudioCommand;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle_event(_state: &mut AppState, _event: Event) -> anyhow::Result<()> {
    match _event {
        Event::Key(_key) => handle_key_event(_state, _key)?,
        Event::Resize(_, _) => {
            // No-op for now; UI will recalc layout during draw
        }
        _ => {}
    }
    Ok(())
}

fn handle_key_event(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    match state.mode {
        Mode::Browse => match key.code {
            KeyCode::Tab => {
                state.toggle_focus();
                state.status_message = state.focus_status_message();
            }
            KeyCode::Enter => {
                if let Err(_e) = state.enter_pads() {
                    // status_message already set inside enter_pads on error paths
                }
            }
            _ => route_key_to_focused_pane(state, key)?,
        },
        Mode::Pads => handle_pads_key_event(state, key)?,
    }
    Ok(())
}

fn route_key_to_focused_pane(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    match state.focus {
        // In a later step we will forward unhandled keys to the explorer component.
        crate::app_state::FocusPane::LeftExplorer => handle_left_pane_key(state, key),
        crate::app_state::FocusPane::RightSelected => handle_right_pane_key(state, key),
    }
}

fn handle_left_pane_key(_state: &mut AppState, _key: KeyEvent) -> anyhow::Result<()> {
    match _key.code {
        KeyCode::Char(' ') => {
            if _state.current_left_is_dir {
                _state.status_message = "Only files can be selected".to_string();
            } else if let Some(path) = _state.current_left_item.clone() {
                _state.toggle_select_file(path);
            }
            Ok(())
        }
        _ => {
            // Forward unhandled keys to the explorer
            let event = Event::Key(_key);
            _state.file_explorer.handle(&event)?;
            // Update current_left_item/dir based on explorer selection
            let idx = _state.file_explorer.selected_idx();
            if let Some(entry) = _state.file_explorer.files().get(idx) {
                _state.current_left_item = Some(entry.path().to_path_buf());
                _state.current_left_is_dir = entry.is_dir();
            }
            Ok(())
        }
    }
}

fn handle_right_pane_key(_state: &mut AppState, _key: KeyEvent) -> anyhow::Result<()> {
    match _key.code {
        KeyCode::Up => {
            _state.selection.move_up();
            Ok(())
        }
        KeyCode::Down => {
            _state.selection.move_down();
            Ok(())
        }
        KeyCode::Char(' ') | KeyCode::Delete => {
            let before_len = _state.selection.items.len();
            _state.selection.remove_at_cursor();
            if _state.selection.items.len() < before_len {
                _state.status_message = _state.selection.status.clone();
            }
            Ok(())
        }
        KeyCode::Char('d') => {
            let before_len = _state.selection.items.len();
            _state.selection.remove_at_cursor();
            if _state.selection.items.len() < before_len {
                _state.status_message = _state.selection.status.clone();
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_pads_key_event(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            state.mode = Mode::Browse;
            state.status_message = "Back to browse".to_string();
            Ok(())
        }
        KeyCode::Char(c) => {
            let k = c.to_ascii_lowercase();
            if !state.pads.key_to_slot.contains_key(&k) {
                return Ok(());
            }
            // Debounce auto-repeat within 100ms
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            if let Some(prev) = state.pads.last_press_ms.get(&k).cloned()
                && now_ms.saturating_sub(prev) < 100
            {
                return Ok(());
            }
            state.pads.last_press_ms.insert(k, now_ms);
            state.pads.active_keys.insert(k);
            let _ = state.audio_tx.send(AudioCommand::Play { key: k });
            Ok(())
        }
        _ => Ok(()),
    }
}
