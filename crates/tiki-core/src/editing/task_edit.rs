use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::domain::{Priority, ProjectId, Task, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// The fields the user changed in the edit form. The project comes by name, so the
/// interface never has to know database ids.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEdit {
    pub title: String,
    pub notes: String,
    pub project_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub priority: Priority,
    pub status: TaskStatus,
    /// When to remind the user; `None` removes the reminder.
    pub remind_at: Option<NaiveDateTime>,
}

impl TaskEdit {
    /// Applies the edit to a saved task; `project_id` is the project already resolved by name.
    pub fn apply_to(
        self,
        task: &mut Task,
        project_id: Option<ProjectId>,
        today: NaiveDate,
    ) -> TikiResult<()> {
        let title = self.title.trim();
        if title.is_empty() {
            return Err(TikiError::EmptyTaskTitle);
        }
        title.clone_into(&mut task.title);
        self.notes.trim().clone_into(&mut task.notes);
        task.project_id = project_id;
        task.due_date = self.due_date;
        task.priority = self.priority;
        task.remind_at = self.remind_at;
        task.move_to(self.status, today);
        Ok(())
    }
}
