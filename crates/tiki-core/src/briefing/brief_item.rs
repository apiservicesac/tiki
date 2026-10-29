use chrono::NaiveDateTime;
use serde::Serialize;

use crate::domain::{ItemKind, Priority, ProjectId, Repeat, TaskId, TaskStatus};

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
    /// A standalone reminder that rings at this moment, and how often it comes back.
    RemindAt { at: NaiveDateTime, repeat: Repeat },
}

/// One line of the brief.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BriefItem {
    pub task_id: TaskId,
    pub kind: ItemKind,
    pub title: String,
    pub project_id: Option<ProjectId>,
    pub project_name: Option<String>,
    pub tags: Vec<String>,
    pub priority: Priority,
    pub detail: BriefDetail,
}

impl BriefItem {
    /// When a standalone reminder rings; `None` for any other line.
    pub fn rings_at(&self) -> Option<NaiveDateTime> {
        match self.detail {
            BriefDetail::RemindAt { at, .. } => Some(at),
            _ => None,
        }
    }

    /// How often a standalone reminder comes back; `Never` for any other line.
    pub fn repeat(&self) -> Repeat {
        match self.detail {
            BriefDetail::RemindAt { repeat, .. } => repeat,
            _ => Repeat::Never,
        }
    }
}
