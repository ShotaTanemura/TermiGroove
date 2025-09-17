use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use termigroove::app_state::{AppState, FocusPane, Mode};
use termigroove::input::handle_event;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

#[test]
fn tab_toggles_focus_and_updates_status() {
    let mut state = AppState::new().unwrap();
    assert!(matches!(state.focus, FocusPane::LeftExplorer));
    handle_event(&mut state, key(KeyCode::Tab)).unwrap();
    assert!(matches!(state.focus, FocusPane::RightSelected));
    assert!(state.status_message.contains("Right focus"));
    handle_event(&mut state, key(KeyCode::Tab)).unwrap();
    assert!(matches!(state.focus, FocusPane::LeftExplorer));
}

#[test]
fn enter_requires_selection_then_switches_to_pads() {
    let mut state = AppState::new().unwrap();
    // No selection -> requires selection
    handle_event(&mut state, key(KeyCode::Enter)).unwrap();
    assert_eq!(state.status_message, "Select at least one file first");

    // Add a selection directly via model
    state
        .selection
        .add_file(std::path::PathBuf::from("/tmp/a.wav"));
    handle_event(&mut state, key(KeyCode::Enter)).unwrap();
    assert!(matches!(state.mode, Mode::Pads));
    assert!(state.status_message.starts_with("[Pads] "));
}

#[test]
fn right_pane_up_down_move_cursor_in_bounds() {
    let mut state = AppState::new().unwrap();
    // prepare some items
    state.selection.add_file("/tmp/a.wav".into());
    state.selection.add_file("/tmp/b.wav".into());
    state.focus = FocusPane::RightSelected;
    assert_eq!(state.selection.right_idx, 1); // cursor at last added

    // Up from 1 -> 0
    handle_event(&mut state, key(KeyCode::Up)).unwrap();
    assert_eq!(state.selection.right_idx, 0);
    // Up at 0 stays 0
    handle_event(&mut state, key(KeyCode::Up)).unwrap();
    assert_eq!(state.selection.right_idx, 0);
    // Down -> 1
    handle_event(&mut state, key(KeyCode::Down)).unwrap();
    assert_eq!(state.selection.right_idx, 1);
    // Down at last stays last
    handle_event(&mut state, key(KeyCode::Down)).unwrap();
    assert_eq!(state.selection.right_idx, 1);
}

#[test]
fn right_pane_remove_space_delete_d() {
    let mut state = AppState::new().unwrap();
    state.selection.add_file("/tmp/a.wav".into());
    state.selection.add_file("/tmp/b.wav".into());
    state.focus = FocusPane::RightSelected;

    // Remove with Space (removes b)
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 1);
    assert!(state.status_message.starts_with("Removed "));

    // Re-add b and then remove with Delete
    state.selection.add_file("/tmp/b.wav".into());
    handle_event(&mut state, key(KeyCode::Delete)).unwrap();
    assert_eq!(state.selection.items.len(), 1);

    // Re-add b and then remove with 'd'
    state.selection.add_file("/tmp/b.wav".into());
    handle_event(&mut state, key(KeyCode::Char('d'))).unwrap();
    assert_eq!(state.selection.items.len(), 1);
}
