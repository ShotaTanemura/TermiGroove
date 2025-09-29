use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use termigroove::state::loop_engine::{AudioBus, Clock, LoopEngine, LoopState};

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
    Scheduled { key: char },
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

    fn play_scheduled(&self, key: char) {
        self.sent
            .borrow_mut()
            .push(RecordedCommand::Scheduled { key });
    }
}

const TEST_BPM: u16 = 120;
const TEST_BARS: u16 = 1;

fn advance(clock: &FakeClock, engine: &mut LoopEngine<AudioBusMock, FakeClock>, steps: usize) {
    for _ in 0..steps {
        clock.advance();
        engine.update();
    }
}

#[test]
fn pause_while_playing_stops_scheduled_events_until_resumed() {
    let clock = FakeClock::new(125);
    let (audio, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio.clone());

    // Record a base loop.
    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 4); // count-in ticks
    engine.record_event('q');
    advance(&clock, &mut engine, 8); // finish recording
    assert!(matches!(engine.state(), LoopState::Playing { .. }));

    // Start and commit an overdub layer.
    sent_commands.borrow_mut().clear();
    engine.record_event('w');
    advance(&clock, &mut engine, 8);
    assert!(matches!(engine.state(), LoopState::Playing { .. }));

    // Pause via Space.
    engine.handle_space(TEST_BPM, TEST_BARS);
    assert!(matches!(engine.state(), LoopState::Paused { .. }), "Space should pause playback");

    sent_commands.borrow_mut().clear();
    advance(&clock, &mut engine, 8);

    assert!(sent_commands
        .borrow()
        .iter()
        .all(|cmd| !matches!(cmd, RecordedCommand::Scheduled { .. })), "no scheduled audio should fire while paused");

    // Resume playback.
    engine.handle_space(TEST_BPM, TEST_BARS);
    assert!(matches!(engine.state(), LoopState::Playing { .. }), "engine should resume playing");

    // Next cycle should replay both base and overdub tracks.
    sent_commands.borrow_mut().clear();
    advance(&clock, &mut engine, 8);

    let commands = sent_commands.borrow();
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, RecordedCommand::Scheduled { key: 'q' })), "base track should resume scheduling");
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, RecordedCommand::Scheduled { key: 'w' })), "overdub track should resume scheduling");
}

