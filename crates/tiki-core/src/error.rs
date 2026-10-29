use thiserror::Error;

/// Every failure the core can report to its callers.
#[derive(Debug, Error)]
pub enum TikiError {
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("database migration error: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("invalid locale catalog: {0}")]
    LocaleCatalog(#[from] serde_json::Error),

    #[error("unknown {kind} value: {value}")]
    UnknownValue { kind: &'static str, value: String },

    #[error("{entity} with id {id} was not found")]
    NotFound { entity: &'static str, id: i64 },

    #[error("the task has not been saved yet")]
    UnsavedTask,

    #[error("the project has not been saved yet")]
    UnsavedProject,

    #[error("a project needs a name")]
    EmptyProjectName,

    #[error("a project named {0} already exists")]
    DuplicateProjectName(String),

    #[error("a task needs a title")]
    EmptyTaskTitle,

    #[error("a reminder needs a time to ring")]
    ReminderNeedsTime,

    #[error("AI request failed: {0}")]
    Ai(String),

    #[error("calendar error: {0}")]
    Calendar(String),

    #[error("plugin error: {0}")]
    Plugin(String),
}

impl TikiError {
    /// Stable key of the failure, sent to the interface with the readable message so it can
    /// show the user a translated text for the ones they can fix (`error.<key>`).
    pub fn key(&self) -> &'static str {
        match self {
            TikiError::Io(_) => "io",
            TikiError::Database(_) => "database",
            TikiError::Migration(_) => "migration",
            TikiError::LocaleCatalog(_) => "locale_catalog",
            TikiError::UnknownValue { .. } => "unknown_value",
            TikiError::NotFound { .. } => "not_found",
            TikiError::UnsavedTask => "unsaved_task",
            TikiError::UnsavedProject => "unsaved_project",
            TikiError::EmptyProjectName => "empty_project_name",
            TikiError::DuplicateProjectName(_) => "duplicate_project_name",
            TikiError::EmptyTaskTitle => "empty_task_title",
            TikiError::ReminderNeedsTime => "reminder_needs_time",
            TikiError::Ai(_) => "ai",
            TikiError::Calendar(_) => "calendar",
            TikiError::Plugin(_) => "plugin",
        }
    }
}

pub type TikiResult<T> = Result<T, TikiError>;
