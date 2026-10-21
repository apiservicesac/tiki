use std::sync::{Arc, Mutex};
use std::thread;

use tauri::AppHandle;
use tiki_core::preferences::IslandHitArea;

use crate::desktop_config::DesktopConfig;
use crate::island_window::IslandWindow;

/// Lets clicks pass through the transparent part of the island window.
///
/// The interface reports where the island is; a background loop watches the pointer and
/// makes the window click-through whenever the pointer is outside that area.
#[derive(Default)]
pub struct ClickThrough {
    hit_area: Mutex<IslandHitArea>,
}

impl ClickThrough {
    pub fn update(&self, hit_area: IslandHitArea) {
        *self.lock() = hit_area;
    }

    /// Starts watching the pointer for the rest of the session.
    pub fn start(app: AppHandle, click_through: Arc<ClickThrough>) {
        thread::spawn(move || {
            let mut passing_through: Option<bool> = None;
            loop {
                thread::sleep(DesktopConfig::CURSOR_POLL_INTERVAL);
                let Some(island) = IslandWindow::find(&app) else {
                    continue;
                };
                let hit_area = *click_through.lock();
                let should_pass = !island.pointer_is_over(&hit_area).unwrap_or(false);
                if passing_through != Some(should_pass) {
                    match island.set_click_through(should_pass) {
                        Ok(()) => passing_through = Some(should_pass),
                        Err(error) => tracing::warn!(%error, "could not change click-through"),
                    }
                }
            }
        });
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, IslandHitArea> {
        self.hit_area
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
