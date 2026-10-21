use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// AI providers Tiki can use. The interface talks to them with their official SDKs; the
/// shell adds the user's key, and only to requests that go to that provider's API.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiProvider {
    #[default]
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAi,
}

impl AiProvider {
    pub const ALL: [AiProvider; 2] = [AiProvider::Anthropic, AiProvider::OpenAi];

    /// Stable key used in storage, in the credential store and by the interface.
    pub fn key(self) -> &'static str {
        match self {
            AiProvider::Anthropic => "anthropic",
            AiProvider::OpenAi => "openai",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "AI provider",
                value: key.to_owned(),
            })
    }

    /// Every request that may carry this provider's key starts with this prefix.
    pub fn api_prefix(self) -> &'static str {
        match self {
            AiProvider::Anthropic => "https://api.anthropic.com/v1/",
            AiProvider::OpenAi => "https://api.openai.com/v1/",
        }
    }

    /// The provider a request is for, or `None` when the URL belongs to no provider.
    pub fn for_url(url: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|provider| url.starts_with(provider.api_prefix()))
    }

    /// The header that carries the key, as each provider expects it.
    pub fn auth_header(self, api_key: &str) -> (&'static str, String) {
        match self {
            AiProvider::Anthropic => ("x-api-key", api_key.to_owned()),
            AiProvider::OpenAi => ("authorization", format!("Bearer {api_key}")),
        }
    }
}
