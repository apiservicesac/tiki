use std::collections::HashMap;

use serde::Deserialize;

use super::Locale;
use crate::error::TikiResult;

/// Names of weekdays and months, Monday first.
#[derive(Debug, Clone, Deserialize)]
pub struct DateNames {
    pub weekdays: Vec<String>,
    pub weekdays_short: Vec<String>,
    pub months: Vec<String>,
}

/// One language file from `/locales`, parsed.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleCatalog {
    pub messages: HashMap<String, String>,
    pub dates: DateNames,
}

impl LocaleCatalog {
    /// Loads the catalog embedded in the binary for the given language.
    pub fn load(locale: Locale) -> TikiResult<Self> {
        let source = match locale {
            Locale::Spanish => include_str!("../../../../locales/es.json"),
            Locale::English => include_str!("../../../../locales/en.json"),
        };
        Ok(serde_json::from_str(source)?)
    }
}
