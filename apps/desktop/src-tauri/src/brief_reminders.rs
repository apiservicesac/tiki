use std::thread;

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tiki_core::alerts::TikiAlert;
use tiki_core::workspace::TikiWorkspace;

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// Delivers every alert (the morning brief, the end-of-day close, item reminders, follow-up
/// nudges, meetings starting soon and the offer to record a meeting that began): always to
/// the island, which shows them in the notch, and as Windows notifications when the core
/// says so (see `AlertDelivery`). The core decides when each one is due, what it says and
/// how it reaches the user; this loop only asks it every minute. The same minute also lets the core delete the trash past its
/// retention, which it does on start and then once a day.
pub struct BriefReminders;

impl BriefReminders {
    pub fn start(app: AppHandle) {
        thread::spawn(move || {
            loop {
                Self::send_due(&app);
                Self::purge_expired_trash(&app);
                thread::sleep(DesktopConfig::REMINDER_CHECK_INTERVAL);
            }
        });
    }

    fn purge_expired_trash(app: &AppHandle) {
        match app
            .state::<AppState>()
            .with_workspace(TikiWorkspace::purge_expired_trash)
        {
            Ok(0) => {}
            Ok(purged) => tracing::info!(purged, "expired trash deleted"),
            Err(error) => tracing::warn!(%error, "could not delete the expired trash"),
        }
    }

    fn send_due(app: &AppHandle) {
        let due = app.state::<AppState>().with_workspace(|workspace| {
            let mut alerts = workspace.take_due_alerts()?;
            alerts.extend(workspace.take_recording_suggestion()?);
            Ok((alerts, workspace.now()))
        });
        match due {
            Ok((alerts, now)) => {
                for due in alerts {
                    let delivery = due.delivery;
                    tracing::info!(kind = ?due.alert.kind, %now, ?delivery, "alert due");
                    if delivery.windows_notification
                        && let Err(error) = Self::show(app, &due.alert)
                    {
                        tracing::warn!(%error, "could not show the Windows notification");
                    }
                    ShellEvent::AlertReady.emit_with(app, &due);
                }
            }
            Err(error) => tracing::warn!(%error, "could not prepare the alerts"),
        }
    }

    fn show(app: &AppHandle, alert: &TikiAlert) -> tauri_plugin_notification::Result<()> {
        app.notification()
            .builder()
            .title(&alert.title)
            .body(alert.notification_body())
            .show()
    }
}
