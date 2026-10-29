use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{Connection, Row, params};
use serde_json::Value;

use crate::activity::{DeliveryRecord, ItemEvent, ItemEventRecord};
use crate::domain::{ProjectId, TaskId};
use crate::error::TikiResult;

/// Reads and writes the activity history, one row per event in `item_events`. The `kind`
/// column holds the event's key and `payload` its other fields as JSON.
pub struct ActivityRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ActivityRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    /// Records what happened to an item at `at`. Call it in the same `Database::transaction`
    /// as the change, so the history never disagrees with the item.
    pub fn record(
        &self,
        item_id: TaskId,
        at: NaiveDateTime,
        events: &[ItemEvent],
    ) -> TikiResult<()> {
        for event in events {
            let mut payload = serde_json::to_value(event)?;
            if let Value::Object(fields) = &mut payload {
                fields.remove("kind");
            }
            self.connection.execute(
                "INSERT INTO item_events (item_id, at, kind, payload) VALUES (?1, ?2, ?3, ?4)",
                params![item_id.0, at, event.key(), payload.to_string()],
            )?;
        }
        Ok(())
    }

    /// Everything that happened to an item, oldest first.
    pub fn events_of(&self, item_id: TaskId) -> TikiResult<Vec<ItemEventRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT item_id, at, kind, payload FROM item_events WHERE item_id = ?1 ORDER BY at, id",
        )?;
        let rows = statement
            .query_map(params![item_id.0], Self::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().map(Self::into_record).collect()
    }

    /// The latest delivery of each delivered item outside the trash, most recent first, at
    /// most `limit`.
    pub fn latest_deliveries(&self, limit: usize) -> TikiResult<Vec<DeliveryRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT e.item_id, MAX(e.at) AS delivered_at
               FROM item_events e JOIN tasks t ON t.id = e.item_id
              WHERE e.kind = ?1 AND t.trashed_at IS NULL
              GROUP BY e.item_id ORDER BY delivered_at DESC, MAX(e.id) DESC LIMIT ?2",
        )?;
        let deliveries = statement
            .query_map(
                params![
                    ItemEvent::Delivered.key(),
                    i64::try_from(limit).unwrap_or(i64::MAX)
                ],
                Self::read_delivery,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(deliveries)
    }

    /// Every delivery from `from` to `to` (both days included) of items outside the trash,
    /// oldest first; only the project's when `project_id` is given.
    pub fn deliveries_between(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        project_id: Option<ProjectId>,
    ) -> TikiResult<Vec<DeliveryRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT e.item_id, e.at FROM item_events e JOIN tasks t ON t.id = e.item_id
              WHERE e.kind = ?1 AND date(e.at) BETWEEN ?2 AND ?3 AND t.trashed_at IS NULL
                AND (?4 IS NULL OR t.project_id = ?4)
              ORDER BY e.at, e.id",
        )?;
        let deliveries = statement
            .query_map(
                params![
                    ItemEvent::Delivered.key(),
                    from,
                    to,
                    project_id.map(|project| project.0)
                ],
                Self::read_delivery,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(deliveries)
    }

    /// How many different items outside the trash were delivered on `day`.
    pub fn count_delivered_on(&self, day: NaiveDate) -> TikiResult<usize> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(DISTINCT e.item_id) FROM item_events e JOIN tasks t ON t.id = e.item_id
              WHERE e.kind = ?1 AND date(e.at) = ?2 AND t.trashed_at IS NULL",
            params![ItemEvent::Delivered.key(), day],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or_default())
    }

    fn read_row(row: &Row<'_>) -> rusqlite::Result<(i64, NaiveDateTime, String, String)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }

    fn into_record(
        (item_id, at, kind, payload): (i64, NaiveDateTime, String, String),
    ) -> TikiResult<ItemEventRecord> {
        let mut event: Value = serde_json::from_str(&payload)?;
        if let Value::Object(fields) = &mut event {
            fields.insert("kind".to_owned(), Value::String(kind));
        }
        Ok(ItemEventRecord {
            item_id: TaskId(item_id),
            at,
            event: serde_json::from_value(event)?,
        })
    }

    fn read_delivery(row: &Row<'_>) -> rusqlite::Result<DeliveryRecord> {
        Ok(DeliveryRecord {
            item_id: TaskId(row.get(0)?),
            at: row.get(1)?,
        })
    }
}
