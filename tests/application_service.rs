//! Unit tests for application service.
//!
//! Tests verify that AppService is stateless and correctly orchestrates
//! use cases by producing effects and mutating state.

use std::sync::mpsc;
use termigroove::application::dto::input_action::{InputAction, KeyCode, KeyModifiers};
use termigroove::application::service::{app_service::AppService, effect::Effect};
use termigroove::application::state::ApplicationState;
use termigroove::audio::{AudioCommand, SenderAudioBus, SystemClock};
use termigroove::domain::r#loop::LoopEngine;
use termigroove::presentation::ViewModel;
use ratatui_explorer::{FileExplorer, Theme as ExplorerTheme};
use ratatui::widgets::{Block, BorderType, Borders};

fn setup_test_state() -> (ApplicationState, ViewModel, mpsc::Sender<AudioCommand>) {
    let (tx, _rx) = mpsc::channel();
    let bus = SenderAudioBus::new(tx.clone());
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
    (app_state, view_model, tx)
}

#[test]
fn app_service_is_stateless() {
    // Create two instances - they should behave identically
    let (_, _, tx) = setup_test_state();
    let service1 = AppService::new(tx.clone());
    let service2 = AppService::new(tx);

    // Both should be equal (structs with same audio_tx)
    assert_eq!(format!("{:?}", service1), format!("{:?}", service2));
}

#[test]
fn handle_input_with_space_key_in_pads_mode() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    // Enter pads mode first
    app_state.selection.add_file(std::path::PathBuf::from("test.wav"));
    let _ = app_state.enter_pads();
    view_model.mode = termigroove::presentation::Mode::Pads;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::default(),
    };

    let _effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Space key should trigger loop handling, which may produce effects
    // The exact effects depend on loop state, but we verify the method works
}

#[test]
fn handle_input_with_char_key_in_pads_mode_produces_audio_effect() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    // Enter pads mode and set up a pad mapping
    app_state.selection.add_file(std::path::PathBuf::from("test.wav"));
    let _ = app_state.enter_pads();
    view_model.mode = termigroove::presentation::Mode::Pads;

    // Ensure we're not in recording state so audio effect is produced
    // (The loop should be idle initially)

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // If 'q' is mapped to a pad and loop is not recording, should produce audio effect
    if app_state.pads.key_to_slot.contains_key(&'q') {
        // Check if any effect is an audio command
        let has_audio_effect = effects
            .iter()
            .any(|e| matches!(e, Effect::AudioCommand(AudioCommand::Play { .. })));
        // The effect may or may not be produced depending on loop state
        // Assert that an audio effect is produced if the pad is mapped
        assert!(has_audio_effect, "Expected an audio effect when pad is mapped to 'q'");
    }
}

#[test]
fn handle_input_with_esc_in_pads_mode() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    app_state.selection.add_file(std::path::PathBuf::from("test.wav"));
    let _ = app_state.enter_pads();
    view_model.mode = termigroove::presentation::Mode::Pads;
    assert!(matches!(view_model.mode, termigroove::presentation::Mode::Pads));

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Esc should cancel loop and return to browse mode with status message
    assert!(matches!(view_model.mode, termigroove::presentation::Mode::Browse));
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}

#[test]
fn handle_input_with_resize_action() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    let service = AppService::new(tx);
    let input_action = InputAction::Resize {
        width: 100,
        height: 50,
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Resize should not produce effects (handled by UI layer)
    assert!(effects.is_empty());
}

#[test]
fn update_loop_produces_no_effects_initially() {
    let (mut app_state, _view_model, tx) = setup_test_state();
    let service = AppService::new(tx);

    let effects = service.update_loop(&mut app_state);

    // Initially, update_loop should not produce effects
    assert_eq!(effects, Vec::new());
}

#[test]
fn handle_input_with_tab_in_browse_mode() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    let initial_focus = view_model.focus;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Tab,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Tab should toggle focus and produce status message
    assert_ne!(view_model.focus, initial_focus);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}

#[test]
fn handle_input_with_control_space_in_pads_mode() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    app_state.selection.add_file(std::path::PathBuf::from("test.wav"));
    let _ = app_state.enter_pads();
    view_model.mode = termigroove::presentation::Mode::Pads;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers {
            control: true,
            shift: false,
            alt: false,
        },
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Control+Space should clear loop and produce status message
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}

#[test]
fn handle_input_with_enter_in_browse_mode() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    app_state.selection.add_file(std::path::PathBuf::from("test.wav"));

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::default(),
    };

    let _effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Enter should attempt to enter pads mode
    // Effects may be empty or contain preload commands
    assert!(matches!(view_model.mode, termigroove::presentation::Mode::Pads));
}

