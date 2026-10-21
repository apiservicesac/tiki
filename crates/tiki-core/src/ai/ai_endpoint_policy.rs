/// Decides which requests the shell may send with the user's API key attached.
///
/// The interface builds requests with the official SDK; the shell adds the key, so it must
/// only ever forward requests to the Anthropic API and nowhere else.
pub struct AiEndpointPolicy;

impl AiEndpointPolicy {
    /// Every allowed request starts with this exact prefix: scheme, host and API version.
    pub const ALLOWED_PREFIX: &'static str = "https://api.anthropic.com/v1/";

    pub fn is_allowed(url: &str) -> bool {
        url.starts_with(Self::ALLOWED_PREFIX)
    }
}
