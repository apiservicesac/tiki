use chrono::NaiveDate;

use crate::domain::{TaskId, TrashPolicy};
use crate::error::TikiResult;
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Deletes items in the trash forever: one at a time, all at once, or the ones past their
/// retention (see `TrashPolicy`). An item deleted forever takes its whole history with it,
/// deliveries included, so no event ever points to an item that no longer exists.
pub struct TrashBin<'context> {
    database: &'context Database,
    policy: TrashPolicy,
}

impl<'context> TrashBin<'context> {
    pub fn new(database: &'context Database, settings: &TikiSettings) -> Self {
        Self {
            database,
            policy: TrashPolicy::new(settings),
        }
    }

    /// Deletes one item in the trash forever; an item outside it is not found.
    pub fn delete_forever(&self, task_id: TaskId) -> TikiResult<()> {
        self.database.tasks().delete_trashed(task_id)
    }

    /// Deletes every item in the trash forever.
    pub fn empty(&self) -> TikiResult<()> {
        self.database.tasks().delete_all_trashed()
    }

    /// Deletes forever the items whose time in the trash is over on `today`. Returns how many
    /// went.
    pub fn purge_expired(&self, today: NaiveDate) -> TikiResult<usize> {
        self.database
            .tasks()
            .delete_trashed_through(self.policy.expired_through(today))
    }
}
