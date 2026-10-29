use serde::Serialize;
use tiki_core::TikiError;

/// Error sent back to the interface: a stable key (see `TikiError::key`) and a readable
/// message for the logs.
#[derive(Debug, Serialize)]
pub struct CommandError {
    key: &'static str,
    message: String,
}

impl From<TikiError> for CommandError {
    fn from(error: TikiError) -> Self {
        tracing::warn!(%error, "command failed");
        Self {
            key: error.key(),
            message: error.to_string(),
        }
    }
}

impl CommandError {
    /// The key of every failure from the operating system or a plugin.
    const SHELL_KEY: &'static str = "shell";

    /// Wraps an error from the operating system or a plugin.
    pub fn new(error: &impl std::fmt::Display) -> Self {
        tracing::warn!(%error, "command failed");
        Self {
            key: Self::SHELL_KEY,
            message: error.to_string(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
