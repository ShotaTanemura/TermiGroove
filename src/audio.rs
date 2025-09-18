use rodio::{Decoder, OutputStream, Sink, Source, buffer::SamplesBuffer};
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Preload { key: char, path: PathBuf },
    Play { key: char },
}

#[derive(Clone)]
struct DecodedSample {
    channels: u16,
    sample_rate: u32,
    samples: std::sync::Arc<Vec<f32>>, // decoded PCM in f32
}

impl DecodedSample {
    fn to_source(&self) -> SamplesBuffer<f32> {
        SamplesBuffer::new(self.channels, self.sample_rate, (*self.samples).clone())
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
                                        samples: std::sync::Arc::new(samples),
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
                AudioCommand::Play { key } => {
                    if let Some(decoded) = cache.get(&key) {
                        match Sink::try_new(&stream_handle) {
                            Ok(sink) => {
                                let source = decoded.to_source();
                                sink.append(source);
                                sinks.push(sink);
                                // Optional: prune finished sinks to avoid unbounded growth
                                sinks.retain(|s| !s.empty());
                            }
                            Err(err) => eprintln!("[audio] Failed to create Sink: {err:?}"),
                        }
                    } else {
                        eprintln!("[audio] Play requested for key '{}' but not cached", key);
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
        // Play for non-cached key; should not panic either
        let _ = tx.send(AudioCommand::Play { key: 'q' });
    }
}
