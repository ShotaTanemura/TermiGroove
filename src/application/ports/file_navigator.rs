//! File navigator port definition.
//!
//! This module defines the `FileNavigator` trait that abstracts file system
//! navigation operations. The application layer depends on this trait,
//! while the presentation layer provides the concrete implementation.

use crate::application::dto::input_action::KeyCode;
use std::path::PathBuf;

/// Represents a file entry in the navigator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// The path to the file or directory
    pub path: PathBuf,
    /// Whether this entry is a directory
    pub is_dir: bool,
}

/// Port for file system navigation operations.
///
/// This trait abstracts the file explorer functionality, allowing the
/// application layer to work with file navigation without depending on
/// specific UI libraries like ratatui_explorer.
pub trait FileNavigator {
    /// Handle a navigation key press (Up, Down, Left, Right, Enter).
    ///
    /// # Arguments
    /// * `key` - The key code representing the navigation action
    ///
    /// # Returns
    /// * `Ok(())` if the key was handled successfully
    /// * `Err(...)` if an error occurred during navigation
    fn handle_navigation_key(&mut self, key: KeyCode) -> anyhow::Result<()>;

    /// Get the currently selected entry, if any.
    ///
    /// # Returns
    /// * `Some(FileEntry)` if there is a selected item
    /// * `None` if no item is selected or the navigator is empty
    fn selected_entry(&self) -> Option<FileEntry>;
}
