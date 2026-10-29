use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tiki_core::preferences::{IslandGeometry, IslandHitArea, IslandPlacement, ScreenRect};

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

    /// Gives the window the size of its orientation, never larger than the screen, and moves
    /// it to the chosen edge. The island fits itself to the window it gets.
    pub fn prepare(&self, placement: IslandPlacement) -> tauri::Result<()> {
        let size = match placement {
            IslandPlacement::Top | IslandPlacement::Bottom => DesktopConfig::HORIZONTAL_WINDOW_SIZE,
            IslandPlacement::Left | IslandPlacement::Right => DesktopConfig::VERTICAL_WINDOW_SIZE,
        };
        let (width, height) = self.fit_to_screen(size)?;
        self.window.set_size(LogicalSize::new(width, height))?;
        self.place(placement)
    }

    /// The size cut down to the free area of the monitor, for small laptop screens.
    fn fit_to_screen(&self, (width, height): (f64, f64)) -> tauri::Result<(f64, f64)> {
        let Some(monitor) = self.window.current_monitor()? else {
            return Ok((width, height));
        };
        let free_area = monitor
            .work_area()
            .size
            .to_logical::<f64>(monitor.scale_factor());
        Ok((width.min(free_area.width), height.min(free_area.height)))
    }

    /// Moves the window to the chosen edge of its monitor.
    fn place(&self, placement: IslandPlacement) -> tauri::Result<()> {
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

    /// Whether the pointer is over the island part of the window.
    pub fn pointer_is_over(&self, hit_area: &IslandHitArea) -> tauri::Result<bool> {
        let cursor = self.window.cursor_position()?;
        let window_origin = self.window.outer_position()?;
        let scale = self.window.scale_factor()?;
        let x = (cursor.x - f64::from(window_origin.x)) / scale;
        let y = (cursor.y - f64::from(window_origin.y)) / scale;
        Ok(hit_area.contains(x, y))
    }

    /// When enabled, clicks go to whatever is behind the window.
    pub fn set_click_through(&self, enabled: bool) -> tauri::Result<()> {
        self.window.set_ignore_cursor_events(enabled)
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
