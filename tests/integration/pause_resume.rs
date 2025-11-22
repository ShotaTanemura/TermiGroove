use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use termigroove::app_state::{AppState, Mode, SampleSlot};
use termigroove::audio::{AudioCommand, SenderAudioBus};
use termigroove::input;
use termigroove::domain::ports::Clock;
use termigroove::state::loop_engine::{LoopEngine, LoopState};

#[derive(Clone)]
struct TestClock {
    now: Rc<RefCell<Duration>>,
}

impl TestClock {
    fn new() -> Self {
        Self {
            now: Rc::new(RefCell::new(Duration::from_millis(0))),
        }
    }

    fn advance_ms(&self, delta: u64) {
        let mut now = self.now.borrow_mut();
        *now += Duration::from_millis(delta);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Duration {
        *self.now.borrow()
    }
}

fn advance_app(state: &mut AppState, clock: &TestClock, millis: u64) {
    for _ in 0..millis {
        clock.advance_ms(1);
        state.update_loop();
    }
}

fn drain_commands(rx: &mpsc::Receiver<AudioCommand>) -> Vec<AudioCommand> {
    let mut cmds = Vec::new();
    while let Ok(cmd) = rx.try_recv() {
        cmds.push(cmd);
    }
    cmds
}

fn setup_test_state(clock: TestClock) -> (AppState, mpsc::Receiver<AudioCommand>, TestClock) {
    let (tx, rx) = mpsc::channel();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(clock.clone(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(120);
    state.set_bars(1);
    state
        .pads
        .key_to_slot
        .insert('q', SampleSlot { file_name: "kick.wav".into() });
    state
        .pads
        .key_to_slot
        .insert('w', SampleSlot { file_name: "snare.wav".into() });
    (state, rx, clock)
}

#[test]
fn pause_all_path_and_resume_alignment() {
    let clock = TestClock::new();
    let (mut state, rx, clock) = setup_test_state(clock);

    input::handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyCode::Char(' ').into(),
        ),
    )
    .unwrap();

    advance_app(&mut state, &clock, 512);
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));

    input::handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyCode::Char('q').into(),
        ),
    )
    .unwrap();

    advance_app(&mut state, &clock, 512);
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    input::handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyCode::Char('w').into(),
        ),
    )
    .unwrap();

    advance_app(&mut state, &clock, 256);
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));

    advance_app(&mut state, &clock, 256);
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    drain_commands(&rx);
    input::handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyCode::Char(' ').into(),
        ),
    )
    .unwrap();

    assert!(matches!(state.loop_state(), LoopState::Paused { .. }));

    let paused_cmds = drain_commands(&rx);
    assert!(
        paused_cmds
            .iter()
            .any(|cmd| matches!(cmd, AudioCommand::PauseAll)),
        "PauseAll command should be emitted when pausing"
    );

    if let LoopState::Paused {
        saved_offset, loop_length, ..
    } = state.loop_state()
    {
        assert!(saved_offset < loop_length);
    } else {
        panic!("engine not paused");
    }

    advance_app(&mut state, &clock, 128);
    assert!(matches!(state.loop_state(), LoopState::Paused { .. }));

    input::handle_event(
        &mut state,
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyCode::Char(' ').into(),
        ),
    )
    .unwrap();

    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    let mut resume_cmds = Vec::new();
    for _ in 0..512 {
        clock.advance_ms(1);
        state.update_loop();
        resume_cmds.extend(drain_commands(&rx));
        let base_resumed = resume_cmds.iter().any(|cmd| {
            matches!(
                cmd,
                AudioCommand::PlayLoop { key: 'q' } | AudioCommand::Play { key: 'q' }
            )
        });
        let overdub_resumed = resume_cmds.iter().any(|cmd| {
            matches!(
                cmd,
                AudioCommand::PlayLoop { key: 'w' } | AudioCommand::Play { key: 'w' }
            )
        });
        if base_resumed && overdub_resumed {
            break;
        }
    }

    assert!(
        resume_cmds
            .iter()
            .any(|cmd| matches!(cmd, AudioCommand::PlayLoop { .. })),
        "scheduled loop commands should resume"
    );
    assert!(
        !resume_cmds
            .iter()
            .any(|cmd| matches!(cmd, AudioCommand::PauseAll)),
        "PauseAll should not fire after resuming"
    );
}

