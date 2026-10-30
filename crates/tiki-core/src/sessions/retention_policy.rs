use chrono::{Days, NaiveDateTime};

/// How long a session's media is kept; its transcript and notes stay for good.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub media_days: u64,
}

impl RetentionPolicy {
    /// Sessions that started before this moment lose their media at `now`.
    pub fn media_cutoff(self, now: NaiveDateTime) -> NaiveDateTime {
        now.checked_sub_days(Days::new(self.media_days))
            .unwrap_or(NaiveDateTime::MIN)
    }
}
