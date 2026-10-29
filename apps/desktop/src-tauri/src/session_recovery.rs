use std::time::Duration;

use tauri::{AppHandle, Manager};
use tiki_core::TikiError;
use tiki_core::sessions::{RecordingMode, Session};
use tiki_recording::media::CrashRecovery;

use crate::app_state::AppState;
use crate::recorder_error::RecorderError;
use crate::session_files::SessionFiles;

/// Saves the session Tiki was recording when it closed (a crash, or the computer turning
/// off): it ends where its tracks end, keeps the media that can play, and is transcribed.
/// It runs at launch, before the transcription queue starts.
pub struct SessionRecovery;

impl SessionRecovery {
    pub fn run(app: &AppHandle) {
        let capturing = app
            .state::<AppState>()
            .with_workspace(|workspace| workspace.sessions().capturing());
        match capturing {
            Ok(Some(session)) => {
                if let Err(error) = Self::recover(app, &session) {
                    tracing::warn!(%error, "could not recover the session cut short");
                }
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "could not look for a session cut short"),
        }
    }

    fn recover(app: &AppHandle, session: &Session) -> Result<(), RecorderError> {
        let session_id = session.id.ok_or(TikiError::UnsavedSession)?;
        let folder = app.state::<SessionFiles>().folder_of(session)?;
        let (recorded, media) =
            match CrashRecovery::recover(&folder, session.mode == RecordingMode::AudioOnly) {
                Ok(recovered) => (
                    recovered.recorded,
                    recovered
                        .media
                        .map(|(file, bytes)| (file.to_owned(), bytes)),
                ),
                Err(error) => {
                    tracing::warn!(%error, "nothing of the session cut short could be saved");
                    (Duration::ZERO, None)
                }
            };
        let recorded = chrono::Duration::from_std(recorded).unwrap_or_default();
        app.state::<AppState>().with_workspace(|workspace| {
            workspace
                .sessions()
                .recover(session_id, recorded, media)
                .map(drop)
        })?;
        tracing::info!(?session_id, "session cut short recovered");
        Ok(())
    }
}
