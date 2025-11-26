//! Loop state DTOs.
//!
//! This module defines DTOs for representing loop state information
//! to the presentation layer, decoupling UI from domain LoopState implementation.

use std::time::Duration;

use crate::domain::r#loop::LoopEngine;
use crate::domain::ports::{AudioBus, Clock};

/// Framework-agnostic representation of loop status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopStatusDto {
    /// Loop is idle (no recording or playback)
    Idle,
    /// Loop is ready to record (metronome countdown)
    Ready,
    /// Loop is currently recording
    Recording,
    /// Loop is playing back recorded events
    Playing,
    /// Loop playback/recording is paused
    Paused,
}

/// Data Transfer Object for loop state information.
///
/// This DTO provides a flattened representation of loop state suitable for
/// UI consumption, decoupling the presentation layer from domain implementation details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStateDto {
    /// Current loop status
    pub status: LoopStatusDto,
    /// Number of metronome ticks remaining (for Ready state)
    pub ticks_remaining: Option<u8>,
    /// Total length of the loop cycle
    pub loop_length: Duration,
    /// Current offset within the loop cycle (for Recording/Playing/Paused states)
    pub current_offset: Option<Duration>,
    /// Saved offset when paused (for Paused state)
    pub saved_offset: Option<Duration>,
    /// Whether the loop was recording when paused (for Paused state)
    pub was_recording: Option<bool>,
    /// Number of tracks in the loop
    pub track_count: usize,
}


impl<A: AudioBus, C: Clock> From<&LoopEngine<A, C>> for LoopStateDto {
    fn from(engine: &LoopEngine<A, C>) -> Self {
        let state = engine.state();
        let track_count = engine.tracks_count();
        let now = engine.now();

        match state {
            crate::domain::r#loop::LoopState::Idle => LoopStateDto {
                status: LoopStatusDto::Idle,
                ticks_remaining: None,
                loop_length: Duration::ZERO,
                current_offset: None,
                saved_offset: None,
                was_recording: None,
                track_count,
            },
            crate::domain::r#loop::LoopState::Ready {
                ticks_remaining,
                loop_length,
            } => LoopStateDto {
                status: LoopStatusDto::Ready,
                ticks_remaining: Some(ticks_remaining),
                loop_length,
                current_offset: None,
                saved_offset: None,
                was_recording: None,
                track_count,
            },
            crate::domain::r#loop::LoopState::Recording {
                start_time,
                loop_length,
            } => {
                let elapsed = now.saturating_sub(start_time);
                LoopStateDto {
                    status: LoopStatusDto::Recording,
                    ticks_remaining: None,
                    loop_length,
                    current_offset: Some(elapsed),
                    saved_offset: None,
                    was_recording: None,
                    track_count,
                }
            }
            crate::domain::r#loop::LoopState::Playing {
                cycle_start,
                loop_length,
            } => {
                let elapsed = now.saturating_sub(cycle_start);
                LoopStateDto {
                    status: LoopStatusDto::Playing,
                    ticks_remaining: None,
                    loop_length,
                    current_offset: Some(elapsed),
                    saved_offset: None,
                    was_recording: None,
                    track_count,
                }
            }
            crate::domain::r#loop::LoopState::Paused {
                cycle_start: _,
                loop_length,
                saved_offset,
                was_recording,
            } => LoopStateDto {
                status: LoopStatusDto::Paused,
                ticks_remaining: None,
                loop_length,
                current_offset: Some(saved_offset),
                saved_offset: Some(saved_offset),
                was_recording: Some(was_recording),
                track_count,
            },
        }
    }
}

impl From<LoopStateDto> for crate::domain::r#loop::LoopState {
    fn from(dto: LoopStateDto) -> Self {
        match dto.status {
            LoopStatusDto::Idle => crate::domain::r#loop::LoopState::Idle,
            LoopStatusDto::Ready => crate::domain::r#loop::LoopState::Ready {
                ticks_remaining: dto.ticks_remaining.unwrap_or(0),
                loop_length: dto.loop_length,
            },
            LoopStatusDto::Recording => crate::domain::r#loop::LoopState::Recording {
                start_time: Duration::ZERO, // Cannot reconstruct exact start_time
                loop_length: dto.loop_length,
            },
            LoopStatusDto::Playing => crate::domain::r#loop::LoopState::Playing {
                cycle_start: Duration::ZERO, // Cannot reconstruct exact cycle_start
                loop_length: dto.loop_length,
            },
            LoopStatusDto::Paused => crate::domain::r#loop::LoopState::Paused {
                cycle_start: Duration::ZERO, // Cannot reconstruct exact cycle_start
                loop_length: dto.loop_length,
                saved_offset: dto.saved_offset.unwrap_or(Duration::ZERO),
                was_recording: dto.was_recording.unwrap_or(false),
            },
        }
    }
}
