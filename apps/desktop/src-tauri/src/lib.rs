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
mod desktop_app;
mod desktop_config;
mod diagnostics_log;
mod island_window;
#[cfg(windows)]
mod registry_consent_store;
mod settings_window;
mod shell_events;
mod shortcuts;
mod tray;

pub use desktop_app::TikiDesktopApp;
