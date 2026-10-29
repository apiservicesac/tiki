use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use super::{ItemKind, Priority, ProjectId, Repeat, TaskId, TaskOrigin, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// A piece of work the user has to finish or follow up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<TaskId>,
    pub kind: ItemKind,
    pub title: String,
    pub notes: String,
    pub project_id: Option<ProjectId>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    /// Day the task moved to "waiting on others"; empty in any other status.
    pub waiting_since: Option<NaiveDate>,
    /// Day the task was delivered; empty while it is open.
    pub completed_on: Option<NaiveDate>,
    /// When the user asked Tiki to remind them (a reminder's moment, or an alarm on a task);
    /// cleared once it rang, unless the reminder repeats.
    pub remind_at: Option<NaiveDateTime>,
    /// When Tiki nudges the user to follow up a waiting item; empty in any other status.
    pub follow_up_at: Option<NaiveDateTime>,
    /// Lowercase tags without "#" (see `Tags`).
    pub tags: Vec<String>,
    /// How often a reminder comes back after it rings.
    pub repeat: Repeat,
    /// Who or what a waiting task waits for, as the user wrote it ("Lucía's answer").
    pub waiting_for: String,
    pub origin: TaskOrigin,
    /// When the task was first saved and last changed, in local time from the workspace
    /// clock; empty until it is saved. Rows saved before version 0.5 hold UTC instead
    /// (the database's `CURRENT_TIMESTAMP` default), up to a few hours off.
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
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
            kind: ItemKind::Task,
            title,
            notes: String::new(),
            project_id: None,
            status: TaskStatus::ToDo,
            priority: Priority::default(),
            due_date: None,
            waiting_since: None,
            completed_on: None,
            remind_at: None,
            follow_up_at: None,
            tags: Vec::new(),
            repeat: Repeat::Never,
            waiting_for: String::new(),
            origin: TaskOrigin::Manual,
            created_at: None,
            updated_at: None,
        })
    }

    #[must_use]
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
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

    /// Whether the item is open work: not delivered and not a standalone reminder.
    pub fn counts_as_open_work(&self) -> bool {
        self.kind == ItemKind::Task && self.status.is_open()
    }

    /// Days the task has been waiting on others, if it is waiting.
    pub fn days_waiting(&self, today: NaiveDate) -> Option<i64> {
        match self.status {
            TaskStatus::Waiting => self.waiting_since.map(|since| (today - since).num_days()),
            _ => None,
        }
    }
}
