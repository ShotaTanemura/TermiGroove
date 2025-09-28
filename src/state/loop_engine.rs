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
    events: Vec<RecordedEvent>,
    metronome_queue: VecDeque<Duration>,
    next_event_index: usize,
}

impl<A: AudioBus, C: Clock> std::fmt::Debug for LoopEngine<A, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopEngine")
            .field("state", &self.state)
            .field("events", &self.events.len())
            .finish()
    }
}

impl<A: AudioBus, C: Clock> LoopEngine<A, C> {
    pub fn new(clock: C, audio: A) -> Self {
        Self {
            audio,
            clock,
            state: LoopState::Idle,
            events: Vec::new(),
            metronome_queue: VecDeque::new(),
            next_event_index: 0,
        }
    }

    pub fn state(&self) -> LoopState {
        self.state
    }

    pub fn handle_space(&mut self, bpm: u16, bars: u16) {
        if !matches!(self.state, LoopState::Idle) {
            return;
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
        if let LoopState::Recording { start_time, .. } = self.state {
            let now = self.clock.now();
            let offset = now.saturating_sub(start_time);
            self.audio.play_pad(key);
            self.events.push(RecordedEvent { key, offset });
            self.events.sort_by_key(|event| event.offset);
        }
    }

    pub fn handle_cancel(&mut self) {
        match self.state {
            LoopState::Ready { .. } | LoopState::Recording { .. } | LoopState::Playing { .. } => {
                self.state = LoopState::Idle;
                self.metronome_queue.clear();
                self.events.clear();
                self.next_event_index = 0;
            }
            LoopState::Idle => {}
        }
    }

    pub fn reset_for_new_tempo(&mut self, _bpm: u16, _bars: u16) {
        self.state = LoopState::Idle;
        self.metronome_queue.clear();
        self.events.clear();
        self.next_event_index = 0;
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
                        self.events.clear();
                        self.next_event_index = 0;
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
                    self.next_event_index = 0;
                    self.state = LoopState::Playing {
                        cycle_start: now,
                        loop_length,
                    };
                }
            }
            LoopState::Playing {
                ref mut cycle_start,
                loop_length,
            } => {
                let elapsed = now.saturating_sub(*cycle_start);
                while self.next_event_index < self.events.len() {
                    let event = &self.events[self.next_event_index];
                    if elapsed >= event.offset {
                        self.audio.play_scheduled(event.key);
                        self.next_event_index += 1;
                    } else {
                        break;
                    }
                }
                if elapsed >= loop_length {
                    *cycle_start = now;
                    self.next_event_index = 0;
                }
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
