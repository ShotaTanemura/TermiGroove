//! Tempo/BPM domain logic module.
//!
//! This module will contain domain entities, value objects, and business logic
//! related to tempo, BPM, and timing calculations.
//!
//! Domain concepts:
//! - BPM (beats per minute) values and constraints
//! - Bar/measure calculations
//! - Tempo-related calculations (loop length, beat intervals)

/// Minimum valid BPM value.
pub const BPM_MIN: u16 = 20;

/// Maximum valid BPM value.
pub const BPM_MAX: u16 = 300;

/// Minimum valid bars value.
pub const BARS_MIN: u16 = 1;

/// Maximum valid bars value.
pub const BARS_MAX: u16 = 256;

/// Clamp BPM value to valid range.
pub fn clamp_bpm(v: u16) -> u16 {
    v.clamp(BPM_MIN, BPM_MAX)
}

/// Clamp bars value to valid range.
pub fn clamp_bars(v: u16) -> u16 {
    v.clamp(BARS_MIN, BARS_MAX)
}
