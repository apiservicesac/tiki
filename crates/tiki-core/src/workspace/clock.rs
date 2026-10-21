use chrono::{Local, NaiveDate};

/// Tells the workspace what day it is, so tests can freeze time.
pub trait Clock: Send + Sync {
    fn today(&self) -> NaiveDate;
}

/// The computer's local date.
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> NaiveDate {
        Local::now().date_naive()
    }
}

/// Always returns the same day.
pub struct FixedClock {
    day: NaiveDate,
}

impl FixedClock {
    pub fn new(day: NaiveDate) -> Self {
        Self { day }
    }
}

impl Clock for FixedClock {
    fn today(&self) -> NaiveDate {
        self.day
    }
}
