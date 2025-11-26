//! Application state for use case progress and domain entities.
//!
//! This module contains `ApplicationState`, which holds:
//! - Use case progress state (selection, pads)
//! - Application configuration (bpm, bars)
//! - Domain entities (LoopEngine)
//!
//! This state is managed by the application layer and can be mutated by
//! application services. It does not contain presentation concerns.

use crate::audio::{AudioCommand, SenderAudioBus, SystemClock};
use crate::domain::r#loop::{LoopEngine, LoopState};
use crate::domain::tempo::{clamp_bars, clamp_bpm};
use crate::selection::SelectionModel;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Application state for use case progress and domain entities.
#[derive(Debug)]
pub struct ApplicationState {
    /// Selection model for file selection use case
    pub selection: SelectionModel,
    /// Pads state for pad mapping and active keys
    pub pads: PadsState,
    /// Current BPM (beats per minute)
    bpm: u16,
    /// Current bars (number of bars in loop)
    bars: u16,
    /// Domain entity: loop engine
    loop_engine: LoopEngine<SenderAudioBus, SystemClock>,
}

/// Pads state containing key mappings and active keys.
#[derive(Debug, Default, Clone)]
pub struct PadsState {
    /// Mapping from key character to sample slot
    pub key_to_slot: BTreeMap<char, SampleSlot>,
    /// Set of currently active (pressed) keys
    pub active_keys: HashSet<char>,
    /// Timestamp of last press for each key (milliseconds)
    pub last_press_ms: BTreeMap<char, u128>,
}

/// Sample slot information.
#[derive(Debug, Default, Clone)]
pub struct SampleSlot {
    /// File name of the sample
    pub file_name: String,
}

impl ApplicationState {
    /// Create a new ApplicationState with the given loop engine.
    pub fn new(loop_engine: LoopEngine<SenderAudioBus, SystemClock>) -> Self {
        Self {
            selection: SelectionModel::default(),
            pads: PadsState::default(),
            bpm: 120,
            bars: 16,
            loop_engine,
        }
    }

    /// Get current loop state.
    pub fn loop_state(&self) -> LoopState {
        self.loop_engine.state()
    }

    /// Update loop engine (call on each frame).
    pub fn update_loop(&mut self) {
        self.loop_engine.update();
    }

    /// Get current BPM.
    pub fn get_bpm(&self) -> u16 {
        self.bpm
    }

    /// Get current bars.
    pub fn get_bars(&self) -> u16 {
        self.bars
    }

    /// Set BPM (clamped to valid range).
    pub fn set_bpm(&mut self, bpm: u16) {
        self.bpm = clamp_bpm(bpm);
    }

    /// Set bars (clamped to valid range).
    pub fn set_bars(&mut self, bars: u16) {
        self.bars = clamp_bars(bars);
    }

    /// Reset loop engine for new tempo (when BPM or bars change).
    pub fn reset_loop_for_tempo(&mut self) {
        self.loop_engine.reset_for_new_tempo(self.bpm, self.bars);
    }

    /// Handle space key press for loop control.
    pub fn handle_loop_space(&mut self) {
        self.loop_engine.handle_space(self.bpm, self.bars);
    }

    /// Record a loop event (pad press during recording).
    pub fn record_loop_event(&mut self, key: char) {
        self.loop_engine.record_event(key);
    }

    /// Cancel the current loop operation.
    pub fn cancel_loop(&mut self) {
        self.loop_engine.handle_cancel();
    }

    /// Clear the loop (remove all tracks).
    pub fn clear_loop(&mut self) {
        self.loop_engine.handle_control_space();
    }

    /// Attempt to enter Pads mode. Validates selection and builds pad mapping.
    /// Returns effects (Preload commands) and error message if validation fails.
    pub fn enter_pads(&mut self) -> anyhow::Result<Vec<AudioCommand>> {
        if self.selection.items.is_empty() {
            anyhow::bail!("Select at least one file first")
        }

        // Validate all selected files are .wav (case-insensitive)
        if let Some(invalid) = self.selection.items.iter().find(|p| !is_wav(p)).cloned() {
            let name = file_name_str(&invalid);
            anyhow::bail!("Unsupported file (only .wav): {}", name)
        }

        // Build mapping from selection order to default pad keys
        let keys = default_pad_keys();
        let mut key_to_slot: BTreeMap<char, SampleSlot> = BTreeMap::new();
        let mut preload_effects = Vec::new();

        for (idx, path) in self.selection.items.iter().enumerate() {
            if idx >= keys.len() {
                break; // ignore overflow for now
            }
            let key = keys[idx];
            let slot = SampleSlot {
                file_name: file_name_str(path),
            };
            key_to_slot.insert(key, slot);

            // Create Preload command for this key
            preload_effects.push(AudioCommand::Preload {
                key,
                path: path.clone(),
            });
        }

        self.pads = PadsState {
            key_to_slot,
            active_keys: HashSet::new(),
            last_press_ms: BTreeMap::new(),
        };

        Ok(preload_effects)
    }
}

/// Check if path has .wav extension (case-insensitive).
fn is_wav(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("wav"))
        .unwrap_or(false)
}

/// Get file name from path as string.
fn file_name_str(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string()
}

/// Default pad keys for mapping samples (QWERTY row-first mapping).
fn default_pad_keys() -> &'static [char] {
    const KEYS: &[char] = &[
        'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k',
        'l', ';', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    ];
    KEYS
}
