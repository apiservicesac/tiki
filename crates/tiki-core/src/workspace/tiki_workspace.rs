use chrono::{NaiveDate, NaiveDateTime};

use super::{AssistantContext, Clock, DeliveredItem, IslandSnapshot, WorkspaceViews};
use crate::activity::ItemEventRecord;
use crate::alerts::{AlertDelivery, AlertScheduler, DueAlert, TikiAlert};
use crate::calendar::{AccountId, CalendarAccount, CalendarBook, CalendarProvider, Meeting};
use crate::capture::{CaptureLesson, ConfirmedCapture};
use crate::domain::{Project, ProjectId, TaskId, TaskStatus};
use crate::editing::{TaskDetails, TaskEdit};
use crate::error::TikiResult;
use crate::i18n::{Locale, Translator};
use crate::items::{ItemService, TrashBin};
use crate::preferences::UserPreferences;
use crate::projects::{ProjectBook, ProjectChange, ProjectSummary};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// The entry point every platform uses: it owns the user's data, settings, preferences and
/// clock, and hands each request to the part that owns the rules (`ItemService`, `TrashBin`,
/// `AlertScheduler`, `CalendarBook`, `ProjectBook`, `WorkspaceViews`).
pub struct TikiWorkspace {
    database: Database,
    settings: TikiSettings,
    preferences: UserPreferences,
    translator: Translator,
    clock: Box<dyn Clock>,
    /// Another app holds the microphone, as the shell reports it.
    in_call: bool,
    /// The day the expired trash was last deleted; empty until the first time this run.
    trash_purged_on: Option<NaiveDate>,
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
            in_call: false,
            trash_purged_on: None,
        })
    }

    /// The moment the workspace's clock reads: the one clock every rule uses.
    pub fn now(&self) -> NaiveDateTime {
        self.clock.now()
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
        self.alerts().reschedule_moved_briefs(&preferences)?;
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

    /// The shell saw a call start or end: while it lasts, alerts arrive without a sound.
    pub fn set_in_call(&mut self, in_call: bool) {
        self.in_call = in_call;
    }

    /// What the island shows right now.
    pub fn island_snapshot(&self) -> TikiResult<IslandSnapshot> {
        self.views().island_snapshot()
    }

    /// Open items in the shape the assistant needs to summarize the day or answer questions.
    pub fn assistant_context(&self) -> TikiResult<AssistantContext> {
        self.views().assistant_context()
    }

    /// A saved item as the edit form shows it.
    pub fn task_details(&self, task_id: TaskId) -> TikiResult<TaskDetails> {
        self.views().task_details(task_id)
    }

    /// Delivered items, most recent delivery first, for the history.
    pub fn delivered_tasks(&self) -> TikiResult<Vec<DeliveredItem>> {
        self.views().delivered_tasks()
    }

    /// Everything that happened to an item, oldest first.
    pub fn activity_of(&self, task_id: TaskId) -> TikiResult<Vec<ItemEventRecord>> {
        self.database.activity().events_of(task_id)
    }

    /// Saves an item exactly as the user confirmed it. A project named for the first time is
    /// created on the spot: Tiki never asks the user to keep a list of projects.
    pub fn save_confirmed(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        self.items().capture(capture)
    }

    /// Saves the user's changes to an item; a new project name creates the project.
    pub fn edit_task(&self, task_id: TaskId, edit: TaskEdit) -> TikiResult<()> {
        self.items().edit(task_id, edit)
    }

    pub fn move_task(&self, task_id: TaskId, status: TaskStatus) -> TikiResult<()> {
        self.items().move_to(task_id, status)
    }

    /// Delivers an item: what the user says with "done" or "entregado".
    pub fn deliver(&self, task_id: TaskId) -> TikiResult<()> {
        self.items().move_to(task_id, TaskStatus::Done)
    }

    /// Brings a delivered item back to "to do"; its delivery stays in the history.
    pub fn reopen(&self, task_id: TaskId) -> TikiResult<()> {
        self.items().reopen(task_id)
    }

    /// Puts the item on hold until `who` answers; a follow-up nudge is scheduled.
    pub fn wait_for(&self, task_id: TaskId, who: &str) -> TikiResult<()> {
        self.items().wait_for(task_id, who)
    }

    /// Puts off an item's reminder until `until`; a reminder that already rang comes back.
    pub fn snooze(&self, task_id: TaskId, until: NaiveDateTime) -> TikiResult<()> {
        self.items().snooze(task_id, until)
    }

    /// Puts off an item's reminder for `minutes` from now, as the alert's buttons do.
    pub fn snooze_for(&self, task_id: TaskId, minutes: i64) -> TikiResult<()> {
        self.snooze(
            task_id,
            self.clock.now() + chrono::Duration::minutes(minutes),
        )
    }

    /// Sends an item to the trash: it leaves every view, the brief, the reports and the
    /// alerts until it is restored or deleted forever.
    pub fn trash_item(&self, task_id: TaskId) -> TikiResult<()> {
        self.items().trash(task_id)
    }

    /// Takes an item out of the trash exactly as it was.
    pub fn restore_item(&self, task_id: TaskId) -> TikiResult<()> {
        self.items().restore(task_id)
    }

    /// Deletes an item in the trash forever, with its whole history.
    pub fn delete_item_forever(&self, task_id: TaskId) -> TikiResult<()> {
        self.trash_bin().delete_forever(task_id)
    }

    /// Deletes every item in the trash forever.
    pub fn empty_trash(&self) -> TikiResult<()> {
        self.trash_bin().empty()
    }

    /// Deletes forever the items whose time in the trash is over. It works once a day, the
    /// first time it is asked (the shell asks on start and then every minute). Returns how
    /// many items went.
    pub fn purge_expired_trash(&mut self) -> TikiResult<usize> {
        let today = self.clock.today();
        if self.trash_purged_on == Some(today) {
            return Ok(0);
        }
        let purged = self.trash_bin().purge_expired(today)?;
        self.trash_purged_on = Some(today);
        Ok(purged)
    }

    /// Remembers a correction the user made to an AI capture.
    pub fn record_capture_lesson(&self, lesson: &CaptureLesson) -> TikiResult<()> {
        self.database
            .capture_lessons()
            .record(lesson, self.settings.capture_lesson_limit)
    }

    /// The user's recent corrections, newest first, for the AI to learn from.
    pub fn capture_lessons(&self) -> TikiResult<Vec<CaptureLesson>> {
        self.database
            .capture_lessons()
            .recent(self.settings.capture_lesson_limit)
    }

    /// The brief due now, if any; it arrives once a day (see `AlertScheduler`).
    pub fn take_due_notification(&self) -> TikiResult<Option<TikiAlert>> {
        self.alerts().take_due_brief()
    }

    /// Reminders and follow-up nudges whose time has come; each rings once.
    pub fn take_due_task_reminders(&self) -> TikiResult<Vec<TikiAlert>> {
        self.alerts().take_due_item_alerts()
    }

    /// "Meeting in 15 min" alerts for meetings about to start; each is announced once.
    pub fn take_due_meeting_alerts(&self) -> TikiResult<Vec<TikiAlert>> {
        self.alerts().take_due_meeting_alerts()
    }

    /// Every alert due now (the brief, item reminders and meetings), each with how it may
    /// reach the user (see `AlertDelivery`).
    pub fn take_due_alerts(&self) -> TikiResult<Vec<DueAlert>> {
        let mut due: Vec<TikiAlert> = self.take_due_notification()?.into_iter().collect();
        due.extend(self.take_due_task_reminders()?);
        due.extend(self.take_due_meeting_alerts()?);
        let delivery =
            AlertDelivery::decide(&self.preferences, self.clock.now().time(), self.in_call);
        Ok(due
            .into_iter()
            .map(|alert| DueAlert { alert, delivery })
            .collect())
    }

    pub fn calendar_accounts(&self) -> TikiResult<Vec<CalendarAccount>> {
        self.calendar_book().accounts()
    }

    pub fn calendar_account(&self, account_id: AccountId) -> TikiResult<CalendarAccount> {
        self.calendar_book().find(account_id)
    }

    /// Remembers a calendar account the user signed in to.
    pub fn connect_calendar(
        &self,
        provider: CalendarProvider,
        email: &str,
    ) -> TikiResult<AccountId> {
        self.calendar_book().connect(provider, email)
    }

    /// Forgets an account and its meetings.
    pub fn disconnect_calendar(&self, account_id: AccountId) -> TikiResult<()> {
        self.calendar_book().disconnect(account_id)
    }

    /// The window of days the calendar copy covers, from the start of today.
    pub fn calendar_window(&self) -> (NaiveDateTime, NaiveDateTime) {
        self.calendar_book().window(self.clock.today())
    }

    /// Replaces what Tiki knows of an account's meetings in the calendar window.
    pub fn store_meetings(&self, account_id: AccountId, meetings: &[Meeting]) -> TikiResult<()> {
        self.calendar_book()
            .store(account_id, self.clock.today(), meetings)
    }

    pub fn projects(&self) -> TikiResult<Vec<Project>> {
        self.database.projects().list()
    }

    /// Saves a project with the next color of the palette.
    pub fn add_project(&self, name: &str) -> TikiResult<ProjectId> {
        self.project_book().create(name)
    }

    /// Every project with their open and pressing work, busiest first.
    pub fn project_summaries(&self) -> TikiResult<Vec<ProjectSummary>> {
        let open_tasks = self.database.tasks().list_open()?;
        self.project_book()
            .summaries(&open_tasks, self.clock.today())
    }

    pub fn change_project(&self, project_id: ProjectId, change: ProjectChange) -> TikiResult<()> {
        self.project_book().change(project_id, change)
    }

    /// Removes a project; their tasks stay, without project.
    pub fn remove_project(&self, project_id: ProjectId) -> TikiResult<()> {
        self.project_book().remove(project_id)
    }

    /// The colors new projects get, offered when changing one.
    pub fn project_colors(&self) -> &[String] {
        &self.settings.project_colors
    }

    fn views(&self) -> WorkspaceViews<'_> {
        WorkspaceViews::new(
            &self.database,
            &self.settings,
            &self.preferences,
            &self.translator,
            self.clock.as_ref(),
            self.in_call,
        )
    }

    fn items(&self) -> ItemService<'_> {
        ItemService::new(&self.database, &self.settings, self.clock.as_ref())
    }

    fn trash_bin(&self) -> TrashBin<'_> {
        TrashBin::new(&self.database, &self.settings)
    }

    fn alerts(&self) -> AlertScheduler<'_> {
        AlertScheduler::new(
            &self.database,
            &self.settings,
            &self.preferences,
            &self.translator,
            self.clock.as_ref(),
        )
    }

    fn calendar_book(&self) -> CalendarBook<'_> {
        CalendarBook::new(&self.database, &self.settings)
    }

    fn project_book(&self) -> ProjectBook<'_> {
        ProjectBook::new(&self.database, &self.settings)
    }
}
