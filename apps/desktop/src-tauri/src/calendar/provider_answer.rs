/// Reads what a calendar provider answered. When it refuses, the provider's own explanation
/// (such as Google's "Calendar API has not been used in project …") goes into the error, so
/// the log says why instead of only the status code.
pub struct ProviderAnswer;

impl ProviderAnswer {
    pub async fn body(response: reqwest::Response) -> Result<String, String> {
        let status = response.status();
        let body = response.text().await.map_err(|error| error.to_string())?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(format!("refused ({status}): {body}"))
        }
    }
}
