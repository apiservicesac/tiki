use serde::{Deserialize, Serialize};

/// When the brief is delivered; it changes the greeting and the title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BriefMoment {
    Morning,
    Evening,
}

impl BriefMoment {
    pub fn key(self) -> &'static str {
        match self {
            BriefMoment::Morning => "morning",
            BriefMoment::Evening => "evening",
        }
    }
}
