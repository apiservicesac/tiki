use chrono::NaiveDateTime;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::calendar::{AccountId, CalendarAccount, CalendarProvider, Meeting, MeetingItem};
use crate::error::{TikiError, TikiResult};

/// Reads and writes connected calendar accounts and their local copy of meetings.
pub struct CalendarRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> CalendarRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn add_account(&self, provider: CalendarProvider, email: &str) -> TikiResult<AccountId> {
        let existing: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM calendar_accounts WHERE provider = ?1 AND email = ?2 COLLATE NOCASE",
                params![provider.key(), email],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            return Ok(AccountId(id));
        }
        self.connection.execute(
            "INSERT INTO calendar_accounts (provider, email) VALUES (?1, ?2)",
            params![provider.key(), email],
        )?;
        Ok(AccountId(self.connection.last_insert_rowid()))
    }

    pub fn accounts(&self) -> TikiResult<Vec<CalendarAccount>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, provider, email FROM calendar_accounts ORDER BY id")?;
        let accounts = statement
            .query_map([], Self::read_account)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    pub fn find_account(&self, account_id: AccountId) -> TikiResult<CalendarAccount> {
        self.connection
            .query_row(
                "SELECT id, provider, email FROM calendar_accounts WHERE id = ?1",
                params![account_id.0],
                Self::read_account,
            )
            .optional()?
            .ok_or(TikiError::NotFound {
                entity: "calendar account",
                id: account_id.0,
            })
    }

    /// Removes an account and its meetings.
    pub fn remove_account(&self, account_id: AccountId) -> TikiResult<()> {
        self.connection.execute(
            "DELETE FROM calendar_accounts WHERE id = ?1",
            params![account_id.0],
        )?;
        Ok(())
    }

    /// Replaces the account's meetings between `from` and `to` with `meetings`, keeping
    /// whether each one was already announced. Meetings before `from` are past and dropped.
    /// Run it inside `Database::transaction`, so a sync is saved whole or not at all.
    pub fn replace_meetings(
        &self,
        account_id: AccountId,
        from: NaiveDateTime,
        to: NaiveDateTime,
        meetings: &[Meeting],
    ) -> TikiResult<()> {
        for meeting in meetings {
            self.connection.execute(
                "INSERT INTO meetings (account_id, external_id, title, starts_at, ends_at, all_day, location, join_url)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(account_id, external_id) DO UPDATE SET
                    title = excluded.title, location = excluded.location, join_url = excluded.join_url,
                    all_day = excluded.all_day, ends_at = excluded.ends_at,
                    alerted = CASE WHEN meetings.starts_at = excluded.starts_at THEN meetings.alerted ELSE 0 END,
                    starts_at = excluded.starts_at",
                params![
                    account_id.0,
                    meeting.external_id,
                    meeting.title,
                    meeting.starts_at,
                    meeting.ends_at,
                    meeting.all_day,
                    meeting.location,
                    meeting.join_url,
                ],
            )?;
        }
        let kept: Vec<String> = meetings
            .iter()
            .map(|meeting| meeting.external_id.clone())
            .collect();
        let kept_json = serde_json::to_string(&kept).unwrap_or_else(|_| "[]".to_owned());
        self.connection.execute(
            "DELETE FROM meetings WHERE account_id = ?1 AND starts_at >= ?2 AND starts_at < ?3
                AND external_id NOT IN (SELECT value FROM json_each(?4))",
            params![account_id.0, from, to, kept_json],
        )?;
        self.connection.execute(
            "DELETE FROM meetings WHERE account_id = ?1 AND starts_at < ?2",
            params![account_id.0, from],
        )?;
        Ok(())
    }

    /// Meetings that start between `from` and `to`, earliest first.
    pub fn meetings_between(
        &self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> TikiResult<Vec<MeetingItem>> {
        let mut statement = self.connection.prepare(
            "SELECT m.title, m.starts_at, m.ends_at, m.all_day, m.location, m.join_url, a.email
               FROM meetings m JOIN calendar_accounts a ON a.id = m.account_id
              WHERE m.ends_at > ?1 AND m.starts_at < ?2
              ORDER BY m.all_day DESC, m.starts_at, m.title",
        )?;
        let meetings = statement
            .query_map(params![from, to], |row| {
                Ok(MeetingItem {
                    title: row.get(0)?,
                    starts_at: row.get(1)?,
                    ends_at: row.get(2)?,
                    all_day: row.get(3)?,
                    location: row.get(4)?,
                    join_url: row.get(5)?,
                    account_email: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(meetings)
    }

    /// Timed meetings starting after `now` and up to `until` that were not announced yet;
    /// they are marked as announced.
    pub fn take_starting(
        &self,
        now: NaiveDateTime,
        until: NaiveDateTime,
    ) -> TikiResult<Vec<MeetingItem>> {
        let mut statement = self.connection.prepare(
            "SELECT m.id, m.title, m.starts_at, m.ends_at, m.all_day, m.location, m.join_url, a.email
               FROM meetings m JOIN calendar_accounts a ON a.id = m.account_id
              WHERE m.all_day = 0 AND m.alerted = 0 AND m.starts_at > ?1 AND m.starts_at <= ?2
              ORDER BY m.starts_at",
        )?;
        let rows = statement
            .query_map(params![now, until], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    MeetingItem {
                        title: row.get(1)?,
                        starts_at: row.get(2)?,
                        ends_at: row.get(3)?,
                        all_day: row.get(4)?,
                        location: row.get(5)?,
                        join_url: row.get(6)?,
                        account_email: row.get(7)?,
                    },
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (id, _) in &rows {
            self.connection
                .execute("UPDATE meetings SET alerted = 1 WHERE id = ?1", params![id])?;
        }
        Ok(rows.into_iter().map(|(_, meeting)| meeting).collect())
    }

    fn read_account(row: &Row<'_>) -> rusqlite::Result<CalendarAccount> {
        let provider: String = row.get(1)?;
        Ok(CalendarAccount {
            account_id: AccountId(row.get(0)?),
            provider: CalendarProvider::from_key(&provider).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
            email: row.get(2)?,
        })
    }
}
