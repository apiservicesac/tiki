use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tiki_core::i18n::Translator;
use tiki_core::sessions::{SessionExport, SessionId};
use tiki_core::{TikiError, TikiResult};

use crate::app_state::AppState;
use crate::session_files::SessionFiles;

/// Writes a session to a folder in the user's Downloads: its media, the transcript as
/// subtitles and as text, and the notes as Markdown.
pub struct SessionExporter;

impl SessionExporter {
    /// The folder the session was written to.
    pub fn export(app: &AppHandle, session_id: SessionId) -> TikiResult<PathBuf> {
        let (session, details, translator) =
            app.state::<AppState>().with_workspace(|workspace| {
                let sessions = workspace.sessions();
                Ok((
                    sessions.find(session_id)?,
                    sessions.details(session_id)?,
                    Translator::new(workspace.preferences().locale)?,
                ))
            })?;
        let export = SessionExport::new(&translator);
        let downloads = app
            .path()
            .download_dir()
            .map_err(|error| TikiError::Io(std::io::Error::other(error.to_string())))?;
        let folder = downloads.join(export.folder_name(&details));
        fs::create_dir_all(&folder)?;
        if let Some(media) = app.state::<SessionFiles>().media_of(&session)?
            && let Some(name) = media.file_name()
        {
            fs::copy(&media, folder.join(name))?;
        }
        fs::write(
            folder.join(translator.text("session.export.subtitles_file")),
            export.subtitles(&details.transcript),
        )?;
        fs::write(
            folder.join(translator.text("session.export.text_file")),
            export.text(&details.transcript),
        )?;
        if let Some(notes) = export.notes(&details) {
            fs::write(
                folder.join(translator.text("session.export.notes_file")),
                notes,
            )?;
        }
        Ok(folder)
    }
}
