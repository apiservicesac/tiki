use serde::{Deserialize, Serialize};

use super::ProjectId;

/// A company the user works for, such as Altamira or Andeanvet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<ProjectId>,
    pub name: String,
    /// Color shown next to the project's tasks, as a CSS hex value.
    pub color: String,
}

impl Project {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            color: color.into(),
        }
    }
}
