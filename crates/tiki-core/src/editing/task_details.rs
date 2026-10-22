use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

use crate::domain::{ItemKind, Priority, ProjectDirectory, Repeat, Task, TaskId, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// A task as the edit form shows it; the project comes by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetails {
    pub task_id: TaskId,
    pub title: String,
    pub notes: String,
    pub project_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub priority: Priority,
    pub status: TaskStatus,
    pub remind_at: Option<NaiveDateTime>,
    pub kind: ItemKind,
    pub tags: Vec<String>,
    pub repeat: Repeat,
    pub waiting_for: String,
}

impl TaskDetails {
    pub fn from_task(task: &Task, projects: &ProjectDirectory) -> TikiResult<Self> {
        Ok(Self {
            task_id: task.id.ok_or(TikiError::UnsavedTask)?,
            title: task.title.clone(),
            notes: task.notes.clone(),
            project_name: projects.name_of(task.project_id).map(str::to_owned),
            due_date: task.due_date,
            priority: task.priority,
            status: task.status,
            remind_at: task.remind_at,
            kind: task.kind,
            tags: task.tags.clone(),
            repeat: task.repeat,
            waiting_for: task.waiting_for.clone(),
        })
    }
}
