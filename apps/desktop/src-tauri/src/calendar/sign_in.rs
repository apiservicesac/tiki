use std::time::Duration;

use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tiki_core::calendar::CalendarProvider;
use tiki_core::{TikiError, TikiResult};
use url::Url;

use super::loopback_receiver::LoopbackReceiver;
use super::oauth_client::OAuthClient;
use super::pkce::Pkce;

/// What a token endpoint answers.
#[derive(Debug, Deserialize)]
pub struct TokenAnswer {
    pub access_token: String,
    /// Some providers rotate it on every refresh; when absent, the old one stays valid.
    pub refresh_token: Option<String>,
}

/// The result of signing in: who the user is and the sign-in to keep.
pub struct SignedIn {
    pub email: String,
    pub refresh_token: String,
}

/// Signs in to a calendar provider in the user's browser and exchanges tokens.
pub struct SignIn {
    http: reqwest::Client,
}

impl SignIn {
    /// How long Tiki waits for the user to finish signing in in the browser.
    const TIMEOUT: Duration = Duration::from_secs(300);
    const USERINFO_MICROSOFT: &'static str =
        "https://graph.microsoft.com/v1.0/me?$select=mail,userPrincipalName";
    const USERINFO_GOOGLE: &'static str = "https://openidconnect.googleapis.com/v1/userinfo";

    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    /// Opens the provider's sign-in page and waits for the user to come back.
    pub async fn sign_in(
        &self,
        app: &AppHandle,
        client: OAuthClient,
        done_page: String,
    ) -> TikiResult<SignedIn> {
        let pkce = Pkce::new().map_err(|error| Self::error(&error))?;
        let receiver = LoopbackReceiver::bind()?;
        let redirect_uri = receiver.redirect_uri()?;
        let mut authorize =
            Url::parse(client.authorize_url).map_err(|error| Self::error(&error))?;
        authorize
            .query_pairs_mut()
            .append_pair("client_id", client.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("scope", client.scopes)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &pkce.state);
        // Google gives a lasting sign-in only with these two; Microsoft uses offline_access.
        match client.provider {
            CalendarProvider::Google => {
                authorize
                    .query_pairs_mut()
                    .append_pair("access_type", "offline")
                    .append_pair("prompt", "consent select_account");
            }
            CalendarProvider::Microsoft => {
                authorize
                    .query_pairs_mut()
                    .append_pair("prompt", "select_account");
            }
        }
        app.opener()
            .open_url(authorize.as_str(), None::<&str>)
            .map_err(|error| Self::error(&error))?;

        let state = pkce.state.clone();
        let code = tauri::async_runtime::spawn_blocking(move || {
            receiver.wait_for_code(&state, Self::TIMEOUT, &done_page)
        })
        .await
        .map_err(|error| Self::error(&error))?
        .map_err(TikiError::Calendar)?;

        let tokens = self
            .token_request(
                client,
                &[
                    ("grant_type", "authorization_code"),
                    ("code", &code),
                    ("redirect_uri", &redirect_uri),
                    ("code_verifier", &pkce.verifier),
                ],
            )
            .await?;
        let refresh_token = tokens.refresh_token.clone().ok_or_else(|| {
            TikiError::Calendar("the provider gave no lasting sign-in".to_owned())
        })?;
        let email = self.email(client.provider, &tokens.access_token).await?;
        Ok(SignedIn {
            email,
            refresh_token,
        })
    }

    /// A fresh access token from a stored sign-in.
    pub async fn refresh(
        &self,
        client: OAuthClient,
        refresh_token: &str,
    ) -> TikiResult<TokenAnswer> {
        self.token_request(
            client,
            &[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ],
        )
        .await
    }

    async fn token_request(
        &self,
        client: OAuthClient,
        fields: &[(&str, &str)],
    ) -> TikiResult<TokenAnswer> {
        let body = Self::form_body(client, fields);
        let response = self
            .http
            .post(client.token_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|error| Self::error(&error))?;
        let status = response.status();
        let body = response.text().await.map_err(|error| Self::error(&error))?;
        if !status.is_success() {
            return Err(TikiError::Calendar(format!(
                "sign-in refused ({status}): {body}"
            )));
        }
        serde_json::from_str(&body).map_err(|error| Self::error(&error))
    }

    /// The form fields of a token request. Built apart, so nothing that cannot cross
    /// threads lives across an await.
    fn form_body(client: OAuthClient, fields: &[(&str, &str)]) -> String {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.append_pair("client_id", client.client_id);
        if let Some(secret) = client.client_secret {
            form.append_pair("client_secret", secret);
        }
        if client.provider == CalendarProvider::Microsoft {
            form.append_pair("scope", client.scopes);
        }
        for (name, value) in fields {
            form.append_pair(name, value);
        }
        form.finish()
    }

    async fn email(&self, provider: CalendarProvider, access_token: &str) -> TikiResult<String> {
        let url = match provider {
            CalendarProvider::Microsoft => Self::USERINFO_MICROSOFT,
            CalendarProvider::Google => Self::USERINFO_GOOGLE,
        };
        let body = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| Self::error(&error))?
            .text()
            .await
            .map_err(|error| Self::error(&error))?;
        let profile: serde_json::Value =
            serde_json::from_str(&body).map_err(|error| Self::error(&error))?;
        ["mail", "email", "userPrincipalName"]
            .iter()
            .find_map(|field| profile[field].as_str().filter(|value| !value.is_empty()))
            .map(str::to_owned)
            .ok_or_else(|| TikiError::Calendar("the provider did not say who signed in".to_owned()))
    }

    fn error(error: &impl std::fmt::Display) -> TikiError {
        TikiError::Calendar(error.to_string())
    }
}
