use serde::Serialize;
use tauri::{AppHandle, State};
use tiki_core::ai::AiProvider;

use super::command_error::CommandResult;
use crate::ai::{AiCredentials, AiProxy, ProxiedRequest, ProxiedResponse};
use crate::shell_events::ShellEvent;

/// Whether each provider has a key; the key itself is never sent to the interface.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    provider: AiProvider,
    configured: bool,
}

#[tauri::command]
pub fn ai_status() -> CommandResult<Vec<ProviderStatus>> {
    let mut statuses = Vec::new();
    for provider in AiProvider::ALL {
        statuses.push(ProviderStatus {
            provider,
            configured: AiCredentials::key(provider)?.is_some(),
        });
    }
    Ok(statuses)
}

#[tauri::command]
pub fn save_ai_key(app: AppHandle, provider: AiProvider, api_key: String) -> CommandResult<()> {
    AiCredentials::save(provider, &api_key)?;
    ShellEvent::PreferencesChanged.emit(&app);
    Ok(())
}

#[tauri::command]
pub fn remove_ai_key(app: AppHandle, provider: AiProvider) -> CommandResult<()> {
    AiCredentials::remove(provider)?;
    ShellEvent::PreferencesChanged.emit(&app);
    Ok(())
}

/// The `fetch` the official SDKs use: the shell adds the key and sends the request.
#[tauri::command]
pub async fn ai_fetch(
    proxy: State<'_, AiProxy>,
    request: ProxiedRequest,
) -> CommandResult<ProxiedResponse> {
    Ok(proxy.send(request).await?)
}
