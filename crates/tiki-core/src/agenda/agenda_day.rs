use chrono::NaiveDate;
use serde::Serialize;

use super::AgendaEntry;

/// The Agenda's lines of one day, earliest first (all-day meetings lead), under its title:
/// "Hoy", "Mañana" or the long date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaDay {
    pub day: NaiveDate,
    pub label: String,
    pub entries: Vec<AgendaEntry>,
}
