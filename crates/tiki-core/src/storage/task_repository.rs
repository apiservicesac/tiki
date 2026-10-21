use rusqlite::{Connection, OptionalExtension, params};

use super::task_row_mapper::TaskRowMapper;
use crate::domain::{Task, TaskId};
use crate::error::{TikiError, TikiResult};

/// Reads and writes tasks.
pub struct TaskRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> TaskRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn insert(&self, task: &Task) -> TikiResult<TaskId> {
        let (plugin, connection_id, record, url) = TaskRowMapper::origin_columns(&task.origin);
        self.connection.execute(
            "INSERT INTO tasks (title, notes, client_id, status, priority, due_date, waiting_since,
                                origin_plugin, origin_connection, origin_record, origin_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task.title,
                task.notes,
                task.client_id.map(|client| client.0),
                task.status.key(),
                task.priority.key(),
                task.due_date,
                task.waiting_since,
                plugin,
                connection_id,
                record,
                url,
            ],
        )?;
        Ok(TaskId(self.connection.last_insert_rowid()))
    }

    pub fn update(&self, task: &Task) -> TikiResult<()> {
        let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
        let changed_rows = self.connection.execute(
            "UPDATE tasks
                SET title = ?2, notes = ?3, client_id = ?4, status = ?5, priority = ?6,
                    due_date = ?7, waiting_since = ?8, updated_at = CURRENT_TIMESTAMP
              WHERE id = ?1",
            params![
                task_id.0,
                task.title,
                task.notes,
                task.client_id.map(|client| client.0),
                task.status.key(),
                task.priority.key(),
                task.due_date,
                task.waiting_since,
            ],
        )?;
        if changed_rows == 0 {
            return Err(TikiError::NotFound {
                entity: "task",
                id: task_id.0,
            });
        }
        Ok(())
    }

    pub fn find(&self, task_id: TaskId) -> TikiResult<Task> {
        let query = format!("SELECT {} FROM tasks WHERE id = ?1", TaskRowMapper::COLUMNS);
        self.connection
            .query_row(&query, params![task_id.0], TaskRowMapper::read_row)
            .optional()?
            .ok_or(TikiError::NotFound {
                entity: "task",
                id: task_id.0,
            })
    }

    /// Every task that is not delivered yet, earliest due date first.
    pub fn list_open(&self) -> TikiResult<Vec<Task>> {
        let query = format!(
            "SELECT {} FROM tasks WHERE status <> 'done'
              ORDER BY due_date IS NULL, due_date, id",
            TaskRowMapper::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let tasks = statement
            .query_map([], TaskRowMapper::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }
}
