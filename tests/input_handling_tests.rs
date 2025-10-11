use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use termigroove::app_state::{AppState, FocusPane, Mode, PopupFocus, SampleSlot};
use termigroove::audio::AudioCommand;
use termigroove::input::handle_event;
use termigroove::state::loop_engine::{LoopEngine, LoopState, SenderAudioBus, SystemClock};

#[test]
fn arrow_focuses_summary_box_and_enter_opens_popup() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Pads;

    // Arrow should focus summary box
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::SummaryBox));

    // Enter should open popup and focus BPM field
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert!(state.is_bpm_popup_open());
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBpm));
}

#[test]
fn popup_confirm_and_cancel() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Pads;
    // open popup
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert!(state.is_bpm_popup_open());

    // Move down to OK, then confirm
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupOk));
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert!(!state.is_bpm_popup_open());

    // Reopen and cancel via Esc
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert!(state.is_bpm_popup_open());
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Esc)),
    )
    .unwrap();
    assert!(!state.is_bpm_popup_open());
}

fn open_popup(state: &mut AppState) {
    handle_event(
        state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    handle_event(
        state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert!(state.is_bpm_popup_open());
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBpm));
}

#[test]
fn popup_focus_traversal_loops_through_fields_and_buttons() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Pads;
    open_popup(&mut state);

    // Down from BPM -> Bars
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBars));

    // Down -> OK
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupOk));

    // Down should loop back to BPM (cycle BPM -> Bars -> OK -> BPM)
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBpm));

    // Up should go to OK
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Up)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupOk));

    // Left/right toggles OK <-> Cancel
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupCancel));

    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Left)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupOk));

    // Up from OK should reach Bars then BPM
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Up)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBars));
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Up)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupFieldBpm));
}

#[test]
fn popup_digit_input_updates_drafts_and_clamps_on_confirm() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Pads;
    open_popup(&mut state);

    // Clear default BPM ("120") then type 3-5-0 -> "350"
    for _ in 0..3 {
        handle_event(
            &mut state,
            ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Backspace)),
        )
        .unwrap();
    }
    assert_eq!(state.draft_bpm().value(), "");

    for digit in ['3', '5', '0'] {
        handle_event(
            &mut state,
            ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Char(digit))),
        )
        .unwrap();
    }
    assert_eq!(state.draft_bpm().value(), "350");

    // Move down to bars and enter 2-5-7 -> "257"
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    for _ in 0..2 {
        handle_event(
            &mut state,
            ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Backspace)),
        )
        .unwrap();
    }
    assert_eq!(state.draft_bars().value(), "");

    for digit in ['2', '5', '7'] {
        handle_event(
            &mut state,
            ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Char(digit))),
        )
        .unwrap();
    }
    assert_eq!(state.draft_bars().value(), "257");

    // Non-digit input should be ignored
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Char('x'))),
    )
    .unwrap();
    assert_eq!(state.draft_bars().value(), "257");

    // Left/right movement should move caret but not change content
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Left)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    assert_eq!(state.draft_bars().value(), "257");

    // Navigate to OK (loop until focus is on OK)
    for _ in 0..3 {
        handle_event(
            &mut state,
            ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
        )
        .unwrap();
        if matches!(state.popup_focus(), PopupFocus::PopupOk) {
            break;
        }
    }
    assert!(matches!(state.popup_focus(), PopupFocus::PopupOk));
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();

    // Values should clamp to configured ranges (BPM -> 300, Bars -> 256)
    assert_eq!(state.get_bpm(), 300);
    assert_eq!(state.get_bars(), 256);
    assert!(!state.is_bpm_popup_open());
}

#[test]
fn popup_esc_discard_and_cancel_button_restore_previous_values() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(128);
    state.set_bars(32);

    // Open popup and modify values, then Esc to discard
    open_popup(&mut state);
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Char('9'))),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Esc)),
    )
    .unwrap();
    assert_eq!(state.get_bpm(), 128);
    assert_eq!(state.get_bars(), 32);
    assert!(!state.is_bpm_popup_open());

    // Reopen and use Cancel
    open_popup(&mut state);
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Down)),
    )
    .unwrap();
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Right)),
    )
    .unwrap();
    assert!(matches!(state.popup_focus(), PopupFocus::PopupCancel));
    handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(KeyEvent::from(KeyCode::Enter)),
    )
    .unwrap();
    assert_eq!(state.get_bpm(), 128);
    assert_eq!(state.get_bars(), 32);
    assert!(!state.is_bpm_popup_open());
}

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

fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn advance_until<F>(state: &mut AppState, predicate: F, iters: usize)
where
    F: Fn(LoopState) -> bool,
{
    for _ in 0..iters {
        state.update_loop();
        if predicate(state.loop_state()) {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn setup_loop_state() -> (AppState, std::sync::mpsc::Receiver<AudioCommand>) {
    let (tx, rx) = mpsc::channel();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(240);
    state.set_bars(1);
    state.pads.key_to_slot.insert(
        'q',
        SampleSlot {
            file_name: "kick.wav".into(),
        },
    );
    (state, rx)
}

#[test]
fn ctrl_space_clears_without_pausing() {
    let (mut state, rx) = setup_loop_state();

    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();
    advance_until(&mut state, |ls| matches!(ls, LoopState::Ready { .. }), 50);
    advance_until(
        &mut state,
        |ls| matches!(ls, LoopState::Recording { .. }),
        50,
    );
    handle_event(&mut state, key(KeyCode::Char('q'))).unwrap();
    advance_until(&mut state, |ls| matches!(ls, LoopState::Playing { .. }), 50);

    handle_event(
        &mut state,
        key_with_mod(KeyCode::Char(' '), KeyModifiers::CONTROL),
    )
    .unwrap();

    assert!(matches!(state.loop_state(), LoopState::Idle));
    assert_eq!(state.status_message, "Loop cleared");

    drop(state);
    thread::sleep(Duration::from_millis(20));
    let cmds = rx.try_iter().collect::<Vec<_>>();
    assert!(
        cmds.iter()
            .all(|cmd| !matches!(cmd, AudioCommand::PauseAll))
    );
}

#[test]
fn browse_space_selection_does_not_pause() {
    let mut state = AppState::new().expect("init");
    state.mode = Mode::Browse;
    state.current_left_item = Some("/tmp/a.wav".into());
    state.current_left_is_dir = false;

    handle_event(&mut state, key(KeyCode::Char(' '))).unwrap();

    assert!(matches!(state.loop_state(), LoopState::Idle));
    assert!(!state.status_message.contains("paused"));
}
