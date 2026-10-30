use chrono::NaiveDateTime;
use serde::Serialize;

use super::{SessionId, Speaker};

/// A transcript line that matches a search, with what the list needs to open it there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSearchHit {
    pub session_id: SessionId,
    pub session_title: String,
    pub started_at: NaiveDateTime,
    pub speaker: Speaker,
    pub start_ms: i64,
    pub text: String,
}
