use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::domain::{Client, ClientId};
use crate::error::{TikiError, TikiResult};

/// Reads and writes clients.
pub struct ClientRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ClientRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn insert(&self, client: &Client) -> TikiResult<ClientId> {
        self.connection.execute(
            "INSERT INTO clients (name, color) VALUES (?1, ?2)",
            params![client.name, client.color],
        )?;
        Ok(ClientId(self.connection.last_insert_rowid()))
    }

    pub fn update(&self, client: &Client) -> TikiResult<()> {
        let client_id = client.id.ok_or(TikiError::NotFound {
            entity: "client",
            id: 0,
        })?;
        let changed_rows = self.connection.execute(
            "UPDATE clients SET name = ?2, color = ?3 WHERE id = ?1",
            params![client_id.0, client.name, client.color],
        )?;
        if changed_rows == 0 {
            return Err(TikiError::NotFound {
                entity: "client",
                id: client_id.0,
            });
        }
        Ok(())
    }

    /// Removes a client; the database leaves their tasks without client.
    pub fn delete(&self, client_id: ClientId) -> TikiResult<()> {
        self.connection
            .execute("DELETE FROM clients WHERE id = ?1", params![client_id.0])?;
        Ok(())
    }

    pub fn find_by_name(&self, name: &str) -> TikiResult<Option<Client>> {
        let client = self
            .connection
            .query_row(
                "SELECT id, name, color FROM clients WHERE name = ?1 COLLATE NOCASE",
                params![name],
                Self::read_row,
            )
            .optional()?;
        Ok(client)
    }

    pub fn list(&self) -> TikiResult<Vec<Client>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, color FROM clients ORDER BY name")?;
        let clients = statement
            .query_map([], Self::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(clients)
    }

    fn read_row(row: &Row<'_>) -> rusqlite::Result<Client> {
        Ok(Client {
            id: Some(ClientId(row.get("id")?)),
            name: row.get("name")?,
            color: row.get("color")?,
        })
    }
}
