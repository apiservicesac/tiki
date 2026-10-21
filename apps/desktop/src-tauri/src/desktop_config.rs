use std::time::Duration;

/// Fixed values of the desktop shell, kept in one place.
pub struct DesktopConfig;

impl DesktopConfig {
    /// Label of the island window in `tauri.conf.json`.
    pub const ISLAND_WINDOW_LABEL: &'static str = "island";
    /// The window does not resize while the island animates: it is as big as the expanded
    /// island (plus its ears and shadow), and clicks outside the island pass through.
    /// Top and bottom use a wide window; left and right use a tall one.
    pub const HORIZONTAL_WINDOW_SIZE: (f64, f64) = (700.0, 560.0);
    pub const VERTICAL_WINDOW_SIZE: (f64, f64) = (440.0, 720.0);
    /// How often the shell checks whether the pointer is over the island.
    pub const CURSOR_POLL_INTERVAL: Duration = Duration::from_millis(30);
    /// How often the shell asks the core whether a brief notification is due.
    pub const REMINDER_CHECK_INTERVAL: Duration = Duration::from_secs(60);
    /// SQLite file inside the app data folder.
    pub const DATABASE_FILE_NAME: &'static str = "tiki.db";
    pub const TRAY_ID: &'static str = "tiki-tray";

    pub const SHORTCUT_QUICK_CAPTURE: &'static str = "ctrl+alt+space";
    pub const SHORTCUT_TOGGLE_ISLAND: &'static str = "ctrl+alt+n";
}
