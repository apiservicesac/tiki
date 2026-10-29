use serde::Serialize;

use super::AlertAction;
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
    /// A standalone reminder.
    Reminder,
    /// A waiting item whose follow-up day came.
    FollowUp,
    /// A meeting starting soon.
    Meeting,
    /// A call started or a meeting began: Tiki offers to record it.
    Recording,
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
    /// What it is about, in one line: the brief's counts, or the item's or meeting's title.
    pub summary: String,
    /// The lines after the summary: what to start with and whom to follow up in a brief;
    /// the project, who it waits for and when it is due for an item; a meeting's time and
    /// place.
    pub tips: Vec<String>,
    /// What the user can do with it, in the order to offer them (see `AlertAction`).
    pub actions: Vec<AlertAction>,
    /// The item an item alert is about.
    pub task_id: Option<TaskId>,
    /// A link to open from the alert: the meeting's video call.
    pub link: Option<String>,
}

impl TikiAlert {
    /// The text of a system notification: the summary, then each tip on its own line.
    pub fn notification_body(&self) -> String {
        std::iter::once(&self.summary)
            .chain(&self.tips)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
