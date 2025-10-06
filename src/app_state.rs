use crate::audio::{AudioCommand, spawn_audio_thread};
use crate::selection::SelectionModel;
use crate::state::loop_engine::{LoopEngine, LoopState, SenderAudioBus, SystemClock};
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Theme as ExplorerTheme;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use tui_input::Input as TextInput;

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
    pub pads: PadsState,
    pub audio_tx: Sender<AudioCommand>,
    loop_engine: LoopEngine<SenderAudioBus, SystemClock>,
    // --- Global tempo & loop state ---
    bpm: u16,
    bars: u16,
    // Popup / editing state for BPM & Bars configuration
    is_popup_open: bool,
    popup_focus: PopupFocus,
    draft_bpm: TextInput,
    draft_bars: TextInput,
}

#[derive(Debug, Default, Clone)]
pub struct PadsState {
    pub key_to_slot: BTreeMap<char, SampleSlot>,
    pub active_keys: HashSet<char>,
    pub last_press_ms: BTreeMap<char, u128>,
}

#[derive(Debug, Default, Clone)]
pub struct SampleSlot {
    pub file_name: String,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let audio_tx = spawn_audio_thread();
        let bus = SenderAudioBus::new(audio_tx.clone());
        let loop_engine = LoopEngine::new(SystemClock::new(), bus);
        Self::from_components(audio_tx, loop_engine)
    }

    pub fn from_components(
        audio_tx: Sender<AudioCommand>,
        loop_engine: LoopEngine<SenderAudioBus, SystemClock>,
    ) -> anyhow::Result<Self> {
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
            selection: SelectionModel::default(),
            current_left_item: None,
            current_left_is_dir: false,
            file_explorer,
            pads: PadsState::default(),
            audio_tx,
            loop_engine,
            bpm: 120,
            bars: 16,
            is_popup_open: false,
            popup_focus: PopupFocus::None,
            draft_bpm: TextInput::new(120.to_string()),
            draft_bars: TextInput::new(16.to_string()),
        })
    }

    pub fn update_loop(&mut self) {
        self.loop_engine.update();
    }

    #[allow(dead_code)]
    pub fn loop_state(&self) -> LoopState {
        self.loop_engine.state()
    }

    pub fn handle_loop_space(&mut self) {
        self.loop_engine.handle_space(self.bpm, self.bars);
        self.update_status_for_loop_state();
    }

    fn update_status_for_loop_state(&mut self) {
        let loop_state = self.loop_engine.state();
        let track_count = self.loop_engine.tracks_count();
        self.status_message = match loop_state {
            LoopState::Idle => "Loop idle".to_string(),
            LoopState::Ready { .. } => "Loop ready".to_string(),
            LoopState::Recording { .. } => "Loop recording".to_string(),
            LoopState::Playing { .. } => format!(
                "Loop playing ({} track{})",
                track_count,
                if track_count == 1 { "" } else { "s" }
            ),
            LoopState::Paused { .. } => format!(
                "Loop paused ({} track{})",
                track_count,
                if track_count == 1 { "" } else { "s" }
            ),
        };
    }

    pub fn record_loop_event(&mut self, key: char) {
        if !matches!(self.loop_engine.state(), LoopState::Recording { .. }) {
            let _ = self.audio_tx.send(AudioCommand::Play { key });
        }
        self.loop_engine.record_event(key);
        self.update_status_for_loop_state();
    }

    pub fn reset_loop_for_tempo(&mut self) {
        self.loop_engine.reset_for_new_tempo(self.bpm, self.bars);
    }

    pub fn cancel_loop(&mut self) {
        self.loop_engine.handle_cancel();
        self.status_message = "Loop cancelled".to_string();
    }

    pub fn clear_loop(&mut self) {
        self.loop_engine.handle_control_space();
        self.status_message = "Loop cleared".to_string();
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

    /// Attempt to enter Pads mode. Validates selection and builds pad mapping.
    pub fn enter_pads(&mut self) -> anyhow::Result<()> {
        if self.selection.items.is_empty() {
            self.status_message = "Select at least one file first".to_string();
            anyhow::bail!("no selection")
        }

        // Validate all selected files are .wav (case-insensitive)
        if let Some(invalid) = self.selection.items.iter().find(|p| !is_wav(p)).cloned() {
            let name = file_name_str(&invalid);
            self.status_message = format!("Unsupported file (only .wav): {}", name);
            anyhow::bail!("non-wav selected")
        }

        // Build mapping from selection order to default pad keys
        let keys = default_pad_keys();
        let mut key_to_slot: BTreeMap<char, SampleSlot> = BTreeMap::new();
        for (idx, path) in self.selection.items.iter().enumerate() {
            if idx >= keys.len() {
                break; // ignore overflow for now
            }
            let key = keys[idx];
            let slot = SampleSlot {
                file_name: file_name_str(path),
            };
            key_to_slot.insert(key, slot);
        }

        self.pads = PadsState {
            key_to_slot,
            active_keys: HashSet::new(),
            last_press_ms: BTreeMap::new(),
        };

        // Send Preload commands to audio thread for each mapped key
        for (key, slot) in &self.pads.key_to_slot {
            // Find the original path by matching file name; if duplicates exist, first match wins.
            if let Some(path) = self
                .selection
                .items
                .iter()
                .find(|p| file_name_str(p) == slot.file_name)
            {
                let _ = self.audio_tx.send(AudioCommand::Preload {
                    key: *key,
                    path: path.clone(),
                });
            }
        }

        self.mode = Mode::Pads;
        self.status_message = "[Pads] Press Esc to go back. Press Q/W/…/< to trigger.".to_string();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupFocus {
    #[default]
    None,
    SummaryBox,
    PopupFieldBpm,
    PopupFieldBars,
    PopupOk,
    PopupCancel,
}

impl AppState {
    // ----- Getters -----
    pub fn get_bpm(&self) -> u16 {
        self.bpm
    }
    pub fn get_bars(&self) -> u16 {
        self.bars
    }
    pub fn is_bpm_popup_open(&self) -> bool {
        self.is_popup_open
    }
    pub fn popup_focus(&self) -> PopupFocus {
        self.popup_focus
    }

    // Access draft text values
    pub fn draft_bpm_mut(&mut self) -> &mut TextInput {
        &mut self.draft_bpm
    }
    pub fn draft_bars_mut(&mut self) -> &mut TextInput {
        &mut self.draft_bars
    }
    pub fn draft_bpm(&self) -> &TextInput {
        &self.draft_bpm
    }
    pub fn draft_bars(&self) -> &TextInput {
        &self.draft_bars
    }

    // ----- Setters (clamped) -----
    pub fn set_bpm(&mut self, bpm: u16) {
        self.bpm = clamp_bpm(bpm);
    }
    pub fn set_bars(&mut self, bars: u16) {
        self.bars = clamp_bars(bars);
    }

    // ----- Popup lifecycle -----
    pub fn open_bpm_bars_popup(&mut self) {
        self.is_popup_open = true;
        self.popup_focus = PopupFocus::PopupFieldBpm;
        self.draft_bpm = TextInput::new(self.bpm.to_string());
        self.draft_bars = TextInput::new(self.bars.to_string());
    }

    pub fn close_bpm_bars_popup(&mut self, apply: bool) {
        if apply {
            let mut changed = false;
            if let Ok(bpm) = self.draft_bpm.value().parse::<u16>() {
                let before = self.bpm;
                self.set_bpm(bpm);
                if self.bpm != before {
                    changed = true;
                }
            }
            if let Ok(bars) = self.draft_bars.value().parse::<u16>() {
                let before = self.bars;
                self.set_bars(bars);
                if self.bars != before {
                    changed = true;
                }
            }
            if changed {
                self.reset_loop_for_tempo();
            }
        }
        self.is_popup_open = false;
        self.popup_focus = PopupFocus::None;
        self.draft_bpm.reset();
        self.draft_bars.reset();
    }

    // ----- Helpers for clamping ranges -----
    // ----- Popup focus navigation -----
    pub fn focus_summary_box(&mut self) {
        self.popup_focus = PopupFocus::SummaryBox;
    }
    pub fn popup_focus_up(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupFieldBpm => PopupFocus::PopupOk,
            PopupFocus::PopupFieldBars => PopupFocus::PopupFieldBpm,
            PopupFocus::PopupOk | PopupFocus::PopupCancel => PopupFocus::PopupFieldBars,
            _ => PopupFocus::SummaryBox,
        };
    }
    pub fn popup_focus_down(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupFieldBpm => PopupFocus::PopupFieldBars,
            PopupFocus::PopupFieldBars => PopupFocus::PopupOk,
            PopupFocus::PopupOk | PopupFocus::PopupCancel => PopupFocus::PopupFieldBpm,
            _ => PopupFocus::SummaryBox,
        };
    }
    pub fn popup_toggle_ok_cancel(&mut self) {
        self.popup_focus = match self.popup_focus {
            PopupFocus::PopupOk => PopupFocus::PopupCancel,
            PopupFocus::PopupCancel => PopupFocus::PopupOk,
            _ => PopupFocus::PopupOk,
        };
    }

    // ----- Draft editing helpers (digits only; backspace) -----
}

