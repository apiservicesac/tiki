//! Translations shared with the interface: the catalogs live in `/locales/*.json`.

mod catalog;
mod date_formatter;
mod locale;
mod translator;

pub use catalog::{DateNames, LocaleCatalog};
pub use date_formatter::DateFormatter;
pub use locale::Locale;
pub use translator::Translator;
