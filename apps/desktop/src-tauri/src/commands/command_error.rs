use serde::Serialize;
use tiki_core::TikiError;

/// Error sent back to the interface; it carries a readable message only.
#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

impl From<TikiError> for CommandError {
    fn from(error: TikiError) -> Self {
        tracing::warn!(%error, "command failed");
        Self {
            message: error.to_string(),
        }
    }
}

impl CommandError {
    /// Wraps an error from the operating system or a plugin.
    pub fn new(error: &impl std::fmt::Display) -> Self {
        tracing::warn!(%error, "command failed");
        Self {
            message: error.to_string(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
