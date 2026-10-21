use serde::Serialize;
use tauri::State;
use tiki_core::ai::AiProvider;

use super::command_error::CommandResult;
use crate::ai::{AiCredentials, AiProxy, ProxiedRequest, ProxiedResponse};

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
pub fn save_ai_key(provider: AiProvider, api_key: String) -> CommandResult<()> {
    Ok(AiCredentials::save(provider, &api_key)?)
}

#[tauri::command]
pub fn remove_ai_key(provider: AiProvider) -> CommandResult<()> {
    Ok(AiCredentials::remove(provider)?)
}

/// The `fetch` the official SDKs use: the shell adds the key and sends the request.
#[tauri::command]
pub async fn ai_fetch(
    proxy: State<'_, AiProxy>,
    request: ProxiedRequest,
) -> CommandResult<ProxiedResponse> {
    Ok(proxy.send(request).await?)
}
