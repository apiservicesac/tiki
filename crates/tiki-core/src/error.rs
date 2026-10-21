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

    #[error("a client needs a name")]
    EmptyClientName,

    #[error("a task needs a title")]
    EmptyTaskTitle,

    #[error("AI request failed: {0}")]
    Ai(String),

    #[error("plugin error: {0}")]
    Plugin(String),
}

pub type TikiResult<T> = Result<T, TikiError>;
