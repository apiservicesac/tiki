use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// How to reach one connection. Secrets are never stored here: `credential_key`
/// names the entry in the operating system's credential store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub connection_id: String,
    pub display_name: String,
    pub base_url: String,
    pub username: String,
    pub credential_key: String,
    /// Plugin-specific values, for example the Odoo database name.
    pub options: BTreeMap<String, String>,
}
