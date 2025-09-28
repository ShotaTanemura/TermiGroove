use rodio::{Decoder, OutputStream, Sink, Source, buffer::SamplesBuffer};
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Preload { key: char, path: PathBuf },
    Play { key: char },
    PlayLoop { key: char },
    PlayMetronome,
}

#[derive(Clone)]
struct DecodedSample {
    channels: u16,
    sample_rate: u32,
    samples: Arc<Vec<f32>>, // decoded PCM in f32
}

impl DecodedSample {
    fn to_source(&self) -> SamplesBuffer<f32> {
        SamplesBuffer::new(self.channels, self.sample_rate, (*self.samples).clone())
    }
}

// Generate a short synthesized metronome tick (sine with quick decay).
fn metronome_sample() -> DecodedSample {
    const SAMPLE_RATE: u32 = 44_100;
    const CHANNELS: u16 = 1;
    const DURATION_MS: u32 = 70;
    const FREQ: f32 = 1_000.0;

    let total_samples = (SAMPLE_RATE as u64 * DURATION_MS as u64 / 1_000) as usize;
    let mut data = Vec::with_capacity(total_samples);
    for n in 0..total_samples {
        let t = n as f32 / SAMPLE_RATE as f32;
        // Simple attack/decay envelope
        let attack = 0.005f32;
        let release = (DURATION_MS as f32 / 1_000.0) - attack;
        let env = if t < attack {
            t / attack
        } else if t > release {
            ((DURATION_MS as f32 / 1_000.0) - t).max(0.0) / (DURATION_MS as f32 / 1_000.0 - release)
        } else {
            1.0
        };
        let sample = (2.0 * PI * FREQ * t).sin() * env * 0.4;
        data.push(sample);
    }
    DecodedSample {
        channels: CHANNELS,
        sample_rate: SAMPLE_RATE,
        samples: Arc::new(data),
    }
}

/// Spawn a background audio thread handling preload/play commands using rodio.
pub fn spawn_audio_thread() -> Sender<AudioCommand> {
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    thread::spawn(move || {
        // Keep output stream alive in thread scope
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(v) => v,
            Err(err) => {
                eprintln!("[audio] Failed to open output stream: {err:?}");
                return;
            }
        };

        let mut cache: BTreeMap<char, DecodedSample> = BTreeMap::new();
        let mut sinks: Vec<Sink> = Vec::new();
        let metronome = metronome_sample();

        while let Ok(cmd) = rx.recv() {
            match cmd {
                AudioCommand::Preload { key, path } => match fs::read(&path) {
                    Ok(bytes) => {
                        let cursor = Cursor::new(bytes);
                        match Decoder::new(cursor) {
                            Ok(decoder) => {
                                let channels = decoder.channels();
                                let sample_rate = decoder.sample_rate();
                                let samples: Vec<f32> = decoder.convert_samples().collect();
                                cache.insert(
                                    key,
                                    DecodedSample {
                                        channels,
                                        sample_rate,
                                        samples: Arc::new(samples),
                                    },
                                );
                            }
                            Err(err) => {
                                eprintln!("[audio] Decoder error for {}: {err:?}", path.display());
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("[audio] Failed to read {}: {err:?}", path.display());
                    }
                },
                AudioCommand::Play { key } | AudioCommand::PlayLoop { key } => {
                    if let Some(decoded) = cache.get(&key) {
                        match Sink::try_new(&stream_handle) {
                            Ok(sink) => {
                                sink.append(decoded.to_source());
                                sinks.push(sink);
                                sinks.retain(|s| !s.empty());
                            }
                            Err(err) => eprintln!("[audio] Failed to create Sink: {err:?}"),
                        }
                    } else {
                        eprintln!("[audio] Play requested for key '{}' but not cached", key);
                    }
                }
                AudioCommand::PlayMetronome => {
                    if let Ok(sink) = Sink::try_new(&stream_handle) {
                        sink.append(metronome.to_source());
                        sinks.push(sink);
                        sinks.retain(|s| !s.empty());
                    }
                }
            }
        }
        eprintln!("[audio] receiver closed; audio thread exiting");
    });
    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_thread_accepts_commands() {
        let tx = spawn_audio_thread();
        // Preload path that likely doesn't exist; still should not panic
        let _ = tx.send(AudioCommand::Preload {
            key: 'q',
            path: PathBuf::from("/no/such/file.wav"),
        });
        // Play variants should not panic either
        let _ = tx.send(AudioCommand::Play { key: 'q' });
        let _ = tx.send(AudioCommand::PlayLoop { key: 'q' });
        let _ = tx.send(AudioCommand::PlayMetronome);
    }
}
