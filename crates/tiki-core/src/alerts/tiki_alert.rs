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
    /// A standalone reminder: the user can mark it done or snooze it.
    Reminder,
    /// A waiting item whose follow-up day came.
    FollowUp,
    /// A meeting starting soon.
    Meeting,
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
    /// A link to open from the alert: the meeting's video call.
    pub link: Option<String>,
}
