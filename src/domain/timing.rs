//! Timing utilities module.
//!
//! This module contains pure functions for timing and clock-related calculations.
//! All functions are stateless and deterministic, making them highly testable
//! and reusable across the domain layer.
//!
//! Domain concepts:
//! - Loop length calculations from BPM and bars
//! - Beat interval calculations
//! - Time offset normalization within loop cycles

use std::time::Duration;

/// Calculate loop length from BPM and number of bars.
///
/// Assumes 4 beats per bar (common time signature).
///
/// # Arguments
/// * `bpm` - Beats per minute (must be > 0)
/// * `bars` - Number of bars in the loop
///
/// # Returns
/// The total duration of the loop
///
/// # Example
/// ```
/// use std::time::Duration;
/// use termigroove::domain::timing::loop_length_from;
///
/// // 120 BPM, 4 bars = 8 seconds
/// let length = loop_length_from(120, 4);
/// assert_eq!(length, Duration::from_secs(8));
/// ```
pub fn loop_length_from(bpm: u16, bars: u16) -> Duration {
    let beats_per_bar = 4.0;
    let beat_seconds = 60.0 / bpm as f64;
    Duration::from_secs_f64(beat_seconds * beats_per_bar * bars as f64)
}

/// Calculate the duration of a single beat from BPM.
///
/// # Arguments
/// * `bpm` - Beats per minute (must be > 0)
///
/// # Returns
/// The duration of one beat
///
/// # Example
/// ```
/// use std::time::Duration;
/// use termigroove::domain::timing::beat_interval_ms;
///
/// // 120 BPM = 0.5 seconds per beat
/// let interval = beat_interval_ms(120);
/// assert_eq!(interval, Duration::from_millis(500));
/// ```
pub fn beat_interval_ms(bpm: u16) -> Duration {
    Duration::from_secs_f64(60.0 / bpm as f64)
}

/// Normalize an elapsed time offset to be within a loop cycle.
///
/// This function wraps elapsed time around the loop length, ensuring
/// the result is always between 0 and loop_length (exclusive).
///
/// # Arguments
/// * `elapsed` - The elapsed time to normalize
/// * `loop_length` - The length of one loop cycle
///
/// # Returns
/// The normalized offset within the loop cycle, or `Duration::ZERO` if
/// loop_length is zero
///
/// # Example
/// ```
/// use std::time::Duration;
/// use termigroove::domain::timing::normalize_offset;
///
/// let loop_length = Duration::from_secs(4);
/// let elapsed = Duration::from_secs(5);
/// let normalized = normalize_offset(elapsed, loop_length);
/// assert_eq!(normalized, Duration::from_secs(1)); // 5 % 4 = 1
/// ```
pub fn normalize_offset(elapsed: Duration, loop_length: Duration) -> Duration {
    if loop_length.is_zero() {
        return Duration::ZERO;
    }
    let loop_nanos = loop_length.as_nanos();
    if loop_nanos == 0 {
        return Duration::ZERO;
    }
    let remainder = elapsed.as_nanos() % loop_nanos;
    Duration::from_nanos(remainder as u64)
}

