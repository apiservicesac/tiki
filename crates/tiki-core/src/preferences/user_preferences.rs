use serde::{Deserialize, Serialize};

use super::IslandPlacement;
use crate::ai::AiProvider;
use crate::i18n::Locale;

/// Everything the user can change in Settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub locale: Locale,
    pub island_placement: IslandPlacement,
    pub ai_provider: AiProvider,
    /// Model chosen for the provider; `None` uses the interface's default for it.
    pub ai_model: Option<String>,
}

impl UserPreferences {
    /// Storage keys, one row per preference in the `preferences` table.
    pub const LOCALE_KEY: &'static str = "locale";
    pub const ISLAND_PLACEMENT_KEY: &'static str = "island_placement";
    pub const AI_PROVIDER_KEY: &'static str = "ai_provider";
    pub const AI_MODEL_KEY: &'static str = "ai_model";
}
