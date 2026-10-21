/// Fixed values of the desktop shell, kept in one place.
pub struct DesktopConfig;

impl DesktopConfig {
    /// Label of the island window in `tauri.conf.json`.
    pub const ISLAND_WINDOW_LABEL: &'static str = "island";
    /// SQLite file inside the app data folder.
    pub const DATABASE_FILE_NAME: &'static str = "tiki.db";
    pub const TRAY_ID: &'static str = "tiki-tray";

    pub const SHORTCUT_QUICK_CAPTURE: &'static str = "ctrl+alt+space";
    pub const SHORTCUT_TOGGLE_ISLAND: &'static str = "ctrl+alt+n";
}
