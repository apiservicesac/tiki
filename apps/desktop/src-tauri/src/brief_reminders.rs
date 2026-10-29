use std::thread;

use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tiki_core::alerts::TikiAlert;

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// Delivers the morning brief, the end-of-day close and task reminders: always to the
/// island, which shows them in the notch, and as Windows notifications when the user wants
/// that too. The core decides when each one is due and what it says; this loop only asks
/// it every minute.
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
        let due = app.state::<AppState>().with_workspace(|workspace| {
            let mut alerts: Vec<TikiAlert> =
                workspace.take_due_notification(now)?.into_iter().collect();
            alerts.extend(workspace.take_due_task_reminders(now)?);
            alerts.extend(workspace.take_due_meeting_alerts(now)?);
            Ok((alerts, workspace.preferences().windows_notifications))
        });
        match due {
            Ok((alerts, also_in_windows)) => {
                for alert in alerts {
                    tracing::info!(kind = ?alert.kind, %now, also_in_windows, "alert due");
                    if also_in_windows && let Err(error) = Self::show(app, &alert) {
                        tracing::warn!(%error, "could not show the Windows notification");
                    }
                    ShellEvent::BriefReady.emit_with(app, &alert);
                }
            }
            Err(error) => tracing::warn!(%error, "could not prepare the alerts"),
        }
    }

    fn show(app: &AppHandle, alert: &TikiAlert) -> tauri_plugin_notification::Result<()> {
        app.notification()
            .builder()
            .title(&alert.title)
            .body(&alert.body)
            .show()
    }
}
