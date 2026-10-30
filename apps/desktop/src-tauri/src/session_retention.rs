use std::thread;

use tauri::{AppHandle, Manager};
use tiki_core::sessions::RetentionPolicy;
use tiki_core::{TikiError, TikiResult};

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::session_files::SessionFiles;
use crate::shell_events::ShellEvent;

/// Removes the media of sessions older than the user's retention (90 days by default),
/// at launch and every `RETENTION_CHECK_INTERVAL`. Transcripts and notes stay.
pub struct SessionRetention;

impl SessionRetention {
    pub fn start(app: AppHandle) {
        thread::spawn(move || {
            loop {
                if let Err(error) = Self::sweep(&app) {
                    tracing::warn!(%error, "could not remove old session media");
                }
                thread::sleep(DesktopConfig::RETENTION_CHECK_INTERVAL);
            }
        });
    }

    fn sweep(app: &AppHandle) -> TikiResult<()> {
        let state = app.state::<AppState>();
        let expired = state.with_workspace(|workspace| {
            let policy = RetentionPolicy {
                media_days: workspace.preferences().session_media_days,
            };
            workspace.sessions().expired_media(policy)
        })?;
        if expired.is_empty() {
            return Ok(());
        }
        let files = app.state::<SessionFiles>();
        for session in expired {
            files.remove_media(&session)?;
            let session_id = session.id.ok_or(TikiError::UnsavedSession)?;
            state.with_workspace(|workspace| {
                workspace.sessions().media_removed(session_id).map(drop)
            })?;
            tracing::info!(?session_id, "old session media removed");
        }
        ShellEvent::RecordingChanged.emit(app);
        Ok(())
    }
}
