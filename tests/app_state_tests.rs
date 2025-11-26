use std::path::PathBuf;
use std::sync::mpsc;

use termigroove::app_state::PopupFocus;
use termigroove::application::state::ApplicationState;
use termigroove::audio::{SenderAudioBus, SystemClock};
use termigroove::domain::r#loop::LoopEngine;
use termigroove::presentation::ViewModel;
use termigroove::selection::SelectionModel;
use ratatui_explorer::{FileExplorer, Theme as ExplorerTheme};
use ratatui::widgets::{Block, BorderType, Borders};
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

fn setup_test_state() -> (ApplicationState, ViewModel) {
    let (tx, _rx) = mpsc::channel();
    let bus = SenderAudioBus::new(tx);
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let app_state = ApplicationState::new(loop_engine);
    let theme = ExplorerTheme::default()
        .with_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    let file_explorer = FileExplorer::with_theme(theme).expect("create file explorer");
    let view_model = ViewModel::new(file_explorer);
    (app_state, view_model)
}

#[test]
fn bpm_bars_defaults_and_clamping() {
    let (mut app_state, _view_model) = setup_test_state();
    assert_eq!(app_state.get_bpm(), 120);
    assert_eq!(app_state.get_bars(), 16);

    app_state.set_bpm(5);
    assert_eq!(app_state.get_bpm(), 20);
    app_state.set_bpm(400);
    assert_eq!(app_state.get_bpm(), 300);

    app_state.set_bars(0);
    assert_eq!(app_state.get_bars(), 1);
    app_state.set_bars(999);
    assert_eq!(app_state.get_bars(), 256);
}

#[test]
fn open_and_close_popup_apply_and_discard() {
    let (mut app_state, mut view_model) = setup_test_state();
    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());
    assert!(view_model.is_bpm_popup_open());
    assert_eq!(view_model.popup_focus(), PopupFocus::PopupFieldBpm);

    set_input_text(view_model.draft_bpm_mut(), "130");
    set_input_text(view_model.draft_bars_mut(), "8");
    // Apply the values manually (simulating AppService behavior)
    if let Ok(bpm) = view_model.draft_bpm().value().parse::<u16>() {
        app_state.set_bpm(bpm);
    }
    if let Ok(bars) = view_model.draft_bars().value().parse::<u16>() {
        app_state.set_bars(bars);
    }
    view_model.close_bpm_bars_popup();
    assert_eq!(app_state.get_bpm(), 130);
    assert_eq!(app_state.get_bars(), 8);

    let bpm_before = app_state.get_bpm();
    let bars_before = app_state.get_bars();
    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());
    set_input_text(view_model.draft_bpm_mut(), "240");
    set_input_text(view_model.draft_bars_mut(), "32");
    // Don't apply (simulating discard)
    view_model.close_bpm_bars_popup();
    assert_eq!(app_state.get_bpm(), bpm_before);
    assert_eq!(app_state.get_bars(), bars_before);
}

#[test]
fn opening_popup_copies_current_values_into_drafts() {
    let (mut app_state, mut view_model) = setup_test_state();

    app_state.set_bpm(140);
    app_state.set_bars(12);
    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());
    assert_eq!(view_model.draft_bpm().value(), "140");
    assert_eq!(view_model.draft_bars().value(), "12");
    view_model.close_bpm_bars_popup();

    app_state.set_bpm(200);
    app_state.set_bars(8);
    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());
    assert_eq!(view_model.draft_bpm().value(), "200");
    assert_eq!(view_model.draft_bars().value(), "8");
}

#[test]
fn close_popup_apply_clamps_and_resets_state() {
    let (mut app_state, mut view_model) = setup_test_state();
    view_model.open_bpm_bars_popup(app_state.get_bpm(), app_state.get_bars());

    set_input_text(view_model.draft_bpm_mut(), "999");
    set_input_text(view_model.draft_bars_mut(), "0");
    // Apply the values manually (simulating AppService behavior with clamping)
    if let Ok(bpm) = view_model.draft_bpm().value().parse::<u16>() {
        app_state.set_bpm(bpm); // set_bpm clamps the value
    }
    if let Ok(bars) = view_model.draft_bars().value().parse::<u16>() {
        app_state.set_bars(bars); // set_bars clamps the value
    }
    view_model.close_bpm_bars_popup();

    assert_eq!(app_state.get_bpm(), 300);
    assert_eq!(app_state.get_bars(), 1);
    assert!(!view_model.is_bpm_popup_open());
    assert!(matches!(view_model.popup_focus(), PopupFocus::None));
    assert_eq!(view_model.draft_bpm().value(), "");
    assert_eq!(view_model.draft_bars().value(), "");
}

fn set_input_text(input: &mut TextInput, value: &str) {
    input.reset();
    for ch in value.chars() {
        let _ = input.handle(InputRequest::InsertChar(ch));
    }
}
