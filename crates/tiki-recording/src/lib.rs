//! Recording and local transcription of Tiki's sessions, kept apart from the core so its
//! heavy dependencies (libopus, whisper.cpp, Windows capture) never reach it.
//!
//! Everything platform-neutral is here and tested with fakes: aligning and mixing the
//! microphone with the system audio, the WAV and Ogg Opus writers, the pipeline with its
//! pause clock, the model download and the transcription queue. The Windows sources and the
//! MP4 writer live behind `cfg(windows)`, and Whisper behind the `whisper` feature.

pub mod audio;
pub mod error;
pub mod media;
pub mod pipeline;
pub mod recording_config;
pub mod transcription;
pub mod video;

pub use error::{RecordingError, RecordingResult};
pub use recording_config::RecordingConfig;
