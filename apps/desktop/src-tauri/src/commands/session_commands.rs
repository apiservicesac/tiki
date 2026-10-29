use tauri::{AppHandle, State};
use tiki_core::sessions::RecordingMode;

use super::command_error::CommandResult;
use crate::session_recorder::SessionRecorder;

/// Starts recording a session in the mode the user clicked.
#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    recorder: State<'_, SessionRecorder>,
    mode: RecordingMode,
) -> CommandResult<()> {
    Ok(recorder.start(&app, mode)?)
}

#[tauri::command]
pub fn pause_recording(app: AppHandle, recorder: State<'_, SessionRecorder>) -> CommandResult<()> {
    Ok(recorder.pause(&app)?)
}

#[tauri::command]
pub fn resume_recording(app: AppHandle, recorder: State<'_, SessionRecorder>) -> CommandResult<()> {
    Ok(recorder.resume(&app)?)
}

/// Stops recording; the session waits for its transcript.
#[tauri::command]
pub fn stop_recording(recorder: State<'_, SessionRecorder>) -> CommandResult<()> {
    Ok(recorder.stop()?)
}
