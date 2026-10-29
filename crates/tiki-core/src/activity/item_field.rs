use serde::{Deserialize, Serialize};

use crate::domain::Task;

/// A field the user can change on an item; an `Edited` event lists the ones that changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemField {
    Title,
    Notes,
    Project,
    DueDate,
    Priority,
    Reminder,
    Kind,
    Tags,
    Repeat,
    WaitingFor,
}

impl ItemField {
    /// The fields that differ between two versions of an item; status changes are recorded
    /// as their own events.
    pub fn changed_between(before: &Task, after: &Task) -> Vec<Self> {
        [
            (ItemField::Title, before.title != after.title),
            (ItemField::Notes, before.notes != after.notes),
            (ItemField::Project, before.project_id != after.project_id),
            (ItemField::DueDate, before.due_date != after.due_date),
            (ItemField::Priority, before.priority != after.priority),
            (ItemField::Reminder, before.remind_at != after.remind_at),
            (ItemField::Kind, before.kind != after.kind),
            (ItemField::Tags, before.tags != after.tags),
            (ItemField::Repeat, before.repeat != after.repeat),
            (
                ItemField::WaitingFor,
                before.waiting_for != after.waiting_for,
            ),
        ]
        .into_iter()
        .filter_map(|(field, changed)| changed.then_some(field))
        .collect()
    }
}
