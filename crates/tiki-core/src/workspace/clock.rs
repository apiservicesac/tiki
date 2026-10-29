use std::sync::{Arc, Mutex, PoisonError};

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

/// A clock moved by hand, for tests and previews where time has to pass.
pub struct ManualClock {
    moment: Mutex<NaiveDateTime>,
}

impl ManualClock {
    pub fn at(moment: NaiveDateTime) -> Self {
        Self {
            moment: Mutex::new(moment),
        }
    }

    /// Moves the clock to `moment`.
    pub fn set(&self, moment: NaiveDateTime) {
        *self.moment.lock().unwrap_or_else(PoisonError::into_inner) = moment;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> NaiveDateTime {
        *self.moment.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A shared clock, so the owner of a workspace can keep moving it.
impl<SharedClock: Clock + ?Sized> Clock for Arc<SharedClock> {
    fn now(&self) -> NaiveDateTime {
        self.as_ref().now()
    }
}
