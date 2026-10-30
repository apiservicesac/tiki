use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A correction the user made to what the AI understood from a sentence. Recent lessons go
/// back to the AI as examples, so it learns the user's words ("Contoso" is Atlas,
/// "next week" means Monday). The fields are kept as the interface sent them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLesson {
    pub sentence: String,
    /// What the AI filled in.
    pub understood: Value,
    /// What the user saved instead.
    pub corrected: Value,
}
