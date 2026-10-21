use chrono::NaiveDate;
use serde::Deserialize;

use crate::domain::Priority;

/// A task the user confirmed after Tiki read their sentence (with the assistant or the rules).
/// The client comes by name, so the interface never has to know database ids.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedCapture {
    pub title: String,
    pub client_name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub priority: Priority,
}
