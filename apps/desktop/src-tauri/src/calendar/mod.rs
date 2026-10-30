//! The shell's side of the calendars: signing in with the provider in the browser (OAuth
//! with PKCE, the user never types a password in Tiki), keeping the sign-in in the
//! operating system's credential store, and copying the meetings of the coming days into
//! the core every few minutes. Reading only: Tiki never changes a calendar.

mod calendar_sync;
mod google_calendar;
mod loopback_receiver;
mod microsoft_calendar;
mod oauth_client;
mod pending_sign_in;
mod pkce;
mod provider_answer;
mod sign_in;
mod sign_in_page;
mod token_store;

pub use calendar_sync::CalendarSync;
pub use pending_sign_in::PendingSignIn;
