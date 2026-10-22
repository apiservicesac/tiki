use serde::Deserialize;

/// A new name and color for a project, as the project panel sends them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChange {
    pub name: String,
    pub color: String,
}
