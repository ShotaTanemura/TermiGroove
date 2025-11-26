//! Input action DTOs.
//!
//! This module defines DTOs for representing user input events in a
//! technology-agnostic way, decoupling the presentation layer from
//! specific input frameworks (e.g., crossterm).

/// Framework-agnostic representation of keyboard key codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    /// Tab key
    Tab,
    /// Enter/Return key
    Enter,
    /// Escape key
    Esc,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Character key (ASCII character)
    Char(char),
    /// Delete key
    Delete,
    /// Backspace key
    Backspace,
    /// Other unmapped key (for future-proofing)
    Other(String),
}

/// Framework-agnostic representation of keyboard modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    /// Control modifier (Ctrl)
    pub control: bool,
    /// Shift modifier
    pub shift: bool,
    /// Alt modifier
    pub alt: bool,
}

/// Input action DTO representing user input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    /// A key was pressed
    KeyPressed {
        /// The key that was pressed
        key: KeyCode,
        /// Modifier keys that were held
        modifiers: KeyModifiers,
    },
    /// A key was released (for future use)
    KeyReleased {
        /// The key that was released
        key: KeyCode,
    },
    /// Terminal window was resized
    Resize {
        /// New width in columns
        width: u16,
        /// New height in rows
        height: u16,
    },
}

// Conversion from crossterm types

impl From<ratatui::crossterm::event::KeyCode> for KeyCode {
    fn from(code: ratatui::crossterm::event::KeyCode) -> Self {
        use ratatui::crossterm::event::KeyCode as CrosstermKeyCode;
        match code {
            CrosstermKeyCode::Tab => KeyCode::Tab,
            CrosstermKeyCode::Enter => KeyCode::Enter,
            CrosstermKeyCode::Esc => KeyCode::Esc,
            CrosstermKeyCode::Up => KeyCode::Up,
            CrosstermKeyCode::Down => KeyCode::Down,
            CrosstermKeyCode::Left => KeyCode::Left,
            CrosstermKeyCode::Right => KeyCode::Right,
            CrosstermKeyCode::Char(c) => KeyCode::Char(c),
            CrosstermKeyCode::Delete => KeyCode::Delete,
            CrosstermKeyCode::Backspace => KeyCode::Backspace,
            other => KeyCode::Other(format!("{:?}", other)),
        }
    }
}

impl From<ratatui::crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(modifiers: ratatui::crossterm::event::KeyModifiers) -> Self {
        use ratatui::crossterm::event::KeyModifiers as CrosstermModifiers;
        KeyModifiers {
            control: modifiers.contains(CrosstermModifiers::CONTROL),
            shift: modifiers.contains(CrosstermModifiers::SHIFT),
            alt: modifiers.contains(CrosstermModifiers::ALT),
        }
    }
}

impl From<ratatui::crossterm::event::Event> for InputAction {
    fn from(event: ratatui::crossterm::event::Event) -> Self {
        use ratatui::crossterm::event::{Event, KeyEventKind};
        match event {
            Event::Key(key_event) => {
                let key = KeyCode::from(key_event.code);
                let modifiers = KeyModifiers::from(key_event.modifiers);
                match key_event.kind {
                    KeyEventKind::Press => InputAction::KeyPressed { key, modifiers },
                    KeyEventKind::Release => InputAction::KeyReleased { key },
                    _ => InputAction::KeyPressed { key, modifiers }, // Default to Press for other kinds
                }
            }
            Event::Resize(width, height) => InputAction::Resize { width, height },
            _ => {
                // For other event types, we can't convert them meaningfully
                // This shouldn't happen in practice, but we need to handle it
                InputAction::KeyPressed {
                    key: KeyCode::Other("Unknown".to_string()),
                    modifiers: KeyModifiers::default(),
                }
            }
        }
    }
}
