use serde::Deserialize;

/// A new name, color and notes for a project, as the project panel sends them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChange {
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub notes: String,
}
