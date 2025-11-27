//! Application layer port definitions.
//!
//! Ports define the interfaces (traits) that the application layer needs from
//! external systems. These are implemented by adapters in outer layers.

mod file_navigator;

pub use file_navigator::{FileEntry, FileNavigator};
