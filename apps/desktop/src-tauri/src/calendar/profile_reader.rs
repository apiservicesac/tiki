use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use tiki_core::calendar::CalendarProvider;
use tiki_core::{TikiError, TikiResult};

use super::provider_answer::ProviderAnswer;
use crate::desktop_config::DesktopConfig;

/// The name and small picture a provider shows for the signed-in user.
pub struct AccountProfile {
    pub display_name: String,
    /// A `data:` URL, when the provider has a picture.
    pub avatar: Option<String>,
}

/// Google's user info. `name` and `picture` need the `profile` scope, which accounts
/// connected before Tiki asked for it lack until they sign in again.
#[derive(Deserialize)]
struct GoogleUserInfo {
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphUser {
    display_name: Option<String>,
}

/// Reads the signed-in user's name and picture from Google or Microsoft Graph.
pub struct ProfileReader<'http> {
    http: &'http reqwest::Client,
}

impl<'http> ProfileReader<'http> {
    const GOOGLE_USERINFO: &'static str = "https://openidconnect.googleapis.com/v1/userinfo";
    /// Google's pictures live here and take their size after `=`: `=s64-c` is 64 pixels,
    /// cropped square.
    const GOOGLE_PICTURE_HOST: &'static str = "googleusercontent.com";
    const MICROSOFT_ME: &'static str = "https://graph.microsoft.com/v1.0/me?$select=displayName";
    const MICROSOFT_PHOTOS: &'static str = "https://graph.microsoft.com/v1.0/me/photos";
    /// The type of a picture sent without one: both providers send JPEG.
    const PICTURE_TYPE: &'static str = "image/jpeg";

    pub fn new(http: &'http reqwest::Client) -> Self {
        Self { http }
    }

    pub async fn read(
        &self,
        provider: CalendarProvider,
        access_token: &str,
    ) -> TikiResult<AccountProfile> {
        match provider {
            CalendarProvider::Google => {
                let info: GoogleUserInfo = self.json(Self::GOOGLE_USERINFO, access_token).await?;
                let avatar = match info.picture {
                    Some(url) => {
                        Self::picture(self.http.get(Self::sized_google_picture(&url))).await?
                    }
                    None => None,
                };
                Ok(AccountProfile {
                    display_name: info.name.unwrap_or_default(),
                    avatar,
                })
            }
            CalendarProvider::Microsoft => {
                let user: GraphUser = self.json(Self::MICROSOFT_ME, access_token).await?;
                let size = DesktopConfig::AVATAR_PIXELS;
                let photo = format!("{}/{size}x{size}/$value", Self::MICROSOFT_PHOTOS);
                Ok(AccountProfile {
                    display_name: user.display_name.unwrap_or_default(),
                    avatar: Self::picture(self.http.get(photo).bearer_auth(access_token)).await?,
                })
            }
        }
    }

    /// The Google picture at Tiki's avatar size.
    fn sized_google_picture(url: &str) -> String {
        if !url.contains(Self::GOOGLE_PICTURE_HOST) {
            return url.to_owned();
        }
        let base = url.split_once('=').map_or(url, |(base, _)| base);
        format!("{base}=s{}-c", DesktopConfig::AVATAR_PIXELS)
    }

    /// The picture as a `data:` URL; `None` when there is none (Microsoft answers 404).
    async fn picture(request: reqwest::RequestBuilder) -> TikiResult<Option<String>> {
        let response = request.send().await.map_err(|error| Self::error(&error))?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or(Self::PICTURE_TYPE)
            .to_owned();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Self::error(&error))?;
        Ok(Some(format!(
            "data:{content_type};base64,{}",
            STANDARD.encode(bytes)
        )))
    }

    async fn json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        access_token: &str,
    ) -> TikiResult<T> {
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| Self::error(&error))?;
        let body = ProviderAnswer::body(response)
            .await
            .map_err(|error| Self::error(&error))?;
        serde_json::from_str(&body).map_err(|error| Self::error(&error))
    }

    fn error(error: &impl std::fmt::Display) -> TikiError {
        TikiError::Calendar(format!("account profile: {error}"))
    }
}
