use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};

use crate::briefing::BriefMoment;
use crate::error::TikiResult;

/// Remembers the day each brief was last sent, so it arrives once a day. The table keeps its
/// first name, `reminder_log`.
pub struct BriefLogRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> BriefLogRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn last_sent(&self, moment: BriefMoment) -> TikiResult<Option<NaiveDate>> {
        Ok(self
            .connection
            .query_row(
                "SELECT sent_on FROM reminder_log WHERE moment = ?1",
                params![moment.key()],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Forgets that a brief was sent, so it arrives again at its (new) time.
    pub fn clear(&self, moment: BriefMoment) -> TikiResult<()> {
        self.connection.execute(
            "DELETE FROM reminder_log WHERE moment = ?1",
            params![moment.key()],
        )?;
        Ok(())
    }

    pub fn mark_sent(&self, moment: BriefMoment, day: NaiveDate) -> TikiResult<()> {
        self.connection.execute(
            "INSERT INTO reminder_log (moment, sent_on) VALUES (?1, ?2)
             ON CONFLICT(moment) DO UPDATE SET sent_on = excluded.sent_on",
            params![moment.key(), day],
        )?;
        Ok(())
    }
}
