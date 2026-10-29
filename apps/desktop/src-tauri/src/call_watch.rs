use std::thread;

use tauri::{AppHandle, Manager};
use tiki_core::presence::CallDetector;

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::registry_consent_store::RegistryConsentStore;
use crate::shell_events::ShellEvent;

/// Watches for calls: when another app takes or releases the microphone, it tells the core,
/// which keeps alerts quiet meanwhile and may offer to record, and the island, which shows
/// it in the notch.
pub struct CallWatch;

impl CallWatch {
    /// Starts watching for the rest of the session.
    pub fn start(app: AppHandle) {
        let own_executable = std::env::current_exe().unwrap_or_default();
        let detector = CallDetector::new(RegistryConsentStore, &own_executable);
        thread::spawn(move || {
            let mut in_call = false;
            loop {
                match detector.another_app_uses_microphone() {
                    Ok(seen) if seen != in_call => {
                        in_call = seen;
                        Self::report(&app, in_call);
                    }
                    Ok(_) => {}
                    Err(error) => tracing::warn!(%error, "could not read who uses the microphone"),
                }
                thread::sleep(DesktopConfig::CALL_CHECK_INTERVAL);
            }
        });
    }

    fn report(app: &AppHandle, in_call: bool) {
        tracing::info!(in_call, "call started or ended");
        let reported = app.state::<AppState>().with_workspace(|workspace| {
            workspace.set_in_call(in_call);
            workspace.take_recording_suggestion()
        });
        match reported {
            Ok(offer) => {
                ShellEvent::InCallChanged.emit(app);
                if let Some(offer) = offer {
                    ShellEvent::AlertReady.emit_with(app, &offer);
                }
            }
            Err(error) => tracing::warn!(%error, "could not tell the core about the call"),
        }
    }
}
