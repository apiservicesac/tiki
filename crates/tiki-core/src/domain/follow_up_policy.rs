use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

use crate::settings::TikiSettings;

/// When an item waiting on others asks the user to follow up. The one place that knows the
/// threshold, so the brief and the automatic nudge always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowUpPolicy {
    after_days: i64,
    time: NaiveTime,
}

impl FollowUpPolicy {
    pub fn new(settings: &TikiSettings) -> Self {
        Self {
            after_days: settings.follow_up_after_days,
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
}
