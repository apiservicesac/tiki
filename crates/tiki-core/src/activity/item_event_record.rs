use chrono::NaiveDateTime;
use serde::Serialize;

use super::ItemEvent;
use crate::domain::TaskId;

/// An event as the history keeps it: which item, and when (local time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEventRecord {
    pub item_id: TaskId,
    pub at: NaiveDateTime,
    pub event: ItemEvent,
}

/// One delivery of an item, for the history and for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryRecord {
    pub item_id: TaskId,
    pub at: NaiveDateTime,
}
