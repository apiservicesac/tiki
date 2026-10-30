use crate::error::RecordingResult;

/// Where audio comes from: the microphone or the system audio (what the others say), as
/// mono samples at `RecordingConfig::MIX_SAMPLE_RATE`. A capture runs from the moment the
/// source is created until it is dropped.
pub trait AudioSource: Send {
    /// Which source it is, for logs and errors ("microphone", "system audio").
    fn name(&self) -> &'static str;

    /// Appends the samples captured since the last call. A silent system may give none:
    /// Windows sends no audio while nothing plays.
    fn drain_into(&mut self, samples: &mut Vec<f32>) -> RecordingResult<()>;
}
