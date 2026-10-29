use chrono::NaiveDateTime;
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::domain::TaskId;
use crate::error::{TikiError, TikiResult};
use crate::sessions::{
    ActionItemId, MeetingLink, ProposedSessionTask, RecordingMode, Session, SessionActionItem,
    SessionClock, SessionId, SessionNotes, SessionSearchHit, SessionStatus, Speaker, Transcript,
    TranscriptSegment,
};

/// Reads and writes recorded sessions, their transcripts, notes and transcription queue.
pub struct SessionRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> SessionRepository<'connection> {
    /// Columns in the order `read_session` expects them.
    const COLUMNS: &'static str = "id, title, mode, status, started_at, paused_ms, paused_since, \
        ended_at, media_file, media_bytes, media_deleted_at, meeting_title, meeting_starts_at, \
        language, failure";

    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn insert(&self, session: &Session) -> TikiResult<SessionId> {
        self.connection.execute(
            "INSERT INTO sessions (title, mode, status, started_at, paused_ms, paused_since,
                                   ended_at, media_file, media_bytes, media_deleted_at,
                                   meeting_title, meeting_starts_at, language, failure)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                session.title,
                session.mode.key(),
                session.status.key(),
                session.clock.started_at,
                session.clock.paused_ms,
                session.clock.paused_since,
                session.ended_at,
                session.media_file,
                Self::stored_bytes(session.media_bytes),
                session.media_deleted_at,
                session
                    .meeting
                    .as_ref()
                    .map(|meeting| meeting.title.as_str()),
                session.meeting.as_ref().map(|meeting| meeting.starts_at),
                session.language,
                session.failure,
            ],
        )?;
        Ok(SessionId(self.connection.last_insert_rowid()))
    }

    pub fn update(&self, session: &Session) -> TikiResult<()> {
        let session_id = session.id.ok_or(TikiError::UnsavedSession)?;
        let changed_rows = self.connection.execute(
            "UPDATE sessions
                SET title = ?2, mode = ?3, status = ?4, paused_ms = ?5, paused_since = ?6,
                    ended_at = ?7, media_file = ?8, media_bytes = ?9, media_deleted_at = ?10,
                    language = ?11, failure = ?12
              WHERE id = ?1",
            params![
                session_id.0,
                session.title,
                session.mode.key(),
                session.status.key(),
                session.clock.paused_ms,
                session.clock.paused_since,
                session.ended_at,
                session.media_file,
                Self::stored_bytes(session.media_bytes),
                session.media_deleted_at,
                session.language,
                session.failure,
            ],
        )?;
        if changed_rows == 0 {
            return Err(Self::not_found(session_id));
        }
        Ok(())
    }

    pub fn find(&self, session_id: SessionId) -> TikiResult<Session> {
        let query = format!("SELECT {} FROM sessions WHERE id = ?1", Self::COLUMNS);
        self.connection
            .query_row(&query, params![session_id.0], Self::read_session)
            .optional()?
            .ok_or(Self::not_found(session_id))
    }

    /// The session recording or paused right now, if any.
    pub fn capturing(&self) -> TikiResult<Option<Session>> {
        let query = format!(
            "SELECT {} FROM sessions WHERE status IN (?1, ?2) ORDER BY started_at DESC LIMIT 1",
            Self::COLUMNS
        );
        Ok(self
            .connection
            .query_row(
                &query,
                params![SessionStatus::Recording.key(), SessionStatus::Paused.key()],
                Self::read_session,
            )
            .optional()?)
    }

    /// Every session, newest first.
    pub fn list(&self) -> TikiResult<Vec<Session>> {
        let query = format!(
            "SELECT {} FROM sessions ORDER BY started_at DESC, id DESC",
            Self::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let sessions = statement
            .query_map([], Self::read_session)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    /// Removes a session with its transcript, notes and queue entry; tasks made from it stay.
    pub fn delete(&self, session_id: SessionId) -> TikiResult<()> {
        self.connection
            .execute("DELETE FROM sessions WHERE id = ?1", params![session_id.0])?;
        Ok(())
    }

    /// Sessions that started before `cutoff` and still keep their media.
    pub fn with_media_before(&self, cutoff: NaiveDateTime) -> TikiResult<Vec<Session>> {
        let query = format!(
            "SELECT {} FROM sessions
              WHERE media_file IS NOT NULL AND media_deleted_at IS NULL AND started_at < ?1
                AND status NOT IN (?2, ?3)
              ORDER BY started_at",
            Self::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let sessions = statement
            .query_map(
                params![
                    cutoff,
                    SessionStatus::Recording.key(),
                    SessionStatus::Paused.key()
                ],
                Self::read_session,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    /// Bytes the kept media takes on disk.
    pub fn media_bytes(&self) -> TikiResult<u64> {
        let bytes: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(media_bytes), 0) FROM sessions WHERE media_deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(u64::try_from(bytes).unwrap_or_default())
    }

    /// Puts a session at the end of the transcription queue.
    pub fn enqueue_transcription(
        &self,
        session_id: SessionId,
        at: NaiveDateTime,
    ) -> TikiResult<()> {
        self.connection.execute(
            "INSERT INTO transcription_jobs (session_id, queued_at) VALUES (?1, ?2)
             ON CONFLICT(session_id) DO UPDATE SET queued_at = excluded.queued_at",
            params![session_id.0, at],
        )?;
        Ok(())
    }

    /// The oldest job in the queue and how many times it was tried.
    pub fn next_transcription(&self) -> TikiResult<Option<(SessionId, u32)>> {
        Ok(self
            .connection
            .query_row(
                "SELECT session_id, attempts FROM transcription_jobs
                  ORDER BY queued_at, session_id LIMIT 1",
                [],
                |row| Ok((SessionId(row.get(0)?), row.get(1)?)),
            )
            .optional()?)
    }

    /// How many times the queued transcription of a session failed; 0 when it is not queued.
    pub fn transcription_attempts(&self, session_id: SessionId) -> TikiResult<u32> {
        Ok(self
            .connection
            .query_row(
                "SELECT attempts FROM transcription_jobs WHERE session_id = ?1",
                params![session_id.0],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or_default())
    }

    pub fn count_transcription_attempt(&self, session_id: SessionId) -> TikiResult<()> {
        self.connection.execute(
            "UPDATE transcription_jobs SET attempts = attempts + 1 WHERE session_id = ?1",
            params![session_id.0],
        )?;
        Ok(())
    }

    pub fn remove_transcription(&self, session_id: SessionId) -> TikiResult<()> {
        self.connection.execute(
            "DELETE FROM transcription_jobs WHERE session_id = ?1",
            params![session_id.0],
        )?;
        Ok(())
    }

    /// Replaces the session's transcript.
    pub fn replace_transcript(
        &self,
        session_id: SessionId,
        transcript: &Transcript,
    ) -> TikiResult<()> {
        self.connection.execute(
            "DELETE FROM transcript_segments WHERE session_id = ?1",
            params![session_id.0],
        )?;
        let mut statement = self.connection.prepare(
            "INSERT INTO transcript_segments (session_id, speaker, start_ms, end_ms, text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for segment in transcript.segments() {
            statement.execute(params![
                session_id.0,
                segment.speaker.key(),
                segment.start_ms,
                segment.end_ms,
                segment.text
            ])?;
        }
        Ok(())
    }

    pub fn transcript(&self, session_id: SessionId) -> TikiResult<Transcript> {
        let mut statement = self.connection.prepare(
            "SELECT speaker, start_ms, end_ms, text FROM transcript_segments
              WHERE session_id = ?1 ORDER BY start_ms, id",
        )?;
        let segments = statement
            .query_map(params![session_id.0], |row| {
                Ok(TranscriptSegment {
                    speaker: Self::read_key(row, 0, Speaker::from_key)?,
                    start_ms: row.get(1)?,
                    end_ms: row.get(2)?,
                    text: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Transcript::merge([segments]))
    }

    /// Transcript lines matching an FTS5 query (see `TranscriptQuery`), newest session first.
    pub fn search(&self, query: &str, limit: usize) -> TikiResult<Vec<SessionSearchHit>> {
        let mut statement = self.connection.prepare(
            "SELECT s.id, s.title, s.started_at, t.speaker, t.start_ms, t.text
               FROM transcript_search f
               JOIN transcript_segments t ON t.id = f.rowid
               JOIN sessions s ON s.id = t.session_id
              WHERE transcript_search MATCH ?1
              ORDER BY s.started_at DESC, t.start_ms LIMIT ?2",
        )?;
        let hits = statement
            .query_map(
                params![query, i64::try_from(limit).unwrap_or(i64::MAX)],
                |row| {
                    Ok(SessionSearchHit {
                        session_id: SessionId(row.get(0)?),
                        session_title: row.get(1)?,
                        started_at: row.get(2)?,
                        speaker: Self::read_key(row, 3, Speaker::from_key)?,
                        start_ms: row.get(4)?,
                        text: row.get(5)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(hits)
    }

    /// Replaces the session's notes and the tasks they propose, written at `at`.
    pub fn replace_notes(
        &self,
        session_id: SessionId,
        notes: &SessionNotes,
        proposed: &[ProposedSessionTask],
        at: NaiveDateTime,
    ) -> TikiResult<()> {
        self.connection.execute(
            "INSERT INTO session_notes (session_id, summary, key_points, decisions, written_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_id) DO UPDATE SET summary = excluded.summary,
                key_points = excluded.key_points, decisions = excluded.decisions,
                written_at = excluded.written_at",
            params![
                session_id.0,
                notes.summary,
                serde_json::to_string(&notes.key_points)?,
                serde_json::to_string(&notes.decisions)?,
                at
            ],
        )?;
        // Proposals already applied stay: they point to a task the user has.
        self.connection.execute(
            "DELETE FROM session_action_items WHERE session_id = ?1 AND task_id IS NULL",
            params![session_id.0],
        )?;
        let mut statement = self.connection.prepare(
            "INSERT INTO session_action_items (session_id, title, due_date) VALUES (?1, ?2, ?3)",
        )?;
        for task in proposed {
            statement.execute(params![session_id.0, task.title, task.due_date])?;
        }
        Ok(())
    }

    pub fn notes(&self, session_id: SessionId) -> TikiResult<Option<SessionNotes>> {
        let stored: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT summary, key_points, decisions FROM session_notes WHERE session_id = ?1",
                params![session_id.0],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        stored
            .map(|(summary, key_points, decisions)| {
                Ok(SessionNotes {
                    summary,
                    key_points: serde_json::from_str(&key_points)?,
                    decisions: serde_json::from_str(&decisions)?,
                })
            })
            .transpose()
    }

    pub fn action_items(&self, session_id: SessionId) -> TikiResult<Vec<SessionActionItem>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, due_date, task_id FROM session_action_items
              WHERE session_id = ?1 ORDER BY id",
        )?;
        let items = statement
            .query_map(params![session_id.0], Self::read_action_item)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// A proposed task and the session it came from.
    pub fn find_action_item(
        &self,
        action_item_id: ActionItemId,
    ) -> TikiResult<(SessionId, SessionActionItem)> {
        self.connection
            .query_row(
                "SELECT id, title, due_date, task_id, session_id FROM session_action_items
                  WHERE id = ?1",
                params![action_item_id.0],
                |row| Ok((SessionId(row.get(4)?), Self::read_action_item(row)?)),
            )
            .optional()?
            .ok_or(TikiError::NotFound {
                entity: "session action item",
                id: action_item_id.0,
            })
    }

    /// Remembers the task a proposal became.
    pub fn link_action_item(
        &self,
        action_item_id: ActionItemId,
        task_id: TaskId,
    ) -> TikiResult<()> {
        self.connection.execute(
            "UPDATE session_action_items SET task_id = ?2 WHERE id = ?1",
            params![action_item_id.0, task_id.0],
        )?;
        Ok(())
    }

    fn read_session(row: &Row<'_>) -> rusqlite::Result<Session> {
        let meeting_title: Option<String> = row.get("meeting_title")?;
        let meeting_starts_at: Option<NaiveDateTime> = row.get("meeting_starts_at")?;
        Ok(Session {
            id: Some(SessionId(row.get("id")?)),
            title: row.get("title")?,
            mode: Self::read_key(row, 2, RecordingMode::from_key)?,
            status: Self::read_key(row, 3, SessionStatus::from_key)?,
            clock: SessionClock {
                started_at: row.get("started_at")?,
                paused_ms: row.get("paused_ms")?,
                paused_since: row.get("paused_since")?,
            },
            ended_at: row.get("ended_at")?,
            media_file: row.get("media_file")?,
            media_bytes: u64::try_from(row.get::<_, i64>("media_bytes")?).unwrap_or_default(),
            media_deleted_at: row.get("media_deleted_at")?,
            meeting: meeting_title
                .zip(meeting_starts_at)
                .map(|(title, starts_at)| MeetingLink { title, starts_at }),
            language: row.get("language")?,
            failure: row.get("failure")?,
        })
    }

    fn read_action_item(row: &Row<'_>) -> rusqlite::Result<SessionActionItem> {
        Ok(SessionActionItem {
            action_item_id: ActionItemId(row.get(0)?),
            title: row.get(1)?,
            due_date: row.get(2)?,
            task_id: row.get::<_, Option<i64>>(3)?.map(TaskId),
        })
    }

    /// Reads a stable key in column `index` back into its value.
    fn read_key<Value>(
        row: &Row<'_>,
        index: usize,
        from_key: impl Fn(&str) -> TikiResult<Value>,
    ) -> rusqlite::Result<Value> {
        let key: String = row.get(index)?;
        from_key(&key).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
        })
    }

    fn stored_bytes(bytes: u64) -> i64 {
        i64::try_from(bytes).unwrap_or(i64::MAX)
    }

    fn not_found(session_id: SessionId) -> TikiError {
        TikiError::NotFound {
            entity: "session",
            id: session_id.0,
        }
    }
}
