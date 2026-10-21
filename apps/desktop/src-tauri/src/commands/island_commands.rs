use tauri::State;
use tiki_core::capture::{CaptureIntent, CaptureOutcome};
use tiki_core::domain::{TaskId, TaskStatus};
use tiki_core::workspace::IslandSnapshot;

use super::command_error::CommandResult;
use crate::app_state::AppState;

#[tauri::command]
pub fn island_snapshot(state: State<'_, AppState>) -> CommandResult<IslandSnapshot> {
    Ok(state.with_workspace(|workspace| workspace.island_snapshot())?)
}

#[tauri::command]
pub fn preview_capture(
    state: State<'_, AppState>,
    sentence: String,
) -> CommandResult<CaptureIntent> {
    Ok(state.with_workspace(|workspace| workspace.preview_capture(&sentence))?)
}

#[tauri::command]
pub fn capture(state: State<'_, AppState>, sentence: String) -> CommandResult<CaptureOutcome> {
    Ok(state.with_workspace(|workspace| workspace.capture(&sentence))?)
}

#[tauri::command]
pub fn move_task(
    state: State<'_, AppState>,
    task_id: i64,
    status: TaskStatus,
) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.move_task(TaskId(task_id), status))?)
}
