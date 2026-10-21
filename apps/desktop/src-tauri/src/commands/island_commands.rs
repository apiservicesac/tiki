use tauri::State;
use tiki_core::capture::{CaptureIntent, CaptureOutcome, ConfirmedCapture};
use tiki_core::domain::{TaskId, TaskStatus};
use tiki_core::editing::{TaskDetails, TaskEdit};
use tiki_core::workspace::{AssistantContext, IslandSnapshot};

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

/// Saves a task exactly as the user confirmed it after the assistant read their sentence.
#[tauri::command]
pub fn save_confirmed_capture(
    state: State<'_, AppState>,
    capture: ConfirmedCapture,
) -> CommandResult<()> {
    state.with_workspace(|workspace| workspace.save_confirmed(capture))?;
    Ok(())
}

/// Client names only: the context the assistant needs to recognize them in a sentence.
#[tauri::command]
pub fn client_names(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let clients = state.with_workspace(|workspace| workspace.clients())?;
    Ok(clients.into_iter().map(|client| client.name).collect())
}

#[tauri::command]
pub fn task_details(state: State<'_, AppState>, task_id: i64) -> CommandResult<TaskDetails> {
    Ok(state.with_workspace(|workspace| workspace.task_details(TaskId(task_id)))?)
}

#[tauri::command]
pub fn edit_task(state: State<'_, AppState>, task_id: i64, edit: TaskEdit) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.edit_task(TaskId(task_id), edit))?)
}

/// Open tasks for the assistant to summarize the day or answer a question.
#[tauri::command]
pub fn assistant_context(state: State<'_, AppState>) -> CommandResult<AssistantContext> {
    Ok(state.with_workspace(|workspace| workspace.assistant_context())?)
}
