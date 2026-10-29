use crate::error::RecordingResult;

/// The media played back: Ogg Opus for an audio-only session, MP4 with the video otherwise.
pub trait MediaSink: Send {
    /// Appends mixed mono samples at `RecordingConfig::MIX_SAMPLE_RATE`.
    fn write_audio(&mut self, mixed: &[f32]) -> RecordingResult<()>;

    /// Closes the file; returns how many bytes it takes on disk.
    fn finish(self: Box<Self>) -> RecordingResult<u64>;
}
