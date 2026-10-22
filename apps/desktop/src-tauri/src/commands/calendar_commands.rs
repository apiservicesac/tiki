use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tiki_core::calendar::{AccountId, CalendarAccount, CalendarProvider};

use super::command_error::{CommandError, CommandResult};
use crate::app_state::AppState;
use crate::calendar::CalendarSync;

/// The connected accounts, and which providers this build can connect.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarStatus {
    accounts: Vec<CalendarAccount>,
    available: Vec<CalendarProvider>,
}

#[tauri::command]
pub fn calendar_status(state: State<'_, AppState>) -> CommandResult<CalendarStatus> {
    Ok(CalendarStatus {
        accounts: state.with_workspace(|workspace| workspace.calendar_accounts())?,
        available: CalendarSync::available(),
    })
}

/// Signs in in the browser and copies the account's meetings. Async: it waits for the user.
#[tauri::command]
pub async fn connect_calendar(
    app: AppHandle,
    provider: CalendarProvider,
) -> CommandResult<CalendarAccount> {
    Ok(CalendarSync::default().connect(&app, provider).await?)
}

#[tauri::command]
pub fn disconnect_calendar(app: AppHandle, account_id: i64) -> CommandResult<()> {
    Ok(CalendarSync::disconnect(&app, AccountId(account_id))?)
}

#[tauri::command]
pub async fn sync_calendars(app: AppHandle) -> CommandResult<()> {
    CalendarSync::default().sync_all(&app).await;
    Ok(())
}

/// Opens a meeting's video call in the browser or its app. Only web links are opened.
#[tauri::command]
pub fn open_meeting_link(app: AppHandle, url: String) -> CommandResult<()> {
    if !url.starts_with("https://") {
        return Err(CommandError::new(&"only https links can be opened"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| CommandError::new(&error))
}
