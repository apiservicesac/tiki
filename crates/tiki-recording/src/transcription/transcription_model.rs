use tiki_core::preferences::TranscriptionQuality;

use crate::recording_config::RecordingConfig;

/// A Whisper model file: where to download it from, and what it must hash to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscriptionModel {
    pub file_name: &'static str,
    pub url: &'static str,
    /// Lowercase hexadecimal SHA-256 of the whole file.
    pub sha256: &'static str,
    pub bytes: u64,
}

impl TranscriptionModel {
    /// The model of the quality the user chose.
    pub fn for_quality(quality: TranscriptionQuality) -> Self {
        match quality {
            TranscriptionQuality::Fast => RecordingConfig::FAST_MODEL,
            TranscriptionQuality::Balanced => RecordingConfig::BALANCED_MODEL,
            TranscriptionQuality::Precise => RecordingConfig::PRECISE_MODEL,
        }
    }
}
