use crate::error::RecordingResult;

/// A stretch of speech and its time in the track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedText {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// What a transcriber heard in one track.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranscribedAudio {
    /// The language it detected ("es", "en").
    pub language: Option<String>,
    pub segments: Vec<TimedText>,
}

impl TranscribedAudio {
    /// How much of the track is speech, to tell which language the session was in.
    pub fn spoken_ms(&self) -> i64 {
        self.segments
            .iter()
            .map(|segment| segment.end_ms - segment.start_ms)
            .sum()
    }
}

/// Turns 16 kHz mono speech into timed text, on this computer.
pub trait Transcriber {
    fn transcribe(&mut self, samples: &[f32]) -> RecordingResult<TranscribedAudio>;
}
