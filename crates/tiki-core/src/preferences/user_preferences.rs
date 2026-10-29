use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use super::{IslandPlacement, QuietHours};
use crate::ai::AiProvider;
use crate::i18n::Locale;
use crate::settings::TikiSettings;

/// Everything the user can change in Settings.
// Each flag is an independent switch in Settings, not a hidden state machine.
#[allow(clippy::struct_excessive_bools)]
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
    /// Master switch for reminders: the island alert, and Windows notifications if chosen.
    pub notifications_enabled: bool,
    /// The island plays a soft chime when it shows an alert.
    pub alert_sound: bool,
    /// Also send each reminder as a Windows notification (it stays in the Windows history).
    pub windows_notifications: bool,
    /// Silences notifications and lets Tiki sleep while the user concentrates.
    pub focus_mode: bool,
    /// The night: Tiki sleeps, and alerts still reach the island but without a sound.
    pub quiet_hours: QuietHours,
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
    pub const ALERT_SOUND_KEY: &'static str = "alert_sound";
    pub const WINDOWS_NOTIFICATIONS_KEY: &'static str = "windows_notifications";
    pub const QUIET_HOURS_ENABLED_KEY: &'static str = "quiet_hours_enabled";
    pub const QUIET_HOURS_FROM_KEY: &'static str = "quiet_hours_from";
    pub const QUIET_HOURS_UNTIL_KEY: &'static str = "quiet_hours_until";
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
            evening_brief_time: settings.evening_brief_time,
            notifications_enabled: true,
            alert_sound: true,
            windows_notifications: true,
            focus_mode: false,
            quiet_hours: settings.quiet_hours,
        }
    }

    /// Whether Tiki may interrupt the user with a notification.
    pub fn allows_notifications(&self) -> bool {
        self.notifications_enabled && !self.focus_mode
    }
}
