use std::io;
use std::path::{Path, PathBuf};

use tiki_core::sessions::Session;

use crate::desktop_config::DesktopConfig;

/// Where sessions keep their files: one folder per session under the app data folder.
pub struct SessionFiles {
    root: PathBuf,
}

impl SessionFiles {
    pub fn new(app_data_folder: &Path) -> Self {
        Self {
            root: app_data_folder.join(DesktopConfig::SESSIONS_FOLDER),
        }
    }

    /// The session's folder, created if it does not exist yet.
    pub fn prepare(&self, session: &Session) -> io::Result<PathBuf> {
        let folder = self.folder_of(session)?;
        std::fs::create_dir_all(&folder)?;
        Ok(folder)
    }

    /// The media played back, while the session keeps it.
    pub fn media_of(&self, session: &Session) -> io::Result<Option<PathBuf>> {
        if !session.has_media() {
            return Ok(None);
        }
        let folder = self.folder_of(session)?;
        Ok(session.media_file.as_ref().map(|file| folder.join(file)))
    }

    /// Removes the media played back; the transcript and notes live in the database.
    pub fn remove_media(&self, session: &Session) -> io::Result<()> {
        match self.media_of(session)? {
            Some(media) => Self::unless_missing(std::fs::remove_file(media)),
            None => Ok(()),
        }
    }

    /// Removes the session's folder and everything in it.
    pub fn remove(&self, session: &Session) -> io::Result<()> {
        Self::unless_missing(std::fs::remove_dir_all(self.folder_of(session)?))
    }

    pub fn folder_of(&self, session: &Session) -> io::Result<PathBuf> {
        session
            .folder_name()
            .map(|name| self.root.join(name))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "the session has no folder yet"))
    }

    /// A file already gone is what removing it wanted.
    fn unless_missing(removed: io::Result<()>) -> io::Result<()> {
        match removed {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
