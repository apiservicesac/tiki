use tauri::{AppHandle, State};
use tiki_core::preferences::UserPreferences;

use super::command_error::CommandResult;
use crate::app_state::AppState;
use crate::island_window::IslandWindow;
use crate::shell_events::ShellEvent;

#[tauri::command]
pub fn preferences(state: State<'_, AppState>) -> CommandResult<UserPreferences> {
    Ok(state.with_workspace(|workspace| Ok(workspace.preferences().clone()))?)
}

/// Saves the preferences and moves the island to the newly chosen edge and orientation.
#[tauri::command]
pub fn update_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: UserPreferences,
) -> CommandResult<()> {
    let placement = preferences.island_placement;
    tracing::info!(
        morning = %preferences.morning_brief_time,
        evening = %preferences.evening_brief_time,
        notifications = preferences.notifications_enabled,
        focus = preferences.focus_mode,
        "preferences saved"
    );
    state.with_workspace(|workspace| workspace.update_preferences(preferences))?;
    if let Some(island) = IslandWindow::find(&app)
        && let Err(error) = island.prepare(placement)
    {
        tracing::warn!(%error, "could not move the island");
    }
    ShellEvent::PreferencesChanged.emit(&app);
    Ok(())
}
