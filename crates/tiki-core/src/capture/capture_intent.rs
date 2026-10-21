use serde::Serialize;

use super::CaptureDraft;
use crate::domain::{ClientId, TaskId};

/// What the user meant with a quick-capture sentence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureIntent {
    /// A task to save, as understood from the sentence.
    Task(CaptureDraft),
    /// A new client, written as "+Name".
    Client { name: String },
}

/// What a confirmed capture created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum CaptureOutcome {
    Task(TaskId),
    Client(ClientId),
}
