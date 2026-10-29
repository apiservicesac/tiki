use tauri::{AppHandle, Manager};
use tiki_core::TikiError;
use tiki_core::sessions::Transcript;
use tiki_recording::RecordingResult;
use tiki_recording::transcription::{TranscriptionJob, TranscriptionQueue};

use crate::app_state::AppState;
use crate::session_files::SessionFiles;
use crate::shell_events::ShellEvent;

/// The transcription queue kept by the core (`transcription_jobs`), so it survives a
/// restart; each change tells the interface.
pub struct CoreTranscriptionQueue<'app> {
    app: &'app AppHandle,
}

impl<'app> CoreTranscriptionQueue<'app> {
    pub fn new(app: &'app AppHandle) -> Self {
        Self { app }
    }
}

impl TranscriptionQueue for CoreTranscriptionQueue<'_> {
    fn next(&self) -> RecordingResult<Option<TranscriptionJob>> {
        let session = self
            .app
            .state::<AppState>()
            .with_workspace(|workspace| workspace.sessions().begin_next_transcription())?;
        let Some(session) = session else {
            return Ok(None);
        };
        ShellEvent::RecordingChanged.emit(self.app);
        Ok(Some(TranscriptionJob {
            session_id: session.id.ok_or(TikiError::UnsavedSession)?,
            folder: self.app.state::<SessionFiles>().folder_of(&session)?,
        }))
    }

    fn complete(
        &self,
        job: &TranscriptionJob,
        transcript: &Transcript,
        language: Option<String>,
    ) -> RecordingResult<()> {
        self.app.state::<AppState>().with_workspace(|workspace| {
            workspace
                .sessions()
                .save_transcript(job.session_id, transcript, language)
                .map(drop)
        })?;
        tracing::info!(session_id = ?job.session_id, lines = transcript.segments().len(), "session transcribed");
        ShellEvent::RecordingChanged.emit(self.app);
        ShellEvent::TranscriptReady.emit_with(self.app, &job.session_id);
        Ok(())
    }

    fn fail(&self, job: &TranscriptionJob, reason: String) -> RecordingResult<()> {
        tracing::warn!(session_id = ?job.session_id, %reason, "could not transcribe the session");
        self.app.state::<AppState>().with_workspace(|workspace| {
            workspace
                .sessions()
                .fail_transcription(job.session_id, reason)
                .map(drop)
        })?;
        ShellEvent::RecordingChanged.emit(self.app);
        Ok(())
    }
}
