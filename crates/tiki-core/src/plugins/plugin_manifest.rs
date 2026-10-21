use serde::Serialize;

use super::ConnectionKind;

/// What a plugin is and which connection kinds it offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginManifest {
    /// Stable identifier, for example `odoo`.
    pub id: String,
    pub display_name: String,
    pub connection_kinds: Vec<ConnectionKind>,
}

impl PluginManifest {
    pub fn offers(&self, kind: ConnectionKind) -> bool {
        self.connection_kinds.contains(&kind)
    }
}
