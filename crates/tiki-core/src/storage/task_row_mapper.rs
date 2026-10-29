use rusqlite::Row;
use rusqlite::types::Type;

use crate::domain::{
    ExternalReference, ItemKind, Priority, ProjectId, Repeat, Task, TaskId, TaskOrigin, TaskStatus,
};

/// Converts between `tasks` table rows and `Task` values.
pub struct TaskRowMapper;

impl TaskRowMapper {
    /// Columns in the order `read_row` expects them.
    pub const COLUMNS: &'static str = "id, kind, tags, repeat, waiting_for, title, notes, project_id, status, priority, due_date, \
        waiting_since, completed_on, remind_at, follow_up_at, origin_plugin, origin_connection, origin_record, \
        origin_url, created_at, updated_at";

    pub fn read_row(row: &Row<'_>) -> rusqlite::Result<Task> {
        Ok(Task {
            id: Some(TaskId(row.get("id")?)),
            kind: Self::read_kind(row)?,
            tags: Self::read_tags(row)?,
            repeat: Self::read_repeat(row)?,
            waiting_for: row.get("waiting_for")?,
            title: row.get("title")?,
            notes: row.get("notes")?,
            project_id: row.get::<_, Option<i64>>("project_id")?.map(ProjectId),
            status: Self::read_status(row)?,
            priority: Self::read_priority(row)?,
            due_date: row.get("due_date")?,
            waiting_since: row.get("waiting_since")?,
            completed_on: row.get("completed_on")?,
            remind_at: row.get("remind_at")?,
            follow_up_at: row.get("follow_up_at")?,
            origin: Self::read_origin(row)?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
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

    /// Tags are stored as a JSON array of strings.
    pub fn tags_column(tags: &[String]) -> String {
        serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_owned())
    }

    fn read_kind(row: &Row<'_>) -> rusqlite::Result<ItemKind> {
        let key: String = row.get("kind")?;
        ItemKind::from_key(&key).map_err(|error| Self::conversion_error(1, error))
    }

    fn read_repeat(row: &Row<'_>) -> rusqlite::Result<Repeat> {
        let key: String = row.get("repeat")?;
        Repeat::from_key(&key).map_err(|error| Self::conversion_error(3, error))
    }

    fn read_tags(row: &Row<'_>) -> rusqlite::Result<Vec<String>> {
        let json: String = row.get("tags")?;
        serde_json::from_str(&json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(error))
        })
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
