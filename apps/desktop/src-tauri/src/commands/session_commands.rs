use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tiki_core::domain::TaskId;
use tiki_core::sessions::{
    ActionItemId, ProposedSessionTask, RecordingMode, SessionDetails, SessionId, SessionNotes,
    SessionSearchHit, SessionSummary,
};
use tiki_recording::video::CaptureTarget;

use super::command_error::{CommandError, CommandResult};
use crate::app_state::AppState;
use crate::session_files::SessionFiles;
use crate::session_recorder::SessionRecorder;
use crate::shell_events::ShellEvent;
use crate::transcription_runner::TranscriptionRunner;
use crate::video_choice::{CaptureTargetView, VideoChoice};

/// A session as its detail shows it, with the file the player opens.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    #[serde(flatten)]
    details: SessionDetails,
    /// The media on disk, for the web view's asset protocol; `None` once it was removed.
    media_path: Option<String>,
}

/// What the sessions take on disk and how long their media is kept.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStorage {
    media_bytes: u64,
    media_days: u64,
}

/// Starts recording a session in the mode the user clicked; a video session records the
/// screen or window they picked.
#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    recorder: State<'_, SessionRecorder>,
    mode: RecordingMode,
    video: Option<VideoChoice>,
) -> CommandResult<()> {
    Ok(recorder.start(&app, mode, video)?)
}

/// The screens and windows a video session can record.
#[tauri::command]
pub fn capture_targets() -> CommandResult<Vec<CaptureTargetView>> {
    Ok(CaptureTarget::list()
        .map_err(|error| CommandError::new(&error))?
        .into_iter()
        .map(CaptureTargetView::from)
        .collect())
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

/// Every session, newest first.
#[tauri::command]
pub fn sessions(state: State<'_, AppState>) -> CommandResult<Vec<SessionSummary>> {
    Ok(state.with_workspace(|workspace| workspace.sessions().list())?)
}

#[tauri::command]
pub fn session_details(
    state: State<'_, AppState>,
    files: State<'_, SessionFiles>,
    session_id: i64,
) -> CommandResult<SessionView> {
    let (details, session) = state.with_workspace(|workspace| {
        let sessions = workspace.sessions();
        Ok((
            sessions.details(SessionId(session_id))?,
            sessions.find(SessionId(session_id))?,
        ))
    })?;
    let media_path = files
        .media_of(&session)
        .map_err(|error| CommandError::new(&error))?
        .map(|path| path.to_string_lossy().into_owned());
    Ok(SessionView {
        details,
        media_path,
    })
}

/// Transcript lines with every word the user typed.
#[tauri::command]
pub fn search_sessions(
    state: State<'_, AppState>,
    text: String,
) -> CommandResult<Vec<SessionSearchHit>> {
    Ok(state.with_workspace(|workspace| workspace.sessions().search(&text))?)
}

/// Forgets a session and removes its folder; tasks made from it stay.
#[tauri::command]
pub fn delete_session(app: AppHandle, session_id: i64) -> CommandResult<()> {
    let session = app
        .state::<AppState>()
        .with_workspace(|workspace| workspace.sessions().delete(SessionId(session_id)))?;
    app.state::<SessionFiles>()
        .remove(&session)
        .map_err(|error| CommandError::new(&error))?;
    ShellEvent::RecordingChanged.emit(&app);
    Ok(())
}

/// Queues a failed transcription again.
#[tauri::command]
pub fn retry_transcription(app: AppHandle, session_id: i64) -> CommandResult<()> {
    app.state::<AppState>().with_workspace(|workspace| {
        workspace
            .sessions()
            .retry_transcription(SessionId(session_id))
            .map(drop)
    })?;
    app.state::<TranscriptionRunner>().wake();
    ShellEvent::RecordingChanged.emit(&app);
    Ok(())
}

#[tauri::command]
pub fn session_storage(state: State<'_, AppState>) -> CommandResult<SessionStorage> {
    Ok(state.with_workspace(|workspace| {
        Ok(SessionStorage {
            media_bytes: workspace.sessions().media_bytes()?,
            media_days: workspace.preferences().session_media_days,
        })
    })?)
}

/// Saves the notes the AI wrote for a session and the tasks they propose.
#[tauri::command]
pub fn save_session_notes(
    app: AppHandle,
    session_id: i64,
    notes: SessionNotes,
    tasks: Vec<ProposedSessionTask>,
) -> CommandResult<()> {
    app.state::<AppState>().with_workspace(|workspace| {
        workspace
            .sessions()
            .save_notes(SessionId(session_id), &notes, &tasks)
    })?;
    ShellEvent::RecordingChanged.emit(&app);
    Ok(())
}

/// Turns a task the AI proposed into a task of the user, pointing back to its session.
#[tauri::command]
pub fn apply_session_task(
    state: State<'_, AppState>,
    action_item_id: i64,
) -> CommandResult<TaskId> {
    Ok(state.with_workspace(|workspace| {
        workspace
            .sessions()
            .apply_action_item(ActionItemId(action_item_id))
    })?)
}
