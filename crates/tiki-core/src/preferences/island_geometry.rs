use super::IslandPlacement;

/// A rectangle on screen, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Where the island window goes for each placement.
///
/// The island touches the chosen edge of the monitor, centered along that edge, and stays
/// inside the work area so it never hides under the taskbar.
pub struct IslandGeometry;

impl IslandGeometry {
    /// Top-left corner of a window of `window_width` × `window_height` pixels.
    pub fn window_origin(
        placement: IslandPlacement,
        work_area: ScreenRect,
        window_width: i32,
        window_height: i32,
    ) -> (i32, i32) {
        let centered_x = work_area.x + (work_area.width - window_width) / 2;
        let centered_y = work_area.y + (work_area.height - window_height) / 2;
        match placement {
            IslandPlacement::Top => (centered_x, work_area.y),
            IslandPlacement::Bottom => (centered_x, work_area.y + work_area.height - window_height),
            IslandPlacement::Left => (work_area.x, centered_y),
            IslandPlacement::Right => (work_area.x + work_area.width - window_width, centered_y),
        }
    }
}
