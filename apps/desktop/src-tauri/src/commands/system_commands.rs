use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use super::command_error::{CommandError, CommandResult};

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
