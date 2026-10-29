//! Audio from the microphone and the system: aligning the two sources to one clock, mixing
//! them for playback and decimating each one for the transcription.

mod audio_mixer;
mod audio_source;
mod decimator;
mod sample_format;
mod source_aligner;

pub use audio_mixer::AudioMixer;
pub use audio_source::AudioSource;
pub use decimator::Decimator;
pub use sample_format::SampleFormat;
pub use source_aligner::SourceAligner;
