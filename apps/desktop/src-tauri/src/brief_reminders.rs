use std::thread;

use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// Sends the morning brief and the end-of-day close as Windows notifications. The core
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

    fn send_due(app: &AppHandle) {
        let now = Local::now().naive_local();
        let due = app
            .state::<AppState>()
            .with_workspace(|workspace| workspace.take_due_notification(now));
        match due {
            Ok(Some(notification)) => {
                let shown = app
                    .notification()
                    .builder()
                    .title(&notification.title)
                    .body(&notification.body)
                    .show();
                if let Err(error) = shown {
                    tracing::warn!(%error, "could not show the brief notification");
                }
                ShellEvent::BriefReady.emit(app);
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "could not prepare the brief notification"),
        }
    }
}
