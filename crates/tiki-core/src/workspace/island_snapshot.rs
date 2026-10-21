use serde::Serialize;

use crate::domain::TaskId;
use crate::mood::Mood;

/// How pressing a line is; the interface turns it into a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Critical,
    Soon,
    Waiting,
    Normal,
}

/// One task line, already translated and ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub task_id: TaskId,
    pub title: String,
    pub client_name: Option<String>,
    pub client_color: Option<String>,
    pub detail_label: String,
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

/// Everything the island needs to draw itself, in the user's language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandSnapshot {
    pub mood: Mood,
    /// Word on the mascot's ticket tag: "200 OK", "¡HOY!"…
    pub mood_label: String,
    /// Tasks that need attention: overdue, due soon or waiting on others.
    pub attention_count: usize,
    /// Text of the closed island: "3 hoy · 2 esperando".
    pub closed_summary: String,
    /// Subtitle of the Today tab.
    pub today_subtitle: String,
    pub sections: Vec<SnapshotSection>,
    /// The brief as Tiki says it out loud.
    pub narration: String,
}
