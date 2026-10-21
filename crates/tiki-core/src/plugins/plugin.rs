use super::{ClientInstance, ConnectionConfig, PluginManifest, TicketSource};
use crate::error::{TikiError, TikiResult};

/// A plugin creates connections of the kinds its manifest declares.
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;

    /// Opens a ticket source; plugins that do not offer one keep this refusal.
    fn open_ticket_source(&self, config: &ConnectionConfig) -> TikiResult<Box<dyn TicketSource>> {
        Err(UnsupportedConnection::error(
            self.manifest(),
            config,
            "ticket source",
        ))
    }

    /// Opens a client instance; plugins that do not offer one keep this refusal.
    fn open_client_instance(
        &self,
        config: &ConnectionConfig,
    ) -> TikiResult<Box<dyn ClientInstance>> {
        Err(UnsupportedConnection::error(
            self.manifest(),
            config,
            "client instance",
        ))
    }
}

/// Builds the error returned when a plugin is asked for a kind it does not offer.
struct UnsupportedConnection;

impl UnsupportedConnection {
    fn error(manifest: &PluginManifest, config: &ConnectionConfig, kind: &str) -> TikiError {
        TikiError::Plugin(format!(
            "plugin '{}' cannot open '{}' as a {kind}",
            manifest.id, config.connection_id
        ))
    }
}
