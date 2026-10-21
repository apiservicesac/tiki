use serde::Serialize;

use crate::briefing::BriefMoment;

/// A notification ready to show: already translated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BriefNotification {
    pub moment: BriefMoment,
    pub title: String,
    pub body: String,
}
