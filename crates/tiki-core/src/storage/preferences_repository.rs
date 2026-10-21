use chrono::NaiveTime;
use rusqlite::{Connection, OptionalExtension, params};

use crate::ai::AiProvider;
use crate::error::{TikiError, TikiResult};
use crate::i18n::Locale;
use crate::preferences::{IslandPlacement, UserPreferences};

/// Reads and writes the user's preferences as key/value rows.
pub struct PreferencesRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> PreferencesRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    /// Stored preferences; any value never saved falls back to `defaults`.
    pub fn load(&self, defaults: UserPreferences) -> TikiResult<UserPreferences> {
        let locale = match self.value(UserPreferences::LOCALE_KEY)? {
            Some(code) => Locale::from_code(&code)?,
            None => defaults.locale,
        };
        let island_placement = match self.value(UserPreferences::ISLAND_PLACEMENT_KEY)? {
            Some(key) => IslandPlacement::from_key(&key)?,
            None => defaults.island_placement,
        };
        let ai_provider = match self.value(UserPreferences::AI_PROVIDER_KEY)? {
            Some(key) => AiProvider::from_key(&key)?,
            None => defaults.ai_provider,
        };
        let ai_model = self
            .value(UserPreferences::AI_MODEL_KEY)?
            .filter(|model| !model.is_empty())
            .or(defaults.ai_model);
        Ok(UserPreferences {
            locale,
            island_placement,
            ai_provider,
            ai_model,
            morning_brief_time: self
                .time(UserPreferences::MORNING_BRIEF_TIME_KEY)?
                .unwrap_or(defaults.morning_brief_time),
            evening_brief_time: self
                .time(UserPreferences::EVENING_BRIEF_TIME_KEY)?
                .unwrap_or(defaults.evening_brief_time),
            notifications_enabled: self
                .flag(UserPreferences::NOTIFICATIONS_ENABLED_KEY)?
                .unwrap_or(defaults.notifications_enabled),
            focus_mode: self
                .flag(UserPreferences::FOCUS_MODE_KEY)?
                .unwrap_or(defaults.focus_mode),
        })
    }

    pub fn save(&self, preferences: &UserPreferences) -> TikiResult<()> {
        self.store(UserPreferences::LOCALE_KEY, preferences.locale.code())?;
        self.store(
            UserPreferences::ISLAND_PLACEMENT_KEY,
            preferences.island_placement.key(),
        )?;
        self.store(
            UserPreferences::AI_PROVIDER_KEY,
            preferences.ai_provider.key(),
        )?;
        self.store(
            UserPreferences::AI_MODEL_KEY,
            preferences.ai_model.as_deref().unwrap_or_default(),
        )?;
        for (key, time) in [
            (
                UserPreferences::MORNING_BRIEF_TIME_KEY,
                preferences.morning_brief_time,
            ),
            (
                UserPreferences::EVENING_BRIEF_TIME_KEY,
                preferences.evening_brief_time,
            ),
        ] {
            self.store(key, &time.format(UserPreferences::TIME_FORMAT).to_string())?;
        }
        self.store(
            UserPreferences::NOTIFICATIONS_ENABLED_KEY,
            &preferences.notifications_enabled.to_string(),
        )?;
        self.store(
            UserPreferences::FOCUS_MODE_KEY,
            &preferences.focus_mode.to_string(),
        )
    }

    fn time(&self, key: &str) -> TikiResult<Option<NaiveTime>> {
        self.value(key)?
            .map(|text| {
                NaiveTime::parse_from_str(&text, UserPreferences::TIME_FORMAT).map_err(|_| {
                    TikiError::UnknownValue {
                        kind: "time",
                        value: text,
                    }
                })
            })
            .transpose()
    }

    fn flag(&self, key: &str) -> TikiResult<Option<bool>> {
        self.value(key)?
            .map(|text| {
                text.parse::<bool>().map_err(|_| TikiError::UnknownValue {
                    kind: "flag",
                    value: text,
                })
            })
            .transpose()
    }

    fn value(&self, key: &str) -> TikiResult<Option<String>> {
        Ok(self
            .connection
            .query_row(
                "SELECT value FROM preferences WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    fn store(&self, key: &str, value: &str) -> TikiResult<()> {
        self.connection.execute(
            "INSERT INTO preferences (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
