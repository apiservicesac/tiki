use chrono::{NaiveDate, NaiveDateTime};

use crate::briefing::BriefMoment;
use crate::preferences::UserPreferences;

/// Decides which brief is due. Each one arrives once a day: the morning brief between its
/// time and the evening close, and the close from its time until midnight. A computer that
/// was off at 08:30 still gets the morning brief when it starts later that morning.
pub struct BriefPlanner<'preferences> {
    preferences: &'preferences UserPreferences,
}

impl<'preferences> BriefPlanner<'preferences> {
    pub fn new(preferences: &'preferences UserPreferences) -> Self {
        Self { preferences }
    }

    /// The brief to send now, given the day each one was last sent.
    pub fn due_moment(
        &self,
        now: NaiveDateTime,
        last_sent: impl Fn(BriefMoment) -> Option<NaiveDate>,
    ) -> Option<BriefMoment> {
        let today = now.date();
        let time = now.time();
        let not_sent_today = |moment| last_sent(moment) != Some(today);

        if time >= self.preferences.evening_brief_time {
            return not_sent_today(BriefMoment::Evening).then_some(BriefMoment::Evening);
        }
        (time >= self.preferences.morning_brief_time && not_sent_today(BriefMoment::Morning))
            .then_some(BriefMoment::Morning)
    }
}
