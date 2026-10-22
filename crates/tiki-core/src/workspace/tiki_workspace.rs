use super::{AssistantContext, Clock, DeliveredItem, IslandSnapshot, SnapshotPresenter};
use chrono::NaiveDateTime;

use crate::briefing::{BriefMoment, DailyBrief, DailyBriefBuilder};
use crate::capture::ConfirmedCapture;
use crate::clients::{ClientBook, ClientChange, ClientSummary};
use crate::domain::{Client, ClientDirectory, ClientId, Task, TaskId, TaskStatus};
use crate::editing::{TaskDetails, TaskEdit};
use crate::error::TikiResult;
use crate::i18n::{Locale, Translator};
use crate::mood::{MoodResolver, MoodSignals};
use crate::preferences::UserPreferences;
use crate::reminders::{NotificationComposer, ReminderPlanner, TikiAlert};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Owns the user's data and settings and exposes the actions every client needs.
pub struct TikiWorkspace {
    database: Database,
    settings: TikiSettings,
    preferences: UserPreferences,
    translator: Translator,
    clock: Box<dyn Clock>,
}

impl TikiWorkspace {
    pub fn new(
        database: Database,
        settings: TikiSettings,
        clock: Box<dyn Clock>,
    ) -> TikiResult<Self> {
        let preferences = database
            .preferences()
            .load(UserPreferences::defaults(&settings))?;
        let translator = Translator::new(preferences.locale)?;
        Ok(Self {
            database,
            settings,
            preferences,
            translator,
            clock,
        })
    }

    pub fn settings(&self) -> &TikiSettings {
        &self.settings
    }

    pub fn preferences(&self) -> &UserPreferences {
        &self.preferences
    }

    /// Saves new preferences; a new language applies to briefs, labels and quick capture.
    /// A brief moved to a time still ahead today arrives today, even if it already did.
    pub fn update_preferences(&mut self, preferences: UserPreferences) -> TikiResult<()> {
        if preferences.locale != self.preferences.locale {
            self.translator = Translator::new(preferences.locale)?;
        }
        self.database.preferences().save(&preferences)?;
        self.reschedule_moved_briefs(&preferences)?;
        self.preferences = preferences;
        Ok(())
    }

    /// Switches the language and remembers it.
    pub fn change_locale(&mut self, locale: Locale) -> TikiResult<()> {
        self.update_preferences(UserPreferences {
            locale,
            ..self.preferences.clone()
        })
    }

    /// Turns focus mode on or off and remembers it.
    pub fn set_focus_mode(&mut self, enabled: bool) -> TikiResult<()> {
        self.update_preferences(UserPreferences {
            focus_mode: enabled,
            ..self.preferences.clone()
        })
    }

    /// What the island shows right now.
    pub fn island_snapshot(&self) -> TikiResult<IslandSnapshot> {
        let clients = self.client_directory()?;
        let brief = self.brief(&clients)?;
        let mood = MoodResolver::new(&self.settings).resolve(&MoodSignals {
            brief: &brief,
            minutes_to_next_meeting: None,
            focus_mode: self.preferences.focus_mode,
        });
        let delivered_today = self
            .database
            .tasks()
            .count_delivered_on(self.clock.today())?;
        Ok(SnapshotPresenter::new(&self.translator, &clients).present(
            &brief,
            mood,
            delivered_today,
        ))
    }

    /// Open tasks in the shape the assistant needs to summarize the day or answer questions.
    pub fn assistant_context(&self) -> TikiResult<AssistantContext> {
        let tasks = self.database.tasks().list_open()?;
        Ok(AssistantContext::build(
            &tasks,
            &self.client_directory()?,
            self.clock.today(),
        ))
    }

