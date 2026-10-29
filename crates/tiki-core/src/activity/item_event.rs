use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::ItemField;
use crate::domain::{ItemKind, TaskStatus};

/// Something that happened to an item. Only `Delivered` counts as work done: the delivered
/// history and the reports read it alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemEvent {
    /// The user captured it.
    Created,
    /// A connector brought it from another system.
    Imported,
    /// The user changed these fields.
    Edited { fields: Vec<ItemField> },
    /// It changed status without being delivered or reopened: a task moving between open
    /// statuses, or any move of a standalone reminder. Reminders are not work, so they are
    /// never delivered: one that rang once closes with `Rang` and a move to done.
    Moved { from: TaskStatus, to: TaskStatus },
    /// Its reminder was put off until this moment.
    Snoozed { until: NaiveDateTime },
    /// Its reminder rang: a standalone reminder, or the alarm the user set on a task.
    Rang,
    /// Tiki nudged the user to follow up the item that waits on others.
    Nudged,
    /// The user delivered the task.
    Delivered,
    /// It came back after being delivered.
    Reopened,
    /// The user sent it to the trash: it shows nowhere else and its reminders stay silent.
    Trashed,
    /// The user took it out of the trash, as it was.
    Restored,
}

impl ItemEvent {
    /// The event a status change of an item of this kind records, if the status changed.
    pub fn status_change(kind: ItemKind, from: TaskStatus, to: TaskStatus) -> Option<Self> {
        match (from, to) {
            _ if from == to => None,
            _ if kind == ItemKind::Reminder => Some(ItemEvent::Moved { from, to }),
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
            ItemEvent::Nudged => "nudged",
            ItemEvent::Delivered => "delivered",
            ItemEvent::Reopened => "reopened",
            ItemEvent::Trashed => "trashed",
            ItemEvent::Restored => "restored",
        }
    }
}
