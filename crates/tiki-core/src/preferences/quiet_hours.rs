use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

/// The stretch of the day, usually the night, when Tiki sleeps and alerts arrive without
/// sound. The range may cross midnight (22:00–07:00): it starts at `from`, ends just before
/// `until`, and is empty when both are the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuietHours {
    pub enabled: bool,
    pub from: NaiveTime,
    pub until: NaiveTime,
}

impl QuietHours {
    /// Whether `time` falls in the quiet hours, when they are on.
    pub fn covers(&self, time: NaiveTime) -> bool {
        if !self.enabled {
            return false;
        }
        if self.from <= self.until {
            (self.from..self.until).contains(&time)
        } else {
            time >= self.from || time < self.until
        }
    }
}
