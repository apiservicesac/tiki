use serde::Deserialize;

/// A new name and color for a client, as the client panel sends them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientChange {
    pub name: String,
    pub color: String,
}
