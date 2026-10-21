use rusqlite_migration::{M, Migrations};

/// Every schema change, in order. Never edit a published step: add a new one.
pub struct SchemaMigrations;

impl SchemaMigrations {
    pub fn all() -> Migrations<'static> {
        Migrations::new(vec![M::up(Self::INITIAL_SCHEMA), M::up(Self::PREFERENCES)])
    }

    const INITIAL_SCHEMA: &'static str = "
        CREATE TABLE clients (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL
        );

        CREATE TABLE tasks (
            id                INTEGER PRIMARY KEY,
            title             TEXT NOT NULL,
            notes             TEXT NOT NULL DEFAULT '',
            client_id         INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            status            TEXT NOT NULL,
            priority          TEXT NOT NULL,
            due_date          TEXT,
            waiting_since     TEXT,
            origin_plugin     TEXT,
            origin_connection TEXT,
            origin_record     TEXT,
            origin_url        TEXT,
            created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX tasks_by_status ON tasks(status);

        CREATE UNIQUE INDEX tasks_by_external_record
            ON tasks(origin_plugin, origin_connection, origin_record)
            WHERE origin_plugin IS NOT NULL;
    ";

    const PREFERENCES: &'static str = "
        CREATE TABLE preferences (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    ";
}
