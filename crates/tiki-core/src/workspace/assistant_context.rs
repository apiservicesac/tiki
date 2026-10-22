use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{Priority, ProjectDirectory, Task, TaskStatus};

/// One open task as the assistant sees it: no ids, no notes, only what a summary needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTask {
    pub title: String,
    pub project_name: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    /// Days waiting on others, when the task is waiting.
    pub waiting_days: Option<i64>,
}

/// The minimum the assistant receives to summarize the day or answer a question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantContext {
    pub today: NaiveDate,
    pub open_tasks: Vec<AssistantTask>,
}

impl AssistantContext {
    pub fn build(tasks: &[Task], projects: &ProjectDirectory, today: NaiveDate) -> Self {
        Self {
            today,
            open_tasks: tasks
                .iter()
                .filter(|task| task.status.is_open())
                .map(|task| AssistantTask {
                    title: task.title.clone(),
                    project_name: projects.name_of(task.project_id).map(str::to_owned),
                    status: task.status,
                    priority: task.priority,
                    due_date: task.due_date,
                    waiting_days: task.days_waiting(today),
                })
                .collect(),
        }
    }
}
