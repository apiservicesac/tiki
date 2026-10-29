//! Tiki desktop shell. It owns the operating-system pieces (island window, tray,
//! global shortcuts, notifications) and delegates every decision to `tiki-core`.

mod ai;
mod app_state;
mod app_updates;
mod brief_reminders;
mod calendar;
#[cfg(windows)]
mod call_watch;
mod click_through;
mod commands;
mod core_transcription_queue;
mod desktop_app;
mod desktop_config;
mod diagnostics_log;
mod island_window;
mod recorder_error;
#[cfg(windows)]
mod registry_consent_store;
mod session_exporter;
mod session_files;
mod session_recorder;
mod session_recovery;
mod session_retention;
mod settings_window;
mod shell_events;
mod shortcuts;
mod transcription_runner;
mod tray;
mod video_choice;

pub use desktop_app::TikiDesktopApp;
