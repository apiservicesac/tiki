//! Where the recorded audio goes: each source as a 16 kHz WAV track for the transcription
//! (and as crash insurance), and the mix as the media played back.

mod crash_recovery;
mod media_sink;
mod ogg_opus_writer;
mod track_writer;

pub use crash_recovery::{CrashRecovery, RecoveredMedia};
pub use media_sink::MediaSink;
pub use ogg_opus_writer::OggOpusWriter;
pub use track_writer::TrackWriter;
