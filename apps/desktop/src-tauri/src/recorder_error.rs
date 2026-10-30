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

    #[error("a video session needs a screen or a window to record")]
    NoVideoTarget,

    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),
}
