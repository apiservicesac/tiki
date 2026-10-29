//! The Agenda: everything that happens at a time, by day. Calendar meetings, standalone
//! reminders and the alarms the user set on tasks share one list ordered by time.

mod agenda_builder;
mod agenda_day;
mod agenda_entry;

pub use agenda_builder::AgendaBuilder;
pub use agenda_day::AgendaDay;
pub use agenda_entry::AgendaEntry;
