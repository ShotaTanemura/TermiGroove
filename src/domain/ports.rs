use std::time::Duration;

/// Port trait for time abstraction.
/// Domain logic depends on this trait to get current time.
/// Infrastructure layer provides concrete implementations (e.g., SystemClock).
pub trait Clock: Clone {
    fn now(&self) -> Duration;
}

/// Port trait for audio operations abstraction.
/// Domain logic depends on this trait to trigger audio playback.
/// Infrastructure layer provides concrete implementations (e.g., SenderAudioBus).
pub trait AudioBus: Clone {
    fn play_metronome_beep(&self);
    fn play_pad(&self, key: char);
    fn play_scheduled(&self, key: char);
    fn pause_all(&self) {}
    fn resume_all(&self) {}
}
