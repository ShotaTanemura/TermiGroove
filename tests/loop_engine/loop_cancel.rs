use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use termigroove::domain::ports::{AudioBus, Clock};
use termigroove::domain::r#loop::{LoopEngine, LoopState};

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

    fn recorded(&self) -> Rc<RefCell<Vec<RecordedCommand>>> {
        self.sent.clone()
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

fn setup_engine() -> (LoopEngine<AudioBusMock, FakeClock>, FakeClock, Rc<RefCell<Vec<RecordedCommand>>>) {
    let clock = FakeClock::new(500);
    let (audio, sent) = AudioBusMock::new();
    let engine = LoopEngine::new(clock.clone(), audio.clone());
    (engine, clock, sent)
}

fn advance_ready(clock: &FakeClock, engine: &mut LoopEngine<AudioBusMock, FakeClock>) {
    for _ in 0..4 {
        clock.advance();
        engine.update();
    }
}

#[test]
fn cancel_during_ready_returns_to_idle_and_clears_ticks() {
    let (mut engine, clock, sent) = setup_engine();

    engine.handle_space(TEST_BPM, TEST_BARS);
    assert!(matches!(engine.state(), LoopState::Ready { .. }));

    engine.handle_cancel();
    assert_eq!(engine.state(), LoopState::Idle);

    clock.advance();
    engine.update();

    let commands = sent.borrow();
    assert_eq!(commands.iter().filter(|c| matches!(c, RecordedCommand::Metronome)).count(), 1, "only initial metronome expected after cancel");
}

#[test]
fn cancel_during_recording_clears_events_and_returns_idle() {
    let (mut engine, clock, sent) = setup_engine();

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_ready(&clock, &mut engine);
    assert!(matches!(engine.state(), LoopState::Recording { .. }));

    engine.record_event('q');
    assert_eq!(sent.borrow().iter().filter(|c| matches!(c, RecordedCommand::Pad { key: 'q' })).count(), 1);

    engine.handle_cancel();
    assert_eq!(engine.state(), LoopState::Idle);

    clock.advance();
    engine.update();

    let commands = sent.borrow();
    let pad_plays = commands
        .iter()
        .filter(|c| matches!(c, RecordedCommand::Pad { key: 'q' }))
        .count();
    assert_eq!(pad_plays, 1, "recorded event should be cleared; no playback after cancel");
}
