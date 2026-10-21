//! Translations shared with the interface: the catalogs live in `/locales/*.json`.

mod catalog;
mod date_formatter;
mod locale;
mod text_normalizer;
mod translator;

pub use catalog::{CaptureVocabulary, DateNames, LocaleCatalog};
pub use date_formatter::DateFormatter;
pub use locale::Locale;
pub use text_normalizer::TextNormalizer;
pub use translator::Translator;
