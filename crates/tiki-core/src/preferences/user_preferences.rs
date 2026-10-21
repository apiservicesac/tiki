use serde::{Deserialize, Serialize};

use super::IslandPlacement;
use crate::i18n::Locale;

/// Everything the user can change in Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub locale: Locale,
    pub island_placement: IslandPlacement,
}

impl UserPreferences {
    /// Storage keys, one row per preference in the `preferences` table.
    pub const LOCALE_KEY: &'static str = "locale";
    pub const ISLAND_PLACEMENT_KEY: &'static str = "island_placement";
}
