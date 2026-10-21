use serde::{Deserialize, Serialize};

use super::ClientId;

/// A company the user works for, such as Altamira or Andeanvet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: Option<ClientId>,
    pub name: String,
    /// Color shown next to the client's tasks, as a CSS hex value.
    pub color: String,
}

impl Client {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            color: color.into(),
        }
    }
}
