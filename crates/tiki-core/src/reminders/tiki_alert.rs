use serde::Serialize;

use crate::briefing::BriefMoment;
use crate::domain::TaskId;

/// What an alert is about; the island draws each kind its own way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertKind {
    Morning,
    Evening,
    /// The reminder the user set on one task.
    Task,
}

impl From<BriefMoment> for AlertKind {
    fn from(moment: BriefMoment) -> Self {
        match moment {
            BriefMoment::Morning => AlertKind::Morning,
            BriefMoment::Evening => AlertKind::Evening,
        }
    }
}

/// An alert ready to show in the notch and, if the user wants, in Windows: already translated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TikiAlert {
    pub kind: AlertKind,
    pub title: String,
    /// Lines separated by line breaks.
    pub body: String,
    /// The task a task reminder is about.
    pub task_id: Option<TaskId>,
}
