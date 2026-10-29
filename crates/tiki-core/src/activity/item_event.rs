use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::ItemField;
use crate::domain::TaskStatus;

/// Something that happened to an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemEvent {
    /// The user captured it.
    Created,
    /// A connector brought it from another system.
    Imported,
    /// The user changed these fields.
    Edited {
        fields: Vec<ItemField>,
    },
    /// It changed status, without being delivered or reopened.
    Moved {
        from: TaskStatus,
        to: TaskStatus,
    },
    /// Its reminder was put off until this moment.
    Snoozed {
        until: NaiveDateTime,
    },
    /// Its reminder or follow-up nudge rang.
    Rang,
    Delivered,
    /// It came back after being delivered.
    Reopened,
}

impl ItemEvent {
    /// The event a status change records, if the status changed.
    pub fn status_change(from: TaskStatus, to: TaskStatus) -> Option<Self> {
        match (from, to) {
            _ if from == to => None,
            (_, TaskStatus::Done) => Some(ItemEvent::Delivered),
            (TaskStatus::Done, _) => Some(ItemEvent::Reopened),
            _ => Some(ItemEvent::Moved { from, to }),
        }
    }

    /// Stable key used in storage.
    pub fn key(&self) -> &'static str {
        match self {
            ItemEvent::Created => "created",
            ItemEvent::Imported => "imported",
            ItemEvent::Edited { .. } => "edited",
            ItemEvent::Moved { .. } => "moved",
            ItemEvent::Snoozed { .. } => "snoozed",
            ItemEvent::Rang => "rang",
            ItemEvent::Delivered => "delivered",
            ItemEvent::Reopened => "reopened",
        }
    }
}
