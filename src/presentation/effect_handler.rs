//! Effect handler for applying side effects to presentation layer.
//!
//! This module provides functions to apply effects produced by the application
//! layer to the presentation layer (e.g., updating status messages) and
//! infrastructure layer (e.g., sending audio commands).

use crate::application::service::Effect;
use crate::audio::AudioCommand;
use crate::presentation::ViewModel;
use std::sync::mpsc::Sender;

/// Apply effects to the presentation layer and infrastructure.
///
/// This function processes effects produced by application services and
/// applies them to the appropriate layers:
/// - `StatusMessage` effects update the view model
/// - `AudioCommand` effects are sent to the audio thread
///
/// # Arguments
///
/// * `view_model` - Mutable reference to the view model (for status messages)
/// * `audio_tx` - Sender for audio commands
/// * `effects` - Vector of effects to apply
pub fn apply_effects(
    view_model: &mut ViewModel,
    audio_tx: &Sender<AudioCommand>,
    effects: Vec<Effect>,
) {
    for effect in effects {
        match effect {
            Effect::StatusMessage(message) => {
                view_model.status_message = message;
            }
            Effect::AudioCommand(cmd) => {
                let _ = audio_tx.send(cmd);
            }
        }
    }
}
