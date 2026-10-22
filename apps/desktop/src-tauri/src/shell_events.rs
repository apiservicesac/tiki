use tauri::{AppHandle, Emitter};

/// Events the shell sends to the interface. The names must match `SHELL_EVENTS`
/// in `src/config/uiConfig.ts`.
#[derive(Debug, Clone, Copy)]
pub enum ShellEvent {
    OpenCapture,
    OpenSettings,
    ToggleIsland,
    /// A brief is due; the island shows it in the notch (the payload is the brief).
    BriefReady,
    /// Preferences or AI keys changed in one window; the others reload them.
    PreferencesChanged,
    /// The calendars were copied again: the Agenda refreshes.
    CalendarSynced,
}

impl ShellEvent {
    pub fn name(self) -> &'static str {
        match self {
            ShellEvent::OpenCapture => "tiki://open-capture",
            ShellEvent::OpenSettings => "tiki://open-settings",
            ShellEvent::ToggleIsland => "tiki://toggle-island",
            ShellEvent::BriefReady => "tiki://brief-ready",
            ShellEvent::PreferencesChanged => "tiki://preferences-changed",
            ShellEvent::CalendarSynced => "tiki://calendar-synced",
        }
    }

    /// Sends the event with data, such as the brief the island should show.
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
