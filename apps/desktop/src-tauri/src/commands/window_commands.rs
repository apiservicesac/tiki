use tauri::{AppHandle, State};

use super::command_error::CommandResult;
use crate::app_state::AppState;
use crate::island_window::IslandWindow;

/// Makes the window exactly as big as the island, so the transparent space around it
/// never blocks clicks on other applications.
#[tauri::command]
pub fn fit_island_window(
    app: AppHandle,
    state: State<'_, AppState>,
    width: f64,
    height: f64,
) -> CommandResult<()> {
    let placement =
        state.with_workspace(|workspace| Ok(workspace.preferences().island_placement))?;
    if let Some(island) = IslandWindow::find(&app)
        && let Err(error) = island.fit(width, height, placement)
    {
        tracing::warn!(%error, "could not resize the island");
    }
    Ok(())
}
