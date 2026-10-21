use tauri::{AppHandle, State};
use tiki_core::preferences::UserPreferences;

use super::command_error::CommandResult;
use crate::app_state::AppState;
use crate::island_window::IslandWindow;

#[tauri::command]
pub fn preferences(state: State<'_, AppState>) -> CommandResult<UserPreferences> {
    Ok(state.with_workspace(|workspace| Ok(workspace.preferences()))?)
}

/// Saves the preferences and moves the island to the newly chosen edge and orientation.
#[tauri::command]
pub fn update_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: UserPreferences,
) -> CommandResult<()> {
    state.with_workspace(|workspace| workspace.update_preferences(preferences))?;
    if let Some(island) = IslandWindow::find(&app)
        && let Err(error) = island.prepare(preferences.island_placement)
    {
        tracing::warn!(%error, "could not move the island");
    }
    Ok(())
}
