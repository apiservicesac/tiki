//! The user's calendars: connected accounts and the meetings Tiki keeps a local copy of, so
//! the Agenda, the "meeting in 15 min" alert and "what now" work offline. The desktop shell
//! talks to the providers; the core never knows their APIs.

mod account_identity;
mod calendar_account;
mod calendar_book;
mod calendar_provider;
mod meeting;
mod meeting_item;
mod meeting_link;
mod meeting_platform;

pub use account_identity::AccountIdentity;
pub use calendar_account::{AccountId, CalendarAccount};
pub use calendar_book::CalendarBook;
pub use calendar_provider::CalendarProvider;
pub use meeting::Meeting;
pub use meeting_item::MeetingItem;
pub use meeting_link::MeetingLink;
pub use meeting_platform::MeetingPlatform;
