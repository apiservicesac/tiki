use std::path::Path;

use rusqlite::Connection;

use super::schema_migrations::SchemaMigrations;
use super::{
    CaptureLessonRepository, PreferencesRepository, ProjectRepository, ReminderLogRepository,
    TaskRepository,
};
use crate::error::TikiResult;

/// An open, migrated SQLite database.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens (or creates) the database file and brings its schema up to date.
    pub fn open(path: &Path) -> TikiResult<Self> {
        Self::prepare(Connection::open(path)?)
    }

    /// A throwaway database, used by tests and previews.
    pub fn open_in_memory() -> TikiResult<Self> {
        Self::prepare(Connection::open_in_memory()?)
    }

    pub fn tasks(&self) -> TaskRepository<'_> {
        TaskRepository::new(&self.connection)
    }

    pub fn projects(&self) -> ProjectRepository<'_> {
        ProjectRepository::new(&self.connection)
    }

    pub fn preferences(&self) -> PreferencesRepository<'_> {
        PreferencesRepository::new(&self.connection)
    }

    pub fn reminder_log(&self) -> ReminderLogRepository<'_> {
        ReminderLogRepository::new(&self.connection)
    }

    pub fn capture_lessons(&self) -> CaptureLessonRepository<'_> {
        CaptureLessonRepository::new(&self.connection)
    }

    fn prepare(mut connection: Connection) -> TikiResult<Self> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        SchemaMigrations::all().to_latest(&mut connection)?;
        Ok(Self { connection })
    }
}
