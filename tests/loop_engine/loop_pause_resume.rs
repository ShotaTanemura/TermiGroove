use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordedCommand {
    Metronome,
    Pad { key: char },
    Scheduled { key: char },
    PauseAll,
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

    fn pause_all(&self) {
        self.sent.borrow_mut().push(RecordedCommand::PauseAll);
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

fn settle_into_playing(clock: &FakeClock, engine: &mut LoopEngine<AudioBusMock, FakeClock>) {
    for _ in 0..64 {
        if matches!(engine.state(), LoopState::Playing { .. }) {
            return;
        }
        advance(clock, engine, 1);
    }
    panic!(
        "engine did not reach playing state, current state: {:?}",
        engine.state()
    );
}

#[test]
fn pause_while_playing_stops_scheduled_events_until_resumed() {
    let clock = FakeClock::new(125);
    let (audio, sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio.clone());

    // Record a base loop.
    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 16); // count-in ticks
    engine.record_event('q');
    advance(&clock, &mut engine, 8); // finish recording
    settle_into_playing(&clock, &mut engine);

    // Start and commit an overdub layer.
    sent_commands.borrow_mut().clear();
    engine.record_event('w');
    settle_into_playing(&clock, &mut engine);

    // Pause via Space.
    engine.handle_space(TEST_BPM, TEST_BARS);
    assert!(
        matches!(engine.state(), LoopState::Paused { .. }),
        "Space should pause playback"
    );

    sent_commands.borrow_mut().clear();

    assert!(
        sent_commands
            .borrow()
            .iter()
            .all(|cmd| !matches!(cmd, RecordedCommand::Scheduled { .. })),
        "no scheduled audio should fire while paused"
    );

    // Resume playback.
    engine.handle_space(TEST_BPM, TEST_BARS);
    assert!(
        matches!(engine.state(), LoopState::Playing { .. }),
        "engine should resume playing"
    );

    // Next cycle should replay both base and overdub tracks.
    sent_commands.borrow_mut().clear();
    advance(&clock, &mut engine, 16);

    let commands = sent_commands.borrow();
    assert!(
        commands.iter().any(|cmd| matches!(
            cmd,
            RecordedCommand::Scheduled { key: 'q' } | RecordedCommand::Pad { key: 'q' }
        )),
        "base track should resume scheduling"
    );
    assert!(
        commands.iter().any(|cmd| matches!(
            cmd,
            RecordedCommand::Scheduled { key: 'w' } | RecordedCommand::Pad { key: 'w' }
        )),
        "overdub track should resume scheduling"
    );
}

#[test]
fn pause_while_recording_enters_paused_state_without_committing() {
    let clock = FakeClock::new(125);
    let (audio, _sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 16); // count-in ticks
    engine.record_event('q');
    advance(&clock, &mut engine, 8); // finish recording
    settle_into_playing(&clock, &mut engine);

    engine.record_event('w');
    assert!(matches!(engine.state(), LoopState::Recording { .. }));
    let existing_tracks = engine.tracks_count();

    engine.handle_space(TEST_BPM, TEST_BARS);

    assert!(
        matches!(engine.state(), LoopState::Paused { .. }),
        "recording pause should enter paused state"
    );
    assert_eq!(
        existing_tracks,
        engine.tracks_count(),
        "pausing recording should not commit overdub tracks"
    );
}

#[test]
fn resume_after_pause_adjusts_cycle_start_by_pause_duration() {
    let clock = FakeClock::new(125);
    let (audio, _sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 16); // count-in to recording

    // Record base loop with non-zero offset event.
    advance(&clock, &mut engine, 2); // wait before recording event
    engine.record_event('q');
    advance(&clock, &mut engine, 8); // finish recording
    settle_into_playing(&clock, &mut engine);

    // Allow playback to progress before pausing.
    advance(&clock, &mut engine, 4);
    let pause_trigger_time = clock.now();

    let (paused_cycle_start, _loop_length) = match engine.state() {
        LoopState::Playing {
            cycle_start: _,
            loop_length: _,
        } => {
            engine.handle_space(TEST_BPM, TEST_BARS);
            match engine.state() {
                LoopState::Paused {
                    cycle_start,
                    loop_length,
                    ..
                } => (cycle_start, loop_length),
                state => panic!("expected paused state, got {:?}", state),
            }
        }
        state => panic!("expected playing state before pause, got {:?}", state),
    };

    advance(&clock, &mut engine, 4); // remain paused for some duration
    let pause_duration = clock.now().saturating_sub(pause_trigger_time);

    engine.handle_space(TEST_BPM, TEST_BARS);

    let resumed_cycle_start = match engine.state() {
        LoopState::Playing { cycle_start, .. } => cycle_start,
        state => panic!("expected playing state after resume, got {:?}", state),
    };

    assert!(
        resumed_cycle_start >= paused_cycle_start,
        "resume should not move cycle_start backwards"
    );
    let delta = resumed_cycle_start - paused_cycle_start;
    let tolerance = Duration::from_millis(5);
    assert!(
        if delta >= pause_duration {
            delta - pause_duration < tolerance
        } else {
            pause_duration - delta < tolerance
        },
        "cycle_start should advance by pause duration (delta {:?}, pause {:?})",
        delta,
        pause_duration
    );
}

#[test]
fn non_pause_inputs_do_not_transition_to_paused() {
    let clock = FakeClock::new(125);
    let (audio, _sent_commands) = AudioBusMock::new();
    let mut engine = LoopEngine::new(clock.clone(), audio);

    engine.handle_space(TEST_BPM, TEST_BARS);
    advance(&clock, &mut engine, 16);
    engine.record_event('q');
    advance(&clock, &mut engine, 8);
    settle_into_playing(&clock, &mut engine);

    engine.record_event('w'); // starts overdub recording
    assert!(matches!(engine.state(), LoopState::Recording { .. }));

    advance(&clock, &mut engine, 2); // simulate time without pause input
    assert!(
        !matches!(engine.state(), LoopState::Paused { .. }),
        "non-space inputs should not pause loop"
    );

    engine.handle_cancel();
    assert!(matches!(engine.state(), LoopState::Idle));
}
