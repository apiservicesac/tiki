use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

use super::{AccountId, CalendarAccount, CalendarProvider, Meeting, MeetingItem};
use crate::error::TikiResult;
use crate::settings::TikiSettings;
use crate::storage::Database;

/// The calendar side of the workspace: accounts, the local copy of meetings, what the
/// Agenda shows and which meetings start soon.
pub struct CalendarBook<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
}

impl<'context> CalendarBook<'context> {
    pub fn new(database: &'context Database, settings: &'context TikiSettings) -> Self {
        Self { database, settings }
    }

    pub fn accounts(&self) -> TikiResult<Vec<CalendarAccount>> {
        self.database.calendar().accounts()
    }

    /// Remembers an account the user signed in to; signing in again reuses it.
    pub fn connect(&self, provider: CalendarProvider, email: &str) -> TikiResult<AccountId> {
        self.database.calendar().add_account(provider, email.trim())
    }

    pub fn disconnect(&self, account_id: AccountId) -> TikiResult<()> {
        self.database.calendar().remove_account(account_id)
    }

    pub fn find(&self, account_id: AccountId) -> TikiResult<CalendarAccount> {
        self.database.calendar().find_account(account_id)
    }

    /// The days the Agenda and the local copy of meetings cover: from the start of `today`,
    /// for the next `agenda_days`.
    pub fn window(&self, today: NaiveDate) -> (NaiveDateTime, NaiveDateTime) {
        let from = today.and_time(NaiveTime::MIN);
        let to = from
            .checked_add_days(Days::new(self.settings.agenda_days))
            .unwrap_or(from);
        (from, to)
    }

    /// Replaces what Tiki knows of the account's meetings in the window that starts `today`.
    pub fn store(
        &self,
        account_id: AccountId,
        today: NaiveDate,
        meetings: &[Meeting],
    ) -> TikiResult<()> {
        let (from, to) = self.window(today);
        self.database
            .calendar()
            .replace_meetings(account_id, from, to, meetings)
    }

    /// The meetings of the Agenda: the window that starts `today`.
    pub fn agenda(&self, today: NaiveDate) -> TikiResult<Vec<MeetingItem>> {
        let (from, to) = self.window(today);
        self.database.calendar().meetings_between(from, to)
    }

    /// The meetings that start on `day`.
    pub fn meetings_on(meetings: &[MeetingItem], day: NaiveDate) -> Vec<&MeetingItem> {
        meetings
            .iter()
            .filter(|meeting| meeting.starts_at.date() == day)
            .collect()
    }

    /// Timed meetings starting soon that were not announced yet; announced from now on.
    pub fn take_starting_soon(&self, now: NaiveDateTime) -> TikiResult<Vec<MeetingItem>> {
        let until = now + chrono::Duration::minutes(self.settings.meeting_alert_minutes);
        self.database.calendar().take_starting(now, until)
    }

    /// The next timed meeting today, if any.
    pub fn next_today(meetings: &[MeetingItem], now: NaiveDateTime) -> Option<&MeetingItem> {
        meetings
            .iter()
            .filter(|meeting| {
                !meeting.all_day
                    && meeting.starts_at > now
                    && meeting.starts_at.date() == now.date()
            })
            .min_by_key(|meeting| meeting.starts_at)
    }
}
