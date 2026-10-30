use chrono::NaiveDateTime;
use serde::Serialize;

use super::{RecordingMode, Session, SessionId, SessionStatus};
use crate::error::{TikiError, TikiResult};

/// The session being recorded, as the notch shows it: a red dot and its timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRecording {
    pub session_id: SessionId,
    pub mode: RecordingMode,
    pub paused: bool,
    /// Recorded so far (pauses excluded); the interface keeps counting from here.
    pub recorded_ms: i64,
}

impl ActiveRecording {
    pub fn of(session: &Session, now: NaiveDateTime) -> TikiResult<Self> {
        Ok(Self {
            session_id: session.id.ok_or(TikiError::UnsavedSession)?,
            mode: session.mode,
            paused: session.status == SessionStatus::Paused,
            recorded_ms: session.recorded(now).num_milliseconds(),
        })
    }
}
