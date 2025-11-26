use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use termigroove::application::dto::loop_state::{LoopStateDto, LoopStatusDto};
use termigroove::domain::r#loop::{LoopEngine, LoopState};
use termigroove::domain::ports::{AudioBus, Clock};

#[derive(Clone)]
struct FakeClock {
    now: Rc<RefCell<Duration>>,
    step: Duration,
}

impl FakeClock {
    fn new(step_ms: u64) -> Self {
        Self {
            now: Rc::new(RefCell::new(Duration::from_millis(0))),
            step: Duration::from_millis(step_ms),
        }
    }

    fn advance(&self) {
        let mut now = self.now.borrow_mut();
        *now += self.step;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Duration {
        *self.now.borrow()
    }
}

#[derive(Clone)]
struct AudioBusMock {
    _sent: Rc<RefCell<Vec<()>>>,
}

impl AudioBusMock {
    fn new() -> Self {
        Self {
            _sent: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl AudioBus for AudioBusMock {
    fn play_metronome_beep(&self) {}
    fn play_pad(&self, _key: char) {}
    fn play_scheduled(&self, _key: char) {}
}

const TEST_BPM: u16 = 120;
const TEST_BARS: u16 = 1;

fn advance_engine(
    clock: &FakeClock,
    engine: &mut LoopEngine<AudioBusMock, FakeClock>,
    steps: usize,
) {
    for _ in 0..steps {
        clock.advance();
        engine.update();
    }
}

#[test]
fn test_dto_conversion_idle() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let engine = LoopEngine::new(clock.clone(), audio_bus);

    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Idle);
    assert_eq!(dto.ticks_remaining, None);
    assert_eq!(dto.loop_length, Duration::ZERO);
    assert_eq!(dto.current_offset, None);
    assert_eq!(dto.saved_offset, None);
    assert_eq!(dto.was_recording, None);
    assert_eq!(dto.track_count, 0);
}

#[test]
fn test_dto_conversion_ready() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Ready);
    assert_eq!(dto.ticks_remaining, Some(4));
    assert!(dto.loop_length > Duration::ZERO);
    assert_eq!(dto.current_offset, None);
    assert_eq!(dto.saved_offset, None);
    assert_eq!(dto.was_recording, None);
    assert_eq!(dto.track_count, 0);
}

#[test]
fn test_dto_conversion_recording() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Recording);
    assert_eq!(dto.ticks_remaining, None);
    assert!(dto.loop_length > Duration::ZERO);
    assert!(dto.current_offset.is_some());
    assert!(dto.current_offset.unwrap() >= Duration::ZERO);
    assert_eq!(dto.saved_offset, None);
    assert_eq!(dto.was_recording, None);
    assert_eq!(dto.track_count, 0);
}

#[test]
fn test_dto_conversion_playing() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4);
    engine.record_event('q');
    advance_engine(&clock, &mut engine, 4);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Playing);
    assert_eq!(dto.ticks_remaining, None);
    assert!(dto.loop_length > Duration::ZERO);
    assert!(dto.current_offset.is_some());
    assert!(dto.current_offset.unwrap() >= Duration::ZERO);
    assert_eq!(dto.saved_offset, None);
    assert_eq!(dto.was_recording, None);
    assert_eq!(dto.track_count, 1);
}

#[test]
fn test_dto_conversion_paused() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4);
    engine.record_event('q');
    advance_engine(&clock, &mut engine, 4);
    engine.handle_space(TEST_BPM, TEST_BARS);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Paused);
    assert_eq!(dto.ticks_remaining, None);
    assert!(dto.loop_length > Duration::ZERO);
    assert!(dto.current_offset.is_some());
    assert!(dto.saved_offset.is_some());
    assert_eq!(dto.current_offset, dto.saved_offset);
    assert_eq!(dto.was_recording, Some(false));
    assert_eq!(dto.track_count, 1);
}

#[test]
fn test_dto_conversion_paused_while_recording() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4);
    engine.handle_space(TEST_BPM, TEST_BARS);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.status, LoopStatusDto::Paused);
    assert_eq!(dto.was_recording, Some(true));
}

#[test]
fn test_dto_conversion_track_count_zero() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let engine = LoopEngine::new(clock.clone(), audio_bus);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.track_count, 0);
}

#[test]
fn test_dto_conversion_track_count_one() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);
    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4);
    engine.record_event('q');
    advance_engine(&clock, &mut engine, 4);
    let dto = LoopStateDto::from(&engine);

    assert_eq!(dto.track_count, 1);
}

#[test]
fn test_dto_reverse_conversion_idle() {
    let dto = LoopStateDto {
        status: LoopStatusDto::Idle,
        ticks_remaining: None,
        loop_length: Duration::ZERO,
        current_offset: None,
        saved_offset: None,
        was_recording: None,
        track_count: 0,
    };

    let state: LoopState = dto.into();
    assert_eq!(state, LoopState::Idle);
}

#[test]
fn test_dto_reverse_conversion_ready() {
    let loop_length = Duration::from_secs(2);
    let dto = LoopStateDto {
        status: LoopStatusDto::Ready,
        ticks_remaining: Some(3),
        loop_length,
        current_offset: None,
        saved_offset: None,
        was_recording: None,
        track_count: 0,
    };

    let state: LoopState = dto.into();
    match state {
        LoopState::Ready {
            ticks_remaining,
            loop_length: len,
        } => {
            assert_eq!(ticks_remaining, 3);
            assert_eq!(len, loop_length);
        }
        _ => panic!("Expected Ready state"),
    }
}

#[test]
fn test_dto_reverse_conversion_paused() {
    let loop_length = Duration::from_secs(2);
    let saved_offset = Duration::from_millis(500);
    let dto = LoopStateDto {
        status: LoopStatusDto::Paused,
        ticks_remaining: None,
        loop_length,
        current_offset: Some(saved_offset),
        saved_offset: Some(saved_offset),
        was_recording: Some(true),
        track_count: 1,
    };

    let state: LoopState = dto.into();
    match state {
        LoopState::Paused {
            loop_length: len,
            saved_offset: offset,
            was_recording,
            ..
        } => {
            assert_eq!(len, loop_length);
            assert_eq!(offset, saved_offset);
            assert!(was_recording);
        }
        _ => panic!("Expected Paused state"),
    }
}

#[test]
fn test_dto_conversion_current_offset_calculation() {
    let clock = FakeClock::new(500);
    let audio_bus = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 2);
    let dto_before = LoopStateDto::from(&engine);

    advance_engine(&clock, &mut engine, 2);
    let dto_after = LoopStateDto::from(&engine);

    if let (Some(offset_before), Some(offset_after)) =
        (dto_before.current_offset, dto_after.current_offset)
    {
        assert!(offset_after >= offset_before);
    }
}
