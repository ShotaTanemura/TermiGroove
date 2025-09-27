use std::path::PathBuf;

use termigroove::app_state::{AppState, PopupFocus};
use termigroove::selection::SelectionModel;
use tui_input::{Input as TextInput, InputRequest};

#[test]
fn add_appends_and_sets_status_and_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    assert_eq!(m.items.len(), 1);
    assert_eq!(m.right_idx, 0);
    assert!(m.status.starts_with("Added "));
}

#[test]
fn toggle_remove_via_add_file_removes_and_updates_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    // toggle a.wav by adding it again
    m.add_file(PathBuf::from("/tmp/a.wav"));
    assert_eq!(m.items.len(), 1);
    assert_eq!(m.items[0].file_name().unwrap(), "b.wav");
    assert!(m.status.starts_with("Removed "));
    // cursor should clamp to last
    assert_eq!(m.right_idx, 0);
}

#[test]
fn remove_at_cursor_repositions_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    m.add_file(PathBuf::from("/tmp/c.wav"));
    m.right_idx = 2;
    m.remove_at_cursor(); // remove c
    assert_eq!(m.items.len(), 2);
    assert_eq!(m.right_idx, 1); // now points to b
}

#[test]
fn move_up_down_bounds() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    m.right_idx = 0;
    m.move_up();
    assert_eq!(m.right_idx, 0);
    m.move_down();
    assert_eq!(m.right_idx, 1);
    m.move_down();
    assert_eq!(m.right_idx, 1);
}

#[test]
fn empty_list_noops_on_nav_and_remove() {
    let mut m = SelectionModel::default();
    // initial
    assert_eq!(m.items.len(), 0);
    assert_eq!(m.right_idx, 0);
    let before_status = m.status.clone();

    // No-ops should not panic and should not alter index/status
    m.move_up();
    m.move_down();
    m.remove_at_cursor();

    assert_eq!(m.items.len(), 0);
    assert_eq!(m.right_idx, 0);
    assert_eq!(m.status, before_status);
}

#[test]
fn bpm_bars_defaults_and_clamping() {
    let mut state = AppState::new().expect("init AppState");
    assert_eq!(state.get_bpm(), 120);
    assert_eq!(state.get_bars(), 16);

    state.set_bpm(5);
    assert_eq!(state.get_bpm(), 20);
    state.set_bpm(400);
    assert_eq!(state.get_bpm(), 300);

    state.set_bars(0);
    assert_eq!(state.get_bars(), 1);
    state.set_bars(999);
    assert_eq!(state.get_bars(), 256);
}

#[test]
fn open_and_close_popup_apply_and_discard() {
    let mut state = AppState::new().expect("init AppState");
    state.open_bpm_bars_popup();
    assert!(state.is_bpm_popup_open());
    assert_eq!(state.popup_focus(), PopupFocus::PopupFieldBpm);

    set_input_text(state.draft_bpm_mut(), "130");
    set_input_text(state.draft_bars_mut(), "8");
    state.close_bpm_bars_popup(true);
    assert_eq!(state.get_bpm(), 130);
    assert_eq!(state.get_bars(), 8);

    let bpm_before = state.get_bpm();
    let bars_before = state.get_bars();
    state.open_bpm_bars_popup();
    set_input_text(state.draft_bpm_mut(), "240");
    set_input_text(state.draft_bars_mut(), "32");
    state.close_bpm_bars_popup(false);
    assert_eq!(state.get_bpm(), bpm_before);
    assert_eq!(state.get_bars(), bars_before);
}

#[test]
fn opening_popup_copies_current_values_into_drafts() {
    let mut state = AppState::new().expect("init AppState");

    state.set_bpm(140);
    state.set_bars(12);
    state.open_bpm_bars_popup();
    assert_eq!(state.draft_bpm().value(), "140");
    assert_eq!(state.draft_bars().value(), "12");
    state.close_bpm_bars_popup(false);

    state.set_bpm(200);
    state.set_bars(8);
    state.open_bpm_bars_popup();
    assert_eq!(state.draft_bpm().value(), "200");
    assert_eq!(state.draft_bars().value(), "8");
}

#[test]
fn close_popup_apply_clamps_and_resets_state() {
    let mut state = AppState::new().expect("init AppState");
    state.open_bpm_bars_popup();

    set_input_text(state.draft_bpm_mut(), "999");
    set_input_text(state.draft_bars_mut(), "0");
    state.close_bpm_bars_popup(true);

    assert_eq!(state.get_bpm(), 300);
    assert_eq!(state.get_bars(), 1);
    assert!(!state.is_bpm_popup_open());
    assert!(matches!(state.popup_focus(), PopupFocus::None));
    assert_eq!(state.draft_bpm().value(), "");
    assert_eq!(state.draft_bars().value(), "");
}

fn set_input_text(input: &mut TextInput, value: &str) {
    input.reset();
    for ch in value.chars() {
        let _ = input.handle(InputRequest::InsertChar(ch));
    }
}
