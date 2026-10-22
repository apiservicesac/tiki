use chrono::NaiveDateTime;
use serde::Serialize;

use crate::domain::{Priority, TaskId, TaskStatus};

/// Why a task appears in the brief.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BriefDetail {
    /// Due in this many days (0 = today).
    DueIn { days: i64 },
    /// Past its due date by this many days.
    Overdue { days: i64 },
    /// Waiting on others for this many days.
    Waiting { days: i64 },
    /// No date pressure; shows the current status.
    InStatus { status: TaskStatus },
    /// A standalone reminder that rings at this moment.
    RemindAt { at: NaiveDateTime },
}

/// One line of the brief.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BriefItem {
    pub task_id: TaskId,
    pub title: String,
    pub project_name: Option<String>,
    pub tags: Vec<String>,
    pub priority: Priority,
    pub detail: BriefDetail,
}
