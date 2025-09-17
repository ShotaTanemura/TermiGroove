use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use termigroove::app_state::{AppState, FocusPane, Mode};
use termigroove::input::handle_event;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

#[test]
fn flow_add_two_from_left_then_navigate_and_remove_right_and_enter_pads() {
    let mut state = AppState::new().expect("state");
    assert!(matches!(state.mode, Mode::Browse));
    assert!(matches!(state.focus, FocusPane::LeftExplorer));

    // Select a file from the left (simulate explorer cursor on a file)
    state.current_left_item = Some("/tmp/a.wav".into());
    state.current_left_is_dir = false;
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 1);
    assert!(state.status_message.starts_with("Added "));

    // Select another file
    state.current_left_item = Some("/tmp/b.wav".into());
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 2);

    // Switch focus to right
    handle_event(&mut state, key(KeyCode::Tab)).unwrap();
    assert!(matches!(state.focus, FocusPane::RightSelected));

    // Navigate Up/Down within bounds
    assert_eq!(state.selection.right_idx, 1);
    handle_event(&mut state, key(KeyCode::Up)).unwrap();
    assert_eq!(state.selection.right_idx, 0);
    handle_event(&mut state, key(KeyCode::Up)).unwrap();
    assert_eq!(state.selection.right_idx, 0);
    handle_event(&mut state, key(KeyCode::Down)).unwrap();
    assert_eq!(state.selection.right_idx, 1);
    handle_event(&mut state, key(KeyCode::Down)).unwrap();
    assert_eq!(state.selection.right_idx, 1);

    // Remove selected with Space
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 1);
    assert!(state.status_message.starts_with("Removed "));

    // With ≥1 selected, Enter switches to Pads
    handle_event(&mut state, key(KeyCode::Enter)).unwrap();
    assert!(matches!(state.mode, Mode::Pads));
    assert!(state.status_message.starts_with("[Pads] "));
}

#[test]
fn left_directory_space_is_no_select_and_sets_status() {
    let mut state = AppState::new().unwrap();
    state.current_left_is_dir = true;
    let before_len = state.selection.items.len();
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), before_len);
    assert_eq!(state.status_message, "Only files can be selected");
}

#[test]
fn left_toggle_removes_existing_file() {
    let mut state = AppState::new().unwrap();
    // Add from left
    state.current_left_item = Some("/tmp/a.wav".into());
    state.current_left_is_dir = false;
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 1);
    // Toggle again -> remove
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    assert_eq!(state.selection.items.len(), 0);
    assert!(state.status_message.starts_with("Removed "));
}

#[test]
fn right_pane_empty_list_noops() {
    let mut state = AppState::new().unwrap();
    state.focus = FocusPane::RightSelected;
    let before_status = state.status_message.clone();
    handle_event(&mut state, key(KeyCode::Up)).unwrap();
    handle_event(&mut state, key(KeyCode::Down)).unwrap();
    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    handle_event(&mut state, key(KeyCode::Delete)).unwrap();
    handle_event(&mut state, key(KeyCode::Char('d'))).unwrap();
    assert_eq!(state.selection.items.len(), 0);
    assert_eq!(state.selection.right_idx, 0);
    assert_eq!(state.status_message, before_status);
}
