use chrono::NaiveDateTime;

use super::Task;
use crate::activity::ItemEvent;

/// How an item goes to the trash and comes back. In the trash it keeps every field, so
/// restoring it brings it back exactly as it was.
impl Task {
    /// Sends the item to the trash at `now`. Returns the event to record, or `None` when it
    /// already was there.
    pub fn trash(&mut self, now: NaiveDateTime) -> Option<ItemEvent> {
        if self.is_trashed() {
            return None;
        }
        self.trashed_at = Some(now);
        Some(ItemEvent::Trashed)
    }

    /// Takes the item out of the trash. Returns the event to record, or `None` when it was
    /// not there.
    pub fn restore(&mut self) -> Option<ItemEvent> {
        self.trashed_at.take().map(|_| ItemEvent::Restored)
    }

    pub fn is_trashed(&self) -> bool {
        self.trashed_at.is_some()
    }
}
