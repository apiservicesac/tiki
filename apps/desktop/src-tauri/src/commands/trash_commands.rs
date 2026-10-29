use tauri::State;
use tiki_core::domain::TaskId;
use tiki_core::workspace::TrashedItem;

use super::command_error::CommandResult;
use crate::app_state::AppState;

/// Sends an item to the trash: it leaves every view and its reminders stay silent.
#[tauri::command]
pub fn trash_item(state: State<'_, AppState>, task_id: i64) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.trash_item(TaskId(task_id)))?)
}

/// Takes an item out of the trash exactly as it was.
#[tauri::command]
pub fn restore_item(state: State<'_, AppState>, task_id: i64) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.restore_item(TaskId(task_id)))?)
}

/// Deletes an item in the trash forever, with its history.
#[tauri::command]
pub fn delete_item_forever(state: State<'_, AppState>, task_id: i64) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.delete_item_forever(TaskId(task_id)))?)
}

/// Deletes every item in the trash forever.
#[tauri::command]
pub fn empty_trash(state: State<'_, AppState>) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.empty_trash())?)
}

/// Items in the trash, most recently trashed first.
#[tauri::command]
pub fn trashed_items(state: State<'_, AppState>) -> CommandResult<Vec<TrashedItem>> {
    Ok(state.with_workspace(|workspace| workspace.trashed_items())?)
}

/// How many days an item stays in the trash before it is deleted forever.
#[tauri::command]
pub fn trash_retention_days(state: State<'_, AppState>) -> CommandResult<u64> {
    Ok(state.with_workspace(|workspace| Ok(workspace.settings().trash_retention_days))?)
}
