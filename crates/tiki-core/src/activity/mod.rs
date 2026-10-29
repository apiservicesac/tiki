//! The activity history ("registro"): what happened to each item and when. Every change is
//! recorded next to the change itself, so deliveries survive a reopening and reports can
//! read the past.

mod item_event;
mod item_event_record;
mod item_field;

pub use item_event::ItemEvent;
pub use item_event_record::{DeliveryRecord, ItemEventRecord};
pub use item_field::ItemField;
