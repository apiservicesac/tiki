use chrono::NaiveDateTime;
use serde::Serialize;

use super::{
    RecordingMode, Session, SessionActionItem, SessionId, SessionNotes, SessionStatus, Transcript,
};
use crate::error::{TikiError, TikiResult};
use crate::i18n::{DateFormatter, Translator};

/// A session as the Sessions tab lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub title: String,
    pub mode: RecordingMode,
    pub status: SessionStatus,
    pub started_at: NaiveDateTime,
    /// "Mié 29 · 10:30", in the user's language.
    pub started_label: String,
    /// What the media lasts (pauses excluded); while recording, what it lasts so far.
    pub recorded_ms: i64,
    /// The media is still on disk (the retention policy has not removed it).
    pub has_media: bool,
    pub media_bytes: u64,
    pub meeting_title: Option<String>,
}

impl SessionSummary {
    pub fn of(session: &Session, now: NaiveDateTime, translator: &Translator) -> TikiResult<Self> {
        let started_at = session.started_at();
        Ok(Self {
            session_id: session.id.ok_or(TikiError::UnsavedSession)?,
            title: session.title.clone(),
            mode: session.mode,
            status: session.status,
            started_at,
            started_label: translator.format(
                "session.started",
                &[
                    (
                        "date",
                        &DateFormatter::new(translator).short(started_at.date()),
                    ),
                    ("time", &DateFormatter::time(started_at.time())),
                ],
            ),
            recorded_ms: session.recorded(now).num_milliseconds(),
            has_media: session.has_media(),
            media_bytes: session.media_bytes,
            meeting_title: session
                .meeting
                .as_ref()
                .map(|meeting| meeting.title.clone()),
        })
    }
}

/// Everything the detail of a session shows: the transcript, the notes and the tasks they
/// propose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
    pub summary: SessionSummary,
    /// The detected language ("es", "en"), once transcribed.
    pub language: Option<String>,
    pub transcript: Transcript,
    pub notes: Option<SessionNotes>,
    pub action_items: Vec<SessionActionItem>,
}
