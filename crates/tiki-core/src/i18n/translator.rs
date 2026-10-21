use super::{Locale, LocaleCatalog};
use crate::error::TikiResult;

/// Turns message keys into text in the user's language.
#[derive(Debug, Clone)]
pub struct Translator {
    locale: Locale,
    catalog: LocaleCatalog,
}

impl Translator {
    pub fn new(locale: Locale) -> TikiResult<Self> {
        Ok(Self {
            locale,
            catalog: LocaleCatalog::load(locale)?,
        })
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn catalog(&self) -> &LocaleCatalog {
        &self.catalog
    }

    /// Text for a key; the key itself when it is missing, so gaps are visible but harmless.
    pub fn text(&self, key: &str) -> String {
        self.catalog
            .messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }

    /// Text for a key with `{name}` placeholders replaced by the given values.
    pub fn format(&self, key: &str, values: &[(&str, &str)]) -> String {
        values.iter().fold(self.text(key), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
    }

    /// Joins items as a spoken list: "a, b y c" / "a, b and c".
    pub fn join_list(&self, items: &[String]) -> String {
        match items {
            [] => String::new(),
            [single] => single.clone(),
            [first_items @ .., last] => format!(
                "{}{}{}",
                first_items.join(&self.text("narration.list_separator")),
                self.text("narration.list_last_separator"),
                last
            ),
        }
    }
}
