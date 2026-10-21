use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// How important a task is compared with the others.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl Priority {
    pub const ALL: [Priority; 4] = [
        Priority::Low,
        Priority::Normal,
        Priority::High,
        Priority::Urgent,
    ];

    /// Stable key used in storage and in translation keys (`priority.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        }
    }

    /// Reads a priority back from its stable key.
    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|priority| priority.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "priority",
                value: key.to_owned(),
            })
    }
}