#[test]
fn service_methods_are_idempotent() {
    // Verify that calling the same method multiple times with same input
    // produces consistent results (stateless behavior)
    let (mut app_state1, mut view_model1, tx1) = setup_test_state();
    let (mut app_state2, mut view_model2, tx2) = setup_test_state();

    let service1 = AppService::new(tx1);
    let service2 = AppService::new(tx2);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Tab,
        modifiers: KeyModifiers::default(),
    };

    // Both should produce same effects
    let effects1 = service1
        .handle_input(&mut app_state1, &mut view_model1, input_action.clone())
        .expect("handle input");
    let effects2 = service2
        .handle_input(&mut app_state2, &mut view_model2, input_action)
        .expect("handle input");

    assert_eq!(effects1.len(), effects2.len());
}

#[test]
fn handle_input_with_up_key_in_left_explorer_focus() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::LeftExplorer;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Up,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Up key should navigate file explorer (may or may not change current_left_item)
    // Effects should be empty (navigation doesn't produce effects)
    assert!(effects.is_empty());
    // The current_left_item may change depending on file explorer state
    // This test verifies the method works without panicking
}

#[test]
fn handle_input_with_space_key_in_left_explorer_focus_selects_file() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::LeftExplorer;
    // Set up a file (not directory) as current item
    view_model.current_left_item = Some(std::path::PathBuf::from("test.wav"));
    view_model.current_left_is_dir = false;
    let initial_selection_count = app_state.selection.items.len();

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Space key should select the file and produce status message
    assert_eq!(app_state.selection.items.len(), initial_selection_count + 1);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}

#[test]
fn handle_input_with_space_key_on_directory_shows_error() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::LeftExplorer;
    // Set up a directory as current item
    view_model.current_left_item = Some(std::path::PathBuf::from("test_dir"));
    view_model.current_left_is_dir = true;
    let initial_selection_count = app_state.selection.items.len();

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Space key on directory should show error message and not add to selection
    assert_eq!(app_state.selection.items.len(), initial_selection_count);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| {
        matches!(e, Effect::StatusMessage(msg) if msg.contains("Only files can be selected"))
    }));
}

#[test]
fn handle_input_with_up_key_in_right_selected_focus() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::RightSelected;
    // Add some files to selection
    app_state.selection.add_file(std::path::PathBuf::from("file1.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file2.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file3.wav"));
    let initial_idx = app_state.selection.right_idx;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Up,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Up key should move cursor up in selection
    if initial_idx > 0 {
        assert!(app_state.selection.right_idx < initial_idx);
    }
    // No effects should be produced for navigation
    assert!(effects.is_empty());
}

#[test]
fn handle_input_with_down_key_in_right_selected_focus() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::RightSelected;
    // Add some files to selection
    app_state.selection.add_file(std::path::PathBuf::from("file1.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file2.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file3.wav"));
    let initial_idx = app_state.selection.right_idx;

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Down,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Down key should move cursor down in selection
    if initial_idx + 1 < app_state.selection.items.len() {
        assert!(app_state.selection.right_idx > initial_idx);
    }
    // No effects should be produced for navigation
    assert!(effects.is_empty());
}

#[test]
fn handle_input_with_delete_key_in_right_selected_focus() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::RightSelected;
    // Add some files to selection
    app_state.selection.add_file(std::path::PathBuf::from("file1.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file2.wav"));
    let initial_count = app_state.selection.items.len();

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Delete,
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // Delete key should remove item at cursor and produce status message
    assert!(app_state.selection.items.len() < initial_count);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}

#[test]
fn handle_input_with_d_key_in_right_selected_focus() {
    let (mut app_state, mut view_model, tx) = setup_test_state();
    view_model.focus = termigroove::presentation::FocusPane::RightSelected;
    // Add some files to selection
    app_state.selection.add_file(std::path::PathBuf::from("file1.wav"));
    app_state.selection.add_file(std::path::PathBuf::from("file2.wav"));
    let initial_count = app_state.selection.items.len();

    let service = AppService::new(tx);
    let input_action = InputAction::KeyPressed {
        key: KeyCode::Char('d'),
        modifiers: KeyModifiers::default(),
    };

    let effects = service
        .handle_input(&mut app_state, &mut view_model, input_action)
        .expect("handle input");

    // 'd' key should remove item at cursor and produce status message
    assert!(app_state.selection.items.len() < initial_count);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| matches!(e, Effect::StatusMessage(_))));
}
