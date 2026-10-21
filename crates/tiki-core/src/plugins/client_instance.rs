use serde::{Deserialize, Serialize};

use super::{PluginFuture, ReadQuery, Record};

/// What a client instance reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub version: String,
    pub installed_modules: Vec<String>,
}

/// Type 2 connection: a client's system, opened read-only while investigating a ticket.
///
/// The trait has no write methods on purpose. Implementations must also refuse to
/// connect when the configured user is able to write (see `verify_read_only`).
pub trait ClientInstance: Send + Sync {
    /// `Ok(true)` only when the user can read but cannot create, edit or delete.
    fn verify_read_only(&self) -> PluginFuture<'_, bool>;

    fn describe(&self) -> PluginFuture<'_, InstanceInfo>;

    /// Runs a read-only query. Every call is recorded by the caller for the audit trail.
    fn query<'call>(&'call self, query: &'call ReadQuery) -> PluginFuture<'call, Vec<Record>>;
}
