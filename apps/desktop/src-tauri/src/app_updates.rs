use std::thread;

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;
use tiki_core::i18n::Translator;

use crate::desktop_config::DesktopConfig;

/// A newer version published on GitHub, signed with Tiki's update key.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    pub version: String,
    pub notes: Option<String>,
}

/// Checks for signed updates and installs them when the user asks. The installer replaces
/// Tiki and starts it again; the user's data stays in the app data folder.
pub struct AppUpdates;

impl AppUpdates {
    pub async fn check(app: &AppHandle) -> tauri_plugin_updater::Result<Option<AvailableUpdate>> {
        let update = app.updater()?.check().await?;
        Ok(update.map(|update| AvailableUpdate {
            version: update.version,
            notes: update.body,
        }))
    }

    /// Downloads, verifies the signature and installs the latest version, then restarts.
    pub async fn install(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
        let Some(update) = app.updater()?.check().await? else {
            return Ok(());
        };
        update.download_and_install(|_, _| {}, || {}).await?;
        app.restart();
    }

    /// Checks now and then every few hours; a new version is announced once per check.
    pub fn watch(app: AppHandle, translator: Translator) {
        thread::spawn(move || {
            loop {
                match tauri::async_runtime::block_on(Self::check(&app)) {
                    Ok(Some(update)) => Self::announce(&app, &translator, &update),
                    Ok(None) => {}
                    Err(error) => tracing::info!(%error, "could not check for updates"),
                }
                thread::sleep(DesktopConfig::UPDATE_CHECK_INTERVAL);
            }
        });
    }

    fn announce(app: &AppHandle, translator: &Translator, update: &AvailableUpdate) {
        let shown = app
            .notification()
            .builder()
            .title(translator.format("update.available.title", &[("version", &update.version)]))
            .body(translator.text("update.available.body"))
            .show();
        if let Err(error) = shown {
            tracing::warn!(%error, "could not announce the update");
        }
    }
}
