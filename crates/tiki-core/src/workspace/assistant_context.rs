use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

use crate::domain::{ItemKind, Priority, Project, ProjectDirectory, Task, TaskId, TaskStatus};

/// One open item as the assistant sees it: its id (so it can propose changes to it), never
/// its private notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTask {
    pub task_id: TaskId,
    pub kind: ItemKind,
    pub title: String,
    pub project_name: Option<String>,
    pub tags: Vec<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub remind_at: Option<NaiveDateTime>,
    /// Days waiting on others, when the task is waiting.
    pub waiting_days: Option<i64>,
}

/// A project with the context the user wrote for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantProject {
    pub name: String,
    pub notes: String,
}

/// What the assistant receives to summarize, answer or propose actions: open items, and every
/// project with the notes the user wrote for it (empty when none).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantContext {
    pub now: NaiveDateTime,
    pub open_tasks: Vec<AssistantTask>,
    pub projects: Vec<AssistantProject>,
}

impl AssistantContext {
    pub fn build(tasks: &[Task], projects: &ProjectDirectory, now: NaiveDateTime) -> Self {
        let today = now.date();
        let mut known: Vec<&Project> = projects.projects().collect();
        known.sort_by(|first, second| first.name.cmp(&second.name));
        Self {
            now,
            open_tasks: tasks
                .iter()
                .filter(|task| task.status.is_open())
                .filter_map(|task| {
                    Some(AssistantTask {
                        task_id: task.id?,
                        kind: task.kind,
                        title: task.title.clone(),
                        project_name: projects.name_of(task.project_id).map(str::to_owned),
                        tags: task.tags.clone(),
                        status: task.status,
                        priority: task.priority,
                        due_date: task.due_date,
                        remind_at: task.remind_at,
                        waiting_days: task.days_waiting(today),
                    })
                })
                .collect(),
            projects: known
                .into_iter()
                .map(|project| AssistantProject {
                    name: project.name.clone(),
                    notes: project.notes.clone(),
                })
                .collect(),
        }
    }
}
