use std::thread;

use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tiki_core::reminders::BriefNotification;

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// Delivers the morning brief and the end-of-day close: always to the island, which shows
/// them in the notch, and as a Windows notification when the user wants that too. The core
/// decides when each one is due and what it says; this loop only asks it every minute.
pub struct BriefReminders;

impl BriefReminders {
    pub fn start(app: AppHandle) {
        thread::spawn(move || {
            loop {
                Self::send_due(&app);
                thread::sleep(DesktopConfig::REMINDER_CHECK_INTERVAL);
            }
        });
    }

    /// Shows a brief as a Windows notification.
    fn show(
        app: &AppHandle,
        notification: &BriefNotification,
    ) -> tauri_plugin_notification::Result<()> {
        app.notification()
            .builder()
            .title(&notification.title)
            .body(&notification.body)
            .show()
    }

    fn send_due(app: &AppHandle) {
        let now = Local::now().naive_local();
        let due = app.state::<AppState>().with_workspace(|workspace| {
            let notification = workspace.take_due_notification(now)?;
            Ok(notification
                .map(|notification| (notification, workspace.preferences().windows_notifications)))
        });
        match due {
            Ok(Some((notification, also_in_windows))) => {
                tracing::info!(moment = ?notification.moment, %now, also_in_windows, "brief due");
                if also_in_windows && let Err(error) = Self::show(app, &notification) {
                    tracing::warn!(%error, "could not show the Windows notification");
                }
                ShellEvent::BriefReady.emit_with(app, &notification);
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "could not prepare the brief notification"),
        }
    }
}
