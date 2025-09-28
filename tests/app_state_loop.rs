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
    assert!(matches!(state.loop_state(), LoopState::Recording { .. }));

    input::handle_event(
        &mut state,
        key_event(ratatui::crossterm::event::KeyCode::Char(' ')),
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
        "should not play metronome after cancel during recording"
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
