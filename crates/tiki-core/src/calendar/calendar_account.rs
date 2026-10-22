use serde::{Deserialize, Serialize};

use super::CalendarProvider;

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
    pub provider: CalendarProvider,
    /// The address the user signed in with.
    pub email: String,
}
