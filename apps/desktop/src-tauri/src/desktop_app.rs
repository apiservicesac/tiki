use std::sync::Arc;

use tauri::{App, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tiki_core::i18n::Translator;

use crate::ai::{AiCredentials, AiProxy};
use crate::app_state::AppState;
use crate::app_updates::AppUpdates;
use crate::brief_reminders::BriefReminders;
use crate::calendar::CalendarSync;
#[cfg(windows)]
use crate::call_watch::CallWatch;
use crate::click_through::ClickThrough;
use crate::commands;
use crate::desktop_config::DesktopConfig;
use crate::diagnostics_log::DiagnosticsLog;
use crate::island_window::IslandWindow;
use crate::session_files::SessionFiles;
use crate::session_recorder::SessionRecorder;
use crate::session_retention::SessionRetention;
use crate::transcription_runner::TranscriptionRunner;
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
        tauri::Builder::default()
            .plugin(tauri_plugin_single_instance::init(
                |app, _arguments, _working_folder| {
                    if let Some(island) = IslandWindow::find(app) {
                        island.bring_forward();
                    }
                },
            ))
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                None,
            ))
            .invoke_handler(tauri::generate_handler![
                commands::island_commands::island_snapshot,
                commands::island_commands::move_task,
                commands::island_commands::snooze,
                commands::island_commands::save_confirmed_capture,
                commands::island_commands::project_names,
                commands::island_commands::task_details,
                commands::island_commands::edit_task,
                commands::island_commands::assistant_context,
                commands::island_commands::delivered_tasks,
                commands::island_commands::record_capture_lesson,
                commands::island_commands::capture_lessons,
                commands::project_commands::project_summaries,
                commands::project_commands::create_project,
                commands::project_commands::change_project,
                commands::project_commands::remove_project,
                commands::project_commands::project_colors,
                commands::settings_commands::preferences,
                commands::settings_commands::update_preferences,
                commands::window_commands::set_island_hit_area,
                commands::window_commands::open_advanced_settings,
                commands::system_commands::autostart_enabled,
                commands::system_commands::set_autostart,
                commands::system_commands::app_version,
                commands::system_commands::check_for_update,
                commands::system_commands::install_update,
                commands::calendar_commands::calendar_status,
                commands::calendar_commands::connect_calendar,
                commands::calendar_commands::disconnect_calendar,
                commands::calendar_commands::sync_calendars,
                commands::calendar_commands::open_meeting_link,
                commands::session_commands::start_recording,
                commands::session_commands::pause_recording,
                commands::session_commands::resume_recording,
                commands::session_commands::stop_recording,
                commands::session_commands::sessions,
                commands::session_commands::session_details,
                commands::session_commands::search_sessions,
                commands::session_commands::delete_session,
                commands::session_commands::retry_transcription,
                commands::session_commands::session_storage,
                commands::session_commands::save_session_notes,
                commands::session_commands::apply_session_task,
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
        DiagnosticsLog::start(&app.path().app_log_dir()?)?;
        tracing::info!(version = %app.package_info().version, "Tiki started");
        let app_data_folder = app.path().app_data_dir()?;
        let state = AppState::open(&app_data_folder)?;
        let preferences = state.with_workspace(|workspace| Ok(workspace.preferences().clone()))?;
        app.manage(state);
        app.manage(SessionFiles::new(&app_data_folder));
        app.manage(SessionRecorder::default());
        app.manage(TranscriptionRunner::start(
            app.handle().clone(),
            &app_data_folder.join(DesktopConfig::MODELS_FOLDER),
        )?);
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
        BriefReminders::start(app.handle().clone());
        CalendarSync::start(app.handle().clone());
        SessionRetention::start(app.handle().clone());
        #[cfg(windows)]
        CallWatch::start(app.handle().clone());
        AppUpdates::watch(app.handle().clone(), Translator::new(preferences.locale)?);
        Ok(())
    }
}
