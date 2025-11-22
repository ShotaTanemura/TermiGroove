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
    Pad { key: char },
    Metronome,
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
fn overdub_first_pad_start_recording_and_schedule_playback() {
    let clock = FakeClock::new(125);
    let (audio_bus, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio_bus.clone());

    // Seed an initial loop so state enters Playing without overdub support.
    engine.handle_space(TEST_BPM, TEST_BARS);
    advance_engine(&clock, &mut engine, 4); // count-in ticks
    engine.record_event('q');
    advance_engine(&clock, &mut engine, 8); // finish recording window
    assert!(matches!(engine.state(), LoopState::Playing { .. }));

    // Clear commands to focus on overdub behavior.
    sent_commands.borrow_mut().clear();

    // Press pad during playback to begin overdub.
    engine.record_event('w');
    assert!(matches!(engine.state(), LoopState::Recording { .. }), "expected overdub to start recording immediately");

    // Overdub stores the initial event at real offset (current fake clock position).
    advance_engine(&clock, &mut engine, 8); // complete loop duration

    assert!(matches!(engine.state(), LoopState::Playing { .. }), "expected overdub to commit and return to playing");

    // During the next cycle we expect scheduled playback for both base and overdub tracks.
    advance_engine(&clock, &mut engine, 8);

    let commands = sent_commands.borrow();
    let metronome_ticks = commands
        .iter()
        .filter(|cmd| matches!(cmd, RecordedCommand::Metronome))
        .count();
    assert_eq!(metronome_ticks, 0, "overdub must not emit metronome ticks");

    let overdub_events: Vec<_> = commands
        .iter()
        .filter_map(|cmd| match cmd {
            RecordedCommand::Pad { key } if *key == 'w' => Some(key),
            _ => None,
        })
        .collect();

    assert!(
        overdub_events.len() >= 1,
        "expected overdub pad to play back in subsequent cycles"
    );
}

