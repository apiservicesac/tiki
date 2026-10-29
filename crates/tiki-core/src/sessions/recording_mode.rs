use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// What a session records, chosen by the user each time. Both are always transcribed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    AudioOnly,
    AudioAndVideo,
}

impl RecordingMode {
    pub const ALL: [RecordingMode; 2] = [RecordingMode::AudioOnly, RecordingMode::AudioAndVideo];

    /// Stable key used in storage and in translation keys (`session.mode.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            RecordingMode::AudioOnly => "audio_only",
            RecordingMode::AudioAndVideo => "audio_and_video",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|mode| mode.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "recording mode",
                value: key.to_owned(),
            })
    }
}

/// How a video session is encoded, chosen when it starts; the recorder maps it to a preset.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoQuality {
    /// Light files for meetings and screen shares.
    #[default]
    Normal,
    /// Smoother and sharper, for demos with motion.
    High,
}
