//! The user's items (tasks and reminders): capturing, editing and moving them through their
//! life, each change saved together with its place in the activity history, and deleting
//! them forever from the trash.

mod item_service;
mod trash_bin;

pub use item_service::ItemService;
pub use trash_bin::TrashBin;
