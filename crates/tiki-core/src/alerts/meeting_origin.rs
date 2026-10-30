use serde::Serialize;

use crate::calendar::{AccountIdentity, MeetingPlatform};

/// Whose meeting an alert announces and where its call runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingOrigin {
    pub account: AccountIdentity,
    pub platform: Option<MeetingPlatform>,
    /// Both in one line, already translated: "Google Meet · Ana Pérez · ana@acme.com".
    pub label: String,
}
