use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use termigroove::domain::ports::{AudioBus, Clock};
use termigroove::state::loop_engine::{LoopEngine, LoopState};

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordedCommand {
    Metronome,
    Pad { key: char },
}

#[derive(Clone)]
struct AudioBusMock {
    sent: Rc<RefCell<Vec<RecordedCommand>>>,
}

impl AudioBusMock {
    fn new() -> (Self, Rc<RefCell<Vec<RecordedCommand>>>) {
        let sent = Rc::new(RefCell::new(Vec::new()));
        (Self { sent: sent.clone() }, sent)
    }
}

impl AudioBus for AudioBusMock {
    fn play_metronome_beep(&self) {
        self.sent.borrow_mut().push(RecordedCommand::Metronome);
    }

    fn play_pad(&self, key: char) {
        self.sent.borrow_mut().push(RecordedCommand::Pad { key });
    }
}

const TEST_BPM: u16 = 120;
const TEST_BARS: u16 = 1;

fn advance_engine(clock: &FakeClock, engine: &mut LoopEngine<AudioBusMock, FakeClock>, steps: usize) {
    for _ in 0..steps {
        clock.advance();
        engine.update();
    }
}

#[test]
fn loop_engine_happy_path_transitions_and_audio_commands() {
    let clock = FakeClock::new(500);
    let (audio_bus, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus.clone());

    assert_eq!(engine.state(), LoopState::Idle);

    engine.handle_space(TEST_BPM, TEST_BARS);
    assert_eq!(engine.state(), LoopState::Ready);

    advance_engine(&clock, &mut engine, 4);
    assert_eq!(engine.state(), LoopState::Recording);

    engine.record_event('q');

    // elapsed loop length: bars * 4 beats; with 500ms step at 120 BPM (0.5s per beat) => 2s -> 4 steps
    advance_engine(&clock, &mut engine, 4);
    assert_eq!(engine.state(), LoopState::Playing);

    // allow the next cycle to trigger scheduled playback
    advance_engine(&clock, &mut engine, 1);

    let commands = sent_commands.borrow();
    let metronome_count = commands
        .iter()
        .filter(|cmd| matches!(cmd, RecordedCommand::Metronome))
        .count();
    assert_eq!(metronome_count, 4, "expected four metronome ticks before recording");

    let pad_plays = commands
        .iter()
        .filter(|cmd| matches!(cmd, RecordedCommand::Pad { key } if *key == 'q'))
        .count();
    assert!(pad_plays >= 2, "expected immediate play and at least one scheduled playback for the recorded pad");
}

#[test]
fn metronome_count_in_handles_delayed_update_without_lag() {
    let clock = FakeClock::new(500);
    let (audio_bus, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus.clone());

    engine.handle_space(TEST_BPM, TEST_BARS);

    // Simulate a stalled main loop: advance time past all four ticks before updating once.
    for _ in 0..4 {
        clock.advance();
    }

    engine.update();

    assert!(matches!(engine.state(), LoopState::Recording { .. }));

    let commands = sent_commands.borrow();
    let metronome_count = commands
        .iter()
        .filter(|cmd| matches!(cmd, RecordedCommand::Metronome))
        .count();
    assert_eq!(metronome_count, 4, "expected all metronome ticks despite delayed update");
}
