use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::ActionItemId;
use crate::domain::TaskId;

/// What the AI wrote about a session, once its transcript was ready.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionNotes {
    pub summary: String,
    pub key_points: Vec<String>,
    pub decisions: Vec<String>,
}

/// A task the AI proposes from the conversation; it becomes a task only with the user's click.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedSessionTask {
    pub title: String,
    pub due_date: Option<NaiveDate>,
}

/// A proposed task as it is kept: once applied, it points to the task it became.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionActionItem {
    pub action_item_id: ActionItemId,
    pub title: String,
    pub due_date: Option<NaiveDate>,
    pub task_id: Option<TaskId>,
}
