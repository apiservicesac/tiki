use tauri::State;
use tiki_core::domain::ProjectId;
use tiki_core::projects::{ProjectChange, ProjectSummary};

use super::command_error::CommandResult;
use crate::app_state::AppState;

/// Every project with their open and pressing work, busiest first.
#[tauri::command]
pub fn project_summaries(state: State<'_, AppState>) -> CommandResult<Vec<ProjectSummary>> {
    Ok(state.with_workspace(|workspace| workspace.project_summaries())?)
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, name: String) -> CommandResult<i64> {
    let project_id = state.with_workspace(|workspace| workspace.add_project(&name))?;
    Ok(project_id.0)
}

#[tauri::command]
pub fn change_project(
    state: State<'_, AppState>,
    project_id: i64,
    change: ProjectChange,
) -> CommandResult<()> {
    Ok(
        state
            .with_workspace(|workspace| workspace.change_project(ProjectId(project_id), change))?,
    )
}

/// Removes a project and sends its items to the trash, where they stay without project.
#[tauri::command]
pub fn remove_project(state: State<'_, AppState>, project_id: i64) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.remove_project(ProjectId(project_id)))?)
}

/// The colors a project can take.
#[tauri::command]
pub fn project_colors(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    Ok(state.with_workspace(|workspace| Ok(workspace.project_colors().to_vec()))?)
}
