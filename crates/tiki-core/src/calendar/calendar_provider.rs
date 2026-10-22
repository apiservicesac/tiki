use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Where a calendar lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarProvider {
    /// Microsoft 365 and Outlook.com.
    Microsoft,
    Google,
}

impl CalendarProvider {
    pub const ALL: [CalendarProvider; 2] = [CalendarProvider::Microsoft, CalendarProvider::Google];

    /// Stable key used in storage.
    pub fn key(self) -> &'static str {
        match self {
            CalendarProvider::Microsoft => "microsoft",
            CalendarProvider::Google => "google",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "calendar provider",
                value: key.to_owned(),
            })
    }
}
