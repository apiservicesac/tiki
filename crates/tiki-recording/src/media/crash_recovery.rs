use std::fs;
use std::path::Path;
use std::time::Duration;

use super::{MediaSink, OggOpusWriter};
use crate::audio::AudioMixer;
use crate::error::RecordingResult;
use crate::recording_config::RecordingConfig;
use crate::transcription::TrackReader;

/// What a session's folder still holds after Tiki closed while recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredMedia {
    /// The file to play back, and its bytes; `None` when nothing could be saved.
    pub media: Option<(&'static str, u64)>,
    /// How long the tracks last: where the session ends.
    pub recorded: Duration,
}

/// Saves what a recording cut short left behind. The WAV tracks are read up to their last
/// flush (a few seconds lost at most). An audio-only session keeps its Ogg Opus, which plays
/// up to its last page; a video's unfinished MP4 cannot play, so it is removed and the audio
/// is rebuilt from the tracks.
pub struct CrashRecovery;

impl CrashRecovery {
    pub fn recover(folder: &Path, audio_only: bool) -> RecordingResult<RecoveredMedia> {
        let microphone = TrackReader::read(&folder.join(RecordingConfig::MICROPHONE_TRACK_FILE))?;
        let system = TrackReader::read(&folder.join(RecordingConfig::SYSTEM_TRACK_FILE))?;
        let samples = microphone.len().max(system.len());
        let recorded = Duration::from_secs_f64(
            Self::seconds(samples) / f64::from(RecordingConfig::TRANSCRIPTION_SAMPLE_RATE),
        );
        let audio = folder.join(RecordingConfig::AUDIO_FILE);
        if audio_only
            && let Ok(kept) = fs::metadata(&audio)
            && kept.len() > 0
        {
            return Ok(RecoveredMedia {
                media: Some((RecordingConfig::AUDIO_FILE, kept.len())),
                recorded,
            });
        }
        let video = folder.join(RecordingConfig::VIDEO_FILE);
        if video.exists() {
            fs::remove_file(video)?;
        }
        let mut rebuilt = Box::new(OggOpusWriter::create_at(
            &audio,
            RecordingConfig::TRANSCRIPTION_SAMPLE_RATE,
        )?);
        rebuilt.write_audio(&AudioMixer::default().mix(
            &Self::padded(microphone, samples),
            &Self::padded(system, samples),
        ))?;
        Ok(RecoveredMedia {
            media: Some((RecordingConfig::AUDIO_FILE, rebuilt.finish()?)),
            recorded,
        })
    }

    /// The track with silence after it, so both are as long as the longer one.
    fn padded(mut samples: Vec<f32>, length: usize) -> Vec<f32> {
        samples.resize(length, 0.0);
        samples
    }

    // Hours of 16 kHz samples are far below 2^52, so an f64 holds them exactly.
    #[allow(clippy::cast_precision_loss)]
    fn seconds(samples: usize) -> f64 {
        samples as f64
    }
}
