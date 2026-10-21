use std::sync::Arc;

use tauri::{AppHandle, State};
use tiki_core::i18n::Translator;
use tiki_core::preferences::IslandHitArea;

use super::command_error::{CommandError, CommandResult};
use crate::app_state::AppState;
use crate::click_through::ClickThrough;
use crate::settings_window::SettingsWindow;

/// Tells the shell where the island is, so the rest of the window lets clicks through.
#[tauri::command]
pub fn set_island_hit_area(click_through: State<'_, Arc<ClickThrough>>, area: IslandHitArea) {
    click_through.update(area);
}

/// Opens the advanced settings window, optionally on one section (`ai`, `plugins`…).
#[tauri::command]
pub fn open_advanced_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    section: Option<String>,
) -> CommandResult<()> {
    let locale = state.with_workspace(|workspace| Ok(workspace.preferences().locale))?;
    let title = Translator::new(locale)?.text("advanced.window_title");
    SettingsWindow::open(&app, &title, section.as_deref())
        .map_err(|error| CommandError::new(&error))
}
