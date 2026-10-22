use keyring_core::{Entry, Error as KeyringError};
use tiki_core::calendar::AccountId;
use tiki_core::{TikiError, TikiResult};

/// Keeps each calendar account's sign-in (its refresh token) in the operating system's
/// credential store, under the account id. It never reaches the database or the interface.
pub struct TokenStore;

impl TokenStore {
    /// Name under which Tiki's entries appear in the credential store.
    const SERVICE: &'static str = "Tiki";

    pub fn save(account_id: AccountId, refresh_token: &str) -> TikiResult<()> {
        Self::entry(account_id)?
            .set_password(refresh_token)
            .map_err(|error| Self::error(&error))
    }

    /// The stored sign-in, or `None` when the account has none.
    pub fn refresh_token(account_id: AccountId) -> TikiResult<Option<String>> {
        match Self::entry(account_id)?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(Self::error(&error)),
        }
    }

    pub fn remove(account_id: AccountId) -> TikiResult<()> {
        match Self::entry(account_id)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(Self::error(&error)),
        }
    }

    fn entry(account_id: AccountId) -> TikiResult<Entry> {
        Entry::new(Self::SERVICE, &format!("calendar-{}", account_id.0))
            .map_err(|error| Self::error(&error))
    }

    fn error(error: &KeyringError) -> TikiError {
        TikiError::Calendar(format!("credential store: {error}"))
    }
}
