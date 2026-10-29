use thiserror::Error;

/// Every failure the recorder and the transcriber can report.
#[derive(Debug, Error)]
pub enum RecordingError {
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not write the WAV track: {0}")]
    Wav(#[from] hound::Error),

    #[error("could not encode the audio as Opus: {0}")]
    Opus(#[from] opus::Error),

    #[error("could not encode the media: {0}")]
    Encoder(String),

    #[error("could not capture {source_name}: {reason}")]
    Capture {
        source_name: &'static str,
        reason: String,
    },

    #[error("could not download the transcription model: {0}")]
    Download(String),

    #[error("the downloaded model does not match its checksum")]
    ChecksumMismatch,

    #[error("could not transcribe: {0}")]
    Transcription(String),
}

pub type RecordingResult<T> = Result<T, RecordingError>;
