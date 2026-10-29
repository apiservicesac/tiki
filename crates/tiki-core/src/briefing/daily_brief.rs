use chrono::NaiveDate;
use serde::Serialize;

use super::{BriefDetail, BriefItem};

/// The user's day, grouped by what needs attention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DailyBrief {
    pub date: NaiveDate,
    pub overdue: Vec<BriefItem>,
    pub due_soon: Vec<BriefItem>,
    pub your_turn: Vec<BriefItem>,
    pub waiting: Vec<BriefItem>,
    /// Waiting items that passed the follow-up threshold.
    pub needs_follow_up: Vec<BriefItem>,
    /// Standalone reminders still to ring, earliest first.
    pub reminders: Vec<BriefItem>,
}

impl DailyBrief {
    /// A brief with every section empty.
    pub fn empty(date: NaiveDate) -> Self {
        Self {
            date,
            overdue: Vec::new(),
            due_soon: Vec::new(),
            your_turn: Vec::new(),
            waiting: Vec::new(),
            needs_follow_up: Vec::new(),
            reminders: Vec::new(),
        }
    }

    pub fn is_all_clear(&self) -> bool {
        self.overdue.is_empty() && self.due_soon.is_empty()
    }

    /// Standalone reminders that ring on `day`, earliest first.
    pub fn reminders_on(&self, day: NaiveDate) -> Vec<BriefItem> {
        self.reminders
            .iter()
            .filter(|item| item.rings_at().is_some_and(|at| at.date() == day))
            .cloned()
            .collect()
    }

    /// Work due the day after the brief's date.
    pub fn due_tomorrow(&self) -> Vec<BriefItem> {
        self.due_soon
            .iter()
            .filter(|item| item.detail == BriefDetail::DueIn { days: 1 })
            .cloned()
            .collect()
    }

    /// Items with date pressure: overdue first, then due soon.
    pub fn pressing_count(&self) -> usize {
        self.overdue.len() + self.due_soon.len()
    }
}
