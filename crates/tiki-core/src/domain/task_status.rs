use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Where a task stands in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    ToDo,
    InProgress,
    /// Waiting on someone or something else; see `Task::waiting_for`.
    Waiting,
    Done,
}

impl TaskStatus {
    /// Every status, in the order a task usually moves through them.
    pub const ALL: [TaskStatus; 4] = [
        TaskStatus::ToDo,
        TaskStatus::InProgress,
        TaskStatus::Waiting,
        TaskStatus::Done,
    ];

    /// Stable key used in storage and in translation keys (`status.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            TaskStatus::ToDo => "to_do",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Waiting => "waiting",
            TaskStatus::Done => "done",
        }
    }

    /// Reads a status back from its stable key.
    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "task status",
                value: key.to_owned(),
            })
    }

    /// Whether the task still needs attention.
    pub fn is_open(self) -> bool {
        self != TaskStatus::Done
    }

    /// Whether the next step belongs to the user (not delivered, not waiting on someone else).
    pub fn is_on_user(self) -> bool {
        matches!(self, TaskStatus::ToDo | TaskStatus::InProgress)
    }
}
