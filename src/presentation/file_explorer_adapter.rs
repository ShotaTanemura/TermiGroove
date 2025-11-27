//! File explorer adapter.
//!
//! This module provides an adapter that implements the `FileNavigator` port trait
//! for `ratatui_explorer::FileExplorer`, bridging the presentation layer's file
//! explorer implementation with the application layer's abstraction.

use crate::application::dto::input_action::KeyCode;
use crate::application::ports::{FileEntry, FileNavigator};
use ratatui::crossterm::event::{Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyModifiers};
use ratatui_explorer::FileExplorer;

/// Adapter that implements `FileNavigator` for `ratatui_explorer::FileExplorer`.
///
/// This adapter wraps a `FileExplorer` and translates between the application
/// layer's `KeyCode` and the underlying `crossterm` events.
pub struct FileExplorerAdapter<'a> {
    explorer: &'a mut FileExplorer,
}

impl<'a> FileExplorerAdapter<'a> {
    /// Create a new adapter wrapping the given file explorer.
    pub fn new(explorer: &'a mut FileExplorer) -> Self {
        Self { explorer }
    }

    /// Convert our KeyCode to crossterm Event for the FileExplorer.
    fn keycode_to_event(key: KeyCode) -> anyhow::Result<Event> {
        let crossterm_key = match key {
            KeyCode::Up => CrosstermKeyCode::Up,
            KeyCode::Down => CrosstermKeyCode::Down,
            KeyCode::Left => CrosstermKeyCode::Left,
            KeyCode::Right => CrosstermKeyCode::Right,
            KeyCode::Enter => CrosstermKeyCode::Enter,
            KeyCode::Tab => CrosstermKeyCode::Tab,
            KeyCode::Esc => CrosstermKeyCode::Esc,
            KeyCode::Backspace => CrosstermKeyCode::Backspace,
            KeyCode::Delete => CrosstermKeyCode::Delete,
            KeyCode::Char(c) => CrosstermKeyCode::Char(c),
            KeyCode::Other(_) => {
                return Err(anyhow::anyhow!(
                    "Cannot convert Other key to crossterm event"
                ));
            }
        };
        Ok(Event::Key(KeyEvent::new(
            crossterm_key,
            KeyModifiers::empty(),
        )))
    }
}

impl FileNavigator for FileExplorerAdapter<'_> {
    fn handle_navigation_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        let event = Self::keycode_to_event(key)?;
        self.explorer.handle(&event)?;
        Ok(())
    }

    fn selected_entry(&self) -> Option<FileEntry> {
        let idx = self.explorer.selected_idx();
        self.explorer.files().get(idx).map(|entry| FileEntry {
            path: entry.path().to_path_buf(),
            is_dir: entry.is_dir(),
        })
    }
}
