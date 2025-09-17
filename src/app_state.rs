use crate::selection::SelectionModel;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Theme as ExplorerTheme;
use std::path::PathBuf;

const HELP_LINE: &str =
    "  Enter: to pads / Space: select / Tab: switch pane / d/Delete: remove / q: quit  ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Browse,
    Pads,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPane {
    #[default]
    LeftExplorer,
    RightSelected,
}

// Default now derived on enums above

#[derive(Debug)]
pub struct AppState {
    pub mode: Mode,
    pub focus: FocusPane,
    pub status_message: String,
    pub selection: SelectionModel,
    // Placeholder for explorer integration: the current item in the left pane
    pub current_left_item: Option<PathBuf>,
    pub current_left_is_dir: bool,
    pub file_explorer: FileExplorer,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let theme = ExplorerTheme::default()
            .add_default_title()
            .with_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .with_title_bottom(|_| HELP_LINE.into());
        let file_explorer = FileExplorer::with_theme(theme)?;
        Ok(Self {
            mode: Mode::Browse,
            focus: FocusPane::LeftExplorer,
            status_message: "Ready".to_string(),
            selection: Default::default(),
            current_left_item: None,
            current_left_is_dir: false,
            file_explorer,
        })
    }
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::LeftExplorer => FocusPane::RightSelected,
            FocusPane::RightSelected => FocusPane::LeftExplorer,
        };
    }

    pub fn focus_status_message(&self) -> String {
        match self.focus {
            FocusPane::LeftExplorer => "Left focus".to_string(),
            FocusPane::RightSelected => "Right focus".to_string(),
        }
    }

    /// Append a file to selection or toggle remove if it already exists.
    pub fn toggle_select_file(&mut self, path: PathBuf) {
        self.selection.add_file(path);
        self.status_message = self.selection.status.clone();
    }

    // Remove item at the current right selection cursor.
    // Keep for future integration when right-pane removal via Enter is needed
    // pub fn remove_selected_at_cursor(&mut self) {
    //     self.selection.remove_at_cursor();
    //     self.status_message = self.selection.status.clone();
    // }
}
