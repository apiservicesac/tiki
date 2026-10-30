use serde::{Deserialize, Serialize};

use crate::sessions::SessionId;

/// Points to the record a task came from in an external system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalReference {
    /// Plugin that created the task, for example `odoo`.
    pub plugin_id: String,
    /// Connection inside that plugin, for example the Andeanvet instance.
    pub connection_id: String,
    /// Identifier of the record in the external system.
    pub record_id: String,
    /// Link to open the record in a browser, when the system has one.
    pub url: Option<String>,
}

/// Where a task was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskOrigin {
    /// Typed by the user (quick capture, voice or the task form).
    Manual,
    /// Imported from a ticket source plugin.
    External(ExternalReference),
    /// Proposed by the AI from a recorded session and applied by the user.
    Session { session_id: SessionId },
}
