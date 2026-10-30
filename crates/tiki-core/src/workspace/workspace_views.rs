use super::{
    AssistantContext, Clock, DeliveredItem, IslandSnapshot, SnapshotPresenter, TodaysMeetings,
    TrashedItem,
};
use crate::briefing::DailyBriefBuilder;
use crate::calendar::CalendarBook;
use crate::domain::{TaskId, TrashPolicy};
use crate::editing::TaskDetails;
use crate::error::TikiResult;
use crate::i18n::Translator;
use crate::mood::{MoodResolver, MoodSignals};
use crate::preferences::UserPreferences;
use crate::projects::ProjectBook;
use crate::sessions::ActiveRecording;
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Reads what the interface and the assistant show: the island snapshot, the delivered
/// history, the trash, the edit form and the assistant's context. It never changes anything.
pub struct WorkspaceViews<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
    preferences: &'context UserPreferences,
    translator: &'context Translator,
    clock: &'context dyn Clock,
    in_call: bool,
}

impl<'context> WorkspaceViews<'context> {
    pub fn new(
        database: &'context Database,
        settings: &'context TikiSettings,
        preferences: &'context UserPreferences,
        translator: &'context Translator,
        clock: &'context dyn Clock,
        in_call: bool,
    ) -> Self {
        Self {
            database,
            settings,
            preferences,
            translator,
            clock,
            in_call,
        }
    }

    /// What the island shows right now.
    pub fn island_snapshot(&self) -> TikiResult<IslandSnapshot> {
        let now = self.clock.now();
        let projects = ProjectBook::new(self.database, self.settings).directory()?;
        let brief = DailyBriefBuilder::new(self.settings).build(
            &self.database.tasks().list_open()?,
            &projects,
            now.date(),
        );
        let meetings = CalendarBook::new(self.database, self.settings).agenda(now.date())?;
        let next_meeting = CalendarBook::next_today(&meetings, now);
        let mood = MoodResolver::new(self.settings).resolve(&MoodSignals {
            brief: &brief,
            minutes_to_next_meeting: next_meeting
                .map(|meeting| (meeting.starts_at - now).num_minutes()),
            focus_mode: self.preferences.focus_mode,
            quiet_hours: self.preferences.quiet_hours.covers(now.time()),
        });
        let delivered_today = self.database.activity().count_delivered_on(now.date())?;
        let recording = self
            .database
            .sessions()
            .capturing()?
            .map(|session| ActiveRecording::of(&session, now))
            .transpose()?;
        Ok(SnapshotPresenter::new(self.translator, &projects).present(
            &brief,
            mood,
            self.in_call,
            recording,
            delivered_today,
            &TodaysMeetings {
                now,
                next: next_meeting.cloned(),
                agenda: meetings.clone(),
            },
        ))
    }

    /// Delivered items, most recent delivery first, for the history.
    pub fn delivered_tasks(&self) -> TikiResult<Vec<DeliveredItem>> {
        let deliveries = self
            .database
            .activity()
            .latest_deliveries(self.settings.delivered_history_limit)?;
        let projects = ProjectBook::new(self.database, self.settings).directory()?;
        let today = self.clock.today();
        let mut delivered = Vec::new();
        for delivery in deliveries {
            let task = self.database.tasks().find(delivery.item_id)?;
            delivered.extend(DeliveredItem::present(
                &task,
                delivery.at.date(),
                &projects,
                self.translator,
                today,
            ));
        }
        Ok(delivered)
    }

    /// Items in the trash, most recently trashed first.
    pub fn trashed_items(&self) -> TikiResult<Vec<TrashedItem>> {
        let projects = ProjectBook::new(self.database, self.settings).directory()?;
        let policy = TrashPolicy::new(self.settings);
        let today = self.clock.today();
        Ok(self
            .database
            .tasks()
            .list_trashed()?
            .iter()
            .filter_map(|task| {
                TrashedItem::present(task, &projects, &policy, self.translator, today)
            })
            .collect())
    }

    /// A saved item as the edit form shows it.
    pub fn task_details(&self, task_id: TaskId) -> TikiResult<TaskDetails> {
        let task = self.database.tasks().find(task_id)?;
        TaskDetails::from_task(
            &task,
            &ProjectBook::new(self.database, self.settings).directory()?,
        )
    }

    /// Open items in the shape the assistant needs to summarize the day or answer questions.
    pub fn assistant_context(&self) -> TikiResult<AssistantContext> {
        Ok(AssistantContext::build(
            &self.database.tasks().list_open()?,
            &ProjectBook::new(self.database, self.settings).directory()?,
            self.clock.now(),
        ))
    }
}
