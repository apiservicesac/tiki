use rusqlite::Connection;

use super::{
    ActivityRepository, BriefLogRepository, CalendarRepository, CaptureLessonRepository,
    PreferencesRepository, ProjectRepository, TaskRepository,
};

/// Every repository over one connection: an open transaction (see `Database::transaction`).
pub struct Repositories<'connection> {
    connection: &'connection Connection,
}

impl<'connection> Repositories<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn tasks(&self) -> TaskRepository<'connection> {
        TaskRepository::new(self.connection)
    }

    pub fn projects(&self) -> ProjectRepository<'connection> {
        ProjectRepository::new(self.connection)
    }

    pub fn preferences(&self) -> PreferencesRepository<'connection> {
        PreferencesRepository::new(self.connection)
    }

    pub fn brief_log(&self) -> BriefLogRepository<'connection> {
        BriefLogRepository::new(self.connection)
    }

    pub fn calendar(&self) -> CalendarRepository<'connection> {
        CalendarRepository::new(self.connection)
    }

    pub fn activity(&self) -> ActivityRepository<'connection> {
        ActivityRepository::new(self.connection)
    }

    pub fn capture_lessons(&self) -> CaptureLessonRepository<'connection> {
        CaptureLessonRepository::new(self.connection)
    }
}
