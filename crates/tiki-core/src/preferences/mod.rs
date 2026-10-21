//! Choices the user makes in Settings, saved in the database.

mod island_geometry;
mod island_hit_area;
mod island_placement;
mod user_preferences;

pub use island_geometry::{IslandGeometry, ScreenRect};
pub use island_hit_area::IslandHitArea;
pub use island_placement::IslandPlacement;
pub use user_preferences::UserPreferences;
