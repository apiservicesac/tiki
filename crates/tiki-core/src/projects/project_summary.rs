use serde::Serialize;

use crate::domain::ProjectId;

/// One project as the project lists show it: with how much work is open for them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub project_id: ProjectId,
    pub name: String,
    pub color: String,
    /// Tasks not delivered yet.
    pub open_count: usize,
    /// Open tasks that are overdue or due soon.
    pub pressing_count: usize,
}
