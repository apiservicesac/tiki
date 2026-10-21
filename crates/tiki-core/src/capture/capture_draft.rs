use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

use crate::domain::{ClientId, Task};
use crate::error::TikiResult;

/// What the parser understood; the user confirms it before it becomes a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDraft {
    pub title: String,
    pub client_id: Option<ClientId>,
    /// Name of the recognized client, for showing the draft before saving.
    pub client_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    /// When to remind the user, if the sentence asked for it ("avísame a las 9").
    pub remind_at: Option<NaiveDateTime>,
}

impl CaptureDraft {
    pub fn into_task(self) -> TikiResult<Task> {
        let mut task = Task::new(self.title)?;
        task.client_id = self.client_id;
        task.due_date = self.due_date;
        task.remind_at = self.remind_at;
        Ok(task)
    }
}
