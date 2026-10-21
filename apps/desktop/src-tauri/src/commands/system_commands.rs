use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use super::command_error::{CommandError, CommandResult};
use crate::app_updates::{AppUpdates, AvailableUpdate};

/// The installed version, shown in Settings.
#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> CommandResult<Option<AvailableUpdate>> {
    AppUpdates::check(&app)
        .await
        .map_err(|error| CommandError::new(&error))
}

/// Installs the latest signed version; Tiki restarts when it finishes.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> CommandResult<()> {
    AppUpdates::install(&app)
        .await
        .map_err(|error| CommandError::new(&error))
}

/// Whether Tiki starts with Windows.
#[tauri::command]
pub fn autostart_enabled(app: AppHandle) -> CommandResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| CommandError::new(&error))
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> CommandResult<()> {
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|error| CommandError::new(&error))
}
