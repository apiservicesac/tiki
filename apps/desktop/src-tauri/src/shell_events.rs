use tauri::{AppHandle, Emitter};

/// Events the shell sends to the interface. The names must match `SHELL_EVENTS`
/// in `src/config/uiConfig.ts`.
#[derive(Debug, Clone, Copy)]
pub enum ShellEvent {
    OpenCapture,
    OpenSettings,
    ToggleIsland,
    /// An alert is due; the island shows it in the notch (the payload is a `DueAlert`: the
    /// alert and how it reaches the user).
    AlertReady,
    /// Preferences or AI keys changed in one window; the others reload them.
    PreferencesChanged,
    /// The calendars were copied again: the Agenda refreshes.
    CalendarSynced,
    /// A call started or ended: the island redraws the notch.
    InCallChanged,
    /// A session started, paused, resumed or stopped recording, or its transcript or notes
    /// moved on: the notch and the Sessions tab refresh.
    RecordingChanged,
    /// The recording shortcut: the island stops the session or asks what to record.
    ToggleRecording,
    /// A session's transcript is ready (the payload is its id): the AI may write its notes.
    TranscriptReady,
    /// The transcription model is downloading (the payload says how far it got).
    ModelDownloadProgress,
    /// A new measure of the computer (the payload is a `SystemStats`).
    SystemStats,
}

impl ShellEvent {
    pub fn name(self) -> &'static str {
        match self {
            ShellEvent::OpenCapture => "tiki://open-capture",
            ShellEvent::OpenSettings => "tiki://open-settings",
            ShellEvent::ToggleIsland => "tiki://toggle-island",
            ShellEvent::AlertReady => "tiki://alert-ready",
            ShellEvent::PreferencesChanged => "tiki://preferences-changed",
            ShellEvent::CalendarSynced => "tiki://calendar-synced",
            ShellEvent::InCallChanged => "tiki://in-call-changed",
            ShellEvent::RecordingChanged => "tiki://recording-changed",
            ShellEvent::ToggleRecording => "tiki://toggle-recording",
            ShellEvent::TranscriptReady => "tiki://transcript-ready",
            ShellEvent::ModelDownloadProgress => "tiki://model-download-progress",
            ShellEvent::SystemStats => "tiki://system-stats",
        }
    }

    /// Sends the event with data, such as the alert the island should show.
    pub fn emit_with(self, app: &AppHandle, payload: &impl serde::Serialize) {
        if let Err(error) = app.emit(self.name(), payload) {
            tracing::warn!(event = self.name(), %error, "could not notify the interface");
        }
    }

    pub fn emit(self, app: &AppHandle) {
        if let Err(error) = app.emit(self.name(), ()) {
            tracing::warn!(event = self.name(), %error, "could not notify the interface");
        }
    }
}
