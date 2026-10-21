use std::fs::{self, File};
use std::path::Path;
use std::sync::Mutex;

use tracing_subscriber::EnvFilter;

use crate::desktop_config::DesktopConfig;

/// Writes what the shell does (reminders, updates, errors) to a file in the app's log
/// folder, so a problem on the user's machine can be explained afterwards. The file starts
/// over on every launch, so it never grows without limit.
pub struct DiagnosticsLog;

impl DiagnosticsLog {
    pub fn start(log_folder: &Path) -> std::io::Result<()> {
        fs::create_dir_all(log_folder)?;
        let file = File::create(log_folder.join(DesktopConfig::LOG_FILE_NAME))?;
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(DesktopConfig::LOG_LEVEL));
        // Ignore a second start (tests, restarts): the first subscriber keeps writing.
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_writer(Mutex::new(file))
            .try_init();
        Ok(())
    }
}
