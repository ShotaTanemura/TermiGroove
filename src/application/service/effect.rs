//! Effect system for representing side effects.
//!
//! Effects represent side effects that need to be executed as a result of
//! application service operations. Effects are pure data structures that
//! can be executed by infrastructure layer components.

use crate::audio::AudioCommand;

/// Represents a side effect that needs to be executed.
///
/// Effects are produced by application services and consumed by infrastructure
/// layer components (e.g., audio thread, UI updates). This decouples the
/// application layer from direct infrastructure dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Send an audio command to the audio thread
    AudioCommand(AudioCommand),
    /// Update the status message displayed in the UI footer
    StatusMessage(String),
    // Future effects can be added here:
    // - UiUpdate(UiUpdateCommand)
    // - Log(LogMessage)
    // - etc.
}

impl From<AudioCommand> for Effect {
    fn from(cmd: AudioCommand) -> Self {
        Effect::AudioCommand(cmd)
    }
}

