use super::{AssistantContext, Clock, DeliveredItem, IslandSnapshot, SnapshotPresenter};
use chrono::NaiveDateTime;

use crate::briefing::{BriefMoment, DailyBrief, DailyBriefBuilder};
use crate::capture::ConfirmedCapture;
use crate::domain::{Project, ProjectDirectory, ProjectId, Task, TaskId, TaskStatus};
use crate::editing::{TaskDetails, TaskEdit};
use crate::error::TikiResult;
use crate::i18n::{Locale, Translator};
use crate::mood::{MoodResolver, MoodSignals};
use crate::preferences::UserPreferences;
use crate::projects::{ProjectBook, ProjectChange, ProjectSummary};
use crate::reminders::{NotificationComposer, ReminderPlanner, TikiAlert};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Owns the user's data and settings and exposes the actions every project needs.
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
        let projects = self.project_directory()?;
        let brief = self.brief(&projects)?;
        let mood = MoodResolver::new(&self.settings).resolve(&MoodSignals {
            brief: &brief,
            minutes_to_next_meeting: None,
            focus_mode: self.preferences.focus_mode,
        });
        let delivered_today = self
            .database
            .tasks()
            .count_delivered_on(self.clock.today())?;
        Ok(SnapshotPresenter::new(&self.translator, &projects).present(
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
            &self.project_directory()?,
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
        let brief = self.brief(&self.project_directory()?)?;
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
        let projects = self.project_directory()?;
        let composer = NotificationComposer::new(&self.translator);
        let today = now.date();
        let mut alerts = Vec::new();
        for mut task in self.database.tasks().list_reminders_due(now)? {
            alerts.push(composer.compose_task(&task, projects.name_of(task.project_id), today));
            task.remind_at = None;
            self.database.tasks().update(&task)?;
        }
        Ok(alerts)
    }

    /// Saves a task exactly as the user confirmed it; an unknown project name is left out.
    pub fn save_confirmed(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        let mut task = Task::new(capture.title)?.with_priority(capture.priority);
        task.due_date = capture.due_date;
        task.project_id = self.project_id_by_name(capture.project_name.as_deref())?;
        task.remind_at = capture.remind_at;
        self.database.tasks().insert(&task)
    }

    /// A saved task as the edit form shows it.
    pub fn task_details(&self, task_id: TaskId) -> TikiResult<TaskDetails> {
        let task = self.database.tasks().find(task_id)?;
        TaskDetails::from_task(&task, &self.project_directory()?)
    }

    /// Saves the user's changes to a task; an unknown project name leaves it without project.
    pub fn edit_task(&self, task_id: TaskId, edit: TaskEdit) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        let project_id = self.project_id_by_name(edit.project_name.as_deref())?;
        edit.apply_to(&mut task, project_id, self.clock.today())?;
        self.database.tasks().update(&task)
    }

    /// Delivered tasks, most recent first, for the history.
    pub fn delivered_tasks(&self) -> TikiResult<Vec<DeliveredItem>> {
        let tasks = self
            .database
            .tasks()
            .list_delivered(self.settings.delivered_history_limit)?;
        let projects = self.project_directory()?;
        let today = self.clock.today();
        Ok(tasks
            .iter()
            .filter_map(|task| DeliveredItem::present(task, &projects, &self.translator, today))
            .collect())
    }

    pub fn projects(&self) -> TikiResult<Vec<Project>> {
        self.database.projects().list()
    }

    pub fn move_task(&self, task_id: TaskId, status: TaskStatus) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        task.move_to(status, self.clock.today());
        self.database.tasks().update(&task)
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

    fn project_book(&self) -> ProjectBook<'_> {
        ProjectBook::new(&self.database, &self.settings)
    }

    fn project_id_by_name(&self, name: Option<&str>) -> TikiResult<Option<ProjectId>> {
        let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
            return Ok(None);
        };
        Ok(self
            .database
            .projects()
            .find_by_name(name)?
            .and_then(|project| project.id))
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

    fn brief(&self, projects: &ProjectDirectory) -> TikiResult<DailyBrief> {
        let tasks = self.database.tasks().list_open()?;
        Ok(DailyBriefBuilder::new(&self.settings).build(&tasks, projects, self.clock.today()))
    }

    fn project_directory(&self) -> TikiResult<ProjectDirectory> {
        Ok(ProjectDirectory::new(self.database.projects().list()?))
    }
}
