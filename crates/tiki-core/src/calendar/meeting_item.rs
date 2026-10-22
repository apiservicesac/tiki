use chrono::NaiveDateTime;
use serde::Serialize;

/// A meeting as the Agenda draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingItem {
    pub title: String,
    pub starts_at: NaiveDateTime,
    pub ends_at: NaiveDateTime,
    pub all_day: bool,
    pub location: String,
    pub join_url: Option<String>,
    /// The account it comes from, for people with several calendars.
    pub account_email: String,
}
