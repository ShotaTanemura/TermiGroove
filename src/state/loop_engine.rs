use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::audio::AudioCommand;

pub trait Clock: Clone {
    fn now(&self) -> Duration;
}

#[derive(Clone)]
pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}

pub trait AudioBus: Clone {
    fn play_metronome_beep(&self);
    fn play_pad(&self, key: char);
    fn play_scheduled(&self, key: char);
}

#[derive(Clone)]
pub struct SenderAudioBus {
    tx: std::sync::mpsc::Sender<AudioCommand>,
}

impl SenderAudioBus {
    pub fn new(tx: std::sync::mpsc::Sender<AudioCommand>) -> Self {
        Self { tx }
    }
}

impl AudioBus for SenderAudioBus {
    fn play_metronome_beep(&self) {
        let _ = self.tx.send(AudioCommand::PlayMetronome);
    }

    fn play_pad(&self, key: char) {
        let _ = self.tx.send(AudioCommand::Play { key });
    }

    fn play_scheduled(&self, key: char) {
        let _ = self.tx.send(AudioCommand::PlayLoop { key });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopState {
    Idle,
    Ready {
        ticks_remaining: u8,
        loop_length: Duration,
    },
    Recording {
        start_time: Duration,
        loop_length: Duration,
    },
    Playing {
        cycle_start: Duration,
        loop_length: Duration,
    },
    Paused {
        cycle_start: Duration,
        loop_length: Duration,
    },
}

#[derive(Debug, Clone)]
struct LoopTrack {
    events: Vec<RecordedEvent>,
    next_event_index: usize,
}

impl LoopTrack {
    fn new(events: Vec<RecordedEvent>) -> Self {
        Self {
            events,
            next_event_index: 0,
        }
    }

    fn reset(&mut self) {
        self.next_event_index = 0;
    }
}

#[derive(Debug, Clone)]
struct RecordedEvent {
    key: char,
    offset: Duration,
}

#[derive(Clone)]
pub struct LoopEngine<A: AudioBus, C: Clock> {
    audio: A,
    clock: C,
    state: LoopState,
    tracks: Vec<LoopTrack>,
    metronome_queue: VecDeque<Duration>,
    overdub_buffer: Vec<RecordedEvent>,
    paused: bool,
}

impl<A: AudioBus, C: Clock> std::fmt::Debug for LoopEngine<A, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopEngine")
            .field("state", &self.state)
            .field("tracks", &self.tracks.len())
            .finish()
    }
}

impl<A: AudioBus, C: Clock> LoopEngine<A, C> {
    pub fn tracks_count(&self) -> usize {
        self.tracks.len()
    }
    fn commit_recording(&mut self, loop_length: Duration, now: Duration) {
        let events = std::mem::take(&mut self.overdub_buffer);
        if !events.is_empty() {
            self.tracks.push(LoopTrack::new(events));
        }
        for track in &mut self.tracks {
            track.reset();
        }
        self.paused = false;
        self.state = LoopState::Playing {
            cycle_start: now,
            loop_length,
        };
    }
    pub fn new(clock: C, audio: A) -> Self {
        Self {
            audio,
            clock,
            state: LoopState::Idle,
            tracks: Vec::new(),
            metronome_queue: VecDeque::new(),
            overdub_buffer: Vec::new(),
            paused: false,
        }
    }

    pub fn state(&self) -> LoopState {
        self.state
    }

    pub fn handle_space(&mut self, bpm: u16, bars: u16) {
        match self.state {
            LoopState::Idle => {}
            LoopState::Playing {
                cycle_start,
                loop_length,
            } => {
                self.state = LoopState::Paused {
                    cycle_start,
                    loop_length,
                };
                self.paused = true;
                return;
            }
            LoopState::Paused {
                cycle_start,
                loop_length,
            } => {
                self.state = LoopState::Playing {
                    cycle_start,
                    loop_length,
                };
                self.paused = false;
                return;
            }
            LoopState::Recording {
                loop_length,
                ..
            } => {
                let now = self.clock.now();
                self.commit_recording(loop_length, now);
                return;
            }
            _ => return,
        }
        let loop_length = loop_length_from(bpm, bars);
        let interval = beat_interval_ms(bpm);
        let now = self.clock.now();
        self.metronome_queue.clear();
        let mut next_tick = now + interval;
        for _ in 0..4 {
            self.metronome_queue.push_back(next_tick);
            next_tick += interval;
        }
        self.state = LoopState::Ready {
            ticks_remaining: 4,
            loop_length,
        };
        self.audio.play_metronome_beep();
        self.update();
    }

