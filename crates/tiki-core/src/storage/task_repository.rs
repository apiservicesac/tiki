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
            "INSERT INTO tasks (title, notes, project_id, status, priority, due_date, waiting_since,
                                completed_on, remind_at,
                                origin_plugin, origin_connection, origin_record, origin_url,
                                kind, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                task.title,
                task.notes,
                task.project_id.map(|project| project.0),
                task.status.key(),
                task.priority.key(),
                task.due_date,
                task.waiting_since,
                task.completed_on,
                task.remind_at,
                plugin,
                connection_id,
                record,
                url,
                task.kind.key(),
                TaskRowMapper::tags_column(&task.tags),
            ],
        )?;
        Ok(TaskId(self.connection.last_insert_rowid()))
    }

    pub fn update(&self, task: &Task) -> TikiResult<()> {
        let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
        let changed_rows = self.connection.execute(
            "UPDATE tasks
                SET title = ?2, notes = ?3, project_id = ?4, status = ?5, priority = ?6,
                    due_date = ?7, waiting_since = ?8, completed_on = ?9, remind_at = ?10,
                    kind = ?11, tags = ?12, updated_at = CURRENT_TIMESTAMP
              WHERE id = ?1",
            params![
                task_id.0,
                task.title,
                task.notes,
                task.project_id.map(|project| project.0),
                task.status.key(),
                task.priority.key(),
                task.due_date,
                task.waiting_since,
                task.completed_on,
                task.remind_at,
                task.kind.key(),
                TaskRowMapper::tags_column(&task.tags),
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

    /// Delivered tasks, most recent first, at most `limit` of them.
    pub fn list_delivered(&self, limit: usize) -> TikiResult<Vec<Task>> {
        let query = format!(
            "SELECT {} FROM tasks WHERE status = 'done'
              ORDER BY completed_on DESC, updated_at DESC, id DESC LIMIT ?1",
            TaskRowMapper::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let tasks = statement
            .query_map(
                params![i64::try_from(limit).unwrap_or(i64::MAX)],
                TaskRowMapper::read_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    /// Open tasks whose reminder time has come.
    pub fn list_reminders_due(&self, now: chrono::NaiveDateTime) -> TikiResult<Vec<Task>> {
        let query = format!(
            "SELECT {} FROM tasks WHERE status <> 'done' AND remind_at IS NOT NULL
                AND remind_at <= ?1 ORDER BY remind_at, id",
            TaskRowMapper::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let tasks = statement
            .query_map(params![now], TaskRowMapper::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    /// How many tasks were delivered on `day`.
    pub fn count_delivered_on(&self, day: chrono::NaiveDate) -> TikiResult<usize> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'done' AND completed_on = ?1",
            params![day],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or_default())
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
