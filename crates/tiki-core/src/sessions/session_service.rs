use chrono::NaiveDateTime;

use super::{
    ActionItemId, MeetingLink, ProposedSessionTask, RecordingMode, RetentionPolicy, Session,
    SessionDetails, SessionId, SessionNotes, SessionSearchHit, SessionStatus, SessionSummary,
    Transcript, TranscriptQuery,
};
use crate::domain::{Task, TaskId, TaskOrigin};
use crate::error::{TikiError, TikiResult};
use crate::i18n::{DateFormatter, Translator};
use crate::items::ItemService;
use crate::settings::TikiSettings;
use crate::storage::Database;
use crate::workspace::Clock;

/// The rules of recorded sessions: one recording at a time, checked status moves, the
/// transcription queue, notes and the tasks they propose, search and media retention.
pub struct SessionService<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
    translator: &'context Translator,
    clock: &'context dyn Clock,
}

impl<'context> SessionService<'context> {
    pub fn new(
        database: &'context Database,
        settings: &'context TikiSettings,
        translator: &'context Translator,
        clock: &'context dyn Clock,
    ) -> Self {
        Self {
            database,
            settings,
            translator,
            clock,
        }
    }

    /// Starts a session the user asked for; the meeting in progress, if any, gives it its
    /// title. Only one session records at a time.
    pub fn start(&self, mode: RecordingMode) -> TikiResult<Session> {
        if self.database.sessions().capturing()?.is_some() {
            return Err(TikiError::AlreadyRecording);
        }
        let now = self.clock.now();
        let meeting = self
            .database
            .calendar()
            .in_progress(now)?
            .into_iter()
            .next()
            .map(|meeting| MeetingLink {
                title: meeting.title,
                starts_at: meeting.starts_at,
            });
        let title = meeting
            .as_ref()
            .map_or_else(|| self.default_title(now), |meeting| meeting.title.clone());
        let mut session = Session::start(title, mode, meeting, now);
        session.id = Some(self.database.sessions().insert(&session)?);
        Ok(session)
    }

    /// The session recording or paused right now.
    pub fn capturing(&self) -> TikiResult<Option<Session>> {
        self.database.sessions().capturing()
    }

    pub fn find(&self, session_id: SessionId) -> TikiResult<Session> {
        self.database.sessions().find(session_id)
    }

    pub fn pause(&self, session_id: SessionId) -> TikiResult<Session> {
        self.change(session_id, Session::pause)
    }

    pub fn resume(&self, session_id: SessionId) -> TikiResult<Session> {
        self.change(session_id, Session::resume)
    }

    /// Stops recording with the media the recorder saved, and queues the transcription.
    pub fn stop(
        &self,
        session_id: SessionId,
        media_file: String,
        media_bytes: u64,
    ) -> TikiResult<Session> {
        let mut session = self.database.sessions().find(session_id)?;
        let now = self.clock.now();
        session.stop(now, media_file, media_bytes)?;
        self.database.transaction(|repositories| {
            repositories.sessions().update(&session)?;
            repositories
                .sessions()
                .enqueue_transcription(session_id, now)
        })?;
        Ok(session)
    }

    /// Whether a session records or waits for its transcript: the transcription model must
    /// be ready.
    pub fn awaits_transcription(&self) -> TikiResult<bool> {
        let sessions = self.database.sessions();
        Ok(sessions.capturing()?.is_some() || sessions.next_transcription()?.is_some())
    }

    /// Takes the oldest session in the transcription queue and marks it as transcribing.
    pub fn begin_next_transcription(&self) -> TikiResult<Option<Session>> {
        let Some((session_id, _attempts)) = self.database.sessions().next_transcription()? else {
            return Ok(None);
        };
        let session = self.change(session_id, |session, _| session.begin_transcription())?;
        Ok(Some(session))
    }

    /// Saves the transcript of a session being transcribed: it is ready, and leaves the queue.
    pub fn save_transcript(
        &self,
        session_id: SessionId,
        transcript: &Transcript,
        language: Option<String>,
    ) -> TikiResult<Session> {
        let mut session = self.database.sessions().find(session_id)?;
        session.finish_transcription(language)?;
        self.database.transaction(|repositories| {
            let sessions = repositories.sessions();
            sessions.replace_transcript(session_id, transcript)?;
            sessions.update(&session)?;
            sessions.remove_transcription(session_id)
        })?;
        Ok(session)
    }

    /// A transcription that failed goes back to the end of the queue, until it failed
    /// `transcription_attempts` times: then the session shows the failure and waits for the
    /// user to retry.
    pub fn fail_transcription(&self, session_id: SessionId, reason: String) -> TikiResult<Session> {
        let attempts = self
            .database
            .sessions()
            .transcription_attempts(session_id)?
            + 1;
        let mut session = self.database.sessions().find(session_id)?;
        let now = self.clock.now();
        let gives_up = attempts >= self.settings.transcription_attempts;
        if gives_up {
            session.fail_transcription(reason)?;
        } else {
            session.requeue()?;
        }
        self.database.transaction(|repositories| {
            let sessions = repositories.sessions();
            sessions.update(&session)?;
            if gives_up {
                sessions.remove_transcription(session_id)
            } else {
                sessions.count_transcription_attempt(session_id)?;
                sessions.enqueue_transcription(session_id, now)
            }
        })?;
        Ok(session)
    }

