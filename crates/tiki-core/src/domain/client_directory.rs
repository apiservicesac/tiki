use std::collections::HashMap;

use super::{Client, ClientId};

/// Looks up clients by id without going back to the database.
#[derive(Debug, Clone, Default)]
pub struct ClientDirectory {
    clients_by_id: HashMap<ClientId, Client>,
}

impl ClientDirectory {
    pub fn new(clients: impl IntoIterator<Item = Client>) -> Self {
        let clients_by_id = clients
            .into_iter()
            .filter_map(|client| client.id.map(|id| (id, client)))
            .collect();
        Self { clients_by_id }
    }

    pub fn find(&self, id: ClientId) -> Option<&Client> {
        self.clients_by_id.get(&id)
    }

    /// Name of the client, if the task has one.
    pub fn name_of(&self, id: Option<ClientId>) -> Option<&str> {
        id.and_then(|client_id| self.find(client_id))
            .map(|client| client.name.as_str())
    }

    pub fn clients(&self) -> impl Iterator<Item = &Client> {
        self.clients_by_id.values()
    }
}
