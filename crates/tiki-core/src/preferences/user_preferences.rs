use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use super::IslandPlacement;
use crate::ai::AiProvider;
use crate::i18n::Locale;
use crate::settings::TikiSettings;

/// Everything the user can change in Settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub locale: Locale,
    pub island_placement: IslandPlacement,
    pub ai_provider: AiProvider,
    /// Model chosen for the provider; `None` uses the interface's default for it.
    pub ai_model: Option<String>,
    /// When the morning brief arrives as a notification.
    pub morning_brief_time: NaiveTime,
    /// When the end-of-day close arrives as a notification.
    pub evening_brief_time: NaiveTime,
    pub notifications_enabled: bool,
    /// Silences notifications and lets Tiki sleep while the user concentrates.
    pub focus_mode: bool,
}

impl UserPreferences {
    /// Storage keys, one row per preference in the `preferences` table.
    pub const LOCALE_KEY: &'static str = "locale";
    pub const ISLAND_PLACEMENT_KEY: &'static str = "island_placement";
    pub const AI_PROVIDER_KEY: &'static str = "ai_provider";
    pub const AI_MODEL_KEY: &'static str = "ai_model";
    pub const MORNING_BRIEF_TIME_KEY: &'static str = "morning_brief_time";
    pub const EVENING_BRIEF_TIME_KEY: &'static str = "evening_brief_time";
    pub const NOTIFICATIONS_ENABLED_KEY: &'static str = "notifications_enabled";
    pub const FOCUS_MODE_KEY: &'static str = "focus_mode";
    /// How times are written in storage.
    pub const TIME_FORMAT: &'static str = "%H:%M";

    /// What a new user starts with.
    pub fn defaults(settings: &TikiSettings) -> Self {
        Self {
            locale: settings.locale,
            island_placement: IslandPlacement::default(),
            ai_provider: AiProvider::default(),
            ai_model: None,
            morning_brief_time: settings.morning_brief_time,
            evening_brief_time: settings.evening_close_time,
            notifications_enabled: true,
            focus_mode: false,
        }
    }

    /// Whether Tiki may interrupt the user with a notification.
    pub fn allows_notifications(&self) -> bool {
        self.notifications_enabled && !self.focus_mode
    }
}
