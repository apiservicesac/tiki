use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

use super::{Task, TaskStatus};
use crate::settings::TikiSettings;

/// When an item waiting on others asks the user to follow up. The one place that knows the
/// threshold and the rhythm, so the brief and the automatic nudges always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowUpPolicy {
    after_days: i64,
    repeat_days: i64,
    time: NaiveTime,
}

impl FollowUpPolicy {
    pub fn new(settings: &TikiSettings) -> Self {
        Self {
            after_days: settings.follow_up_after_days,
            repeat_days: settings.follow_up_repeat_days,
            time: settings.follow_up_time,
        }
    }

    /// Whether an item waiting this many days needs a follow-up.
    pub fn is_due(&self, days_waiting: i64) -> bool {
        days_waiting >= self.after_days
    }

    /// The first nudge for an item waiting since `since`.
    pub fn first_nudge(&self, since: NaiveDate) -> Option<NaiveDateTime> {
        since
            .checked_add_days(Days::new(self.after_days.unsigned_abs()))
            .map(|day| day.and_time(self.time))
    }

    /// The nudge after one that rang at `rang_at`, while the item still waits.
    pub fn next_nudge(&self, rang_at: NaiveDateTime) -> Option<NaiveDateTime> {
        rang_at
            .date()
            .checked_add_days(Days::new(self.repeat_days.unsigned_abs()))
            .map(|day| day.and_time(self.time))
    }

    /// Schedules the first nudge of an item that just started waiting.
    pub fn schedule_first_nudge(&self, task: &mut Task, was_waiting: bool) {
        if task.status == TaskStatus::Waiting && !was_waiting {
            task.follow_up_at = task.waiting_since.and_then(|since| self.first_nudge(since));
        }
    }
}
