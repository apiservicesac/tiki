use chrono::{Duration, NaiveDateTime};
use serde::Serialize;

use super::{MeetingLink, RecordingMode, SessionClock, SessionId, SessionStatus};
use crate::error::{TikiError, TikiResult};

/// A recorded work session: a call or a meeting the user chose to keep.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: Option<SessionId>,
    pub title: String,
    pub mode: RecordingMode,
    pub status: SessionStatus,
    pub clock: SessionClock,
    pub ended_at: Option<NaiveDateTime>,
    /// The file played back, inside the session's folder; empty until it stops.
    pub media_file: Option<String>,
    pub media_bytes: u64,
    /// When the retention policy (or the user) removed the media; the transcript stays.
    pub media_deleted_at: Option<NaiveDateTime>,
    pub meeting: Option<MeetingLink>,
    /// Language the transcription detected ("es", "en").
    pub language: Option<String>,
    /// Why the transcription failed, for the logs and a retry.
    pub failure: Option<String>,
}

impl Session {
    /// A session that starts recording at `now`.
    pub fn start(
        title: impl Into<String>,
        mode: RecordingMode,
        meeting: Option<MeetingLink>,
        now: NaiveDateTime,
    ) -> Self {
        Self {
            id: None,
            title: title.into(),
            mode,
            status: SessionStatus::Recording,
            clock: SessionClock::start(now),
            ended_at: None,
            media_file: None,
            media_bytes: 0,
            media_deleted_at: None,
            meeting,
            language: None,
            failure: None,
        }
    }

    pub fn started_at(&self) -> NaiveDateTime {
        self.clock.started_at
    }

    /// The name of the session's folder: its start day and id ("2026-10-29-12").
    pub fn folder_name(&self) -> Option<String> {
        self.id.map(|SessionId(id)| {
            format!("{}-{id}", self.clock.started_at.date().format("%Y-%m-%d"))
        })
    }

    /// What the media lasts; while recording, what it lasts so far.
    pub fn recorded(&self, now: NaiveDateTime) -> Duration {
        self.clock.recorded(self.ended_at.unwrap_or(now))
    }

    /// Only audio-only sessions pause: a video keeps its frames' own clock.
    pub fn pause(&mut self, now: NaiveDateTime) -> TikiResult<()> {
        if self.mode != RecordingMode::AudioOnly {
            return Err(TikiError::PauseNeedsAudioOnly);
        }
        self.status = self.status.checked_move(SessionStatus::Paused)?;
        self.clock.pause(now);
        Ok(())
    }

    pub fn resume(&mut self, now: NaiveDateTime) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Recording)?;
        self.clock.resume(now);
        Ok(())
    }

    /// Stops recording: the media is saved and the session waits for its transcript.
    pub fn stop(
        &mut self,
        now: NaiveDateTime,
        media_file: String,
        media_bytes: u64,
    ) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Queued)?;
        self.clock.resume(now);
        self.ended_at = Some(now);
        self.media_file = Some(media_file);
        self.media_bytes = media_bytes;
        Ok(())
    }

    /// Tiki closed while recording: the session ends where its tracks end (`recorded`,
    /// pauses excluded), with the media that could be saved, and waits for its transcript.
    pub fn recover(&mut self, recorded: Duration, media: Option<(String, u64)>) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Queued)?;
        // The tracks hold no paused time, so a pause in progress just ends with them.
        self.clock.paused_since = None;
        self.ended_at =
            Some(self.clock.started_at + Duration::milliseconds(self.clock.paused_ms) + recorded);
        (self.media_file, self.media_bytes) =
            media.map_or((None, 0), |(file, bytes)| (Some(file), bytes));
        Ok(())
    }

    pub fn begin_transcription(&mut self) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Transcribing)?;
        self.failure = None;
        Ok(())
    }

    pub fn finish_transcription(&mut self, language: Option<String>) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Ready)?;
        self.language = language;
        Ok(())
    }

    pub fn fail_transcription(&mut self, reason: String) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Failed)?;
        self.failure = Some(reason);
        Ok(())
    }

    /// Back to the queue: a failed transcription the user retries, or one cut short when
    /// Tiki closed.
    pub fn requeue(&mut self) -> TikiResult<()> {
        self.status = self.status.checked_move(SessionStatus::Queued)?;
        Ok(())
    }

    pub fn has_media(&self) -> bool {
        self.media_file.is_some() && self.media_deleted_at.is_none()
    }
}
