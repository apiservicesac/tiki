//! Commands the interface calls with `invoke`. Each one is a thin call into the core.
//!
//! Tauri needs the modules themselves to be public: re-exporting the functions hides the
//! helper items `generate_handler!` looks for.

// Tauri hands `State`, `AppHandle` and arguments to commands by value.
#![allow(clippy::needless_pass_by_value)]

pub mod ai_commands;
mod command_error;
pub mod island_commands;
pub mod settings_commands;
pub mod system_commands;
pub mod window_commands;
