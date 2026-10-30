use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// The calendar meeting a session recorded, copied when it starts: the meeting itself
/// leaves the local calendar copy once it is past.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingLink {
    pub title: String,
    pub starts_at: NaiveDateTime,
}
