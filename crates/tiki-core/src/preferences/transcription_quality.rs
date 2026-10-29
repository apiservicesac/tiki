use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// How carefully sessions are transcribed: a faster model, the balanced default, or a more
/// precise one that is bigger and slower. The recorder maps each one to a Whisper model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionQuality {
    Fast,
    #[default]
    Balanced,
    Precise,
}

impl TranscriptionQuality {
    pub const ALL: [TranscriptionQuality; 3] = [
        TranscriptionQuality::Fast,
        TranscriptionQuality::Balanced,
        TranscriptionQuality::Precise,
    ];

    /// Stable key used in storage and in translation keys (`transcription.quality.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            TranscriptionQuality::Fast => "fast",
            TranscriptionQuality::Balanced => "balanced",
            TranscriptionQuality::Precise => "precise",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|quality| quality.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "transcription quality",
                value: key.to_owned(),
            })
    }
}
