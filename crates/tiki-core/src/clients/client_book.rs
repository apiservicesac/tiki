use chrono::NaiveDate;

use super::{ClientChange, ClientSummary};
use crate::domain::{Client, ClientId, Task};
use crate::error::{TikiError, TikiResult};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Creates, changes and removes clients, and lists them with their workload.
pub struct ClientBook<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
}

impl<'context> ClientBook<'context> {
    pub fn new(database: &'context Database, settings: &'context TikiSettings) -> Self {
        Self { database, settings }
    }

    /// Every client, busiest first: pressing work, then open work, then by name.
    pub fn summaries(
        &self,
        open_tasks: &[Task],
        today: NaiveDate,
    ) -> TikiResult<Vec<ClientSummary>> {
        let mut summaries: Vec<ClientSummary> = self
            .database
            .clients()
            .list()?
            .into_iter()
            .filter_map(|client| {
                let client_id = client.id?;
                let tasks = open_tasks
                    .iter()
                    .filter(|task| task.client_id == Some(client_id) && task.status.is_open());
                let (open_count, pressing_count) = tasks.fold((0, 0), |(open, pressing), task| {
                    let is_pressing = task.is_overdue(today)
                        || task.is_due_within(today, self.settings.due_soon_window_days);
                    (open + 1, pressing + usize::from(is_pressing))
                });
                Some(ClientSummary {
                    client_id,
                    name: client.name,
                    color: client.color,
                    open_count,
                    pressing_count,
                })
            })
            .collect();
        summaries.sort_by(|first, second| {
            second
                .pressing_count
                .cmp(&first.pressing_count)
                .then(second.open_count.cmp(&first.open_count))
                .then_with(|| first.name.to_lowercase().cmp(&second.name.to_lowercase()))
        });
        Ok(summaries)
    }

    /// Saves a new client with the next color of the palette.
    pub fn create(&self, name: &str) -> TikiResult<ClientId> {
        let name = self.available_name(name, None)?;
        let existing_clients = self.database.clients().list()?.len();
        let palette = &self.settings.client_colors;
        let color = palette
            .get(existing_clients % palette.len().max(1))
            .cloned()
            .unwrap_or_default();
        self.database.clients().insert(&Client::new(name, color))
    }

    pub fn change(&self, client_id: ClientId, change: ClientChange) -> TikiResult<()> {
        let name = self.available_name(&change.name, Some(client_id))?;
        self.database.clients().update(&Client {
            id: Some(client_id),
            name,
            color: change.color,
        })
    }

    /// Removes a client; their tasks stay, without client.
    pub fn remove(&self, client_id: ClientId) -> TikiResult<()> {
        self.database.clients().delete(client_id)
    }

    /// The trimmed name, if no other client already uses it (ignoring case).
    fn available_name(&self, name: &str, client_id: Option<ClientId>) -> TikiResult<String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TikiError::EmptyClientName);
        }
        let taken = self
            .database
            .clients()
            .find_by_name(name)?
            .is_some_and(|existing| existing.id != client_id);
        if taken {
            return Err(TikiError::DuplicateClientName(name.to_owned()));
        }
        Ok(name.to_owned())
    }
}
