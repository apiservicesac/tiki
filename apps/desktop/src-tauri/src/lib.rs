//! Tiki desktop shell. It owns the operating-system pieces (island window, tray,
//! global shortcuts, notifications) and delegates every decision to `tiki-core`.

mod ai;
mod app_state;
mod click_through;
mod commands;
mod desktop_app;
mod desktop_config;
mod island_window;
mod shell_events;
mod shortcuts;
mod tray;

pub use desktop_app::TikiDesktopApp;
