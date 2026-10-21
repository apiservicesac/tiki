use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};

/// Tells the workspace what day and time it is, so tests can freeze time.
pub trait Clock: Send + Sync {
    fn now(&self) -> NaiveDateTime;

    fn today(&self) -> NaiveDate {
        self.now().date()
    }
}

/// The computer's local date and time.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> NaiveDateTime {
        Local::now().naive_local()
    }
}

/// Always returns the same moment.
pub struct FixedClock {
    moment: NaiveDateTime,
}

impl FixedClock {
    /// The start of `day`.
    pub fn new(day: NaiveDate) -> Self {
        Self::at(day.and_time(NaiveTime::MIN))
    }

    pub fn at(moment: NaiveDateTime) -> Self {
        Self { moment }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> NaiveDateTime {
        self.moment
    }
}
