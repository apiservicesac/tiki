use serde::Serialize;

use crate::calendar::MeetingItem;
use crate::workspace::SnapshotItem;

/// One line of the Agenda, with the time it shows: "10:00", "10:00–10:30" or "Todo el día".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AgendaEntry {
    /// A meeting from a connected calendar.
    Meeting {
        time_label: String,
        meeting: MeetingItem,
    },
    /// A standalone reminder, at the moment it rings.
    Reminder {
        time_label: String,
        item: SnapshotItem,
    },
    /// A task, at the moment its alarm rings.
    Task {
        time_label: String,
        item: SnapshotItem,
    },
}
