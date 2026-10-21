use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::{ClientId, Priority, TaskId, TaskOrigin, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// A piece of work the user has to finish or follow up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<TaskId>,
    pub title: String,
    pub notes: String,
    pub client_id: Option<ClientId>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    /// Day the task moved to "waiting on others"; empty in any other status.
    pub waiting_since: Option<NaiveDate>,
    pub origin: TaskOrigin,
}

impl Task {
    /// Creates a manual task in "to do" with normal priority.
    pub fn new(title: impl Into<String>) -> TikiResult<Self> {
        let title = title.into().trim().to_owned();
        if title.is_empty() {
            return Err(TikiError::EmptyTaskTitle);
        }
        Ok(Self {
            id: None,
            title,
            notes: String::new(),
            client_id: None,
            status: TaskStatus::ToDo,
            priority: Priority::default(),
            due_date: None,
            waiting_since: None,
            origin: TaskOrigin::Manual,
        })
    }

    #[must_use]
    pub fn with_client(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    #[must_use]
    pub fn with_due_date(mut self, due_date: NaiveDate) -> Self {
        self.due_date = Some(due_date);
        self
    }

    #[must_use]
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    #[must_use]
    pub fn with_origin(mut self, origin: TaskOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Moves the task to a new status and keeps track of when it started waiting.
    pub fn move_to(&mut self, status: TaskStatus, today: NaiveDate) {
        if status == TaskStatus::WaitingOnOthers && self.status != TaskStatus::WaitingOnOthers {
            self.waiting_since = Some(today);
        } else if status != TaskStatus::WaitingOnOthers {
            self.waiting_since = None;
        }
        self.status = status;
    }

    /// Days until the due date: negative when overdue, `None` without a due date.
    pub fn days_until_due(&self, today: NaiveDate) -> Option<i64> {
        self.due_date.map(|due| (due - today).num_days())
    }

    /// Whether the task is open and its due date has passed.
    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        self.status.is_open() && self.days_until_due(today).is_some_and(|days| days < 0)
    }

    /// Whether the task is open and due between today and `window_days` from now.
    pub fn is_due_within(&self, today: NaiveDate, window_days: i64) -> bool {
        self.status.is_open()
            && self
                .days_until_due(today)
                .is_some_and(|days| (0..=window_days).contains(&days))
    }

    /// Days the task has been waiting on others, if it is waiting.
    pub fn days_waiting(&self, today: NaiveDate) -> Option<i64> {
        match self.status {
            TaskStatus::WaitingOnOthers => {
                self.waiting_since.map(|since| (today - since).num_days())
            }
            _ => None,
        }
    }
}
