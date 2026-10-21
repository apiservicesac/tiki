use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::domain::ExternalReference;

/// A ticket or project task as a ticket source reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalTicket {
    pub reference: ExternalReference,
    pub title: String,
    /// Stage name in the source system, for example "In progress".
    pub stage: String,
    pub deadline: Option<NaiveDate>,
    /// Client name as the source knows it; used to link the ticket with its instance.
    pub client_hint: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
