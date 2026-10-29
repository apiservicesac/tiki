use chrono::{NaiveDate, NaiveDateTime};

use super::{FollowUpPolicy, ItemKind, Task, TaskStatus};
use crate::activity::ItemEvent;

/// How an item moves through its life: status changes, ringing and snoozing.
impl Task {
    /// Moves the task to a new status, keeping track of when it started waiting and when it
    /// was delivered (`completed_on` is the current delivery; the history keeps every one).
    /// Returns the event to record, if the status changed.
    pub fn move_to(&mut self, status: TaskStatus, today: NaiveDate) -> Option<ItemEvent> {
        let event = ItemEvent::status_change(self.status, status);
        if status == TaskStatus::Waiting && self.status != TaskStatus::Waiting {
            self.waiting_since = Some(today);
        } else if status != TaskStatus::Waiting {
            self.waiting_since = None;
        }
        if status == TaskStatus::Done && self.status != TaskStatus::Done {
            self.completed_on = Some(today);
        } else if status != TaskStatus::Done {
            self.completed_on = None;
        }
        self.status = status;
        event
    }

    /// Rings the item's reminder at `now`. A repeating reminder moves to its next
    /// occurrence, a one-time reminder is done once it rang, and the alarm the user set on a
    /// task is cleared so it rings once. Returns the events to record.
    pub fn ring(&mut self, now: NaiveDateTime) -> Vec<ItemEvent> {
        let mut events = vec![ItemEvent::Rang];
        let rang_at = self.remind_at.take();
        if self.kind == ItemKind::Reminder {
            self.remind_at = rang_at.and_then(|at| self.repeat.next_after(at, now));
            if self.remind_at.is_none() {
                events.extend(self.move_to(TaskStatus::Done, now.date()));
            }
        }
        events
    }

    /// Rings the follow-up nudge at `now`; the next one comes after the policy's rhythm, for
    /// as long as the item waits.
    pub fn ring_follow_up(&mut self, now: NaiveDateTime, policy: &FollowUpPolicy) -> ItemEvent {
        self.follow_up_at = policy.next_nudge(now);
        ItemEvent::Rang
    }

    /// Puts the reminder off until `until`; a reminder that already rang comes back to life
    /// (its delivery stays in the history). Returns the events to record.
    pub fn snooze(&mut self, until: NaiveDateTime) -> Vec<ItemEvent> {
        self.remind_at = Some(until);
        let mut events = vec![ItemEvent::Snoozed { until }];
        if self.status == TaskStatus::Done {
            events.extend(self.move_to(TaskStatus::ToDo, until.date()));
        }
        events
    }
}
