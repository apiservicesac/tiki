use tauri::{AppHandle, Emitter};

/// Events the shell sends to the interface. The names must match `SHELL_EVENTS`
/// in `src/config/uiConfig.ts`.
#[derive(Debug, Clone, Copy)]
pub enum ShellEvent {
    OpenCapture,
    OpenSettings,
    ToggleIsland,
    /// A brief notification was just shown; the island peeks out to go with it.
    BriefReady,
    /// Preferences or AI keys changed in one window; the others reload them.
    PreferencesChanged,
}

impl ShellEvent {
    pub fn name(self) -> &'static str {
        match self {
            ShellEvent::OpenCapture => "tiki://open-capture",
            ShellEvent::OpenSettings => "tiki://open-settings",
            ShellEvent::ToggleIsland => "tiki://toggle-island",
            ShellEvent::BriefReady => "tiki://brief-ready",
            ShellEvent::PreferencesChanged => "tiki://preferences-changed",
        }
    }

    pub fn emit(self, app: &AppHandle) {
        if let Err(error) = app.emit(self.name(), ()) {
            tracing::warn!(event = self.name(), %error, "could not notify the interface");
        }
    }
}
