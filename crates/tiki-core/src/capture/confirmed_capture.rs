use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::domain::{ItemKind, Priority, Repeat, TaskStatus};

/// An item the user confirmed after the AI read their sentence, or after picking the fields.
/// The project comes by name, so the interface never has to know database ids.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedCapture {
    pub title: String,
    pub project_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub priority: Priority,
    /// When to remind the user, if the sentence asked for it.
    #[serde(default)]
    pub remind_at: Option<NaiveDateTime>,
    /// A task, or a standalone reminder.
    #[serde(default)]
    pub kind: ItemKind,
    #[serde(default)]
    pub tags: Vec<String>,
    /// How often a reminder comes back.
    #[serde(default)]
    pub repeat: Repeat,
    /// Who or what the item waits for, as the user wrote it; when set, the item is waiting.
    #[serde(default)]
    pub waiting_for: Option<String>,
}

impl ConfirmedCapture {
    /// A capture that waits for someone starts in "waiting", any other in "to do".
    pub fn status(&self) -> TaskStatus {
        match self.waiting_for.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => TaskStatus::Waiting,
            _ => TaskStatus::ToDo,
        }
    }
}
