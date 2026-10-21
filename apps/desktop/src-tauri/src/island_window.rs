use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tiki_core::preferences::{IslandGeometry, IslandPlacement, ScreenRect};

use crate::desktop_config::DesktopConfig;

/// The always-on-top window that hosts the island on the chosen screen edge.
pub struct IslandWindow {
    window: WebviewWindow,
}

impl IslandWindow {
    pub fn find(app: &AppHandle) -> Option<Self> {
        app.get_webview_window(DesktopConfig::ISLAND_WINDOW_LABEL)
            .map(|window| Self { window })
    }

    /// Resizes the window to the island (in logical pixels) and moves it to its edge.
    pub fn fit(&self, width: f64, height: f64, placement: IslandPlacement) -> tauri::Result<()> {
        self.window.set_size(LogicalSize::new(width, height))?;
        self.place(placement)
    }

    /// Moves the window to the chosen edge of its monitor.
    pub fn place(&self, placement: IslandPlacement) -> tauri::Result<()> {
        let Some(monitor) = self.window.current_monitor()? else {
            return Ok(());
        };
        let area = monitor.work_area();
        let work_area = ScreenRect {
            x: area.position.x,
            y: area.position.y,
            width: Self::signed(area.size.width),
            height: Self::signed(area.size.height),
        };
        let window_size = self.window.outer_size()?;
        let (x, y) = IslandGeometry::window_origin(
            placement,
            work_area,
            Self::signed(window_size.width),
            Self::signed(window_size.height),
        );
        self.window.set_position(PhysicalPosition::new(x, y))
    }

    /// Shows the window and gives it the keyboard focus.
    pub fn bring_forward(&self) {
        if let Err(error) = self.window.show().and_then(|()| self.window.set_focus()) {
            tracing::warn!(%error, "could not bring the island forward");
        }
    }

    fn signed(pixels: u32) -> i32 {
        i32::try_from(pixels).unwrap_or(i32::MAX)
    }
}
