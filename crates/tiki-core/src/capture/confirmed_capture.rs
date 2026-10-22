use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::domain::{ItemKind, Priority};

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
}
