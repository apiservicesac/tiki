use tiki_core::calendar::CalendarProvider;

/// How Tiki signs in to a calendar provider. The client ids come from the app registrations
/// (Azure and Google Cloud) and are set when building the installer; a build without them
/// simply cannot connect that provider. They are public by design: desktop apps use PKCE.
#[derive(Debug, Clone, Copy)]
pub struct OAuthClient {
    pub provider: CalendarProvider,
    pub client_id: &'static str,
    /// Google's desktop clients come with a secret that is not secret (it ships in every
    /// copy of the app); Microsoft's public clients have none.
    pub client_secret: Option<&'static str>,
    pub authorize_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static str,
}

impl OAuthClient {
    const MICROSOFT_CLIENT_ID: Option<&'static str> = option_env!("TIKI_MICROSOFT_CLIENT_ID");
    const GOOGLE_CLIENT_ID: Option<&'static str> = option_env!("TIKI_GOOGLE_CLIENT_ID");
    const GOOGLE_CLIENT_SECRET: Option<&'static str> = option_env!("TIKI_GOOGLE_CLIENT_SECRET");

    /// The client for a provider, or `None` when this build has no registration for it.
    pub fn of(provider: CalendarProvider) -> Option<Self> {
        match provider {
            CalendarProvider::Microsoft => Self::MICROSOFT_CLIENT_ID
                .filter(|id| !id.is_empty())
                .map(|client_id| Self {
                    provider,
                    client_id,
                    client_secret: None,
                    authorize_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                    token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                    scopes: "offline_access openid email User.Read Calendars.Read",
                }),
            CalendarProvider::Google => {
                Self::GOOGLE_CLIENT_ID
                    .filter(|id| !id.is_empty())
                    .map(|client_id| Self {
                        provider,
                        client_id,
                        client_secret: Self::GOOGLE_CLIENT_SECRET,
                        authorize_url: "https://accounts.google.com/o/oauth2/v2/auth",
                        token_url: "https://oauth2.googleapis.com/token",
                        scopes: "openid email https://www.googleapis.com/auth/calendar.readonly",
                    })
            }
        }
    }
}