    /// The user retries a failed transcription.
    pub fn retry_transcription(&self, session_id: SessionId) -> TikiResult<Session> {
        let mut session = self.database.sessions().find(session_id)?;
        session.requeue()?;
        let now = self.clock.now();
        self.database.transaction(|repositories| {
            repositories.sessions().update(&session)?;
            repositories
                .sessions()
                .enqueue_transcription(session_id, now)
        })?;
        Ok(session)
    }

    /// Tiki closed in the middle of a transcription: it goes back to the queue.
    pub fn requeue_interrupted_transcriptions(&self) -> TikiResult<()> {
        for mut session in self.database.sessions().list()? {
            if session.status == SessionStatus::Transcribing {
                session.requeue()?;
                self.database.sessions().update(&session)?;
            }
        }
        Ok(())
    }

    /// Saves the AI notes of a session and the tasks they propose.
    pub fn save_notes(
        &self,
        session_id: SessionId,
        notes: &SessionNotes,
        proposed: &[ProposedSessionTask],
    ) -> TikiResult<()> {
        let now = self.clock.now();
        self.database.transaction(|repositories| {
            repositories
                .sessions()
                .replace_notes(session_id, notes, proposed, now)
        })
    }

    /// Turns a proposed task into a task of the user, pointing back to its session. Applying
    /// it again returns the same task.
    pub fn apply_action_item(&self, action_item_id: ActionItemId) -> TikiResult<TaskId> {
        let (session_id, item) = self.database.sessions().find_action_item(action_item_id)?;
        if let Some(task_id) = item.task_id {
            return Ok(task_id);
        }
        let mut task = Task::new(item.title)?.with_origin(TaskOrigin::Session { session_id });
        task.due_date = item.due_date;
        ItemService::new(self.database, self.settings, self.clock).create(
            task,
            |repositories, task_id| {
                repositories
                    .sessions()
                    .link_action_item(action_item_id, task_id)
            },
        )
    }

    /// Every session, newest first.
    pub fn list(&self) -> TikiResult<Vec<SessionSummary>> {
        let now = self.clock.now();
        self.database
            .sessions()
            .list()?
            .iter()
            .map(|session| SessionSummary::of(session, now, self.translator))
            .collect()
    }

    pub fn details(&self, session_id: SessionId) -> TikiResult<SessionDetails> {
        let sessions = self.database.sessions();
        let session = sessions.find(session_id)?;
        Ok(SessionDetails {
            summary: SessionSummary::of(&session, self.clock.now(), self.translator)?,
            language: session.language,
            transcript: sessions.transcript(session_id)?,
            notes: sessions.notes(session_id)?,
            action_items: sessions.action_items(session_id)?,
        })
    }

    /// Transcript lines with every word the user typed, newest session first.
    pub fn search(&self, text: &str) -> TikiResult<Vec<SessionSearchHit>> {
        match TranscriptQuery::from_text(text) {
            Some(query) => self
                .database
                .sessions()
                .search(&query, self.settings.session_search_limit),
            None => Ok(Vec::new()),
        }
    }

    /// Forgets a session whose recording could not start (no microphone, for example).
    pub fn abandon(&self, session_id: SessionId) -> TikiResult<()> {
        let session = self.database.sessions().find(session_id)?;
        if session.status != SessionStatus::Recording || session.media_file.is_some() {
            return Err(TikiError::SessionTransition {
                from: session.status.key(),
                to: SessionStatus::Recording.key(),
            });
        }
        self.database.sessions().delete(session_id)
    }

    /// Forgets a session; the caller removes its folder. Tasks made from it stay.
    pub fn delete(&self, session_id: SessionId) -> TikiResult<Session> {
        let session = self.database.sessions().find(session_id)?;
        if session.status.is_capturing() {
            return Err(TikiError::AlreadyRecording);
        }
        self.database.sessions().delete(session_id)?;
        Ok(session)
    }

    /// Sessions whose media the retention policy removes now.
    pub fn expired_media(&self, policy: RetentionPolicy) -> TikiResult<Vec<Session>> {
        self.database
            .sessions()
            .with_media_before(policy.media_cutoff(self.clock.now()))
    }

    /// The media of a session was removed from disk; its transcript and notes stay.
    pub fn media_removed(&self, session_id: SessionId) -> TikiResult<Session> {
        self.change(session_id, |session, now| {
            session.media_deleted_at = Some(now);
            Ok(())
        })
    }

    /// Bytes the kept media takes on disk.
    pub fn media_bytes(&self) -> TikiResult<u64> {
        self.database.sessions().media_bytes()
    }

    /// "Sesión del Mié 29, 10:30" / "Session on Wed 29, 10:30".
    fn default_title(&self, now: NaiveDateTime) -> String {
        self.translator.format(
            "session.default_title",
            &[
                (
                    "date",
                    &DateFormatter::new(self.translator).short(now.date()),
                ),
                ("time", &DateFormatter::time(now.time())),
            ],
        )
    }

    fn change(
        &self,
        session_id: SessionId,
        apply: impl FnOnce(&mut Session, NaiveDateTime) -> TikiResult<()>,
    ) -> TikiResult<Session> {
        let mut session = self.database.sessions().find(session_id)?;
        apply(&mut session, self.clock.now())?;
        self.database.sessions().update(&session)?;
        Ok(session)
    }
}