    /// The brief notification due at `now`, if any; it is marked as sent so it arrives once
    /// a day. Nothing is due while notifications are off or focus mode is on.
    pub fn take_due_notification(&self, now: NaiveDateTime) -> TikiResult<Option<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(None);
        }
        let log = self.database.reminder_log();
        let moment = ReminderPlanner::new(&self.preferences)
            .due_moment(now, |moment| log.last_sent(moment).ok().flatten());
        let Some(moment) = moment else {
            return Ok(None);
        };
        log.mark_sent(moment, now.date())?;
        if moment == BriefMoment::Evening {
            // A late start skips the morning brief instead of sending both at once.
            log.mark_sent(BriefMoment::Morning, now.date())?;
        }
        let brief = self.brief(&self.client_directory()?)?;
        Ok(Some(
            NotificationComposer::new(&self.translator).compose(&brief, moment),
        ))
    }

    /// Task reminders whose time has come; each one is cleared so it rings once. Nothing is
    /// due while notifications are off or focus mode is on: they ring when it ends.
    pub fn take_due_task_reminders(&self, now: NaiveDateTime) -> TikiResult<Vec<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(Vec::new());
        }
        let clients = self.client_directory()?;
        let composer = NotificationComposer::new(&self.translator);
        let today = now.date();
        let mut alerts = Vec::new();
        for mut task in self.database.tasks().list_reminders_due(now)? {
            alerts.push(composer.compose_task(&task, clients.name_of(task.client_id), today));
            task.remind_at = None;
            self.database.tasks().update(&task)?;
        }
        Ok(alerts)
    }

    /// Saves a task exactly as the user confirmed it; an unknown client name is left out.
    pub fn save_confirmed(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        let mut task = Task::new(capture.title)?.with_priority(capture.priority);
        task.due_date = capture.due_date;
        task.client_id = self.client_id_by_name(capture.client_name.as_deref())?;
        task.remind_at = capture.remind_at;
        self.database.tasks().insert(&task)
    }

    /// A saved task as the edit form shows it.
    pub fn task_details(&self, task_id: TaskId) -> TikiResult<TaskDetails> {
        let task = self.database.tasks().find(task_id)?;
        TaskDetails::from_task(&task, &self.client_directory()?)
    }

    /// Saves the user's changes to a task; an unknown client name leaves it without client.
    pub fn edit_task(&self, task_id: TaskId, edit: TaskEdit) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        let client_id = self.client_id_by_name(edit.client_name.as_deref())?;
        edit.apply_to(&mut task, client_id, self.clock.today())?;
        self.database.tasks().update(&task)
    }

    /// Delivered tasks, most recent first, for the history.
    pub fn delivered_tasks(&self) -> TikiResult<Vec<DeliveredItem>> {
        let tasks = self
            .database
            .tasks()
            .list_delivered(self.settings.delivered_history_limit)?;
        let clients = self.client_directory()?;
        let today = self.clock.today();
        Ok(tasks
            .iter()
            .filter_map(|task| DeliveredItem::present(task, &clients, &self.translator, today))
            .collect())
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
        self.client_book().create(name)
    }

    /// Every client with their open and pressing work, busiest first.
    pub fn client_summaries(&self) -> TikiResult<Vec<ClientSummary>> {
        let open_tasks = self.database.tasks().list_open()?;
        self.client_book()
            .summaries(&open_tasks, self.clock.today())
    }

    pub fn change_client(&self, client_id: ClientId, change: ClientChange) -> TikiResult<()> {
        self.client_book().change(client_id, change)
    }

    /// Removes a client; their tasks stay, without client.
    pub fn remove_client(&self, client_id: ClientId) -> TikiResult<()> {
        self.client_book().remove(client_id)
    }

    /// The colors new clients get, offered when changing one.
    pub fn client_colors(&self) -> &[String] {
        &self.settings.client_colors
    }

    fn client_book(&self) -> ClientBook<'_> {
        ClientBook::new(&self.database, &self.settings)
    }

    fn client_id_by_name(&self, name: Option<&str>) -> TikiResult<Option<ClientId>> {
        let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
            return Ok(None);
        };
        Ok(self
            .database
            .clients()
            .find_by_name(name)?
            .and_then(|client| client.id))
    }

    fn reschedule_moved_briefs(&self, preferences: &UserPreferences) -> TikiResult<()> {
        let now = self.clock.now().time();
        let moves = [
            (
                BriefMoment::Morning,
                self.preferences.morning_brief_time,
                preferences.morning_brief_time,
            ),
            (
                BriefMoment::Evening,
                self.preferences.evening_brief_time,
                preferences.evening_brief_time,
            ),
        ];
        for (moment, old_time, new_time) in moves {
            if new_time != old_time && new_time > now {
                self.database.reminder_log().clear(moment)?;
            }
        }
        Ok(())
    }

    fn brief(&self, clients: &ClientDirectory) -> TikiResult<DailyBrief> {
        let tasks = self.database.tasks().list_open()?;
        Ok(DailyBriefBuilder::new(&self.settings).build(&tasks, clients, self.clock.today()))
    }

    fn client_directory(&self) -> TikiResult<ClientDirectory> {
        Ok(ClientDirectory::new(self.database.clients().list()?))
    }
}
