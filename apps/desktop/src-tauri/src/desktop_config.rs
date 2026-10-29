use std::time::Duration;

/// Fixed values of the desktop shell, kept in one place.
pub struct DesktopConfig;

impl DesktopConfig {
    /// Label of the island window in `tauri.conf.json`.
    pub const ISLAND_WINDOW_LABEL: &'static str = "island";
    /// `WebView2` flags: Tauri's defaults plus sound without a click, for the island's chime.
    /// Every window must use the same flags (they share one `WebView2` profile), so this must
    /// match `additionalBrowserArgs` of the island in `tauri.conf.json`.
    pub const BROWSER_ARGS: &'static str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required";
    /// The advanced settings window and the page it loads.
    pub const SETTINGS_WINDOW_LABEL: &'static str = "settings";
    pub const SETTINGS_PAGE: &'static str = "settings.html";
    pub const SETTINGS_WINDOW_SIZE: (f64, f64) = (920.0, 640.0);
    pub const SETTINGS_WINDOW_MIN_SIZE: (f64, f64) = (760.0, 520.0);
    /// The window does not resize while the island animates: it is as big as the expanded
    /// island (plus its ears and shadow), and clicks outside the island pass through.
    /// Top and bottom use a wide window; left and right use a tall one.
    pub const HORIZONTAL_WINDOW_SIZE: (f64, f64) = (700.0, 560.0);
    pub const VERTICAL_WINDOW_SIZE: (f64, f64) = (440.0, 720.0);
    /// How often the shell checks whether the pointer is over the island.
    pub const CURSOR_POLL_INTERVAL: Duration = Duration::from_millis(30);
    /// How often the shell asks the core whether a brief notification is due.
    pub const REMINDER_CHECK_INTERVAL: Duration = Duration::from_secs(60);
    /// How often the shell looks whether another app holds the microphone (a call).
    pub const CALL_CHECK_INTERVAL: Duration = Duration::from_secs(5);
    /// How often Tiki looks for a new signed version.
    pub const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
    /// Diagnostics log in the app's log folder, rewritten on every launch.
    pub const LOG_FILE_NAME: &'static str = "tiki.log";
    pub const LOG_LEVEL: &'static str = "info";
    /// How often the calendars are copied.
    pub const CALENDAR_SYNC_INTERVAL: Duration = Duration::from_secs(5 * 60);
    /// SQLite file inside the app data folder.
    pub const DATABASE_FILE_NAME: &'static str = "tiki.db";
    /// Folder inside the app data folder with one folder per recorded session.
    pub const SESSIONS_FOLDER: &'static str = "sessions";
    pub const TRAY_ID: &'static str = "tiki-tray";

    pub const SHORTCUT_QUICK_CAPTURE: &'static str = "ctrl+alt+space";
    pub const SHORTCUT_TOGGLE_ISLAND: &'static str = "ctrl+alt+n";
    /// Stops the session being recorded, or asks what to record.
    pub const SHORTCUT_TOGGLE_RECORDING: &'static str = "ctrl+alt+r";
}
