use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

use crate::domain::{ClientDirectory, Priority, Task, TaskId, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// A task as the edit form shows it; the client comes by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetails {
    pub task_id: TaskId,
    pub title: String,
    pub notes: String,
    pub client_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub priority: Priority,
    pub status: TaskStatus,
    pub remind_at: Option<NaiveDateTime>,
}

impl TaskDetails {
    pub fn from_task(task: &Task, clients: &ClientDirectory) -> TikiResult<Self> {
        Ok(Self {
            task_id: task.id.ok_or(TikiError::UnsavedTask)?,
            title: task.title.clone(),
            notes: task.notes.clone(),
            client_name: clients.name_of(task.client_id).map(str::to_owned),
            due_date: task.due_date,
            priority: task.priority,
            status: task.status,
            remind_at: task.remind_at,
        })
    }
}
