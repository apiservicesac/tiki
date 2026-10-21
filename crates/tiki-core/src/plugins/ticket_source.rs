use chrono::NaiveDateTime;

use super::{ExternalTicket, PluginFuture};
use crate::domain::ExternalReference;

/// Type 1 connection: where the user's tickets and project tasks come from.
pub trait TicketSource: Send + Sync {
    /// Tickets assigned to the user that changed after `since` (all of them when `None`).
    fn list_assigned(&self, since: Option<NaiveDateTime>) -> PluginFuture<'_, Vec<ExternalTicket>>;

    /// Posts a comment on the ticket. Callers only do this after the user approves it.
    fn comment<'call>(
        &'call self,
        reference: &'call ExternalReference,
        message: &'call str,
    ) -> PluginFuture<'call, ()>;

    /// Moves the ticket to another stage. Callers only do this after the user approves it.
    fn set_stage<'call>(
        &'call self,
        reference: &'call ExternalReference,
        stage: &'call str,
    ) -> PluginFuture<'call, ()>;
}
