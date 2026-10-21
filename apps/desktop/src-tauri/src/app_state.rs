use std::path::Path;
use std::sync::Mutex;

use tiki_core::storage::Database;
use tiki_core::workspace::{SystemClock, TikiWorkspace};
use tiki_core::{TikiResult, TikiSettings};

use crate::desktop_config::DesktopConfig;

/// The user's workspace, shared by every command.
pub struct AppState {
    workspace: Mutex<TikiWorkspace>,
}

impl AppState {
    /// Opens the database in the app data folder, creating the folder on first run.
    pub fn open(app_data_folder: &Path) -> TikiResult<Self> {
        std::fs::create_dir_all(app_data_folder)?;
        let database = Database::open(&app_data_folder.join(DesktopConfig::DATABASE_FILE_NAME))?;
        let workspace =
            TikiWorkspace::new(database, TikiSettings::default(), Box::new(SystemClock))?;
        Ok(Self {
            workspace: Mutex::new(workspace),
        })
    }

    /// Runs an action with exclusive access to the workspace.
    pub fn with_workspace<T>(
        &self,
        action: impl FnOnce(&mut TikiWorkspace) -> TikiResult<T>,
    ) -> TikiResult<T> {
        let mut workspace = self
            .workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        action(&mut workspace)
    }
}
