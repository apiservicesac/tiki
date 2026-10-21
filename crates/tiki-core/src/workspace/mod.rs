//! The entry point clients use: one object that answers "what does the island show now?"
//! and applies the user's actions, so every platform shares the same behavior.

mod clock;
mod island_snapshot;
mod snapshot_presenter;
mod tiki_workspace;

pub use clock::{Clock, FixedClock, SystemClock};
pub use island_snapshot::{IslandSnapshot, SnapshotItem, SnapshotSection, Urgency};
pub use snapshot_presenter::SnapshotPresenter;
pub use tiki_workspace::TikiWorkspace;
