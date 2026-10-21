use std::time::Duration;

/// Fixed values of the desktop shell, kept in one place.
pub struct DesktopConfig;

impl DesktopConfig {
    /// Label of the island window in `tauri.conf.json`.
    pub const ISLAND_WINDOW_LABEL: &'static str = "island";
    /// The window never resizes: it is as big as the expanded island (plus its ears and
    /// shadow), so the island animates with CSS only. Clicks outside it pass through.
    pub const ISLAND_WINDOW_WIDTH: f64 = 700.0;
    pub const ISLAND_WINDOW_HEIGHT: f64 = 560.0;
    /// How often the shell checks whether the pointer is over the island.
    pub const CURSOR_POLL_INTERVAL: Duration = Duration::from_millis(30);
    /// SQLite file inside the app data folder.
    pub const DATABASE_FILE_NAME: &'static str = "tiki.db";
    pub const TRAY_ID: &'static str = "tiki-tray";

    pub const SHORTCUT_QUICK_CAPTURE: &'static str = "ctrl+alt+space";
    pub const SHORTCUT_TOGGLE_ISLAND: &'static str = "ctrl+alt+n";
}
