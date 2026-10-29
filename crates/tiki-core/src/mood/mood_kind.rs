use serde::{Deserialize, Serialize};

/// Every face the mascot can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    /// Nothing due soon and nothing late.
    Calm,
    /// Something is due today or tomorrow.
    Hurry,
    /// Something is overdue.
    Worried,
    /// A meeting is about to start.
    Alert,
    /// Something has waited on others past the follow-up threshold.
    Impatient,
    /// A new ticket or email arrived, or Tiki is investigating.
    Curious,
    /// A task was delivered or a source was connected.
    Happy,
    /// Focus mode or the night's quiet hours: Tiki keeps quiet.
    Sleep,
}

impl Mood {
    /// Stable key shared with the interface (`mood.label.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Mood::Calm => "calm",
            Mood::Hurry => "hurry",
            Mood::Worried => "worried",
            Mood::Alert => "alert",
            Mood::Impatient => "impatient",
            Mood::Curious => "curious",
            Mood::Happy => "happy",
            Mood::Sleep => "sleep",
        }
    }
}
