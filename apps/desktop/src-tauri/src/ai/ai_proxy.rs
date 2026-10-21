use serde::{Deserialize, Serialize};
use tiki_core::ai::AiProvider;
use tiki_core::{TikiError, TikiResult};

use super::ai_credentials::AiCredentials;

/// A request built by an official SDK in the interface, without the API key.
#[derive(Debug, Deserialize)]
pub struct ProxiedRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// The provider's answer, handed back to the SDK unchanged.
#[derive(Debug, Serialize)]
pub struct ProxiedResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// Sends the SDK's requests for it, adding the user's key only for that provider's API.
pub struct AiProxy {
    http: reqwest::Client,
}

impl AiProxy {
    /// Headers the shell sets itself; the interface may not choose them.
    const SHELL_HEADERS: [&'static str; 4] =
        ["x-api-key", "authorization", "host", "content-length"];

    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, request: ProxiedRequest) -> TikiResult<ProxiedResponse> {
        let provider = AiProvider::for_url(&request.url)
            .ok_or_else(|| TikiError::Ai(format!("not an AI provider URL: {}", request.url)))?;
        let api_key = AiCredentials::key(provider)?
            .ok_or_else(|| TikiError::Ai(format!("no API key for {}", provider.key())))?;

        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .map_err(|error| TikiError::Ai(error.to_string()))?;
        let (auth_name, auth_value) = provider.auth_header(&api_key);
        let mut outgoing = self
            .http
            .request(method, &request.url)
            .header(auth_name, auth_value);
        for (name, value) in &request.headers {
            if !Self::SHELL_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
                outgoing = outgoing.header(name, value);
            }
        }
        if let Some(body) = request.body {
            outgoing = outgoing.body(body);
        }

        let response = outgoing.send().await.map_err(|error| Self::error(&error))?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|text| (name.to_string(), text.to_owned()))
            })
            .collect();
        let body = response.text().await.map_err(|error| Self::error(&error))?;
        Ok(ProxiedResponse {
            status,
            headers,
            body,
        })
    }

    fn error(error: &reqwest::Error) -> TikiError {
        TikiError::Ai(error.to_string())
    }
}
