use std::path::Path;

use rusqlite::Connection;

use super::schema_migrations::SchemaMigrations;
use super::{
    BriefLogRepository, CalendarRepository, CaptureLessonRepository, PreferencesRepository,
    ProjectRepository, Repositories, TaskRepository,
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
        self.repositories().tasks()
    }

    pub fn projects(&self) -> ProjectRepository<'_> {
        self.repositories().projects()
    }

    pub fn preferences(&self) -> PreferencesRepository<'_> {
        self.repositories().preferences()
    }

    pub fn brief_log(&self) -> BriefLogRepository<'_> {
        self.repositories().brief_log()
    }

    pub fn calendar(&self) -> CalendarRepository<'_> {
        self.repositories().calendar()
    }

    pub fn capture_lessons(&self) -> CaptureLessonRepository<'_> {
        self.repositories().capture_lessons()
    }

    /// Runs `work` in one transaction: every change it makes is saved together, or, when it
    /// fails, none is. Transactions do not nest.
    pub fn transaction<T>(
        &self,
        work: impl FnOnce(&Repositories<'_>) -> TikiResult<T>,
    ) -> TikiResult<T> {
        let transaction = self.connection.unchecked_transaction()?;
        let result = work(&Repositories::new(&transaction))?;
        transaction.commit()?;
        Ok(result)
    }

    fn repositories(&self) -> Repositories<'_> {
        Repositories::new(&self.connection)
    }

    fn prepare(mut connection: Connection) -> TikiResult<Self> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        SchemaMigrations::all().to_latest(&mut connection)?;
        Ok(Self { connection })
    }
}
