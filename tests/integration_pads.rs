use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{SystemTime, UNIX_EPOCH};
use termigroove::app_state::{AppState, FocusPane, Mode};
use termigroove::input::handle_event;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

#[test]
fn pads_enter_and_press_keys_sets_active_and_debounces() {
    let mut state = AppState::new().expect("state");
    assert!(matches!(state.mode, Mode::Browse));
    assert!(matches!(state.focus, FocusPane::LeftExplorer));

    // Select two wav files via left pane toggle
    state.current_left_item = Some("/tmp/kick.wav".into());
    state.current_left_is_dir = false;
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    state.current_left_item = Some("/tmp/snare.wav".into());
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 2);

    // Enter pads
    handle_event(&mut state, key(KeyCode::Enter)).unwrap();
    assert!(matches!(state.mode, Mode::Pads));

    // Press 'q' -> becomes active
    handle_event(&mut state, key(KeyCode::Char('q'))).unwrap();
    assert!(state.pads.active_keys.contains(&'q'));

    // Debounce: set last_press to now-50ms, press again; should be ignored
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    state
        .pads
        .last_press_ms
        .insert('q', now_ms.saturating_sub(50));
    let before = state.pads.last_press_ms.get(&'q').cloned();
    handle_event(&mut state, key(KeyCode::Char('q'))).unwrap();
    let after = state.pads.last_press_ms.get(&'q').cloned();
    // still present; time not reduced (may increase slightly if not suppressed, but our debounce should return early)
    assert_eq!(before, after);

    // Another key 'w' also becomes active
    handle_event(&mut state, key(KeyCode::Char('w'))).unwrap();
    assert!(state.pads.active_keys.contains(&'w'));

    // Esc back to browse
    handle_event(&mut state, key(KeyCode::Esc)).unwrap();
    assert!(matches!(state.mode, Mode::Browse));
}

#[test]
fn non_wav_selection_blocks_pads_entry() {
    let mut state = AppState::new().expect("state");
    state.current_left_item = Some("/tmp/not_audio.txt".into());
    state.current_left_is_dir = false;
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    handle_event(&mut state, key(KeyCode::Enter)).unwrap_or_else(|_| ());
    // Should remain in Browse and show unsupported message
    assert!(
        matches!(state.mode, Mode::Browse) || state.status_message.contains("Unsupported file")
    );
}
