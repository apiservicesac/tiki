use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A meeting copied from a calendar, in the user's local time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meeting {
    /// The provider's id of the event, unique within its account.
    pub external_id: String,
    pub title: String,
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
    pub all_day: bool,
    pub location: String,
    /// The video call link (Teams, Meet, Zoom), when there is one.
    pub join_url: Option<String>,
}
