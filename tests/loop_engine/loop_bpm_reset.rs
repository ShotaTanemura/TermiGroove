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

    fn advance(&self, steps: usize) {
        let mut now = self.now.borrow_mut();
        for _ in 0..steps {
            *now += self.step;
        }
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

const BPM: u16 = 120;
const BARS: u16 = 1;

fn setup_engine() -> (LoopEngine<AudioBusMock, FakeClock>, FakeClock, Rc<RefCell<Vec<RecordedCommand>>>) {
    let clock = FakeClock::new(500);
    let (audio, sent) = AudioBusMock::new();
    let engine = LoopEngine::new(clock.clone(), audio);
    (engine, clock, sent)
}

#[test]
fn tempo_change_resets_loop_and_state() {
    let (mut engine, clock, sent) = setup_engine();

    engine.handle_space(BPM, BARS);
    clock.advance(4);
    engine.update();
    assert!(matches!(engine.state(), LoopState::Recording { .. }));

    engine.record_event('q');
    assert_eq!(sent.borrow().iter().filter(|c| matches!(c, RecordedCommand::Pad { key: 'q' })).count(), 1);

    engine.reset_for_new_tempo(90, 2);
    assert_eq!(engine.state(), LoopState::Idle);

    clock.advance(4);
    engine.update();
    let pad_plays = sent.borrow().iter().filter(|c| matches!(c, RecordedCommand::Pad { key: 'q' })).count();
    assert_eq!(pad_plays, 1, "events should not replay after tempo reset");
}
