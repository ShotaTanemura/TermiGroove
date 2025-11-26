//! Presentation layer view model.
//!
//! This module contains `ViewModel`, which holds all presentation-specific state
//! that is used for rendering the UI. This includes UI mode, focus, status messages,
//! file explorer, popup state, and draft input fields.
//!
//! This state is managed by the presentation layer and can be mutated by
//! presentation components (e.g., effect handlers).

use ratatui_explorer::FileExplorer;
use std::path::PathBuf;
use tui_input::Input as TextInput;

/// Application mode - controls which screen is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// File browser mode for selecting WAV files
    #[default]
    Browse,
    /// Pads mode for triggering samples
    Pads,
}

/// Which pane has keyboard focus in Browse mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPane {
    /// Left pane: file explorer
    #[default]
    LeftExplorer,
    /// Right pane: selected files list
    RightSelected,
}

/// Popup focus states for BPM/Bars configuration dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupFocus {
    /// No popup open
    #[default]
    None,
    /// Summary box focus
    SummaryBox,
    /// BPM input field
    PopupFieldBpm,
    /// Bars input field
    PopupFieldBars,
    /// OK button
    PopupOk,
    /// Cancel button
    PopupCancel,
}

/// Presentation view model containing UI-specific state.
#[derive(Debug)]
pub struct ViewModel {
    /// Current application mode (Browse or Pads)
    pub mode: Mode,
    /// Current focus pane (LeftExplorer or RightSelected)
    pub focus: FocusPane,
    /// Status message displayed in footer
    pub status_message: String,
    /// File explorer widget for directory navigation
    pub file_explorer: FileExplorer,
    /// Current item in left pane
    pub current_left_item: Option<PathBuf>,
    /// Whether current left item is a directory
    pub current_left_is_dir: bool,
    /// Whether BPM/Bars popup is open
    pub is_popup_open: bool,
    /// Current focus within popup
    pub popup_focus: PopupFocus,
    /// Draft BPM input field
    pub draft_bpm: TextInput,
    /// Draft bars input field
    pub draft_bars: TextInput,
}

impl ViewModel {
    /// Create a new ViewModel with default values.
    pub fn new(file_explorer: FileExplorer) -> Self {
        Self {
            mode: Mode::Browse,
            focus: FocusPane::LeftExplorer,
            status_message: "Ready".to_string(),
            file_explorer,
            current_left_item: None,
            current_left_is_dir: false,
            is_popup_open: false,
            popup_focus: PopupFocus::None,
            draft_bpm: TextInput::new(120.to_string()),
            draft_bars: TextInput::new(16.to_string()),
        }
    }

    /// Toggle focus between LeftExplorer and RightSelected.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::LeftExplorer => FocusPane::RightSelected,
            FocusPane::RightSelected => FocusPane::LeftExplorer,
        };
    }

    /// Get status message based on current focus.
    pub fn focus_status_message(&self) -> String {
        match self.focus {
            FocusPane::LeftExplorer => "Focus: Left Explorer".to_string(),
            FocusPane::RightSelected => "Focus: Right Selected".to_string(),
        }
    }

    /// Check if BPM popup is open.
    pub fn is_bpm_popup_open(&self) -> bool {
        self.is_popup_open
    }

    /// Get current popup focus.
    pub fn popup_focus(&self) -> PopupFocus {
        self.popup_focus
    }

    /// Get mutable reference to draft BPM input.
    pub fn draft_bpm_mut(&mut self) -> &mut TextInput {
        &mut self.draft_bpm
    }

    /// Get mutable reference to draft bars input.
    pub fn draft_bars_mut(&mut self) -> &mut TextInput {
        &mut self.draft_bars
    }

    /// Get immutable reference to draft BPM input.
    pub fn draft_bpm(&self) -> &TextInput {
        &self.draft_bpm
    }

    /// Get immutable reference to draft bars input.
    pub fn draft_bars(&self) -> &TextInput {
        &self.draft_bars
    }

    /// Open BPM/Bars popup.
    pub fn open_bpm_bars_popup(&mut self, bpm: u16, bars: u16) {
        self.is_popup_open = true;
        self.popup_focus = PopupFocus::PopupFieldBpm;
        self.draft_bpm = TextInput::new(bpm.to_string());
        self.draft_bars = TextInput::new(bars.to_string());
    }

    /// Close BPM/Bars popup.
    pub fn close_bpm_bars_popup(&mut self) {
        self.is_popup_open = false;
        self.popup_focus = PopupFocus::None;
        self.draft_bpm.reset();
        self.draft_bars.reset();
    }

    /// Focus summary box.
    pub fn focus_summary_box(&mut self) {
        self.popup_focus = PopupFocus::SummaryBox;
    }

    /// Move popup focus up.
    pub fn popup_focus_up(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupFieldBpm => PopupFocus::PopupOk,
            PopupFocus::PopupFieldBars => PopupFocus::PopupFieldBpm,
            PopupFocus::PopupOk | PopupFocus::PopupCancel => PopupFocus::PopupFieldBars,
            _ => PopupFocus::SummaryBox,
        };
    }

    /// Move popup focus down.
    pub fn popup_focus_down(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupFieldBpm => PopupFocus::PopupFieldBars,
            PopupFocus::PopupFieldBars => PopupFocus::PopupOk,
            PopupFocus::PopupOk | PopupFocus::PopupCancel => PopupFocus::PopupFieldBpm,
            _ => PopupFocus::SummaryBox,
        };
    }

    /// Toggle between OK and Cancel buttons in popup.
    pub fn popup_toggle_ok_cancel(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupOk => PopupFocus::PopupCancel,
            PopupFocus::PopupCancel => PopupFocus::PopupOk,
            _ => PopupFocus::PopupOk,
        };
    }
}

