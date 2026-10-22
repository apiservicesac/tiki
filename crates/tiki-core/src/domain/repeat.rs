use chrono::{Datelike, Days, Months, NaiveDateTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// How often a reminder comes back after it rings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    #[default]
    Never,
    Daily,
    /// Monday to Friday.
    Weekdays,
    Weekly,
    Monthly,
}

impl Repeat {
    pub const ALL: [Repeat; 5] = [
        Repeat::Never,
        Repeat::Daily,
        Repeat::Weekdays,
        Repeat::Weekly,
        Repeat::Monthly,
    ];

    /// Stable key used in storage.
    pub fn key(self) -> &'static str {
        match self {
            Repeat::Never => "never",
            Repeat::Daily => "daily",
            Repeat::Weekdays => "weekdays",
            Repeat::Weekly => "weekly",
            Repeat::Monthly => "monthly",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|repeat| repeat.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "repeat",
                value: key.to_owned(),
            })
    }

    /// The first moment after `now` in this rhythm, counted from `at`; `None` for `Never`.
    pub fn next_after(self, at: NaiveDateTime, now: NaiveDateTime) -> Option<NaiveDateTime> {
        if self == Repeat::Never {
            return None;
        }
        let mut next = self.step(at)?;
        while next <= now {
            next = self.step(next)?;
        }
        Some(next)
    }

    fn step(self, at: NaiveDateTime) -> Option<NaiveDateTime> {
        match self {
            Repeat::Never => None,
            Repeat::Daily => at.checked_add_days(Days::new(1)),
            Repeat::Weekdays => {
                let mut next = at.checked_add_days(Days::new(1))?;
                while matches!(next.weekday(), Weekday::Sat | Weekday::Sun) {
                    next = next.checked_add_days(Days::new(1))?;
                }
                Some(next)
            }
            Repeat::Weekly => at.checked_add_days(Days::new(7)),
            Repeat::Monthly => at.checked_add_months(Months::new(1)),
        }
    }
}
