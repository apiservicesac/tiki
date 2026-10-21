//! Local SQLite storage: works offline and keeps the data on the user's machine.

mod client_repository;
mod database;
mod preferences_repository;
mod schema_migrations;
mod task_repository;
mod task_row_mapper;

pub use client_repository::ClientRepository;
pub use database::Database;
pub use preferences_repository::PreferencesRepository;
pub use task_repository::TaskRepository;