const BPM_MIN: u16 = 20;
const BPM_MAX: u16 = 300;
const BARS_MIN: u16 = 1;
const BARS_MAX: u16 = 256;

fn clamp_bpm(v: u16) -> u16 {
    v.clamp(BPM_MIN, BPM_MAX)
}
fn clamp_bars(v: u16) -> u16 {
    v.clamp(BARS_MIN, BARS_MAX)
}

fn is_wav(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("wav"))
        .unwrap_or(false)
}

fn file_name_str(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string()
}

fn default_pad_keys() -> &'static [char] {
    // Typical QWERTY row-first mapping
    const KEYS: &[char] = &[
        'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k',
        'l', ';', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    ];
    KEYS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_pads_requires_selection() {
        let mut state = AppState::new().expect("init AppState");
        let res = state.enter_pads();
        assert!(res.is_err());
        assert!(
            state
                .status_message
                .contains("Select at least one file first")
        );
    }

    #[test]
    fn enter_pads_rejects_non_wav() {
        let mut state = AppState::new().expect("init AppState");
        state.toggle_select_file(PathBuf::from("track.mp3"));
        let res = state.enter_pads();
        assert!(res.is_err());
        assert!(state.status_message.contains("Unsupported file"));
    }

    #[test]
    fn enter_pads_builds_mapping_for_wavs() {
        let mut state = AppState::new().expect("init AppState");
        state.toggle_select_file(PathBuf::from("kick.wav"));
        state.toggle_select_file(PathBuf::from("snare.wav"));
        let res = state.enter_pads();
        assert!(res.is_ok());
        assert!(matches!(state.mode, Mode::Pads));

        let keys = default_pad_keys();
        let q = keys[0];
        let w = keys[1];
        let slot_q = state.pads.key_to_slot.get(&q).expect("q mapped");
        let slot_w = state.pads.key_to_slot.get(&w).expect("w mapped");
        assert_eq!(slot_q.file_name, "kick.wav");
        assert_eq!(slot_w.file_name, "snare.wav");
    }

    #[test]
    fn is_wav_case_insensitive() {
        assert!(is_wav(Path::new("KICK.WAV")));
        assert!(!is_wav(Path::new("notwav.txt")));
    }
}
