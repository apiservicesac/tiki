use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// What a pending item is. Tickets and meetings join when their connectors arrive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// Work to do and deliver.
    #[default]
    Task,
    /// A standalone heads-up at a time ("call mom at 5"); it is done once it rings.
    Reminder,
}

impl ItemKind {
    pub const ALL: [ItemKind; 2] = [ItemKind::Task, ItemKind::Reminder];

    /// Stable key used in storage.
    pub fn key(self) -> &'static str {
        match self {
            ItemKind::Task => "task",
            ItemKind::Reminder => "reminder",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "item kind",
                value: key.to_owned(),
            })
    }
}
