use rusqlite::Row;
use rusqlite::types::Type;

use crate::domain::{ClientId, ExternalReference, Priority, Task, TaskId, TaskOrigin, TaskStatus};

/// Converts between `tasks` table rows and `Task` values.
pub struct TaskRowMapper;

impl TaskRowMapper {
    /// Columns in the order `read_row` expects them.
    pub const COLUMNS: &'static str = "id, title, notes, client_id, status, priority, due_date, \
        waiting_since, completed_on, remind_at, origin_plugin, origin_connection, origin_record, \
        origin_url";

    pub fn read_row(row: &Row<'_>) -> rusqlite::Result<Task> {
        Ok(Task {
            id: Some(TaskId(row.get("id")?)),
            title: row.get("title")?,
            notes: row.get("notes")?,
            client_id: row.get::<_, Option<i64>>("client_id")?.map(ClientId),
            status: Self::read_status(row)?,
            priority: Self::read_priority(row)?,
            due_date: row.get("due_date")?,
            waiting_since: row.get("waiting_since")?,
            completed_on: row.get("completed_on")?,
            remind_at: row.get("remind_at")?,
            origin: Self::read_origin(row)?,
        })
    }

    /// Splits an origin into the four nullable `origin_*` columns.
    pub fn origin_columns(
        origin: &TaskOrigin,
    ) -> (Option<&str>, Option<&str>, Option<&str>, Option<&str>) {
        match origin {
            TaskOrigin::Manual => (None, None, None, None),
            TaskOrigin::External(reference) => (
                Some(reference.plugin_id.as_str()),
                Some(reference.connection_id.as_str()),
                Some(reference.record_id.as_str()),
                reference.url.as_deref(),
            ),
        }
    }

    fn read_status(row: &Row<'_>) -> rusqlite::Result<TaskStatus> {
        let key: String = row.get("status")?;
        TaskStatus::from_key(&key).map_err(|error| Self::conversion_error(4, error))
    }

    fn read_priority(row: &Row<'_>) -> rusqlite::Result<Priority> {
        let key: String = row.get("priority")?;
        Priority::from_key(&key).map_err(|error| Self::conversion_error(5, error))
    }

    fn read_origin(row: &Row<'_>) -> rusqlite::Result<TaskOrigin> {
        let plugin_id: Option<String> = row.get("origin_plugin")?;
        let Some(plugin_id) = plugin_id else {
            return Ok(TaskOrigin::Manual);
        };
        Ok(TaskOrigin::External(ExternalReference {
            plugin_id,
            connection_id: row
                .get::<_, Option<String>>("origin_connection")?
                .unwrap_or_default(),
            record_id: row
                .get::<_, Option<String>>("origin_record")?
                .unwrap_or_default(),
            url: row.get("origin_url")?,
        }))
    }

    fn conversion_error(column: usize, error: crate::TikiError) -> rusqlite::Error {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
    }
}
