//! Local transcription: the Whisper model download, the transcriber and the queue of
//! sessions waiting for their transcript.

mod model_downloader;
mod track_reader;
mod transcriber;
mod transcription_model;
mod transcription_worker;
#[cfg(feature = "whisper")]
mod whisper_transcriber;

pub use model_downloader::{DownloadProgress, ModelDownloader};
pub use track_reader::TrackReader;
pub use transcriber::{TimedText, TranscribedAudio, Transcriber};
pub use transcription_model::TranscriptionModel;
pub use transcription_worker::{TranscriptionJob, TranscriptionQueue, TranscriptionWorker};
#[cfg(feature = "whisper")]
pub use whisper_transcriber::WhisperTranscriber;
