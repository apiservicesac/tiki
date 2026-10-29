use thiserror::Error;
use tiki_core::TikiError;
use tiki_recording::RecordingError;

/// Why the recorder refused or stopped: a rule of the core, a device or a file.
#[derive(Debug, Error)]
pub enum RecorderError {
    #[error(transparent)]
    Core(#[from] TikiError),

    #[error(transparent)]
    Recording(#[from] RecordingError),

    #[error("no session is being recorded")]
    NotRecording,

    #[error("video recording is not available yet")]
    VideoUnavailable,

    #[error("could not start the recorder thread: {0}")]
    Thread(#[from] std::io::Error),
}
