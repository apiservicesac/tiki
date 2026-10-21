use serde::Serialize;

use crate::domain::ClientId;

/// One client as the client lists show it: with how much work is open for them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSummary {
    pub client_id: ClientId,
    pub name: String,
    pub color: String,
    /// Tasks not delivered yet.
    pub open_count: usize,
    /// Open tasks that are overdue or due soon.
    pub pressing_count: usize,
}
