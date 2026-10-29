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

    pub fn folder_of(&self, session: &Session) -> io::Result<PathBuf> {
        session
            .folder_name()
            .map(|name| self.root.join(name))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "the session has no folder yet"))
    }
}
