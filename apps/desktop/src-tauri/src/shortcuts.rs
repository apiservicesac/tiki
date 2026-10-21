use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{Builder, Shortcut, ShortcutEvent, ShortcutState};

use crate::desktop_config::DesktopConfig;
use crate::island_window::IslandWindow;
use crate::shell_events::ShellEvent;

/// Keyboard shortcuts that work from any application.
pub struct GlobalShortcuts;

impl GlobalShortcuts {
    pub fn plugin() -> Result<TauriPlugin<Wry>, tauri_plugin_global_shortcut::Error> {
        Ok(Builder::new()
            .with_shortcuts([
                DesktopConfig::SHORTCUT_QUICK_CAPTURE,
                DesktopConfig::SHORTCUT_TOGGLE_ISLAND,
            ])?
            .with_handler(Self::handle)
            .build())
    }

    fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if Self::matches(shortcut, DesktopConfig::SHORTCUT_QUICK_CAPTURE) {
            if let Some(island) = IslandWindow::find(app) {
                island.bring_forward();
            }
            ShellEvent::OpenCapture.emit(app);
        } else if Self::matches(shortcut, DesktopConfig::SHORTCUT_TOGGLE_ISLAND) {
            ShellEvent::ToggleIsland.emit(app);
        }
    }

    fn matches(shortcut: &Shortcut, definition: &str) -> bool {
        definition
            .parse::<Shortcut>()
            .is_ok_and(|expected| expected.id() == shortcut.id())
    }
}
