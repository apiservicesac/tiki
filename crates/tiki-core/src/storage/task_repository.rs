use chrono::NaiveDateTime;
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

    /// Saves a new task created at `now` (the workspace clock's local time).
    pub fn insert(&self, task: &Task, now: NaiveDateTime) -> TikiResult<TaskId> {
        let (plugin, connection_id, record, url) = TaskRowMapper::origin_columns(&task.origin);
        self.connection.execute(
            "INSERT INTO tasks (title, notes, project_id, status, priority, due_date, waiting_since,
                                completed_on, remind_at,
                                origin_plugin, origin_connection, origin_record, origin_url,
                                kind, tags, repeat, waiting_for, follow_up_at,
                                created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                     ?18, ?19, ?19)",
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
                task.repeat.key(),
                task.waiting_for,
                task.follow_up_at,
                now,
            ],
        )?;
        Ok(TaskId(self.connection.last_insert_rowid()))
    }

    /// Saves the changes to a task made at `now` (the workspace clock's local time).
    pub fn update(&self, task: &Task, now: NaiveDateTime) -> TikiResult<()> {
        let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
        let changed_rows = self.connection.execute(
            "UPDATE tasks
                SET title = ?2, notes = ?3, project_id = ?4, status = ?5, priority = ?6,
                    due_date = ?7, waiting_since = ?8, completed_on = ?9, remind_at = ?10,
                    kind = ?11, tags = ?12, repeat = ?13, waiting_for = ?14, follow_up_at = ?15,
                    updated_at = ?16
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
                task.repeat.key(),
                task.waiting_for,
                task.follow_up_at,
                now,
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

    /// Open items whose reminder or follow-up nudge is due at `now`.
    pub fn list_due_alerts(&self, now: chrono::NaiveDateTime) -> TikiResult<Vec<Task>> {
        let query = format!(
            "SELECT {} FROM tasks WHERE status <> 'done' AND (remind_at <= ?1 OR follow_up_at <= ?1)
              ORDER BY min(coalesce(remind_at, follow_up_at), coalesce(follow_up_at, remind_at)), id",
            TaskRowMapper::COLUMNS
        );
        let mut statement = self.connection.prepare(&query)?;
        let tasks = statement
            .query_map(params![now], TaskRowMapper::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
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
