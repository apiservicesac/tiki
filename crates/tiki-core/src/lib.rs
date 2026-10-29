//! Tiki core: everything that does not depend on a user interface.
//!
//! The desktop shell (Tauri) and any future project (mobile, CLI) only talk to the
//! types exported here, so the same rules run on every platform.

pub mod activity;
pub mod ai;
pub mod alerts;
pub mod briefing;
pub mod calendar;
pub mod capture;
pub mod domain;
pub mod editing;
pub mod error;
pub mod i18n;
pub mod mood;
pub mod plugins;
pub mod preferences;
pub mod projects;
pub mod settings;
pub mod storage;
pub mod workspace;

pub use error::{TikiError, TikiResult};
pub use settings::TikiSettings;
