use chrono::{NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};

use super::AccountIdentity;

/// Database identifier of a connected calendar account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountId(pub i64);

/// A calendar account the user connected. Its sign-in tokens never live here: the desktop
/// shell keeps them in the operating system's credential store, under the account id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAccount {
    pub account_id: AccountId,
    #[serde(flatten)]
    pub identity: AccountIdentity,
    /// When the shell last asked the provider for the name and picture.
    #[serde(skip)]
    pub identity_checked_at: Option<NaiveDateTime>,
}

impl CalendarAccount {
    /// Whether the name and picture should be asked again: never asked, or `every` passed.
    pub fn identity_due(&self, now: NaiveDateTime, every: TimeDelta) -> bool {
        self.identity_checked_at
            .is_none_or(|checked_at| now - checked_at >= every)
    }
}
