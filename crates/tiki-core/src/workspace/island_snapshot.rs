use serde::Serialize;

use chrono::NaiveDateTime;

use crate::agenda::AgendaDay;
use crate::domain::{ItemKind, Priority, ProjectId, Repeat, TaskId};
use crate::mood::Mood;

/// How pressing a line is; the interface turns it into a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Critical,
    Soon,
    Waiting,
    Normal,
    /// A standalone reminder.
    Reminder,
}

/// One item line, already translated and ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub task_id: TaskId,
    /// A task, or a standalone reminder: the Tasks tab lists only tasks, the Agenda both.
    pub kind: ItemKind,
    pub title: String,
    pub project_id: Option<ProjectId>,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    pub detail_label: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    /// When it rings (a reminder's moment, or the alarm on a task), and how often a
    /// reminder comes back.
    pub at: Option<NaiveDateTime>,
    pub repeat: Repeat,
    pub urgency: Urgency,
}

/// A titled group of lines: "Vence pronto", "Te toca"…
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSection {
    /// Stable key: `overdue`, `due_soon`, `your_turn` or `waiting`.
    pub key: String,
    pub title: String,
    pub items: Vec<SnapshotItem>,
}

/// The one item Tiki suggests doing now, and the reason in the user's language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusSuggestion {
    pub item: SnapshotItem,
    pub reason: String,
}

/// Everything the island needs to draw itself, in the user's language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandSnapshot {
    pub mood: Mood,
    /// Word on the mascot's ticket tag: "200 OK", "¡HOY!"…
    pub mood_label: String,
    /// Tasks that need attention: overdue, due soon or waiting on others.
    pub attention_count: usize,
    /// Overdue or due soon.
    pub due_count: usize,
    /// Waiting on others.
    pub waiting_count: usize,
    /// Open work: every open task, without standalone reminders.
    pub open_count: usize,
    /// Delivered today.
    pub delivered_today: usize,
    /// Text of the closed island: "3 hoy · 2 esperando".
    pub closed_summary: String,
    /// Subtitle of the Today tab.
    pub today_subtitle: String,
    /// Today only: overdue, due soon, today's reminders and follow-ups to send.
    pub sections: Vec<SnapshotSection>,
    /// Every open item, for the Tasks tab.
    pub open_items: Vec<SnapshotItem>,
    /// What to do now, with why; `None` when nothing presses.
    pub focus: Option<FocusSuggestion>,
    /// Meetings from today on and the items that ring at a time, by day.
    pub agenda: Vec<AgendaDay>,
    /// The brief as Tiki says it out loud.
    pub narration: String,
}
