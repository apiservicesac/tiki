use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Languages Tiki speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    Spanish,
    English,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::Spanish, Locale::English];

    /// ISO 639-1 code, also the name of the catalog file.
    pub fn code(self) -> &'static str {
        match self {
            Locale::Spanish => "es",
            Locale::English => "en",
        }
    }

    pub fn from_code(code: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|locale| locale.code() == code)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "locale",
                value: code.to_owned(),
            })
    }
}
