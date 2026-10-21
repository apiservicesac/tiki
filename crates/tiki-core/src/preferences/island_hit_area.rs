use serde::{Deserialize, Serialize};

/// The part of the island window that should receive the mouse, in logical pixels
/// relative to the window. Everything outside lets clicks pass through to other apps.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct IslandHitArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl IslandHitArea {
    pub fn contains(&self, point_x: f64, point_y: f64) -> bool {
        point_x >= self.x
            && point_x < self.x + self.width
            && point_y >= self.y
            && point_y < self.y + self.height
    }
}
