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
fn control_space_clears_tracks_and_resets_to_idle() {
    let clock = FakeClock::new(125);
    let (audio, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio.clone());

    // Record base track.
    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 4);
    engine.record_event('q');
    advance(&clock, &mut engine, 8);
    assert!(matches!(engine.state(), LoopState::Playing { .. }));

    // Record overdub track.
    engine.record_event('w');
    advance(&clock, &mut engine, 8);

    assert!(matches!(engine.state(), LoopState::Playing { .. }));

    // Issue clear command (simulated via direct call for now).
    engine.handle_control_space();

    assert!(matches!(engine.state(), LoopState::Idle), "engine should return to Idle after clear");

    sent_commands.borrow_mut().clear();
    advance(&clock, &mut engine, 8);
    assert!(sent_commands
        .borrow()
        .iter()
        .all(|cmd| !matches!(cmd, RecordedCommand::Scheduled { .. })), "no scheduled playback after clear");
}

