use tauri::State;
use tiki_core::clients::{ClientChange, ClientSummary};
use tiki_core::domain::ClientId;

use super::command_error::CommandResult;
use crate::app_state::AppState;

/// Every client with their open and pressing work, busiest first.
#[tauri::command]
pub fn client_summaries(state: State<'_, AppState>) -> CommandResult<Vec<ClientSummary>> {
    Ok(state.with_workspace(|workspace| workspace.client_summaries())?)
}

#[tauri::command]
pub fn create_client(state: State<'_, AppState>, name: String) -> CommandResult<i64> {
    let client_id = state.with_workspace(|workspace| workspace.add_client(&name))?;
    Ok(client_id.0)
}

#[tauri::command]
pub fn change_client(
    state: State<'_, AppState>,
    client_id: i64,
    change: ClientChange,
) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.change_client(ClientId(client_id), change))?)
}

/// Removes a client; their tasks stay, without client.
#[tauri::command]
pub fn remove_client(state: State<'_, AppState>, client_id: i64) -> CommandResult<()> {
    Ok(state.with_workspace(|workspace| workspace.remove_client(ClientId(client_id)))?)
}

/// The colors a client can take.
#[tauri::command]
pub fn client_colors(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    Ok(state.with_workspace(|workspace| Ok(workspace.client_colors().to_vec()))?)
}
