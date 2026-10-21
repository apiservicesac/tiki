//! Plugin contracts. A plugin offers one or both connection kinds:
//!
//! - [`TicketSource`]: where the user's tickets and project tasks come from.
//! - [`ClientInstance`]: a client's system that Tiki inspects, read-only, while investigating.
//!
//! The rest of the app only knows these traits, so adding Slack or Jira never touches the
//! island, the brief or the investigator.

mod client_instance;
mod connection_config;
mod connection_kind;
mod external_ticket;
mod plugin;
mod plugin_future;
mod plugin_manifest;
mod plugin_registry;
mod read_query;
mod ticket_source;

pub use client_instance::{ClientInstance, InstanceInfo};
pub use connection_config::ConnectionConfig;
pub use connection_kind::ConnectionKind;
pub use external_ticket::ExternalTicket;
pub use plugin::Plugin;
pub use plugin_future::PluginFuture;
pub use plugin_manifest::PluginManifest;
pub use plugin_registry::PluginRegistry;
pub use read_query::{FilterOperator, QueryFilter, ReadQuery, Record};
pub use ticket_source::TicketSource;
