use serde::{Deserialize, Serialize};

use super::CalendarProvider;

/// Who an account belongs to, as Tiki shows it next to what comes from it. The provider is
/// the company behind the account (Google, Microsoft), not one of its services, so a future
/// mail account of the same provider shows the same way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountIdentity {
    pub provider: CalendarProvider,
    /// The address the user signed in with.
    pub email: String,
    /// The name the provider shows; empty until Tiki learns it.
    pub display_name: String,
    /// The provider's small picture as a `data:` URL, kept locally so it shows offline;
    /// `None` until Tiki learns it or when the provider gives none.
    pub avatar: Option<String>,
}
