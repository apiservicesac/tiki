//! The entry point projects use: one object that answers "what does the island show now?"
//! and applies the user's actions, so every platform shares the same behavior.

mod assistant_context;
mod clock;
mod delivered_item;
mod island_snapshot;
mod snapshot_presenter;
mod tiki_workspace;
mod workspace_views;

pub use assistant_context::{AssistantContext, AssistantProject, AssistantTask};
pub use clock::{Clock, FixedClock, ManualClock, SystemClock};
pub use delivered_item::DeliveredItem;
pub use island_snapshot::{
    FocusSuggestion, IslandSnapshot, SnapshotItem, SnapshotSection, Urgency,
};
pub use snapshot_presenter::{SnapshotPresenter, TodaysMeetings};
pub use tiki_workspace::TikiWorkspace;
pub use workspace_views::WorkspaceViews;
