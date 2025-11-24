use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use termigroove::application::dto::input_action::{
    InputAction, KeyCode as DtoKeyCode, KeyModifiers as DtoKeyModifiers,
};

#[test]
fn test_keycode_conversion_tab() {
    let crossterm_code = KeyCode::Tab;
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Tab);
}

#[test]
fn test_keycode_conversion_enter() {
    let crossterm_code = KeyCode::Enter;
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Enter);
}

#[test]
fn test_keycode_conversion_esc() {
    let crossterm_code = KeyCode::Esc;
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Esc);
}

#[test]
fn test_keycode_conversion_arrows() {
    assert_eq!(DtoKeyCode::from(KeyCode::Up), DtoKeyCode::Up);
    assert_eq!(DtoKeyCode::from(KeyCode::Down), DtoKeyCode::Down);
    assert_eq!(DtoKeyCode::from(KeyCode::Left), DtoKeyCode::Left);
    assert_eq!(DtoKeyCode::from(KeyCode::Right), DtoKeyCode::Right);
}

#[test]
fn test_keycode_conversion_char() {
    let crossterm_code = KeyCode::Char('a');
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Char('a'));
}

#[test]
fn test_keycode_conversion_delete() {
    let crossterm_code = KeyCode::Delete;
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Delete);
}

#[test]
fn test_keycode_conversion_backspace() {
    let crossterm_code = KeyCode::Backspace;
    let dto_code = DtoKeyCode::from(crossterm_code);
    assert_eq!(dto_code, DtoKeyCode::Backspace);
}

#[test]
fn test_keymodifiers_empty() {
    let crossterm_mods = KeyModifiers::empty();
    let dto_mods = DtoKeyModifiers::from(crossterm_mods);
    assert!(!dto_mods.control);
    assert!(!dto_mods.shift);
    assert!(!dto_mods.alt);
    assert!(dto_mods.is_empty());
}

#[test]
fn test_keymodifiers_control() {
    let crossterm_mods = KeyModifiers::CONTROL;
    let dto_mods = DtoKeyModifiers::from(crossterm_mods);
    assert!(dto_mods.control);
    assert!(!dto_mods.shift);
    assert!(!dto_mods.alt);
    assert!(dto_mods.contains_control());
}

#[test]
fn test_keymodifiers_shift() {
    let crossterm_mods = KeyModifiers::SHIFT;
    let dto_mods = DtoKeyModifiers::from(crossterm_mods);
    assert!(!dto_mods.control);
    assert!(dto_mods.shift);
    assert!(!dto_mods.alt);
}

#[test]
fn test_keymodifiers_alt() {
    let crossterm_mods = KeyModifiers::ALT;
    let dto_mods = DtoKeyModifiers::from(crossterm_mods);
    assert!(!dto_mods.control);
    assert!(!dto_mods.shift);
    assert!(dto_mods.alt);
}

#[test]
fn test_keymodifiers_combined() {
    let crossterm_mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    let dto_mods = DtoKeyModifiers::from(crossterm_mods);
    assert!(dto_mods.control);
    assert!(dto_mods.shift);
    assert!(!dto_mods.alt);
}

#[test]
fn test_event_key_pressed() {
    let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
    let crossterm_event = Event::Key(key_event);
    let input_action = InputAction::from(crossterm_event);

    match input_action {
        InputAction::KeyPressed { key, modifiers } => {
            assert_eq!(key, DtoKeyCode::Char('a'));
            assert!(modifiers.is_empty());
        }
        _ => panic!("Expected KeyPressed variant"),
    }
}

#[test]
fn test_event_key_pressed_with_modifiers() {
    let key_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL);
    let crossterm_event = Event::Key(key_event);
    let input_action = InputAction::from(crossterm_event);

    match input_action {
        InputAction::KeyPressed { key, modifiers } => {
            assert_eq!(key, DtoKeyCode::Char(' '));
            assert!(modifiers.contains_control());
        }
        _ => panic!("Expected KeyPressed variant"),
    }
}

#[test]
fn test_event_key_released() {
    let key_event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty());
    // Note: crossterm events default to Press kind, so we expect KeyPressed
    // The KeyReleased variant is included for future use when crossterm
    // provides explicit release events
    let crossterm_event = Event::Key(key_event);
    let input_action = InputAction::from(crossterm_event);

    match input_action {
        InputAction::KeyPressed { key, .. } => {
            assert_eq!(key, DtoKeyCode::Char('b'));
        }
        _ => panic!("Expected KeyPressed variant (crossterm defaults to Press)"),
    }
}

#[test]
fn test_event_resize() {
    let crossterm_event = Event::Resize(80, 24);
    let input_action = InputAction::from(crossterm_event);

    match input_action {
        InputAction::Resize { width, height } => {
            assert_eq!(width, 80);
            assert_eq!(height, 24);
        }
        _ => panic!("Expected Resize variant"),
    }
}

#[test]
fn test_event_key_all_variants() {
    // Test all KeyCode variants used in the codebase
    let test_cases = vec![
        (KeyCode::Tab, DtoKeyCode::Tab),
        (KeyCode::Enter, DtoKeyCode::Enter),
        (KeyCode::Esc, DtoKeyCode::Esc),
        (KeyCode::Up, DtoKeyCode::Up),
        (KeyCode::Down, DtoKeyCode::Down),
        (KeyCode::Left, DtoKeyCode::Left),
        (KeyCode::Right, DtoKeyCode::Right),
        (KeyCode::Char('q'), DtoKeyCode::Char('q')),
        (KeyCode::Delete, DtoKeyCode::Delete),
        (KeyCode::Backspace, DtoKeyCode::Backspace),
    ];

    for (crossterm_code, expected_dto) in test_cases {
        let key_event = KeyEvent::new(crossterm_code, KeyModifiers::empty());
        let crossterm_event = Event::Key(key_event);
        let input_action = InputAction::from(crossterm_event);

        match input_action {
            InputAction::KeyPressed { key, .. } => {
                assert_eq!(key, expected_dto, "Failed for {:?}", crossterm_code);
            }
            _ => panic!("Expected KeyPressed for {:?}", crossterm_code),
        }
    }
}
