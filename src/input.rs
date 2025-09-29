use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app_state::AppState;
use crate::app_state::{Mode, PopupFocus};
use std::time::{SystemTime, UNIX_EPOCH};
use tui_input::InputRequest;
use tui_input::backend::crossterm::to_input_request;

pub fn handle_event(state: &mut AppState, event: Event) -> anyhow::Result<()> {
    match event {
        Event::Key(key) => handle_key_event(state, key)?,
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

fn handle_left_pane_key(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Char(' ') => {
            if state.current_left_is_dir {
                state.status_message = "Only files can be selected".to_string();
            } else if let Some(path) = state.current_left_item.clone() {
                state.toggle_select_file(path);
            }
            Ok(())
        }
        _ => {
            // Forward unhandled keys to the explorer
            let event = Event::Key(key);
            state.file_explorer.handle(&event)?;
            // Update current_left_item/dir based on explorer selection
            let idx = state.file_explorer.selected_idx();
            if let Some(entry) = state.file_explorer.files().get(idx) {
                state.current_left_item = Some(entry.path().to_path_buf());
                state.current_left_is_dir = entry.is_dir();
            }
            Ok(())
        }
    }
}

fn handle_right_pane_key(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up => {
            state.selection.move_up();
            Ok(())
        }
        KeyCode::Down => {
            state.selection.move_down();
            Ok(())
        }
        KeyCode::Char(' ') | KeyCode::Delete => {
            let before_len = state.selection.items.len();
            state.selection.remove_at_cursor();
            if state.selection.items.len() < before_len {
                state.status_message = state.selection.status.clone();
            }
            Ok(())
        }
        KeyCode::Char('d') => {
            let before_len = state.selection.items.len();
            state.selection.remove_at_cursor();
            if state.selection.items.len() < before_len {
                state.status_message = state.selection.status.clone();
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_pads_key_event(state: &mut AppState, key: KeyEvent) -> anyhow::Result<()> {
    // If popup is open, handle its lifecycle
    if state.is_bpm_popup_open() {
        match key.code {
            KeyCode::Esc => state.close_bpm_bars_popup(false),
            KeyCode::Enter => match state.popup_focus() {
                PopupFocus::PopupOk => state.close_bpm_bars_popup(true),
                PopupFocus::PopupCancel => state.close_bpm_bars_popup(false),
                _ => {}
            },
            KeyCode::Up => state.popup_focus_up(),
            KeyCode::Down => state.popup_focus_down(),
            KeyCode::Left | KeyCode::Right => state.popup_toggle_ok_cancel(),
            _ => {
                if let Some(req) = to_input_request(&Event::Key(key)) {
                    // Enforce digits-only input for InsertChar requests
                    let should_apply = match req {
                        InputRequest::InsertChar(c) => c.is_ascii_digit(),
                        _ => true,
                    };
                    if should_apply {
                        match state.popup_focus() {
                            PopupFocus::PopupFieldBpm => {
                                let _ = state.draft_bpm_mut().handle(req);
                            }
                            PopupFocus::PopupFieldBars => {
                                let _ = state.draft_bars_mut().handle(req);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            state.cancel_loop();
            state.mode = Mode::Browse;
            state.status_message = "Back to browse".to_string();
            Ok(())
        }
        KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.clear_loop();
            Ok(())
        }
        KeyCode::Char(' ') => {
            state.handle_loop_space();
            Ok(())
        }
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            // Any arrow focuses the summary box
            state.focus_summary_box();
            Ok(())
        }
        KeyCode::Enter => {
            if matches!(state.popup_focus(), PopupFocus::SummaryBox) {
                state.open_bpm_bars_popup();
            }
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
            state.record_loop_event(k);
            Ok(())
        }
        _ => Ok(()),
    }
}
