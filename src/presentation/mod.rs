//! Presentation layer.
//!
//! This module contains presentation-specific components including view models,
//! effect handlers, and UI rendering logic.

pub mod effect_handler;
pub(crate) mod file_explorer_adapter;
pub mod view_model;

pub use view_model::{FocusPane, Mode, PopupFocus, ViewModel};
