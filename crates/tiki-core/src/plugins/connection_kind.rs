use serde::{Deserialize, Serialize};

/// The two kinds of connection a plugin can offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionKind {
    /// Type 1: tickets and project tasks assigned to the user.
    TicketSource,
    /// Type 2: a client's system, opened read-only for analysis.
    ClientInstance,
}
