use super::{ItemKind, Repeat, Tags, Task, TaskStatus};
use crate::error::{TikiError, TikiResult};

/// The rules every saved item follows, whatever path changed it (capture, edit, a move).
impl Task {
    /// Brings the item in line with the rules, or tells which one it breaks:
    /// - it has a title, and its text fields carry no stray spaces;
    /// - an open reminder has a time to ring (without one it would show nowhere);
    /// - only a reminder with a time repeats;
    /// - only a waiting item says what it waits for and has a follow-up nudge.
    pub fn normalize(&mut self) -> TikiResult<()> {
        let title = self.title.trim();
        if title.is_empty() {
            return Err(TikiError::EmptyTaskTitle);
        }
        self.title = title.to_owned();
        self.notes = self.notes.trim().to_owned();
        self.tags = Tags::normalize(&self.tags);
        if self.kind == ItemKind::Reminder && self.status.is_open() && self.remind_at.is_none() {
            return Err(TikiError::ReminderNeedsTime);
        }
        if self.kind != ItemKind::Reminder || self.remind_at.is_none() {
            self.repeat = Repeat::Never;
        }
        if self.status == TaskStatus::Waiting {
            self.waiting_for = self.waiting_for.trim().to_owned();
        } else {
            self.waiting_for.clear();
            self.follow_up_at = None;
        }
        Ok(())
    }
}
