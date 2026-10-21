use tauri::window::Color;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::desktop_config::DesktopConfig;

/// The advanced settings window: a normal window with its own title bar, opened from the
/// island. Only one exists at a time.
pub struct SettingsWindow;

impl SettingsWindow {
    /// Opens the window on `section`, or brings the open one forward.
    pub fn open(app: &AppHandle, title: &str, section: Option<&str>) -> tauri::Result<()> {
        if let Some(window) = app.get_webview_window(DesktopConfig::SETTINGS_WINDOW_LABEL) {
            window.unminimize()?;
            window.show()?;
            return window.set_focus();
        }
        let page = match section {
            Some(section) => format!("{}#{section}", DesktopConfig::SETTINGS_PAGE),
            None => DesktopConfig::SETTINGS_PAGE.to_owned(),
        };
        let (width, height) = DesktopConfig::SETTINGS_WINDOW_SIZE;
        let (min_width, min_height) = DesktopConfig::SETTINGS_WINDOW_MIN_SIZE;
        WebviewWindowBuilder::new(
            app,
            DesktopConfig::SETTINGS_WINDOW_LABEL,
            WebviewUrl::App(page.into()),
        )
        .title(title)
        .inner_size(width, height)
        .min_inner_size(min_width, min_height)
        .additional_browser_args(DesktopConfig::BROWSER_ARGS)
        .decorations(false)
        // The island's notch color, so the window never flashes white while it loads.
        .background_color(Color(10, 10, 13, 255))
        .shadow(true)
        .center()
        .focused(true)
        .build()?;
        Ok(())
    }
}
