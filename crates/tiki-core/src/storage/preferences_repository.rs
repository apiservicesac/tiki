use rusqlite::{Connection, OptionalExtension, params};

use crate::ai::AiProvider;
use crate::error::TikiResult;
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
        )
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
