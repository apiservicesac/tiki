//! The shell's side of the AI assistant: it keeps the API keys and forwards the requests
//! the official SDKs build in the interface. The SDKs never see a key.

mod ai_credentials;
mod ai_proxy;

pub use ai_credentials::AiCredentials;
pub use ai_proxy::{AiProxy, ProxiedRequest, ProxiedResponse};
