use chrono::{Days, NaiveDate, NaiveDateTime};

use crate::settings::TikiSettings;

/// How long an item stays in the trash before Tiki deletes it forever. The one place that
/// counts the days, so the trash's labels and its cleanup always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrashPolicy {
    retention: Days,
}

impl TrashPolicy {
    pub fn new(settings: &TikiSettings) -> Self {
        Self {
            retention: Days::new(settings.trash_retention_days),
        }
    }

    /// The day an item sent to the trash at `trashed_at` is deleted forever.
    pub fn deletes_on(&self, trashed_at: NaiveDateTime) -> NaiveDate {
        trashed_at.date() + self.retention
    }

    /// On `today`, items trashed on this day or earlier are deleted forever.
    pub fn expired_through(&self, today: NaiveDate) -> NaiveDate {
        today - self.retention
    }
}