    pub fn record_event(&mut self, key: char) {
        match self.state {
            LoopState::Recording { start_time, .. } => {
                let now = self.clock.now();
                let offset = now.saturating_sub(start_time);
                self.audio.play_pad(key);
                self.overdub_buffer.push(RecordedEvent { key, offset });
                self.overdub_buffer
                    .sort_by_key(|event| event.offset);
            }
            LoopState::Playing {
                cycle_start,
                loop_length,
            } => {
                // Start overdub immediately without metronome.
                let now = self.clock.now();
                let elapsed = now.saturating_sub(cycle_start);
                let offset = if loop_length.is_zero() {
                    Duration::ZERO
                } else {
                    let loop_nanos = loop_length.as_nanos();
                    if loop_nanos == 0 {
                        Duration::ZERO
                    } else {
                        let elapsed_nanos = elapsed.as_nanos();
                        let remainder = elapsed_nanos % loop_nanos;
                        Duration::from_nanos(remainder as u64)
                    }
                };
                self.audio.play_pad(key);
                self.state = LoopState::Recording {
                    start_time: cycle_start,
                    loop_length,
                };
                self.paused = false;
                self.overdub_buffer.clear();
                self.overdub_buffer.push(RecordedEvent { key, offset });
            }
            _ => {}
        }
    }

    pub fn handle_cancel(&mut self) {
        match self.state {
            LoopState::Ready { .. }
            | LoopState::Recording { .. }
            | LoopState::Playing { .. }
            | LoopState::Paused { .. } => {
                self.state = LoopState::Idle;
                self.metronome_queue.clear();
                self.tracks.clear();
                self.overdub_buffer.clear();
                self.paused = false;
            }
            LoopState::Idle => {}
        }
    }

    pub fn handle_control_space(&mut self) {
        self.metronome_queue.clear();
        self.tracks.clear();
        self.overdub_buffer.clear();
        self.paused = false;
        self.state = LoopState::Idle;
    }

    pub fn reset_for_new_tempo(&mut self, _bpm: u16, _bars: u16) {
        self.state = LoopState::Idle;
        self.metronome_queue.clear();
        self.tracks.clear();
        self.overdub_buffer.clear();
        self.paused = false;
    }

    pub fn update(&mut self) {
        let now = self.clock.now();
        match self.state {
            LoopState::Ready {
                ref mut ticks_remaining,
                loop_length,
            } => {
                while matches!(self.metronome_queue.front(), Some(&due) if now >= due) {
                    self.metronome_queue.pop_front();
                    if *ticks_remaining == 0 {
                        break;
                    }
                    *ticks_remaining -= 1;
                    if *ticks_remaining == 0 {
                        self.tracks.clear();
                        self.overdub_buffer.clear();
                        self.paused = false;
                        self.state = LoopState::Recording {
                            start_time: now,
                            loop_length,
                        };
                        break;
                    } else {
                        self.audio.play_metronome_beep();
                    }
                }
            }
            LoopState::Recording {
                start_time,
                loop_length,
            } => {
                if now.saturating_sub(start_time) >= loop_length {
                    self.commit_recording(loop_length, now);
                }
            }
            LoopState::Playing {
                ref mut cycle_start,
                loop_length,
            } => {
                let elapsed = now.saturating_sub(*cycle_start);
                if !self.paused {
                    for track in &mut self.tracks {
                        while track.next_event_index < track.events.len() {
                            let event = &track.events[track.next_event_index];
                            if elapsed >= event.offset {
                                self.audio.play_scheduled(event.key);
                                track.next_event_index += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                if elapsed >= loop_length {
                    *cycle_start = now;
                    for track in &mut self.tracks {
                        track.reset();
                    }
                }
            }
            LoopState::Paused { .. } => {
                // No scheduling while paused.
            }
            LoopState::Idle => {}
        }
    }
}

fn loop_length_from(bpm: u16, bars: u16) -> Duration {
    let beats_per_bar = 4.0;
    let beat_seconds = 60.0 / bpm as f64;
    Duration::from_secs_f64(beat_seconds * beats_per_bar * bars as f64)
}

fn beat_interval_ms(bpm: u16) -> Duration {
    Duration::from_secs_f64(60.0 / bpm as f64)
}
