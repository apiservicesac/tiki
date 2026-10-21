use std::sync::Arc;

use tauri::State;
use tiki_core::preferences::IslandHitArea;

use crate::click_through::ClickThrough;

/// Tells the shell where the island is, so the rest of the window lets clicks through.
#[tauri::command]
pub fn set_island_hit_area(click_through: State<'_, Arc<ClickThrough>>, area: IslandHitArea) {
    click_through.update(area);
}
