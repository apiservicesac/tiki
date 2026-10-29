use super::{
    AssistantContext, Clock, DeliveredItem, IslandSnapshot, SnapshotPresenter, TodaysMeetings,
};
use chrono::NaiveDateTime;

use crate::alerts::{BriefPlanner, NotificationComposer, TikiAlert};
use crate::briefing::{BriefMoment, DailyBrief, DailyBriefBuilder};
use crate::calendar::{AccountId, CalendarAccount, CalendarBook, CalendarProvider, Meeting};
use crate::capture::{CaptureLesson, ConfirmedCapture};
use crate::domain::{FollowUpPolicy, Project, ProjectDirectory, ProjectId, TaskId, TaskStatus};
use crate::editing::{TaskDetails, TaskEdit};
use crate::error::TikiResult;
use crate::i18n::{Locale, Translator};
use crate::mood::{MoodResolver, MoodSignals};
use crate::preferences::UserPreferences;
use crate::projects::{ProjectBook, ProjectChange, ProjectSummary};
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
        let now = self.clock.now();
        let projects = self.project_directory()?;
        let brief = self.brief(&projects)?;
        let meetings = self.calendar_book().agenda(now.date())?;
        let next_meeting = CalendarBook::next_today(&meetings, now);
        let mood = MoodResolver::new(&self.settings).resolve(&MoodSignals {
            brief: &brief,
            minutes_to_next_meeting: next_meeting
                .map(|meeting| (meeting.starts_at - now).num_minutes()),
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
            &TodaysMeetings {
                now,
                next: next_meeting.cloned(),
                agenda: meetings.clone(),
            },
        ))
    }

    /// Open tasks in the shape the assistant needs to summarize the day or answer questions.
    pub fn assistant_context(&self) -> TikiResult<AssistantContext> {
        let tasks = self.database.tasks().list_open()?;
        Ok(AssistantContext::build(
            &tasks,
            &self.project_directory()?,
            self.clock.now(),
        ))
    }

    /// The brief notification due now, if any; it is marked as sent so it arrives once
    /// a day. Nothing is due while notifications are off or focus mode is on.
    pub fn take_due_notification(&self) -> TikiResult<Option<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(None);
        }
        let now = self.clock.now();
        let log = self.database.brief_log();
        let morning_sent = log.last_sent(BriefMoment::Morning)?;
        let evening_sent = log.last_sent(BriefMoment::Evening)?;
        let moment = BriefPlanner::new(&self.preferences).due_moment(now, |moment| match moment {
            BriefMoment::Morning => morning_sent,
            BriefMoment::Evening => evening_sent,
        });
        let Some(moment) = moment else {
            return Ok(None);
        };
        log.mark_sent(moment, now.date())?;
        if moment == BriefMoment::Evening {
            // A late start skips the morning brief instead of sending both at once.
            log.mark_sent(BriefMoment::Morning, now.date())?;
        }
        let brief = self.brief(&self.project_directory()?)?;
        let composer = NotificationComposer::new(&self.translator);
        let mut alert = composer.compose(&brief, moment);
        if moment == BriefMoment::Morning {
            let meetings = self.calendar_book().agenda(now.date())?;
            let today = CalendarBook::meetings_on(&meetings, now.date());
            if let Some(line) = composer.meetings_line(&today) {
                alert.body = format!("{}\n{line}", alert.body);
            }
        }
        Ok(Some(alert))
    }

    /// "Meeting in 15 min" alerts for meetings about to start; each is announced once.
    /// Nothing is announced while notifications are off or focus mode is on.
    pub fn take_due_meeting_alerts(&self) -> TikiResult<Vec<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(Vec::new());
        }
        let now = self.clock.now();
        let composer = NotificationComposer::new(&self.translator);
        Ok(self
            .calendar_book()
            .take_starting_soon(now)?
            .iter()
            .map(|meeting| composer.compose_meeting(meeting, now))
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

    fn calendar_book(&self) -> CalendarBook<'_> {
        CalendarBook::new(&self.database, &self.settings)
    }

    /// Task reminders whose time has come; each one is cleared so it rings once, all in one
    /// transaction so an alert is never shown without its reminder being moved on. Nothing is
    /// due while notifications are off or focus mode is on: they ring when it ends.
    pub fn take_due_task_reminders(&self) -> TikiResult<Vec<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(Vec::new());
        }
        let now = self.clock.now();
        let projects = self.project_directory()?;
        let composer = NotificationComposer::new(&self.translator);
        let today = now.date();
        self.database.transaction(|repositories| {
            let mut alerts = Vec::new();
            for mut task in repositories.tasks().list_reminders_due(now)? {
                alerts.push(composer.compose_task(&task, projects.name_of(task.project_id), today));
                task.ring(now);
                repositories.tasks().update(&task)?;
            }
            Ok(alerts)
        })
    }

    /// Saves an item exactly as the user confirmed it. A project named for the first time is
    /// created on the spot: Tiki never asks the user to keep a list of projects.
    pub fn save_confirmed(&self, capture: ConfirmedCapture) -> TikiResult<TaskId> {
        let project_id = self.project_named(capture.project_name.as_deref())?;
        let mut task = capture.into_task(project_id, self.clock.today())?;
        self.follow_up_policy().update_nudge(&mut task, None);
        task.normalize()?;
        self.database.tasks().insert(&task)
    }

    /// Puts off a reminder for `minutes`; a reminder that already rang comes back to life.
    pub fn snooze(&self, task_id: TaskId, minutes: i64) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        task.snooze(self.clock.now() + chrono::Duration::minutes(minutes));
        self.database.tasks().update(&task)
    }

    /// A saved task as the edit form shows it.
    pub fn task_details(&self, task_id: TaskId) -> TikiResult<TaskDetails> {
        let task = self.database.tasks().find(task_id)?;
        TaskDetails::from_task(&task, &self.project_directory()?)
    }

    /// Saves the user's changes to an item; a new project name creates the project.
    pub fn edit_task(&self, task_id: TaskId, edit: TaskEdit) -> TikiResult<()> {
        let mut task = self.database.tasks().find(task_id)?;
        let waiting_since_before = task.waiting_since;
        let project_id = self.project_named(edit.project_name.as_deref())?;
        edit.apply_to(&mut task, project_id, self.clock.today());
        self.follow_up_policy()
            .update_nudge(&mut task, waiting_since_before);
        task.normalize()?;
        self.database.tasks().update(&task)
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
        let waiting_since_before = task.waiting_since;
        task.move_to(status, self.clock.today());
        self.follow_up_policy()
            .update_nudge(&mut task, waiting_since_before);
        task.normalize()?;
        self.database.tasks().update(&task)
    }

    fn follow_up_policy(&self) -> FollowUpPolicy {
        FollowUpPolicy::new(&self.settings)
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

    /// The project with this name, created when it does not exist yet; `None` for no name.
    fn project_named(&self, name: Option<&str>) -> TikiResult<Option<ProjectId>> {
        let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
            return Ok(None);
        };
        match self.database.projects().find_by_name(name)? {
            Some(project) => Ok(project.id),
            None => self.project_book().create(name).map(Some),
        }
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
                self.database.brief_log().clear(moment)?;
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
