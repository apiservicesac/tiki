use std::sync::Arc;

use tauri::{App, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tiki_core::i18n::Translator;

use crate::ai::{AiCredentials, AiProxy};
use crate::app_state::AppState;
use crate::click_through::ClickThrough;
use crate::commands;
use crate::island_window::IslandWindow;
use crate::shortcuts::GlobalShortcuts;
use crate::tray::IslandTray;

/// Builds and runs the desktop application.
pub struct TikiDesktopApp;

impl TikiDesktopApp {
    /// Starts the event loop; returns when the user quits.
    ///
    /// # Panics
    ///
    /// When the operating system refuses to create the window or the tray.
    pub fn run() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        tauri::Builder::default()
            .plugin(tauri_plugin_single_instance::init(
                |app, _arguments, _working_folder| {
                    if let Some(island) = IslandWindow::find(app) {
                        island.bring_forward();
                    }
                },
            ))
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                None,
            ))
            .invoke_handler(tauri::generate_handler![
                commands::island_commands::island_snapshot,
                commands::island_commands::preview_capture,
                commands::island_commands::capture,
                commands::island_commands::move_task,
                commands::settings_commands::preferences,
                commands::settings_commands::update_preferences,
                commands::window_commands::set_island_hit_area,
                commands::ai_commands::ai_status,
                commands::ai_commands::save_ai_key,
                commands::ai_commands::remove_ai_key,
                commands::ai_commands::ai_fetch,
            ])
            .setup(|app| Self::setup(app))
            .run(tauri::generate_context!())
            .expect("Tiki could not start");
    }

    fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState::open(&app.path().app_data_dir()?)?;
        let preferences = state.with_workspace(|workspace| Ok(workspace.preferences().clone()))?;
        app.manage(state);
        AiCredentials::use_system_store()?;
        app.manage(AiProxy::new());

        IslandTray::install(app, &Translator::new(preferences.locale)?)?;
        app.handle().plugin(GlobalShortcuts::plugin()?)?;

        if let Some(island) = IslandWindow::find(app.handle()) {
            island.prepare(preferences.island_placement)?;
        }
        let click_through = Arc::new(ClickThrough::default());
        app.manage(Arc::clone(&click_through));
        ClickThrough::start(app.handle().clone(), click_through);
        Ok(())
    }
}
