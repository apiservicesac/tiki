use super::{Clock, IslandSnapshot, SnapshotPresenter};
use crate::briefing::DailyBriefBuilder;
use crate::capture::{
    CaptureIntent, CaptureIntentReader, CaptureOutcome, ConfirmedCapture, QuickCaptureParser,
};
use crate::domain::{Client, ClientDirectory, ClientId, Task, TaskId, TaskStatus};
use crate::error::TikiResult;
use crate::i18n::{Locale, Translator};
use crate::mood::{MoodResolver, MoodSignals};
use crate::preferences::{IslandPlacement, UserPreferences};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Owns the user's data and settings and exposes the actions every client needs.
pub struct TikiWorkspace {
    database: Database,
    settings: TikiSettings,
    preferences: UserPreferences,
    translator: Translator,
    clock: Box<dyn Clock>,
    focus_mode: bool,
}

impl TikiWorkspace {
    pub fn new(
        database: Database,
        settings: TikiSettings,
        clock: Box<dyn Clock>,
    ) -> TikiResult<Self> {
        let defaults = UserPreferences {
            locale: settings.locale,
            island_placement: IslandPlacement::default(),
        };
        let preferences = database.preferences().load(defaults)?;
        let translator = Translator::new(preferences.locale)?;
        Ok(Self {
            database,
            settings,
            preferences,
            translator,
            clock,
            focus_mode: false,
        })
    }

    pub fn settings(&self) -> &TikiSettings {
        &self.settings
    }

    pub fn preferences(&self) -> UserPreferences {
        self.preferences
    }

    /// Saves new preferences; a new language applies to briefs, labels and quick capture.
    pub fn update_preferences(&mut self, preferences: UserPreferences) -> TikiResult<()> {
        if preferences.locale != self.preferences.locale {
            self.translator = Translator::new(preferences.locale)?;
        }
        self.database.preferences().save(preferences)?;
        self.preferences = preferences;
        Ok(())
    }

    /// Switches the language and remembers it.
    pub fn change_locale(&mut self, locale: Locale) -> TikiResult<()> {
        self.update_preferences(UserPreferences {
            locale,
            ..self.preferences
        })
    }

    pub fn set_focus_mode(&mut self, enabled: bool) {
        self.focus_mode = enabled;
    }

    /// What the island shows right now.
    pub fn island_snapshot(&self) -> TikiResult<IslandSnapshot> {
        let today = self.clock.today();
        let tasks = self.database.tasks().list_open()?;
        let clients = self.client_directory()?;
        let brief = DailyBriefBuilder::new(&self.settings).build(&tasks, &clients, today);
        let mood = MoodResolver::new(&self.settings).resolve(&MoodSignals {
            brief: &brief,
            minutes_to_next_meeting: None,
            focus_mode: self.focus_mode,
        });
        Ok(SnapshotPresenter::new(&self.translator, &clients).present(&brief, mood))
    }

    /// Reads a quick-capture sentence without saving it, so the user can confirm it.
    pub fn preview_capture(&self, sentence: &str) -> TikiResult<CaptureIntent> {
        let clients = self.database.clients().list()?;
        let task_parser = QuickCaptureParser::new(&self.translator.catalog().capture, &clients);
        CaptureIntentReader::new(&task_parser).read(sentence, self.clock.today())
    }

    /// Reads a quick-capture sentence and saves the task or client it describes.
    pub fn capture(&self, sentence: &str) -> TikiResult<CaptureOutcome> {
        match self.preview_capture(sentence)? {
            CaptureIntent::Task(draft) => {
                let task_id = self.database.tasks().insert(&draft.into_task()?)?;
                Ok(CaptureOutcome::Task(task_id))
            }
            CaptureIntent::Client { name } => Ok(CaptureOutcome::Client(self.add_client(&name)?)),
        }
    }

    /// Saves a task exactly as the user confirmed it; an unknown client name is left out.
    pub fn save_confirmed(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        let mut task = Task::new(capture.title)?.with_priority(capture.priority);
        task.due_date = capture.due_date;
        if let Some(name) = capture.client_name {
            task.client_id = self
                .database
                .clients()
                .find_by_name(&name)?
                .and_then(|client| client.id);
        }
        self.database.tasks().insert(&task)
    }

    pub fn clients(&self) -> TikiResult<Vec<Client>> {
        self.database.clients().list()
    }

    pub fn move_task(&self, task_id: TaskId, status: TaskStatus) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        task.move_to(status, self.clock.today());
        self.database.tasks().update(&task)
    }

    /// Saves a client with the next color of the palette.
    pub fn add_client(&self, name: &str) -> TikiResult<ClientId> {
        let existing_clients = self.database.clients().list()?.len();
        let palette = &self.settings.client_colors;
        let color = palette
            .get(existing_clients % palette.len().max(1))
            .cloned()
            .unwrap_or_default();
        self.database.clients().insert(&Client::new(name, color))
    }

    fn client_directory(&self) -> TikiResult<ClientDirectory> {
        Ok(ClientDirectory::new(self.database.clients().list()?))
    }
}
