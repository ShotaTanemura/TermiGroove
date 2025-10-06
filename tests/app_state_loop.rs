use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termigroove::app_state::{AppState, Mode, SampleSlot};
use termigroove::audio::AudioCommand;
use termigroove::input;
use termigroove::state::loop_engine::{LoopEngine, LoopState, SenderAudioBus, SystemClock};

fn key_event(code: ratatui::crossterm::event::KeyCode) -> ratatui::crossterm::event::Event {
    ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::from(code))
}

fn key_event_with_modifiers(
    code: ratatui::crossterm::event::KeyCode,
    modifiers: ratatui::crossterm::event::KeyModifiers,
) -> ratatui::crossterm::event::Event {
    ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(code, modifiers))
}

fn drain_commands(rx: &mpsc::Receiver<AudioCommand>) -> Vec<AudioCommand> {
    let mut cmds = Vec::new();
    while let Ok(cmd) = rx.try_recv() {
        cmds.push(cmd);
    }
    cmds
}

fn update_until<F>(state: &mut AppState, predicate: F, max_iters: usize)
where
    F: Fn(LoopState) -> bool,
{
    for _ in 0..max_iters {
        state.update_loop();
        if predicate(state.loop_state()) {
            break;
        }
        thread::sleep(Duration::from_millis(15));
    }
}

fn wait_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

#[test]
fn loop_ready_record_play_sequence_dispatches_audio_commands() {
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(600);
    state.set_bars(1);
    state.pads.key_to_slot.insert(
        'q',
        SampleSlot {
            file_name: "kick.wav".into(),
        },
    );

    // Space to enter Ready
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Ready { .. }));

    // Wait until Recording
    update_until(&mut state, |s| matches!(s, LoopState::Recording { .. }), 60);
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));

    // Trigger pad event while recording
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char('q')),
    )
    .unwrap();
    state.update_loop();

    // Wait until Playing
    update_until(&mut state, |s| matches!(s, LoopState::Playing { .. }), 60);
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    // Allow scheduled playback in playing state
    update_until(&mut state, |_| false, 10);

    drop(state); // drop before draining channel
    thread::sleep(Duration::from_millis(50));
    let commands = drain_commands(&rx);

    let metronome_count = commands
        .iter()
        .filter(|cmd| matches!(cmd, AudioCommand::PlayMetronome))
        .count();
    let pad_count = commands
        .iter()
        .filter(|cmd| {
            matches!(
                cmd,
                AudioCommand::Play { key: 'q' } | AudioCommand::PlayLoop { key: 'q' }
            )
        })
        .count();

    assert!(metronome_count >= 4, "expected metronome ticks to be sent");
    assert!(pad_count >= 2, "expected pad play and loop scheduling");
}

#[test]
fn app_state_handles_pause_resume_and_clear_commands() {
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(600);
    state.set_bars(1);
    state.pads.key_to_slot.insert(
        'q',
        SampleSlot {
            file_name: "kick.wav".into(),
        },
    );
    state.pads.key_to_slot.insert(
        'w',
        SampleSlot {
            file_name: "snare.wav".into(),
        },
    );

    // Enter recording and capture base loop.
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    update_until(&mut state, |s| matches!(s, LoopState::Recording { .. }), 60);
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char('q')),
    )
    .unwrap();
    update_until(&mut state, |s| matches!(s, LoopState::Playing { .. }), 60);

    // Overdub another layer.
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char('w')),
    )
    .unwrap();
    update_until(&mut state, |s| matches!(s, LoopState::Playing { .. }), 60);

    // Space should pause.
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Paused { .. }));

    // While paused, no playback occurs.
    drain_commands(&rx);
    update_until(&mut state, |_| false, 10);
    wait_ms(20);
    let commands_paused = drain_commands(&rx);
    assert!(
        commands_paused
            .iter()
            .all(|cmd| !matches!(cmd, AudioCommand::PlayLoop { .. })),
        "no loop playback while paused"
    );

    // Resume with Space.
    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    // Allow scheduling to confirm playback resumes.
    update_until(&mut state, |_| false, 10);
    wait_ms(20);
    let commands_resume = drain_commands(&rx);
    assert!(
        commands_resume
            .iter()
            .any(|cmd| matches!(cmd, AudioCommand::PlayLoop { key: 'q' })),
        "base track resumes playback"
    );
    assert!(
        commands_resume
            .iter()
            .any(|cmd| matches!(cmd, AudioCommand::PlayLoop { key: 'w' })),
        "overdub track resumes playback"
    );

    // Control+Space should clear everything.
    input::handle_event(
        &mut state,
        key_event_with_modifiers(
            ratatui::crossterm::event::KeyCode::Char(' '),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        ),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Idle));

    state.update_loop();

    drop(state);
    thread::sleep(Duration::from_millis(20));
    let commands = drain_commands(&rx);

    assert!(
        commands
            .iter()
            .all(|cmd| !matches!(cmd, AudioCommand::PlayLoop { .. })),
        "no loop playback after clear"
    );
}

#[test]
fn space_during_recording_stops_without_new_metronome() {
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(600);
    state.set_bars(1);

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();

    update_until(
        &mut state,
        |s| matches!(s, LoopState::Recording { .. }),
        100,
    );
    state.update_loop();

    drop(state);
    thread::sleep(Duration::from_millis(20));
    let commands = drain_commands(&rx);

    assert!(
        commands
            .iter()
            .filter(|cmd| matches!(cmd, AudioCommand::PlayMetronome))
            .count()
            >= 4
    );
}

#[test]
fn space_during_playing_stops_without_restart_of_metronome() {
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    let bus = SenderAudioBus::new(tx.clone());
    let loop_engine = LoopEngine::new(SystemClock::new(), bus);
    let mut state = AppState::from_components(tx, loop_engine).expect("init");
    state.mode = Mode::Pads;
    state.set_bpm(600);
    state.set_bars(1);
    state.pads.key_to_slot.insert(
        'q',
        SampleSlot {
            file_name: "kick.wav".into(),
        },
    );

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();

    update_until(
        &mut state,
        |s| matches!(s, LoopState::Recording { .. }),
        100,
    );
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char('q')),
    )
    .unwrap();

    update_until(&mut state, |s| matches!(s, LoopState::Playing { .. }), 100);
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Paused { .. }));

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Playing { .. }));

    input::handle_event(
        &mut state,
        key_event_with_modifiers(
            ratatui::crossterm::event::KeyCode::Char(' '),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        ),
    )
    .unwrap();
    assert!(matches!(state.loop_state(), LoopState::Idle));

    state.update_loop();

    drop(state);
    thread::sleep(Duration::from_millis(20));
    let commands = drain_commands(&rx);

    let metronome_count = commands
        .iter()
        .filter(|cmd| matches!(cmd, AudioCommand::PlayMetronome))
        .count();
    assert_eq!(
        metronome_count, 4,
        "should not restart metronome when stopping from playing"
    );
}
