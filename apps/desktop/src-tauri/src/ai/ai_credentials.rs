use keyring_core::{Entry, Error as KeyringError};
use tiki_core::ai::AiProvider;
use tiki_core::{TikiError, TikiResult};

/// API keys for the AI providers, kept in the operating system's credential store
/// (Windows Credential Manager). They never reach the database or the interface.
pub struct AiCredentials;

impl AiCredentials {
    /// Name under which Tiki's entries appear in the credential store.
    const SERVICE: &'static str = "Tiki";

    /// Selects the operating system's store. Called once at startup.
    pub fn use_system_store() -> TikiResult<()> {
        #[cfg(windows)]
        keyring_core::set_default_store(
            windows_native_keyring_store::Store::new().map_err(|error| Self::error(&error))?,
        );
        Ok(())
    }

    pub fn save(provider: AiProvider, api_key: &str) -> TikiResult<()> {
        Self::entry(provider)?
            .set_password(api_key.trim())
            .map_err(|error| Self::error(&error))
    }

    /// The stored key, or `None` when the user has not added one.
    pub fn key(provider: AiProvider) -> TikiResult<Option<String>> {
        match Self::entry(provider)?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(Self::error(&error)),
        }
    }

    pub fn remove(provider: AiProvider) -> TikiResult<()> {
        match Self::entry(provider)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(Self::error(&error)),
        }
    }

    fn entry(provider: AiProvider) -> TikiResult<Entry> {
        Entry::new(Self::SERVICE, provider.key()).map_err(|error| Self::error(&error))
    }

    fn error(error: &KeyringError) -> TikiError {
        TikiError::Ai(format!("credential store: {error}"))
    }
}
