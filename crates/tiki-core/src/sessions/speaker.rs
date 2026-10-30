use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Who said a line: the microphone is the user, the system audio everyone else. Each source
/// is transcribed apart, so the label needs no guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speaker {
    You,
    Others,
}

impl Speaker {
    pub const ALL: [Speaker; 2] = [Speaker::You, Speaker::Others];

    /// Stable key used in storage and in translation keys (`session.speaker.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Speaker::You => "you",
            Speaker::Others => "others",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|speaker| speaker.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "speaker",
                value: key.to_owned(),
            })
    }
}
