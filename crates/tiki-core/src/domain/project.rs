use serde::{Deserialize, Serialize};

use super::ProjectId;

/// What a group of work belongs to: a client, a product, a team or personal life.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<ProjectId>,
    pub name: String,
    /// Color shown next to the project's tasks, as a CSS hex value.
    pub color: String,
    /// Context the user keeps for the project (who is who, how they work); Tiki's AI reads it.
    pub notes: String,
}

impl Project {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            color: color.into(),
            notes: String::new(),
        }
    }
}
