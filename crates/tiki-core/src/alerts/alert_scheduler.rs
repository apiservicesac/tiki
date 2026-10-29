use super::{BriefPlanner, NotificationComposer, TikiAlert};
use crate::briefing::{BriefMoment, DailyBriefBuilder};
use crate::calendar::CalendarBook;
use crate::domain::FollowUpPolicy;
use crate::error::{TikiError, TikiResult};
use crate::i18n::Translator;
use crate::preferences::UserPreferences;
use crate::projects::ProjectBook;
use crate::settings::TikiSettings;
use crate::storage::Database;
use crate::workspace::Clock;

/// Decides which alerts are due now (the morning brief and the end-of-day close, item
/// reminders, follow-up nudges and meetings starting soon) and marks each one so it rings
/// once. Nothing is due while notifications are off or focus mode is on.
pub struct AlertScheduler<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
    preferences: &'context UserPreferences,
    translator: &'context Translator,
    clock: &'context dyn Clock,
}

impl<'context> AlertScheduler<'context> {
    pub fn new(
        database: &'context Database,
        settings: &'context TikiSettings,
        preferences: &'context UserPreferences,
        translator: &'context Translator,
        clock: &'context dyn Clock,
    ) -> Self {
        Self {
            database,
            settings,
            preferences,
            translator,
            clock,
        }
    }

    /// The brief due now, if any; it is marked as sent so it arrives once a day.
    pub fn take_due_brief(&self) -> TikiResult<Option<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(None);
        }
        let now = self.clock.now();
        let log = self.database.brief_log();
        let morning_sent = log.last_sent(BriefMoment::Morning)?;
        let evening_sent = log.last_sent(BriefMoment::Evening)?;
        let moment = BriefPlanner::new(self.preferences).due_moment(now, |moment| match moment {
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
        let projects = ProjectBook::new(self.database, self.settings).directory()?;
        let brief = DailyBriefBuilder::new(self.settings).build(
            &self.database.tasks().list_open()?,
            &projects,
            now.date(),
        );
        let composer = NotificationComposer::new(self.translator);
        let mut alert = composer.compose(&brief, moment);
        if moment == BriefMoment::Morning {
            let meetings = CalendarBook::new(self.database, self.settings).agenda(now.date())?;
            let today = CalendarBook::meetings_on(&meetings, now.date());
            alert.tips.extend(composer.meetings_line(&today));
        }
        Ok(Some(alert))
    }

    /// Reminders and follow-up nudges whose time has come; each moves on so it rings once,
    /// all in one transaction so an alert is never shown without its item being moved on.
    pub fn take_due_item_alerts(&self) -> TikiResult<Vec<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(Vec::new());
        }
        let now = self.clock.now();
        let projects = ProjectBook::new(self.database, self.settings).directory()?;
        let composer = NotificationComposer::new(self.translator);
        let policy = FollowUpPolicy::new(self.settings);
        self.database.transaction(|repositories| {
            let mut alerts = Vec::new();
            for mut task in repositories.tasks().list_due_alerts(now)? {
                let project_name = projects.name_of(task.project_id);
                let mut events = Vec::new();
                if task.remind_at.is_some_and(|at| at <= now) {
                    alerts.push(composer.compose_reminder(&task, project_name, now.date()));
                    events.extend(task.ring(now));
                }
                if task.follow_up_at.is_some_and(|at| at <= now) {
                    alerts.push(composer.compose_follow_up(&task, project_name, now.date()));
                    events.push(task.ring_follow_up(now, &policy));
                }
                let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
                repositories.tasks().update(&task, now)?;
                repositories.activity().record(task_id, now, &events)?;
            }
            Ok(alerts)
        })
    }

    /// "Meeting in 15 min" alerts for meetings about to start; each is announced once.
    pub fn take_due_meeting_alerts(&self) -> TikiResult<Vec<TikiAlert>> {
        if !self.preferences.allows_notifications() {
            return Ok(Vec::new());
        }
        let now = self.clock.now();
        let composer = NotificationComposer::new(self.translator);
        Ok(CalendarBook::new(self.database, self.settings)
            .take_starting_soon(now)?
            .iter()
            .map(|meeting| composer.compose_meeting(meeting, now))
            .collect())
    }

    /// A brief moved to a time still ahead today arrives today, even if it already did.
    pub fn reschedule_moved_briefs(&self, changed: &UserPreferences) -> TikiResult<()> {
        let now = self.clock.now().time();
        let moves = [
            (
                BriefMoment::Morning,
                self.preferences.morning_brief_time,
                changed.morning_brief_time,
            ),
            (
                BriefMoment::Evening,
                self.preferences.evening_brief_time,
                changed.evening_brief_time,
            ),
        ];
        for (moment, old_time, new_time) in moves {
            if new_time != old_time && new_time > now {
                self.database.brief_log().clear(moment)?;
            }
        }
        Ok(())
    }
}
