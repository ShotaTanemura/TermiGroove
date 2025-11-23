use std::time::Duration;

use termigroove::domain::timing::{beat_interval_ms, loop_length_from, normalize_offset};

#[test]
fn test_loop_length_from() {
    // 120 BPM, 4 bars = 8 seconds (120 beats/min = 2 beats/sec, 4 bars = 16 beats, 16/2 = 8 sec)
    let length = loop_length_from(120, 4);
    assert_eq!(length, Duration::from_secs(8));

    // 60 BPM, 1 bar = 4 seconds (60 beats/min = 1 beat/sec, 1 bar = 4 beats, 4/1 = 4 sec)
    let length = loop_length_from(60, 1);
    assert_eq!(length, Duration::from_secs(4));

    // 180 BPM, 2 bars = 2.666... seconds (180 beats/min = 3 beats/sec, 2 bars = 8 beats, 8/3 ≈ 2.667 sec)
    let length = loop_length_from(180, 2);
    let expected = Duration::from_secs_f64(8.0 / 3.0);
    assert!((length.as_secs_f64() - expected.as_secs_f64()).abs() < 0.001);
}

#[test]
fn test_beat_interval_ms() {
    // 120 BPM = 0.5 seconds per beat
    let interval = beat_interval_ms(120);
    assert_eq!(interval, Duration::from_millis(500));

    // 60 BPM = 1 second per beat
    let interval = beat_interval_ms(60);
    assert_eq!(interval, Duration::from_secs(1));

    // 180 BPM = 0.333... seconds per beat
    let interval = beat_interval_ms(180);
    let expected = Duration::from_secs_f64(1.0 / 3.0);
    assert!((interval.as_secs_f64() - expected.as_secs_f64()).abs() < 0.001);
}

#[test]
fn test_normalize_offset_zero_length() {
    // Zero loop length should return zero
    let result = normalize_offset(Duration::from_secs(5), Duration::ZERO);
    assert_eq!(result, Duration::ZERO);
}

#[test]
fn test_normalize_offset_exact_multiple() {
    // Elapsed time is exact multiple of loop length
    let loop_length = Duration::from_secs(4);
    let result = normalize_offset(Duration::from_secs(8), loop_length);
    assert_eq!(result, Duration::ZERO);

    let result = normalize_offset(Duration::from_secs(12), loop_length);
    assert_eq!(result, Duration::ZERO);
}

#[test]
fn test_normalize_offset_with_remainder() {
    // Elapsed time has remainder
    let loop_length = Duration::from_secs(4);
    let result = normalize_offset(Duration::from_secs(5), loop_length);
    assert_eq!(result, Duration::from_secs(1));

    let result = normalize_offset(Duration::from_secs(7), loop_length);
    assert_eq!(result, Duration::from_secs(3));

    let result = normalize_offset(Duration::from_secs(9), loop_length);
    assert_eq!(result, Duration::from_secs(1));
}

#[test]
fn test_normalize_offset_within_loop() {
    // Elapsed time is within loop length (no wrapping needed)
    let loop_length = Duration::from_secs(4);
    let result = normalize_offset(Duration::from_secs(2), loop_length);
    assert_eq!(result, Duration::from_secs(2));

    let result = normalize_offset(Duration::from_secs(3), loop_length);
    assert_eq!(result, Duration::from_secs(3));
}

#[test]
fn test_normalize_offset_millisecond_precision() {
    // Test with millisecond precision
    let loop_length = Duration::from_millis(1000);
    let result = normalize_offset(Duration::from_millis(1500), loop_length);
    assert_eq!(result, Duration::from_millis(500));

    let result = normalize_offset(Duration::from_millis(2500), loop_length);
    assert_eq!(result, Duration::from_millis(500));
}

#[test]
fn test_normalize_offset_deterministic() {
    // Verify function is deterministic (same inputs produce same outputs)
    let loop_length = Duration::from_secs(4);
    let elapsed = Duration::from_secs(5);

    let result1 = normalize_offset(elapsed, loop_length);
    let result2 = normalize_offset(elapsed, loop_length);
    assert_eq!(result1, result2);
}
